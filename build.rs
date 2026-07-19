use std::path::PathBuf;

const GENERATED_MARKER: &str = ".project-root-patch-generated";

fn main() {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set"));
    let marker = manifest_dir.join(GENERATED_MARKER);

    println!("cargo:rerun-if-changed={}", marker.display());

    if marker.is_file() {
        let workspace_root = project_root::get_project_root()
            .unwrap_or_else(|e| panic!("Failed to determine workspace root: {}", e));
        println!("cargo:rustc-env=PROJECT_ROOT={}", workspace_root.display());
    } else {
        println!("cargo:rustc-env=PROJECT_ROOT=");
    }
}
