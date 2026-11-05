---
format: aep.planning-md/3
id: story:rfcxml-bcp14-extract
kind: story
status: implemented
title: 'Extract requirements from bcp14 markup: RFC 9220 gives R3-1 SHOULD and R3-2 MAY as bcp14-xml'
relations:
- decomposes: epic:rfcxml-reader
- depends_on: story:rfcxml-source-reader
scope:
- confidence: inferred
  path: crates/rfcspec-cli/tests/rfcxml.rs
- confidence: inferred
  path: crates/rfcspec-extract/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-model/src/lib.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T15:46:56Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T16:17:34Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T16:51:54Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

`rfcspec extract` over a document read from RFCXML takes its requirements from the markup: one
requirement per sentence containing a `<bcp14>` element (design section 5, "XML: one requirement per
sentence containing a `<bcp14>` element"), `source: bcp14-xml`, the level read from the element, with
no capitals heuristic. Everything else S2 does (prose candidates, dialogue steps, examples, ABNF
comments) runs over the XML document as over a text one. RFC 9220 is the test case, read from the
fixture `story:rfcxml-source-reader` commits.

Decisions taken here (reversible):

- The level is the whitespace-collapsed content of the first `<bcp14>` element in the sentence. An
  element whose content is not one of the eleven levels (`rfcspec_extract::LEVELS`) gives no
  requirement; `rfcspec extract` names its source line on standard error and still exits 0.
- A capitals keyword outside `<bcp14>` is not a requirement on the XML path: the markup is the
  authority. Inside a `<sourcecode type="abnf">` figure there is no `<bcp14>` markup, so ABNF
  comments are scanned for capitals as on the text path, `source: abnf-comment`.
- `EXTRACTOR_VERSION` stays 1: no text catalogue changes, and the bulk summary tells an XML row apart
  by its `format` (`story:bulk-xml-summary`), not by a new extractor version.

## Acceptance

`rfcspec extract --profile <p> --rfc 9220`, over a profile locking the RFC 9220 XML, exits 0 and
prints a catalogue with exactly two requirements, `R3-1` (`SHOULD`) and `R3-2` (`MAY`), both
`source: bcp14-xml`, whose ids, levels, sections and whitespace-collapsed texts equal those
`rfcspec extract` gives for `rfc9220.txt` (where they are `capitals-text`).

Values below were read on 2026-11-05 from the files and digests `story:rfcxml-source-reader` names,
and from the text path run by the `rfcspec` binary of `main` on `rfc9220.txt`.

- `R3-1` is "If a server advertises support for Extended CONNECT but receives an Extended CONNECT
  request with a ":protocol" value that is unknown or is not supported, the server SHOULD respond
  to the request with a 501 (Not Implemented) status code (Section 15.6.2 of [HTTP]).", XML lines 130
  to 133; `R3-2` is "A server MAY provide more information via a "problem details" response
  [RFC7807].", XML lines 133 to 134. (Text path: lines 91 to 94 and 94 to 96.)
- The RFC 2119 boilerplate paragraph of section 2 holds 11 `<bcp14>` elements and gives no
  requirement, by the rule that excludes it on the text path.
- Prose candidates: the same two sentences, whitespace-collapsed, as on the text path (the
  `may be obtained at` sentence of `status-of-memo` and the `must include` sentence of `copyright`).
  None comes from a cited document's abstract.
- Unit tests on inline XML documents: a sentence with `<bcp14>MUST NOT</bcp14>` gives level
  `MUST NOT`; a sentence with two elements takes the first as its level and is one requirement; a
  sentence with a capitals `MUST` and no element gives no requirement; `<bcp14>must</bcp14>` gives no
  requirement and one standard-error line naming its line; a `;` comment with `MUST` in a
  `<sourcecode type="abnf">` gives one `abnf-comment` requirement.
- `rfcspec bulk check --summary bulk/rfc-summary.jsonl --profiles profiles` passes unchanged, and the
  catalogues of RFC 868, 6455 and 1035 are byte-identical to before (the existing tests).
- `cargo test -p rfcspec-extract -p rfcspec-cli`, strict Clippy and `task check` pass.

## Domain relations

- Section -> Requirement, one-to-many, the section owns its requirements: `ess/domains/catalogue.yaml`,
  entity `rfcspec.catalogue.Section`, relation `requirements` (`kind: owns`, `via: section_id`).
- Requirement -> Rfc, exactly one: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Requirement`, relation `rfc` (`kind: references`). Its `source` is
  `rfcspec.catalogue.RequirementSource` variant `Bcp14Xml` (wire `bcp14-xml`, "A sentence containing a
  bcp14 element in RFCXML"), and the invariant `any: [source == Prose, defined(level)]` requires the
  level this story reads from the element.

## Scope

`crates/rfcspec-extract/src/lib.rs` (sentence requirements from BCP 14 marks for XML documents),
`crates/rfcspec-model/src/lib.rs` (a `BCP14_XML` source constant beside `CAPITALS_TEXT`),
`crates/rfcspec-cli/tests/rfcxml.rs` (extended).

Depends on `story:rfcxml-source-reader`: it reads the BCP 14 marks and the fixture that story adds,
and extends its `tests/rfcxml.rs`.

## Shared files and order

`crates/rfcspec-extract/src/lib.rs` (`story:ntp-source-catalogue` adds extraction tests and fixes)
and `crates/rfcspec-model/src/lib.rs` (`story:ntp-timer-model`) are also changed by epic:ntp-profile.

The RFCXML waves start after the epic:ntp-profile pull request merges into `main` (decision taken by the coordinating session, 2026-11-05). `epic:ntp-profile` reaches this store with that merge; `depends_on: epic:ntp-profile` is then recorded on `story:rfcxml-source-reader`, which every later story in this epic depends on, before the first RFCXML wave.
