//! Adversarial cases for story:ntp-poll-machine: the `model-` exemption from the requirement
//! link, and poll mutants that the exploration gate lets through and the lowering pins reject.

use std::{fs, path::PathBuf};

use rfcspec_ess::{explore, lower};
use rfcspec_model::{Arm, Effect, Interval, Overlay, Transition, read_yaml};

mod poll_pins;

fn profile(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../profiles")
        .join(name)
}

fn overlay(name: &str) -> Overlay {
    read_yaml(&profile(name).join("overlay.yaml")).unwrap()
}

fn poll_transition<'a>(overlay: &'a mut Overlay, label: &str) -> &'a mut Transition {
    overlay
        .machines
        .iter_mut()
        .find(|m| m.name == "poll")
        .unwrap()
        .participants
        .iter_mut()
        .flat_map(|p| p.transitions.iter_mut())
        .find(|t| t.label == label)
        .unwrap_or_else(|| panic!("no poll transition {label}"))
}

fn explore_poll(overlay: &Overlay) -> serde_json::Value {
    let out = tempfile::tempdir().unwrap();
    for generated in lower(overlay, "ntp").unwrap() {
        fs::write(out.path().join(&generated.file), &generated.text).unwrap();
    }
    let inputs = out.path().join("poll.inputs.json");
    fs::write(&inputs, "[]\n").unwrap();
    explore(&out.path().join("poll.protospec.yaml"), &inputs).unwrap()
}

/// The exemption is decided by the label alone, in every profile. A transition that sends a
/// protocol message is protocol behaviour, not a model construct, and must still lead with the
/// requirement it implements; the poll horizon (cancel only) and discard (no effect) stay allowed.
#[test]
fn a_transition_that_sends_a_message_is_refused_without_a_requirement_even_when_labelled_model() {
    let mut ntp = overlay("ntp");
    let real = poll_transition(&mut ntp, "poll-reachable");
    real.implements.clear();
    real.label = "model-poll-reachable".into();
    let lowered = lower(&ntp, "ntp");
    assert!(
        lowered
            .as_ref()
            .is_err_and(|e| e.to_string().contains("implements no requirement")),
        "ntp: a sending transition labelled model- lowered without a requirement link"
    );

    let mut websocket = overlay("websocket");
    let real = websocket
        .machines
        .iter_mut()
        .flat_map(|m| m.participants.iter_mut())
        .flat_map(|p| p.transitions.iter_mut())
        .find(|t| t.effects.iter().any(|e| matches!(e, Effect::Send(_))))
        .unwrap();
    let original = real.label.clone();
    real.implements.clear();
    real.label = format!("model-{original}");
    let lowered = lower(&websocket, "websocket");
    assert!(
        lowered
            .as_ref()
            .is_err_and(|e| e.to_string().contains("implements no requirement")),
        "websocket: transition {original} relabelled model- lowered without a requirement link"
    );
}

/// RFC 5905 section 9.2 (R9.2-REACH-1, status satisfied by `poll.client.R9.2-REACH-1-poll-emptied`):
/// a server whose register has emptied is unreachable. The mutant keeps the client reachable on
/// that poll, so it never becomes unreachable and never backs off. Names, arms and links are
/// unchanged. Exploration passes it: `event_count` has only a maximum, so no property can demand
/// that a state is reached (design gap G14). The transition pin of the lowering test rejects it.
#[test]
fn a_mutant_whose_register_never_empties_passes_exploration_and_the_transition_pin_rejects_it() {
    let mut mutant = overlay("ntp");
    poll_transition(&mut mutant, "poll-emptied").to = "reachable".into();
    assert_eq!(explore_poll(&mutant)["verdict"], "passed");
    assert_ne!(poll_pins::transition_pins(&mutant), poll_pins::TRANSITIONS);
}

/// RFC 5905 section 9.2: a valid packet makes the server reachable. The mutant stays unreachable
/// on a response from the unreachable state, again with names, arms and links unchanged.
/// Exploration passes it (design gap G14); the transition pin of the lowering test rejects it.
#[test]
fn a_mutant_whose_response_never_makes_the_server_reachable_passes_exploration_and_the_transition_pin_rejects_it()
 {
    let mut mutant = overlay("ntp");
    poll_transition(&mut mutant, "response-unreachable").to = "unreachable".into();
    assert_eq!(explore_poll(&mutant)["verdict"], "passed");
    assert_ne!(poll_pins::transition_pins(&mutant), poll_pins::TRANSITIONS);
}

/// What the 48 s horizon leaves of `backoff-doubles-interval`: two backoff polls are seen only
/// when the first two backoff intervals sum to 32 s or less, so exploration checks that the first
/// backoff interval is not 16 s. A client that backs off to a fixed 17 s, which never doubles,
/// passes exploration; the arm pin of the lowering test, which checks the doubling and the
/// ceiling, rejects it.
#[test]
fn a_mutant_that_backs_off_to_a_fixed_17_s_passes_exploration_and_the_arm_pin_rejects_it() {
    let mut mutant = overlay("ntp");
    for label in ["poll-backoff-start", "poll-backoff"] {
        for effect in &mut poll_transition(&mut mutant, label).effects {
            if let Effect::Arm(Arm { timer, interval }) = effect
                && timer == "poll"
            {
                *interval = Interval::Fixed { millis: 17000 };
            }
        }
    }
    assert_eq!(explore_poll(&mutant)["verdict"], "passed");
    assert_ne!(poll_pins::arm_pins(&mutant), poll_pins::expected_arms());
}
