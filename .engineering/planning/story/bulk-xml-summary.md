---
format: aep.planning-md/3
id: story:bulk-xml-summary
kind: story
status: active
title: Bulk fetch and summarise RFCXML where the index lists it, recording each row's source format
relations:
- decomposes: epic:rfcxml-reader
- depends_on: story:rfcxml-bcp14-extract
scope:
- confidence: inferred
  path: AGENTS.md
- confidence: inferred
  path: crates/rfcspec-cli/src/bulk.rs
- confidence: inferred
  path: crates/rfcspec-cli/src/main.rs
- confidence: inferred
  path: crates/rfcspec-cli/tests/bulk.rs
- confidence: inferred
  path: crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml
- confidence: inferred
  path: crates/rfcspec-docs/src/bulk.rs
- confidence: inferred
  path: crates/rfcspec-model/src/bulk.rs
- confidence: inferred
  path: crates/rfcspec-source/src/corpus.rs
- confidence: inferred
  path: crates/rfcspec-source/src/index.rs
- confidence: inferred
  path: ess/domains/catalogue.yaml
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T15:46:56Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T16:51:54Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
---
## Story

The bulk path reads RFCXML where the index lists it: `rfcspec bulk fetch` downloads `rfc<N>.xml` for
every index entry with an `XML` format (1,390 RFCs, RFC 8650 onward, design section 2.1) and
`rfc<N>.txt` for the rest, and `rfcspec bulk summarise` counts the requirements of each RFC from the
source it fetched, recording which. Shown on the committed fixture; regenerating the committed
summary and the RFC index pages is `story:bulk-xml-regenerate`.

Decisions taken here (reversible):

- A row gains `format`, written only as `"format":"xml"`; a row without it was read from text. Text
  rows serialize byte for byte as before, so the committed `bulk/rfc-summary.jsonl` stays valid
  under `rfcspec bulk check` until it is regenerated, and the format name stays `rfcspec-bulk/1`.
- `EXTRACTOR_VERSION` stays 1 (no text catalogue changes; see `story:rfcxml-bcp14-extract`).
- An RFC with XML is read from XML only; its text is not fetched for the bulk cache, and its row
  `sha256` is that of the XML file.
- Rows: one per index entry with a `TXT` or an `XML` format (before: `TXT` only).

## Acceptance

`rfcspec bulk summarise` over the fixture index, extended with the RFC 9220 entry, the four locked
texts (RFC 868, 1035, 5905 and 6455, as `story:ntp-source-catalogue` leaves the test) and the RFC 9220 XML fixture writes a summary that `rfcspec bulk check` accepts, whose RFC 9220
row has `"format":"xml"`, `sha256` `8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae`
and counts `{"MAY":1,"SHOULD":1}`, and whose RFC 868, 1035, 5905 and 6455 rows are byte-identical to
those of the summary before this story. Independently: `rfcspec bulk fetch` against a local server
serving the extended fixture index writes `<dir>/xml/rfc9220.xml` (before: it fetches only
`rfc9220.txt`), and `rfcspec bulk check` exits 1 naming the lock and both formats when a profile locks RFC 9220 as
`sources/rfc9220.xml` and its row has no `"format":"xml"` (before: a row has no `format` key to check).

- Index. `IndexEntry::has_xml` is true when the formats include `XML`. The fixture
  `crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml` gains the RFC 9220 entry as the 2026-11-05
  index gives it (formats `HTML`, `TXT`, `PDF`, `XML`; the existing fixture holds no `XML` format).
- Fetch. Against a 127.0.0.1 server, `rfcspec bulk fetch --cache <dir> --index-url ... --base-url ...`
  writes `<dir>/xml/rfc9220.xml` and no `<dir>/txt/rfc9220.txt`, and `<dir>/txt/rfc<N>.txt` for the
  entries without XML; a second run downloads nothing; `--refresh` downloads both kinds again; a
  failed XML download exits 1 naming the RFC and its `.xml` URL. The cache refusal inside a Git
  working tree is unchanged.
- Summarise. With `--cache`, reads `<cache>/xml/`; with `--index` and `--texts`, a new `--xml <DIR>`
  (required with `--index`) names the XML directory. A missing XML for an entry with an `XML` format
  exits 1, writing nothing, naming the RFC and the expected path `<xml>/rfc<N>.xml`. Counts come from
  `rfcspec_doc` reading XML and `rfcspec_extract::extract`, the functions `rfcspec extract` uses. An
  entry with `XML` and no `TXT` gets a row (a test writes such an index).
- Check. `rfcspec bulk check` exits 1 naming the line when a row has a `format` other than `xml`;
  when an RFC is locked as `sources/rfc<N>.xml` and its row has no `"format":"xml"`, or locked as
  `.txt` and its row has one (naming the lock and both formats); and, as before, when a locked
  RFC's digest or counts differ, the counts now recomputed by the reader of the locked format. Tests
  build a temporary profiles directory locking the RFC 9220 XML.
- `rfcspec bulk check --summary bulk/rfc-summary.jsonl --profiles profiles` passes on the committed
  summary unchanged; `rfcspec-docs generate --check` passes; `cargo test -p rfcspec-source -p
  rfcspec-cli -p rfcspec-docs`, strict Clippy and `task check` pass.

## Domain relations

- Summary row -> Rfc, one-to-one, a projection: `ess/domains/catalogue.yaml`, view
  `rfcspec.catalogue.Rfcs`, `source: rfcspec.catalogue.Rfc`. This story adds the view field
  `format_read` (the entity field of type `rfcspec.catalogue.SourceFormat` already exists) and
  changes the `digest` summary of `rfcspec.catalogue.Rfc` from "the plain text read in bulk" to the
  source read in bulk. No new noun.
- Profile lock -> source file, one file per RFC per profile: inferable (inferred from
  `crates/rfcspec-model/src/lib.rs:82-83`, `Lock::upsert`; no `ess/1` document declares the lock).
  The format check rests on it: one locked format per RFC.

## Scope

`crates/rfcspec-source/src/index.rs` (`has_xml`), `crates/rfcspec-source/src/corpus.rs` (XML download
into `<cache>/xml/`), `crates/rfcspec-model/src/bulk.rs` (`BulkRow.format`), `crates/rfcspec-cli/src/bulk.rs`
(summarise and check), `crates/rfcspec-cli/src/main.rs` (`--xml` on `bulk summarise`, help texts),
`crates/rfcspec-cli/tests/bulk.rs`, `crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml`,
`crates/rfcspec-docs/src/bulk.rs` (reads rows with `format`; pages unchanged here),
`ess/domains/catalogue.yaml` (view field, `digest` summary), `AGENTS.md`.

Depends on `story:rfcxml-bcp14-extract`: XML counts are that story's extraction, and the test
reads the fixture of `story:rfcxml-source-reader`.

## Shared files and order

`crates/rfcspec-cli/tests/bulk.rs`, `crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml`,
`crates/rfcspec-docs/src/bulk.rs` and `AGENTS.md` are also changed by `story:ntp-source-catalogue` (RFC 5905 in the fixture,
four locked texts, `rows` 4). The counts above are those after that story; they are re-read from
`main` before this story starts.

The RFCXML waves start after the epic:ntp-profile pull request merges into `main` (decision taken by the coordinating session, 2026-11-05). `epic:ntp-profile` reaches this store with that merge; `depends_on: epic:ntp-profile` is then recorded on `story:rfcxml-source-reader`, which every later story in this epic depends on, before the first RFCXML wave.
