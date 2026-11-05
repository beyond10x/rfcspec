---
format: aep.planning-md/3
id: review-result:next4-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, round 2, four follow-up epics
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
revision: 2
---
needs-revision

- story:dns-query-machine — the acceptance "`ess protocol explore` … prints `"verdict": "passed"`" cannot hold for the model the story describes. The server has only the states `idle` and `replied`, and `late-response` is an input offered "in either state", so it repeats. The guards story's Notes say such a cycle ends in "step bound exhausted". The story should add a server state so `late-response` is taken at most once. — `.engineering/planning/story/dns-query-machine.md:100-106` (cycle rule at `.engineering/planning/story/websocket-machine-guards.md:83-89`)

I checked this with `ess protocol explore`, using the story's witness (client `query` id 1, server `late-response` id 2) and hand-built `ess-protospec/1` models. None of these is a committed model.

| Model | Result |
|---|---|
| Story as written (client `qid`, `set`, ID guard, two discards; server `idle`/`replied`; `late-response` loops idle→idle and replied→replied) | `inconclusive`, "state bound exhausted" |
| Same, with `late-response` going idle→replied | `inconclusive`, "state bound exhausted" |
| `late-response` only from idle, going to `replied` | `inconclusive`, "dead end with outstanding queue…" |
| Three server states: `late-response` idle→`late`, and `answer` allowed from both `idle` and `late` | `passed` |
| The three-state model with the accept guard removed | `failed`, "property violated" naming `answer-matches-query` |

Round 1's six findings are fixed:
- `story:bulk-summary-format` now requires `rows_sha256` over all row lines, so editing the counts of an unlocked row is refused. `story:bulk-fetch-corpus` tests exactly that case.
- `story:proposal-record-replay` has a `requirement-from-prose` bullet with a test.
- `story:dns-query-machine` depends on `story:websocket-machine-guards` and no longer re-claims its overlay model.
- `story:websocket-ping-machine`, `story:websocket-fragments-machine` and `story:websocket-proposal-slice` are ordered by `depends_on` ping → fragments → slice. Their pinned figures are stated as holding in that order.
- The figures reconcile: 17/36/4 → 21/32/4 → 29/33/8, reviewed 57 → 70, and unreviewed 139 → 126 (82 MUST, 11 MUST NOT, 7 SHOULD, 1 SHOULD NOT, 24 MAY, 1 OPTIONAL).

What I read: all 16 ids (4 epics, 12 stories) with `aep plan artifact show`, plus the round-1 `review-result`. In the worktree `~/.local/state/worktree/trees/b10x/rfcspec/websocket-wave` I also checked:
- RFC 6455 lines 2058-2063.
- The `ProposalTask` variants in `ess/domains/review.yaml`.
- `Entry::unreviewed` in `crates/rfcspec-docs/src/catalog.rs`.
- `enabled_actions` in the `ess` repository, and the existence of `story:websocket-review-matrix`.

What I could not establish:
- No `rfcspec` binary exists, so I did not run `extract`, `lower` or `check`.
- The explore results above are from my hand-built models, not the committed ones.
- The ping and fragments machines do not list their states. I could not tell whether their "taken at most once per run" claim holds (for example the Ping input after the Close).
- The live-index figures (9,847 rows, eleven range pages, the 4 MiB page total and the 8 MiB summary limit) need a live `rfc-index.xml`.
- The feasibility of the site and size limits is outside my lane.

```findings
- file: .engineering/planning/story/dns-query-machine.md
  line: 100
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance that explore prints \"verdict\": \"passed\" cannot hold for the described model: the server has only idle and replied and late-response is an input offered in either state, a repeatable input cycle that explore ends as inconclusive \"state bound exhausted\" (reproduced on a hand-built model; a three-state server where late-response is taken once passes and its unguarded mutant fails with \"property violated\"), so the body must give the server a state shape that takes late-response at most once per run, as websocket-machine-guards Notes lines 83-89 require"
```
