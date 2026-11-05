---
format: aep.planning-md/3
id: review-result:text-reader-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:text-reader-structure
relations:
- reviews: story:text-reader-structure
revision: 1
---
unit: story:text-reader-structure (rfcspec wave 5); the findings cover the uncommitted working tree of a local path on base 60e0a56
verdict: NEEDS-CHANGE
cases: executed 319→322, red 2
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: 1 (scratch directory, listed in part 6)
needs-coordinator: none

**1. Diff stat.** My only change to the tree is one new untracked test file, `crates/rfcspec-extract/tests/adversary_text_structure.rs`. `git --no-pager diff --stat` still shows only the implementor's tracked changes (`90 files changed, 1924 insertions(+), 1583 deletions(-)`). I changed no implementation file.

**2. Cases added.** All three are in `a local path`. Each was run alone before the suite ran.

- `a_list_item_wrapped_at_its_marks_column_keeps_its_sentence_whole` is **red**. It uses RFC 3995 lines 2537-2539, a `*` item whose wrapped lines sit at the mark's column:
  ```
  left: [("R1-1", "MUST", "*  The Printer MUST deliver the \"job-impressions-completed\" attribute")]
  right: [("R1-1", "MUST", "*  The Printer MUST deliver the \"job-impressions-completed\" attribute in an Event Notification only for the combinations of Events and Subscribed Events shown in Table 7.")]
  ```
  The base reader gave the whole sentence (I ran a copy of the base in scratch). The new item rule at `crates/rfcspec-doc/src/lib.rs:362` cuts the sentence and drops its condition.
- `a_table_of_contents_with_a_title_ending_in_an_abbreviation_gives_no_requirement` is **red**. It uses the RFC 2848 table of contents, where one wrapped title ends in "I.N.":
  ```
  left: [("R-1", "REQUIRED"), ("R1-1", "MUST")]
  right: [("R1-1", "MUST")]
  R-1 text: "REQUIRED and OPTIONAL elements for PINT compliance ......... 11 6."
  ```
  The rule at `lib.rs:128` rejects the whole table when any one line ends in punctuation.
- `a_numbered_list_without_closing_punctuation_inside_a_section_is_no_table_of_contents` is **green now and red on a mutant**. With `&& sections[start].is_none()` (`lib.rs:119`) replaced by `&& true` in a scratch copy, every `rfcspec-doc` and `rfcspec-extract` test stayed green. This case then failed with `left: []` against `right: [("R1-1","MUST"),("R1-2","MUST")]`.

**3. Suite.** I ran it after the cases existed:
`CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 RUST_TEST_THREADS=4 nice -n 10 ionice -c3 cargo test --no-fail-fast -p rfcspec-doc -p rfcspec-extract -p rfcspec-cli`
- Result: 320 passed, 2 failed, ending in `error: 1 target failed: -p rfcspec-extract --test adversary_text_structure`, `EXIT=101`.
- The only failures are the two red cases above.
- 319 is the count with my 3 cases taken out of the 322.

**4. Findings that need a change or a note.**
- **`crates/rfcspec-doc/src/lib.rs:362`** (NEEDS-CHANGE, introduced). A line at or left of a list mark always ends the item, even in the middle of a sentence.
  - What reaches it: `rfcspec bulk summarise` over text-only RFCs. In the corpus, 241 item continuations sit at the mark's column; 5 of them carry a BCP 14 keyword (RFC 2639:497, 3851:708, 3995:2537, 3996:904, 7978:1047).
  - For RFC 3995 the committed catalogue now holds a cut requirement text; for RFC 2639 one sentence became two REQUIRED requirements.
  - Suggested fix: end the item only when its previous line ends in closing punctuation, or when the next line does not start lower-case.
- **`lib.rs:128`** (CONFIRMED, pre-existing). A table of contents is missed when any one line ends in punctuation, or when its numbers have no trailing dot ("1.1 Glossary", "1  Introduction").
  - Requirements taken from dot-leader table-of-contents lines remain in RFC 1459, 2060, 2234, 2447, 2639, 2848, 3160, 3196 and 3501.
  - The base gave the same results, so this is incomplete coverage of the story's table-of-contents cause, not a regression.
- **`lib.rs:119`** (CONFIRMED, introduced, note). The suite has no case for the "only before the first section heading" guard. The green case above now covers it.

**5. Attacked and not broken:**
- **Count decreases:** all are table-of-contents lines. I diffed every changed RFC between the base and new reader. Every removed requirement that was not a table-of-contents line came back split, and no RFC lost body keywords. RFC 3726 (−30), 6471, 2567, 7530 and 6117 are all dot-leader lines.
- **Dashes and operators:** a dash or `+` in running text or in formulas causes no false split. The 7 formula-continuation hits in the corpus carry no keyword.
- **`o` and `+` in ASCII art:** these are split only in RFCs 578, 5673 and 5972, none with a keyword.
- **Appendix headings:** every unindented `Appendix X.` / `X.n.` line in the corpus is a real heading. Lines with no `Appendix` prefix and no numeric part are rejected.
- **The three changed test pins:**
  - The RFC 6455 read SHA changed only through the table of contents becoming a figure and one-line items moving from heading to paragraph (51 block lines).
  - RFC 5905 changed only in its table of contents and the new appendix sections.
  - Extractor version 2 is intended.
- **Locked-profile pins:** they pass when run against the base reader, and they pin the full catalogue JSON.
- **Regeneration:** `bulk summarise` from the cache, written into scratch, is byte-identical to the committed `bulk/rfc-summary.jsonl` and `bulk/rfc-sections.jsonl`. `bulk check` holds.
- **`bulk compare`:** its output format and exit codes match the story.
- **Content rules:** no `/home/<name>/` paths, no Python and no config-file comments in the diff.
- **Asides:** no text lost; a group that is not entirely aside lines keeps its bars, as it did before.

**6. Written outside the worktree:** one path, `a local path` (543M). It holds base and new source copies, each with its own `target/`, a probe `dump.rs`, a regenerated summary and logs. I did not delete it, because the rules forbid deleting a `target/` by hand. I also took no worktree session lease explicitly.

**7. Findings block.**
```findings
- file: crates/rfcspec-doc/src/lib.rs
  line: 362
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a list item's continuation line at its mark's column ends the item mid-sentence, truncating requirement texts such as RFC 3995 line 2537 and splitting one RFC 2639 sentence into two requirements
- file: crates/rfcspec-doc/src/lib.rs
  line: 128
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: one table-of-contents line ending in punctuation (RFC 2848 "I.N.") or numbers without a trailing dot defeat the table-of-contents rule, so dot-leader requirements remain in RFC 1459, 2060, 2234, 2447, 2639, 2848, 3160, 3196 and 3501
- file: crates/rfcspec-doc/src/lib.rs
  line: 119
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: removing the before-the-first-section guard of the table-of-contents rule left every rfcspec-doc and rfcspec-extract test green until the added numbered-list case
```
