//! Record and the mechanical checks against the in-process double, on copies of
//! `profiles/websocket` and `profiles/dns` in temporary directories under the workspace's
//! `target/`, each removed when its test ends.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use tempfile::TempDir;

use super::*;
use crate::double::Double;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A fresh copy of the files of `profiles/<profile>` that proposing reads, in a temporary
/// directory under the workspace's `target/` that is removed when the returned guard drops.
fn copy_profile(profile: &str, rfc: u32, name: &str) -> (TempDir, PathBuf) {
    let scratch = repository().join("target/rfcspec-propose");
    fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::Builder::new()
        .prefix(&format!("{name}-"))
        .tempdir_in(scratch)
        .unwrap();
    let dir = temp.path().join(profile);
    let from = repository().join("profiles").join(profile);
    let source = format!("sources/rfc{rfc}.txt");
    for file in ["profile.yaml", "overlay.yaml", "rfcspec.lock", &source] {
        fs::create_dir_all(dir.join(file).parent().unwrap()).unwrap();
        fs::copy(from.join(file), dir.join(file)).unwrap();
    }
    (temp, dir)
}

/// A fresh copy of `profiles/websocket`; see [`copy_profile`].
fn websocket(name: &str) -> (TempDir, PathBuf) {
    copy_profile("websocket", 6455, name)
}

/// A fresh copy of `profiles/dns`; see [`copy_profile`].
fn dns(name: &str) -> (TempDir, PathBuf) {
    copy_profile("dns", 1035, name)
}

const R5_1_1: &str = "To avoid confusing network intermediaries (such as intercepting proxies) and for security reasons that are further discussed in Section 10.3, a client MUST mask all frames that it sends to the server (see Section 5.3 for further details).";

/// A valid `status` answer for `R5.1-1`.
fn answer() -> Value {
    json!({
        "id": "R5.1-1",
        "source": "capitals-text",
        "level": "MUST",
        "quote": R5_1_1,
        "span": {"start": 1490, "end": 1493},
        "status": {"unsupported": "masking is not modelled: the frame layout leaves the mask to the codec"}
    })
}

fn proposals(dir: &Path) -> Vec<PathBuf> {
    match fs::read_dir(dir.join(PROPOSALS_DIR)) {
        Err(_) => Vec::new(),
        Ok(entries) => entries.map(|entry| entry.unwrap().path()).collect(),
    }
}

fn sha256(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

#[test]
fn a_status_answer_is_recorded_canonically_and_a_second_record_reads_it() {
    let (_temp, dir) = websocket("recorded");
    let workspace = Workspace::load(&dir).unwrap();
    let model = Double::new("double-model", [answer(), answer()]);

    let outcome = record_blocking(&workspace, &model, "double-model", Task::Status, "R5.1-1")
        .unwrap_or_else(|error| panic!("{error}"));
    let Outcome::Recorded(path) = outcome else {
        panic!("{outcome:?}");
    };
    assert_eq!(proposals(&dir), std::slice::from_ref(&path));
    let bytes = fs::read_to_string(&path).unwrap();
    assert!(bytes.ends_with("}\n"), "{bytes}");
    assert!(bytes.starts_with("{\n  \"cited_spans\""), "{bytes}");

    let document: Value = serde_json::from_str(&bytes).unwrap();
    assert_eq!(document["format"], FORMAT);
    assert_eq!(document["profile"], "websocket");
    assert_eq!(document["item"], "rfc6455/R5.1-1");
    assert_eq!(document["task"], "status");
    assert_eq!(document["model_id"], "double-model");
    assert_eq!(document["response"], answer());
    assert_eq!(
        document["cited_spans"],
        json!([{"start": 1490, "end": 1493}])
    );
    let digest = document["input_digest"].as_str().unwrap();
    assert_eq!(digest.len(), 64);
    let id = sha256(&format!("status\ndouble-model\n{digest}"));
    assert_eq!(document["proposal_id"], id.as_str());
    assert_eq!(
        path.file_name().unwrap().to_str().unwrap(),
        format!("{id}.json")
    );
    let keys: Vec<&String> = document.as_object().unwrap().keys().collect();
    let sorted = {
        let mut sorted = keys.clone();
        sorted.sort();
        sorted
    };
    assert_eq!(keys, sorted);

    let again = record_blocking(&workspace, &model, "double-model", Task::Status, "R5.1-1")
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(again, Outcome::AlreadyExists(path.clone()));
    assert_eq!(
        model.requests().len(),
        1,
        "an existing proposal is not asked again"
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), bytes);
}

#[test]
fn the_model_is_forced_to_answer_one_overlay_requirement_entry() {
    let (_temp, dir) = websocket("forced");
    let workspace = Workspace::load(&dir).unwrap();
    let model = Double::new("double-model", [answer()]);
    record_blocking(&workspace, &model, "double-model", Task::Status, "R5.1-1")
        .unwrap_or_else(|error| panic!("{error}"));
    let requests = model.requests();
    let request = &requests[0];
    assert_eq!(request.tools.len(), 1);
    let schema = &request.tools[0].input_schema;
    let mut required: Vec<&str> = schema["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    required.sort_unstable();
    assert_eq!(
        required,
        ["id", "level", "quote", "source", "span", "status"]
    );
    assert_eq!(
        request.tool_choice,
        llm_core::ToolChoice::Named(request.tools[0].name.clone())
    );
}

/// Each answer fails one check: nothing is written, and the error names the proposal and why.
#[test]
fn an_answer_failing_a_check_writes_nothing_and_names_the_proposal_and_the_reason() {
    let mut not_in_source = answer();
    not_in_source["quote"] = json!("a client MUST NOT mask any frame");
    let mut past_the_end = answer();
    past_the_end["span"] = json!({"start": 1490, "end": 99_999});
    let mut other_id = answer();
    other_id["id"] = json!("R5.1-2");
    let mut satisfied = answer();
    satisfied["status"] = json!({"satisfied": "layout.frame-header"});
    let mut empty_reason = answer();
    empty_reason["status"] = json!({"not-applicable": "  "});
    let cases = [
        (
            "not-in-source",
            not_in_source,
            "does not occur at lines 1490-1493",
        ),
        (
            "past-the-end",
            past_the_end,
            "lines 1490-99999 is outside the source",
        ),
        ("other-id", other_id, "answers R5.1-2, asked about R5.1-1"),
        ("satisfied", satisfied, "status satisfied"),
        ("empty-reason", empty_reason, "empty reason"),
    ];
    for (name, answer, reason) in cases {
        let (_temp, dir) = websocket(name);
        let workspace = Workspace::load(&dir).unwrap();
        let model = Double::new("double-model", [answer]);
        let error = record_blocking(&workspace, &model, "double-model", Task::Status, "R5.1-1")
            .expect_err(name);
        let ProposeError::CheckFailed {
            proposal_id,
            reason: found,
        } = &error
        else {
            panic!("{name}: {error}");
        };
        assert_eq!(proposal_id.len(), 64, "{name}");
        assert!(found.contains(reason), "{name}: {found}");
        let message = error.to_string();
        assert!(message.contains(proposal_id.as_str()), "{name}: {message}");
        assert!(message.contains(reason), "{name}: {message}");
        assert_eq!(proposals(&dir), Vec::<PathBuf>::new(), "{name}");
    }
}

#[test]
fn an_answer_that_is_no_overlay_requirement_entry_is_refused() {
    let (_temp, dir) = websocket("not-an-entry");
    let workspace = Workspace::load(&dir).unwrap();
    let model = Double::new("double-model", [json!({"id": "R5.1-1"})]);
    let error = record_blocking(&workspace, &model, "double-model", Task::Status, "R5.1-1")
        .expect_err("refused");
    assert!(
        matches!(&error, ProposeError::CheckFailed { reason, .. } if reason.contains("not an overlay requirement entry")),
        "{error}"
    );
    assert_eq!(proposals(&dir), Vec::<PathBuf>::new());
}

#[test]
fn pending_lists_catalogue_requirements_of_the_sections_without_an_overlay_status() {
    let (_temp, dir) = websocket("pending");
    let workspace = Workspace::load(&dir).unwrap();
    assert_eq!(
        workspace.pending(Task::Status, &["5.1".into()]),
        [
            "R5.1-1", "R5.1-2", "R5.1-3", "R5.1-4", "R5.1-5", "R5.1-6", "R5.1-7"
        ]
    );
    assert_eq!(
        workspace.pending(Task::Status, &["5.2".into()]),
        Vec::<String>::new(),
        "section 5.2 is reviewed"
    );
    assert!(
        workspace
            .pending(Task::RequirementFromProse, &["5.5.3".into()])
            .contains(&"P-34".to_string())
    );
}

/// Write a decision on `id` (a section 5.1 requirement) into the overlay of `dir`, judging the
/// item's current catalogue text.
fn decide_by_hand(dir: &Path, id: &str, kind: rfcspec_model::DecisionKind) {
    let workspace = Workspace::load(dir).unwrap();
    let text = workspace
        .catalogue
        .requirements
        .iter()
        .find(|r| r.id == id)
        .unwrap()
        .text
        .clone();
    let mut overlay = workspace.overlay.clone();
    overlay.decisions.push(rfcspec_model::Decision {
        item: format!("rfc6455/{id}"),
        text_digest: rfcspec_model::text_digest(&text),
        kind,
        proposal_id: Some("0".repeat(64)),
        edit: None,
    });
    fs::write(
        dir.join(rfcspec_model::OVERLAY_FILE),
        rfcspec_model::to_yaml(&overlay).unwrap(),
    )
    .unwrap();
}

#[test]
fn recording_skips_an_item_with_a_decision_and_does_not_ask_the_model() {
    let (_temp, dir) = websocket("decided");
    decide_by_hand(&dir, "R5.1-1", rfcspec_model::DecisionKind::Reject);
    let workspace = Workspace::load(&dir).unwrap();
    let pending = workspace.pending(Task::Status, &["5.1".into()]);
    assert!(
        !pending.contains(&"R5.1-1".to_string()),
        "a rejected item is not offered again"
    );
    assert_eq!(pending.len(), 6);
    let model = Double::new("double-model", [answer()]);
    let outcome = record_blocking(&workspace, &model, "double-model", Task::Status, "R5.1-1")
        .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(outcome, Outcome::Decided);
    assert!(model.requests().is_empty(), "the model was not asked");
    assert_eq!(proposals(&dir), Vec::<PathBuf>::new());
}

const P_56: &str =
    "Queries sent using UDP may be lost, and hence a retransmission strategy is required.";
const R_UDP_1: &str = "Queries or their responses may be reordered by the network, or by processing in name servers, so resolvers should not depend on them being returned in order.";

/// A `requirement-from-prose` answer for prose candidate `id` of RFC 1035.
fn prose(id: &str, quote: &str, start: usize, end: usize) -> Value {
    json!({
        "id": id,
        "source": "prose",
        "level": "MUST",
        "quote": quote,
        "span": {"start": start, "end": end},
        "status": {"unsupported": "retransmission is not modelled"}
    })
}

/// `profiles/dns` reviewed P-56 and P-57 as `R-UDP-2` and `R-UDP-1`: under other ids, but
/// quoting their sentences within their spans. They are not pending again.
#[test]
fn a_prose_candidate_an_overlay_requirement_quotes_is_not_pending() {
    let (_temp, dir) = dns("prose-pending");
    let workspace = Workspace::load(&dir).unwrap();
    let pending = workspace.pending(Task::RequirementFromProse, &["4.2.1".into()]);
    assert!(pending.contains(&"P-58".to_string()), "{pending:?}");
    for reviewed in ["P-56", "P-57"] {
        assert!(!pending.contains(&reviewed.to_string()), "{pending:?}");
    }
}

/// Accepting a proposal for P-56, or an edit of P-58 that quotes `R-UDP-1`, would put a
/// requirement into the overlay twice under two ids: review refuses both.
#[test]
fn review_refuses_an_entry_duplicating_an_overlay_requirement() {
    let (_temp, dir) = dns("prose-duplicate");
    let workspace = Workspace::load(&dir).unwrap();
    let candidate = |id: &str| {
        workspace
            .catalogue
            .prose_candidates
            .iter()
            .find(|c| c.id == id)
            .unwrap()
            .clone()
    };
    let p58 = candidate("P-58");
    let model = Double::new(
        "double-model",
        [
            prose("P-56", P_56, 1761, 1762),
            prose("P-58", &p58.text, p58.span.start, p58.span.end),
        ],
    );
    let id = |outcome: Result<Outcome, ProposeError>| match outcome {
        Ok(Outcome::Recorded(path)) => path.file_stem().unwrap().to_str().unwrap().to_owned(),
        other => panic!("{other:?}"),
    };
    let p56 = id(record_blocking(
        &workspace,
        &model,
        "double-model",
        Task::RequirementFromProse,
        "P-56",
    ));
    let p58 = id(record_blocking(
        &workspace,
        &model,
        "double-model",
        Task::RequirementFromProse,
        "P-58",
    ));
    let overlay = fs::read_to_string(dir.join(rfcspec_model::OVERLAY_FILE)).unwrap();
    let edit: rfcspec_model::Requirement =
        serde_json::from_value(prose("P-58", R_UDP_1, 1762, 1764)).unwrap();
    for (name, proposal, verdict, existing) in [
        ("accept P-56", &p56, review::Verdict::Accept, "R-UDP-2"),
        (
            "edit P-58",
            &p58,
            review::Verdict::Edit(Some(rfcspec_model::Edit::Requirement(edit))),
            "R-UDP-1",
        ),
    ] {
        match review::decide(&dir, proposal, verdict) {
            Err(review::ReviewError::AlreadyExists { reason, .. }) => {
                assert!(reason.contains(existing), "{name}: {reason}");
            }
            other => panic!("{name}: {other:?}"),
        }
    }
    assert_eq!(
        fs::read_to_string(dir.join(rfcspec_model::OVERLAY_FILE)).unwrap(),
        overlay
    );
}
