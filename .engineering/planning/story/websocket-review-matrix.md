---
format: aep.planning-md/3
id: story:websocket-review-matrix
kind: story
status: implemented
title: Review RFC 6455 sections 5.2, 5.5 and 7 completely and show unreviewed requirements in the catalogue
relations:
- decomposes: epic:websocket-profile
- depends_on: story:websocket-frame-codec
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/rfcspec-cli/src/checks.rs
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-docs
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/time.md
- confidence: cited
  path: website/docs/catalog/websocket.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T00:11:33Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T00:11:33Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-05T00:42:15Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

The overlay reviews every RFC 6455 requirement in the sections the profile scopes (5.2, 5.5 and 7,
with their subsections) and leaves the rest unreviewed; `rfcspec check` enforces the design section
5 rule that every MUST-level requirement in a profile scope has a status; and the catalogue index
and profile page count catalogue requirements without a status as `unreviewed`, so a partly reviewed
RFC is shown as such. This story closes the epic acceptance.

## Acceptance

On a clean checkout `task check` exits 0, every requirement of sections 5.2, 5.5 and 7 has a status in
`profiles/websocket/overlay.yaml`, and `website/docs/catalog/websocket.md` and the index show the
unreviewed rest by count.

- The in-scope requirements carry 65 keyword occurrences (section 5.2: 11, section 5.5 with
  subsections: 21, section 7 with subsections: 33; counts from `story:websocket-source-catalogue`),
  29 of them MUST or MUST NOT. Each of these requirements has a status: satisfied by an element of
  the `close` machine or `layout.frame-header`, `unsupported` with a reason, or `not-applicable` with a
  reason. The three section 7.4.1 sentences "1005 / 1006 / 1015 is a reserved value and MUST NOT be
  set as a status code in a Close control frame by an endpoint" (lines 2533, 2540, 2595) are
  `unsupported: ess-gap G1` (design section 7, close codes need guards beyond equality). The three
  `abnf-comment` requirements of section 5.2 (lines 1706, 1710, 1714) have a status.
- `rfcspec check` refuses a profile in which any catalogue requirement (every level) of a section in scope
  (the section or a subsection of one listed in `profile.yaml`) has no overlay status, naming the
  profile and the requirement id; removing the status of one in-scope MUST requirement, or of one in-scope MAY requirement, makes
  `rfcspec check --profile profiles/websocket` exit 1, and a requirement outside the scope without a
  status passes. A test covers both. `profiles/time` passes unchanged (RFC 868 has no BCP 14
  requirement).
- `rfcspec-docs` reads each profile catalogue offline from its locked source through the reader and
  extractor. The index status counts gain `unreviewed`, the catalogue requirements with no overlay
  status: 0 for `time`; for `websocket`, exactly the number of catalogue requirements `rfcspec
  extract` lists outside sections 5.2, 5.5 and 7 (they carry the other 149 keyword occurrences); the
  test computes that number from the extract output of the committed source and asserts the index
  cell equals it, and asserts 0 unreviewed requirements inside the scope. The profile page states the unreviewed count by level
  beside the requirement matrix, which keeps one row per reviewed requirement. A unit test asserts
  both rows of the index and the fixture profile of `story:catalog-index` (no locked source, so no
  unreviewed count, shown as such).
- The page shows RFC 6455 linked to `https://www.rfc-editor.org/rfc/rfc6455`, sections in scope
  `5.2; 5.5; 7`, the `tcp` semantics row, requirement counts by level and by status, ESS gaps G1 and
  G9 with their requirements, and downloads byte-identical to `profiles/websocket/generated/`.
- `task check` runs `rfcspec check --all profiles` (two profiles), `lower --check` and the
  exploration of the `close` model (`"verdict": "passed"`), `generate --check` and `cargo test -p
  rfc6455-websocket`; `task website` builds and `rfcspec-docs provenance` on that build lists
  `/rfcspec/docs/catalog/websocket` in `.well-known/b10x-routes.json`.
- `AGENTS.md` State names `profiles/websocket` and the gate steps for it.

## Domain relations

- Profile -> Status, one-to-many, owned: `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`,
  relation `statuses`; at most one status per profile and requirement (identity
  `rfcspec.profile.StatusKey`). A requirement with no status in the profile is the `unreviewed` case.
- Status -> Requirement, one: `ess/domains/profile.yaml`, entity `rfcspec.profile.Status`, relation
  `requirement`. The rule that every requirement in scope needs a status is stated in that
  entity summary and assigned there to `rfcspec check`.
- Section -> Section parent: `ess/domains/catalogue.yaml`, entity `rfcspec.catalogue.Section`,
  relation `parent`; "in scope" includes the subsections of a listed section.

## Scope

`profiles/websocket/overlay.yaml`, `crates/rfcspec-cli` (the in-scope-MUST-needs-a-status rule as a new function in `src/checks.rs` reading `profile.yaml` sections, called from `fn check` in `src/main.rs`; tests),
`crates/rfcspec-docs` (`catalog.rs` unreviewed counts, `Cargo.toml` adding `rfcspec-doc`,
`rfcspec-extract` and `rfcspec-source`, tests), `Cargo.lock`, `website/docs/catalog/index.md`,
`website/docs/catalog/time.md`, `website/docs/catalog/websocket.md` (regenerated), `AGENTS.md`.

Depends on `story:websocket-frame-codec`: it gives statuses that name the elements the machine and
codec stories create, and edits the same overlay, `crates/rfcspec-cli`, `Cargo.lock` and generated
pages.
