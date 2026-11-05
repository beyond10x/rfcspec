---
format: aep.planning-md/3
id: story:explorer-profile-layer
kind: story
status: active
title: 'Show a profiled RFC''s reviewed layer in the explorer: each requirement''s status, review decision and recorded proposal, the machines, layouts and their witnesses'
relations:
- decomposes: epic:rfc-explorer
- depends_on: story:explorer-rfc-view
- depends_on: story:explorer-search
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T17:48:52Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-05T19:00:27Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":3}}}
---
## Story

For an RFC a profile locks (RFC 868 in `time`, RFC 1035 in `dns`, RFC 5905 in `ntp`, RFC 6455 in `websocket`), the explorer view of `story:explorer-rfc-view` adds the reviewed layer below the bulk data:

- each requirement of the profile overlay with its id, level, quote and source span, and its status: satisfied with its element and evidence, unsupported or not applicable with its reason and ESS gap;
- where a review decision exists for the requirement: its kind (accept, edit, reject), a reject reason, and the recorded proposal it decided (proposal id and model id), linked to `profiles/<name>/proposals/<proposal_id>.json` on GitHub;
- the unreviewed requirements of the mechanical catalogue counted by level, as the profile page counts them;
- the machines, each with its `ess-protospec/1` download and its input witness `profiles/<name>/witnesses/<machine>.inputs.json`, and the layouts with their fields and bit widths.

## Decision: data per profile; the profile pages stay

- `rfcspec-docs` writes one JSON per profile, `website/static/data/profiles/<name>.json`, from the same `Entry` the profile page is rendered from (`crates/rfcspec-docs/src/catalog.rs`, `load`), so page and explorer cannot disagree. The Docusaurus profile pages stay the per-profile report (semantics, ESS gaps, conformance, downloads); their RFCs section links each locked RFC to its explorer view, and the explorer links back.
- The quotes are those the profile pages already publish from the overlay. The explorer shows no other RFC text.
- Witnesses link to the repository file on `main`, as the overlay link of the profile page does (`crates/rfcspec-docs/src/page.rs`, `SOURCE`); they are not copied into downloads.
- The JSON shapes reuse the rows of the existing ESS views where they fit (`rfcspec.profile.Statuses`, `rfcspec.profile.Machines`, `rfcspec.review.Decisions`), generated to Rust and TypeScript with `ess generate types --target rust` and `--target typescript --root <view>.row`, both drift-checked; any wrapper type is declared in `ess/domains/profile.yaml` first.

## Domain relations

- `Profile -> Status`, one-to-many, Profile owns: `ess/domains/profile.yaml`, `rfcspec.profile.Profile` relation `statuses` (owns, many, via `profile`).
- `Status -> Requirement`, many-to-one: `ess/domains/profile.yaml`, `rfcspec.profile.Status` relation `requirement` (references, one, via `requirement_id`).
- `Profile -> Machine`, one-to-many, Profile owns: `ess/domains/profile.yaml`, `rfcspec.profile.Profile` relation `machines` (owns, many, via `profile`).
- `Rfc -> Layout`, one-to-many, Rfc owns: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc` relation `layouts` (owns, many, via `rfc`).
- `Decision -> Proposal`, many-to-one, the far side may be absent: `ess/domains/review.yaml`, `rfcspec.review.Decision` relation `proposal` (references, one, via `proposal_id`); the reverse is `rfcspec.review.Proposal` relation `decision`.
- Decision to requirement is a field, not a relation: `rfcspec.review.Decision` fields `item` (ItemId) and `text_digest` key a decision to the item text it judged (`ess/domains/review.yaml`). The explorer joins on the item id.
- Which profile file the RFC view loads for `?rfc=<n>`: one source, the profile marks of `rfc-index.json` that `story:explorer-search` writes from the profiles' locks. The view loads `profiles/<name>.json` for each mark, and none for an RFC without one (RFC 9220). This is also a data reason for `depends_on story:explorer-search`; this story writes no second mapping.
- `Profile -> Rfc` is a field, `rfcspec.profile.Profile` `rfcs` (`ess/domains/profile.yaml`), not a relation.
- `Machine -> witness`, one-to-one: inferable (inferred from `Taskfile.yml:47-90` and `crates/rfcspec-ess/src/lib.rs:764`, which read `witnesses/<machine>.inputs.json` for every machine of the four profiles; no ess/1 document declares a witness).

## Acceptance

Before: the explorer view of RFC 6455 shows only the bulk data, and no page shows which proposal decided which requirement. After: `/rfcspec/explorer/?rfc=6455` lists every requirement of `profiles/websocket/overlay.yaml` with its status, each decided requirement showing its decision and linking its proposal file.

Checked by:

- RFC 6455 decisions: 85 accepted or edited, 29 rejected with their reason, re-read from the overlay before acceptance.
- RFC 6455 machines: `close`, `ping` and `fragments` each link their protospec download and witness.
- `?rfc=868` shows the `tcp` and `udp` machines with their witnesses and no decisions.
- `?rfc=9220` shows no reviewed layer.

- `cargo test -p rfcspec-docs`: the websocket JSON holds the reviewed and unreviewed counts the profile page prints, every decision with a proposal id names a file under `profiles/websocket/proposals/`, and every machine names an existing witness.
- `task explorer`, `vitest`: the reviewed layer renders a fixture profile; an RFC no profile locks has none.
- `task docs-check` drift-checks the four files.

## Scope

- `crates/rfcspec-docs/src/explorer.rs`: the profile data; `src/catalog.rs`: the explorer link in the RFCs section of a profile page.
- `website/static/data/profiles/*.json` (generated).
- `explorer/src/`: the reviewed-layer section of the RFC view, with tests; `explorer/src/generated/` for the view rows.
- `ess/domains/profile.yaml`, only if a wrapper type is needed.
- `Taskfile.yml`: the generated Rust and TypeScript view-row types join the drift check; the generated Rust types for the view rows beside the shard types.

Shared with `story:explorer-search`: both extend `crates/rfcspec-docs/src/explorer.rs` and write under `explorer/src/`, which `story:explorer-rfc-view` creates. That is why this story runs after `story:explorer-search` (`depends_on`), not in the same wave.

## Open questions

- The epic asks for "their exploration witnesses". Read here as the committed input witnesses. If it means the `ess protocol explore` report and its verdict, that report is not committed (the gate only greps `"verdict": "passed"`, `Taskfile.yml:47`); unclear, ask the operator.
- Whether a decided proposal should show its response and cited spans inline or only link the file: this story links the file and shows proposal id and model id.
