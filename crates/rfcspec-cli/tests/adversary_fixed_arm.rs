//! Adversarial cases for the fixed-arm bounds check (story:fixed-arm-bounds-check): what
//! `rfcspec check` admits for a fixed `millis` arm must be what ESS admits once the overlay is
//! lowered, at the boundaries 0, 1, `max_time_ms` and `max_time_ms` + 1, for declared bounds and for
//! the lowering default.

use std::{fs, path::PathBuf};

use rfcspec_model::{Arm, Effect, Interval, Overlay, read_yaml};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn problems(overlay: &Overlay) -> Vec<String> {
    let source =
        fs::read_to_string(repository().join("profiles/websocket/sources/rfc6455.txt")).unwrap();
    let catalogue = rfcspec_extract::extract(&rfcspec_doc::read(6455, "digest", &source));
    rfcspec_extract::check(overlay, &catalogue, &source)
}

/// The committed WebSocket overlay with the close client's fixed arm of `close-timeout` set to
/// `millis`, and the close machine's bounds dropped when `default_bounds`.
fn close_fixed(millis: u64, default_bounds: bool) -> Overlay {
    let mut overlay: Overlay =
        read_yaml(&repository().join("profiles/websocket/overlay.yaml")).unwrap();
    let close = overlay
        .machines
        .iter_mut()
        .find(|m| m.name == "close")
        .unwrap();
    if default_bounds {
        close.bounds = None;
    }
    let Effect::Arm(arm) = &mut close.participants[0].transitions[0].effects[1] else {
        panic!("the second effect arms");
    };
    *arm = Arm {
        timer: "close-timeout".into(),
        interval: Interval::Fixed { millis },
    };
    overlay
}

/// Whether ESS refuses the lowered close model for its fixed interval.
fn ess_refuses_fixed(overlay: &Overlay) -> (bool, String) {
    let out = tempfile::tempdir().unwrap();
    rfcspec_model::write_generated(
        out.path(),
        &rfcspec_ess::lower(overlay, "websocket").unwrap(),
    )
    .unwrap();
    let text = match rfcspec_ess::explore(
        &out.path().join("close.protospec.yaml"),
        &repository().join("profiles/websocket/witnesses/close.inputs.json"),
    ) {
        Ok(report) => report.to_string(),
        Err(error) => format!("{error:#}"),
    };
    (text.contains("fixed interval"), text)
}

fn disagreements(default_bounds: bool, values: &[u64]) -> Vec<String> {
    let mut out = Vec::new();
    for &millis in values {
        let overlay = close_fixed(millis, default_bounds);
        let refused = problems(&overlay);
        let check_refuses = refused.iter().any(|p| p.contains("with millis"));
        let (ess_refuses, report) = ess_refuses_fixed(&overlay);
        if check_refuses != ess_refuses {
            out.push(format!(
                "millis {millis} (default bounds {default_bounds}): check {refused:?}, ess refuses {ess_refuses}: {report}"
            ));
        }
    }
    out
}

#[test]
fn check_and_ess_agree_on_a_fixed_arm_under_declared_bounds() {
    let wrong = disagreements(false, &[0, 1, 29_999, 30_000, 30_001]);
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn check_and_ess_agree_on_a_fixed_arm_under_the_default_bounds() {
    let wrong = disagreements(true, &[0, 1, 999, 1000, 1001]);
    assert!(wrong.is_empty(), "{wrong:#?}");
}

#[test]
fn a_fixed_arm_above_the_default_max_time_is_named_with_the_default() {
    let refused = problems(&close_fixed(1001, true));
    assert!(
        refused.iter().any(|p| p.ends_with(
            "arm close-timeout with millis 1001 above the machine's max_time_ms 1000"
        )),
        "{refused:?}"
    );
}
