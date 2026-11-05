//! Pins on the lowered `poll` model of profiles/ntp that exploration cannot give: the (from, to,
//! trigger) of every transition, and every timer arm. `event_count` has only a maximum, so a
//! mutant that never reaches a state, or arms a timer for a different interval that still keeps
//! the count low within the 48 s horizon, passes `ess protocol explore`; these comparisons reject
//! it. Shared by `ntp_poll.rs` and `adversary_ntp_poll.rs`.

use rfcspec_ess::lower;
use rfcspec_model::Overlay;
use serde_json::{Value, json};

/// The lowered `poll` model of `overlay`.
pub fn lowered(overlay: &Overlay) -> Value {
    let generated = lower(overlay, "ntp")
        .unwrap()
        .into_iter()
        .find(|g| g.file == "poll.protospec.yaml")
        .expect("lower writes poll.protospec.yaml");
    rfcspec_model::from_yaml(&generated.text).unwrap()
}

fn transitions(model: &Value) -> impl Iterator<Item = (&str, &Value)> {
    model["participants"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|p| {
            let name = p["name"].as_str().unwrap();
            p["transitions"]
                .as_array()
                .unwrap()
                .iter()
                .map(move |t| (name, t))
        })
}

/// Every transition as `participant.name: from -> to on trigger`, the trigger being
/// `input:<name>`, `timer:<name>` or `receive:<message>`.
pub fn transition_pins(overlay: &Overlay) -> Vec<String> {
    transitions(&lowered(overlay))
        .map(|(participant, t)| {
            let trigger = &t["trigger"];
            let on = match trigger["kind"].as_str().unwrap() {
                "receive" => format!("receive:{}", trigger["message"].as_str().unwrap()),
                kind => format!("{kind}:{}", trigger["name"].as_str().unwrap()),
            };
            format!(
                "{participant}.{}: {} -> {} on {on}",
                t["name"].as_str().unwrap(),
                t["from"].as_str().unwrap(),
                t["to"].as_str().unwrap()
            )
        })
        .collect()
}

pub const TRANSITIONS: [&str; 14] = [
    "client.R9.2-REACH-1-mobilize: idle -> unreachable on input:mobilize",
    "client.R9.2-REACH-1-poll-reachable: reachable -> reachable on timer:poll",
    "client.R9.2-REACH-1-poll-emptied: reachable -> unreachable on timer:poll",
    "client.R9.2-REACH-1-poll-unreachable: unreachable -> unreachable on timer:poll",
    "client.R13-BACKOFF-1-poll-backoff-start: unreachable -> backoff on timer:poll",
    "client.R13-BACKOFF-1-poll-backoff: backoff -> backoff on timer:poll",
    "client.R9.2-REACH-1-response-unreachable: unreachable -> reachable on receive:response",
    "client.R9.2-REACH-1-response-reachable: reachable -> reachable on receive:response",
    "client.R9.2-REACH-1-response-backoff: backoff -> reachable on receive:response",
    "client.model-horizon-unreachable: unreachable -> ended on timer:horizon",
    "client.model-horizon-reachable: reachable -> ended on timer:horizon",
    "client.model-horizon-backoff: backoff -> ended on timer:horizon",
    "client.model-horizon-discard: ended -> ended on receive:response",
    "server.R9.2-FXMIT-1-answer: listening -> listening on receive:request",
];

/// Every timer arm as (transition name, timer, `after`), in model order.
pub fn arm_pins(overlay: &Overlay) -> Vec<(String, String, Value)> {
    let mut arms = Vec::new();
    for (_, t) in transitions(&lowered(overlay)) {
        for effect in t["effects"].as_array().into_iter().flatten() {
            if effect["kind"] == "arm" {
                arms.push((
                    t["name"].as_str().unwrap().to_owned(),
                    effect["timer"].as_str().unwrap().to_owned(),
                    effect["after"].clone(),
                ));
            }
        }
    }
    arms
}

/// RFC 5905 Figure 6 lines 909-910: MINPOLL 4 (16 s) and MAXPOLL 17 (36 h). The two backoff
/// transitions double the last interval up to 2^MAXPOLL s; every other `poll` arm is 2^MINPOLL s;
/// `horizon` is the 48 s model horizon.
pub fn expected_arms() -> Vec<(String, String, Value)> {
    let fixed = json!({"kind": "fixed", "millis": 16000});
    let backoff =
        json!({"kind": "backoff", "timer": "poll", "factor": 2, "ceiling_ms": 131_072_000});
    [
        ("R9.2-REACH-1-mobilize", "poll", fixed.clone()),
        (
            "R9.2-REACH-1-mobilize",
            "horizon",
            json!({"kind": "fixed", "millis": 48000}),
        ),
        ("R9.2-REACH-1-poll-reachable", "poll", fixed.clone()),
        ("R9.2-REACH-1-poll-emptied", "poll", fixed.clone()),
        ("R9.2-REACH-1-poll-unreachable", "poll", fixed.clone()),
        ("R13-BACKOFF-1-poll-backoff-start", "poll", backoff.clone()),
        ("R13-BACKOFF-1-poll-backoff", "poll", backoff),
        ("R9.2-REACH-1-response-backoff", "poll", fixed),
    ]
    .into_iter()
    .map(|(name, timer, after)| (name.to_owned(), timer.to_owned(), after))
    .collect()
}
