---
format: aep.planning-md/3
id: review-result:hardening-design-round-1
kind: review-result
status: active
title: design critic, round 1, hardening follow-ups
relations:
- reviews: epic:hardening-follow-ups
- reviews: story:overlay-type-checks
- reviews: story:proposal-level-check
- reviews: story:channel-capacity
revision: 1
---
approve

I read the epic and its three stories in full, the graph, the relations vocabulary and the validator output. No finding changes what was drafted.

- **Cycles and chains:** There are no cycles and the set does not serialise. `story:channel-capacity` depends_on `story:overlay-type-checks`, and `story:proposal-level-check` has no dependency edge, so it can run in parallel with the other two.
- **Shared surface:** The one edge matches a shared file. Both stories edit `crates/rfcspec-extract/src/check.rs`, which the type-check story's scope and the capacity story's "refuses 0" acceptance both name. That is the file-order edge the rubric describes, so it is not a chain defect.
- **Split abstraction:** None. The three stories split along separate gaps, and each can be shown working on its own. They are the overlay type errors in `rfcspec check`, the propose-time level check in `rfcspec-propose`, and the lowering channel capacity.
- **Hidden dependencies:** I found none. The type-check story reads the committed websocket overlay and profiles. The level-check story reads the ten existing recorded proposals. The capacity story reads its own regenerated `fragments.protospec.yaml`. None of these is produced by a sibling, except that the capacity story's edited `overlay.yaml` must still pass the new type checks. The existing edge covers that.
- **Horizontal slices:** None. Each story is an end-to-end behaviour change with its own test.

**What I read:** 4 artifacts, with `aep plan artifact show` on each, plus `relations`, `graph` and `validate`. The store validates clean (79 artifacts). I walked 4 declared edges in the graph output, all within the set and the epic. I traced 1 acceptance read of produced state to a producer in the set, and an edge records it. I also grepped `check.rs` for its function layout.

**What I could not establish:**
- Whether the existing check pass can type an input-field operand from the trigger message without a model change in `rfcspec-model`. That is a feasibility and scope question, outside my lane.
- Acceptance checkability and coverage of the two cited adversary findings belong to the acceptance and scope critics. I did not assess them.

```findings
[]
```
