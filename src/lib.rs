//! Expose a consuming Cargo workspace's root to an external crate's build
//! script.
//!
//! An external crate adds `project-root-patch` as a build dependency and calls
//! [`get_project_root`] from `build.rs`. The consuming workspace must run:
//!
//! ```text
//! cargo project-root-patch install .
//! ```
//!
//! The command injects a local helper and adds a `[patch.crates-io]` override.
//! This places the substituted helper inside the consuming workspace, where its
//! build script can record that workspace's root. See the
//! [repository README](https://github.com/milyin/project-root-patch#readme) for
//! the complete setup and implementation details.
#![forbid(unsafe_code)]

use std::path::PathBuf;

/// Error message shown when the crate is built as a regular (unpatched) dependency
/// and therefore cannot determine the destination workspace root.
const NOT_IN_WORKSPACE: &str =
    "The crate `project-root-patch` is being used as a regular Cargo dependency.\n\
    Because it is not located within your workspace, it cannot determine the path to the workspace root.\n\
    Please add `project-root-patch` as a member of your workspace and patch your dependencies to use the local path.\n\n\
    You can do this with the helper tool:\n\n\
    cargo install project-root-patch\n\
    cargo project-root-patch install <path>\n\n\
    where `<path>` is the path to your workspace root.\n\n\
    If the patch is already applied and the error persists, verify the version of the patched crate.";

/// Returns the absolute path to the Cargo workspace root that built this crate.
///
/// The function is intended to be called by an external crate's `build.rs`.
/// The consuming workspace must first run `cargo project-root-patch install .`
/// so that Cargo substitutes the injected local helper for the registry copy.
///
/// # Panics
///
/// Panics with setup instructions when Cargo is using the regular registry or
/// path dependency instead of the helper injected into the consuming workspace.
///
/// # Examples
///
/// ```no_run
/// let lockfile = project_root_patch::get_project_root().join("Cargo.lock");
/// println!("cargo:rerun-if-changed={}", lockfile.display());
/// ```
pub fn get_project_root() -> PathBuf {
    let project_root = env!("PROJECT_ROOT");
    if project_root.is_empty() {
        panic!("{NOT_IN_WORKSPACE}");
    }
    project_root.into()
}

#[cfg(test)]
mod tests {
    #[test]
    #[should_panic(expected = "regular Cargo dependency")]
    fn unpatched_crate_panics_with_setup_instructions() {
        let _ = super::get_project_root();
    }
}
