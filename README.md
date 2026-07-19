# project-root-patch

[![CI](https://github.com/milyin/project-root-patch/actions/workflows/ci.yml/badge.svg)](https://github.com/milyin/project-root-patch/actions/workflows/ci.yml)

Expose the consuming Cargo workspace's root to an external crate's `build.rs`.

## External crate

Add the library as a build dependency:

```toml
[build-dependencies]
project-root-patch = "0.1"
```

The external crate can then locate files owned by the consuming workspace:

```rust,no_run
let workspace_root = project_root_patch::get_project_root();
let lockfile = workspace_root.join("Cargo.lock");
```

Without the workspace setup below, `get_project_root()` panics with an
explanatory message. Cargo normally builds `project-root-patch` from its
registry cache, which is outside the consuming workspace and cannot identify
that workspace.

One use case is a build script that runs a nested model build to inspect type
sizes and alignments. Using the consuming workspace's `Cargo.lock` ensures that
the model build resolves exactly the same dependency versions.

## Consuming workspace

1. Install the Cargo subcommand:

   ```sh
   cargo install project-root-patch --version 0.1.0 --locked
   ```

2. From the workspace root, inject the local helper:

   ```sh
   cargo project-root-patch install .
   ```

   A path to the workspace's `Cargo.toml` can be used instead of `.`. Passing a
   standalone package creates a workspace in that package's manifest.

3. Review and commit the generated `project-root-patch/` directory and the
   workspace manifest changes. Then build the workspace normally.

The installed CLI version must satisfy the external crate's
`project-root-patch` version requirement. Re-run the command after updating the
CLI so that it refreshes the generated helper.

## How the patch works

The command creates `<workspace>/project-root-patch/`, adds it as a workspace
member, and adds the following source override:

```toml
[patch.crates-io]
project-root-patch = { path = "project-root-patch" }
```

Cargo therefore compiles the external crate's
`project_root_patch::get_project_root()` call against the injected package, not
the registry package. The injected package has the same library API. Its
`build.rs` calls [`project-root`](https://crates.io/crates/project-root) while
located inside the consuming workspace and records the root found from that
workspace's `Cargo.lock`.

The override applies to dependencies from crates.io. A dependency obtained
from another source requires a patch for that source instead.

## Maintainers

See [RELEASING.md](RELEASING.md) for the CI publication procedure.

## License

Licensed under either Apache-2.0 or MIT, at your option.
