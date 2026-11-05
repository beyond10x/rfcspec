---
format: aep.planning-md/3
id: story:overlay-type-checks
kind: story
status: implemented
title: Refuse guard, set and local-field type errors in rfcspec check
relations:
- decomposes: epic:hardening-follow-ups
scope:
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T11:45:37Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-05T11:45:38Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-05T11:48:22Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Story

`rfcspec check` accepts an overlay machine that `ess protocol explore` later refuses: a guard comparing operands of different types (Boolean `fin` with Integer `1`, `[GuardTypeMismatch]`), a `set` whose value type differs from its field (`[ValueTypeMismatch]`), and two local fields of one name (`[DuplicateIdentity]`). The overlay check names each at its overlay path instead.

Source: review-result:frames-adversary-1, finding 2 (`crates/rfcspec-extract/src/check.rs:168`).

## Acceptance

- A unit test in `crates/rfcspec-extract/src/check.rs` builds each of the three overlays on the committed websocket copy and asserts one problem each, naming the machine, participant and transition or field, and that the committed profiles still check clean.
- An operand typed by an input field takes the type of that field in the trigger message; a literal takes its own type; a local operand the type of its field.
- `rfcspec check --all profiles` exits 0; `task check` exits 0.
