---
format: aep.planning-md/3
id: review-result:explorer-scope-round-2
kind: review-result
status: active
title: scope critic, round 2, RFC explorer
relations:
- reviews: epic:rfc-explorer
- reviews: story:xref-reference-extract
- reviews: story:explorer-rfc-view
- reviews: story:explorer-search
- reviews: story:explorer-profile-layer
revision: 1
---
approve

The round-1 finding is closed. `epic:rfc-explorer` now carries the citation promise at `.engineering/planning/epic/rfc-explorer.md:23`. Both of its sources check out.

- **`epic:rfcxml-reader` revision 1:** commit `3085a61` holds the quoted sentence "`<xref>` gives the references between RFCs for the explorer" at line 17.
- **`decision-blocker:rfcxml-xref-reference`:** it is cleared, option A, and its order (ESS declaration, extraction story, explorer) matches the story `depends_on` chain.

**Coverage.** I extracted 8 promises from the epic's current body and traced all 8 to an item.

| Promise (epic line) | Claimed by |
|---|---|
| Any RFC opens as structured data (14) | `story:explorer-rfc-view` |
| Profiles are the deepest layer (14) | `story:explorer-profile-layer` |
| One page per RFC with metadata, requirements by section and level, and the update graph as links (20) | `story:explorer-rfc-view` |
| Search and filter by number, title, level counts and status (21) | `story:explorer-search` |
| Profiled RFC shows overlay status, review decision and proposal, machines and layouts, and witnesses (22) | `story:explorer-profile-layer` |
| RFCXML-read RFC links the RFCs it cites, as `Reference` (23) | `story:xref-reference-extract` extracts; `story:explorer-rfc-view` displays |
| Order: after `story:websocket-proposal-slice-2` and `approval-record:ntp-profile` (34) | Not an outcome; recorded as `depends_on` and in the epic |
| Acceptance, four lines (27-30) | One named story each, with a matching acceptance in the story body |

**The other three defects.**
- **Reach beyond the parent:** none.
  - The `reference_counts` self and non-RFC counts come from the decision blocker's "count them in the read summary, do not store them".
  - The `Reference` fields come from the same decision.
  - The sidecar and per-range shards serve the epic's "requirements by section and level" and "one page per RFC".
  - The explorer links added to range and profile pages serve "as links". They are small.
- **Two items claiming one outcome:** none. `story:xref-reference-extract` says it changes no bulk output and leaves the sidecar to `story:explorer-rfc-view`.
- **Silent narrowing:** none. Each narrowing is named in the story that makes it:
  - requirement counts only, without sentences, in `story:explorer-rfc-view` § Decision;
  - level presence or absence in `story:explorer-search` § Open questions;
  - committed input witnesses in `story:explorer-profile-layer` § Open questions.

**What I read.** 7 artifacts: the epic, its four stories and the cleared decision blocker, plus `review-result:explorer-scope-round-1` and `epic:rfcxml-reader` revision 1 from git history. I ran `aep plan artifact show` on each, `aep plan artifact graph`, and `git show` on every revision of `rfcxml-reader.md`.

**What I could not establish.**
- None that affects the verdict.
- Out of my lane, with no effect on the verdict: the extra filters (stream, area, source format) and the profile marks in `story:explorer-search` go beyond the epic's parenthetical list. They are an extension of "search and filter" and I did not raise them in round 1 either.

```findings
[]
```
