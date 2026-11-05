---
format: aep.planning-md/3
id: story:fixed-arm-bounds-check
kind: story
status: proposed
title: Refuse a fixed timer arm of 0 ms or above max_time_ms
relations:
- depends_on: story:send-payload-type-check
scope:
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T20:58:31Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

`rfcspec check` refuses a fixed `arm` (`millis`) of 0 or above the machine's `max_time_ms`, naming machine, participant, transition and timer, as it already does for the backoff form. `ess specify validate` refuses both values ("fixed interval must be positive and within max_time_ms"), so today the check admits an overlay that lowering then fails.

Source: review-result:ntp-timer-adversary-1, finding 3 (pre-existing, `crates/rfcspec-extract/src/check.rs:260`).

## Acceptance

Both refusals follow the backoff form already in `crates/rfcspec-extract/src/check.rs` (lines 275-297): `let head = format!("{item}: arm {} with backoff", arm.timer);` followed by `"{head} ceiling_ms 0, not a positive integer"` and `"{head} ceiling_ms {} above the machine's max_time_ms {max_time_ms}"`. The existing test reads, for example, `"{CLOSE_START}: arm close-timeout with backoff ceiling_ms 30001 above the machine's max_time_ms 30000"` (line 1380), where `CLOSE_START` is `transition close.client.R7.1.2-1-start-closing-handshake` (line 1332).

- Unit tests in `check.rs`, on the websocket `close` machine (`max_time_ms: 30000`, `profiles/websocket/overlay.yaml:1310`), with its fixed `arm` of `close-timeout` changed:
  - `millis: 0` gives exactly `transition close.client.R7.1.2-1-start-closing-handshake: arm close-timeout with millis 0, not a positive integer`;
  - `millis: 30001` gives exactly `transition close.client.R7.1.2-1-start-closing-handshake: arm close-timeout with millis 30001 above the machine's max_time_ms 30000`;
  - `millis: 30000` gives no problem.
- `rfcspec check --all profiles` exits 0.
- The full gate (`task check`) is not part of this story: it runs once at the wave close, over the whole `check.rs` chain.

## Scope

- `crates/rfcspec-extract/src/check.rs`

## Order

Four stories edit `crates/rfcspec-extract/src/check.rs` and run in this order: story:send-payload-type-check → story:fixed-arm-bounds-check → story:xml-overlay-check → story:websocket-grammar-review; story:status-evidence follows the last. This story depends on story:send-payload-type-check because both edit `check.rs`, and runs in wave 6 (coordinator plan, 2026-11-05).
