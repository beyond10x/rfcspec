---
format: aep.planning-md/3
id: story:xml-appendix-numbering
kind: story
status: draft
title: The RFCXML reader numbers appendix subsections
scope:
- confidence: cited
  path: crates/rfcspec-doc/src/xml.rs
- confidence: cited
  path: crates/rfcspec-doc/tests/rfcxml.rs
revision: 2
---
## Story

The RFCXML reader numbers appendix subsections. `number_from_pn` in `crates/rfcspec-doc/src/xml.rs` reads no number from `pn="section-b.2.1"` (only `section-appendix.x` gets one), so requirements in appendix subsections are sectionless (`R-…`) on the XML path. The text path gives them `RB.2.1-…`. Found by story:text-reader-structure on RFC 8881 Appendix B.2.1 (a MUST NOT), 2026-11-05.

## Acceptance

- RFC 8881 read from RFCXML gives the Appendix B.2.1 MUST NOT the id `RB.2.1-1`, equal to the text path.
- `rfcspec bulk compare` over the same 25 RFCs lists 8881 with fewer differing ids than 98.

## Scope

- `crates/rfcspec-doc/src/xml.rs`
- `crates/rfcspec-doc/tests/rfcxml.rs`
- the regenerated bulk files, which need new gates exception digests
