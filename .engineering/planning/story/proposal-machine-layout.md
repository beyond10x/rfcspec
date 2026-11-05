---
format: aep.planning-md/3
id: story:proposal-machine-layout
kind: story
status: implemented
title: Propose machines, layouts and requirement links as reviewable overlay elements
relations:
- decomposes: epic:ai-proposals
- depends_on: story:proposal-review
- depends_on: story:websocket-ping-machine
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-propose
- confidence: cited
  path: crates/rfcspec-propose/tests/fixtures/no-machines
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:51Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:51Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-05T09:12:52Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

The rest of the epic outcome: proposals for model elements, not only for requirement statuses.
`rfcspec propose` gains three tasks, each with its design section 9 mechanical check, and
`rfcspec review` merges an accepted or edited element into the overlay:

- `machine`: one overlay `machines` entry (participants, states, transitions, each transition with
  `implements` requirement ids), proposed from named sections;
- `bit-widths`: one overlay `layouts` entry (fields with bit widths, presence and constraints),
  proposed from a packet diagram and its ABNF comments;
- `requirement-link`: a requirement entry with status `satisfied <element>` naming an element the
  overlay holds.

No committed profile changes: the WebSocket machines themselves are `epic:websocket-frames`, which
may use this path or not.

## Acceptance

`cargo test -p rfcspec-propose -p rfcspec-cli` exits 0 offline, each task proven on a temporary copy
of the fixture profile `crates/rfcspec-propose/tests/fixtures/no-machines/` through the in-process
`llm_core::Model` double of `story:proposal-record-replay`, with record, replay byte for byte, review
and refusal.

- Fixture profile `crates/rfcspec-propose/tests/fixtures/no-machines/`: `profile.yaml` (RFC 6455,
  sections 5.2 and 5.5, one `tcp` layer), `rfcspec.lock` (the digest of `profiles/websocket`'s lock)
  and `overlay.yaml` with no `machines` entry, no status for any requirement of sections 5.5.2 and
  5.5.3, and a copy of the `frame-header` layout. The test copies the fixture into a temporary
  directory and copies `profiles/websocket/sources/rfc6455.txt` beside it, so no RFC text is
  committed twice. No test reads or copies `profiles/websocket/overlay.yaml`, so the `ping` machine
  of `story:websocket-ping-machine` and later overlay changes cannot collide with a proposed machine.

- `machine` (item: the profile name, the whole-profile `ItemId`): before writing, the check adds the
  machine to a copy of the overlay and requires that `rfcspec_ess::lower` succeeds on it, that every
  `implements` id is a catalogue requirement id, that every transition implements at least one, and
  that the machine transport is a layer of `profile.yaml`; `cited_spans` are the spans of the
  implemented requirements. Test: a double answer for a Ping/Pong machine from sections 5.5.2 and
  5.5.3 is recorded, replayed, accepted with `rfcspec review`, and `rfcspec lower --profile <copy>
  --check` then fails only because the generated model is new (the lowering itself succeeds); answers
  with an unknown requirement id, a transition implementing nothing, and an undeclared transport are
  each refused with no file written.
- `bit-widths` (item `rfc6455/layout/<slug>`): the layout quote occurs verbatim at its span, and the
  overlay layout checks of `crates/rfcspec-extract/src/check.rs` hold on the overlay with the layout
  added (widths 1 to 64, presence on an earlier field, constraints within width). Test: a double
  answer reproducing the committed `frame-header` layout under another name is accepted; one with a
  65-bit field is refused.
- `requirement-link` (item: the requirement): the named element exists in the overlay
  (`<machine>.<participant>.<transition>` or `layout.<name>`, the forms of `Status::Satisfied` in
  `crates/rfcspec-model/src/lib.rs`), and for a machine element the transition lists the requirement
  in `implements`. Test: a link to `layout.frame-header` is accepted; a link to a missing transition
  is refused.
- `rfcspec review` accepts, edits and rejects each kind with the rules of `story:proposal-review`; a
  machine or layout whose name the overlay already holds is refused (`AlreadyExists`) and the
  overlay is left byte-identical.
- Still no test calls a model or reads a credential.

## Domain relations

- Profile -> Machine, one-to-many: `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`,
  relation `machines`. A machine proposal names the profile as its item, the whole-profile spelling
  `<profile>` of `rfcspec.catalogue.ItemId` in `ess/domains/catalogue.yaml` (that summary keys
  machines by profile and slug, not by an item id).
- Proposal -> Profile and Decision -> Proposal: `ess/domains/review.yaml`, relations `profile`
  (`rfcspec.review.Proposal`) and `proposal` (`rfcspec.review.Decision`), as in
  `story:proposal-review`.
- Profile -> Status, one-to-many, owned: `ess/domains/profile.yaml`, `rfcspec.profile.Profile`,
  relation `statuses`; Status -> Requirement, one: `rfcspec.profile.Status`, relation `requirement`.

## Scope

`crates/rfcspec-propose` (prompts, response shapes and checks for `machine`, `bit-widths` and
`requirement-link`; tests), `crates/rfcspec-propose/tests/fixtures/no-machines/` (the fixture
profile, new), `crates/rfcspec-propose/Cargo.toml` and `Cargo.lock` (`rfcspec-ess`
dependency), `crates/rfcspec-cli` (`review` merging machines and layouts in `src/main.rs`; tests),
`crates/rfcspec-extract/src/check.rs` (layout checks callable on one layout, if needed).

Depends on `story:proposal-review`: it extends the `review` subcommand and the overlay decision
rules that story adds, and the recorder of `story:proposal-record-replay` beneath it.
