# project-root-patch

Make the consuming Cargo workspace's root available to an external crate's
`build.rs`.

## Why this exists

[`project-root`](https://crates.io/crates/project-root) is called by the
`build.rs` of a crate that belongs to a workspace. In that situation it can
discover that workspace's root.

That does not directly help an external crate from crates.io: its sources are
built from Cargo's cache, outside the consuming workspace. Its `build.rs` may
still need reflection on that workspace—for example, its `Cargo.lock`.

`project-root-patch` installs a small helper crate into the consuming workspace
and patches dependencies to use it. The helper's `build.rs` calls
`project-root`, records the workspace root, and exposes it through
`project_root_patch::get_project_root()` to external dependents.

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

   The command creates a `project-root-patch/` member and adds a
   `[patch.crates-io]` entry. It does not modify the external crate that needs
   the root path.

3. Build the workspace normally. Every dependency on `project-root-patch`
   resolves to the local helper through the patch.

## Use from Rust

Add `project-root-patch` as a build dependency of the crate that needs the
path, then call its library API:

```rust
let project_root = project_root_patch::get_project_root();
let lockfile = project_root.join("Cargo.lock");
```

`get_project_root()` panics with setup instructions when the helper was not
installed into the destination workspace. This avoids silently using a path
from Cargo's cache.
