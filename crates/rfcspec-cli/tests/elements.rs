//! `rfcspec review` over `machine`, `bit-widths` and `requirement-link` proposals: each recorded
//! through the in-process model double into a temporary copy of the fixture profile
//! `crates/rfcspec-propose/tests/fixtures/no-machines`, replayed, and settled by accept, edit or
//! reject; a machine or layout whose name the overlay already holds is refused and leaves the
//! overlay byte-identical. No test calls a model or reads a credential.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rfcspec_model::{
    DecisionKind, Edit, Layout, Machine, OVERLAY_FILE, Overlay, Requirement, Status, read_yaml,
    to_yaml,
};
use rfcspec_propose::{
    Outcome, Task, Workspace, double::Double, record_blocking, record_sections_blocking,
};
use serde_json::{Value, json};
use tempfile::TempDir;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A temporary copy of the fixture profile with the locked RFC 6455 text beside it.
fn fixture() -> (TempDir, PathBuf) {
    let scratch = repository().join("target/rfcspec-cli");
    fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::Builder::new()
        .prefix("elements-")
        .tempdir_in(scratch)
        .unwrap();
    let dir = temp.path().join("no-machines");
    let from = repository().join("crates/rfcspec-propose/tests/fixtures/no-machines");
    fs::create_dir_all(dir.join("sources")).unwrap();
    for file in ["profile.yaml", "overlay.yaml", "rfcspec.lock"] {
        fs::copy(from.join(file), dir.join(file)).unwrap();
    }
    fs::copy(
        repository().join("profiles/websocket/sources/rfc6455.txt"),
        dir.join("sources/rfc6455.txt"),
    )
    .unwrap();
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

fn path(dir: &Path) -> &str {
    dir.to_str().unwrap()
}

fn review(profile: &Path, proposal: &str, verdict: &[&str]) -> Output {
    let mut args = vec!["review", "--profile", path(profile), "--proposal", proposal];
    args.extend_from_slice(verdict);
    rfcspec(&args)
}

fn accepted(profile: &Path, proposal: &str, verdict: &[&str]) {
    let output = review(profile, proposal, verdict);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

fn overlay(profile: &Path) -> Overlay {
    read_yaml(&profile.join(OVERLAY_FILE)).unwrap()
}

fn overlay_bytes(profile: &Path) -> String {
    fs::read_to_string(profile.join(OVERLAY_FILE)).unwrap()
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

fn replays(profile: &Path) {
    let output = rfcspec(&["propose", "--profile", path(profile), "--replay"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}

fn id(outcome: Result<Outcome, rfcspec_propose::ProposeError>) -> String {
    match outcome {
        Ok(Outcome::Recorded(path)) => path.file_stem().unwrap().to_str().unwrap().to_owned(),
        other => panic!("{other:?}"),
    }
}

/// Record a proposal for `task` on `item` of `sections` through a double named `model`.
fn record(
    profile: &Path,
    model: &str,
    task: Task,
    item: &str,
    sections: &[&str],
    answer: Value,
) -> String {
    let double = Double::new(model, [answer]);
    let workspace = Workspace::load(profile).unwrap();
    let sections: Vec<String> = sections.iter().map(|s| (*s).to_owned()).collect();
    if sections.is_empty() {
        id(record_blocking(&workspace, &double, model, task, item))
    } else {
        id(record_sections_blocking(
            &workspace, &double, model, task, item, &sections,
        ))
    }
}

/// A Ping/Pong machine read from sections 5.5.2 and 5.5.3.
fn ping_pong(name: &str) -> Value {
    json!({
        "name": name,
        "transport": "tcp",
        "messages": [
            {"name": "ping", "fields": {"data": "Integer"}},
            {"name": "pong", "fields": {"data": "Integer"}}
        ],
        "participants": [
            {
                "name": "client",
                "initial": "open",
                "states": ["open", "pinging"],
                "inputs": [{"name": "ping", "fields": {"data": "Integer"}}],
                "transitions": [
                    {
                        "implements": ["R5.5.2-1", "R5.5.2-4"],
                        "label": "send-ping",
                        "from": "open",
                        "to": "pinging",
                        "on": {"input": "ping"},
                        "effects": [{"send": {"message": "ping", "payload": {"data": {"input": "data"}}}}]
                    },
                    {
                        "implements": ["R5.5.2-2"],
                        "label": "pong-received",
                        "from": "pinging",
                        "to": "open",
                        "on": {"receive": "pong"}
                    }
                ]
            },
            {
                "name": "server",
                "initial": "open",
                "states": ["open"],
                "transitions": [
                    {
                        "implements": ["R5.5.2-2", "R5.5.2-3", "R5.5.3-1"],
                        "label": "pong",
                        "from": "open",
                        "to": "open",
                        "on": {"receive": "ping"},
                        "effects": [{"send": {"message": "pong", "payload": {"data": {"input": "data"}}}}]
                    }
                ]
            }
        ],
        "properties": []
    })
}

fn frame_header(profile: &Path, name: &str) -> Value {
    let mut layout = serde_json::to_value(&overlay(profile).layouts[0]).unwrap();
    layout["name"] = json!(name);
    layout
}

fn link(id: &str, quote: &str, start: usize, end: usize, element: &str) -> Value {
    json!({
        "id": id,
        "source": "capitals-text",
        "level": "MUST",
        "quote": quote,
        "span": {"start": start, "end": end},
        "status": {"satisfied": element}
    })
}

fn write_entry(profile: &Path, entry: String) -> PathBuf {
    let file = profile.join("edited.yaml");
    fs::write(&file, entry).unwrap();
    file
}

#[test]
fn a_ping_pong_machine_is_recorded_replayed_accepted_and_lowers_as_a_new_model() {
    let (_temp, profile) = fixture();
    let proposal = record(
        &profile,
        "double-model",
        Task::Machine,
        "no-machines",
        &["5.5.2", "5.5.3"],
        ping_pong("ping-pong"),
    );
    replays(&profile);
    let before = overlay(&profile);
    accepted(&profile, &proposal, &["accept"]);

    let after = overlay(&profile);
    let machine: Machine = serde_json::from_value(ping_pong("ping-pong")).unwrap();
    assert_eq!(after.machines, [machine]);
    assert_eq!(after.requirements, before.requirements);
    assert_eq!(after.layouts, before.layouts);
    assert_eq!(after.decisions.len(), 1);
    let decision = &after.decisions[0];
    assert_eq!(decision.item, "no-machines");
    assert_eq!(decision.kind, DecisionKind::Accept);
    assert_eq!(decision.proposal_id.as_deref(), Some(proposal.as_str()));
    assert_eq!(decision.text_digest.len(), 64);
    replays(&profile);

    let output = rfcspec(&["lower", "--profile", path(&profile), "--check"]);
    let lowered = stderr(&output);
    assert_eq!(output.status.code(), Some(1), "{lowered}");
    let generated = profile.join("generated/ping-pong.protospec.yaml");
    assert_eq!(
        lowered.lines().collect::<Vec<_>>(),
        [format!(
            "rfcspec lower: {} is missing or differs from a fresh run",
            generated.display()
        )],
        "the generated model is new, nothing else"
    );
    let out = profile.join("lowered");
    let output = rfcspec(&["lower", "--profile", path(&profile), "--out", path(&out)]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert!(out.join("ping-pong.protospec.yaml").is_file());

    let checked = stderr(&rfcspec(&["check", "--profile", path(&profile)]));
    let unreviewed = |line: &&str| line.contains("is in scope but has no status");
    assert!(
        checked.lines().all(|line| unreviewed(&line)),
        "only the fixture's unreviewed requirements: {checked}"
    );
    let text = overlay_bytes(&profile);
    fs::write(
        profile.join(OVERLAY_FILE),
        text.replace(&decision.text_digest, &"0".repeat(64)),
    )
    .unwrap();
    let checked = stderr(&rfcspec(&["check", "--profile", path(&profile)]));
    let named: Vec<&str> = checked.lines().filter(|line| !unreviewed(line)).collect();
    assert_eq!(named.len(), 1, "{checked}");
    assert!(
        named[0].contains("decision no-machines: text_digest"),
        "{checked}"
    );
    fs::write(profile.join(OVERLAY_FILE), text).unwrap();

    let again = record(
        &profile,
        "double-model",
        Task::Machine,
        "no-machines",
        &["5.5.2"],
        ping_pong("ping-pong"),
    );
    let stderr = refused(&profile, &again, &["accept"], "AlreadyExists");
    assert!(stderr.contains("machine ping-pong"), "{stderr}");
}

#[test]
fn a_machine_is_edited_or_rejected_and_an_edit_of_another_kind_is_refused() {
    let (_temp, profile) = fixture();
    let first = record(
        &profile,
        "double-model",
        Task::Machine,
        "no-machines",
        &["5.5.2", "5.5.3"],
        ping_pong("ping-pong"),
    );
    let second = record(
        &profile,
        "double-model",
        Task::Machine,
        "no-machines",
        &["5.5.2"],
        ping_pong("ping-pong"),
    );
    let layout = write_entry(
        &profile,
        to_yaml(
            &serde_json::from_value::<Layout>(frame_header(&profile, "frame-header-copy")).unwrap(),
        )
        .unwrap(),
    );
    refused(
        &profile,
        &first,
        &["edit", "--entry", path(&layout)],
        "the edit is a layout, the proposal a machine",
    );
    refused(&profile, &first, &["edit"], "DecisionIncomplete");

    let mut edited: Machine = serde_json::from_value(ping_pong("pings")).unwrap();
    edited.participants[1].transitions[0].implements = vec!["R5.5.2-2".into()];
    let entry = write_entry(&profile, to_yaml(&edited).unwrap());
    accepted(&profile, &first, &["edit", "--entry", path(&entry)]);
    accepted(
        &profile,
        &second,
        &["reject", "--reason", "one machine is enough"],
    );

    let after = overlay(&profile);
    assert_eq!(after.machines, std::slice::from_ref(&edited));
    let decisions: Vec<(&str, DecisionKind, Option<&Edit>)> = after
        .decisions
        .iter()
        .map(|d| (d.item.as_str(), d.kind, d.edit.as_ref()))
        .collect();
    assert_eq!(
        decisions,
        [
            (
                "no-machines",
                DecisionKind::Edit,
                Some(&Edit::Machine(edited.clone()))
            ),
            ("no-machines", DecisionKind::Reject, None),
        ]
    );
    assert_ne!(
        after.decisions[0].text_digest,
        after.decisions[1].text_digest
    );
    replays(&profile);

    let mut unlowerable = edited.clone();
    unlowerable.name = "unlowerable".into();
    unlowerable.transport = "udp".into();
    let third = record(
        &profile,
        "double-model",
        Task::Machine,
        "no-machines",
        &["5.5"],
        ping_pong("ping-pong"),
    );
    let entry = write_entry(&profile, to_yaml(&unlowerable).unwrap());
    refused(
        &profile,
        &third,
        &["edit", "--entry", path(&entry)],
        "transport udp is no layer of profile.yaml",
    );
}

#[test]
fn a_layout_is_accepted_edited_or_rejected_and_one_the_overlay_holds_is_refused() {
    let (_temp, profile) = fixture();
    let copy = record(
        &profile,
        "double-model",
        Task::BitWidths,
        "frame-header-copy",
        &["5.2"],
        frame_header(&profile, "frame-header-copy"),
    );
    let second = record(
        &profile,
        "double-model",
        Task::BitWidths,
        "frame-header-second",
        &["5.2"],
        frame_header(&profile, "frame-header-second"),
    );
    let third = record(
        &profile,
        "double-model",
        Task::BitWidths,
        "frame-header-third",
        &["5.2"],
        frame_header(&profile, "frame-header-third"),
    );
    let held = record(
        &profile,
        "double-model",
        Task::BitWidths,
        "frame-header",
        &["5.2"],
        frame_header(&profile, "frame-header"),
    );
    replays(&profile);
    let stderr = refused(&profile, &held, &["accept"], "AlreadyExists");
    assert!(stderr.contains("layout frame-header"), "{stderr}");

    accepted(&profile, &copy, &["accept"]);
    let mut edited: Layout =
        serde_json::from_value(frame_header(&profile, "frame-header-second")).unwrap();
    edited.length.clear();
    let entry = write_entry(&profile, to_yaml(&edited).unwrap());
    accepted(&profile, &second, &["edit", "--entry", path(&entry)]);
    accepted(&profile, &third, &["reject", "--reason", "a duplicate"]);

    let after = overlay(&profile);
    let names: Vec<&str> = after.layouts.iter().map(|l| l.name.as_str()).collect();
    assert_eq!(
        names,
        ["frame-header", "frame-header-copy", "frame-header-second"]
    );
    assert_eq!(after.layouts[2], edited);
    let decisions: Vec<(&str, DecisionKind)> = after
        .decisions
        .iter()
        .map(|d| (d.item.as_str(), d.kind))
        .collect();
    assert_eq!(
        decisions,
        [
            ("rfc6455/layout/frame-header-copy", DecisionKind::Accept),
            ("rfc6455/layout/frame-header-second", DecisionKind::Edit),
            ("rfc6455/layout/frame-header-third", DecisionKind::Reject),
        ]
    );
    assert_eq!(after.decisions[1].edit, Some(Edit::Layout(edited)));
    replays(&profile);

    let committed = fs::read_to_string(
        repository().join("crates/rfcspec-propose/tests/fixtures/no-machines/overlay.yaml"),
    )
    .unwrap();
    let written = overlay_bytes(&profile);
    for line in committed.lines().filter(|line| !line.ends_with("[]")) {
        assert!(written.contains(line), "kept as written: {line}");
    }
}

#[test]
fn a_link_is_accepted_edited_or_rejected_and_the_requirement_gets_its_status() {
    let (_temp, profile) = fixture();
    let accept = record(
        &profile,
        "double-model",
        Task::RequirementLink,
        "R5.2-5",
        &[],
        link(
            "R5.2-5",
            "the most significant bit MUST be 0",
            1608,
            1610,
            "layout.frame-header",
        ),
    );
    let edit = record(
        &profile,
        "double-model",
        Task::RequirementLink,
        "R5.2-6",
        &[],
        link(
            "R5.2-6",
            "the minimal number of bytes MUST be used to encode the length",
            1611,
            1614,
            "layout.frame-header",
        ),
    );
    let reject = record(
        &profile,
        "double-model",
        Task::RequirementLink,
        "R5.2-7",
        &[],
        link(
            "R5.2-7",
            "Any extension MUST specify the length of the \"Extension data\"",
            1647,
            1649,
            "layout.frame-header",
        ),
    );
    replays(&profile);
    accepted(&profile, &accept, &["accept"]);
    let mut edited: Requirement = serde_json::from_value(link(
        "R5.2-6",
        "the minimal number of bytes MUST be used to encode the length",
        1611,
        1614,
        "layout.frame-header",
    ))
    .unwrap();
    edited.status = Status::Unsupported("the codec does not check minimal lengths".into());
    let entry = write_entry(&profile, to_yaml(&edited).unwrap());
    accepted(&profile, &edit, &["edit", "--entry", path(&entry)]);
    accepted(
        &profile,
        &reject,
        &["reject", "--reason", "not this layout"],
    );

    let after = overlay(&profile);
    let statuses: Vec<(&str, &Status)> = after
        .requirements
        .iter()
        .map(|r| (r.id.as_str(), &r.status))
        .collect();
    assert_eq!(
        statuses,
        [
            ("R5.2-1", &Status::Satisfied("layout.frame-header".into())),
            ("R5.2-5", &Status::Satisfied("layout.frame-header".into())),
            ("R5.2-6", &edited.status),
        ]
    );
    let kinds: Vec<DecisionKind> = after.decisions.iter().map(|d| d.kind).collect();
    assert_eq!(
        kinds,
        [
            DecisionKind::Accept,
            DecisionKind::Edit,
            DecisionKind::Reject
        ]
    );
    assert_eq!(after.decisions[1].edit, Some(Edit::Requirement(edited)));
    replays(&profile);
}
