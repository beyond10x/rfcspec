---
format: aep.planning-md/3
id: story:catalog-recorded-runs
kind: story
status: implemented
title: Record verify runs as reports and show the latest verdict in the catalogue
relations:
- decomposes: epic:public-protocol-catalog
- depends_on: story:catalog-semantics
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli
- confidence: cited
  path: crates/rfcspec-conformance
- confidence: cited
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: cited
  path: ess/domains/profile.yaml
- confidence: cited
  path: profiles
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/time.md
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T23:34:09Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-04T23:34:09Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-04T23:50:42Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Story

`rfcspec verify --report <path>` records a run as a deterministic report, and the catalogue shows
the newest committed report of each profile as its conformance verdict
(`decision-blocker:catalog-conformance-verdict`).

## Acceptance

A test run of `rfcspec verify --report` against the in-process reference server writes a report
that `rfcspec check` accepts and the catalogue renders as the overall verdict, target and date.
`profiles/time` has no committed report and shows "no recorded run", and a report whose overlay
digest is stale fails `task check`.

- `rfcspec verify --profile P --target T --report profiles/<name>/runs/<target-slug>.json`
  writes a report (`format: rfcspec-run/1`): the target as given, UTC time of the run, the rfcspec
  version, the SHA-256 of the overlay, one row per requirement with its verdict (the verify
  verdicts), and the overall verdict by the design section 11 rule. The slug is derived from
  the target (for example `tcp-127.0.0.1-37`) and a test asserts its derivation. Without
  `--report` the behaviour and output of verify are unchanged.
- Deterministic: the same target, time, version and rows give the same bytes (fixed key order,
  trailing newline). A test injects the time through the library API, runs twice and compares
  bytes; the command line takes the clock.
- Tests run `verify --report` against the in-process correct server of `story:rfc868-verify-probe`
  into a scratch directory under the workspace `target/`, and against the 5-octet server; the
  overall verdicts are pass and fail. No report is committed for `profiles/time`, because no
  reproducible command produces one.
- `rfcspec check --profile P` fails, naming the file, when a report under `P/runs/` has an
  overlay digest different from the current `overlay.yaml`, or does not parse. A test covers both
  with a fixture profile; `profiles/time` (no `runs/`) passes.
- The index gains a "Conformance" column and the profile page a "Conformance" section: the
  newest report by run time, with overall verdict, target and date (UTC, `YYYY-MM-DD`), linking
  its file on GitHub (`page::SOURCE`); otherwise "no recorded run". A unit test uses a fixture
  profile holding two reports and asserts the newer one is shown; `time` shows "no recorded run".
- Round trip: one test copies `profiles/time` into a scratch directory under the workspace
  `target/`, runs `rfcspec verify --report` there against the in-process correct server, then
  `rfcspec check` on the copy exits 0 and `rfcspec-docs` rendering of the copy shows verdict pass,
  that target and that date; after one byte of the copy's `overlay.yaml` changes, `rfcspec
  check` on it exits 1 naming the report.
- `task check` runs `rfcspec check` on every directory under `profiles/` (`rfcspec check
  --all profiles`), so a stale report in any profile fails it; a test asserts `--all` visits
  each profile and fails when one of them fails.
- Stage: a profile with at least one report under `runs/` reports stage S7 in the index (the
  stage rule of `story:catalog-index` extended); a fixture test asserts S7, `time` stays S6.

## Domain relations

- Profile -> recorded run, one-to-many, zero allowed, the profile directory owns `runs/`. Inferable
  (inferred from the decision recorded in `decision-blocker:catalog-conformance-verdict`; no
  `ess/1` document declares a run).
- Run -> overlay, one, by digest: the report pins the overlay it was run against. Inferable
  (inferred from the same decision).
- Run row -> Requirement, one per requirement: `ess/domains/profile.yaml`, entity
  `rfcspec.profile.Status`, relation `requirement`, read through the status the row reports on.

A run is a new noun with no `ess/` declaration. `AGENTS.md` requires one before a story is
written around it, so this story declares it in `ess/domains/profile.yaml` (or links reports as
`rfcspec.profile.Evidence` on statuses) before code, and `ess specify validate --path ess` passes.

## Scope

`crates/rfcspec-conformance` (report type and writer), `crates/rfcspec-cli` (`verify --report`,
`check` stale-run rule), `crates/rfcspec-model` if the report type lives there, `ess/domains/profile.yaml`,
`crates/rfcspec-docs` (`catalog.rs` index column and page section, tests, fixtures), `Taskfile.yml` (`check --all`),
`website/docs/catalog/index.md` and `time.md` (regenerated).

Depends on `story:catalog-semantics`: both change `crates/rfcspec-cli` `check`,
`crates/rfcspec-docs/src/catalog.rs`, `ess/domains/profile.yaml` and possibly `crates/rfcspec-model`,
so they run one after the other.
