---
format: aep.planning-md/3
id: story:ntp-timer-model
kind: story
status: proposed
title: Carry the ESS capped backoff timer and lossy channels through the overlay machine model and its lowering
relations:
- decomposes: epic:ntp-profile
scope:
- confidence: cited
  path: crates/rfcspec-ess
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T13:50:22Z", actor: "human:timo", revision: 3}
---
## Story

The overlay machine model arms a timer only for a fixed delay (`crates/rfcspec-model/src/lib.rs`,
`Arm {timer, millis}`) and lowers every channel reliable and without duplication
(`crates/rfcspec-ess/src/lib.rs`, `loss: false, duplication: false`). The NTP poll process
needs the capped doubling `ess-protospec/1` already has (ess 0.86.0,
`crates/specify/ess-domain/src/protocol.rs`, `Duration::Backoff {timer, factor, ceiling_ms}`:
"Multiply the last armed interval, capped at a positive ceiling"), and an NTP exchange runs over
a lossy datagram channel (section 8: "inherently resistant to lost or duplicate packets"). This
story carries exactly those two constructs into the overlay model, the check and the lowering, with
synthetic test overlays; it adds no profile content.

## Outcome

An overlay machine can arm a timer with a capped backoff and declare that its channels may lose
messages, and both lower to the `ess-protospec/1` constructs of ess 0.86.0; the existing profiles'
lowered models do not change.

## Acceptance

`cargo test -p rfcspec-model -p rfcspec-ess -p rfcspec-extract` exits 0 with the tests below, and
`rfcspec lower --check` exits 0 for `time`, `websocket` and `dns` with no change under their
`generated/`.

- Model: an `arm` effect is either `{timer, millis}` (unchanged) or `{timer, backoff: {factor,
  ceiling_ms}}`; both keys together, or neither, is a parse error, and unknown keys stay parse
  errors. It lowers to `after: {kind: backoff, timer: <the armed timer>, factor, ceiling_ms}`. A
  test lowers `{arm: {timer: poll, backoff: {factor: 2, ceiling_ms: 131072000}}}` and asserts that
  JSON.
- Model: a machine may declare `loss: true`; it lowers to `loss: true` on both channels. Absent
  or false writes `loss: false` as today. A test asserts both.
- `rfcspec check` refuses, naming the machine, participant and transition: a backoff with
  `factor` 0 or `ceiling_ms` 0; a backoff whose `ceiling_ms` exceeds the machine's
  `max_time_ms` (declared or the lowering default 1000); a backoff of a timer the participant does
  not declare. One test each. (`ess specify validate` refuses the same; the check names the
  overlay line instead of the lowered one.)
- A test lowers a two-participant overlay whose client arms `t` fixed 1000 ms, then on `t` arms
  it with backoff factor 2 ceiling 4000 ms, and runs `ess protocol explore` on the result: the
  verdict is `passed` once the machine ends by cancelling `t`; the counterexample-free report is
  asserted, not the state count.
- The crate documentation of `rfcspec-ess` names the backoff and `loss`.

## Scope

`crates/rfcspec-model/src/lib.rs` (`Arm`, machine `loss`), `crates/rfcspec-ess/src/lib.rs`
(lowering, tests), `crates/rfcspec-extract/src/check.rs` (the three refusals),
`crates/rfcspec-cli/tests` only if a CLI-level refusal test is needed. Reads `profiles/`, writes
nothing there.

Choices taken:

- No `duplication` flag. Probed against ess 0.86.0 on 2026-11-05: a two-poll exchange with
  `duplication: true` on capacity-2 channels ends in "step bound exhausted" at `max_steps: 12`
  (19828 states) and did not finish in 180 s at `max_steps: 64`, because a queued message can be
  duplicated again after every delivery and configurations differing in transmission ids are not
  merged. Such a model can never pass exploration; the exchange story models a replayed packet
  with an input instead, and the gap is recorded by `story:ntp-poll-machine`.
- Loss is per machine, not lowered from the layer's `delivery` semantics: lowering it from
  semantics would change the `time` UDP model.
- A backoff always reads the interval of the timer it arms; ESS allows another timer's interval,
  which no NTP rule needs.
- No duration read from a local field (gap G7): ess 0.86.0 has only `fixed` and `backoff`.

No dependency: independent of `story:ntp-source-catalogue` and `story:ntp-header-codec` (it
touches no profile and no generator file), so it can run beside them in one wave.
