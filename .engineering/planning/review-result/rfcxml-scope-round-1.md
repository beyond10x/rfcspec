---
format: aep.planning-md/3
id: review-result:rfcxml-scope-round-1
kind: review-result
status: active
title: scope critic, round 1, RFCXML reader
relations:
- reviews: epic:rfcxml-reader
- reviews: story:rfcxml-source-reader
- reviews: story:rfcxml-bcp14-extract
- reviews: story:bulk-xml-summary
- reviews: story:bulk-xml-regenerate
revision: 1
---
needs-revision
epic:rfcxml-reader — the Outcome still promises ABNF "to parse into a grammar and a codec" and diagrams "to read into layouts", but the set delivers only verbatim figure blocks. No artifact in the store records that the codec and layout work is deferred: the "later wire-layer epic" does not exist, there is no decision-blocker for it, and the epic body is unchanged. The epic's own sentence is "`<sourcecode type="abnf">` gives ABNF grammars as text to parse into a grammar and a codec, with no AI (milestone M1)". The decision-blocker for xref shows what an honest record looks like. Amend the epic Outcome and title to say ABNF-to-codec and diagram-to-layout are out of scope, or draft the blocker or epic that holds them. — .engineering/planning/epic/rfcxml-reader.md:15
story:bulk-xml-regenerate — The story regenerates the committed 9,847-row `bulk/rfc-summary.jsonl`, adds a `Source` column to the website RFC index pages and changes the `bulk-regenerate` task. No sentence of the epic asks for it. The epic's Outcome concerns reading RFCXML into the normalized document, requirements, anchors and the RFC 9220 test case. Either add a bulk and index-page bullet to the epic, which is the operator's call, or drop the story. — .engineering/planning/epic/rfcxml-reader.md:11
story:bulk-xml-summary — The story adds `rfcspec bulk fetch` and `summarise` over XML, `BulkRow.format`, the `--xml` flag, index `has_xml` and a `catalogue.yaml` view field. None of this is traceable to the epic Outcome. The only link is the "1,390 RFCs" count, which sits in a sentence about the reader. The epic needs a bulk-path bullet or the story needs to go. — .engineering/planning/epic/rfcxml-reader.md:11

**What you read.** 6 artifacts: the epic, the 4 stories, and `decision-blocker:rfcxml-xref-reference` with `epic:rfc-explorer`. I ran `aep plan artifact show` on each and `aep plan artifact graph`. I also grepped `docs/design/rfcspec.md`, `AGENTS.md` and `.engineering/planning`.

**Promise count.** I extracted 6 promises from the epic:
- Read XML into the normalized document: traced to `story:rfcxml-source-reader`.
- Exact bcp14 requirements, one per sentence: traced to `story:rfcxml-bcp14-extract`.
- Section anchors give ids and spans: traced to `story:rfcxml-source-reader`.
- ABNF grammar and codec: not claimed.
- Diagram layouts: not claimed.
- xref references for the explorer: not claimed by a story, but covered by the open `decision-blocker:rfcxml-xref-reference`, so it is not a gap.

That is 4 of 6 traced to an item. The 5 promises other than the xref one are the ones I counted against the set. The RFC 9220 test case and the Limits section are claimed too.

**What I could not establish.** I did not see the drafter's own report, only the summary in the task. If the ABNF and diagram deferral is recorded elsewhere, finding 1 narrows to "record it in the epic body". Out of my lane: whether `bulk-xml-regenerate`'s acceptance can be checked without network access (acceptance), and the ordering of the `depends_on` chain (parallel-safety).

```findings
- file: .engineering/planning/epic/rfcxml-reader.md
  line: 15
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "The Outcome promises ABNF to parse into a grammar and a codec and diagrams to read into layouts, but the set delivers only verbatim figure blocks, and no store artifact records the deferral: the wire-layer epic does not exist, there is no blocker, and the epic body is unchanged. Amend the epic or draft the holder."
- file: .engineering/planning/epic/rfcxml-reader.md
  line: 11
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:bulk-xml-regenerate regenerates the committed 9,847-row bulk summary and adds a Source column to the website RFC index pages; no sentence of the epic Outcome asks for either."
- file: .engineering/planning/epic/rfcxml-reader.md
  line: 11
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:bulk-xml-summary adds bulk fetch and summarise over XML, BulkRow.format, the --xml flag and a catalogue view field; none of this is traceable to the epic Outcome, whose only link is the 1,390 RFC count."
```
