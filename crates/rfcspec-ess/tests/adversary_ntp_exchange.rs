//! Adversarial cases for story:ntp-exchange-machine: the lowered `exchange` model of
//! profiles/ntp, explored with witnesses other than the committed one, and single-transition
//! mutants the committed witness may not distinguish.

use std::{fs, path::PathBuf};

use rfcspec_ess::{explore, lower};
use rfcspec_model::{Effect, Overlay, Transition, Value, read_yaml};

fn ntp() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/ntp")
}

fn overlay() -> Overlay {
    read_yaml(&ntp().join("overlay.yaml")).unwrap()
}

/// Lower `overlay` into a scratch directory and explore it with `witness` as its inputs file.
fn explore_with(overlay: &Overlay, witness: &str) -> serde_json::Value {
    let out = tempfile::tempdir().unwrap();
    for generated in lower(overlay, "ntp").unwrap() {
        fs::write(out.path().join(&generated.file), &generated.text).unwrap();
    }
    let inputs = out.path().join("exchange.inputs.json");
    fs::write(&inputs, witness).unwrap();
    explore(&out.path().join("exchange.protospec.yaml"), &inputs).unwrap()
}

fn committed_witness() -> String {
    fs::read_to_string(ntp().join("witnesses/exchange.inputs.json")).unwrap()
}

fn client_transition<'a>(overlay: &'a mut Overlay, label: &str) -> &'a mut Transition {
    overlay.machines[0]
        .participants
        .iter_mut()
        .find(|p| p.name == "client")
        .unwrap()
        .transitions
        .iter_mut()
        .find(|t| t.label == label)
        .unwrap()
}

fn drop_xmt_reset(transition: &mut Transition) {
    let before = transition.effects.len();
    transition.effects.retain(
        |e| !matches!(e, Effect::Set(set) if set.field == "xmt" && set.value == Value::Integer(0)),
    );
    assert_eq!(transition.effects.len(), before - 1, "{transition:?}");
}

/// RFC 5905 Appendix A.5.1.1 (`packet()`, source lines 4545-4549): a packet whose origin
/// timestamp is zero is unsynchronized and never accepted, which is what makes zeroing `xmt`
/// after the bogus check (R8-REPLAY-1, lines 1593-1595) a replay protection at all. The model's
/// guard `org = xmt` accepts an origin of 0 once `xmt` is zero, so a server packet with origin 0
/// is a third valid packet in a run of two polls.
#[test]
fn a_server_packet_with_origin_zero_is_never_a_valid_packet() {
    let report = explore_with(
        &overlay(),
        "[\n  {\"kind\":\"input\",\"participant\":\"server\",\"name\":\"replay\",\"payload\":{\"org\":0}}\n]\n",
    );
    assert_eq!(report["verdict"], "passed", "{report:#}");
}

/// The committed witness replays origin 1 only, so the property never sees a duplicate of the
/// second poll's response. A mutant that keeps `xmt` after the check in `second` alone (R8-REPLAY-1
/// dropped for the second poll) must still fail on `one-valid-packet-per-poll` under the witness
/// `task check` explores with.
#[test]
fn a_mutant_that_keeps_xmt_after_the_second_check_fails_under_the_committed_witness() {
    let mut mutant = overlay();
    drop_xmt_reset(client_transition(&mut mutant, "accept-second"));
    let report = explore_with(&mutant, &committed_witness());
    assert_eq!(report["verdict"], "failed", "{report:#}");
    let reason = report["reason"].as_str().unwrap();
    assert!(reason.contains("one-valid-packet-per-poll"), "{reason}");
}

/// Control for the case above: the same mutant does fail once the witness also replays the
/// second poll's origin, so the gap is the witness, not the property.
#[test]
fn the_second_check_mutant_fails_when_the_witness_replays_origin_two() {
    let mut mutant = overlay();
    drop_xmt_reset(client_transition(&mut mutant, "accept-second"));
    let report = explore_with(
        &mutant,
        "[\n  {\"kind\":\"input\",\"participant\":\"server\",\"name\":\"replay\",\"payload\":{\"org\":2}}\n]\n",
    );
    assert_eq!(report["verdict"], "failed", "{report:#}");
}

/// Control: the committed model passes a witness replaying the second poll's origin.
#[test]
fn the_committed_model_passes_a_replay_of_origin_two() {
    let report = explore_with(
        &overlay(),
        "[\n  {\"kind\":\"input\",\"participant\":\"server\",\"name\":\"replay\",\"payload\":{\"org\":2}}\n]\n",
    );
    assert_eq!(report["verdict"], "passed", "{report:#}");
}

/// The other single-transition mutants the committed witness does distinguish.
#[test]
fn single_transition_mutants_of_the_first_poll_fail_under_the_committed_witness() {
    let mut no_reset = overlay();
    drop_xmt_reset(client_transition(&mut no_reset, "accept-first"));
    let mut no_guard_first = overlay();
    client_transition(&mut no_guard_first, "accept-first")
        .guards
        .clear();
    let mut no_guard_second = overlay();
    client_transition(&mut no_guard_second, "accept-second")
        .guards
        .clear();
    for (name, mutant) in [
        ("no reset in first", no_reset),
        ("no guard in first", no_guard_first),
        ("no guard in second", no_guard_second),
    ] {
        let report = explore_with(&mutant, &committed_witness());
        assert_eq!(report["verdict"], "failed", "{name}: {report:#}");
    }
}
