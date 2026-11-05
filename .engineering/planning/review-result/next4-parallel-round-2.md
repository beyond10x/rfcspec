---
format: aep.planning-md/3
id: review-result:next4-parallel-round-2
kind: review-result
status: active
title: parallel critic, round 2, four follow-up epics
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

- story:websocket-machine-guards — it extends the close machine's overlay model in `crates/rfcspec-model/src/lib.rs` and `crates/rfcspec-extract/src/check.rs` (cited), but has no `depends_on` edge to any story of `epic:websocket-profile`. That epic is still active, and `aep plan artifact waves` puts this story in wave 1, so it collides with `story:websocket-close-machine`, `story:websocket-frame-codec` and `story:websocket-source-catalogue` on both files. The collision rests on `waves` output and on the guards body. The close and frame-codec stories (both active) are outside the set, so the call is yours. `story:bulk-summary-format` records its own edge to `story:websocket-review-matrix`, the wave's last story. This story needs the same edge, or an explicit statement that the wave has landed. — `.engineering/planning/story/websocket-machine-guards.md:20-25`, `aep plan artifact waves` (`collision: story:websocket-close-machine story:websocket-machine-guards crates/rfcspec-model/src/lib.rs`, same for `check.rs`)

**What I read:** 17 artifacts (4 epics and 13 stories), plus `review-result:next4-parallel-round-1` and the three active websocket stories for context. Commands were `aep plan artifact show`, `graph`, `waves` and `relations`, and `cat -n` on every story file.

**Round 1 status:**
- **Fixed:** all seven round-1 findings. The edges are present in `aep plan artifact graph`:
  - dns-source-catalogue to websocket-fragments-machine;
  - dns-query-machine to websocket-machine-guards;
  - proposal-record-replay to dns-query-machine;
  - websocket-proposal-slice to websocket-fragments-machine;
  - proposal-machine-layout to websocket-ping-machine;
  - bulk-summary-format to proposal-machine-layout and websocket-proposal-slice;
  - bulk-fetch-corpus to dns-source-catalogue.
- **Bodies updated:** the dns-query-machine body now narrows to `state_implication`. The bulk-fetch-corpus acceptance now carries the RFC 1035 row. The proposal-machine-layout fixture no longer reads the committed overlay.
- **Wave order:** `waves` now reproduces the planned order guards, ping, fragments, dns-source-catalogue, header-codec, query-machine, record-replay, review, then layout and slice together, then the three bulk stories.

**Surfaces:** 13 stories placed cited, 0 inferred, 0 unplaceable. The four epics are containers with no scope. I traced 5 cross-epic acceptance reads to a producer in the set, and 5 of them are recorded by a direct `depends_on` edge. The in-epic chains are fully edged.

**What I could not establish:**
- `profiles/websocket` does not exist in this tree, so no websocket path or symbol could be checked against code.
- `story:proposal-machine-layout` and `story:websocket-proposal-slice` are both in wave 15 with no edge between them. `waves` lists no direct collision for the pair. The only overlap is `crates/rfcspec-cli/tests` (a directory, in the layout scope) against `crates/rfcspec-cli/tests/websocket.rs` (in the slice scope), which is inferred. The slice's recordings replay through `crates/rfcspec-propose`, which the layout story also edits. Under your serial plan I don't count this as a finding, but the two have no stated order.
- Out of my lane: the pins (17 to 21, 57 reviewed, 126 unreviewed, 70 reviewed) and the RFC line numbers belong to the acceptance critic. Whether `dns-query-machine` should have been a separate story belongs to the design critic.

```findings
- file: .engineering/planning/story/websocket-machine-guards.md
  line: 20
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "extends the close machine's overlay model in crates/rfcspec-model/src/lib.rs and crates/rfcspec-extract/src/check.rs (cited) with no depends_on edge to any story of the still-active epic:websocket-profile, so aep plan artifact waves places it in wave 1 and reports collisions with story:websocket-close-machine, story:websocket-frame-codec and story:websocket-source-catalogue on both files (those stories are outside the set, so the call is the operator's); add an edge to story:websocket-review-matrix recording the shared files, as story:bulk-summary-format already does, or state that the wave has landed"
```
