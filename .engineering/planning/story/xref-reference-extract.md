---
format: aep.planning-md/3
id: story:xref-reference-extract
kind: story
status: draft
title: Record xref citations of other RFCs as catalogue References
relations:
- decomposes: epic:rfc-explorer
revision: 1
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
non-RFC reference is a `<reference>` with no RFC `seriesInfo`.

RFC 9220 is the test case, read from the fixture `crates/rfcspec-doc/tests/fixtures/rfc9220.xml`.

## Acceptance

Before this story, `rfcspec read` on the RFC 9220 XML lists no references. After it:

- `rfcspec read` prints a `references` list for the document, with one entry per `<xref>` to
  another RFC. Each entry holds the citing section, the target RFC, the target section if any, and
  the kind.
- The read summary counts the self-references (the table of contents and in-text section links)
  and the non-RFC references.
- RFC 9220's entries include target RFC 8441 (normative) and target RFC 9110 section `15.6.2`
  (normative). All values are re-read from the fixture before acceptance.
- A text document has no `references` key: its JSON is byte-identical to before.

## Scope

- `crates/rfcspec-model/src/lib.rs`: a `Reference` type and the `Document.references` field.
- `crates/rfcspec-doc/src/xml.rs`: resolve `<xref>` against `<reference>` and `<references>`.
- `crates/rfcspec-doc/tests/rfcxml.rs` and `crates/rfcspec-cli/tests/rfcxml.rs`.

Depends on the ESS declaration made with the decision (`ess/domains/catalogue.yaml`). The explorer
(`epic:rfc-explorer`) reads these references.
