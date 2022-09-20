# Cargo Workspace Dependency Inheritor

Utility that inherits dependencies from the main workspace if they occur 'n' or more times in the workspace.

Workspace Inheritance was stabilized in version 1.64.
See [workspace.package][1], [workspace.dependencies][2], and [inheriting-a-dependency-from-a-workspace][3] for more information.

## How to Use

To inherit a dependency that occurs five or more times in the workspace, use the following command:

```bash
cargo dependency-inheritor --path "path/to/workspace/Cargo.toml" --occurrences 5
```

**This command edits your toml files, make sure to have a back up**

## Process

Dependencies can be inherited from a workspace by specifying the dependency in the workspace's [workspace.dependencies] table. After that, add it to the [dependencies] table with workspace = true.
This crate automates the process.

1. Read packages defined in [workspace] section of the workspace-file.
2. Note which dependencies occur 'n' or more times.
3. Update all dependencies that occurred 'n' or more times:
   1. Turn 'dependency = "0.1.3"' into inline tables.
   2. Add 'workspace=true' key-value to the dependency inline table.
   3. Remove 'version' from inline table if exists (this will be specified in the workspace file).
4. Add [workspace.dependencies] table to root workspace file with all the dependencies that occurred 'n' times and their version.

Result:

```toml
// in a project
[dependencies]
tokio = { workspace = true }

// in the workspace
[workspace.dependencies]
tokio = "1.0"
```

[1]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-workspacepackage-table
[2]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-workspacedependencies-table
[3]: https://doc.rust-lang.org/nightly/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace
