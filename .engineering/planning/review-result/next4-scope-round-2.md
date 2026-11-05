---
format: aep.planning-md/3
id: review-result:next4-scope-round-2
kind: review-result
status: active
title: scope critic, round 2, four follow-up epics
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

Round 1's one finding is fixed. `story:dns-query-machine` now has `depends_on: story:websocket-machine-guards` (`.engineering/planning/story/dns-query-machine.md:10`). It says it uses the guard constructs "as delivered and adds nothing to them" (lines 49-54). It claims only `state_implication` (lines 71-78, scope at 136). The graph confirms the edge. The overlap with `story:websocket-machine-guards` (`.engineering/planning/story/websocket-machine-guards.md:28-33`) is gone.

Coverage, from the parents as written:
- **`epic:bulk-catalogue`:** the index row fields and counts are in `story:bulk-summary-format`. The committed digest-pinned summary, the networked fetch, the offline `task check` and the separate regeneration step are in `story:bulk-fetch-corpus`. The site index with reviewed profiles marked, and no page per RFC, is in `story:bulk-site-index`. The RFC 6455 row equality is checked on the fixture in summary-format and on the committed file in fetch-corpus. These are two checks, not one outcome claimed twice.
- **`epic:ai-proposals`:** status and prose-level proposals are in `story:proposal-record-replay`. Machine and layout elements are in `story:proposal-machine-layout`. Cited spans checked by `rfcspec check`, the recording and the no-model-in-tests rule are in record-replay. Accept, edit and reject are in `story:proposal-review`. The live run as a step for Timo, the RFC 6455 slice and the unreviewed-count drop are in `story:websocket-proposal-slice`.
- **`epic:websocket-frames`:**
  - Ping and Pong is in `story:websocket-ping-machine` (6 requirements, 17 to 21 satisfied).
  - Data frames and fragmentation are in `story:websocket-fragments-machine` (13 requirements, 21 to 29 satisfied).
  - Both stories cover explore with witnesses and the mutants.
  - `story:websocket-machine-guards` is a named prerequisite for both machines, not reach.
  - The drafts leave some requirements unsupported with a stated reason, such as R5.4-3, R5.4-7, R5.5.3-1 and R5.5.3-2. That is a recorded decision, not a silent narrowing.
- **`epic:dns-profile`:** the RFC 1035 lock and the `dns` index row are in `story:dns-source-catalogue`. The header codec is in `story:dns-header-codec`. The query machine with its mutant and the `udp` / `by-id` semantics row are in `story:dns-query-machine`. The NTP exclusion matches nothing drafted.

No item reaches beyond its parent. The extras (the `date` and `lossy` row fields, the new `equals` layout vectors, the `state_implication` property, README and AGENTS edits) each trace to an acceptance clause or are needed for one.

What I read: all 4 epics and all 13 stories, whole bodies, plus `epic:websocket-profile` for its exclusions and `review-result:next4-scope-round-1`. Commands: `aep plan artifact show` for each, and `aep plan artifact graph` for the decomposes and depends_on edges.

Promises extracted and traced: 29 of 29. That is 7 for bulk, 8 for ai-proposals, 8 for websocket-frames and 6 for dns, plus the one NTP exclusion, which nothing drafted contradicts.

What I could not establish:
- `profiles/websocket` is not in this checkout, so I could not check overlay ids, spans or the counts 17, 21, 29, 126 and 139.
- Out of my lane (acceptance): the epic says a mutant is "reported". Ping's mutant is reported as `inconclusive` and the others as `failed`. Whether that satisfies "reported" is for `plan-critic-acceptance`.
- Out of my lane (design and parallel-safety): `rfcspec check` replays proposals (record-replay) and the `websocket` task also runs `propose --replay` (proposal-slice). Both run on every gate, which is redundant. Both claims are true, so it is not a double claim of one outcome.

```findings
[]
```
