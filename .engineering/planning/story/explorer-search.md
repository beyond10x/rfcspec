---
format: aep.planning-md/3
id: story:explorer-search
kind: story
status: proposed
title: Search and filter the whole RFC index in the explorer by number, title, status, stream, area, source format and requirement levels
relations:
- decomposes: epic:rfc-explorer
- depends_on: story:explorer-rfc-view
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T17:48:52Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
## Story

The explorer front page `/rfcspec/explorer/` lists every RFC of the index and filters the list in the browser: by number or number range, by words in the title, by status, stream, area and source format, and by requirement level (at least one requirement of a chosen level, or none at all). Each result opens the RFC view of `story:explorer-rfc-view`. The navbar, the footer and the RFC index overview link to it.

## Decision: one index file, filtered in the browser

- One generated file, `website/static/data/rfc-index.json`: per RFC its number, title, status, stream, area, date, source format, counts by level and the profiles locking it; no SHA-256 (the RFC view shows it). Measured on the committed summary, its rows without `sha256` are 2,025,670 bytes, 347,929 gzipped (`jq -c "del(.sha256)"` over `bulk/rfc-summary.jsonl`, 2026-11-05). Cap: 3 MiB, by test.
- No server and no search service: the site is static (GitHub Pages). Matching is substring (title, case-insensitive) and equality (status, stream, area, format), with no ranking, stemming or prebuilt full-text index. Fuzzy or full-text search is a later milestone.
- The filter state lives in the query string, so a filtered list is a link.
- The index row, including its profile marks (not an `Rfc` field), is an ESS value type in `ess/domains/catalogue.yaml`. Both sides are generated from it the way `story:explorer-rfc-view` generates its shard types, as `uilab` does (`uilab/Taskfile.yml:25-26`): `ess generate types --target rust` for the Rust writer of `rfc-index.json`, and `--target typescript` for the app. Both are drift-checked.

## Domain relations

- Only `Rfc` fields are read (`title`, `status`, `stream`, `area`, `date`, `format_read`, `requirement_counts`): `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc`. No relation is navigated.
- Which profile locks an RFC: inferable (inferred from `crates/rfcspec-docs/src/bulk.rs`, `locked`, which reads each `profiles/*/rfcspec.lock`). `ess/domains/profile.yaml` gives `rfcspec.profile.Profile` a field `rfcs` (List<ProfileRfc>), not a relation; no ess/1 document declares a Profile -> Rfc relation.

## Acceptance

Before: the only way to find an RFC on the site is the eleven range tables. After: `/rfcspec/explorer/?status=INTERNET%20STANDARD&level=MUST` lists exactly the rows of `bulk/rfc-summary.jsonl` with that status and a MUST count above zero, the count printed above the list equals the count a `jq` filter over the summary gives, and each row links to `?rfc=<n>`.

Checked by:

- `cargo test -p rfcspec-docs`: `rfc-index.json` holds one entry per summary row in number order, the profile marks of RFC 868, 1035, 5905 and 6455, and is under 3 MiB.
- `task explorer`, `vitest` over a fixture index: number and range, title words case-insensitive, status, stream, area, format, level present, no requirements, combined filters, and the query-string round trip.
- `task docs-check` drift-checks the file.
- The types drift check covers the index-row types, Rust and TypeScript.

## Scope

- `ess/domains/catalogue.yaml`: the index-row value type, with the profile marks.
- `crates/rfcspec-docs/src/explorer.rs`: the index file, written through the generated Rust type; `src/bulk.rs`: the link from the RFC index overview.
- The generated Rust types from `ess generate types --target rust` (the index row added to those of `story:explorer-rfc-view`).
- `website/static/data/rfc-index.json` (generated).
- `explorer/src/`: the list view and its filters, with tests; `explorer/src/generated/` for the index row from `ess generate types --target typescript`.
- `Taskfile.yml`: the index-row root in the types generation and its drift check.
- `website/docusaurus.config.ts`: navbar and footer item "RFC explorer", in the `pathname:///` link form.

## Open questions

- The epic says "keyword level counts". Read here as "has at least one requirement of this level" plus "has none". Whether numeric thresholds (at least N MUST) or sorting by a count are wanted is unclear; ask the operator. Either is additive to this story.
