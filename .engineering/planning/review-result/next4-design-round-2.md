---
format: aep.planning-md/3
id: review-result:next4-design-round-2
kind: review-result
status: active
title: design critic, round 2, four follow-up epics
relations:
- reviews: epic:bulk-catalogue
- reviews: epic:ai-proposals
- reviews: epic:websocket-frames
- reviews: epic:dns-profile
- reviews: story:bulk-summary-format
- reviews: story:bulk-fetch-corpus
- reviews: story:bulk-site-index
- reviews: story:proposal-record-replay
- reviews: story:proposal-review
- reviews: story:websocket-proposal-slice
- reviews: story:proposal-machine-layout
- reviews: story:websocket-machine-guards
- reviews: story:websocket-ping-machine
- reviews: story:websocket-fragments-machine
- reviews: story:dns-source-catalogue
- reviews: story:dns-header-codec
- reviews: story:dns-query-machine
revision: 1
---
approve

I read all 17 artifacts, and no finding would change what was drafted.

The four round-1 findings are fixed in the bodies and in the graph:
- **Guards and `state_implication`:** `story:dns-query-machine` now has `depends_on story:websocket-machine-guards` (dns-query-machine.md:49-75). Its model work is cut to `state_implication` only.
- **Websocket pins:** `story:websocket-proposal-slice` depends on `story:websocket-fragments-machine` (websocket-proposal-slice.md:216-218, 248-259). Its pins are stated as 70 + A reviewed and 126 − A unreviewed, relative to the fragments state.
- **Ping fixture:** `story:proposal-machine-layout` now tests against its own fixture `tests/fixtures/no-machines/` and never reads `profiles/websocket/overlay.yaml` (proposal-machine-layout.md:338-344). The `AlreadyExists` collision is gone.
- **DNS lock and bulk summary:** `story:bulk-fetch-corpus` has `depends_on story:dns-source-catalogue`. `story:bulk-summary-format` depends on the last stories of the ai epic, and `EXTRACTOR_VERSION` is introduced after the dns epic (bulk-summary-format.md:50-54). The dns stories no longer read bulk state.

The order frames → dns → ai → bulk holds at story level:
- **dns after frames:** `story:dns-source-catalogue` depends on `story:websocket-fragments-machine`, the last frames story.
- **ai after dns:** `story:proposal-record-replay` depends on `story:dns-query-machine`, the last dns story. The other ai stories descend from it.
- **bulk after ai:** `story:bulk-summary-format` depends on `story:proposal-machine-layout` and `story:websocket-proposal-slice`. The other bulk stories descend from it.

`aep plan artifact waves` places all 17 stories in 18 waves, so there is no cycle. The four epics carry only `depends_on epic:websocket-profile`, so nothing orders them at epic level, but the story edges enforce the order completely.

The chain is long (the stories sit in waves 5 to 18, mostly one per wave). Each cross-epic and intra-epic edge has a written reason beside it, either a shared file or the recorded order, so I do not report it as a serialising chain. The unrecorded alternative would be splitting the shared surfaces (`AGENTS.md`, `Taskfile.yml`, `main.rs`, `Cargo.lock`), which is the drafter's trade-off.

What I read: 17 artifacts, with `aep plan artifact show` on each. I also ran `relations`, `graph`, `waves` and `validate` (valid, 67 artifacts), and read `review-result:next4-design-round-1`. I checked the worktree for `quote_problem`, the absence of `EXTRACTOR_VERSION`, `ProposalTask`, `layout.frame-header`, and the 57 / 17 / 36 / 4 pins at `catalog.rs:1019`. I walked about 61 non-review edges, including edges to `epic:websocket-profile`, `story:websocket-review-matrix` and `epic:public-protocol-catalog` outside the set, and found no cycle. I traced about 14 acceptance reads to a producer inside the set, and an edge records every one.

What I could not establish:
- **Redundant edge, not a finding:** `story:proposal-machine-layout depends_on story:websocket-ping-machine` is now transitive through `story:proposal-review` and `story:proposal-record-replay`. Its body (proposal-machine-layout.md:390-391) justifies only the review edge. It is harmless.
- **Outside my lane (parallel-safety), no verdict impact:** `story:bulk-summary-format` justifies its edge to `story:websocket-proposal-slice` by shared `Cargo.lock`, `main.rs`, `rfcspec-model` and `rfcspec-extract`. That slice's scope touches none of them (websocket-proposal-slice.md:285-291). The edge is really the recorded epic order.
- **Outside my lane (acceptance):** `story:proposal-record-replay` does not name which unreviewed items its three recorded proposals cover.
- **Operator-visible, not a design defect:** the bulk epic waits behind `story:websocket-proposal-slice`, which needs Timo's live recording run (websocket-proposal-slice.md:267-274). This is a consequence of the chosen order.

```findings
[]
```
