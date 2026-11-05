//! The `machine`, `bit-widths` and `requirement-link` tasks against the in-process double, on
//! temporary copies of the fixture profile `tests/fixtures/no-machines` with the locked RFC 6455
//! text of `profiles/websocket` copied beside it.

use std::{
    fs,
    path::{Path, PathBuf},
};

use serde_json::{Value, json};
use tempfile::TempDir;

use super::*;
use crate::double::Double;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A temporary copy of the fixture profile, with `sources/rfc6455.txt` copied from
/// `profiles/websocket`.
fn fixture() -> (TempDir, PathBuf) {
    let scratch = repository().join("target/rfcspec-propose");
    fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::Builder::new()
        .prefix("no-machines-")
        .tempdir_in(scratch)
        .unwrap();
    let dir = temp.path().join("no-machines");
    let from = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/no-machines");
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

fn proposals(dir: &Path) -> Vec<PathBuf> {
    match fs::read_dir(dir.join(PROPOSALS_DIR)) {
        Err(_) => Vec::new(),
        Ok(entries) => entries.map(|entry| entry.unwrap().path()).collect(),
    }
}

fn sections(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

/// A Ping/Pong machine read from sections 5.5.2 and 5.5.3.
pub(crate) fn ping_pong() -> Value {
    json!({
        "name": "ping-pong",
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

/// The committed `frame-header` layout of `profiles/websocket`, under `name`.
pub(crate) fn frame_header(name: &str) -> Value {
    let overlay: rfcspec_model::Overlay = read_yaml(
        &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/no-machines/overlay.yaml"),
    )
    .unwrap();
    let mut layout = serde_json::to_value(&overlay.layouts[0]).unwrap();
    assert_eq!(layout["name"], "frame-header");
    layout["name"] = json!(name);
    layout
}

/// A `requirement-link` answer for `R5.2-5`, satisfied by `element`.
pub(crate) fn link(element: &str) -> Value {
    json!({
        "id": "R5.2-5",
        "source": "capitals-text",
        "level": "MUST",
        "quote": "the most significant bit MUST be 0",
        "span": {"start": 1608, "end": 1610},
        "status": {"satisfied": element}
    })
}

fn recorded(outcome: Result<Outcome, ProposeError>) -> PathBuf {
    match outcome {
        Ok(Outcome::Recorded(path)) => path,
        other => panic!("{other:?}"),
    }
}

fn refused(outcome: Result<Outcome, ProposeError>, name: &str, reason: &str) {
    match outcome {
        Err(ProposeError::CheckFailed {
            proposal_id,
            reason: found,
        }) => {
            assert_eq!(proposal_id.len(), 64, "{name}");
            assert!(found.contains(reason), "{name}: {found}");
        }
        other => panic!("{name}: {other:?}"),
    }
}

#[test]
fn a_machine_is_recorded_with_the_spans_of_its_requirements_and_replays() {
    let (_temp, dir) = fixture();
    let workspace = Workspace::load(&dir).unwrap();
    let model = Double::new("double-model", [ping_pong()]);
    let path = recorded(record_sections_blocking(
        &workspace,
        &model,
        "double-model",
        Task::Machine,
        "no-machines",
        &sections(&["5.5.2", "5.5.3"]),
    ));
    let document: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(document["task"], "machine");
    assert_eq!(document["item"], "no-machines");
    assert_eq!(document["profile"], "no-machines");
    assert_eq!(document["response"], ping_pong());
    assert_eq!(
        document["cited_spans"],
        json!([
            {"start": 2040, "end": 2040},
            {"start": 2042, "end": 2043},
            {"start": 2043, "end": 2044},
            {"start": 2047, "end": 2048},
            {"start": 2064, "end": 2067}
        ])
    );
    assert_eq!(document["inputs"]["sections"], json!(["5.5.2", "5.5.3"]));
    let requests = model.requests();
    assert_eq!(requests[0].tools[0].name.as_str(), "propose_machine");

    assert_eq!(replay(&dir).unwrap(), Vec::<String>::new());
    let again = record_sections_blocking(
        &workspace,
        &model,
        "double-model",
        Task::Machine,
        "no-machines",
        &sections(&["5.5.2", "5.5.3"]),
    )
    .unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(again, Outcome::AlreadyExists(path));
    assert_eq!(model.requests().len(), 1);
}

#[test]
fn a_machine_failing_a_check_is_refused_and_nothing_is_written() {
    let mut unknown = ping_pong();
    unknown["participants"][0]["transitions"][0]["implements"] = json!(["R5.5.2-9"]);
    let mut nothing = ping_pong();
    nothing["participants"][1]["transitions"][0]["implements"] = json!([]);
    let mut transport = ping_pong();
    transport["transport"] = json!("udp");
    let mut unlowerable = ping_pong();
    let mut observer = unlowerable["participants"][1].clone();
    observer["name"] = json!("observer");
    unlowerable["participants"]
        .as_array_mut()
        .unwrap()
        .push(observer);
    let cases = [
        (
            "unknown-id",
            unknown,
            "implements R5.5.2-9, which is no catalogue requirement",
        ),
        ("implements-nothing", nothing, "implements no requirement"),
        (
            "transport",
            transport,
            "transport udp is no layer of profile.yaml",
        ),
        ("unlowerable", unlowerable, "does not lower"),
    ];
    for (name, answer, reason) in cases {
        let (_temp, dir) = fixture();
        let workspace = Workspace::load(&dir).unwrap();
        let model = Double::new("double-model", [answer]);
        refused(
            record_sections_blocking(
                &workspace,
                &model,
                "double-model",
                Task::Machine,
                "no-machines",
                &sections(&["5.5.2", "5.5.3"]),
            ),
            name,
            reason,
        );
        assert_eq!(proposals(&dir), Vec::<PathBuf>::new(), "{name}");
    }
}

#[test]
fn a_machine_proposal_names_the_profile_and_some_sections() {
    let (_temp, dir) = fixture();
    let workspace = Workspace::load(&dir).unwrap();
    let model = Double::new("double-model", [ping_pong()]);
    for (item, asked) in [
        ("websocket", vec!["5.5.2".to_owned()]),
        ("no-machines", vec![]),
    ] {
        let error = record_sections_blocking(
            &workspace,
            &model,
            "double-model",
            Task::Machine,
            item,
            &asked,
        )
        .expect_err(item);
        assert!(matches!(error, ProposeError::Other(_)), "{error}");
    }
    assert!(model.requests().is_empty());
}

#[test]
fn a_layout_reproducing_frame_header_under_another_name_is_recorded_and_a_65_bit_field_refused() {
    let (_temp, dir) = fixture();
    let workspace = Workspace::load(&dir).unwrap();
    let model = Double::new("double-model", [frame_header("frame-header-copy")]);
    let path = recorded(record_sections_blocking(
        &workspace,
        &model,
        "double-model",
        Task::BitWidths,
        "frame-header-copy",
        &sections(&["5.2"]),
    ));
    let document: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(document["task"], "bit-widths");
    assert_eq!(document["item"], "rfc6455/layout/frame-header-copy");
    assert_eq!(
        document["cited_spans"],
        json!([{"start": 1604, "end": 1604}])
    );
    assert_eq!(model.requests()[0].tools[0].name.as_str(), "propose_layout");
    assert_eq!(replay(&dir).unwrap(), Vec::<String>::new());

    let mut wide = frame_header("frame-header-wide");
    wide["fields"][6]["bits"] = json!(65);
    let mut misquoted = frame_header("frame-header-misquoted");
    misquoted["span"] = json!({"start": 1605, "end": 1605});
    let mut presence = frame_header("frame-header-presence");
    presence["fields"][7]["presence"]["field"] = json!("masking-key");
    let mut range = frame_header("frame-header-range");
    range["fields"][7]["constraints"] = json!([{"range": [126, 65536]}]);
    let other_name = frame_header("frame-header-other");
    let cases = [
        ("frame-header-wide", wide, "65 bits, not 1 to 64"),
        ("frame-header-misquoted", misquoted, "does not occur at"),
        (
            "frame-header-presence",
            presence,
            "which is no earlier field",
        ),
        ("frame-header-range", range, "does not fit 16 bits"),
        (
            "frame-header-named",
            other_name,
            "the layout is named frame-header-other, asked for frame-header-named",
        ),
    ];
    for (name, answer, reason) in cases {
        let (_temp, dir) = fixture();
        let workspace = Workspace::load(&dir).unwrap();
        let model = Double::new("double-model", [answer]);
        refused(
            record_sections_blocking(
                &workspace,
                &model,
                "double-model",
                Task::BitWidths,
                name,
                &sections(&["5.2"]),
            ),
            name,
            reason,
        );
        assert_eq!(proposals(&dir), Vec::<PathBuf>::new(), "{name}");
    }
}

#[test]
fn a_link_to_an_overlay_layout_is_recorded_and_one_to_a_missing_transition_refused() {
    let (_temp, dir) = fixture();
    let workspace = Workspace::load(&dir).unwrap();
    assert!(
        workspace
            .pending(Task::RequirementLink, &sections(&["5.2"]))
            .contains(&"R5.2-5".to_owned())
    );
    let model = Double::new("double-model", [link("layout.frame-header")]);
    let path = recorded(record_blocking(
        &workspace,
        &model,
        "double-model",
        Task::RequirementLink,
        "R5.2-5",
    ));
    let document: Value = serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap();
    assert_eq!(document["task"], "requirement-link");
    assert_eq!(document["item"], "rfc6455/R5.2-5");
    assert_eq!(
        document["inputs"]["context"]["elements"],
        json!(["layout.frame-header"])
    );
    assert_eq!(replay(&dir).unwrap(), Vec::<String>::new());

    let cases = [
        (
            "missing-transition",
            link("ping-pong.server.R5.5.2-2-pong"),
            "ping-pong.server.R5.5.2-2-pong is no element of the overlay",
        ),
        (
            "unsupported",
            {
                let mut answer = link("layout.frame-header");
                answer["status"] = json!({"unsupported": "no"});
                answer
            },
            "a link states satisfied",
        ),
    ];
    for (name, answer, reason) in cases {
        let (_temp, dir) = fixture();
        let workspace = Workspace::load(&dir).unwrap();
        let model = Double::new("double-model", [answer]);
        refused(
            record_blocking(
                &workspace,
                &model,
                "double-model",
                Task::RequirementLink,
                "R5.2-5",
            ),
            name,
            reason,
        );
        assert_eq!(proposals(&dir), Vec::<PathBuf>::new(), "{name}");
    }
}

#[test]
fn a_link_to_a_transition_that_does_not_implement_the_requirement_is_refused() {
    let (_temp, dir) = fixture();
    let mut overlay: rfcspec_model::Overlay = read_yaml(&dir.join(OVERLAY_FILE)).unwrap();
    overlay
        .machines
        .push(serde_json::from_value(ping_pong()).unwrap());
    fs::write(
        dir.join(OVERLAY_FILE),
        rfcspec_model::to_yaml(&overlay).unwrap(),
    )
    .unwrap();
    let workspace = Workspace::load(&dir).unwrap();
    let mut answer = link("ping-pong.server.R5.5.2-2-R5.5.2-3-R5.5.3-1-pong");
    answer["id"] = json!("R5.5.2-4");
    answer["level"] = json!("MAY");
    answer["quote"] = json!("An endpoint MAY send a Ping frame");
    answer["span"] = json!({"start": 2047, "end": 2048});
    let model = Double::new("double-model", [answer]);
    refused(
        record_blocking(
            &workspace,
            &model,
            "double-model",
            Task::RequirementLink,
            "R5.5.2-4",
        ),
        "not-implemented",
        "does not list R5.5.2-4 in implements",
    );
    let model = Double::new(
        "double-model",
        [{
            let mut answer = link("ping-pong.client.R5.5.2-1-R5.5.2-4-send-ping");
            answer["id"] = json!("R5.5.2-4");
            answer["level"] = json!("MAY");
            answer["quote"] = json!("An endpoint MAY send a Ping frame");
            answer["span"] = json!({"start": 2047, "end": 2048});
            answer
        }],
    );
    recorded(record_blocking(
        &workspace,
        &model,
        "double-model",
        Task::RequirementLink,
        "R5.5.2-4",
    ));
}

/// The `bit-widths` acceptance names "constraints within width", and `rfcspec-gen`
/// refuses any constraint value outside its field's bounds (`fields.rs`, `validate`), so an
/// `equals` or `one_of` value no 16-bit field can hold is refused before it is recorded.
#[test]
fn an_equals_or_one_of_value_outside_the_field_width_is_refused() {
    let mut equals = frame_header("frame-header-equals");
    equals["fields"][7]["constraints"] = json!([{"equals": 65536}]);
    let mut one_of = frame_header("frame-header-one-of");
    one_of["fields"][7]["constraints"] = json!([{"one_of": [126, 70000]}]);
    for (name, answer, value) in [
        ("frame-header-equals", equals, "65536"),
        ("frame-header-one-of", one_of, "70000"),
    ] {
        let (_temp, dir) = fixture();
        let workspace = Workspace::load(&dir).unwrap();
        let model = Double::new("double-model", [answer]);
        refused(
            record_sections_blocking(
                &workspace,
                &model,
                "double-model",
                Task::BitWidths,
                name,
                &sections(&["5.2"]),
            ),
            name,
            value,
        );
        assert_eq!(proposals(&dir), Vec::<PathBuf>::new(), "{name}");
    }
}

/// A reject writes only the decision, so a layout or machine proposal whose name the overlay
/// already holds can be rejected; accepting it stays refused.
#[test]
fn a_layout_or_machine_whose_name_the_overlay_holds_can_be_rejected_but_not_accepted() {
    let (_temp, dir) = fixture();
    let mut overlay: rfcspec_model::Overlay = read_yaml(&dir.join(OVERLAY_FILE)).unwrap();
    overlay
        .machines
        .push(serde_json::from_value(ping_pong()).unwrap());
    fs::write(
        dir.join(OVERLAY_FILE),
        rfcspec_model::to_yaml(&overlay).unwrap(),
    )
    .unwrap();
    let workspace = Workspace::load(&dir).unwrap();
    let model = Double::new("double-model", [frame_header("frame-header"), ping_pong()]);
    let layout = recorded(record_sections_blocking(
        &workspace,
        &model,
        "double-model",
        Task::BitWidths,
        "frame-header",
        &sections(&["5.2"]),
    ));
    let machine = recorded(record_sections_blocking(
        &workspace,
        &model,
        "double-model",
        Task::Machine,
        "no-machines",
        &sections(&["5.5.2", "5.5.3"]),
    ));
    for (path, held) in [
        (layout, "layout frame-header"),
        (machine, "machine ping-pong"),
    ] {
        let id = path.file_stem().unwrap().to_str().unwrap().to_owned();
        match review::decide(&dir, &id, review::Verdict::Accept) {
            Err(review::ReviewError::AlreadyExists { reason, .. }) => {
                assert!(reason.contains(held), "{held}: {reason}");
            }
            other => panic!("{held}: {other:?}"),
        }
        let reviewed = review::decide(&dir, &id, review::Verdict::Reject)
            .unwrap_or_else(|error| panic!("{held}: {error}"));
        assert_eq!(
            reviewed.decision.kind,
            rfcspec_model::DecisionKind::Reject,
            "{held}"
        );
        assert_eq!(reviewed.overlay.layouts, overlay.layouts, "{held}");
        assert_eq!(reviewed.overlay.machines, overlay.machines, "{held}");
        review::write(&dir, &reviewed).unwrap();
    }
    assert_eq!(
        Workspace::load(&dir).unwrap().overlay.decisions.len(),
        overlay.decisions.len() + 2
    );
}
