---
format: aep.planning-md/3
id: review-result:hardening-scope-round-1
kind: review-result
status: active
title: scope critic, round 1, hardening follow-ups
relations:
- reviews: epic:hardening-follow-ups
- reviews: story:overlay-type-checks
- reviews: story:proposal-level-check
- reviews: story:channel-capacity
revision: 1
---
approve

What I read: 7 artifacts (the epic, its 3 stories, `review-result:frames-adversary-1`, `review-result:dnsai-adversary-1`, `story:websocket-fragments-machine`), using `aep plan artifact show` on each and `aep plan artifact graph`. I read the epic first and extracted 4 promises:
- close frames finding 2, the overlay type checks (`check.rs:168`);
- close dnsai finding 3, the level comparison (`lib.rs:611`);
- lift the capacity-4 limit noted in `story:websocket-fragments-machine`, so the fragments machine sends every frame kind in one run;
- `rfcspec check` and `rfcspec propose` refuse earlier what a later stage refuses.

All 4 trace to a story:
- finding 2 to `story:overlay-type-checks`;
- finding 3 to `story:proposal-level-check`;
- the capacity limit and the single-run claim to `story:channel-capacity`;
- the earlier-refusal promise to the first two stories.

Nothing reaches beyond the epic.
- The `capacity` field in `story:channel-capacity` is the only route to the single-run promise.
- The `check` refusal of capacity 0, the regenerated website pages and the downloads all follow from that field.
- `story:overlay-type-checks` covers the three refusals named in finding 2 and nothing else.
- `story:proposal-level-check` leaves `requirement-from-prose` free to choose its level, as finding 3 allows.
- The three stories claim distinct outcomes. The graph shows no other item claiming any of them.

What I could not establish:
- Other findings in the two adversary results are not claimed: `review.rs:260` (reject of a layout or machine proposal), the finding marked undecided, and the earlier findings now fixed on other stories. The epic's "Sources:" line names only finding 2, finding 3 and the capacity note, so these are not promised and not gaps.
- Out of my lane: `story:channel-capacity` and `story:overlay-type-checks` both edit `check.rs`, and the dependency edge between them already orders them. That belongs to the parallel-safety critic.

```findings
[]
```
