---
format: aep.planning-md/3
id: review-result:catalog-design-round-2
kind: review-result
status: active
title: design critic, round 2, protocol catalogue
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

I found no design defects in the set. Round 1's blocker is fixed. The index story now renders the profile name as plain text (`.engineering/planning/story/catalog-index.md`, index acceptance) and says `story:catalog-profile-page` adds the link. The profile-page story claims the link and depends on the index story. The "first of five stories" wording is also correct now (`catalog-index.md:30`).

**What I read.** I read 7 artifacts in full: the epic, the 5 stories and `review-result:catalog-design-round-1`. I used `aep plan artifact show` on each, plus `relations`, `graph` and `validate`; `validate` reported valid across 29 artifacts. I also read `website/docusaurus.config.ts` and confirmed `website/docs/design.md` exists, which the profile-page story links to.

**Graph.**
- I walked 11 edges among the set, the epic and the two cleared decision-blockers, including the edge from the epic to `epic:rfc868-thin-slice` outside the set. There is no cycle.
- The stories chain index, profile-page, downloads, semantics, recorded-runs. Each `depends_on` edge has its shared-file reason written in the downstream story's own body, so under the rubric this is a recorded trade-off and not a finding.
- You kept round 1's warning on the serial order deliberately, so I do not repeat it. The alternative is still a per-section module split of `catalog.rs`.
- I traced about 14 acceptance reads to producers inside the set, and every one is covered by a `depends_on` path, directly or through the chain. Examples are the index's `Overlay.kind`, the fixture profile and the generator's outputs map, which later stories read. The `check` rule on semantics is read by the recorded-runs round trip, and it is covered by a direct edge.
- Reads of `profiles/time/generated/` and of the `story:rfc868-verify-probe` server go to implemented artifacts outside the set, so I did not count them.
- No slice is horizontal: each story leaves a visible, checkable page or column.

**What I could not establish.**
- Out of my lane (scope): the epic's acceptance puts conformance "no recorded run" on the index row. It is delivered only by `story:catalog-recorded-runs`, so the epic closes only when the whole chain lands.
- Out of my lane (parallel safety): `story:catalog-recorded-runs` says its `rfcspec check` stale-run test uses a fixture profile. After `story:catalog-semantics`, `check` refuses a profile that has machines but no semantics. The fixture therefore needs either no machines or declared semantics, and the body does not say which.
- I did not run `task website`, so whether Docusaurus counts links the way the stories assume is unverified.

```findings
[]
```
