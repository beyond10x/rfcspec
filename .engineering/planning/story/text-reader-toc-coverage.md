---
format: aep.planning-md/3
id: story:text-reader-toc-coverage
kind: story
status: draft
title: The text reader recognises every dot-leader table of contents
scope:
- confidence: cited
  path: crates/rfcspec-doc/src/lib.rs
- confidence: cited
  path: crates/rfcspec-extract/tests/adversary_text_structure.rs
revision: 2
---
## Story

The plain-text reader recognises every dot-leader table of contents before the first section heading. Today the rule in `crates/rfcspec-doc/src/lib.rs` (around line 128) gives up on the whole table when one line ends in punctuation (RFC 2848 "I.N.") or when the numbers have no trailing dot ("1.1 Glossary", "1  Introduction"). Requirements read from table-of-contents lines therefore remain in RFC 1459, 2060, 2234, 2447, 2639, 2848, 3160, 3196 and 3501. Found by review-result:text-reader-adversary-1 (pre-existing: the base reader behaved the same), 2026-11-05.

## Acceptance

- `a_table_of_contents_with_a_title_ending_in_an_abbreviation_gives_no_requirement` in `crates/rfcspec-extract/tests/adversary_text_structure.rs` passes with its `#[ignore]` removed.
- None of the nine RFCs above has a requirement whose text ends in a dot leader and a page number.

## Scope

- `crates/rfcspec-doc/src/lib.rs`
- `crates/rfcspec-extract/tests/adversary_text_structure.rs`
- the regenerated bulk files
