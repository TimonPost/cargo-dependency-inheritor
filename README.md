# Cargo Workspace Dependency Inheritor

Utility that inherits dependencies from the main workspace if they occur 'n' or more times in the workspace.

Workspace Inheritance was stabilized in version 1.64.
See [workspace.package][1], [workspace.dependencies][2], and [inheriting-a-dependency-from-a-workspace][3] for more information.

## How to Use

To inherit a dependency that occurs five or more times in the workspace, use the following command:

```
cargo dependency-inheritance --path "path/to/workspace/Cargo.toml" --occurences 5
```

**This command edits your toml files, make sure to have a back up**

## Process

1. Read packages defined in [workspace] section of workspace file. 
2. Note which dependencies occur 'n' times.
3. Update all dependencies that occured 'n' times by adding 'workspace=true' key-value.
4. Add [workspace.dependencies] table to root workspace file with all the dependencies that occured 'n' times and their version.

Rsult:
```
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
