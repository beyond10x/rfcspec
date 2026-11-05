---
format: aep.planning-md/3
id: review-result:websocket-design-round-1
kind: review-result
status: active
title: design critic, round 1, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
approve

All four stories were read against the epic, with no design finding. The one thing worth a line is the linear chain, covered in the next paragraph.

The four stories form a linear chain: source-catalogue, then close-machine, then frame-codec, then review-matrix. Each edge has its reason written in the body (shared `overlay.yaml`, `crates/rfcspec-model`, `check.rs`, `Taskfile.yml`, `Cargo.lock`, generated pages), so under the rubric it is a trade-off, not a finding. The alternative is to split the shared surfaces so the close machine and the frame codec can run in parallel. The drafter chose the order, and I did not report it.

**What you read:** 5 artifacts (the epic and 4 stories), read with `aep plan artifact show` on each, plus `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate` (valid, 38 artifacts). I also read `crates/rfcspec-cli/src/checks.rs` and `profiles/time/profile.yaml`, and read `crates/rfcspec-docs/src/catalog.rs` for how stage is derived.

- **Edges:** I walked 8 declared edges in the set, filtered from the graph's 132 total. They include the epic's `depends_on` to `epic:public-protocol-catalog`, which is `implemented`. No cycle.
- **Acceptance reads:** I traced 9 reads of state a sibling produces:
  - catalogue ids, overlay and profile (source-catalogue to close-machine);
  - the machine and layout elements the matrix statuses name;
  - the 214 keyword counts the matrix reuses;
  - the stage levels S5 and S6.

  The first 3 are recorded by a direct `depends_on` edge, and the matrix's reads of the layout and the 214-keyword counts are covered transitively through its edge to the frame-codec. The remaining reads are file-level overlaps, also ordered by the chain. None is unrecorded.
- **Arithmetic:** the matrix's keyword counts agree with the source story: 11 + 21 + 33 = 65, 10 + 9 + 10 = 29, and 214 − 65 = 149.
- **Source story acceptance:** `check` does not require semantics when the overlay has no machines (`checks.rs:54`), so the source story's acceptance does not read anything the machine story adds.

**What you could not establish, and what is out of my lane:**
- Out of my lane (scope or consistency): the epic body names the semantics layer as `websocket` over `tcp` and "the `websocket` semantics row" (epic lines 30 and 46). The close-machine and review-matrix stories define one layer `tcp` and a "`tcp` semantics row". The epic should be reconciled with the stories.
- Out of my lane (acceptance): I did not verify the many line-number and count claims against the RFC text.
- Out of my lane (parallel safety): the shared-file overlaps are all declared, so I have no parallel-safety concern to pass on.

```findings
[]
```
