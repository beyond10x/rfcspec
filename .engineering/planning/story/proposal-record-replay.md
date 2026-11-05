---
format: aep.planning-md/3
id: story:proposal-record-replay
kind: story
status: implemented
title: Record status proposals through the llm crates and replay them byte for byte offline
relations:
- decomposes: epic:ai-proposals
- depends_on: story:dns-query-machine
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-propose
- confidence: cited
  path: ess/domains/review.yaml
- confidence: cited
  path: ess/scenarios
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:50Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:50Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-05T08:47:04Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

Stage S3 for requirement entries: `rfcspec propose` asks a model, through the beyond10x `llm`
crates, for overlay requirement entries for catalogue requirements that have no overlay status,
checks each answer mechanically and records it as a proposal document under the profile;
`rfcspec propose --replay` rebuilds every recorded proposal from its recorded inputs and the
recorded response, offline, and fails unless the result is byte-identical to the file. Two tasks
get prompts in this story: `status` (a status `unsupported` or `not-applicable` with a reason, and
the level as the reviewer reads it) for a BCP 14 requirement, and `requirement-from-prose` (a level
for a `prose_candidates` sentence of the catalogue, design section 5). A proposal changes nothing:
neither the overlay nor any generated file is written.

## Acceptance

`cargo test -p rfcspec-propose -p rfcspec-cli` exits 0 offline, with no network access and no
credential in the environment, and proves record, replay and the mechanical checks against an
in-process `llm_core::Model` test double.

- ESS: `ess/domains/review.yaml` `rfcspec.review.ProposalTask` gains the variant `Status` (wire
  `status`, summary "a status unsupported or not-applicable with a reason, and a level, for one
  requirement"); `ess specify validate --path ess` exits 0; a new scenario
  `ess/scenarios/record-proposal-already-exists.yaml` records a `status` proposal for
  `rfc6455/R5.1-1` twice and expects `recorded` then `already-exists`.
- New crate `crates/rfcspec-propose` (design section 13). The proposal document
  `rfcspec-proposal/1`, one JSON file per proposal at `<profile>/proposals/<proposal_id>.json`, holds
  the fields of `rfcspec.review.Proposal`: `proposal_id`, `profile`, `item` (the requirement as
  `rfc<N>/R<section>-<ordinal>`), `task`, `model_id`, `input_digest`, `response`, `cited_spans`
  (line spans of the locked source, as the overlay uses), plus the prompt inputs it was built from.
  `input_digest` is the SHA-256 of the canonical prompt and inputs; `proposal_id` is the SHA-256 of
  task, model id and input digest. Serialization is canonical (sorted keys, two-space indent,
  trailing newline), so the same proposal always has the same bytes.
- The response of both tasks is one overlay requirement entry in the `rfcspec-overlay/1` shape
  (`id`, `source`, `level`, `quote`, `span`, `status`); the model is called with
  `llm_tool_call::call_tool` and that entry as the forced tool input schema.
- Mechanical checks before a proposal is written (design section 9; ESS
  `rfcspec.review.RecordProposal` outcome `check-failed`): the response parses as an overlay
  requirement entry; its `quote` occurs verbatim at its `span` and every cited span lies in the
  locked source, using the same quote test as `rfcspec check` (`quote_problem` in
  `crates/rfcspec-extract/src/check.rs`, made public); the id is the catalogue id of the item it
  was asked about; a `status` answer is `unsupported` or `not-applicable` with a non-empty reason
  (ESS `rfcspec.profile.RecordStatus` refuses `Satisfied`). A unit test feeds the double one answer
  per failure (a quote not in the source, a span past the end, another id, a `satisfied` status, an
  empty reason) and asserts no file is written and the error names the proposal id and the reason.
- Record: `rfcspec propose --profile <dir> --task status --section <n>... --routes <llm.routes/1
  file> --route <name>` builds the model with `llm_models::routes::Routes::parse` and
  `Routes::port`, resolving secrets through `Routes::resolvers`, so credentials are references
  (`env:`, `file:`, `keychain:`) the operator injects and never values in the repository; a proposal
  already recorded under its id is read, not requested again (ESS outcome `already-exists`). A test
  asserts that a routes file whose secret is an `env:` variable that is unset is refused before any
  request is sent, naming the endpoint.
- Replay: `rfcspec propose --profile <dir> --replay` reads every `proposals/*.json`, rebuilds the
  prompt and inputs from the locked source and the recorded inputs, recomputes `input_digest` and
  `proposal_id`, takes the response from the recording, reruns the checks and compares the
  serialized bytes with the file. It exits 0 when all are identical and 1 naming each file that
  differs, has a wrong id or fails a check. A test records three proposals into a temporary copy of
  `profiles/websocket` through the double, replays them byte for byte, then flips one byte of a
  recorded quote and one byte of `input_digest` and asserts exit 1 naming each file.
- `requirement-from-prose`: a test records through the double one proposal with task
  `requirement-from-prose` into a temporary copy of `profiles/websocket`, for one `prose_candidates`
  sentence of its catalogue (the section 5.5.3 Pong-echo sentence, lines 2060-2062, lowercase
  "must"), answered with a `source: prose` requirement entry. `rfcspec propose --profile <copy>
  --replay` then exits 0 and reproduces the file byte for byte, and `rfcspec check --profile <copy>`
  passes on the entry's cited span. The test asserts both.
- `rfcspec check` also runs the replay of the profile proposals, so `task check` refuses a
  hand-edited or stale proposal; a profile without `proposals/` passes unchanged.
- No test, gate step or `task check` step calls a model or reads a credential; the only `Model` in
  the test suite is the in-process double. `README.md` gains no live instructions; the live run is
  the try-it step of `story:websocket-proposal-slice`.

## Domain relations

- Proposal -> Profile, many-to-one, references: `ess/domains/review.yaml`, entity
  `rfcspec.review.Proposal`, relation `profile`. A proposal lives under its profile directory.
- Proposal -> Decision, at most one (the `decision` field is optional and unset until review),
  references: `ess/domains/review.yaml`, entity `rfcspec.review.Proposal`, relation `decision`. This
  story records proposals only, all in state `Recorded`; nothing sets the relation.
- Proposal -> catalogue item: `ess/domains/review.yaml`, entity `rfcspec.review.Proposal`, field
  `item` of type `rfcspec.catalogue.ItemId` (a typed field, not a `relations:` entry); the spelling
  `rfc<N>/R<section>-<ordinal>` is the `rfcspec.catalogue.ItemId` summary in
  `ess/domains/catalogue.yaml`. Several proposals may name one item (another model or other inputs
  give another `proposal_id`, the identity in `rfcspec.review.ProposalId`).

## Scope

`ess/domains/review.yaml` (the `Status` task variant), `ess/scenarios/` (one scenario), new
`crates/rfcspec-propose` (proposal document, prompts for `status` and `requirement-from-prose`,
digests, checks, record through `llm-core`, `llm-tool-call` and `llm-models` at the `llm` release tag
0.20.0, replay; tests with an in-process model), `crates/rfcspec-extract/src/check.rs`
(`quote_problem` public), `crates/rfcspec-cli` (`propose` subcommand in `src/main.rs`, the replay in
`fn check`, tests), `Cargo.toml`, `Cargo.lock`, `AGENTS.md` (State and Gate: the new crate and
subcommand).

Depends on `story:dns-query-machine`, the last story of `epic:dns-profile` in the recorded order
(websocket-frames, dns-profile, ai-proposals, bulk-catalogue): it shares `AGENTS.md`, `Cargo.toml`,
`Cargo.lock`, `crates/rfcspec-cli/tests` and `crates/rfcspec-extract/src/check.rs` with the dns and
frames stories. It reads the committed `profiles/websocket` catalogue and overlay and writes no profile
file.
