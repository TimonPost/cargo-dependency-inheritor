# Cargo Dependency Inheritance

Utility to inherit dependencies from workspace file if it occurs 'n' or more times throughout the project.

Workspace Inheritance has been stabilized in the 1.64 release.
See [workspace.package][1], [workspace.dependencies][2], and [inheriting-a-dependency-from-a-workspace][3] for more information.


## How to Use

To inherit any dependency that occurs 5 or more times in the workspace use the following command:

```
cargo dependency-inheritance --path "path/to/workspace/Cargo.toml" --occurences 5
```

**This command edits your toml files, make sure to have a back up**

## Process

1. Read packages defined in [workspace] section of workspace file. 
2. Note which dependencies occur 'n' times.
3. Update all dependencies that occured 'n' times by adding 'workspace=true' key-value.
4. Add [workspace.dependencies] table to root workspace file with all the dependencies that occured 'n' times and their version.

[1]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-workspacepackage-table
[2]: https://doc.rust-lang.org/nightly/cargo/reference/workspaces.html#the-workspacedependencies-table
[3]: https://doc.rust-lang.org/nightly/cargo/reference/specifying-dependencies.html#inheriting-a-dependency-from-a-workspace
