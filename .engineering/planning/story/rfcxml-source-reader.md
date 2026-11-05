---
format: aep.planning-md/3
id: story:rfcxml-source-reader
kind: story
status: implemented
title: Lock RFC 9220's RFCXML v3 source and read it into the normalized document, with section anchors and bcp14 marks
relations:
- decomposes: epic:rfcxml-reader
- depends_on: epic:ntp-profile
scope:
- confidence: inferred
  path: AGENTS.md
- confidence: inferred
  path: Cargo.lock
- confidence: inferred
  path: README.md
- confidence: inferred
  path: crates/rfcspec-cli/src/main.rs
- confidence: inferred
  path: crates/rfcspec-cli/tests/rfcxml.rs
- confidence: inferred
  path: crates/rfcspec-doc
- confidence: inferred
  path: crates/rfcspec-model/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-source/src/lib.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T15:46:56Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-05T15:46:57Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-05T16:17:34Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":4}}}
---
## Story

`rfcspec fetch --format xml` locks an RFC's RFCXML v3 source (design D7, section 3) the way it locks
plain text today, and `rfcspec read` turns a locked XML source into the same normalized document
(`rfcspec-model` `Document`: blocks of kind heading, paragraph and figure, each with its source-line
span and section number) that the text reader produces, plus what only the markup knows: the
section anchor of each block, the type of each figure and the position of every `<bcp14>` element.
Extraction is `story:rfcxml-bcp14-extract`; this story stops at S1.

RFC 9220 is the test case (design section 12.1: "RFC 9220 is the XML-path test case"). Its XML is
committed once, as the fixture `crates/rfcspec-doc/tests/fixtures/rfc9220.xml`; tests lock it into a
temporary profile through `rfcspec fetch` against a 127.0.0.1 server, as the existing fetch tests do.
No profile under `profiles/` gains RFC 9220 in this epic (see the epic decomposition notes in
`story:bulk-xml-regenerate`).

Decisions taken here (reversible):

- The format is chosen by the caller: `--format xml|text`, default `text`, so every existing
  command line and profile behaves as before. Choosing XML automatically when the index lists it
  (design section 3) needs the index at fetch time and is not part of this story.
- The lock format stays `rfcspec-lock/1`: the entry file `sources/rfc<N>.xml` names the format, and
  `read_locked` reports it from the extension, refusing any other extension.
- Text documents serialize byte for byte as before: the new `Document` and `Block` fields are
  omitted for text.

## Acceptance

`rfcspec read --profile <p> --rfc 9220`, over a profile in which `rfcspec fetch --rfc 9220 --format
xml` locked the fixture, exits 0 and prints a document marked `xml` whose headings and paragraphs of
sections 1 to 5 equal, whitespace-collapsed, the 14 blocks `rfcspec read` gives for those sections
of `rfc9220.txt`, whose section blocks carry the section `anchor` from the source (`websockets-upgrade-over-http3` for section 3; the text reader records none), whose blocks hold no text of the table of contents or of a cited document, whose
blocks record every `<bcp14>` mark of the source with its position (13 in RFC 9220; the text reader
records none), and whose figure blocks carry their `type` from the source.

Values below were read from `https://www.rfc-editor.org/rfc/rfc9220.xml` on 2026-11-05 (331 lines,
21906 bytes, SHA-256 `8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae`) and
`rfc9220.txt` (168 lines, SHA-256
`4bac5e1368db199468345dbd2d31ebb4abb7509732b1ae08108cc4860e8967db`, the digest in
`bulk/rfc-summary.jsonl`). If the fetched file has another digest, the fixture records what was
fetched and every value here is re-read before the story is accepted.

- Fetch. `rfcspec fetch --rfc 9220 --profile <p> --format xml --base-url <local>` writes
  `sources/rfc9220.xml` and the lock entry `{rfc: 9220, file: sources/rfc9220.xml, sha256: 8a60d0b5...}`;
  a second run changes no byte. A lock entry whose file ends neither in `.txt` nor in `.xml` makes
  `read` exit 1 naming the file. A changed byte in the locked XML makes `read` exit 1 naming both
  digests, as for text. The existing fetch tests pass unchanged.
- Document. The JSON has `"format": "xml"` and `furniture: []`; a text document has no `format` key
  (a test asserts `rfcspec read --profile profiles/websocket --rfc 6455` output is byte-identical to
  before). Spans are 1-based lines of the XML source: the paragraph `pn="section-3-3"` spans lines
  130 to 134.
- Sections. Every block of a `<section>` carries its number read from `pn` (`section-3` is `3`,
  `section-5-2...` lies in `5`) and its `anchor` (`websockets-upgrade-over-http3` for section 3).
  Numbered headings render as the text reader renders them, `3.  WebSockets Upgrade over HTTP/3`.
  Sections numbered `false` (the abstract, `status-of-memo`, `copyright`, `acknowledgments`,
  `authors-addresses`) carry an anchor and no number. The `<references pn="section-6">` heading is
  `6.  Normative References`.
- Text. Markup is removed. An `<xref>` renders as the plain text xml2rfc renders it: `[RFC8441]`
  (`format="default"`), `Section 15.6.2 of [HTTP]` and `Appendix A.3 of [HTTP/3]` (with
  `section`), the cited title for `format="title"`; an `<eref>` renders as its target. The
  `<dl>` of section 5 is one paragraph, one line per term and definition (`Value: 0x08`). The
  14 blocks of sections 1 to 5 equal those of the text document of the same RFC, kind, section and
  whitespace-collapsed text, in order (a test reads both).
- Excluded. The `<toc>` (lines 65 to 95) and every `<reference>` front matter, the cited documents' titles
  and abstracts (lines 244 to 246, 262 to 264, 279 to 281, 293 to 295, 308 to 310), give no block.
- BCP 14 marks. Each `<bcp14>` element is recorded on its block with its keyword and its position in
  the block text: the paragraph of section 2 holds 11 (`MUST` ... `OPTIONAL`), `section-3-3` holds
  `SHOULD` then `MAY`; RFC 9220 has 13 in all.
- Figures. `<sourcecode>` and `<artwork>` become figure blocks with their lines verbatim and their
  `type` (`abnf`, `ascii-art`, ...). RFC 9220 has none, so a unit test reads an inline document
  holding one `<sourcecode type="abnf">` and one `<artwork type="ascii-art">`.
- `cargo test -p rfcspec-doc -p rfcspec-source -p rfcspec-cli`, `cargo clippy` strict and
  `task check` pass; the RFC 868, 6455 and 1035 reader tests pass unchanged.

## Domain relations

- Rfc -> Section, one-to-many, the RFC owns its sections: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Rfc`, relation `sections` (`kind: owns`, `via: rfc`). The XML section tree is
  the source the `Section` summary names ("from the XML section tree").
- Section -> Section, parent, at most one: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Section`, relation `parent` (`kind: references`). Field `anchor` holds "The
  section anchor (XML) or number (text)".
- Rfc -> SourceFormat, one per read: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc` field
  `format_read`, enum `rfcspec.catalogue.SourceFormat` (`xml`, `text`).
- Profile lock -> source file, one file per RFC per profile: inferable (inferred from
  `crates/rfcspec-model/src/lib.rs:82-83`, `Lock::upsert` replaces the entry of the same RFC; no
  `ess/1` document declares the lock). So a profile locks the XML or the text of an RFC, not both.

## Scope

`crates/rfcspec-source/src/lib.rs` (format-aware `fetch`, `store`, `source_file`, `read_locked`),
`crates/rfcspec-model/src/lib.rs` (`Document` format, `Block` anchor, figure type and BCP 14 marks),
`crates/rfcspec-doc` (`Cargo.toml` gains `roxmltree`, a new XML reader module, tests),
`crates/rfcspec-doc/tests/fixtures/rfc9220.xml` (new), `crates/rfcspec-cli/src/main.rs` (`--format` on
`fetch`, `read_document` dispatching on the locked format), `crates/rfcspec-cli/tests/` (a new
`rfcxml.rs`), `Cargo.lock`, `AGENTS.md` and `README.md` (the reader is no longer text-only).

## Shared files and order

`crates/rfcspec-model/src/lib.rs` (`story:ntp-timer-model` rewrites `Arm` and adds `Machine.loss`),
`crates/rfcspec-doc/src/lib.rs` (`story:ntp-source-catalogue` adds reader tests) and `AGENTS.md`
(`story:ntp-timer-model`) are also changed by epic:ntp-profile.

The RFCXML waves start after the epic:ntp-profile pull request merges into `main` (decision taken by the coordinating session, 2026-11-05). `epic:ntp-profile` reaches this store with that merge; `depends_on: epic:ntp-profile` is then recorded on `story:rfcxml-source-reader`, which every later story in this epic depends on, before the first RFCXML wave.
