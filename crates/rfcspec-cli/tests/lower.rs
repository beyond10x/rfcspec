//! `rfcspec lower` and the gate that runs it.

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
        "generated/tcp.protospec.yaml",
        "generated/udp.protospec.yaml",
    ] {
        fs::create_dir_all(to.join(file).parent().unwrap()).unwrap();
        fs::copy(from.join(file), to.join(file)).unwrap();
    }
}

#[test]
fn lower_writes_tcp_and_udp_models_equal_to_the_committed_ones() {
    let out = tempfile::tempdir().unwrap();
    let output = rfcspec(&[
        "lower",
        "--profile",
        "profiles/time",
        "--out",
        out.path().to_str().unwrap(),
    ]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for model in ["tcp.protospec.yaml", "udp.protospec.yaml"] {
        assert_eq!(
            fs::read(out.path().join(model)).unwrap(),
            fs::read(repository().join("profiles/time/generated").join(model)).unwrap(),
            "{model}"
        );
    }
}

#[test]
fn lower_check_passes_on_the_committed_profile() {
    let output = rfcspec(&["lower", "--profile", "profiles/time", "--check"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn lower_check_exits_one_when_a_model_is_removed_or_edited() {
    let profile = tempfile::tempdir().unwrap();
    copy_profile(profile.path());
    let dir = profile.path().to_str().unwrap();
    fs::remove_file(profile.path().join("generated/udp.protospec.yaml")).unwrap();
    let output = rfcspec(&["lower", "--profile", dir, "--check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("udp.protospec.yaml"));

    copy_profile(profile.path());
    let tcp = profile.path().join("generated/tcp.protospec.yaml");
    let text = fs::read_to_string(&tcp)
        .unwrap()
        .replace("max: 1", "max: 2");
    fs::write(&tcp, text).unwrap();
    let output = rfcspec(&["lower", "--profile", dir, "--check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("tcp.protospec.yaml"));
}

/// The commands of one task in `Taskfile.yml`, as strings.
fn task_commands(task: &str) -> Vec<String> {
    let text = fs::read_to_string(repository().join("Taskfile.yml")).unwrap();
    let taskfile: serde_json::Value = rfcspec_model::from_yaml(&text).unwrap();
    taskfile["tasks"][task]["cmds"]
        .as_array()
        .unwrap()
        .iter()
        .map(|cmd| {
            if let Some(text) = cmd.as_str().or_else(|| cmd["cmd"].as_str()) {
                text.to_string()
            } else {
                format!("task: {}", cmd["task"].as_str().unwrap())
            }
        })
        .collect()
}

#[test]
fn task_check_runs_check_lower_check_and_explores_both_models() {
    assert!(task_commands("check").contains(&"task: time".to_string()));
    let time = task_commands("time").join("\n");
    for expected in [
        "-- check --profile profiles/time",
        "-- lower --profile profiles/time --check",
        "--path profiles/time/generated/tcp.protospec.yaml --inputs profiles/time/witnesses/tcp.inputs.json | grep -q '\"verdict\": \"passed\"'",
        "--path profiles/time/generated/udp.protospec.yaml --inputs profiles/time/witnesses/udp.inputs.json | grep -q '\"verdict\": \"passed\"'",
    ] {
        assert!(time.contains(expected), "{expected} not in\n{time}");
    }
}

#[test]
fn exploring_a_removed_model_reports_no_passed_verdict() {
    let profile = tempfile::tempdir().unwrap();
    copy_profile(profile.path());
    fs::remove_file(profile.path().join("generated/tcp.protospec.yaml")).unwrap();
    let output = Command::new("ess")
        .args(["protocol", "explore", "--path"])
        .arg(profile.path().join("generated/tcp.protospec.yaml"))
        .arg("--inputs")
        .arg(repository().join("profiles/time/witnesses/tcp.inputs.json"))
        .output()
        .expect("ess on PATH");
    assert!(!String::from_utf8_lossy(&output.stdout).contains("\"verdict\": \"passed\""));
}
