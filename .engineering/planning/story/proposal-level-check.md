---
format: aep.planning-md/3
id: story:proposal-level-check
kind: story
status: implemented
title: Refuse a status or link proposal whose level differs from the catalogue keyword
relations:
- decomposes: epic:hardening-follow-ups
scope:
- confidence: cited
  path: crates/rfcspec-propose/src/elements_tests.rs
- confidence: cited
  path: crates/rfcspec-propose/src/lib.rs
- confidence: cited
  path: crates/rfcspec-propose/src/tests.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T11:45:38Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-05T11:45:39Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-05T11:48:39Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Story

A `status` or `requirement-link` proposal whose level differs from the BCP 14 keyword of the catalogue requirement it is about passes record, review and `rfcspec check` (a MUST proposed as MAY). The mechanical check refuses it as `check-failed` with the catalogue level and the proposed one. A `requirement-from-prose` proposal keeps choosing its level, since prose has none.

Source: review-result:dnsai-adversary-1, finding 3 (`crates/rfcspec-propose/src/lib.rs:611`).

## Acceptance

- A unit test feeds the model double a `status` answer for `rfc6455/R5.1-1` (MUST) with level MAY and asserts no file is written and the error names the proposal id, MUST and MAY; the same for a `requirement-link` answer.
- The ten recorded proposals under `profiles/websocket/proposals/` still replay byte for byte (`rfcspec propose --replay` exits 0).
- `task check` exits 0.
