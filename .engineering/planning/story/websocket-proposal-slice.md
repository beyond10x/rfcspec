---
format: aep.planning-md/3
id: story:websocket-proposal-slice
kind: story
status: implemented
title: Propose and review statuses for RFC 6455 sections 5.1 and 5.3, replayed in the gate
relations:
- decomposes: epic:ai-proposals
- depends_on: story:proposal-review
- depends_on: story:websocket-fragments-machine
- depends_on: story:proposal-machine-layout
- depends_on: story:reject-decision-reason
- depends_on: story:bulk-site-index
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: README.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli/tests/websocket.rs
- confidence: cited
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/proposals
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/websocket.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:52Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:52Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-05T11:29:49Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Story

The first real proposal set. The session runs `rfcspec propose` live once, against a model served on the local GPU (Ollama on loopback, protocol `chat-completions`, auth `anonymous`: no credential and no spend), over the catalogue requirements
of RFC 6455 sections 5.1 (Overview) and 5.3 (Client-to-Server Masking), none of which has an overlay
status today; the recordings are committed under `profiles/websocket/proposals/`; each proposal is
reviewed into `profiles/websocket/overlay.yaml` with `rfcspec review`; and the gate replays the set
offline. The two sections hold 11 BCP 14 keyword occurrences (7 in 5.1, lines 1487-1522; 4 in 5.3,
lines 1785-1833; counted with `grep -o` on the locked source) and overlap neither the reviewed scope
as `story:websocket-fragments-machine` leaves it (5.2, 5.4, 5.5, 5.6, 7) nor any other machine of
`epic:websocket-frames` (5.5.2, 5.5.3). This story runs after that story and closes the epic
acceptance.

## Acceptance

On a clean checkout with no credential in the environment and no network, `task check` exits 0,
and:

- `profiles/websocket/proposals/` holds one `rfcspec-proposal/1` file with task `status` for every
  catalogue requirement that `rfcspec extract --profile profiles/websocket --rfc 6455` lists in
  sections 5.1 and 5.3, each with the `model_id` of the live run; a test asserts that the set of
  proposal items equals that set of catalogue ids.
- `rfcspec propose --profile profiles/websocket --replay` exits 0, reproducing every file byte for
  byte, and is a step of the `websocket` task in `Taskfile.yml`; every cited span passes
  `rfcspec check --profile profiles/websocket`.
- `profiles/websocket/overlay.yaml` holds one decision per proposal (accept, edit or reject, made
  with `rfcspec review`; a reject carries its reason), and the accepted and edited requirements
  carry their status.
- Let A be the number of accept and edit decisions recorded in this slice. Against the state
  `story:websocket-fragments-machine` leaves (Reviewed: 70 requirements, 126 unreviewed),
  `website/docs/catalog/websocket.md` (regenerated with `task docs-generate`) reads "Reviewed:
  70 + A requirements" and 126 − A unreviewed, and the `websocket` row of
  `website/docs/catalog/index.md` shows 126 − A unreviewed; the docs test in
  `crates/rfcspec-docs/src/catalog.rs` (`the_index_counts_unreviewed_requirements_for_both_profiles`)
  computes the expected cell as the catalogue requirements outside the reviewed scope minus those
  with an overlay status, and asserts it.
- `profiles/websocket/profile.yaml` keeps its sections (5.2, 5.4, 5.5, 5.6, 7, as
  `story:websocket-fragments-machine` leaves them): the reviewed slice is outside
  the scope the in-scope status rule enforces, so a rejected proposal leaves its requirement
  unreviewed without failing the gate.
- No credential, routes file with a secret value, or token is committed: the routes file used for
  the live run lives outside the repository and names its secret as an `env:` reference;
  `b10x-gates` security and privacy checks pass on the commit.
- `AGENTS.md` State no longer excepts S3, names `profiles/websocket/proposals/` and the `propose
  --replay` gate step; `README.md` gains a short "propose and review" section linking the design
  section 9 contract.

## The live run

1. Write an `llm.routes/1` file outside the repository with one `chat-completions` endpoint at
   `http://127.0.0.1:11434/v1`, auth `anonymous`, and one route to the served model.
2. `rfcspec propose --profile profiles/websocket --task status --section 5.1 --section 5.3
   --routes <file> --route <name>`.
3. Review the new `profiles/websocket/proposals/*.json` with `rfcspec review` and commit them.

## Domain relations

- Proposal -> Profile, many-to-one: `ess/domains/review.yaml`, `rfcspec.review.Proposal`, relation
  `profile`; Decision -> Proposal, at most one: `rfcspec.review.Decision`, relation `proposal`.
- Profile -> Status, one-to-many, owned: `ess/domains/profile.yaml`, `rfcspec.profile.Profile`,
  relation `statuses`; a requirement without a status is the `unreviewed` case.
- Section -> Section parent: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Section`, relation
  `parent`; section 5.1 and 5.3 hold no subsections.

## Scope

`profiles/websocket/proposals/` (new, recorded), `profiles/websocket/overlay.yaml` (decisions and
statuses), `Taskfile.yml` (replay step), `crates/rfcspec-docs/src/catalog.rs` (test),
`crates/rfcspec-cli/tests/websocket.rs` (proposal set test), `website/docs/catalog/index.md`,
`website/docs/catalog/websocket.md` (regenerated),
`AGENTS.md`, `README.md`.

Depends on `story:proposal-review`: the decisions are written with `rfcspec review` and checked by
its overlay rules. Depends on `story:proposal-record-replay` through it, for the recorder and the
replay step. Depends on `story:websocket-fragments-machine`: both edit
`profiles/websocket/overlay.yaml`, `crates/rfcspec-docs/src/catalog.rs`,
`crates/rfcspec-cli/tests/websocket.rs` and the websocket pages, and this story's pins are stated
relative to the sections and counts that story leaves.
