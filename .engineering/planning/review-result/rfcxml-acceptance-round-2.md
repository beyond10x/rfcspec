---
format: aep.planning-md/3
id: review-result:rfcxml-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, round 2, RFCXML reader
relations:
- reviews: epic:rfcxml-reader
- reviews: story:rfcxml-source-reader
- reviews: story:rfcxml-bcp14-extract
- reviews: story:bulk-xml-summary
- reviews: story:bulk-xml-regenerate
revision: 1
---
needs-revision

story:rfcxml-source-reader — the headline acceptance names the heading and paragraph equality, the `<bcp14>` marks and the figure `type`, but not the section `anchor` that the title promises ("with section anchors and bcp14 marks"). The headline can pass with no anchor recorded, because anchors are checked only in the Sections bullet. — .engineering/planning/story/rfcxml-source-reader.md:55 (the title is at line 6, the Sections bullet at line 79)

story:bulk-xml-regenerate — the headline says the `Source` column shows `xml` or `text`, while the Pages bullet says `XML` or `text`. A checker comparing the generated pages gets a different answer from each sentence, so the headline must use the same cell values as the Pages bullet. — .engineering/planning/story/bulk-xml-regenerate.md:42 (conflicts with line 63)

**What I read:** 5 of the 5 ids given (epic:rfcxml-reader, story:rfcxml-source-reader, story:rfcxml-bcp14-extract, story:bulk-xml-summary, story:bulk-xml-regenerate). I ran `aep plan artifact show` on each and read the whole body, and I read review-result:rfcxml-acceptance-round-1 to see which findings had landed. In the tree I checked `Taskfile.yml`: `bulk-regenerate` is networked and `check` does not run it. I also checked that `bulk/rfc-summary.jsonl` has 9,848 lines (a header plus 9,847 rows) and that no row carries `format`.

All four round-1 findings landed:
- The epic now has an Acceptance section with five checks, each tied to a story.
- The bulk-xml-summary headline now covers the XML fetch and the `bulk check` format refusal, each with a "before" state.
- The bulk-xml-regenerate headline now states the before state and covers both the summary and the `Source` column.
- The source-reader headline now covers the bcp14 marks and the figure type.

story:rfcxml-bcp14-extract has no finding.

**What I could not establish:**
- I did not re-read the RFC 9220 XML (line spans, 13 `<bcp14>` marks, digest) or the live RFC index (1,390 XML entries). Both depend on network fetches, and I have not verified them.
- The figure-`type` clause in the source-reader headline holds trivially over RFC 9220, which has no figures. The inline unit test in the Figures bullet is what actually checks it, so I left it out as a finding.
- Out of lane, so I did not let them set the verdict: bulk-xml-summary and bulk-xml-regenerate both edit `crates/rfcspec-docs/src/bulk.rs` and `AGENTS.md`. Both stories also depend on the unmerged epic:ntp-profile.

```findings
- file: .engineering/planning/story/rfcxml-source-reader.md
  line: 55
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the headline acceptance records bcp14 marks and figure types but not the section anchor the title promises, so it can pass with no anchor recorded; anchors are checked only in the Sections bullet"
- file: .engineering/planning/story/bulk-xml-regenerate.md
  line: 42
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the headline gives the Source column values as `xml` or `text` while the Pages bullet (line 63) gives `XML` or `text`, so the check has two different expected cell values"
```
