//! `rfcspec generate` and the gate that runs it.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfcspec(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rfcspec"))
        .args(args)
        .current_dir(repository())
        .output()
        .unwrap()
}

fn copy_profile(to: &Path) {
    let from = repository().join("profiles/time");
    for file in [
        "profile.yaml",
        "rfcspec.lock",
        "sources/rfc868.txt",
        "overlay.yaml",
        "generated/rust/Cargo.toml",
        "generated/rust/src/lib.rs",
    ] {
        fs::create_dir_all(to.join(file).parent().unwrap()).unwrap();
        fs::copy(from.join(file), to.join(file)).unwrap();
    }
}

#[test]
fn generate_rust_writes_the_committed_crate() {
    let out = tempfile::tempdir().unwrap();
    let output = rfcspec(&[
        "generate",
        "rust",
        "--profile",
        "profiles/time",
        "--out",
        out.path().to_str().unwrap(),
    ]);
    assert!(output.status.success());
    for file in ["Cargo.toml", "src/lib.rs"] {
        assert_eq!(
            fs::read(out.path().join(file)).unwrap(),
            fs::read(repository().join("profiles/time/generated/rust").join(file)).unwrap(),
            "{file}"
        );
    }
}

#[test]
fn generate_check_passes_on_the_committed_crate() {
    assert!(rfcspec(&["generate", "--check"]).status.success());
}

#[test]
fn generate_check_exits_one_on_drift() {
    let profile = tempfile::tempdir().unwrap();
    copy_profile(profile.path());
    let lib = profile.path().join("generated/rust/src/lib.rs");
    let text = fs::read_to_string(&lib)
        .unwrap()
        .replace("from_be_bytes", "from_le_bytes");
    fs::write(&lib, text).unwrap();
    let output = rfcspec(&[
        "generate",
        "--check",
        "--profile",
        profile.path().to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("lib.rs"));
}

#[test]
fn the_generated_crate_is_a_workspace_member() {
    let manifest = fs::read_to_string(repository().join("Cargo.toml")).unwrap();
    assert!(manifest.contains("\"profiles/time/generated/rust\""));
}

#[test]
fn task_check_runs_generate_check_and_the_codec_tests() {
    let text = fs::read_to_string(repository().join("Taskfile.yml")).unwrap();
    let taskfile: serde_json::Value = rfcspec_model::from_yaml(&text).unwrap();
    let time: Vec<&str> = taskfile["tasks"]["time"]["cmds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|cmd| cmd.as_str())
        .collect();
    assert!(time.contains(&"cargo run -q --locked -p rfcspec-cli -- generate --check"));
    assert!(time.contains(&"cargo test -q --locked -p rfc868-time"));
}
