use std::{
    env, fs,
    path::{Path, PathBuf},
    process::Command,
};

use assert_cmd::{cargo::cargo_bin, prelude::*};
use predicates::prelude::*;
use tempfile::{Builder as TempDirBuilder, TempDir};
use toml_edit::DocumentMut;

fn keep_tmp_enabled() -> bool {
    env::var("PROJECT_ROOT_PATCH_KEEP_TMP")
        .map(|v| matches!(v.as_str(), "1" | "true" | "yes" | "on"))
        .unwrap_or(false)
}

fn make_tmp(prefix: &str) -> (TempDir, PathBuf) {
    let tmp = TempDirBuilder::new()
        .prefix(prefix)
        .tempdir()
        .expect("create tempdir");
    let tmp_path = tmp.path().to_path_buf();
    (tmp, tmp_path)
}

fn create_fixture(tmp_root: &Path, fixture: &str) -> PathBuf {
    let root = tmp_root.join(fixture);
    let program = r#"fn main() {
    println!("{}", project_root::get_project_root().unwrap().display());
}
"#;

    match fixture {
        "simple-package" => {
            fs::create_dir_all(root.join("src")).expect("create package source directory");
            fs::write(
                root.join("Cargo.toml"),
                r#"[package]
name = "simple_pkg"
version = "0.1.0"
edition = "2021"

[dependencies]
project-root = "=0.2.2"
"#,
            )
            .expect("write package manifest");
            fs::write(root.join("src/main.rs"), program).expect("write package source");
        }
        "simple-workspace-package" => {
            fs::create_dir_all(root.join("src")).expect("create package source directory");
            fs::write(
                root.join("Cargo.toml"),
                r#"[workspace]

[package]
name = "simple_pkg"
version = "0.1.0"
edition = "2021"

[dependencies]
project-root = "=0.2.2"
"#,
            )
            .expect("write workspace package manifest");
            fs::write(root.join("src/main.rs"), program).expect("write package source");
        }
        "workspace-package" => {
            let member = root.join("simple-member");
            fs::create_dir_all(member.join("src")).expect("create member source directory");
            fs::write(
                root.join("Cargo.toml"),
                "[workspace]\nmembers = [\"simple-member\"]\n",
            )
            .expect("write workspace manifest");
            fs::write(
                member.join("Cargo.toml"),
                r#"[package]
name = "simple_member"
version = "0.1.0"
edition = "2021"

[dependencies]
project-root = "=0.2.2"
"#,
            )
            .expect("write member manifest");
            fs::write(member.join("src/main.rs"), program).expect("write member source");
        }
        other => panic!("unknown fixture: {other}"),
    }

    eprintln!("[test] {fixture} created at: {}", root.display());
    root
}

fn run_install(manifest: &Path) {
    let bin = cargo_bin("cargo-project-root-patch");
    Command::new(bin)
        .arg("install")
        .arg(manifest)
        .assert()
        .success();
}

fn run_reinstall_without_cargo(manifest: &Path) {
    Command::new(cargo_bin("cargo-project-root-patch"))
        .arg("install")
        .arg(manifest)
        .env("CARGO", "cargo-must-not-be-called-during-reinstall")
        .assert()
        .success();
}

fn generate_registry_lockfile(workspace: &Path) {
    let output = Command::new("cargo")
        .arg("generate-lockfile")
        .current_dir(workspace)
        .output()
        .expect("run cargo generate-lockfile");
    assert!(
        output.status.success(),
        "cargo generate-lockfile failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let lockfile = fs::read_to_string(workspace.join("Cargo.lock")).expect("read Cargo.lock");
    assert!(
        lockfile.contains("registry+https://github.com/rust-lang/crates.io-index"),
        "pre-install lockfile should resolve project-root from crates.io"
    );
}

fn read_manifest_doc(manifest: &Path) -> DocumentMut {
    let manifest_text = fs::read_to_string(manifest).expect("read manifest");
    manifest_text.parse().expect("parse manifest as toml")
}

fn assert_workspace_members(doc: &DocumentMut, expected: &[&str]) {
    assert!(
        doc.contains_key("workspace"),
        "workspace section should exist"
    );
    let members = doc["workspace"]["members"]
        .as_array()
        .expect("members array");
    for m in expected {
        assert!(
            members.iter().any(|v| v.as_str() == Some(m)),
            "workspace members should include '{}'",
            m
        );
    }
}

fn assert_helper_patch(doc: &DocumentMut) {
    let patch_tbl = doc["patch"]["crates-io"]
        .as_table()
        .expect("patch.crates-io table");
    let helper = patch_tbl
        .get("project-root")
        .expect("patch entry for helper crate");
    let helper_tbl = helper.as_table().expect("helper patch table");
    let path_value = helper_tbl
        .get("path")
        .and_then(|i| i.as_str())
        .expect("path value");
    eprintln!("[test] patch crates-io.project-root.path = {}", path_value);
    assert!(
        path_value.contains("project-root-patch"),
        "path should reference 'project-root-patch'"
    );
}

fn assert_helper_files_exist(base_dir: &Path) {
    let local_helper_dir = base_dir.join("project-root-patch");
    eprintln!(
        "[test] installed helper dir: {}",
        local_helper_dir.display()
    );
    assert!(
        local_helper_dir.join("src/lib.rs").exists(),
        "src/lib.rs should exist"
    );
    assert!(
        local_helper_dir.join("build.rs").exists(),
        "build.rs should exist"
    );
    assert!(
        local_helper_dir
            .join(".project-root-patch-generated")
            .exists(),
        "generated marker should exist"
    );
    let helper_manifest = read_manifest_doc(&local_helper_dir.join("Cargo.toml"));
    assert_eq!(
        helper_manifest["package"]["name"].as_str(),
        Some("project-root")
    );
    assert_eq!(
        helper_manifest["package"]["version"].as_str(),
        Some("0.2.2")
    );
    let upstream = local_helper_dir.join("upstream").join("project-root-0.2.2");
    assert!(
        upstream.join("src/lib.rs").is_file(),
        "vendored upstream source should exist"
    );
    assert!(
        upstream.join(".cargo-checksum.json").is_file(),
        "vendored upstream checksum should exist"
    );
}

fn maybe_keep_tmp(tmp: TempDir) {
    if keep_tmp_enabled() {
        let kept_path = tmp.keep();
        eprintln!("[test] kept temp dir at: {}", kept_path.display());
    }
}

fn cargo_run_and_assert_workspace(dir: &Path, pkg: Option<&str>) {
    let pkg_suffix = pkg.map(|p| format!(" -p {}", p)).unwrap_or_default();
    eprintln!(
        "[test] running `cargo run{}` in {}",
        pkg_suffix,
        dir.display()
    );

    let mut cmd = Command::new("cargo");
    cmd.arg("run").arg("--quiet").arg("--offline");
    if let Some(p) = pkg {
        cmd.arg("-p").arg(p);
    }
    let output = cmd
        .current_dir(dir)
        .output()
        .expect("failed to execute cargo run");

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "cargo run failed.\nstatus: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr
    );

    let printed = stdout.trim();
    let actual = fs::canonicalize(Path::new(printed)).expect("canonicalize reported workspace dir");
    let expected = fs::canonicalize(dir).expect("canonicalize workspace dir");
    eprintln!("[test] program printed: {}", printed);
    eprintln!("[test] actual root   : {}", actual.display());
    eprintln!("[test] expected root : {}", expected.display());
    assert_eq!(
        actual, expected,
        "printed workspace root does not match expected"
    );

    let mut locked = Command::new("cargo");
    locked.args(["check", "--quiet", "--offline", "--locked"]);
    if let Some(package) = pkg {
        locked.arg("-p").arg(package);
    }
    let output = locked
        .current_dir(dir)
        .output()
        .expect("failed to execute locked cargo check");
    assert!(
        output.status.success(),
        "locked cargo check failed:\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn installs_into_standalone_crate() {
    let (tmp, tmp_root) = make_tmp("project-root-patch-test-");
    let dst_pkg = create_fixture(&tmp_root, "simple-package");
    let manifest = dst_pkg.join("Cargo.toml");

    run_install(&dst_pkg);

    let doc: DocumentMut = read_manifest_doc(&manifest);
    assert_workspace_members(&doc, &[".", "project-root-patch"]);
    assert!(doc["workspace"]["exclude"]
        .as_array()
        .expect("exclude array")
        .iter()
        .any(|value| value.as_str() == Some("project-root-patch/upstream/project-root-0.2.2")));
    assert_helper_patch(&doc);
    assert_helper_files_exist(&dst_pkg);

    // Running the package should print its workspace root path (the package dir itself)
    cargo_run_and_assert_workspace(&dst_pkg, None);

    maybe_keep_tmp(tmp);
}

#[test]
fn reinstall_is_idempotent() {
    let (tmp, tmp_root) = make_tmp("project-root-patch-test-");
    let dst_pkg = create_fixture(&tmp_root, "simple-package");
    let manifest = dst_pkg.join("Cargo.toml");

    run_install(&manifest);
    run_reinstall_without_cargo(&manifest);

    let doc = read_manifest_doc(&manifest);
    let members = doc["workspace"]["members"]
        .as_array()
        .expect("members array");
    assert_eq!(
        members
            .iter()
            .filter(|value| value.as_str() == Some("project-root-patch"))
            .count(),
        1
    );
    assert_helper_patch(&doc);
    cargo_run_and_assert_workspace(&dst_pkg, None);

    maybe_keep_tmp(tmp);
}

#[test]
fn refuses_to_overwrite_an_unrecognized_directory() {
    let (tmp, tmp_root) = make_tmp("project-root-patch-test-");
    let dst_pkg = create_fixture(&tmp_root, "simple-package");
    let local_helper = dst_pkg.join("project-root-patch");
    fs::create_dir(&local_helper).expect("create unrecognized directory");
    fs::write(local_helper.join("keep.txt"), "user data").expect("write user data");

    Command::new(cargo_bin("cargo-project-root-patch"))
        .arg("install")
        .arg(&dst_pkg)
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "refusing to overwrite unrecognized directory",
        ));

    assert_eq!(
        fs::read_to_string(local_helper.join("keep.txt")).expect("read user data"),
        "user data"
    );
    let doc = read_manifest_doc(&dst_pkg.join("Cargo.toml"));
    assert!(!doc.contains_key("workspace"));

    maybe_keep_tmp(tmp);
}

#[test]
fn rejects_extra_install_arguments() {
    Command::new(cargo_bin("cargo-project-root-patch"))
        .args(["install", ".", "unexpected"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "install requires exactly one <path>",
        ));
}

#[test]
fn installs_into_existing_workspace() {
    let (tmp, tmp_root) = make_tmp("project-root-patch-test-");
    let dst_ws = create_fixture(&tmp_root, "workspace-package");
    let ws_manifest = dst_ws.join("Cargo.toml");

    generate_registry_lockfile(&dst_ws);
    run_install(&ws_manifest);

    let doc: DocumentMut = read_manifest_doc(&ws_manifest);
    assert_workspace_members(&doc, &["simple-member", "project-root-patch"]);
    assert_helper_patch(&doc);
    assert_helper_files_exist(&dst_ws);

    // Running the workspace member should print the workspace root path (the workspace dir)
    cargo_run_and_assert_workspace(&dst_ws, Some("simple_member"));
    let lockfile = fs::read_to_string(dst_ws.join("Cargo.lock")).expect("read updated Cargo.lock");
    assert!(
        !lockfile.contains("git+"),
        "installed proxy must not introduce a Git source"
    );

    maybe_keep_tmp(tmp);
}

#[test]
fn installs_into_simple_workspace_root() {
    // Workspace with [workspace] in the root manifest and a root package
    let (tmp, tmp_root) = make_tmp("project-root-patch-test-");
    let dst_ws = create_fixture(&tmp_root, "simple-workspace-package");
    let ws_manifest = dst_ws.join("Cargo.toml");

    run_install(&ws_manifest);

    let doc: DocumentMut = read_manifest_doc(&ws_manifest);
    // Expect the helper crate to be added as a workspace member
    assert_workspace_members(&doc, &["project-root-patch"]);
    assert_helper_patch(&doc);
    assert_helper_files_exist(&dst_ws);

    // Running the root package should print the workspace root path (the workspace dir)
    cargo_run_and_assert_workspace(&dst_ws, None);

    maybe_keep_tmp(tmp);
}
