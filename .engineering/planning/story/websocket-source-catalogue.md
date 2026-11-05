---
format: aep.planning-md/3
id: story:websocket-source-catalogue
kind: story
status: implemented
title: 'Lock, read and extract RFC 6455: page-split paragraphs, sections, figures, BCP 14 requirements with section ids'
relations:
- decomposes: epic:websocket-profile
scope:
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-doc
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/profile.yaml
- confidence: cited
  path: profiles/websocket/rfcspec.lock
- confidence: cited
  path: profiles/websocket/sources/rfc6455.txt
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/websocket.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T00:11:32Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T00:11:32Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T00:42:15Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Story

`rfcspec fetch --rfc 6455` locks RFC 6455 into a new profile `profiles/websocket`, and the reader
and the extractor learn what the first BCP 14 RFC of the catalogue needs and RFC 868 did not:
paragraphs continued across a page break, numbered sections, multi-line figures (the section 5.2
packet diagram and bit ABNF), the RFC 2119 boilerplate paragraph, keywords inside ABNF comments,
and requirement ids `R<section>-<ordinal>` (design section 5). The profile enters the protocol
catalogue with an overlay that reviews nothing yet.

## Acceptance

`rfcspec extract --profile profiles/websocket --rfc 6455` lists requirements whose texts carry the
214 BCP 14 keyword occurrences of RFC 6455 outside its boilerplate paragraph, each with its section
id, and `rfcspec check --all profiles` passes on two profiles.

Values below were read from `https://www.rfc-editor.org/rfc/rfc6455.txt` on 2026-11-05 (3979 lines,
SHA-256 `765775326aee0ecca9b04bde3fd1f52932d498e33e34e428bd61b8a24da0fa3b`). If the fetched file has
another digest, the lock records what was fetched and every count here is re-read before the story
is accepted.

- `rfcspec fetch --rfc 6455 --profile profiles/websocket` writes `sources/rfc6455.txt` and a lock
  entry `{rfc: 6455, file: sources/rfc6455.txt, sha256: <hex>}`; a second run changes no byte.
- Furniture: the reader removes the 71 `[Page N]` footers (pages 1 to 71), the 71 form-feed lines
  and the 70 header lines `RFC 6455 ... December 2011`; no block contains `[Page` or starts with
  `RFC 6455 `. The RFC 868 reader tests pass unchanged.
- A paragraph whose last line before a page break has no closing punctuation continues after the
  break: the sentence "The list of extensions that are needed SHOULD appear in the /reason/ part of
  the Close frame." (section 7.4.1, status code 1010) is one requirement with span 2572-2583. The
  overlay quote check skips furniture lines inside a span; a test quoting that sentence at that span
  passes `check`.
- Every block carries the number of the section it lies in, read from the 88 numbered headings
  (14 top-level) from `1.  Introduction` (line 175) to `14.2.  Informative References` (line 3856);
  blocks before line 175 carry none.
- The section 5.2 packet diagram (lines 1535-1552) and the bit ABNF from `ws-frame` (line 1687) to
  line 1783 are figures, not paragraphs; the field description at line 1561 ("MUST be 0 unless an
  extension is negotiated ...") stays a paragraph.
- Capitals-text requirements come from paragraphs, skipping the boilerplate paragraph (lines
  658-660, which holds one each of the ten keywords it lists). Keywords inside ABNF comments of
  figures are requirements with source `abnf-comment`: exactly 5, at lines 1409 (section 4.3),
  1706, 1710, 1714 (section 5.2, `frame-rsv1` to `frame-rsv3`, "MUST be 0 unless negotiated
  otherwise") and 2678 (section 9.1).
- Counted as disjoint whole words (a `MUST NOT` is not also a `MUST`), the requirement texts carry
  113 MUST, 25 MUST NOT, 24 SHOULD, 4 SHOULD NOT, 47 MAY, 1 OPTIONAL and no REQUIRED, SHALL, SHALL
  NOT, RECOMMENDED or NOT RECOMMENDED: 214 in all. (The whole file has 140 MUST of which 26 in MUST
  NOT, 30 SHOULD of which 5 in SHOULD NOT, 48 MAY, 2 OPTIONAL, 1 REQUIRED, 1 RECOMMENDED, 1 SHALL and
  1 SHALL NOT, as design section 5 states; the boilerplate holds the other 10.) A test asserts each
  count.
- In the sections the profile scopes: section 5.2 carries 10 MUST (3 of them the ABNF comments) and
  1 MAY; section 5.5 with 5.5.1 to 5.5.3 carries 6 MUST, 3 MUST NOT, 3 SHOULD, 9 MAY; section 7 with
  its subsections carries 6 MUST, 4 MUST NOT, 11 SHOULD, 3 SHOULD NOT, 9 MAY. A test asserts these.
- Ids are `R<section>-<ordinal>`, the ordinal counting requirements of that section in document
  order (ESS `rfcspec.catalogue.ItemId`): `R5.2-1` is "The first fragment MAY also be the final
  fragment." (lines 1556-1557).
- A requirement level is the keyword that occurs first in the sentence (the longest keyword at that
  position), not the first of a fixed list: "The server MUST close the underlying TCP connection
  immediately; the client SHOULD wait ..." (section 5.5.1, lines 2025-2029) has level MUST.
- `profiles/websocket/profile.yaml`: name `websocket`, RFC 6455 "The WebSocket Protocol", sections
  `["5.2", "5.5", "7"]`, a section including its subsections. `profiles/websocket/overlay.yaml`:
  `rfcspec-overlay/1`, rfc 6455, `kind: K1`, no requirements, layouts, machines or findings.
- `rfcspec check --all profiles` checks `time` and `websocket` and exits 0. On `profiles/time`,
  `check`, `lower --check` and `generate --check` pass with no change under `profiles/time/`.
- `rfcspec-docs generate` lists `time` and `websocket` (RFC 6455 The WebSocket Protocol, K1, stage
  S4, no reviewed requirements) in `website/docs/catalog/index.md` and writes
  `website/docs/catalog/websocket.md`; `rfcspec-docs generate --check` passes.
- The RFC 6455 catalogue is byte-identical across two runs.

## Domain relations

- Rfc -> Section, one-to-many, the RFC owns its sections: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Rfc`, relation `sections`.
- Section -> Section, zero or one parent: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Section`, relation `parent`. "A section includes its subsections" in the
  profile scope reads this relation.
- Section -> Requirement, one-to-many, the section owns its requirements: `ess/domains/catalogue.yaml`,
  entity `rfcspec.catalogue.Section`, relation `requirements` (via `section_id`); the ordinal rule
  is in the summary of type `rfcspec.catalogue.ItemId`.
- Profile -> Rfc, one-to-many, the profile references RFCs and owns none. Inferable (inferred from
  `ess/domains/profile.yaml`, type `rfcspec.profile.ProfileRfc`, a field typed
  `rfcspec.catalogue.RfcNumber` with no `relations:` entry, and `crates/rfcspec-model/src/lib.rs:45`).

## Scope

`crates/rfcspec-doc` (page-split paragraphs, sections, figures), `crates/rfcspec-extract`
(`lib.rs` boilerplate, ABNF comments, ids, level; `check.rs` furniture inside a span),
`crates/rfcspec-model` (section on `Block`, `source` on `Bcp14Requirement`), `crates/rfcspec-cli`
(tests), `profiles/websocket/profile.yaml`, `profiles/websocket/overlay.yaml`,
`profiles/websocket/rfcspec.lock`, `profiles/websocket/sources/rfc6455.txt`,
`website/docs/catalog/index.md` and `website/docs/catalog/websocket.md` (regenerated).

The RFC text names its authors and employers and uses example hosts, as the committed
`rfc868.txt` names ISI and SRI; it is committed verbatim. If the Gates scan refuses a literal in it,
stop and report rather than edit the source.

First of four stories; the others depend on it through `profiles/websocket/overlay.yaml`,
`crates/rfcspec-model`, `crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-cli` and the
generated catalogue pages.
