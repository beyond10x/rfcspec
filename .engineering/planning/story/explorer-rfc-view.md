---
format: aep.planning-md/3
id: story:explorer-rfc-view
kind: story
status: proposed
title: 'Open any RFC of the index in the RFC explorer: metadata, requirements by section and level, update graph and references as links'
relations:
- decomposes: epic:rfc-explorer
- depends_on: story:xref-reference-extract
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T17:48:52Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
---
## Story

Any RFC of the index opens in the RFC explorer at `/rfcspec/explorer/?rfc=<n>`: its index metadata (title, status, stream, area, date, source format, SHA-256), its BCP 14 requirements counted by section and level, the RFCs it obsoletes, updates, is obsoleted by and is updated by, and, for an RFC read from RFCXML, the RFCs it cites (the References of `story:xref-reference-extract`). Each RFC number links to its own explorer view when the summary has a row for it.

## Decision: per-range JSON read by one Vue page, not 9,847 generated pages

Decided at decomposition, 2026-11-05. Per-RFC data is generated JSON, one file per thousand RFC numbers (the ranges the RFC index pages use, 11 files), read by one browser page. No Markdown page per RFC. Reasons:

- 9,847 MDX pages would each pass through the Docusaurus build, and `website/sidebars.ts` autogenerates the `catalog` directory, so each would also enter the sidebar shipped with every page. The Pages job has `timeout-minutes: 15` (`.github/workflows/pages.yml:20`); its last five runs took 2 to 3 minutes (`gh run list --workflow pages.yml`, 2026-11-05). The build time of 9,847 pages was not measured.
- One file per RFC would be 9,847 committed generated files rewritten by every bulk regeneration. One per range keeps the boundary of the range pages and their size test.
- Search and filter (`story:explorer-search`) need code in the browser anyway, so the RFC view is one more route of the same app.
- Browser code is Vue 3 with TypeScript, Vite and pnpm (workspace rule); the documentation site stays Docusaurus. The explorer is a standalone Vite app, as `website/static/pitch/` is a standalone page. It is built into `website/static/explorer/` (ignored by Git) before the Docusaurus build and linked with `pathname:///rfcspec/explorer/...`, the link form the downloads use. CI installs pnpm with `corepack enable`, the version pinned by `packageManager` in `explorer/package.json`.

## Decision: what the per-RFC data holds

- Per section: its number or anchor, its title, and its requirement counts by level. Requirements outside any section the reader recognises are counted under one entry without a section.
- No requirement sentences for an RFC no profile locks. AGENTS.md: "no RFC text or RFCXML, `rfc-index.xml` or cache file is committed"; the sentences of the 240,000 or so bulk requirements are RFC text in bulk. The view links each section to the RFC Editor rendering instead.
- The section data comes from the same `rfcspec bulk summarise` run as the summary, written to a sidecar `bulk/rfc-sections.jsonl` with its own header (index SHA-256, extractor version, row count, rows SHA-256). `rfcspec-bulk/1` rows keep their shape and the range pages their size. The sidecar is regenerated only by `task bulk-regenerate` (network) and checked offline by `rfcspec bulk check`.
- The sidecar rows also carry each RFCXML RFC's References. The sidecar writer in `crates/rfcspec-cli/src/bulk.rs` reads them from `Document.references`, which `story:xref-reference-extract` puts on the `Document` that `bulk summarise` already reads for its level counts (`counts`, `rfcspec_doc::read_format`). Text rows carry none. The per-range shards take their References from the sidecar, so the 9220 shard holds the References `rfcspec read` gives.
- The JSON shapes are ESS value types in `ess/domains/catalogue.yaml` over `Rfc`, `Section`, `LevelCount` and `Reference`; no new entity. Both sides are generated from them, as `uilab` does (`uilab/Taskfile.yml:25-26`): `ess generate types --target rust` for the Rust shard writer, which writes the shards through the generated types, and `ess generate types --target typescript` (ess 0.86.0 has the target) for the TypeScript declarations of the app. Both are drift-checked.

## Domain relations

- `Rfc -> Section`, one-to-many, Rfc owns: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc` relation `sections` (owns, many, via `rfc`).
- `Section -> Requirement`, one-to-many, Section owns: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Section` relation `requirements` (owns, many, via `section_id`).
- `Rfc -> Reference`, one-to-many, Rfc owns, rebuilt when the Rfc is re-read: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc` relation `references` (owns, many, via `rfc`).
- `Reference -> Section` (the citing section), many-to-one, the far side may be absent: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Reference` relation `section` (references, one, via `section_id`).
- Reference target and update graph are fields, not relations: `Reference.target_rfc` (Integer) and `Rfc.obsoletes`, `obsoleted_by`, `updates`, `updated_by` (List<Integer>) name RFC numbers that may never have been read (`ess/domains/catalogue.yaml`, their field summaries). A number is therefore a link only when the summary has a row for it, as on the range pages (`crates/rfcspec-docs/src/bulk.rs`, `Links::rfc`).

## Acceptance

Before: `/rfcspec/explorer/?rfc=9220` does not exist and the RFC index pages link each number only to the RFC Editor. After: the built site at `/rfcspec/explorer/?rfc=9220` shows RFC 9220 with its sections, its requirement counts by section and level and its References, among them RFC 8441 (normative) linked to `?rfc=8441`.

Checked by:

- `task docs-check` drift-checks 11 range files under `website/static/data/rfcs/` that hold every row of `bulk/rfc-summary.jsonl` once.
- `?rfc=6455` shows updated-by 7936, 8307 and 8441 as links.
- The types drift check fails when the generated Rust or TypeScript types differ from what `ess/domains/catalogue.yaml` determines.
- `cargo test -p rfcspec-docs`: 11 shards, every summary row once in number order; the RFC 9220 references equal those `rfcspec read` gives on `crates/rfcspec-doc/tests/fixtures/rfc9220.xml`; every shard under 2 MiB and all together under 16 MiB.
- `rfcspec bulk check` refuses a sidecar whose header disagrees with its rows or whose RFC numbers differ from the summary.
- `task explorer`: `vue-tsc`, `vitest` (the RFC view renders a fixture shard; an unknown number reads "not in the index"), `vite build`.
- The range-page test still holds with its 4 MiB cap, with an explorer link added to each RFC cell (about 50 bytes a row, about 0.5 MB on the 2.5 MB of today).

## Scope

- `crates/rfcspec-model/src/bulk.rs`: sidecar header and row types.
- `crates/rfcspec-cli` (`bulk summarise`, `bulk check`) and its tests; the sidecar writer is in `crates/rfcspec-cli/src/bulk.rs` and reads `Document.references` for XML rows.
- `bulk/rfc-sections.jsonl` (new), written by `task bulk-regenerate`; the commit names the index SHA-256, the row count and the fetch time, as AGENTS.md requires.
- `ess/domains/catalogue.yaml`: the explorer value types.
- `crates/rfcspec-docs/src/explorer.rs` (new), `src/main.rs`, `src/bulk.rs` (explorer link per row); `website/static/data/rfcs/*.json` (generated).
- `explorer/` (new): Vue 3, TypeScript, Vite, pnpm; `explorer/src/generated/` from `ess generate types --target typescript`.
- The generated Rust types from `ess generate types --target rust`, used by `crates/rfcspec-docs/src/explorer.rs`.
- `Taskfile.yml` (`explorer`, the types drift check, `website` building the explorer first), `.github/workflows/pages.yml` (pnpm and the explorer build before the Docusaurus build), `.gitignore`.
- `AGENTS.md`: rows for `explorer/` and the sidecar.

## Open questions

- The size of the section data is unknown until the regeneration runs; the caps above are a budget, not a measurement. If a shard exceeds 2 MiB, stop and report the sizes instead of raising the cap.
- Answered: the unified project-site publication (`.github/workflows/b10x-docs-site.yml`, which calls `beyond10x/website` `project-site.yml`) carries `website/build/explorer/` and `website/build/data/` through unchanged. Session `b10x-atlas` answered on 2026-11-05, reading website `origin/main` at the pinned `7d3d414`: both pass through with no size limit, file-count limit or path allowlist (`project-site.yml:171-236`); the only ceiling is GitHub Pages' 1 GB; the provenance step (`pages.yml:40`) runs after the build, so it covers the explorer files.
- Whether the RFC Editor HTML rendering has a `#section-<n>` anchor for every plain-text RFC cannot be read from the repository. Where it has none, the section link falls back to the RFC page.
- The sidecar counts in full against the common Gates scan limit in every commit that changes it. Record its size in the commit.
