#![forbid(unsafe_code)]
//! S5: lower the overlay's machines to `ess-protospec/1`, one model per machine (one per
//! transport). Each machine has two participants joined by one FIFO channel in each direction; a
//! transition's lowered name is the requirement and dialogue-step ids it implements, then its label.
//! A machine carries inputs with typed fields, timers armed for a fixed delay, payload values that
//! are literals or fields of the triggering input or received message, `flush`, and the
//! `event_count` and `flush_before_close` properties; its bounds default to [`BOUNDS`].
//!
//! The models are validated and explored by the `ess` command, never by this crate.

use std::{collections::BTreeMap, path::Path, process::Command};

use anyhow::{Context, Result, bail, ensure};
use rfcspec_model::{
    Bounds, Effect, Generated, Machine, Overlay, Property as OverlayProperty, Trigger,
    Value as OverlayValue, to_yaml,
};
use serde::Serialize;

/// The exploration bounds of a lowered model whose machine names none.
pub const BOUNDS: Bounds = Bounds {
    max_steps: 32,
    max_states: 256,
    max_time_ms: 1000,
};

/// The capacity of each lowered channel.
const CAPACITY: u32 = 4;

/// Lower every machine of an overlay to `<machine>.protospec.yaml`. `profile` names the models
/// (`rfc<N>-<profile>-<machine>`).
pub fn lower(overlay: &Overlay, profile: &str) -> Result<Vec<Generated>> {
    overlay
        .machines
        .iter()
        .map(|machine| {
            let model = model(overlay.rfc, profile, machine)?;
            Ok(Generated {
                file: format!("{}.protospec.yaml", machine.name),
                text: to_yaml(&model)?,
            })
        })
        .collect()
}

/// Run `ess protocol explore` on a model with its input witnesses and return its JSON report.
pub fn explore(model: &Path, inputs: &Path) -> Result<serde_json::Value> {
    let output = Command::new("ess")
        .args(["protocol", "explore", "--path"])
        .arg(model)
        .arg("--inputs")
        .arg(inputs)
        .output()
        .context("running `ess protocol explore` (is ess 0.86.0 on PATH?)")?;
    serde_json::from_slice(&output.stdout).with_context(|| {
        format!(
            "`ess protocol explore` printed no report for {}: {}",
            model.display(),
            String::from_utf8_lossy(&output.stderr)
        )
    })
}

#[derive(Serialize)]
struct Model {
    format: &'static str,
    name: String,
    bounds: Bounds,
    participants: Vec<Participant>,
    messages: Vec<Message>,
    channels: Vec<Channel>,
    properties: Vec<Property>,
}

#[derive(Serialize)]
struct Participant {
    name: String,
    initial: String,
    states: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    inputs: Vec<Message>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    timers: Vec<String>,
    transitions: Vec<Transition>,
}

#[derive(Serialize)]
struct Message {
    name: String,
    fields: Vec<FieldType>,
}

#[derive(Serialize)]
struct FieldType {
    name: String,
    #[serde(rename = "type")]
    kind: String,
}

fn fields(fields: &BTreeMap<String, String>) -> Vec<FieldType> {
    fields
        .iter()
        .map(|(name, kind)| FieldType {
            name: name.clone(),
            kind: kind.clone(),
        })
        .collect()
}

#[derive(Serialize)]
struct Transition {
    name: String,
    from: String,
    to: String,
    trigger: TriggerOut,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    effects: Vec<EffectOut>,
}

#[derive(Serialize)]
struct TriggerOut {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
}

#[derive(Default, Serialize)]
struct EffectOut {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exchange: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logical: Option<Source>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<BTreeMap<String, Source>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<Delay>,
}

/// A value source: a literal, or a field of the triggering input or received message.
#[derive(Serialize)]
struct Source {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    field: Option<String>,
}

#[derive(Serialize)]
struct Delay {
    kind: &'static str,
    millis: u64,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Value {
    Integer(i64),
    Boolean(bool),
    Text(String),
}

fn literal(value: Value) -> Source {
    Source {
        kind: "literal",
        value: Some(value),
        field: None,
    }
}

fn source(value: &OverlayValue) -> Source {
    match value {
        OverlayValue::Integer(n) => literal(Value::Integer(*n)),
        OverlayValue::Boolean(b) => literal(Value::Boolean(*b)),
        OverlayValue::Field(field) => Source {
            kind: "input",
            value: None,
            field: Some(field.input.clone()),
        },
    }
}

#[derive(Serialize)]
struct Channel {
    name: String,
    from: String,
    to: String,
    ordering: &'static str,
    capacity: u32,
    loss: bool,
    duplication: bool,
}

#[derive(Serialize)]
struct Property {
    kind: &'static str,
    name: String,
    participant: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    event: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
}

fn channel(from: &str, to: &str) -> String {
    format!("{from}-to-{to}")
}

fn model(rfc: u32, profile: &str, machine: &Machine) -> Result<Model> {
    ensure!(
        machine.participants.len() == 2,
        "machine {}: lowering needs exactly two participants, found {}",
        machine.name,
        machine.participants.len()
    );
    let names: Vec<&str> = machine
        .participants
        .iter()
        .map(|p| p.name.as_str())
        .collect();
    let peer_of = |name: &str| -> Result<&str> {
        let index = names
            .iter()
            .position(|n| *n == name)
            .with_context(|| format!("machine {}: no participant {name}", machine.name))?;
        Ok(names[1 - index])
    };
    let mut participants = Vec::new();
    for (index, participant) in machine.participants.iter().enumerate() {
        let me = names[index];
        let peer = names[1 - index];
        let mut transitions = Vec::new();
        for transition in &participant.transitions {
            let name = transition.name();
            if transition.implements.is_empty() {
                bail!(
                    "machine {}: transition {}.{name} implements no requirement or dialogue step",
                    machine.name,
                    participant.name
                );
            }
            let trigger = match &transition.on {
                Trigger::Input(input) => TriggerOut {
                    kind: "input",
                    name: Some(input.clone()),
                    channel: None,
                    message: None,
                },
                Trigger::Receive(message) => TriggerOut {
                    kind: "receive",
                    name: None,
                    channel: Some(channel(peer, me)),
                    message: Some(message.clone()),
                },
                Trigger::Timer(timer) => TriggerOut {
                    kind: "timer",
                    name: Some(timer.clone()),
                    channel: None,
                    message: None,
                },
            };
            let mut effects = Vec::new();
            for effect in &transition.effects {
                match effect {
                    Effect::Send(send) => effects.push(EffectOut {
                        kind: "send",
                        channel: Some(channel(me, peer)),
                        message: Some(send.message.clone()),
                        exchange: Some(literal(Value::Text("session".into()))),
                        logical: Some(literal(Value::Text(name.clone()))),
                        payload: Some(
                            send.payload
                                .iter()
                                .map(|(field, value)| (field.clone(), source(value)))
                                .collect(),
                        ),
                        ..EffectOut::default()
                    }),
                    Effect::Emit(event) => effects.push(EffectOut {
                        kind: "emit",
                        name: Some(event.clone()),
                        ..EffectOut::default()
                    }),
                    Effect::Close(true) => effects.push(EffectOut {
                        kind: "close",
                        ..EffectOut::default()
                    }),
                    Effect::Close(false) | Effect::Flush(false) => {}
                    Effect::Arm(arm) => effects.push(EffectOut {
                        kind: "arm",
                        timer: Some(arm.timer.clone()),
                        after: Some(Delay {
                            kind: "fixed",
                            millis: arm.millis,
                        }),
                        ..EffectOut::default()
                    }),
                    Effect::Cancel(timer) => effects.push(EffectOut {
                        kind: "cancel",
                        timer: Some(timer.clone()),
                        ..EffectOut::default()
                    }),
                    Effect::Flush(true) => effects.push(EffectOut {
                        kind: "flush",
                        channel: Some(channel(me, peer)),
                        ..EffectOut::default()
                    }),
                }
            }
            transitions.push(Transition {
                name,
                from: transition.from.clone(),
                to: transition.to.clone(),
                trigger,
                effects,
            });
        }
        participants.push(Participant {
            name: participant.name.clone(),
            initial: participant.initial.clone(),
            states: participant.states.clone(),
            inputs: participant
                .inputs
                .iter()
                .map(|input| Message {
                    name: input.name().to_owned(),
                    fields: fields(&input.fields()),
                })
                .collect(),
            timers: participant.timers.clone(),
            transitions,
        });
    }
    let mut properties = Vec::new();
    for property in &machine.properties {
        properties.push(match property {
            OverlayProperty::EventCount(count) => Property {
                kind: "event_count",
                name: count.name.clone(),
                participant: count.participant.clone(),
                event: Some(count.event.clone()),
                max: Some(count.max),
                channel: None,
            },
            OverlayProperty::FlushBeforeClose(flush) => Property {
                kind: "flush_before_close",
                name: flush.name.clone(),
                participant: flush.flush_before_close.clone(),
                event: None,
                max: None,
                channel: Some(channel(
                    &flush.flush_before_close,
                    peer_of(&flush.flush_before_close)?,
                )),
            },
        });
    }
    Ok(Model {
        format: "ess-protospec/1",
        name: format!("rfc{rfc}-{profile}-{}", machine.name),
        bounds: machine.bounds.unwrap_or(BOUNDS),
        participants,
        messages: machine
            .messages
            .iter()
            .map(|message| Message {
                name: message.name.clone(),
                fields: fields(&message.fields),
            })
            .collect(),
        channels: [(names[0], names[1]), (names[1], names[0])]
            .into_iter()
            .map(|(from, to)| Channel {
                name: channel(from, to),
                from: from.into(),
                to: to.into(),
                ordering: "fifo",
                capacity: CAPACITY,
                loss: false,
                duplication: false,
            })
            .collect(),
        properties,
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use rfcspec_model::{
        Send, Transition as OverlayTransition, drift, read_yaml, write_generated as write,
    };

    use super::*;

    fn profile() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/time")
    }

    fn overlay() -> Overlay {
        read_yaml(&profile().join("overlay.yaml")).unwrap()
    }

    fn parsed(lowered: &Generated) -> serde_json::Value {
        let value: serde_json::Value = rfcspec_model::from_yaml(&lowered.text).unwrap();
        value
    }

    fn transitions<'a>(
        model: &'a serde_json::Value,
        participant: &str,
    ) -> Vec<&'a serde_json::Value> {
        model["participants"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == participant)
            .unwrap()["transitions"]
            .as_array()
            .unwrap()
            .iter()
            .collect()
    }

    fn sends(transition: &serde_json::Value) -> Vec<String> {
        transition["effects"]
            .as_array()
            .map(|effects| {
                effects
                    .iter()
                    .filter(|e| e["kind"] == "send")
                    .map(|e| e["message"].as_str().unwrap().to_string())
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn lowering_writes_tcp_and_udp_models_and_every_transition_name_carries_an_id() {
        let overlay = overlay();
        let lowered = lower(&overlay, "time").unwrap();
        let files: Vec<&str> = lowered.iter().map(|l| l.file.as_str()).collect();
        assert_eq!(files, ["tcp.protospec.yaml", "udp.protospec.yaml"]);
        let requirement_ids: Vec<&str> =
            overlay.requirements.iter().map(|r| r.id.as_str()).collect();
        for file in &lowered {
            let model = parsed(file);
            assert_eq!(model["format"], "ess-protospec/1");
            for participant in ["client", "server"] {
                for transition in transitions(&model, participant) {
                    let name = transition["name"].as_str().unwrap();
                    let step = name.starts_with('D')
                        && name[1..]
                            .split('-')
                            .take(2)
                            .all(|n| n.parse::<u32>().is_ok());
                    let requirement = requirement_ids.iter().any(|id| name.starts_with(id));
                    assert!(step || requirement, "{name}");
                }
            }
        }
    }

    #[test]
    fn a_transition_implementing_nothing_is_refused() {
        let mut overlay = overlay();
        overlay.machines[0].participants[0].transitions[0]
            .implements
            .clear();
        let error = lower(&overlay, "time").unwrap_err().to_string();
        assert!(error.contains("implements no requirement"), "{error}");
    }

    #[test]
    fn each_model_declares_time_sent_at_most_once_for_the_server() {
        for file in lower(&overlay(), "time").unwrap() {
            let model = parsed(&file);
            assert_eq!(
                model["properties"],
                serde_json::json!([{"kind": "event_count", "name": "time-sent-once", "participant": "server", "event": "time-sent", "max": 1}])
            );
        }
    }

    #[test]
    fn tcp_server_can_close_without_sending_and_udp_server_replies_or_not() {
        let lowered = lower(&overlay(), "time").unwrap();
        let tcp = parsed(&lowered[0]);
        let unable = transitions(&tcp, "server")
            .into_iter()
            .find(|t| t["name"] == "R-TCP-3-unable-close")
            .unwrap();
        assert_eq!(
            (unable["from"].as_str(), unable["to"].as_str()),
            (Some("listening"), Some("closed"))
        );
        assert!(!sends(unable).contains(&"time".to_string()));
        assert!(
            unable["effects"]
                .as_array()
                .unwrap()
                .iter()
                .any(|e| e["kind"] == "close")
        );

        let udp = parsed(&lowered[1]);
        let server = transitions(&udp, "server");
        let from_listening: Vec<Vec<String>> = server
            .iter()
            .filter(|t| t["from"] == "listening")
            .map(|t| sends(t))
            .collect();
        assert_eq!(from_listening, [vec!["time".to_string()], vec![]]);
    }

    #[test]
    fn committed_models_equal_a_fresh_lowering() {
        let lowered = lower(&overlay(), "time").unwrap();
        assert_eq!(
            drift(&profile().join("generated"), &lowered),
            Vec::<PathBuf>::new()
        );
    }

    #[test]
    fn drift_names_a_missing_or_changed_model() {
        let lowered = lower(&overlay(), "time").unwrap();
        let out = tempfile::tempdir().unwrap();
        write(out.path(), &lowered).unwrap();
        assert!(drift(out.path(), &lowered).is_empty());
        fs::remove_file(out.path().join("udp.protospec.yaml")).unwrap();
        fs::write(
            out.path().join("tcp.protospec.yaml"),
            "format: ess-protospec/1\n",
        )
        .unwrap();
        assert_eq!(drift(out.path(), &lowered).len(), 2);
    }

    #[test]
    fn ess_explore_passes_both_committed_models_with_their_witnesses() {
        for machine in ["tcp", "udp"] {
            let report = explore(
                &profile().join(format!("generated/{machine}.protospec.yaml")),
                &profile().join(format!("witnesses/{machine}.inputs.json")),
            )
            .unwrap();
            assert_eq!(report["verdict"], "passed", "{machine}: {report:#}");
        }
    }

    #[test]
    fn a_mutant_tcp_server_that_sends_time_twice_violates_the_event_count() {
        let mut overlay = overlay();
        let server = overlay.machines[0]
            .participants
            .iter_mut()
            .find(|p| p.name == "server")
            .unwrap();
        server.transitions.push(OverlayTransition {
            implements: vec!["D1-3".into()],
            label: "mutant-send-time-again".into(),
            from: "sent".into(),
            to: "sent".into(),
            on: Trigger::Receive("fin".into()),
            effects: vec![
                Effect::Send(Send {
                    message: "time".into(),
                    payload: [("seconds".to_string(), 2629584000.into())].into(),
                }),
                Effect::Emit("time-sent".into()),
            ],
        });
        let out = tempfile::tempdir().unwrap();
        write(out.path(), &lower(&overlay, "time").unwrap()).unwrap();
        let report = explore(
            &out.path().join("tcp.protospec.yaml"),
            &profile().join("witnesses/tcp.inputs.json"),
        )
        .unwrap();
        assert_eq!(report["verdict"], "failed", "{report:#}");
        let reason = report["reason"].as_str().unwrap();
        assert!(
            reason.contains("property violated") && reason.contains("time-sent-once"),
            "{reason}"
        );
    }

    fn websocket() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/websocket")
    }

    fn websocket_overlay() -> Overlay {
        read_yaml(&websocket().join("overlay.yaml")).unwrap()
    }

    fn close_model() -> serde_json::Value {
        let lowered = lower(&websocket_overlay(), "websocket").unwrap();
        assert_eq!(lowered.len(), 1);
        assert_eq!(lowered[0].file, "close.protospec.yaml");
        parsed(&lowered[0])
    }

    fn participant<'a>(model: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
        model["participants"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["name"] == name)
            .unwrap()
    }

    fn effect_kinds(transition: &serde_json::Value) -> Vec<&str> {
        transition["effects"]
            .as_array()
            .map(|effects| {
                effects
                    .iter()
                    .map(|e| e["kind"].as_str().unwrap())
                    .collect()
            })
            .unwrap_or_default()
    }

    #[test]
    fn the_close_machine_lowers_states_inputs_timers_flush_and_properties() {
        let model = close_model();
        assert_eq!(model["name"], "rfc6455-websocket-close");
        assert_eq!(
            model["bounds"],
            serde_json::json!({"max_steps": 64, "max_states": 1024, "max_time_ms": 30000})
        );
        let client = participant(&model, "client");
        let server = participant(&model, "server");
        assert_eq!(
            client["states"],
            serde_json::json!(["open", "closing", "closed"])
        );
        assert_eq!(server["states"], serde_json::json!(["open", "closed"]));
        assert_eq!(
            client["inputs"],
            serde_json::json!([{"name": "close", "fields": [{"name": "code", "type": "Integer"}]}])
        );
        assert_eq!(client["timers"], serde_json::json!(["close-timeout"]));

        let start = transitions(&model, "client")
            .into_iter()
            .find(|t| t["trigger"]["kind"] == "input")
            .unwrap();
        assert_eq!(
            (start["from"].as_str(), start["to"].as_str()),
            (Some("open"), Some("closing"))
        );
        let send = &start["effects"][0];
        assert_eq!(send["kind"], "send");
        assert_eq!(send["message"], "close-frame");
        assert_eq!(
            send["payload"]["code"],
            serde_json::json!({"kind": "input", "field": "code"})
        );
        assert_eq!(
            send["payload"]["masked"],
            serde_json::json!({"kind": "literal", "value": true})
        );
        assert_eq!(
            start["effects"][1],
            serde_json::json!({"kind": "arm", "timer": "close-timeout", "after": {"kind": "fixed", "millis": 5000}})
        );

        let echoed = transitions(&model, "client")
            .into_iter()
            .find(|t| t["from"] == "closing" && t["trigger"]["kind"] == "receive")
            .unwrap();
        assert_eq!(
            echoed["effects"][0],
            serde_json::json!({"kind": "cancel", "timer": "close-timeout"})
        );
        let timeout = transitions(&model, "client")
            .into_iter()
            .find(|t| t["trigger"]["kind"] == "timer")
            .unwrap();
        assert_eq!(timeout["trigger"]["name"], "close-timeout");
        assert_eq!(effect_kinds(timeout), ["emit"]);

        let echo = &transitions(&model, "server")[0];
        assert_eq!(effect_kinds(echo), ["send", "flush", "close"]);
        assert_eq!(echo["effects"][1]["channel"], "server-to-client");

        assert_eq!(
            model["properties"],
            serde_json::json!([
                {"kind": "flush_before_close", "name": "server-flushes-close", "participant": "server", "channel": "server-to-client"},
                {"kind": "event_count", "name": "closed-once", "participant": "client", "event": "connection-closed", "max": 1}
            ])
        );
    }

    #[test]
    fn every_close_transition_name_leads_with_catalogue_requirement_ids() {
        let overlay = websocket_overlay();
        for machine in &overlay.machines {
            for participant in &machine.participants {
                for transition in &participant.transitions {
                    let name = transition.name();
                    assert!(!transition.implements.is_empty(), "{name}");
                    let lead = transition.implements.join("-");
                    assert!(name.starts_with(&format!("{lead}-")), "{name}");
                    for id in &transition.implements {
                        assert!(id.starts_with('R') && id.contains('-'), "{id}");
                        assert!(overlay.requirements.iter().any(|r| &r.id == id), "{id}");
                    }
                }
            }
        }
        let mut nothing = websocket_overlay();
        nothing.machines[0].participants[1].transitions[0]
            .implements
            .clear();
        let error = lower(&nothing, "websocket").unwrap_err().to_string();
        assert!(error.contains("implements no requirement"), "{error}");
    }

    #[test]
    fn the_committed_close_model_equals_a_fresh_lowering() {
        let lowered = lower(&websocket_overlay(), "websocket").unwrap();
        assert_eq!(
            drift(&websocket().join("generated"), &lowered),
            Vec::<PathBuf>::new()
        );
        assert_eq!(
            fs::read_to_string(websocket().join("witnesses/close.inputs.json")).unwrap(),
            "[\n  {\"kind\":\"input\",\"participant\":\"client\",\"name\":\"close\",\"payload\":{\"code\":1000}}\n]\n"
        );
    }

    #[test]
    fn ess_explore_passes_the_committed_close_model() {
        let report = explore(
            &websocket().join("generated/close.protospec.yaml"),
            &websocket().join("witnesses/close.inputs.json"),
        )
        .unwrap();
        assert_eq!(report["verdict"], "passed", "{report:#}");
    }

    #[test]
    fn a_client_that_closes_on_the_closing_timeout_is_inconclusive_gap_g9() {
        let mut overlay = websocket_overlay();
        let client = &mut overlay.machines[0].participants[0];
        let timeout = client
            .transitions
            .iter_mut()
            .find(|t| t.on == Trigger::Timer("close-timeout".into()))
            .unwrap();
        timeout.effects.push(Effect::Close(true));
        let out = tempfile::tempdir().unwrap();
        write(out.path(), &lower(&overlay, "websocket").unwrap()).unwrap();
        let report = explore(
            &out.path().join("close.protospec.yaml"),
            &websocket().join("witnesses/close.inputs.json"),
        )
        .unwrap();
        assert_eq!(report["verdict"], "inconclusive", "{report:#}");
        assert_eq!(
            report["reason"],
            "dead end with outstanding queue, timer or flush obligations"
        );
    }
}
