---
format: aep.planning-md/3
id: review-result:hardening-parallel-round-1
kind: review-result
status: active
title: parallel critic, round 1, hardening follow-ups
relations:
- reviews: epic:hardening-follow-ups
- reviews: story:overlay-type-checks
- reviews: story:proposal-level-check
- reviews: story:channel-capacity
revision: 1
---
approve

I found no collision between these stories that the plan fails to name. I read all four artifacts and the code they cite.

- **`story:overlay-type-checks` and `story:channel-capacity`:** both scope `crates/rfcspec-extract/src/check.rs` (cited, both scopes). A `depends_on` edge runs from `story:channel-capacity` to `story:overlay-type-checks`, so the shared file is ordered.
- **`story:channel-capacity` and `story:overlay-type-checks` on `profiles/websocket/overlay.yaml`:** `story:channel-capacity` edits `profiles/websocket/overlay.yaml`, and `story:overlay-type-checks` builds its test overlays on the committed websocket copy. The same edge orders them.
- **`story:proposal-level-check`:** it shares no file with the others. Its code and tests are in `rfcspec-propose` (`lib.rs`, `tests.rs`, `elements_tests.rs`).
  - Its tests copy `profiles/websocket/overlay.yaml`, which `story:channel-capacity` edits. That is harmless:
    - All ten recorded proposals are `status` proposals with no `context`, so replay does not read the overlay's element list.
    - The tests in `crates/rfcspec-propose/src/tests.rs` and `elements_tests.rs` name no `fragments.*` elements, so the fragments edit cannot break them.
  - `rfcspec-propose` depends on `rfcspec-extract` only as a library. That is a rebuild dependency, not a shared file.

**What I read:** 4 artifacts (`aep plan artifact show` on each, plus `aep plan artifact graph`). I also read `crates/rfcspec-propose/src/lib.rs` and `tests.rs` (`context`, `elements`, `replay`, `copy_profile`, `before_the_slice`), `crates/rfcspec-ess/src/lib.rs` (`CAPACITY`), and the ten files in `profiles/websocket/proposals/`. Surfaces: 3 of 3 stories cited, 0 inferred, 0 unplaced. I traced 1 acceptance read to a producer in the set (`story:overlay-type-checks` reading the overlay that `story:channel-capacity` edits), and a `depends_on` edge covers it.

**What I could not establish:** the `depends_on` edge carries no recorded reason. I could not tell whether the edge states `check.rs` as its reason. The CLI output shows only the edge, not a reason field, and neither body mentions the shared file. That is a presentation point, not a collision. Whether the split is right is the design critic's lane, not mine.

```findings
[]
```
