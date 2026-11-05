//! `rfcspec propose`: proposals recorded through the in-process model double into copies of
//! `profiles/websocket`, replayed byte for byte by the binary, and refused when edited; and a
//! routes file whose secret is unset, refused before any request. No test calls a model or reads
//! a credential.

use std::{
    fs,
    io::ErrorKind,
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rfcspec_docs::catalog;
use rfcspec_model::{OVERLAY_FILE, Overlay, Requirement, read_yaml, to_yaml};
use rfcspec_propose::{Outcome, Task, Workspace, double::Double, record_blocking};
use serde_json::{Value, json};
use tempfile::TempDir;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A fresh copy of `profiles/websocket` as it stood before its section 5.1 and 5.3 proposals
/// were reviewed (without `proposals/`, the decisions, or the requirement entries of those two
/// sections), in a temporary directory under the workspace's build directory that is removed
/// when the returned guard drops.
fn websocket(name: &str) -> (TempDir, PathBuf) {
    let scratch = repository().join("target/rfcspec-cli");
    fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::Builder::new()
        .prefix(&format!("propose-{name}-"))
        .tempdir_in(scratch)
        .unwrap();
    let dir = temp.path().join("websocket");
    let from = repository().join("profiles/websocket");
    for file in catalog::files(&from).unwrap() {
        if file.starts_with("proposals/") {
            continue;
        }
        fs::create_dir_all(dir.join(&file).parent().unwrap()).unwrap();
        fs::copy(from.join(&file), dir.join(&file)).unwrap();
    }
    let path = dir.join(OVERLAY_FILE);
    let text = fs::read_to_string(&path).unwrap();
    fs::write(&path, before_the_slice(&text)).unwrap();
    (temp, dir)
}

fn rfcspec(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rfcspec"))
        .args(args)
        .current_dir(repository())
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn replay(profile: &Path) -> Output {
    rfcspec(&[
        "propose",
        "--profile",
        profile.to_str().unwrap(),
        "--replay",
    ])
}

fn check(profile: &Path) -> Output {
    rfcspec(&["check", "--profile", profile.to_str().unwrap()])
}

/// A `status` answer for a section 5.1 requirement, quoting the catalogue sentence at its span.
fn status_answer(id: &str, quote: &str, start: usize, end: usize) -> Value {
    json!({
        "id": id,
        "source": "capitals-text",
        "level": "MUST",
        "quote": quote,
        "span": {"start": start, "end": end},
        "status": {"not-applicable": "section 5.1 is outside the profile's scope"}
    })
}

/// Record `R5.1-1`, `R5.1-2` and `R5.1-5` into `profile` through the double.
fn record_three(profile: &Path) -> Vec<PathBuf> {
    let answers = [
        status_answer(
            "R5.1-1",
            "a client MUST mask all frames that it sends to the server",
            1490,
            1493,
        ),
        status_answer(
            "R5.1-2",
            "The server MUST close the connection upon receiving a frame that is not masked.",
            1493,
            1496,
        ),
        status_answer(
            "R5.1-5",
            "A client MUST close a connection if it detects a masked frame.",
            1499,
            1500,
        ),
    ];
    let model = Double::new("double-model", answers);
    let workspace = Workspace::load(profile).unwrap();
    ["R5.1-1", "R5.1-2", "R5.1-5"]
        .into_iter()
        .map(
            |item| match record_blocking(&workspace, &model, "double-model", Task::Status, item) {
                Ok(Outcome::Recorded(path)) => path,
                other => panic!("{item}: {other:?}"),
            },
        )
        .collect()
}

/// Replace the first `from` in `path` by `to`, which differs from it in one byte.
fn flip(path: &Path, from: &str, to: &str) {
    assert_eq!(from.len(), to.len());
    let text = fs::read_to_string(path).unwrap();
    assert!(text.contains(from), "{from} in {}", path.display());
    fs::write(path, text.replacen(from, to, 1)).unwrap();
}

#[test]
fn three_recorded_proposals_replay_byte_for_byte_and_an_edited_byte_is_named() {
    let (_temp, profile) = websocket("replay");
    let files = record_three(&profile);
    let recorded: Vec<String> = files
        .iter()
        .map(|file| fs::read_to_string(file).unwrap())
        .collect();

    let output = replay(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let replayed: Vec<String> = files
        .iter()
        .map(|file| fs::read_to_string(file).unwrap())
        .collect();
    assert_eq!(replayed, recorded, "replay writes nothing");

    flip(
        &files[0],
        "\"quote\": \"a client MUST mask all",
        "\"quote\": \"a client MUST mask alL",
    );
    let digest: Value = serde_json::from_str(&recorded[1]).unwrap();
    let digest = digest["input_digest"].as_str().unwrap();
    let last = digest.chars().last().unwrap();
    let flipped = format!(
        "{}{}",
        &digest[..digest.len() - 1],
        if last == '0' { '1' } else { '0' }
    );
    flip(&files[1], digest, &flipped);

    let output = replay(&profile);
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    for file in &files[..2] {
        let name = file.file_name().unwrap().to_str().unwrap();
        assert!(stderr.contains(name), "{name} in {stderr}");
    }
    let untouched = files[2].file_name().unwrap().to_str().unwrap();
    assert!(!stderr.contains(untouched), "{untouched} in {stderr}");
}

#[test]
fn check_runs_the_replay_and_refuses_an_edited_proposal() {
    let (_temp, profile) = websocket("check");
    let files = record_three(&profile);
    let output = check(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    flip(
        &files[2],
        "\"quote\": \"A client MUST close",
        "\"quote\": \"A client MUST closE",
    );
    let output = check(&profile);
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    let name = files[2].file_name().unwrap().to_str().unwrap();
    assert!(stderr.contains(name), "{name} in {stderr}");
}

#[test]
fn a_file_not_named_by_its_proposal_id_is_named() {
    let (_temp, profile) = websocket("renamed");
    let files = record_three(&profile);
    let renamed = files[0].with_file_name("0000.json");
    fs::rename(&files[0], &renamed).unwrap();
    let output = replay(&profile);
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("0000.json"), "{stderr}");
}

const PONG: &str = "A Pong frame sent in response to a Ping frame must have identical \"Application data\" as found in the message body of the Ping frame being replied to.";

#[test]
fn a_requirement_from_prose_replays_and_its_entry_passes_check() {
    let (_temp, profile) = websocket("prose");
    let answer = json!({
        "id": "P-34",
        "source": "prose",
        "level": "MUST",
        "quote": PONG,
        "span": {"start": 2060, "end": 2062},
        "status": {"unsupported": "the ping machine does not compare the application data of a Pong with its Ping"}
    });
    let model = Double::new("double-model", [answer.clone()]);
    let workspace = Workspace::load(&profile).unwrap();
    let path = match record_blocking(
        &workspace,
        &model,
        "double-model",
        Task::RequirementFromProse,
        "P-34",
    ) {
        Ok(Outcome::Recorded(path)) => path,
        other => panic!("{other:?}"),
    };
    let recorded = fs::read_to_string(&path).unwrap();
    let document: Value = serde_json::from_str(&recorded).unwrap();
    assert_eq!(document["task"], "requirement-from-prose");
    assert_eq!(document["item"], "rfc6455/P-34");

    let output = replay(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(fs::read_to_string(&path).unwrap(), recorded);

    let output = check(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let mut overlay: Overlay = read_yaml(&profile.join(OVERLAY_FILE)).unwrap();
    let entry: Requirement = serde_json::from_value(answer).unwrap();
    overlay.requirements.push(entry);
    fs::write(profile.join(OVERLAY_FILE), to_yaml(&overlay).unwrap()).unwrap();
    let output = check(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

#[test]
fn a_profile_without_proposals_replays_nothing_and_passes() {
    let (_temp, profile) = websocket("none");
    assert!(!profile.join("proposals").exists());
    let output = replay(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

const NEVER_SET: &str = "RFCSPEC_PROPOSE_TEST_NEVER_SET";

#[test]
fn a_routes_file_whose_env_secret_is_unset_is_refused_before_any_request_naming_the_endpoint() {
    let (_temp, profile) = websocket("unset");
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.set_nonblocking(true).unwrap();
    let port = listener.local_addr().unwrap().port();
    let routes = profile.join("routes.yaml");
    fs::write(
        &routes,
        format!(
            "format: llm.routes/1\nendpoints:\n  lab: {{base_url: http://127.0.0.1:{port}/v1, protocol: chat-completions,\n        auth: {{kind: bearer, secret: env:{NEVER_SET}}}}}\nroutes:\n  small: lab/small-model\n"
        ),
    )
    .unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_rfcspec"))
        .args([
            "propose",
            "--profile",
            profile.to_str().unwrap(),
            "--task",
            "status",
            "--section",
            "5.1",
            "--routes",
            routes.to_str().unwrap(),
            "--route",
            "small",
        ])
        .env_remove(NEVER_SET)
        .current_dir(repository())
        .output()
        .unwrap();
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("`lab`"), "{stderr}");
    assert!(!stderr.contains(NEVER_SET), "{stderr}");
    assert_eq!(
        listener.accept().map(|_| ()).unwrap_err().kind(),
        ErrorKind::WouldBlock,
        "no request reached the endpoint"
    );
    assert!(!profile.join("proposals").exists());
}

/// The text of the websocket overlay without its decisions and without the requirement entries
/// of sections 5.1 and 5.3, every other line as written.
fn before_the_slice(overlay: &str) -> String {
    let overlay = overlay.split("\ndecisions:\n").next().unwrap();
    let mut out = String::new();
    let mut skipping = false;
    for line in overlay.lines() {
        if line.starts_with("  - ") || !line.starts_with("    ") {
            skipping = line.starts_with("  - id: R5.1-") || line.starts_with("  - id: R5.3-");
        }
        if !skipping {
            out.push_str(line);
            out.push('\n');
        }
    }
    out
}
