//! story:ntp-poll-machine: the `poll` machine of profiles/ntp (RFC 5905 sections 9.2 and 13),
//! lowered from the reviewed overlay, explored with its committed witness, and a mutant that polls
//! at a fixed interval where the RFC backs off.

use std::{fs, path::PathBuf};

use rfcspec_ess::{explore, lower};
use rfcspec_model::{Arm, Effect, Interval, Machine, Overlay, read_yaml};

mod poll_pins;

fn ntp() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/ntp")
}

fn overlay() -> Overlay {
    read_yaml(&ntp().join("overlay.yaml")).unwrap()
}

fn poll(overlay: &mut Overlay) -> &mut Machine {
    overlay
        .machines
        .iter_mut()
        .find(|m| m.name == "poll")
        .expect("the ntp overlay has a machine `poll`")
}

/// Lower `overlay` into a scratch directory and explore its `poll` model with `witness`.
fn explore_poll(overlay: &Overlay, witness: &str) -> serde_json::Value {
    let out = tempfile::tempdir().unwrap();
    for generated in lower(overlay, "ntp").unwrap() {
        fs::write(out.path().join(&generated.file), &generated.text).unwrap();
    }
    let inputs = out.path().join("poll.inputs.json");
    fs::write(&inputs, witness).unwrap();
    explore(&out.path().join("poll.protospec.yaml"), &inputs).unwrap()
}

fn committed_witness() -> String {
    fs::read_to_string(ntp().join("witnesses/poll.inputs.json")).unwrap()
}

fn lowered_poll() -> serde_json::Value {
    let generated = lower(&overlay(), "ntp")
        .unwrap()
        .into_iter()
        .find(|g| g.file == "poll.protospec.yaml")
        .expect("lower writes poll.protospec.yaml");
    rfcspec_model::from_yaml(&generated.text).unwrap()
}

#[test]
fn the_witness_is_an_empty_list() {
    assert_eq!(committed_witness(), "[]\n");
}

#[test]
fn the_poll_model_has_its_name_bounds_lossy_channels_messages_and_property() {
    let model = lowered_poll();
    assert_eq!(model["name"], "rfc5905-ntp-poll");
    assert_eq!(model["bounds"]["max_steps"], 128);
    assert_eq!(model["bounds"]["max_states"], 8192);
    assert_eq!(model["bounds"]["max_time_ms"], 131_072_000);
    for channel in model["channels"].as_array().unwrap() {
        assert_eq!(channel["capacity"], 4, "{channel}");
        assert_eq!(channel["loss"], true, "{channel}");
        assert_eq!(channel["duplication"], false, "{channel}");
    }
    let messages: Vec<(&str, usize)> = model["messages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|m| {
            (
                m["name"].as_str().unwrap(),
                m["fields"].as_array().unwrap().len(),
            )
        })
        .collect();
    assert_eq!(messages, [("request", 0), ("response", 0)]);
    let client = &model["participants"][0];
    assert_eq!(client["name"], "client");
    assert_eq!(
        client["states"],
        serde_json::json!(["idle", "unreachable", "reachable", "backoff", "ended"])
    );
    assert_eq!(client["timers"], serde_json::json!(["poll", "horizon"]));
    assert_eq!(
        client["inputs"],
        serde_json::json!([{"name": "mobilize", "fields": []}])
    );
    let server = &model["participants"][1];
    assert_eq!(server["name"], "server");
    assert_eq!(server["states"], serde_json::json!(["listening"]));
    assert_eq!(
        model["properties"],
        serde_json::json!([{
            "kind": "event_count",
            "name": "backoff-doubles-interval",
            "participant": "client",
            "event": "backoff-poll",
            "max": 1
        }])
    );
}

/// RFC 5905 Figure 6 lines 909-910: MINPOLL 4 (16 s) and MAXPOLL 17 (36 h). The two backoff
/// transitions double the interval up to 2^MAXPOLL s; every other `poll` arm is 2^MINPOLL s.
#[test]
fn backoff_transitions_lower_to_a_capped_backoff_and_the_others_to_a_fixed_16_s() {
    let model = lowered_poll();
    let mut arms: Vec<(String, String, serde_json::Value)> = Vec::new();
    for transition in model["participants"][0]["transitions"].as_array().unwrap() {
        for effect in transition["effects"].as_array().into_iter().flatten() {
            if effect["kind"] == "arm" {
                arms.push((
                    transition["name"].as_str().unwrap().to_owned(),
                    effect["timer"].as_str().unwrap().to_owned(),
                    effect["after"].clone(),
                ));
            }
        }
    }
    let backoff = serde_json::json!({
        "kind": "backoff", "timer": "poll", "factor": 2, "ceiling_ms": 131_072_000
    });
    let fixed = serde_json::json!({"kind": "fixed", "millis": 16000});
    let mut backoffs = 0;
    let mut fixeds = 0;
    for (name, timer, after) in &arms {
        if timer == "horizon" {
            assert_eq!(
                after,
                &serde_json::json!({"kind": "fixed", "millis": 48000}),
                "{name}"
            );
        } else if name.starts_with("R13-BACKOFF-1-") {
            assert_eq!(after, &backoff, "{name}");
            backoffs += 1;
        } else {
            assert_eq!(after, &fixed, "{name}");
            fixeds += 1;
        }
    }
    assert_eq!((backoffs, fixeds), (2, 5), "{arms:#?}");
}

/// Exploration cannot tell a client that never reaches a state from the reviewed one
/// (`event_count` has only a maximum, design gap G14), so the lowering pins the (from, to,
/// trigger) of every transition.
#[test]
fn every_poll_transition_lowers_to_its_reviewed_from_to_and_trigger() {
    assert_eq!(
        poll_pins::transition_pins(&overlay()),
        poll_pins::TRANSITIONS
    );
}

/// Within the 48 s horizon exploration checks only that the first backoff interval is not 16 s;
/// the doubling (factor 2) and the 2^MAXPOLL s ceiling are checked here, on the lowered arms.
#[test]
fn every_poll_timer_arm_lowers_to_its_reviewed_interval() {
    assert_eq!(poll_pins::arm_pins(&overlay()), poll_pins::expected_arms());
}

#[test]
fn the_horizon_transitions_implement_nothing_and_are_labelled_model_horizon() {
    let model = lowered_poll();
    let mut horizon = 0;
    for transition in model["participants"][0]["transitions"].as_array().unwrap() {
        let name = transition["name"].as_str().unwrap();
        if transition["trigger"]["kind"] == "timer" && transition["trigger"]["name"] == "horizon" {
            assert!(name.starts_with("model-horizon"), "{name}");
            assert_eq!(transition["to"], "ended", "{name}");
            horizon += 1;
        } else if name.starts_with("model-horizon") {
            assert_eq!(transition["from"], "ended", "{name}");
        } else {
            assert!(
                name.starts_with('R'),
                "{name} does not lead with a requirement id"
            );
        }
    }
    assert_eq!(horizon, 3);
}

#[test]
fn the_committed_poll_model_passes_exploration_with_its_witness() {
    let report = explore_poll(&overlay(), &committed_witness());
    assert_eq!(report["verdict"], "passed", "{report:#}");
}

/// The mutant of the acceptance line: the backoff transitions arm `poll` fixed 16000 ms, so a
/// backed-off client polls at 32 s and 48 s within the horizon.
#[test]
fn a_mutant_whose_backoff_transitions_poll_at_a_fixed_16_s_fails_backoff_doubles_interval() {
    let mut mutant = overlay();
    let mut mutated = 0;
    for participant in &mut poll(&mut mutant).participants {
        for transition in &mut participant.transitions {
            if !transition.implements.iter().any(|id| id == "R13-BACKOFF-1") {
                continue;
            }
            for effect in &mut transition.effects {
                if let Effect::Arm(Arm { timer, interval }) = effect
                    && timer == "poll"
                {
                    *interval = Interval::Fixed { millis: 16000 };
                    mutated += 1;
                }
            }
        }
    }
    assert_eq!(mutated, 2);
    let report = explore_poll(&mutant, &committed_witness());
    assert_eq!(report["verdict"], "failed", "{report:#}");
    let reason = report["reason"].as_str().unwrap();
    assert!(reason.contains("backoff-doubles-interval"), "{reason}");
}
