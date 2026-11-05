---
format: aep.planning-md/3
id: review-result:next4-scope-round-1
kind: review-result
status: active
title: scope critic, round 1, four follow-up epics
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
needs-revision

story:dns-query-machine — claims the same overlay-model constructs that story:websocket-machine-guards already claims, and has no `depends_on` on it, so both would be marked done for one outcome — .engineering/planning/story/dns-query-machine.md:48-54 (against .engineering/planning/story/websocket-machine-guards.md:20-35)

The two bodies cover the same ground:
- The DNS story says "This story carries exactly those four into the overlay model, the check and the lowering". Its four are participant `fields`, the `set` effect, transition `guards` over literal, `{local:}` and `{input:}` operands, and a property.
- The machine-guards story is "the prefactoring both machine stories depend on". It claims participant local fields, `set`, equality guards over literal, `{local:}` and `{input:}` operands, and the `never_state` property.
- Both land in `crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-ess` and `crates/rfcspec-extract/src/check.rs`, with the same refusals (undeclared field, trigger lacking a field).
- `aep plan artifact graph` shows dns-query-machine depends only on dns-header-codec, so nothing orders it after machine-guards.
- The fix is for dns-query-machine's body to say it depends on `story:websocket-machine-guards`. It should claim only what that story lacks, which is the `state_implication` property (`other`, `other_state`), and drop the other three.
- Machine-guards is the first of the two to land, so it keeps its claim. Parallel safety (ordering between the two) is out of my lane.

Everything else traces both ways:
- **epic:bulk-catalogue (4 promises, 4 traced).** The index row fields, the committed digest-pinned summary, the networked fetch with an offline check, and the site index with profile links are claimed once each, in summary-format, fetch-corpus and site-index. The RFC 6455 row equality is claimed in summary-format and fetch-corpus.
- **epic:ai-proposals (7 promises, 7 traced).**
  - Status and prose-level proposals: proposal-record-replay.
  - Machine and layout elements: proposal-machine-layout.
  - Cited spans checked by `rfcspec check`, the recording, and the no-model-in-tests rule: proposal-record-replay.
  - The reviewer accepting, editing or rejecting a proposal: proposal-review.
  - The live run as a step for Timo, and the slice of RFC 6455: websocket-proposal-slice.
- **epic:websocket-frames (5 promises, 5 traced).** Ping and Pong is in ping-machine; data frames and fragmentation are in fragments-machine. The explore verdicts and mutants, a status for every requirement of §5.4, §5.5.2, §5.5.3 and §5.6, and the rising satisfied count (17 to 21 to 29) are all claimed. Only machine-guards has no sentence of its own in the epic, but it is a named prerequisite that the Pong-echo and `never_state` mutants need, so I did not flag it as reach. R5.5.3-2 passes from ping-machine to fragments-machine, and ping-machine's body says so (lines 72-74), so it is a recorded hand-off, not a double claim.
- **epic:dns-profile (6 promises, 6 traced).** The RFC 1035 lock and the `dns` index row are in dns-source-catalogue; the header codec is in dns-header-codec; the query machine with its mutant and the `udp` / `by-id` semantics row are in dns-query-machine. The NTP exclusion in the epic matches nothing drafted.

**What I read:**
- The four epics and all 13 stories, whole bodies.
- The store checks: `aep plan artifact graph` for the set's edges, and the epic files for line numbers.
- Also `epic:websocket-profile`, to check the exclusions.

**What I could not establish:**
- `profiles/websocket` is not in this checkout, so I could not check overlay ids, spans, or counts such as 17, 21, 29, 139 and 126.
- Whether the mutants that explore reports as `inconclusive` count as "reported" belongs to the acceptance critic.
- Proposal-record-replay has `rfcspec check` replay proposals while websocket-proposal-slice adds a replay step to the `websocket` task. That overlap is for the design and parallel-safety critics.

```findings
- file: .engineering/planning/story/dns-query-machine.md
  line: 48
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the body claims participant fields, the set effect, equality guards and {local}/{input} operands in the overlay model, check and lowering, the same outcome story:websocket-machine-guards (websocket-machine-guards.md:20-35) already claims, and it has no depends_on on that story, so both would be marked done for one outcome; it should depend on story:websocket-machine-guards and claim only the state_implication property"
```
