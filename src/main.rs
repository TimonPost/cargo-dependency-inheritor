use cargo_metadata::camino::Utf8Path;
use std::{
    collections::{HashMap, HashSet},
    ops::Add,
    path::{Path, PathBuf},
    vec,
};
use toml_edit::{Array, Document, Formatted, InlineTable, Item, Table, Value};

use clap::Parser;

/// Utility to inherit dependencies if it occurs 'n' or more times throughout the workspace.
/// This utility will modify the dependencies by appending 'workspace = true' and update the workspace file with the inheritable dependencies.
///
/// Workspace Inheritance has been stabilized in the 1.64 release.
/// See [workspace.package][1], [workspace.dependencies][2], and [inheriting-a-dependency-from-a-workspace][3] for more information.
///
/// [1]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-workspacepackage-table
/// [2]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-workspacedependencies-table
/// [3]: https://doc.rust-lang.org/nightly/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Full path to the `Cargo.toml` file that defines the workspace.
    #[clap(short, long, value_parser)]
    path: PathBuf,

    /// If a dependency is used more then the given number of occurrences, add the 'workspace = true' key value to it.
    #[clap(short, long, value_parser)]
    occurrences: usize,
}

fn main() {
    let args = Args::parse();

    // Gather metadata on the workspace.
    let mut cmd = cargo_metadata::MetadataCommand::new();
    cmd.manifest_path(args.path.clone());

    let metadata = cmd.exec().unwrap();

    // Gather all dependencies that occur more then the configured number of times throughout the workspace.
    let mut duplicated_dependencies = HashMap::new();
    let mut workspace_packages = HashMap::new();

    for package in metadata.workspace_packages() {
        for package_dependency in &package.dependencies {
            let mut detected_dependency = duplicated_dependencies
                .entry(&package_dependency.name)
                .or_insert(Entry::default());

            detected_dependency.version = package_dependency.req.to_string();
            detected_dependency.count += 1;
            detected_dependency
                .workspace_packages
                .push(package.manifest_path.to_string());

            // Store the package and the dependencies if more then the configured number of dependency occurrences are found.
            if detected_dependency.count >= args.occurrences {
                workspace_packages
                    .entry(&package.manifest_path)
                    .or_insert_with(|| HashSet::new())
                    .insert(package_dependency.name.clone());
            }
        }
    }

    // Update the toml definition of the workspace. And add the new 'workspace = true' key value pair.
    for (package_toml, dependency_candidate) in workspace_packages {
        let toml_contents = if let Ok(doc) = std::fs::read_to_string(package_toml) {
            doc
        } else {
            continue;
        };
        let mut toml_document = if let Ok(doc) = toml_contents.parse::<Document>() {
            doc
        } else {
            continue;
        };

        // Fetch the dependency table from the workspace package toml document.
        if let Some(Item::Table(dependency_table)) = toml_document.get_mut("dependencies") {
            // Iterate all packages with deps that ocurred more then the configured number times.
            for (key, val) in dependency_table.iter_mut() {
                if !dependency_candidate.contains(key.get()) {
                    continue;
                }

                match val {
                    Item::None => todo!(),
                    Item::Table(t) => {
                        t.iter().for_each(|(k, v)| {
                            println!("t{:?}", k);
                        });
                    }
                    Item::ArrayOfTables(_) => todo!(),
                    Item::Value(val) => match val {
                        Value::InlineTable(table) => {
                            // dependency specified as `dep = {version="x"}`.

                            table.remove("version");
                            table.insert("workspace", Value::Boolean(Formatted::new(true)));
                        }
                        Value::String(_) => {
                            // dependency specified as `dep = "x"`
                            let mut new_table = InlineTable::new();
                            new_table.insert("workspace", Value::Boolean(Formatted::new(true)));

                            // preserve any line decoration such as comments.
                            let decor = val.decor().clone();
                            *val = Value::InlineTable(new_table);
                            *val.decor_mut() = decor;
                        }
                        Value::Integer(_)
                        | Value::Float(_)
                        | Value::Boolean(_)
                        | Value::Datetime(_)
                        | Value::Array(_) => {
                            // dependency not specified in those forms.
                        }
                    },
                }
            }
        }

        if let Err(_) = std::fs::write(package_toml, toml_document.to_string()) {
            println!("Failed to write: {:?}", package_toml);
        }
    }

    // Print the results.
    for (d, entry) in &duplicated_dependencies {
        if entry.count >= args.occurrences {
            println!("==== Dependency: '{d}' ({}) =====", entry.count);

            for workspace_package in &entry.workspace_packages {
                println!("  - {workspace_package}");
            }
        }
    }

    if let Ok(toml_contents) = std::fs::read_to_string(args.path.clone()) {
        if let Ok(mut doc) = toml_contents.parse::<Document>() {
            create_workspace_dependency_table(&mut doc, &duplicated_dependencies, args.occurrences);

            if let Err(_) = std::fs::write(args.path, doc.to_string()) {
                println!("Failed to write");
            }
        } else {
            println!("failed to parse workspace definition");
        };
    } else {
        println!("failed to update workspace definition");
    };
}

fn create_workspace_dependency_table(
    document: &mut Document,
    workspace_deps: &HashMap<&String, Entry>,
    occurrences: usize,
) {
    let mut new_table = Table::new();

    for (key, val) in workspace_deps {
        if val.count >= occurrences {
            new_table.insert(
                key,
                Item::Value(Value::String(Formatted::new(val.version.clone()))),
            );
        }
    }

    document.insert("workspace.dependencies", Item::Table(new_table));
}

struct Entry {
    pub count: usize,
    pub workspace_packages: Vec<String>,
    pub version: String,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            count: 0,
            workspace_packages: vec![],
            version: String::new(),
        }
    }
}
