---
format: aep.planning-md/3
id: story:catalog-semantics
kind: story
status: implemented
title: Declare per-layer communication semantics in profile.yaml and show them on the profile page
relations:
- decomposes: epic:public-protocol-catalog
- depends_on: story:catalog-downloads
scope:
- confidence: cited
  path: crates/rfcspec-cli
- confidence: cited
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: cited
  path: crates/rfcspec-model
- confidence: cited
  path: ess/domains/profile.yaml
- confidence: cited
  path: ess/scenarios/revise-profile-party-not-a-role.yaml
- confidence: cited
  path: profiles/time/profile.yaml
- confidence: cited
  path: website/docs/catalog/time.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T23:34:09Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-04T23:34:09Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-04T23:50:41Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

A profile declares its communication semantics (design section 6), one entry per layer, where
each transport is its own layer (`decision-blocker:catalog-multi-transport-semantics`).
`rfcspec check` validates them, and the profile page shows one row per layer.

## Acceptance

`profiles/time/profile.yaml` declares layers `tcp` and `udp` with the Time Protocol's semantics over each transport, `rfcspec check --profile profiles/time` accepts it, and
`website/docs/catalog/time.md` shows a semantics table with exactly those two rows.

- `rfcspec-model` `Profile` gains `semantics: Vec<LayerSemantics>`, defaulting to empty, with
  the fields of `rfcspec.profile.LayerSemantics`: `layer` (a name), `unit`, `delivery`
  (`ordered`, `reliable`, `duplicates`), `direction`, `multiplexing`, `exchange`, `correlation`,
  `lifetime`, optional `over` (`profile`, `endpoint`, `name`), `roles`. The enum values are the
  wire spellings of `ess/domains/profile.yaml`; an unknown value or field is a parse error.
- `ess/domains/profile.yaml`: `LayerSemantics.layer` becomes a layer name (a `String` newtype,
  for example `rfcspec.profile.LayerName`) instead of the `rfcspec.profile.Layer` enum, so that
  `tcp` and `udp` are legal; the `distinct by s.layer` invariant stays. Every scenario using
  `layer: Protocol` (`ess/scenarios/revise-profile-party-not-a-role.yaml`) is updated, and
  `ess specify validate --path ess` passes.
- `rfcspec check` refuses, each with a test, naming the profile and layer: a layer name given
  twice; a profile with machines in the overlay but no semantics; a machine `transport` with no
  layer of that name; an `over.profile` naming no directory under the profile root parent.
  `profiles/time` passes.
- `profiles/time/profile.yaml` values describe the Time Protocol over each transport (RFC 868),
  not the transport itself: `tcp` bytes, ordered, reliable, no duplicates, simplex (server to
  client), single, one-shot, correlation none, lifetime connection; `udp` datagrams, unordered,
  lossy, duplicates, half-duplex, single, request-response, correlation none, connectionless.
  Roles `[client, server]` for both (the overlay participant names). `over` is omitted for both:
  no `tcp` or `udp` profile exists yet.
- `rfcspec lower --check` and `rfcspec generate --check` stay green with no change to
  `profiles/time/generated/`.
- The profile page gains a "Communication semantics" table, one row per layer in declaration
  order, one column per axis; a profile without semantics (the fixture of `story:catalog-index`,
  which has no machines) shows "No communication semantics declared." A unit test asserts both.

## Domain relations

- Profile -> LayerSemantics, one-to-many, owned as a value list, one per distinct layer:
  `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`, field `semantics` and its
  invariant `distinct: {in: semantics, as: s, by: s.layer}`. Layer = transport per the decision
  recorded in `decision-blocker:catalog-multi-transport-semantics`.
- Machine -> LayerSemantics, many-to-one by transport name. Inferable (inferred from
  `crates/rfcspec-model/src/lib.rs`, overlay `Machine.transport`, and the decision above; no
  `relations:` entry declares it).
- LayerSemantics -> Profile through `over`, zero or one: `ess/domains/profile.yaml`, type
  `rfcspec.profile.Layering`, field `profile: rfcspec.profile.ProfileName` (a field, no
  `relations:` entry).

## Scope

`crates/rfcspec-model` (`Profile.semantics`), `crates/rfcspec-cli` (`check`), `ess/domains/profile.yaml`,
`ess/scenarios/revise-profile-party-not-a-role.yaml`, `profiles/time/profile.yaml`,
`crates/rfcspec-docs` (`catalog.rs` page section, tests), `website/docs/catalog/time.md`
(regenerated).

Depends on `story:catalog-downloads`: it adds a section to the same generated profile page and
edits the same `crates/rfcspec-docs/src/catalog.rs`, so the two cannot run in parallel; it comes
after downloads to keep one line of change through that file.
