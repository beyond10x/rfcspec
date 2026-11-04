---
format: aep.planning-md/3
id: review-result:catalog-parallel-round-2
kind: review-result
status: active
title: parallel critic, round 2, protocol catalogue
relations:
- reviews: epic:public-protocol-catalog
- reviews: story:catalog-index
- reviews: story:catalog-profile-page
- reviews: story:catalog-downloads
- reviews: story:catalog-semantics
- reviews: story:catalog-recorded-runs
revision: 1
---
approve

All five stories now carry a structured `scope`, every path marked cited. `aep plan artifact waves` runs them in five ordered waves (index, profile-page, downloads, semantics, recorded-runs) and reports no unassessed story inside the set. It lists 25 collisions, and every one is between two stories that a `depends_on` chain already orders. That chain runs from recorded-runs through semantics, downloads and profile-page to index. The round-1 finding is fixed. Round 1 also asked for `main.rs` in the profile-page scope, and it is there now.

**What I read.** Six artifacts (the epic and five stories) with `aep plan artifact show`, plus `review-result:catalog-parallel-round-1`, `aep plan artifact graph` and `aep plan artifact waves`. I also read `aep plan artifact show story:rfc868-verify-probe` and listed `crates/`, `crates/rfcspec-docs/src`, `profiles/` and `website/`.
- **Surfaces:** 5 of 5 stories cited, 0 inferred, 0 unplaceable. The epic has no scope field, so it is not assessed as an item.
- **Acceptance reads:** I traced 6 to a producer in the set, and all 6 have a `depends_on` edge, direct or through the chain. They are:
  - profile-page reads index's fixture and `Overlay.kind`;
  - downloads reads the page that profile-page generates;
  - semantics reads the page and `catalog.rs`;
  - recorded-runs reads the `check` rule and `profile.yaml`;
  - recorded-runs also reads the stage rule and the index column from index;
  - recorded-runs also reads the page section from profile-page.

**What I could not establish.**
- `crates/rfcspec-docs/src/catalog.rs`, `website/docs/catalog/` and `website/static/downloads/` do not exist yet. Several stories would create them, so those are creations, not collisions.
- `story:catalog-recorded-runs` lists the directory `profiles` as one scope entry, which is wider than the paths its body writes. It also leaves out `crates/rfcspec-model`, which its prose covers with "if the report type lives there". Neither changes the verdict, because the story is last in the chain and collides with nothing unordered.
- The five `story:rfc868-*` items show as unassessed in `waves`. They are implemented and outside this set, so I did not assess them. `story:catalog-recorded-runs` reads `story:rfc868-verify-probe`'s in-process server, and that story is implemented.
- Out of my lane: the round-1 note that `story:catalog-index` says "three stories" while five are drafted. I did not re-check whether it was fixed.

```findings
[]
```
