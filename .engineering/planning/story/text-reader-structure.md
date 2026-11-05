---
format: aep.planning-md/3
id: story:text-reader-structure
kind: story
status: draft
title: The text reader recognises appendix headings, lists and asides
relations:
- informed_by: epic:rfcxml-reader
revision: 1
---
## Story

The plain-text reader recognises appendix headings and keeps list items and asides as their own
paragraphs, so its requirement ids match those the RFCXML reader gives for the same RFC.

Found on 2026-11-05 while comparing both paths over 158 cached RFCs for `story:rfcxml-bcp14-extract`:
25 give different (id, level) lists. Causes, classified from the rows (only the first was checked in
the XML source):

- An `Appendix A.` heading is not recognised, so the text reader keeps the last numbered section
  (`R13.2-1` where XML gives `RA-1`): RFC 9588, 9609 and 9644 (checked), 8944 and 9231 (from ids).
- `*` bullet lists and `|` asides are joined into one paragraph: 8860, 8951, 9077, 9399, 9441, 9504,
  9651, 9700.
- Headings such as "REQUIRED Attributes 5.2." are read as paragraphs: 8881.
- Not classified: 8839, 9028, 9140, 9175, 9210, 9329, 9350, 9483, 9560, 9595, 9665.

This changes text-path catalogues, so `EXTRACTOR_VERSION` moves and the committed bulk summary is
regenerated.

## Acceptance

Before: RFC 9644 read from text gives ids under `R13.2` for its appendix requirements. After: the
text reader gives `RA-…` ids for them, equal to the XML path, and the comparison over the same 158
RFCs reports fewer differing RFCs, each remaining one classified.

## Scope

`crates/rfcspec-doc/src/lib.rs` (text reader), `crates/rfcspec-extract/src/lib.rs` (`EXTRACTOR_VERSION`),
`bulk/rfc-summary.jsonl` (regenerated).
