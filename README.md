# project-root-patch

Make the consuming Cargo workspace's root available to an external crate's
`build.rs`.

## How it works

An external crate declares a normal dependency on `project-root-patch` and
calls this API from its `build.rs`:

```rust
let project_root = project_root_patch::get_project_root();
```

By default, Cargo builds the `project-root-patch` dependency from its registry
cache. In that mode, `project_root_patch::get_project_root()` panics with setup
instructions because it cannot determine the consuming workspace.

To make the function return the consuming workspace's root, run this command
from that workspace's root directory:

```sh
cargo project-root-patch install .
```

The command:

1. Creates `<workspace>/project-root-patch/`, a local non-published helper
   package with the same package name and the `project_root_patch` library API.
2. Adds that directory as a workspace member.
3. Adds this override to the workspace manifest:

   ```toml
   [patch.crates-io]
   project-root-patch = { path = "project-root-patch" }
   ```

Cargo applies the patch across the dependency graph. Therefore the external
crate's `project_root_patch::get_project_root()` call is compiled from the
injected local helper, rather than from the registry dependency. That helper's
`build.rs` calls the upstream [`project-root`](https://crates.io/crates/project-root)
crate while it is inside the consuming workspace, so it can record the real
workspace root.

## Example: lockfile-accurate model builds

An external crate may need to run Cargo internally to collect type layout data,
such as structure sizes and alignments. It must use the same `Cargo.lock` as
the consuming workspace; otherwise the nested build can resolve different
dependencies and report the wrong layouts. The external crate's `build.rs` can
obtain that lockfile with:

```rust
let lockfile = project_root_patch::get_project_root().join("Cargo.lock");
```

## Set up a workspace

1. Install the Cargo subcommand. Until the package is published, install it
   directly from this repository:

   ```sh
   cargo install --git https://github.com/milyin/project-root-patch project-root-patch
   ```

2. From the root of the workspace that needs this capability, install the
   local helper:

   ```sh
   cargo project-root-patch install .
   ```

3. Build the workspace normally. Every dependency on `project-root-patch`
   resolves to the local helper through the patch.
