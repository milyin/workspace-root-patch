# project-root-patch

Expose the consuming Cargo workspace's absolute root path to a build-time Rust
dependency.

## Why this exists

[`project-root`](https://crates.io/crates/project-root) discovers the workspace
that contains the crate being built. That is useful, and
`project-root-patch` uses it internally.

The catch is placement: a normal crates.io dependency is built from Cargo's
cache, outside the consuming workspace. Calling `project-root` there therefore
finds the cache, not the project that depends on it.

`project-root-patch` copies a small helper crate into the consuming workspace
and adds a Cargo patch for it. The helper is then built *inside* that workspace,
where `project-root` can discover the intended root.

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
   `[patch.crates-io]` entry. It does not modify the dependency that needs the
   root path.

3. Build the workspace normally. Every dependency on `project-root-patch`
   resolves to the local helper through the patch.

## Use from Rust

Add `project-root-patch` as a build dependency of the crate that needs the
path, then call its library API:

```rust
let workspace_root = project_root_patch::get_project_root();
let lockfile = workspace_root.join("Cargo.lock");
```

`get_project_root()` panics with setup instructions when the helper was not
installed into the destination workspace. This avoids silently using a path
from Cargo's cache.
