---
format: aep.planning-md/3
id: review-result:websocket-parallel-round-2
kind: review-result
status: active
title: parallel critic, round 2, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
approve

The round 1 finding is fixed. `story:websocket-review-matrix` now puts the in-scope status rule in `crates/rfcspec-cli/src/checks.rs`, called from `fn check` in `crates/rfcspec-cli/src/main.rs`. Its scope lists both files. I confirmed that `checks.rs` exists and `main.rs:235` has `fn check`. No nonexistent `check.rs` is named any more (`.engineering/planning/story/websocket-review-matrix.md:88`).

The four stories form a `depends_on` chain: source, then close, then codec, then matrix. Every surface they share has an ordering edge. Each of the close, codec and matrix bodies names the shared files as the reason, so no collision inside the set goes unnamed.

**What I read:** 5 artifacts (the epic and 4 stories) and `review-result:websocket-parallel-round-1`. I ran `aep plan artifact show` on each, plus `aep plan artifact waves` and `aep plan artifact graph`. In the tree I read `crates/rfcspec-cli/src/main.rs` and `crates/rfcspec-cli/src/checks.rs`, and listed `crates/rfcspec-cli/src` and `crates/rfcspec-docs`.

**Surfaces:** 4 stories placed cited, 0 inferred, 0 unplaceable. The epic is a container with no scope. I traced 5 acceptance reads to a producer in the set, and 3 of them have a direct `depends_on` edge:
- close reads the overlay and ids from source (direct edge).
- codec reads the overlay from close (direct edge).
- matrix reads the codec layout (direct edge).
- matrix reads the close machine elements (transitive, through the chain).
- matrix reads the source keyword counts and ids (transitive, through the chain).

**What I could not establish:**
- `aep plan artifact waves` also reports collisions with `story:catalog-index`, `story:catalog-profile-page`, `story:catalog-downloads`, `story:catalog-semantics` and `story:catalog-recorded-runs`. They share `website/docs/catalog/index.md`, `website/docs/catalog/time.md`, `Taskfile.yml`, `crates/rfcspec-cli`, `crates/rfcspec-docs` and `crates/rfcspec-model`. Those items are outside the set, so I make no finding.
- The source acceptance's `rfcspec-docs generate` lines read a generator that `story:catalog-index` produces (outside the set). The only edge on the source story is `decomposes`, though the epic `depends_on` `epic:public-protocol-catalog`. This is out of my set, so it carries no finding. The operator may want that pairing reviewed.
- The numeric and line-number claims in the acceptances belong to `plan-critic-acceptance`.

```findings
[]
```
