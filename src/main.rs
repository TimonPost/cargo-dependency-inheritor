//! # Cargo Workspace Dependency Inheritor
//!
//! Utility that inherits dependencies from the main workspace if they occur `n` or more times in the workspace.
//!
//! Workspace Inheritance was stabilized in version 1.64.
//! See [`[workspace.package]`][1], [`[workspace.dependencies]`][2], and [inheriting-a-dependency-from-a-workspace][3] for more information.
//!
//! ## How to Use
//!
//! To inherit a dependency that occurs five or more times in the workspace, use the following command:
//!
//! (currently some issues using it via cargo subcommand, download this lib and compile it your self instead)
//!
//! ```bash
//! cargo install cargo-dependency-inheritor
//! cargo dependency-inheritor --workspace-path "path/to/workspace/Cargo.toml" -n 5
//! ```
//!
//! **This command edits your toml files, make sure to have a back up**
//!
//! ## Process
//!
//! Dependencies can be inherited from a workspace by specifying the dependency in the workspace's [`[workspace.dependencies]`][2] table. After that, add it to the `[dependencies]` table with workspace = true.
//! This crate automates the process.
//!
//! 1. Read packages defined in `[workspace]` section of the workspace-file.
//! 2. Note which dependencies occur `n` or more times.
//! 3. Update all dependencies that occurred `n` or more times:
//!    1. Turn `dependency = "0.1.3"` into inline tables.
//!    2. Add `workspace = true` key-value to the dependency inline table.
//!    3. Remove `version` from inline table if exists (this will be specified in the workspace file).
//! 4. Add [`[workspace.dependencies]`][2] table to root workspace file with all the dependencies that occurred `n` times and their version.
//!
//! Result:
//!
//! ```toml
//! ## in a project
//! [dependencies]
//! tokio = { workspace = true }
//!
//! ## in the workspace
//! [workspace.dependencies]
//! tokio = "1.0"
//! ```
//!
//! [1]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-package-table
//! [2]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-dependencies-table
//! [3]: https://doc.rust-lang.org/nightly/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace

use std::{
    collections::{HashMap, HashSet},
    path::PathBuf,
};
use toml_edit::{Document, Formatted, InlineTable, Item, Table, Value};

use clap::{AppSettings, Parser};

#[derive(clap::Args)]
#[clap(author, version, about, long_about = None,global_setting(AppSettings::DeriveDisplayOrder) )]
struct DependencyInheritor {
    /// Full path to the `Cargo.toml` file that defines the rust workspace.
    #[clap(short, long, value_parser)]
    workspace_path: PathBuf,

    /// If a dependency is used throughout the workspace more then 'n times', add the 'workspace = true' key value to it.
    #[clap(short, long, value_parser)]
    number: usize,
}

#[derive(Parser)]
#[clap(bin_name = "cargo")]
enum Cargo {
    DependencyInheritor(DependencyInheritor),
}

fn main() {
    let args = Cargo::parse();
    match args {
        Cargo::DependencyInheritor(args) => {
            // Gather metadata on the workspace.
            let mut cmd = cargo_metadata::MetadataCommand::new();
            cmd.manifest_path(args.workspace_path.clone());

            let metadata = cmd.exec().unwrap();

            // Gather all dependencies that occur more then the configured number of times throughout the workspace.
            let mut duplicated_dependencies = HashMap::<&String, Entry>::new();
            let mut workspace_packages = HashMap::<_, HashSet<_>>::new();

            for package in metadata.workspace_packages() {
                for package_dependency in &package.dependencies {
                    let mut detected_dependency = duplicated_dependencies
                        .entry(&package_dependency.name)
                        .or_default();

                    detected_dependency.version = package_dependency.req.to_string();
                    detected_dependency.count += 1;
                    detected_dependency
                        .workspace_packages
                        .push(package.manifest_path.to_string());

                    // Store the package and the dependencies if more then the configured number of dependency occurrences are found.
                    if detected_dependency.count >= args.number {
                        workspace_packages
                            .entry(&package.manifest_path)
                            .or_default()
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
                            Item::Table(_) => {
                                // TODO
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
                                    new_table
                                        .insert("workspace", Value::Boolean(Formatted::new(true)));

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

                if let Err(e) = std::fs::write(package_toml, toml_document.to_string()) {
                    eprintln!("Failed to write to {:?}: {:?}", package_toml, e);
                }
            }

            // Print the results.
            for (d, entry) in &duplicated_dependencies {
                if entry.count >= args.number {
                    println!("==== Dependency: '{d}' ({}) =====", entry.count);

                    for workspace_package in &entry.workspace_packages {
                        println!("  - {workspace_package}");
                    }
                }
            }

            if let Ok(toml_contents) = std::fs::read_to_string(args.workspace_path.clone()) {
                if let Ok(mut doc) = toml_contents.parse::<Document>() {
                    edit_workspace_dependency_table(
                        &mut doc,
                        &duplicated_dependencies,
                        args.number,
                    );

                    if let Err(e) = std::fs::write(&args.workspace_path, doc.to_string()) {
                        eprintln!("Failed to write to {:?}: {:?}", args.workspace_path, e);
                    }
                } else {
                    println!("failed to parse workspace definition");
                };
            } else {
                println!("failed to update workspace definition");
            };
        }
    }
}

fn edit_workspace_dependency_table(
    document: &mut Document,
    workspace_deps: &HashMap<&String, Entry>,
    occurrences: usize,
) {
    // Crate table if not exist, otherwise edit.
    if let Some(Item::Table(table)) = document.get_mut("workspace.dependencies") {
        for (key, val) in workspace_deps {
            if val.count >= occurrences && !table.contains_key(key.as_str()) {
                table.insert(
                    key,
                    Item::Value(Value::String(Formatted::new(val.version.clone()))),
                );
            }
        }
    } else {
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
}

#[derive(Default)]
struct Entry {
    pub count: usize,
    pub workspace_packages: Vec<String>,
    pub version: String,
}
