//! Adversarial cases on the send payload check (story:send-payload-type-check).
//!
//! `ess protocol explore` 0.86.0 refuses a send whose payload does not name exactly its message's
//! declared fields (`[PayloadFieldsMismatch]: send payload must name exactly the message's declared
//! fields`), both for a payload field the message does not declare and for a declared field the
//! payload leaves out. The overlay check skips the first and ignores the second, so an overlay that
//! `rfcspec check` passes is refused by ESS once lowered.

use std::path::PathBuf;

use rfcspec_extract::{check, extract};
use rfcspec_model::{Catalogue, Effect, FieldRef, Machine, Overlay, Send, Value, read_yaml};

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn source_6455() -> String {
    std::fs::read_to_string(root().join("profiles/websocket/sources/rfc6455.txt")).unwrap()
}

fn catalogue_6455() -> Catalogue {
    extract(&rfcspec_doc::read(6455, "digest", &source_6455()))
}

fn websocket() -> Overlay {
    read_yaml(&root().join("profiles/websocket/overlay.yaml")).unwrap()
}

fn machine<'a>(overlay: &'a mut Overlay, name: &str) -> &'a mut Machine {
    overlay
        .machines
        .iter_mut()
        .find(|m| m.name == name)
        .unwrap()
}

/// The server's pong send in the ping machine: `{message: pong, payload: {data: {input: data}}}`.
fn server_pong(overlay: &mut Overlay) -> &mut Send {
    let server = machine(overlay, "ping")
        .participants
        .iter_mut()
        .find(|p| p.name == "server")
        .unwrap();
    let sends: Vec<&mut Send> = server
        .transitions
        .iter_mut()
        .filter(|t| t.name() == "R5.5.2-2-R5.5.2-3-pong")
        .flat_map(|t| &mut t.effects)
        .filter_map(|e| match e {
            Effect::Send(send) if send.message == "pong" => Some(send),
            _ => None,
        })
        .collect();
    assert_eq!(sends.len(), 1);
    sends.into_iter().next().unwrap()
}

const PONG: &str = "transition ping.server.R5.5.2-2-R5.5.2-3-pong";

fn problems(overlay: &Overlay) -> Vec<String> {
    check(overlay, &catalogue_6455(), &source_6455())
}

fn assert_one_naming(problems: &[String], parts: &[&str]) {
    assert_eq!(problems.len(), 1, "{problems:#?}");
    for part in parts {
        assert!(problems[0].contains(part), "{part:?} not in {problems:#?}");
    }
}

/// ESS: `[PayloadFieldsMismatch]` (probe m2: an `extra: {literal 1}` added to this payload).
#[test]
#[ignore = "story:payload-field-names: payload field names are not compared with the message's declared fields"]
fn a_payload_field_the_message_does_not_declare_is_named() {
    let mut overlay = websocket();
    server_pong(&mut overlay)
        .payload
        .insert("extra".into(), Value::Integer(1));
    assert_one_naming(&problems(&overlay), &[PONG, "pong", "extra"]);
}

/// ESS: `[PayloadFieldsMismatch]` (probe m4: the payload's `data` renamed `other`). A misspelt
/// payload field is the likeliest way to reach this, and the type check is silent on it because
/// the misspelt name is not declared.
#[test]
#[ignore = "story:payload-field-names: payload field names are not compared with the message's declared fields"]
fn a_misspelt_payload_field_of_the_wrong_type_is_named() {
    let mut overlay = websocket();
    let send = server_pong(&mut overlay);
    send.payload.clear();
    send.payload.insert("Data".into(), Value::Boolean(true));
    let problems = problems(&overlay);
    assert!(
        !problems.is_empty(),
        "the check passes a payload ESS refuses"
    );
    assert!(
        problems.iter().all(|p| p.starts_with(PONG)),
        "{problems:#?}"
    );
}

/// ESS: `[PayloadFieldsMismatch]` (probe m4, the declared `data` absent from the payload).
#[test]
#[ignore = "story:payload-field-names: payload field names are not compared with the message's declared fields"]
fn a_declared_message_field_the_payload_leaves_out_is_named() {
    let mut overlay = websocket();
    server_pong(&mut overlay).payload.clear();
    assert_one_naming(&problems(&overlay), &[PONG, "pong", "data"]);
}

/// Control: a Boolean trigger field sent into an Integer message field is named.
#[test]
fn an_input_of_the_wrong_type_is_named_with_both_types() {
    let mut overlay = websocket();
    let close = machine(&mut overlay, "close");
    let client = close
        .participants
        .iter_mut()
        .find(|p| p.name == "client")
        .unwrap();
    let mut seen = 0;
    for transition in &mut client.transitions {
        for effect in &mut transition.effects {
            if let Effect::Send(send) = effect
                && send.message == "close-frame"
                && send.payload.contains_key("masked")
            {
                send.payload
                    .insert("code".into(), send.payload.get("masked").cloned().unwrap());
                seen += 1;
            }
        }
    }
    assert!(seen > 0);
    let problems = problems(&overlay);
    assert!(
        problems
            .iter()
            .any(|p| p.contains("payload code (Integer) given") && p.contains("(Boolean)")),
        "{problems:#?}"
    );
}

/// Control: an `{input: x}` naming no trigger field is named once, not also as a type mismatch.
#[test]
fn an_undeclared_input_is_named_once() {
    let mut overlay = websocket();
    server_pong(&mut overlay).payload.insert(
        "data".into(),
        Value::Field(FieldRef {
            input: "nothing".into(),
        }),
    );
    assert_eq!(
        problems(&overlay),
        [format!(
            "{PONG}: payload data reads nothing, which is no field of its trigger"
        )]
    );
}

/// Control: the committed overlays of every profile hold.
#[test]
fn the_committed_websocket_overlay_holds() {
    assert_eq!(problems(&websocket()), Vec::<String>::new());
}
