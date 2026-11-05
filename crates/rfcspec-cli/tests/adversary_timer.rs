//! Adversarial cases for the capped backoff timer and lossy channels (story:ntp-timer-model):
//! what `rfcspec check` admits must be what `ess protocol explore` runs, and an `arm` with both
//! keys is a parse error.

use std::{fs, path::PathBuf};

use rfcspec_model::{Arm, Backoff, Effect, Interval, Overlay, from_yaml, read_yaml};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn websocket() -> Overlay {
    read_yaml(&repository().join("profiles/websocket/overlay.yaml")).unwrap()
}

fn problems(overlay: &Overlay) -> Vec<String> {
    let source =
        fs::read_to_string(repository().join("profiles/websocket/sources/rfc6455.txt")).unwrap();
    let catalogue = rfcspec_extract::extract(&rfcspec_doc::read(6455, "digest", &source));
    rfcspec_extract::check(overlay, &catalogue, &source)
}

/// The committed WebSocket overlay with the close client's only arm of `close-timeout` replaced.
fn close_arm(interval: Interval) -> Overlay {
    let mut overlay = websocket();
    let close = overlay
        .machines
        .iter_mut()
        .find(|m| m.name == "close")
        .unwrap();
    let Effect::Arm(arm) = &mut close.participants[0].transitions[0].effects[1] else {
        panic!("the second effect arms");
    };
    *arm = Arm {
        timer: "close-timeout".into(),
        interval,
    };
    overlay
}

/// Lower the overlay and explore its `close` model with the committed close witness inputs.
fn explore_close(overlay: &Overlay) -> serde_json::Value {
    let out = tempfile::tempdir().unwrap();
    rfcspec_model::write_generated(
        out.path(),
        &rfcspec_ess::lower(overlay, "websocket").unwrap(),
    )
    .unwrap();
    rfcspec_ess::explore(
        &out.path().join("close.protospec.yaml"),
        &repository().join("profiles/websocket/witnesses/close.inputs.json"),
    )
    .unwrap()
}

/// ESS 0.86.0 executes a backoff as the timer's last armed interval times the factor and fails
/// with "backoff has no prior interval" when the timer was never armed before. The close client
/// arms `close-timeout` exactly once; with that arm a backoff, the check must refuse it, or the
/// lowered model must explore.
#[test]
fn a_backoff_of_a_timer_never_armed_before_is_refused_by_the_check_or_explores() {
    let overlay = close_arm(Interval::Backoff(Backoff {
        factor: 2,
        ceiling_ms: 8000,
    }));
    let refused = problems(&overlay);
    let report = explore_close(&overlay);
    assert!(
        !refused.is_empty() || report["verdict"] == "passed",
        "rfcspec check admitted it ({refused:?}) and ess protocol explore reported:\n{report:#}"
    );
}

/// The story: "both keys together, or neither, is a parse error". A null is still the key.
#[test]
fn an_arm_naming_both_keys_is_a_parse_error_even_when_one_is_null() {
    let admitted: Vec<&str> = [
        "{timer: t, millis: null, backoff: {factor: 2, ceiling_ms: 4}}",
        "{timer: t, millis: 1000, backoff: null}",
    ]
    .into_iter()
    .filter(|text| from_yaml::<Arm>(text).is_ok())
    .collect();
    assert!(admitted.is_empty(), "parsed: {admitted:?}");
}

/// The unit's machine `retry` (crates/rfcspec-ess/src/lib.rs, BACKOFF_MACHINE).
const RETRY: &str = "name: retry
transport: udp
bounds: {max_steps: 32, max_states: 256, max_time_ms: 8000}
messages: []
participants:
  - name: client
    initial: idle
    states: [idle, waiting, backing, done]
    inputs: [start, stop]
    timers: [t]
    transitions:
      - implements: [T-1]
        label: start
        from: idle
        to: waiting
        on: {input: start}
        effects: [{arm: {timer: t, millis: 1000}}]
      - implements: [T-2]
        label: back-off
        from: waiting
        to: backing
        on: {timer: t}
        effects: [{arm: {timer: t, backoff: {factor: 2, ceiling_ms: 4000}}}]
      - implements: [T-3]
        label: stop
        from: backing
        to: done
        on: {input: stop}
        effects: [{cancel: t}]
  - name: server
    initial: idle
    states: [idle]
    transitions: []
properties: []
";

fn explore_retry(machine: &str) -> serde_json::Value {
    let overlay = Overlay {
        format: rfcspec_model::OVERLAY_FORMAT.into(),
        rfc: 1,
        kind: None,
        requirements: vec![],
        layouts: vec![],
        machines: vec![from_yaml(machine).unwrap()],
        findings: vec![],
        decisions: vec![],
    };
    let out = tempfile::tempdir().unwrap();
    rfcspec_model::write_generated(out.path(), &rfcspec_ess::lower(&overlay, "test").unwrap())
        .unwrap();
    let inputs = out.path().join("retry.inputs.json");
    fs::write(
        &inputs,
        "[{\"kind\":\"input\",\"participant\":\"client\",\"name\":\"start\",\"payload\":{}},{\"kind\":\"input\",\"participant\":\"client\",\"name\":\"stop\",\"payload\":{}}]\n",
    )
    .unwrap();
    rfcspec_ess::explore(&out.path().join("retry.protospec.yaml"), &inputs).unwrap()
}

/// The unit's exploration test asserts only that the uncancelled machine does not pass; a model
/// ESS cannot run at all ("no transitions exercised") also does not pass. The uncancelled machine
/// is inconclusive because the timer is still armed at the dead end, after running its back-off.
#[test]
fn the_uncancelled_retry_is_inconclusive_for_its_outstanding_timer() {
    let uncancelled = explore_retry(&RETRY.replace("effects: [{cancel: t}]", ""));
    assert_eq!(uncancelled["verdict"], "inconclusive", "{uncancelled:#}");
    assert_eq!(
        uncancelled["reason"], "dead end with outstanding queue, timer or flush obligations",
        "{uncancelled:#}"
    );
    assert_eq!(uncancelled["explored_transitions"], 7, "{uncancelled:#}");
}
