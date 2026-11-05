//! `rfcspec review`: proposals recorded through the in-process model double into copies of
//! `profiles/websocket`, each settled by accept, edit or reject into the overlay; every refusal
//! leaves the overlay byte-identical; and `rfcspec check` names a decision whose proposal is gone
//! or whose text digest is not the catalogue's. No test calls a model or reads a credential.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rfcspec_docs::catalog;
use rfcspec_model::{
    DecisionKind, OVERLAY_FILE, Overlay, Requirement, Status, read_yaml, text_digest,
};
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
        .prefix(&format!("review-{name}-"))
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

fn review(profile: &Path, proposal: &str, verdict: &[&str]) -> Output {
    let mut args = vec![
        "review",
        "--profile",
        profile.to_str().unwrap(),
        "--proposal",
        proposal,
    ];
    args.extend_from_slice(verdict);
    rfcspec(&args)
}

fn check(profile: &Path) -> Output {
    rfcspec(&["check", "--profile", profile.to_str().unwrap()])
}

fn overlay(profile: &Path) -> Overlay {
    read_yaml(&profile.join(OVERLAY_FILE)).unwrap()
}

fn overlay_bytes(profile: &Path) -> String {
    fs::read_to_string(profile.join(OVERLAY_FILE)).unwrap()
}

/// A `status` answer quoting the catalogue sentence at its span.
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

const R5_1_1: &str = "a client MUST mask all frames that it sends to the server";
const R5_1_2: &str =
    "The server MUST close the connection upon receiving a frame that is not masked.";
const R5_1_5: &str = "A client MUST close a connection if it detects a masked frame.";

fn answers() -> [(&'static str, Value); 3] {
    [
        ("R5.1-1", status_answer("R5.1-1", R5_1_1, 1490, 1493)),
        ("R5.1-2", status_answer("R5.1-2", R5_1_2, 1493, 1496)),
        ("R5.1-5", status_answer("R5.1-5", R5_1_5, 1499, 1500)),
    ]
}

/// Record one `status` proposal per `(item, answer)` through a double named `model`; returns
/// each proposal id.
fn record(profile: &Path, model: &str, answers: &[(&str, Value)]) -> Vec<String> {
    let double = Double::new(model, answers.iter().map(|(_, answer)| answer.clone()));
    let workspace = Workspace::load(profile).unwrap();
    answers
        .iter()
        .map(
            |(item, _)| match record_blocking(&workspace, &double, model, Task::Status, item) {
                Ok(Outcome::Recorded(path)) => {
                    path.file_stem().unwrap().to_str().unwrap().to_owned()
                }
                other => panic!("{item}: {other:?}"),
            },
        )
        .collect()
}

/// The catalogue text of a requirement of RFC 6455.
fn catalogue_text(profile: &Path, id: &str) -> String {
    let text = fs::read_to_string(profile.join("sources/rfc6455.txt")).unwrap();
    let catalogue = rfcspec_extract::extract(&rfcspec_doc::read(6455, "digest", &text));
    catalogue
        .requirements
        .into_iter()
        .find(|r| r.id == id)
        .unwrap()
        .text
}

fn position(overlay: &Overlay, id: &str) -> Option<usize> {
    overlay.requirements.iter().position(|r| r.id == id)
}

#[test]
fn accept_edit_and_reject_each_write_their_decision_and_the_profile_still_checks() {
    let (_temp, profile) = websocket("verdicts");
    let original = overlay_bytes(&profile);
    let ids = record(&profile, "double-model", &answers());
    let before = overlay(&profile);

    let output = review(&profile, &ids[0], &["accept"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let mut edited: Requirement = serde_json::from_value(answers()[1].1.clone()).unwrap();
    edited.status = Status::Unsupported("the frame layout leaves the mask to the codec".into());
    let entry = profile.join("edited.yaml");
    fs::write(&entry, rfcspec_model::to_yaml(&edited).unwrap()).unwrap();
    let output = review(
        &profile,
        &ids[1],
        &["edit", "--entry", entry.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let output = review(
        &profile,
        &ids[2],
        &["reject", "--reason", "the level is wrong"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let after = overlay(&profile);
    assert_eq!(
        after.requirements.len(),
        before.requirements.len() + 2,
        "accept and edit add a requirement, reject none"
    );
    let accepted: Requirement = serde_json::from_value(answers()[0].1.clone()).unwrap();
    assert_eq!(
        after.requirements[position(&after, "R5.1-1").unwrap()],
        accepted
    );
    assert_eq!(
        after.requirements[position(&after, "R5.1-2").unwrap()],
        edited
    );
    assert_eq!(position(&after, "R5.1-5"), None);
    assert_eq!(
        (
            position(&after, "R5.1-1"),
            position(&after, "R5.1-2"),
            position(&after, "R5.2-1")
        ),
        (Some(0), Some(1), Some(2)),
        "in catalogue order"
    );
    assert_eq!(after.layouts, before.layouts);
    assert_eq!(after.machines, before.machines);
    assert_eq!(after.findings, before.findings);

    let decisions: Vec<(&str, DecisionKind, Option<&str>)> = after
        .decisions
        .iter()
        .map(|d| (d.item.as_str(), d.kind, d.proposal_id.as_deref()))
        .collect();
    assert_eq!(
        decisions,
        [
            (
                "rfc6455/R5.1-1",
                DecisionKind::Accept,
                Some(ids[0].as_str())
            ),
            ("rfc6455/R5.1-2", DecisionKind::Edit, Some(ids[1].as_str())),
            (
                "rfc6455/R5.1-5",
                DecisionKind::Reject,
                Some(ids[2].as_str())
            ),
        ]
    );
    for (decision, id) in after.decisions.iter().zip(["R5.1-1", "R5.1-2", "R5.1-5"]) {
        assert_eq!(
            decision.text_digest,
            text_digest(&catalogue_text(&profile, id)),
            "{id}"
        );
    }
    assert_eq!(after.decisions[0].edit, None);
    assert_eq!(
        after.decisions[1]
            .edit
            .as_ref()
            .and_then(rfcspec_model::Edit::requirement),
        Some(&edited)
    );
    assert_eq!(after.decisions[2].edit, None);
    let reasons: Vec<Option<&str>> = after
        .decisions
        .iter()
        .map(|d| d.reason.as_deref())
        .collect();
    assert_eq!(
        reasons,
        [None, None, Some("the level is wrong")],
        "a reject holds its reason, accept and edit none"
    );

    let written = overlay_bytes(&profile);
    for line in original.lines() {
        assert!(written.contains(line), "kept as written: {line}");
    }

    let output = check(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let output = rfcspec(&[
        "propose",
        "--profile",
        profile.to_str().unwrap(),
        "--replay",
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

/// Run `review` and assert it exits 1 naming `code` and leaves the overlay byte-identical.
fn refused(profile: &Path, proposal: &str, verdict: &[&str], code: &str) -> String {
    let before = overlay_bytes(profile);
    let output = review(profile, proposal, verdict);
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{code}: {stderr}");
    assert!(stderr.contains(code), "{code} in {stderr}");
    assert_eq!(
        overlay_bytes(profile),
        before,
        "{code}: the overlay is unchanged"
    );
    stderr
}

#[test]
fn an_unknown_proposal_id_is_refused_as_not_found() {
    let (_temp, profile) = websocket("unknown");
    record(&profile, "double-model", &answers()[..1]);
    refused(&profile, &"a".repeat(64), &["accept"], "ProposalNotFound");
    refused(&profile, "../overlay", &["accept"], "ProposalNotFound");
}

#[test]
fn a_proposal_with_a_decision_is_refused_as_a_state_conflict() {
    let (_temp, profile) = websocket("conflict");
    let ids = record(&profile, "double-model", &answers()[..1]);
    let output = review(&profile, &ids[0], &["reject", "--reason", "no"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let stderr = refused(&profile, &ids[0], &["accept"], "ProposalStateConflict");
    assert!(stderr.contains("rejected"), "{stderr}");
}

#[test]
fn an_item_decided_under_the_same_text_digest_is_refused_as_already_existing() {
    let (_temp, profile) = websocket("decided");
    let first = record(&profile, "double-model", &answers()[..1]);
    let second = record(&profile, "other-model", &answers()[..1]);
    assert_ne!(first, second);
    let output = review(&profile, &first[0], &["reject", "--reason", "no"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let stderr = refused(&profile, &second[0], &["accept"], "AlreadyExists");
    assert!(stderr.contains("rfc6455/R5.1-1"), "{stderr}");
}

#[test]
fn an_item_with_an_overlay_status_and_no_decision_is_refused_as_already_existing() {
    let (_temp, profile) = websocket("hand-reviewed");
    let quote =
        "MUST be 0 unless an extension is negotiated that defines meanings for non-zero values.";
    let ids = record(
        &profile,
        "double-model",
        &[("R5.2-2", status_answer("R5.2-2", quote, 1561, 1562))],
    );
    let stderr = refused(&profile, &ids[0], &["accept"], "AlreadyExists");
    assert!(stderr.contains("rfc6455/R5.2-2"), "{stderr}");
}

#[test]
fn an_edit_without_an_entry_file_is_refused_as_incomplete() {
    let (_temp, profile) = websocket("incomplete");
    let ids = record(&profile, "double-model", &answers()[..1]);
    refused(&profile, &ids[0], &["edit"], "DecisionIncomplete");
}

#[test]
fn a_reject_with_an_empty_reason_is_refused_as_incomplete() {
    let (_temp, profile) = websocket("empty-reason");
    let ids = record(&profile, "double-model", &answers()[..1]);
    for reason in ["", "  "] {
        let stderr = refused(
            &profile,
            &ids[0],
            &["reject", "--reason", reason],
            "DecisionIncomplete",
        );
        assert!(stderr.contains("reason"), "{stderr}");
    }
}

#[test]
fn check_refuses_a_reject_decision_with_an_empty_reason() {
    let (_temp, profile) = websocket("check-reason");
    let ids = record(&profile, "double-model", &answers()[..1]);
    let output = review(
        &profile,
        &ids[0],
        &["reject", "--reason", "the level is wrong"],
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let output = check(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let text = overlay_bytes(&profile);
    assert_eq!(
        text.matches("reason: the level is wrong").count(),
        1,
        "{text}"
    );
    fs::write(
        profile.join(OVERLAY_FILE),
        text.replace("reason: the level is wrong", "reason: ''"),
    )
    .unwrap();
    let output = check(&profile);
    let stderr = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stderr.contains("decision rfc6455/R5.1-1: reject decision carries an empty reason"),
        "{stderr}"
    );
}

#[test]
fn an_edited_entry_whose_quote_is_not_at_its_span_is_refused() {
    let (_temp, profile) = websocket("bad-edit");
    let ids = record(&profile, "double-model", &answers()[..1]);
    let mut edited: Requirement = serde_json::from_value(answers()[0].1.clone()).unwrap();
    edited.span.start = 1494;
    edited.span.end = 1496;
    let entry = profile.join("edited.yaml");
    fs::write(&entry, rfcspec_model::to_yaml(&edited).unwrap()).unwrap();
    let stderr = refused(
        &profile,
        &ids[0],
        &["edit", "--entry", entry.to_str().unwrap()],
        "requirement R5.1-1: quote",
    );
    assert!(stderr.contains("does not occur at"), "{stderr}");
}

#[test]
fn check_names_the_item_of_a_decision_without_its_proposal_or_with_another_text_digest() {
    let (_temp, profile) = websocket("check");
    let ids = record(&profile, "double-model", &answers());
    for (id, verdict) in ids.iter().zip([
        &["accept"][..],
        &["reject", "--reason", "no"][..],
        &["accept"][..],
    ]) {
        let output = review(&profile, id, verdict);
        assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    }
    let output = check(&profile);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let proposal = profile.join("proposals").join(format!("{}.json", ids[1]));
    let kept = fs::read(&proposal).unwrap();
    fs::remove_file(&proposal).unwrap();
    let output = check(&profile);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("decision rfc6455/R5.1-2"), "{stderr}");
    assert!(!stderr.contains("decision rfc6455/R5.1-1"), "{stderr}");
    fs::write(&proposal, kept).unwrap();

    let digest = text_digest(&catalogue_text(&profile, "R5.1-5"));
    let text = overlay_bytes(&profile);
    assert_eq!(text.matches(&digest).count(), 1);
    fs::write(
        profile.join(OVERLAY_FILE),
        text.replace(&digest, &text_digest("another text")),
    )
    .unwrap();
    let output = check(&profile);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(stderr.contains("decision rfc6455/R5.1-5"), "{stderr}");
    assert!(!stderr.contains("decision rfc6455/R5.1-2"), "{stderr}");
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
