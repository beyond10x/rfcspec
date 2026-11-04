//! `rfcspec extract` and `rfcspec check` as a user runs them.

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
    ] {
        fs::create_dir_all(to.join(file).parent().unwrap()).unwrap();
        fs::copy(from.join(file), to.join(file)).unwrap();
    }
}

#[test]
fn extract_reports_no_bcp14_requirement_eleven_steps_and_five_examples() {
    let output = rfcspec(&["extract", "--profile", "profiles/time", "--rfc", "868"]);
    assert!(output.status.success());
    let catalogue: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(catalogue["requirements"].as_array().unwrap().len(), 0);
    assert_eq!(catalogue["dialogue_steps"].as_array().unwrap().len(), 11);
    let examples = catalogue["examples"].as_array().unwrap();
    assert_eq!(examples.len(), 5);
    assert_eq!(examples[4]["seconds"], -1297728000);
    let step = &catalogue["dialogue_steps"][0];
    assert_eq!(step["party"], "S");
    assert_eq!(step["span"], serde_json::json!({"start": 32, "end": 32}));
}

#[test]
fn extract_output_is_byte_identical_across_two_runs() {
    let args = ["extract", "--profile", "profiles/time", "--rfc", "868"];
    assert_eq!(rfcspec(&args).stdout, rfcspec(&args).stdout);
}

#[test]
fn check_exits_zero_on_the_committed_profile() {
    let output = rfcspec(&["check", "--profile", "profiles/time"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn check_exits_one_naming_the_item_when_a_quote_changes_by_one_character() {
    let profile = tempfile::tempdir().unwrap();
    copy_profile(profile.path());
    let overlay = profile.path().join("overlay.yaml");
    let text = fs::read_to_string(&overlay).unwrap();
    let changed = text.replacen("make no reply", "make no replY", 1);
    assert_ne!(text, changed);
    fs::write(&overlay, changed).unwrap();

    let output = rfcspec(&["check", "--profile", profile.path().to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("requirement R-UDP-2: quote"), "{stderr}");
}
