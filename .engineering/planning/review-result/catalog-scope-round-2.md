---
format: aep.planning-md/3
id: review-result:catalog-scope-round-2
kind: review-result
status: active
title: scope critic, round 2, protocol catalogue
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

All five stories were read against the epic's promises. Nothing found would change what was drafted.

- The round 1 finding is fixed. `story:catalog-recorded-runs` now has a Stage bullet (`.engineering/planning/story/catalog-recorded-runs.md:349-350`). A profile with at least one report under `runs/` reports S7, with a fixture test asserting S7 and `time` staying at S6. This extends the stage rule of `story:catalog-index`.
- All 9 promises I extracted from the epic trace to a story, and I found no gap. The epic text is at `.engineering/planning/epic/public-protocol-catalog.md`.
- Nothing claims anything the epic does not ask for. `verify --report`, `check --all` and `Overlay.kind` trace to the epic's conformance verdict and kind column, and to the two cleared decision-blockers.
- No two stories claim one outcome.
- The only narrowing is named. S1 to S3 are never reported, and `story:catalog-index` says why.

What I read: 7 artifacts (the epic, the five stories and `review-result:catalog-scope-round-1`), via `aep plan artifact show` on each and `aep plan artifact graph`. The graph shows no other artifact claiming any part of the epic. I did not run `kinds` or `relations`.

What I could not establish:
- Whether the `story:catalog-semantics` rule that a profile with machines but no semantics is refused goes beyond "shows its semantics". I read it as traceable to the epic's acceptance and did not make it a finding.
- Out of my lane, for the design critic: `story:catalog-index` line 38 still says S7 "store[s] nothing … and [is] never reported", while `story:catalog-recorded-runs` now reports it. The recorded-runs story names the extension, so this is a wording tension, not a scope defect.

```findings
[]
```
