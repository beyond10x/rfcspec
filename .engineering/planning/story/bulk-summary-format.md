---
format: aep.planning-md/3
id: story:bulk-summary-format
kind: story
status: active
title: Summarise rfc-index.xml and RFC texts into the rfcspec-bulk/1 summary and check it offline
relations:
- decomposes: epic:bulk-catalogue
- depends_on: story:websocket-review-matrix
- depends_on: story:proposal-machine-layout
- depends_on: story:websocket-proposal-slice
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/rfcspec-cli/src/bulk.rs
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: cited
  path: crates/rfcspec-cli/tests/bulk.rs
- confidence: cited
  path: crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml
- confidence: cited
  path: crates/rfcspec-extract
- confidence: cited
  path: crates/rfcspec-model
- confidence: cited
  path: crates/rfcspec-source
- confidence: cited
  path: ess/domains/catalogue.yaml
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:52Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:52Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":1}}}
---
## Story

`rfcspec bulk summarise` turns a copy of `rfc-index.xml` and a directory of RFC plain texts into the
bulk summary `rfcspec-bulk/1`: one JSON line per index entry with a TXT format, holding the index
metadata, the SHA-256 of the text read and the counts of its mechanical catalogue requirements by
BCP 14 level. `rfcspec bulk check` validates a summary offline. Both run on a small committed
fixture; fetching the corpus and committing the real summary is `story:bulk-fetch-corpus`, the site
is `story:bulk-site-index`. Stage S2 only: no review, no overlay, no profile is created.

## Acceptance

`rfcspec bulk summarise` over the fixture index and texts writes a summary that `rfcspec bulk check`
accepts, whose RFC 868 row has no counts, whose RFC 1035 row has the SHA-256 of
`profiles/dns/rfcspec.lock` and counts by level equal to those of `rfcspec extract --profile
profiles/dns --rfc 1035`, and whose RFC 6455 row has the SHA-256 of
`profiles/websocket/rfcspec.lock` (`765775326aee…`) and counts by level equal to those of
`rfcspec extract --profile profiles/websocket --rfc 6455`.

- Format. Line 1 is the header
  `{"format":"rfcspec-bulk/1","index_sha256":…,"extractor":…,"rows":N,"rows_sha256":…}`, where
  `rows_sha256` is the SHA-256 (64 lower-case hex digits) over all row lines, each with its
  trailing newline, in file order;
  each further line is one row, ascending by RFC number: `rfc`, `title`, `status` (the index
  `current-status`, verbatim, such as `PROPOSED STANDARD`), `stream`, `area` (omitted when the entry
  has none), `date` (`YYYY-MM` from the index month and year), `obsoletes`, `obsoleted_by`, `updates`,
  `updated_by` (RFC numbers, omitted when empty), `sha256` (64 lower-case hex digits of the text bytes),
  `counts` (an object keyed by the BCP 14 level as the extractor spells it, `MUST NOT`, zeros omitted,
  `{}` for none) and `lossy: true` only when the text is not valid UTF-8 and was read with
  replacement characters. Serialisation is byte-identical across two runs (a test asserts it).
- `extractor` is a new constant `EXTRACTOR_VERSION` in `rfcspec-extract` (starting at 1), to be raised
  whenever extraction output changes; the workspace version (0.0.0) carries no information. The
  constant is introduced when the bulk wave runs, after `epic:dns-profile`, so it starts at 1 for
  the extractor that wave and the dns reader or extractor fixes leave; extractor changes made
  before it exists need no bump.
- An index entry without a TXT format (RFC 8, 9, 51, 418, 500, 530 and 598 in the 4 November 2026
  index) and every `rfc-not-issued-entry` produce no row. A TXT entry whose text is missing from the
  directory makes `summarise` exit 1 naming the RFC number and the expected path `<dir>/rfc<N>.txt`.
- Counts are computed by `rfcspec_doc::read` and `rfcspec_extract::extract`, the functions `rfcspec
  extract` uses, so the 2119 boilerplate paragraph is excluded and `abnf-comment` requirements count.
- `rfcspec bulk check --summary <file> --profiles <dir>` exits 1 naming the line when: the header
  format is not `rfcspec-bulk/1`; `rows` differs from the number of rows; `rows_sha256` differs
  from the SHA-256 recomputed over the row lines (so editing any field of any row, counts
  included, is refused whether or not its RFC is locked); numbers are not strictly
  ascending (a duplicate is named as such); a `sha256` is missing or not 64 lower-case hex digits; a
  `counts` key is not one of the eleven levels or a value is not a positive integer; `extractor`
  differs from `EXTRACTOR_VERSION`; or an RFC locked in `<dir>/*/rfcspec.lock` has no row, a row
  whose `sha256` differs from the lock, or counts that differ from extracting the locked source
  (rows of locked RFCs, 868, 1035 and 6455 under `profiles/`, are recomputed from their locked
  text in addition to the `rows_sha256` check). It reads no network. One test per refusal, each
  mutating one line of a valid summary; the `rows_sha256` test edits the counts of a row whose RFC
  no profile locks (a copy of the fixture summary with one extra synthetic row and the header
  recomputed, then that row's counts changed).
- Fixture: `crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml`, the verbatim entries for RFC 8,
  868, 1035 and 6455 and one `rfc-not-issued-entry` cut from the real index inside its root element
  (a few KB, no RFC text); the texts are read from `profiles/time/sources/rfc868.txt`,
  `profiles/dns/sources/rfc1035.txt` and `profiles/websocket/sources/rfc6455.txt`, copied by the
  test into a temporary directory. The summary has three rows; RFC 6455 has `updated_by` 7936, 8307,
  8441, stream `IETF`, area `art`, date `2011-12`; RFC 868 has status `INTERNET STANDARD`, stream
  `Legacy`, no area; RFC 1035 has the lock digest and counts named above.
- `ess/domains/catalogue.yaml` declares the summary row as what it is, the existing entity
  `rfcspec.catalogue.Rfc` read in bulk: `Rfc` gains `requirement_counts` (a list of a new struct
  `rfcspec.catalogue.LevelCount`, level and count) and the view `rfcspec.catalogue.Rfcs` gains status,
  stream, area, digest and `requirement_counts`; the `digest` summary also names the bulk text. No
  new entity. `ess specify validate --path ess` passes.

## Domain relations

- Summary row -> Rfc, one-to-one, the row is a projection and owns nothing:
  `ess/domains/catalogue.yaml`, view `rfcspec.catalogue.Rfcs`, `source: rfcspec.catalogue.Rfc`.
- Requirement -> Rfc, many-to-one: `ess/domains/catalogue.yaml`, entity `rfcspec.catalogue.Requirement`,
  relation `rfc`. A count by level counts the requirements whose `rfc` is the row.
- Rfc -> Rfc update edges (obsoletes, obsoleted by, updates, updated by), zero or many, no ownership:
  inferable (inferred from `ess/domains/catalogue.yaml`, entity `rfcspec.catalogue.Rfc`, fields
  `obsoletes` to `updated_by` typed `List<Integer>`, whose summary says they name RFC numbers, not
  catalogue entries, and may name an RFC never read; no `relations:` entry). The summary copies the
  numbers and does not require a row for each.
- Profile -> Rfc, one-to-many, referencing: inferable (inferred from `ess/domains/profile.yaml`, type
  `rfcspec.profile.ProfileRfc`, field `rfc` typed `rfcspec.catalogue.RfcNumber`, no `relations:`
  entry). The check reads the profile locks for that reason.

## Scope

`crates/rfcspec-model` (the `rfcspec-bulk/1` header and row types), `crates/rfcspec-source` (an
`index` module parsing `rfc-index.xml` with `roxmltree`; parsing only, no network in this story),
`crates/rfcspec-extract` (`EXTRACTOR_VERSION`), `crates/rfcspec-cli` (`bulk summarise` and `bulk
check` in `src/main.rs`, the check in a new `src/bulk.rs`, `tests/bulk.rs`,
`tests/fixtures/bulk/rfc-index.xml`), `ess/domains/catalogue.yaml`, `Cargo.lock`. No `Taskfile.yml`
change: the gate step arrives with the committed summary in `story:bulk-fetch-corpus`.

Depends on `story:proposal-machine-layout` and `story:websocket-proposal-slice`, the two last stories of
`epic:ai-proposals`, which are not ordered against each other: it shares `Cargo.lock` and
`crates/rfcspec-cli/src/main.rs` with them and takes the whole `crates/rfcspec-model` and
`crates/rfcspec-extract` directories they edit. Depends on `story:websocket-review-matrix`, the last story of the WebSocket wave: it edits the same
`crates/rfcspec-cli/src/main.rs`, `Cargo.lock` and extraction (`crates/rfcspec-extract` through
`story:websocket-source-catalogue`), and the RFC 6455 counts this story checks are those of the
extractor that wave leaves. The RFC 1035 row reads `profiles/dns`, which `epic:dns-profile`
commits before this epic runs (recorded order: websocket-frames, dns-profile, ai-proposals,
bulk-catalogue).
