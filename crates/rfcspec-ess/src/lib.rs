#![forbid(unsafe_code)]
//! S5: lower the overlay's machines to `ess-protospec/1`, one model per machine (one per
//! transport). Each machine has two participants joined by one FIFO channel in each direction; a
//! transition's lowered name is the requirement and dialogue-step ids it implements, then its label.
//!
//! The models are validated and explored by the `ess` command, never by this crate.

use std::{collections::BTreeMap, path::Path, process::Command};

use anyhow::{Context, Result, bail, ensure};
use rfcspec_model::{Effect, Generated, Machine, Overlay, Trigger, to_yaml};
use serde::Serialize;

/// The exploration bounds of every lowered model.
const BOUNDS: Bounds = Bounds {
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
struct Bounds {
    max_steps: u32,
    max_states: u32,
    max_time_ms: u32,
}

#[derive(Serialize)]
struct Participant {
    name: String,
    initial: String,
    states: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    inputs: Vec<Message>,
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

#[derive(Serialize)]
struct EffectOut {
    kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    channel: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    exchange: Option<Literal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    logical: Option<Literal>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<BTreeMap<String, Literal>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    name: Option<String>,
}

#[derive(Serialize)]
struct Literal {
    kind: &'static str,
    value: Value,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Value {
    Integer(i64),
    Text(String),
}

fn literal(value: Value) -> Literal {
    Literal {
        kind: "literal",
        value,
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
    event: String,
    max: u32,
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
                                .map(|(field, value)| {
                                    (field.clone(), literal(Value::Integer(*value)))
                                })
                                .collect(),
                        ),
                        name: None,
                    }),
                    Effect::Emit(event) => effects.push(EffectOut {
                        kind: "emit",
                        channel: None,
                        message: None,
                        exchange: None,
                        logical: None,
                        payload: None,
                        name: Some(event.clone()),
                    }),
                    Effect::Close(true) => effects.push(EffectOut {
                        kind: "close",
                        channel: None,
                        message: None,
                        exchange: None,
                        logical: None,
                        payload: None,
                        name: None,
                    }),
                    Effect::Close(false) => {}
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
                    name: input.clone(),
                    fields: vec![],
                })
                .collect(),
            transitions,
        });
    }
    Ok(Model {
        format: "ess-protospec/1",
        name: format!("rfc{rfc}-{profile}-{}", machine.name),
        bounds: BOUNDS,
        participants,
        messages: machine
            .messages
            .iter()
            .map(|message| Message {
                name: message.name.clone(),
                fields: message
                    .fields
                    .iter()
                    .map(|(name, kind)| FieldType {
                        name: name.clone(),
                        kind: kind.clone(),
                    })
                    .collect(),
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
        properties: machine
            .properties
            .iter()
            .map(|property| Property {
                kind: "event_count",
                name: property.name.clone(),
                participant: property.participant.clone(),
                event: property.event.clone(),
                max: property.max,
            })
            .collect(),
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
                    payload: [("seconds".to_string(), 2629584000)].into(),
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
}
