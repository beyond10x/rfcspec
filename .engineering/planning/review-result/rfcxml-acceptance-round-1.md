---
format: aep.planning-md/3
id: review-result:rfcxml-acceptance-round-1
kind: review-result
status: active
title: acceptance critic, round 1, RFCXML reader
relations:
- reviews: epic:rfcxml-reader
- reviews: story:rfcxml-source-reader
- reviews: story:rfcxml-bcp14-extract
- reviews: story:bulk-xml-summary
- reviews: story:bulk-xml-regenerate
revision: 1
---
needs-revision
epic:rfcxml-reader — the body has no Acceptance section (only Outcome, Limits and Test case), so nothing says how anyone would tell the epic is done, and the Outcome's five bullets (`<xref>` for the explorer among them) are not tied to a check — .engineering/planning/epic/rfcxml-reader.md:9
story:rfcxml-source-reader — the one-sentence acceptance at line 55 covers only the heading/paragraph equality with the text path, while the title's "section anchors and bcp14 marks" (the `<bcp14>` positions at 14 of 13-in-all and the figure `type`) are checked only in the sub-bullets, so the headline can pass with no marks or figure types recorded; the headline must name them, or the bullets must be split out — .engineering/planning/story/rfcxml-source-reader.md:55
story:bulk-xml-summary — the headline acceptance starts from a prepared fixture index plus locked texts and the XML fixture, so it can pass while `rfcspec bulk fetch` never writes `<dir>/xml/rfc9220.xml` (the title's first verb) and while `bulk check` keeps accepting a format mismatch; those independent outcomes live only in the bullets — .engineering/planning/story/bulk-xml-summary.md:53
story:bulk-xml-regenerate — the headline joins "`task check` exits 0" (already true before this work, so it does not mark the change) with "the committed summary has 9,847 rows, 1,390 `"format":"xml"`", and omits the second half of the title, the `Source` column on the RFC index pages, which is checked only in the Pages bullet; the sentence should say what was true before (no row carries `format`; pages have no `Source` column) and cover both outcomes or be split — .engineering/planning/story/bulk-xml-regenerate.md:40

What I read: 5 artifacts of 5 given (epic:rfcxml-reader, story:rfcxml-source-reader, story:rfcxml-bcp14-extract, story:bulk-xml-summary, story:bulk-xml-regenerate), via `aep plan artifact show <id>` for each, plus `aep plan artifact lifecycle story` and `kinds`. I also checked claims against the tree: `bulk/rfc-summary.jsonl` has 9,847 rows plus a header, and has the RFC 9220 text digest and counts `{"MAY":1,"SHOULD":1}`. I ran the existing `rfcspec` binary on `rfc9220.txt`. It gave 14 blocks for sections 1 to 5, and R3-1 SHOULD and R3-2 MAY as `capitals-text`, so those claims hold.

What I could not establish:
- The XML line spans (130 to 134), the 13 `<bcp14>` elements and the XML digest, because `rfc9220.xml` is not in the corpus cache and I did not fetch it.
- Whether 1,390 rows have an `XML` format, because the index was not re-read.
- `story:rfcxml-bcp14-extract` is the one story I found no acceptance defect in.
- Out of lane, no effect on the verdict: `story:bulk-xml-summary` and `story:bulk-xml-regenerate` both edit `crates/rfcspec-docs/src/bulk.rs` and `AGENTS.md`, and three stories share `crates/rfcspec-cli/src/main.rs`. That is for the design and parallel-safety critics.

```findings
- file: .engineering/planning/epic/rfcxml-reader.md
  line: 9
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the body has no Acceptance section (only Outcome, Limits and Test case), so nothing says how anyone would tell the epic is done, and the Outcome's five bullets are not tied to a check"
- file: .engineering/planning/story/rfcxml-source-reader.md
  line: 55
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the one-sentence acceptance covers only heading/paragraph equality with the text path, while the title's section anchors and bcp14 marks and the figure type are checked only in the sub-bullets, so the headline can pass with none of them recorded"
- file: .engineering/planning/story/bulk-xml-summary.md
  line: 53
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the headline acceptance starts from a prepared fixture index and locked files, so it can pass while `bulk fetch` never writes the XML file and while `bulk check` accepts a format mismatch; those independent outcomes live only in the bullets"
- file: .engineering/planning/story/bulk-xml-regenerate.md
  line: 40
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the headline joins `task check` exits 0 (already true before the work) with the committed-summary row and format counts, and omits the Source column on the RFC index pages named in the title, which is checked only in the Pages bullet"
```
