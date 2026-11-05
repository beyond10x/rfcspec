---
format: aep.planning-md/3
id: story:xml-overlay-check
kind: story
status: draft
title: rfcspec check covers profiles locked as RFCXML
relations:
- informed_by: epic:rfcxml-reader
- depends_on: story:fixed-arm-bounds-check
scope:
- confidence: cited
  path: crates/rfcspec-cli/tests/rfcxml.rs
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-extract/tests/adversary_bcp14_xml.rs
revision: 6
---
## Story

`rfcspec check` checks a profile whose RFC is locked as RFCXML the way it checks a text-locked one.
Today it computes page furniture over the raw source as if it were text
(`crates/rfcspec-extract/src/check.rs:24`, `furniture_lines(source)`), and the check that a
mechanically read requirement keeps its catalogue id covers only `capitals-text` and `abnf-comment`
(`check.rs:358`), so an overlay entry with `source: bcp14-xml` is not compared with the catalogue.

Found while implementing `story:rfcxml-bcp14-extract` (2026-11-05). No committed profile locks XML,
so nothing reaches it today; the epic excludes XML-backed profile overlays.

## Acceptance

Also: the quote check reads raw source lines (`check.rs:655`), so the exact catalogue text of a sentence holding `<bcp14>` or `<xref>` is refused at its own span; the adversary on `story:rfcxml-bcp14-extract` left three ignored tests for this story in `crates/rfcspec-extract/tests/adversary_bcp14_xml.rs`, which it un-ignores.

Before: an overlay entry `source: bcp14-xml` whose id, level or quote disagrees with the XML
catalogue passes `rfcspec check`. After: over a temporary profile locking the RFC 9220 XML fixture,
`rfcspec check` exits 1 naming the entry when its id, level or quote differs from what
`rfcspec extract` gives, and exits 0 on a matching overlay; furniture lines are taken from the
document, empty for XML.

## Scope

`crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-cli/tests/rfcxml.rs`, `crates/rfcspec-extract/tests/adversary_bcp14_xml.rs` (the three ignored tests it un-ignores).

## Order

Four stories edit `crates/rfcspec-extract/src/check.rs` and run in this order: story:send-payload-type-check → story:fixed-arm-bounds-check → story:xml-overlay-check → story:websocket-grammar-review; story:status-evidence follows the last. This story depends on story:fixed-arm-bounds-check because both edit `check.rs`, and story:websocket-grammar-review depends on this one for the same reason.
