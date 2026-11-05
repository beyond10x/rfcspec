---
format: aep.planning-md/3
id: story:send-payload-type-check
kind: story
status: implemented
title: Refuse a send payload typed differently from its message field
relations:
- decomposes: epic:hardening-follow-ups
scope:
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T20:58:30Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T21:24:57Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-05T21:35:59Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Story

`rfcspec check` types guard operands and `set` values (story:overlay-type-checks) but not the payload a `send` effect gives a message field. A payload value typed differently from its message field passes `rfcspec check`; whether `ess protocol explore` refuses it is not yet known.

## Acceptance

A unit test in `crates/rfcspec-extract/src/check.rs` builds an overlay whose `send` gives a Boolean to an Integer message field and asserts exactly one problem naming the transition, the field and both types. `rfcspec check --all profiles` exits 0 on the committed profiles. The full gate (`task check`) is not part of this story: it runs once at the wave close, over the whole `check.rs` chain.

## Order

Four stories edit `crates/rfcspec-extract/src/check.rs` and run in this order, each depending on the one before it: story:send-payload-type-check → story:fixed-arm-bounds-check → story:xml-overlay-check → story:websocket-grammar-review; story:status-evidence follows the last. This story is first in that chain and runs in wave 5 (coordinator plan, 2026-11-05).

## Scope

- cited: `crates/rfcspec-extract/src/check.rs` (the fix and its unit test).
