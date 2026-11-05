---
format: aep.planning-md/3
id: review-result:websocket-design-round-2
kind: review-result
status: active
title: design critic, round 2, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
approve

The set is a four-story chain: source-catalogue, close-machine, frame-codec, review-matrix. The epic's `depends_on` edge runs to `epic:public-protocol-catalog`, which is implemented. Each chain edge has its reason written in the child's body, namely the shared `overlay.yaml`, `crates/rfcspec-model`, `check.rs`, `Taskfile.yml`, `Cargo.lock`, `crates/rfcspec-cli` and the generated pages. Under the rubric that is a file-sharing trade-off, not a serialising-chain finding. The alternative is to split those shared surfaces so the machine and codec can run in parallel, and the drafter chose the order.

The round-1 gap in the epic body (a `websocket` layer over `tcp`) is fixed. The epic now names a single layer `tcp` with no `over`, matching the close-machine and review-matrix stories.

**What you read:** 6 artifacts, namely the epic, the 4 stories and review-result round 1. I ran `aep plan artifact show` on each, then `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate`. The validator reports valid, with 42 artifacts.

- **Edges:** I walked 12 declared edges in the set, filtered from the full graph. 4 are `decomposes`, 3 are chain `depends_on` edges and 1 is the epic's `depends_on` to `epic:public-protocol-catalog` (implemented). The other 4 are the `depends_on` and `decomposes` edges of `story:catalog-index`, which the matrix story reads. Review-result edges are `reviews` only and don't count as needs-first. I found no cycle, and no edge leaves the set except to the implemented catalogue epic.
- **Acceptance reads:** I traced 8 reads of state a sibling produces. All 8 are covered by an edge, 5 directly and 3 transitively.
  - Directly recorded (5):
    - close-machine reads the catalogue ids, `overlay.yaml`, `profile.yaml` and the S4 index row from source-catalogue.
    - frame-codec reads the S5 stage and shared overlay from close-machine.
    - review-matrix reads the layout and statuses from frame-codec.
  - Transitive through the chain (3):
    - frame-codec reads the RFC 6455 octet examples and catalogue ids from source-catalogue.
    - review-matrix reads the 214 keyword counts and the G9 gap row from source-catalogue and close-machine.
- **Arithmetic:** the matrix counts agree with the source story. 11 + 21 + 33 = 65 in scope, 10 + 9 + 10 = 29 MUST or MUST NOT, 214 − 65 = 149 outside scope, and the source's 113 + 25 + 24 + 4 + 47 + 1 = 214.

**What you could not establish:**
- Out of my lane (acceptance): I did not verify the line numbers or keyword counts against the RFC 6455 text.
- Out of my lane (parallel safety): the shared-file overlaps are all declared with an edge, so I have none to pass on.

```findings
[]
```
