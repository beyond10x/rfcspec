---
format: aep.planning-md/3
id: review-result:explorer-scope-round-1
kind: review-result
status: active
title: scope critic, round 1, RFC explorer
relations:
- reviews: epic:rfc-explorer
- reviews: story:xref-reference-extract
- reviews: story:explorer-rfc-view
- reviews: story:explorer-search
- reviews: story:explorer-profile-layer
revision: 1
---
needs-revision

- story:xref-reference-extract — no sentence of `epic:rfc-explorer` promises citations or references between RFCs, so this story, and the "RFCs it cites" part of `story:explorer-rfc-view`, reach beyond the parent. The epic's only link promise is the update graph: "the RFCs it updates or obsoletes and is updated or obsoleted by, as links". The only trace is `epic:rfcxml-reader` ("which blocks epic:rfc-explorer, where the links are used"). The decision blocker quotes the epic as saying "`<xref>` gives the references between RFCs for the explorer", but that sentence is not in the epic body. The fix is to add the citation promise to the epic (or have the story cite the operator direction that adds it), so the work arrives with approved authority. — `.engineering/planning/epic/rfc-explorer.md:20` (compare `.engineering/planning/epic/rfcxml-reader.md:27`)

**What I read.**
- 6 artifacts: the epic, its 4 stories and the cleared decision blocker.
- Commands: `aep plan artifact show` for each, `aep plan artifact graph` and `aep plan artifact relations`.
- Source: the epic and `epic:rfcxml-reader` files.
- Promises: I extracted 7 from the epic and traced all 7 to an item.
  - Every RFC opens as structured data: `story:explorer-rfc-view`.
  - Metadata, requirements by section and level, and the update graph as links: `story:explorer-rfc-view`.
  - Search and filter by number, title, level counts and status: `story:explorer-search`.
  - Profiles as the deepest layer, covering overlay status, review decision, recorded proposal, machines and layouts, and witnesses: `story:explorer-profile-layer`.
  - Only the first promise (every RFC opens as structured data) is exact. The others are claimed with narrowing, below.
- Narrowings:
  - "Requirements by section and level" becomes counts only, with the reason stated in `story:explorer-rfc-view`.
  - "Keyword level counts" becomes presence or absence of a level, named in `story:explorer-search` § Open questions.
  - "Exploration witnesses" becomes committed input witnesses, named in `story:explorer-profile-layer` § Open questions.
  - All three are recorded, so none is silent and none is a finding.
- No two items claim the same outcome. The extraction and the display are split by `depends_on`.

**What I could not establish.**
- The `rfc-explorer.md` frontmatter shows revision 1 only. Whether an operator message adds the citation promise outside the store is not visible to me.
- Out of my lane, with no effect on the verdict:
  - `story:explorer-search` adds stream, area and source-format filters, and a profile mark per RFC, beyond the epic's parenthetical list (number, title, level counts, status). It reads as small extension, not a finding.
  - The epic's Order names `story:websocket-proposal-slice-2`; `story:explorer-profile-layer` cites 85 accepted or edited and 29 rejected for RFC 6455 and depends on that work. That ordering is for the design and parallel-safety critics.

```findings
- file: .engineering/planning/epic/rfc-explorer.md
  line: 20
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:xref-reference-extract, and the cited-RFC references in story:explorer-rfc-view, reach beyond the parent: the epic promises only the update graph as links and no sentence promises citations between RFCs, so the epic must carry that promise or the story must cite where the operator asked for it"
```
