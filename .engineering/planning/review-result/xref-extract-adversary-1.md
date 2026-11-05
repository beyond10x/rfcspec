---
format: aep.planning-md/3
id: review-result:xref-extract-adversary-1
kind: review-result
status: active
title: Adversary on story:xref-reference-extract, pass 1
relations:
- reviews: story:xref-reference-extract
revision: 1
---
I added 5 cases and 3 of them fail. All three come from markup that real RFCs contain, except the third, which nothing in the corpus reaches.

```
unit: story:xref-reference-extract, uncommitted working tree on base eac203c in the xref-reference-extract worktree
verdict: NEEDS-CHANGE
cases: executed 35→40, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: decide whether an xref to a <referencegroup> (STD 94, BCP 26, BCP 195) cites its member RFCs
```

**1. `git --no-pager diff --stat`**

The stat lists only the implementor's 7 files. My one file is untracked (`?? crates/rfcspec-doc/tests/xref_adversary.rs`), so the stat cannot show it. I changed no file outside tests.

**2. Cases added**, all in `crates/rfcspec-doc/tests/xref_adversary.rs`. I ran this file on its own before running anything else:
- `an_rfc_series_info_written_with_its_prefix_still_names_the_rfc`: **red.** Input is `<seriesInfo name="RFC" value="RFC9785"/>`, the way RFC 9786 writes it. It should give a reference to 9785.
  ```
  left: []
  right: [Reference { section: Some("s"), target_rfc: 9785, target_section: None, kind: Normative }]
  ```
- `an_xref_to_a_reference_group_of_one_rfc_cites_that_rfc`: **red.** An xref to `STD94`, whose only member is RFC 8949, should cite 8949.
  ```
  left: []
  right: [Reference { section: Some("s"), target_rfc: 8949, target_section: None, kind: Normative }]
  ```
- `an_xref_to_an_anchor_that_does_not_exist_is_not_a_self_reference`: **red.**
  ```
  left: Some(ReferenceCounts { self_references: 1, non_rfc: 0 })
  right: Some(ReferenceCounts { self_references: 0, non_rfc: 0 })
  ```
- `the_kind_defaults_to_informative_and_non_normative_is_informative`: green. No existing test reaches the informative default or the `non-normative` branch, because every group in the suite is named Normative or Informative. This case covers both. I did not run a mutation against it.
- `xrefs_in_a_figure_a_table_and_the_abstract_take_their_section`: green.
- `corpus_probe`: ignored by default. It runs when `RFCSPEC_XML_CORPUS` is set and reads the corpus without writing anything.

**3. Suite run**

Command: `cargo test -p rfcspec-doc -p rfcspec-model` with the required env.
```
tests/rfcxml.rs            test result: ok. 11 passed
tests/rfcxml_adversary.rs  test result: ok. 8 passed
src/lib.rs                 test result: ok. 16 passed
tests/xref_adversary.rs    test result: FAILED. 2 passed; 3 failed; 1 ignored
EXIT=101
```
Cargo stopped at the first red binary, so the rfcspec-model and doc tests did not run. The "before" count of 35 is the same run with my file left out.

**Corpus probe** over all 1,390 files, every one read:

| Count | Value |
|---|---|
| references stored | 59,454 |
| self_references | 132,526 |
| non_rfc | 13,666 |
| xrefs to a target that exists nowhere | **0** |
| files with zero references but xrefs to RFC references | **0** (10 files have zero references and none of them has an RFC xref) |
| files where the informative default fires | 1 (rfc8967, 7 RFC `<reference>`s) |
| xrefs to a `<referencegroup>`, all counted as non_rfc | **320, in 83 files** |
| RFC xrefs lost to a non-numeric `seriesInfo` | 2 (rfc9786 → 9785) |

Group names seen: "Normative References" 1,309, "Informative References" 1,326, "References" 1,234. The odd ones are "Informational References", "Informative references", names with stray spaces, and "Related Protocols". The implementation reads all of them correctly or falls back to the default.

**4. Findings**

| file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|
| `crates/rfcspec-doc/src/xml.rs:183` | A `seriesInfo` RFC value with an `RFC` prefix fails to parse, so the xref is counted as non_rfc. Fix: strip the `RFC` prefix, or fall back to the DOI `10.17487/RFCnnnn`. | NEEDS-CHANGE | introduced | RFC 9786, 2 xrefs (corpus) |
| `crates/rfcspec-doc/src/xml.rs:148` | Every `<referencegroup>` counts as non_rfc, so citations of STD 94/90/96, BCP 26, BCP 195 and others are dropped from the reference graph. The story's own definition of non-RFC is "a `<reference>` with no RFC seriesInfo", and a group of RFCs is not that. One-member groups resolve unambiguously; multi-member groups need a decision. | NEEDS-CHANGE | introduced | 320 xrefs in 83 corpus RFCs |
| `crates/rfcspec-doc/src/xml.rs:157` | An xref whose target matches no anchor or `pn` anywhere is counted as a self-reference, which hides a broken source. | INFEASIBLE | introduced | nothing found: 0 such xrefs in 1,390 RFCs, so it only matters for a malformed source |

**5. Attacked and could not break**
- Nested `<references>` groups and the names they use: the innermost named group wins.
- Self-reference through a `<reference>` carrying the citing RFC's number.
- A `seriesInfo` inside the `<reference>`'s `<front>` (RFC 8660 and 8661): resolved correctly.
- A reference that has both a DOI and an RFC `seriesInfo`.
- An xref in a figure name, a table cell, the abstract or a nested section: each gets the right citing section.
- `<relref>`: none in the corpus.
- An xref to `BCP14` as a group: 1 file.
- Text-document byte identity: both fields are skipped when empty or `None`.

**6. Paths written outside the worktree:** none.

```findings
- file: crates/rfcspec-doc/src/xml.rs
  line: 183
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: seriesInfo name="RFC" value="RFC9785" (RFC 9786) fails to parse as a number, so its 2 xrefs to RFC 9785 count as non_rfc instead of references
- file: crates/rfcspec-doc/src/xml.rs
  line: 148
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: every xref to a referencegroup is counted non_rfc, dropping 320 RFC citations in 83 corpus RFCs, including one-member groups such as STD94 = RFC 8949
- file: crates/rfcspec-doc/src/xml.rs
  line: 157
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: an xref whose target names no anchor is counted as a self-reference, hiding a broken source; no corpus RFC has one
```
