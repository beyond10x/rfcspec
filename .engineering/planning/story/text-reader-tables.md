---
format: aep.planning-md/3
id: story:text-reader-tables
kind: story
status: draft
title: The text reader reads table cells as paragraphs
scope:
- confidence: cited
  path: crates/rfcspec-doc/src/lib.rs
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
revision: 2
---
## Story

The plain-text reader reads table cells as paragraphs, so BCP 14 keywords in a table are requirements as they are on the RFCXML path. Today a text table is read as a figure, so its keywords are lost. This is the largest remaining cause of text/XML id differences after story:text-reader-structure: RFC 8881 (98 ids), 9140 (18) and 9399 (12) in `rfcspec bulk compare` on 2026-11-05.

## Acceptance

- `rfcspec bulk compare` over the 25 RFCs of story:text-reader-structure lists fewer than 7 RFCs, and 9140 is not listed.
- `locked_profiles` still passes: no locked profile id moves.

## Scope

- `crates/rfcspec-doc/src/lib.rs`
- `crates/rfcspec-extract/src/lib.rs` (`EXTRACTOR_VERSION`)
- the regenerated bulk files
