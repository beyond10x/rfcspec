---
format: aep.planning-md/3
id: story:proposal-review
kind: story
status: implemented
title: Review a recorded proposal into the overlay with accept, edit or reject
relations:
- decomposes: epic:ai-proposals
- depends_on: story:proposal-record-replay
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: crates/rfcspec-cli/src/checks.rs
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: crates/rfcspec-propose
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:50Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:51Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-05T08:56:01Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Story

Stage S4 for recorded proposals: `rfcspec review` settles one recorded proposal with accept, edit
or reject, writing a decision into the profile overlay (ESS `rfcspec.review.RecordDecision` and
`rfcspec.review.DecideProposal`). Accept and edit also write the proposed (or edited) requirement
entry into the overlay; reject writes only the decision, so the proposal is not offered again and
the requirement stays unreviewed. The catalogue unreviewed count drops by the accepted and edited
number, because it already counts catalogue requirements without an overlay status
(`crates/rfcspec-docs/src/catalog.rs`, `Entry::unreviewed`).

## Acceptance

`cargo test -p rfcspec-model -p rfcspec-extract -p rfcspec-cli -p rfcspec-docs` exits 0 offline and
proves each verdict end to end on a temporary copy of `profiles/websocket` holding proposals
recorded through the in-process model of `story:proposal-record-replay`.

- `rfcspec-overlay/1` gains an optional `decisions` list (`crates/rfcspec-model/src/lib.rs`,
  `Overlay`), each entry the fields of `rfcspec.review.Decision` this stage writes: `item`,
  `text_digest` (SHA-256 of the requirement text in the catalogue, whitespace collapsed), `kind`
  (`accept`, `edit` or `reject`), `proposal_id`, and for `edit` the replacement entry. The profile
  of the key is the overlay profile. An overlay without `decisions` reads and writes unchanged, so
  `profiles/time/overlay.yaml` and its recorded runs keep their digests.
- `rfcspec review --profile <dir> --proposal <id> accept` adds the response entry to `requirements`
  in catalogue order and a decision of kind `accept`; `... edit --entry <file>` adds the entry from
  the file instead and a decision of kind `edit` carrying it; `... reject --reason <text>` adds a
  decision of kind `reject` and no requirement. Each command writes the overlay only after the
  overlay check (`rfcspec_extract::check`) and the profile checks hold on the result; otherwise it
  writes nothing and exits 1 with the problems.
- Refusals, each a test, each leaving the overlay byte-identical: an unknown proposal id
  (`ProposalNotFound`); a proposal that already has a decision (`ProposalStateConflict`); an item
  that already has a decision under the same text digest (`AlreadyExists`); an item that already
  has an overlay status without a decision (`AlreadyExists`, so a hand-reviewed requirement is not
  overwritten); `edit` without an entry file (`DecisionIncomplete`); an edited entry whose quote is
  not at its span.
- `rfcspec check` adds: every decision names a recorded proposal of the profile, at most one
  decision per proposal, the decision `text_digest` equals the digest of the current catalogue text
  of its item, and every accept or edit decision has its requirement in `requirements`. A test
  removes the proposal file of a decision, and changes one `text_digest`, and asserts exit 1 naming
  the item each time.
- `rfcspec propose` (record mode) skips catalogue requirements that have a decision; replay is
  unaffected, since it replays recorded files.
- The docs test asserts that after accepting two proposals and rejecting one in the temporary
  profile, `Entry::unreviewed` is two fewer and the rejected requirement is still unreviewed.

## Domain relations

- Decision -> Proposal, at most one, references: `ess/domains/review.yaml`, entity
  `rfcspec.review.Decision`, relation `proposal` (via `proposal_id`).
- Proposal -> Decision, at most one, references: `ess/domains/review.yaml`, entity
  `rfcspec.review.Proposal`, relation `decision`. The proposal file is never rewritten (replay is
  byte for byte), so the proposal state (`Recorded`, `Accepted`, `Edited`, `Rejected`) is read from
  the overlay decision that names it.
- Decision -> Profile, many-to-one, references: `ess/domains/review.yaml`, entity
  `rfcspec.review.Decision`, relation `profile`; the identity is `rfcspec.review.DecisionKey`
  (profile, item, text digest), so one item holds at most one decision per text in one profile.
- Profile -> Status, one-to-many, owned: `ess/domains/profile.yaml`, entity
  `rfcspec.profile.Profile`, relation `statuses`; Status -> Requirement, one:
  `rfcspec.profile.Status`, relation `requirement`. An accepted entry gives its requirement the
  profile status.

## Scope

`crates/rfcspec-model/src/lib.rs` (`Overlay.decisions` and its entry type), `crates/rfcspec-extract/src/check.rs`
(decision checks), `crates/rfcspec-cli` (`review` subcommand in `src/main.rs`, decision rules in
`src/checks.rs`, tests), `crates/rfcspec-propose` (skip decided items when recording),
`crates/rfcspec-docs/src/catalog.rs` (test only), `AGENTS.md` (the `review` subcommand).

Depends on `story:proposal-record-replay`: it decides the proposal documents that story creates and
extends the `rfcspec-propose` crate and the `rfcspec-cli` subcommands it adds.
