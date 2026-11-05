---
format: aep.planning-md/3
id: story:xref-reference-extract
kind: story
status: active
title: Record xref citations of other RFCs as catalogue References
relations:
- decomposes: epic:rfc-explorer
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T17:48:52Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-05T17:48:52Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
---
## Story

Reading an RFC from its RFCXML source records each `<xref>` to another RFC as a
`rfcspec.catalogue.Reference` (`ess/domains/catalogue.yaml`). The decision is recorded in
`decision-blocker:rfcxml-xref-reference`, option A. Each Reference holds:

- the citing section, where the xref lies inside one;
- the cited RFC's number, resolved through the `<reference>` whose anchor the xref targets and its
  `seriesInfo name="RFC"`;
- the cited section from the xref's `section` attribute;
- `normative` or `informative`, from the `<references>` group that holds the cited `<reference>`.

Self-references and references to non-RFC documents are counted, not stored. A self-reference is
an xref to an anchor inside the same RFC, or a `<reference>` whose RFC number is the citing RFC. A
non-RFC reference is a `<reference>` with no RFC `seriesInfo`. The counts go into the document JSON
as `reference_counts: {self_references: N, non_rfc: M}`, omitted for text documents, because
`rfcspec read` prints only the document JSON and has no separate summary
(`crates/rfcspec-cli/src/main.rs`, `Command::Read`).

RFC 9220 is the test case, read from the fixture `crates/rfcspec-doc/tests/fixtures/rfc9220.xml`.
Counted from the fixture on 2026-11-05: 35 `<xref>` elements. 16 target a local anchor
(`section-*` or `name-*`), all inside `<toc>`: self-references 16. 19 target one of the 8
`<reference>` elements, all in the one `<references>` group "Normative References"
(`pn="section-6"`), and every `<reference>` has a `seriesInfo name="RFC"` (9110, 9113, 9114, 2119,
6455, 7807, 8174, 8441), none of them 9220: non-RFC 0. So 19 references, all normative.

`bulk summarise` reads every XML source through the same reader (`crates/rfcspec-cli/src/bulk.rs`,
`counts`, which calls `rfcspec_doc::read_format`), so its `Document` carries `references` once this
story lands. This story changes no bulk output: the sidecar `bulk/rfc-sections.jsonl` that writes
the References for each RFCXML RFC belongs to `story:explorer-rfc-view`, which comes after this one.

## Acceptance

Before this story, `rfcspec read` on the RFC 9220 XML lists no references; after it, the document
JSON carries a `references` list of 19 entries, one per `<xref>` to another RFC.
The values are re-read from the fixture
before acceptance.

Checked by:

- The document JSON carries `reference_counts: {self_references: 16, non_rfc: 0}` for RFC 9220.
- Each entry holds the citing section, the target RFC, the target section if any, and the kind.
- RFC 9220's entries include target RFC 8441 (normative) and target RFC 9110 section `15.6.2`
  (normative).
- A text document has no `references` and no `reference_counts` key: its JSON is byte-identical to
  before (`WEBSOCKET_READ_SHA256` in `crates/rfcspec-cli/tests/rfcxml.rs`).
- `rfcspec bulk summarise` output is unchanged by this story.

## Scope

- `crates/rfcspec-model/src/lib.rs`: a `Reference` type, the `Document.references` field and the
  `Document.reference_counts` field.
- `crates/rfcspec-doc/src/xml.rs`: resolve `<xref>` against `<reference>` and `<references>`.
- `crates/rfcspec-doc/tests/rfcxml.rs` and `crates/rfcspec-cli/tests/rfcxml.rs`.
- `crates/rfcspec-cli/src/bulk.rs`: read only; `bulk summarise` gets `Document.references` through
  `rfcspec_doc::read_format` without a change here.

Depends on the ESS declaration made with the decision (`ess/domains/catalogue.yaml`). The explorer
(`epic:rfc-explorer`) reads these references.
