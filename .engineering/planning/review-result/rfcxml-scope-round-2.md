---
format: aep.planning-md/3
id: review-result:rfcxml-scope-round-2
kind: review-result
status: active
title: scope critic, round 2, RFCXML reader
relations:
- reviews: epic:rfcxml-reader
- reviews: story:rfcxml-source-reader
- reviews: story:rfcxml-bcp14-extract
- reviews: story:bulk-xml-summary
- reviews: story:bulk-xml-regenerate
revision: 1
---
approve

Every promise in the revised epic is claimed by exactly one story, and no story reaches past the epic. Extracted: 9 promises (4 Outcome bullets, 5 Acceptance lines). Traced to an item: 9 of 9. The Limits section and the RFC 9220 test case are also carried.

| Promise | Claimed by |
|---|---|
| XML read into the same normalized document (`epic.md` Outcome intro) | story:rfcxml-source-reader |
| `<bcp14>` marks give each requirement exactly, no capitals heuristic | story:rfcxml-bcp14-extract |
| `<section>` anchors give section ids and spans | story:rfcxml-source-reader (Sections) |
| `<sourcecode>` and `<artwork>` kept verbatim with `type` | story:rfcxml-source-reader (Figures) |
| Bulk index reads XML; 1,390 RFCs counted from `<bcp14>` marks; each row names its source | story:bulk-xml-summary (the mechanism) and story:bulk-xml-regenerate (the committed summary and the `Source` column) |
| Acceptance: fetch and read | story:rfcxml-source-reader |
| Acceptance: extract | story:rfcxml-bcp14-extract |
| Acceptance: `bulk fetch`, `summarise` and `check` | story:bulk-xml-summary |
| Acceptance: 9,847 rows of which 1,390 are XML, and the `Source` column | story:bulk-xml-regenerate |

The three round-1 findings are resolved:
- **ABNF and diagram deferral:** the epic's "Out of scope, and where it is held" section now names epic:wire-layer (draft), which exists and is `informed_by` this epic.
- **Bulk reach (both findings):** the bulk bullet in the Outcome makes both bulk stories traceable. Neither claims anything beyond it: the `format` field, `--xml`, the catalogue view field and the `Source` column are all how "each RFC index row says which source it was read from" is delivered.

Other checks:
- **xref:** the epic names `decision-blocker:rfcxml-xref-reference` as the holder. The graph shows it `blocks epic:rfc-explorer`, and it is open.
- **Other exclusions:** automatic XML choice in `fetch` and XML-backed profile overlays are excluded. story:rfcxml-source-reader and story:bulk-xml-regenerate each repeat the same exclusions, so none of them claims work the epic excludes.
- **Overlap:** bulk-xml-summary and bulk-xml-regenerate split cleanly. The first does the code and the fixture, the second the committed data and the pages, so neither will be marked done for the other's work.

**What I read:** 5 artifacts (the epic and its 4 stories), plus the round-1 verdict, epic:wire-layer and decision-blocker:rfcxml-xref-reference. Commands: `aep plan artifact show` for each, `aep plan artifact graph`, and `cat` of the critic procedure and rubric.

**What I could not establish:** none. Two unease notes, both out of my lane, neither affecting the verdict:
- **Acceptance lane:** the epic's acceptance says the XML reader's "section blocks equal the text reader's". story:rfcxml-source-reader pins that to sections 1 to 5 only (14 blocks). It does not say how the references section is compared.
- **Parallel-safety lane:** the `depends_on: epic:ntp-profile` edge is deferred until that epic merges, as the stories state.

```findings
[]
```
