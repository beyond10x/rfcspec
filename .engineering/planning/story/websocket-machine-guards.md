---
format: aep.planning-md/3
id: story:websocket-machine-guards
kind: story
status: implemented
title: Carry local fields, set, equality guards and never_state through the overlay machine model and its lowering
relations:
- decomposes: epic:websocket-frames
- depends_on: story:websocket-review-matrix
scope:
- confidence: cited
  path: crates/rfcspec-ess/src/lib.rs
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:47Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:48Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T08:07:14Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Story

The overlay machine model (`crates/rfcspec-model`) and its lowering (`crates/rfcspec-ess`) carry
what the close machine needed: typed inputs, payload values that are literals or trigger fields,
timers, `flush`, `close`, and the `event_count` and `flush_before_close` properties. The Ping and
data-frame machines of `epic:websocket-frames` need four more `ess-protospec/1` constructs that
ESS 0.86.0 already has (`ess:crates/specify/ess-domain/src/protocol.rs`: `Participant.fields` /
`LocalField`, `Effect::Set`, `Transition.guards` as a conjunction of `Equality`, `Property::NeverState`;
design section 6.3):

- participant local fields: name, type `Integer` or `Boolean`, initial literal;
- the effect `set: {field, value}`, the value a literal or a field of the triggering input or
  received message;
- transition `guards`, each an equality of two operands, each a literal, a local field
  (`{local: <field>}`) or a trigger field (`{input: <field>}`, as payload values name them today);
- the property `never_state: {name, participant, state}`.

This story adds them to the overlay model, `rfcspec check` and the lowering, and nothing to any
profile. It is the prefactoring both machine stories depend on, so they do not edit the same crates
in parallel.

## Acceptance

A test machine with a local field, a `set`, guards and a `never_state` property lowers to an
`ess-protospec/1` model that `ess protocol explore` passes, its mutants are reported, and both
committed profiles lower byte-identically.

- Unit tests in `crates/rfcspec-ess` build a two-participant machine in test code (no profile file):
  the client sets a local field from an input field and sends it; the server echoes it; the client
  receives the echo under a guard that the received field equals the local field and reaches a
  final state. Lowered, `ess protocol explore` with a one-input witness prints `"verdict": "passed"`.
- The same test with the echo payload changed to a literal the witness does not use: explore reports
  `inconclusive` with "dead end with outstanding queue, timer or flush obligations" (the guarded
  receive is never enabled, `ess:crates/verify/ess-conformance/src/protocol/engine.rs`,
  `enabled_actions` filters out actions with no enabled transition).
- A test machine whose reachable transition enters a state named by a `never_state` property:
  explore reports `failed` with "property violated" and the property name.
- `rfcspec check` refuses, naming the machine, participant and transition, one test each: a guard or
  `set` naming an undeclared local field; a guard or `set` value naming a field the trigger input or
  received message lacks; a local field whose initial literal does not match its type; a
  `never_state` naming an undeclared participant or state.
- `rfcspec lower --profile profiles/time --check` and `rfcspec lower --profile profiles/websocket
  --check` pass with no change under either `generated/`: absent fields, guards and properties are
  not serialised.
- The crate doc of `crates/rfcspec-ess/src/lib.rs` names the new constructs; `cargo test -p
  rfcspec-ess -p rfcspec-extract -p rfcspec-model` passes.

## Domain relations

- Profile -> Machine, one-to-many, the profile owns its machines: `ess/domains/profile.yaml`, entity
  `rfcspec.profile.Profile`, relation `machines`. The `rfcspec.profile.Machine` summary makes the
  lowered `ess-protospec/1` document the authority for participants, states, triggers, effects and
  timers, so local fields, guards and properties are protospec constructs, not new RFCspec nouns, and
  `ess/` gains no declaration.
- Transition -> local field, many-to-many by field name within one participant. Inferable (inferred
  from `ess:crates/specify/ess-domain/src/protocol.rs`, `Participant.fields` and `Equality`; no
  `ess/1` document of this repository declares it).

## Scope

`crates/rfcspec-model/src/lib.rs` (participant fields, `Effect::Set`, guards, `Property::NeverState`),
`crates/rfcspec-ess/src/lib.rs` (lowering and tests), `crates/rfcspec-extract/src/check.rs` (the
refusals). No profile, generated file or page changes.

## Notes

- Exploration does not merge configurations that differ only in step count or transmission ids
  (`ess:crates/verify/ess-conformance/src/protocol/mod.rs`, `Configuration`), and zero-field inputs
  are offered in every state (`engine.rs`, `enabled_actions`). A machine whose input transitions form
  a cycle therefore ends in "step bound exhausted". The test machines, and the machines of the two
  dependent stories, take each input transition at most once per run.
