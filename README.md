# project-root-patch

[![CI](https://github.com/milyin/project-root-patch/actions/workflows/ci.yml/badge.svg)](https://github.com/milyin/project-root-patch/actions/workflows/ci.yml)

Make `project_root::get_project_root()` return the consuming Cargo workspace's
root when it is called by an external crate's `build.rs`.

## Problem

The upstream [`project-root`](https://crates.io/crates/project-root) function
searches for `Cargo.lock` from the calling process's current directory. An
external crate from crates.io is built from Cargo's registry cache, outside the
workspace that consumes it. Its `build.rs` therefore cannot use the function to
reliably find the consuming workspace.

This matters when the external build script needs workspace-owned state. For
example, a nested model build that inspects structure sizes and alignments must
use the consuming workspace's `Cargo.lock` to resolve identical dependency
versions.

## Workspace setup

The external crate continues using the original API and dependency:

```toml
[build-dependencies]
project-root = "0.2.2"
```

```rust,no_run
let workspace_root = project_root::get_project_root()?;
let lockfile = workspace_root.join("Cargo.lock");
# Ok::<(), std::io::Error>(())
```

The consuming workspace owner performs the patch once:

1. Install the Cargo subcommand:

   ```sh
   cargo install project-root-patch --version 0.1.0 --locked
   ```

2. Run it from the consuming workspace root:

   ```sh
   cargo project-root-patch install .
   ```

   A path to the workspace's `Cargo.toml` can be used instead of `.`. Passing a
   standalone package adds a workspace section to that package's manifest.

3. Run `cargo check` once to update `Cargo.lock`, then review and commit the
   generated `project-root-patch/` directory, workspace manifest, and lockfile.

The installed proxy currently targets `project-root` 0.2.2. The external
crate's version requirement must accept that version.

## What installation creates

The command uses `cargo vendor` to obtain the original `project-root` 0.2.2
source through Cargo's configured crates.io registry or mirror. This is the
only installation step that may require network access. Normal workspace builds
use only committed local files.

The generated directory contains:

```text
project-root-patch/
  Cargo.toml
  build.rs
  src/lib.rs
  upstream/project-root-0.2.2/  # source verified and copied by cargo vendor
```

The generated package is named `project-root`, has version `0.2.2`, and exposes
the original `get_project_root() -> std::io::Result<PathBuf>` interface. Its
`build.rs` compiles the vendored upstream implementation as a private module
while located inside the consuming workspace and records the root it finds.
The library function returns that recorded path.

The command adds the generated package as a workspace member, excludes the
nested vendored package from workspace membership, and adds:

```toml
[patch.crates-io]
project-root = { path = "project-root-patch" }
```

Cargo therefore resolves crates.io dependencies on `project-root` 0.2.2 to the
generated proxy. External crates require no source changes or dependency on
`project-root-patch`.

## Maintainers

`project-root-patch` is a Cargo utility only; it does not provide a library API.
See [RELEASING.md](RELEASING.md) for the CI publication procedure.

## License

Licensed under either Apache-2.0 or MIT, at your option. The vendored upstream
source retains its own license and package metadata.
