---
format: aep.planning-md/3
id: review-result:rfcxml-extract-adversary-1
kind: review-result
status: active
title: Adversary on story:rfcxml-bcp14-extract, pass 1
relations:
- reviews: story:rfcxml-bcp14-extract
revision: 1
---
```
unit: story:rfcxml-bcp14-extract, uncommitted working tree on base e706076 in the rfcxml-bcp14-extract worktree
verdict: NEEDS-CHANGE
cases: executed 220→227, red 5
origin: introduced 3 / pre-existing 0 / undecided 3
wrote-outside-worktree: 1 (scratch directory, listed in part 6)
needs-coordinator: none
```

**1. `git --no-pager diff --stat`**

```
 AGENTS.md                          |   2 +-
 crates/rfcspec-cli/src/main.rs     |   6 +-
 crates/rfcspec-cli/tests/rfcxml.rs | 155 ++++++++++++++++++++++++
 crates/rfcspec-extract/src/lib.rs  | 239 ++++++++++++++++++++++++++++++++++---
 crates/rfcspec-model/src/lib.rs    |  11 +-
 5 files changed, 392 insertions(+), 21 deletions(-)
?? crates/rfcspec-extract/tests/        (mine: adversary_bcp14_xml.rs)
```

All five tracked paths are the implementor's changes. The only file I added is the untracked test file `the rfcxml-bcp14-extract worktree/crates/rfcspec-extract/tests/adversary_bcp14_xml.rs`. I changed no implementation file.

**2. Cases added** (first run, that file alone: `cargo test -p rfcspec-extract --test adversary_bcp14_xml`, 2 passed, 5 failed)

- **`two_definitions_of_a_dl_each_with_bcp14_are_two_requirements`: RED.** This is the shape of RFC 9742 section 8. Two `<dt>/<dd>` pairs each hold their own `<bcp14>`, and the second term starts in lower case.
  ```
  assertion `left == right` failed: ["facility-filter/pattern-match: When writing this node, implementations MUST ensure the pattern is safe. remote/destination/signing/cert-signer: When writing this subtree, implementations MUST NOT reuse a key."]
    left: [("R1-1", "MUST", "bcp14-xml")]
   right: [("R1-1", "MUST", "bcp14-xml"), ("R1-2", "MUST NOT", "bcp14-xml")]
  ```
- **`check_refuses_a_bcp14_xml_overlay_requirement_whose_id_is_no_catalogue_requirement`: RED.**
  ```
  an overlay requirement R3-9 (bcp14-xml) passes the check against a catalogue without R3-9
  ```
  The same entry with `capitals-text` is refused.
- **`check_refuses_a_bcp14_xml_overlay_requirement_outside_its_catalogue_span`: RED.**
  ```
  R3-2 (bcp14-xml) quoting line 128 passes, outside the catalogue span 133 to 134
  ```
  The same entry with `capitals-text` is refused.
- **`check_accepts_the_catalogue_text_of_r3_2_quoted_at_its_catalogue_span`: RED.**
  ```
  left: ["requirement R3-2: quote \"A server MAY provide more information via a \\\"problem details\\\" response [RFC7807].\" does not occur at lines 133-134"]
  right: []
  ```
- **`a_non_level_second_element_does_not_report_that_its_sentence_gives_no_requirement`: RED.**
  ```
  the problem says the sentence gives no requirement, but the catalogue holds [("R1-1", "MUST", "bcp14-xml")]: line 5: <bcp14>must</bcp14> holds no BCP 14 level; its sentence gives no requirement
  ```
- **`a_type_abnf_figure_without_a_rule_start_line_gives_its_comment_as_abnf_comment`: green.** It catches a mutant the unit's own tests miss. I made a mutated copy in scratch that replaces the `figure_type == Some("abnf")` clause at lib.rs:148 with `false`. On that copy the unit's lib tests ran 67 passed, 0 failed (`an_xml_abnf_comment_with_capitals_must_is_an_abnf_comment_requirement ... ok`), and this case FAILED.
- **`the_units_own_abnf_test_line_is_a_rule_line_so_it_cannot_see_the_type_clause`: green.** It records why the unit's ABNF test misses that mutant: `abnf_rule("frame = %x00-FF ; MUST be zero")` is true, so the old line heuristic already takes the branch.

**3. Suite run** (after the cases existed)

Command: `cargo test -p rfcspec-extract -p rfcspec-cli`. EXIT=101.
- rfcspec-cli: 153 passed across 16 targets.
- rfcspec-extract unit tests: 67 passed.
- `adversary_bcp14_xml`: 2 passed, 5 failed, the same 5 as above.
- Doc-tests did not run because cargo stops at the first failing target.
- Before/after: 227 cases ran. The before figure of 220 is the same run with my 7 cases subtracted, not a separate run.
- Also run: `rfcspec bulk check --summary bulk/rfc-summary.jsonl --profiles profiles` printed "holds", exit 0.

**4. Findings**

- **lib.rs:175 / xml.rs:351. NEEDS-CHANGE, undecided. A `<dl>` loses requirements.**
  - Measured: the reader turns a whole `<dl>` into one paragraph. A definition followed by a term that starts in lower case gets no sentence break at lib.rs:383, so both definitions become one sentence, and only its first `<bcp14>` counts.
  - What reaches it: plain `rfcspec extract` on published RFCs. In RFC 9742 the `MUST NOT` at XML line 1564 disappears into R8-1, where the text path gives two requirements. My sample flags merged sentences in 8748, 8846, 8881 (2), 8895, 9147, 9483, 9553 (2) and 9742.
  - This breaks the acceptance rule "one requirement per sentence containing a `<bcp14>`".
  - Possible fix (not applied): treat each dl entry line as a sentence boundary, or have the reader emit one block per entry.
  - Origin is undecided because I did not run it against the base. The base's capitals-only extraction would probably merge the same sentence.
- **check.rs:358. NEEDS-CHANGE, introduced. The check never ties a `bcp14-xml` overlay entry to the catalogue.**
  - Measured: the id-and-span check filters to `capitals-text` and `abnf-comment` only, so a `bcp14-xml` entry with a made-up id, or with a span outside its catalogue span, passes.
  - What reaches it: the XML catalogue now says `bcp14-xml`. `propose` copies the catalogue source into its entry (rfcspec-propose/src/lib.rs:339), so review-accept writes `bcp14-xml` entries. At the base the same XML profile produced `capitals-text` entries, and those were checked.
  - So the implementor's "left alone" item is exactly how a wrong overlay passes today.
- **check.rs:655 (`quote_problem` over raw source lines, check.rs:23). CONFIRMED, undecided. A correct quote fails on XML.** The quote check reads the raw XML lines, so the exact catalogue text of R3-2 at its own span is refused. Every faithful quote of a sentence that holds `<bcp14>` or `<xref>` fails the same way. No story in epic:rfcxml-reader covers check or propose over XML.
- **lib.rs:148. CONFIRMED, introduced. Untested clause.** The `type="abnf"` clause is not pinned by any unit test; the mutant evidence is in part 2.
- **lib.rs:167. INFEASIBLE, introduced. Wrong problem message.** The problem text says "its sentence gives no requirement" even when an earlier element in the same sentence gave one. In 552 published RFCXML files sampled (8650 to 9750, every other number) no `<bcp14>` held anything but a level, so nobody is shown to reach this.
- **lib.rs:146-161. CONFIRMED, introduced. Judgement: ids are not stable between the text and XML paths.**
  - Measured: of 157 RFCs I fetched in both formats, 38 give different (id, level) lists.
  - Main cause: requirements inside figures, mostly `description` strings in YANG modules and `OPTIONAL` in ASN.1 (9644, 8776, 9399, 9196). The text path counts them as `capitals-text`; the XML path drops them because figures carry no marks. Every id after them in the section shifts.
  - This follows the story's "markup is the authority" decision, so it is a design consequence, not a bug in the code. It will move bulk counts once `story:bulk-xml-summary` reads XML, and any overlay carried from a text lock to an XML lock will point at the wrong sentences.

**5. Attacked, could not break**
- Sentence splitting at `<bcp14>` boundaries, including a `<bcp14>` start on a following line and trimmed leading whitespace: offsets line up.
- `<bcp14>` inside nested markup (`<em>`), in a list item, in a table cell, in the abstract: one paragraph each, marks found.
- `MUST NOT` split across lines inside `<bcp14>` (RFC 8860, 9012 have this): collapsed correctly.
- `NOT RECOMMENDED`, `REQUIRED`, `OPTIONAL`: in LEVELS.
- `<bcp14>MUST</bcp14> <bcp14>NOT</bcp14>` and lower-case `<bcp14>`: zero cases in the 552-RFC sample.
- Text-path catalogues: unchanged; the existing byte-identity tests and bulk check pass, so EXTRACTOR_VERSION 1 holds.
- check.rs:24 (page-furniture detection over raw XML): I found nothing that makes a wrong overlay pass.

**6. Paths written outside the worktree**
- `~/.cache/scratch/rfcxml-bcp14-adversary/`: `probe.sh`, `probe.out`, `compare.sh`, `compare.out`, `suite.log`, and `cmp/` (157 temporary profiles holding fetched RFC text and XML). The mutated copy and its build directory have already been deleted.
- I did not take a worktree lease.

```findings
- file: crates/rfcspec-extract/src/lib.rs
  line: 175
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: undecided
  message: a <dl> read as one paragraph runs definitions into one sentence when the next term starts in lower case, so the second definition's <bcp14> requirement is lost (RFC 9742 loses its MUST NOT at XML line 1564)
- file: crates/rfcspec-extract/src/check.rs
  line: 358
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: an overlay requirement with source bcp14-xml, which propose now copies from the XML catalogue, passes check with an id that is not in the catalogue or a span outside the catalogue span
- file: crates/rfcspec-extract/src/check.rs
  line: 655
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: the quote check reads raw XML lines, so the exact catalogue text of R3-2 quoted at its catalogue span is refused for an XML-locked profile
- file: crates/rfcspec-extract/src/lib.rs
  line: 148
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: replacing the type="abnf" clause with false leaves all 67 unit tests green, because the unit's ABNF test line is already a rule line
- file: crates/rfcspec-extract/src/lib.rs
  line: 167
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the problem says the sentence gives no requirement even when an earlier element in that sentence gave one; no published RFC in a 552-file sample has a non-level <bcp14>
- file: crates/rfcspec-extract/src/lib.rs
  line: 146
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: XML extraction drops the capitals requirements the text path takes from figures (YANG descriptions, ASN.1), so 38 of 157 sampled RFCs get different id and level lists than from their text
```
