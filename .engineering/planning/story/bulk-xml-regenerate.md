---
format: aep.planning-md/3
id: story:bulk-xml-regenerate
kind: story
status: proposed
title: Regenerate the committed bulk summary from RFCXML where it exists and show each row's source on the RFC index pages
relations:
- decomposes: epic:rfcxml-reader
- depends_on: story:bulk-xml-summary
scope:
- confidence: inferred
  path: AGENTS.md
- confidence: inferred
  path: README.md
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: bulk/rfc-summary.jsonl
- confidence: inferred
  path: crates/rfcspec-docs/src/bulk.rs
- confidence: inferred
  path: website/docs/catalog/rfcs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T15:46:57Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":4}}}
---
## Story

The committed bulk summary and the RFC index pages under `/docs/catalog/rfcs` use RFCXML where it
exists: `task bulk-regenerate` runs `bulk fetch` and `bulk summarise` as `story:bulk-xml-summary`
left them, the summary is committed, and the RFC index pages show for each RFC whether its counts
came from XML or from text.

No profile under `profiles/` gains RFC 9220 here. Design section 12.1 places it in the websocket
profile, but a profile RFC with an empty section list reads as "the whole RFC" in scope on the
catalogue page (`crates/rfcspec-docs/src/catalog.rs`, `rfcs`), and the profile has one overlay for
RFC 6455 only; deciding what of RFC 9220 the profile covers (layering over an HTTP/3 stream,
section 6 semantics) is profile work, not reader work.

## Acceptance

Before this story no row of the committed summary carries `format` and the RFC index pages have no
`Source` column. After it, the RFC index pages under `website/docs/catalog/rfcs/` show a `Source`
column (`XML` or `text`) for every row, on a clean checkout without network `task check` exits 0,
and the committed `bulk/rfc-summary.jsonl`
has one row per index entry with a `TXT` or `XML` format (9,847 for the index whose SHA-256 is
`dd963eda48804cfe9f2b225016c01e8ed9a7b60699d47f4c24fe0b8f4f3fe04c`), of which the 1,390 with an
`XML` format carry `"format":"xml"`, the RFC 9220 row among them with SHA-256
`8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae`.

Counts read on 2026-11-05 from `https://www.rfc-editor.org/rfc-index.xml` (SHA-256 as above, the
same index the committed summary records): 9,847 entries with `TXT`, 1,390 with `XML`, none with
`XML` and no `TXT`. If the index has changed when the story runs, the commit names the new SHA-256
and counts and the values here are re-read.

- Regenerate. `task bulk-regenerate` (networked, never run by `task check`) writes the summary; the
  commit message names the index SHA-256, the row count, the XML row count and the time of the
  fetch. No RFC text, XML or index file is committed, and the summary stays under 8 MiB (the
  existing size test).
- Rows. The 8,457 text rows keep their bytes unless the RFC Editor serves other text; the RFC 868,
  1035, 5905 and 6455 rows match `profiles/time`, `profiles/dns`, `profiles/ntp` and `profiles/websocket` as before (`bulk
  check`). The 1,390 XML rows replace text rows of the same RFCs; their counts are what the XML
  extraction gives and are not compared with the text counts they replace.
- Pages. `task docs-generate` regenerates `website/docs/catalog/rfcs/`: each range page gains a
  `Source` column, `XML` or `text`, linked to the RFC Editor file the counts came from
  (`https://www.rfc-editor.org/rfc/rfc9220.xml` for RFC 9220); the overview says the index lists
  every RFC published as plain text or RFCXML, that counts come from RFCXML where it exists, and
  how many rows each source gave. `rfcspec-docs generate --check` passes on the result.
- `cargo test -p rfcspec-docs -p rfcspec-cli`, strict Clippy and `task check` pass.

## Domain relations

- Summary row -> Rfc, one-to-one, a projection: `ess/domains/catalogue.yaml`, view
  `rfcspec.catalogue.Rfcs`, `source: rfcspec.catalogue.Rfc`; the page column is the view field
  `format_read` that `story:bulk-xml-summary` adds, of type `rfcspec.catalogue.SourceFormat`.
- Profile -> Rfc, one-to-many, referencing: inferable (inferred from `ess/domains/profile.yaml`, type
  `rfcspec.profile.ProfileRfc`, field `rfc`, no `relations:` entry). Read only to mark the RFCs a
  profile holds, as before.

## Scope

`bulk/rfc-summary.jsonl` (generated), `crates/rfcspec-docs/src/bulk.rs` (the `Source` column and the
overview text, tests), `website/docs/catalog/rfcs/` (generated), `Taskfile.yml` (the
`bulk-regenerate` description), `AGENTS.md` and `README.md` (the `bulk/` row).

Depends on `story:bulk-xml-summary`: it runs that story's fetch and summarise and reads its
`format` field.

## Shared files and order

`website/docs/catalog/rfcs/` (`05000.md` and `index.md`), `crates/rfcspec-docs/src/bulk.rs`, `Taskfile.yml`
and `AGENTS.md` are also changed by epic:ntp-profile (`story:ntp-source-catalogue` marks RFC 5905 with
`[ntp]`; `story:ntp-header-codec`, `story:ntp-exchange-machine` and `story:ntp-poll-machine` edit
`Taskfile.yml`; `story:ntp-timer-model` and `story:ntp-poll-machine` edit `AGENTS.md`). The pages
are regenerated from `main` after the NTP merge, so they carry the `[ntp]` marker.

The RFCXML waves start after the epic:ntp-profile pull request merges into `main` (decision taken by the coordinating session, 2026-11-05). `epic:ntp-profile` reaches this store with that merge; `depends_on: epic:ntp-profile` is then recorded on `story:rfcxml-source-reader`, which every later story in this epic depends on, before the first RFCXML wave.
