---
format: aep.planning-md/3
id: review-result:catalog-acceptance-round-1
kind: review-result
status: active
title: acceptance critic, round 1, protocol catalogue
relations:
- reviews: epic:public-protocol-catalog
- reviews: story:catalog-index
- reviews: story:catalog-profile-page
- reviews: story:catalog-downloads
- reviews: story:catalog-semantics
- reviews: story:catalog-recorded-runs
revision: 1
---
needs-revision

- epic:public-protocol-catalog — the epic has no `## Acceptance` section (headings are Outcome, What the catalogue shows, Depends on), so nothing states when the whole catalogue counts as done; its sibling `epic:rfc868-thin-slice` closes with one — .engineering/planning/epic/public-protocol-catalog.md:19
- story:catalog-recorded-runs — the headline says the report `verify --report` writes is accepted by `rfcspec check` and rendered by the catalogue, but no bullet performs that round trip: the verify tests write to a scratch dir and assert only pass/fail verdicts, and the check and render tests use hand-made fixture profiles — .engineering/planning/story/catalog-recorded-runs.md:20
- story:catalog-recorded-runs — the headline says "a report whose overlay digest is stale fails `task check`", but the only test is a fixture profile run through `rfcspec check`, and `task check` runs `check` only on `profiles/time` (no `runs/`), so a stale report in any other profile would not fail it — .engineering/planning/story/catalog-recorded-runs.md:20

**What I read:** 6 artifacts (the 6 ids given), whole bodies via `aep plan artifact show <id>`. I also read `aep plan artifact kinds` and `aep plan artifact lifecycle epic`, ran `aep plan artifact validate` (valid, 25 artifacts), and checked the tree. In `profiles/time/overlay.yaml` the 4 MUST, 2 SHOULD and 6 satisfied counts hold, and `generated/` holds the four files the downloads story names. The enum wire spellings in `ess/domains/profile.yaml` match the semantics story's values. `page::cell`, `page::SOURCE`, `website/docs/design.md` and `onBrokenLinks: 'throw'` all exist.

**Could not establish:** whether `task website` runs offline (`npm ci`), and whether the `.well-known/b10x-routes.json` provenance check works, since I did not run the build. Both are outside what I read.

**Out of my lane (not counted):**
- scope: the epic promises a conformance verdict on the index and communication semantics per profile, but `story:catalog-index` and `story:catalog-profile-page` exclude them and defer to later stories.
- design: `story:catalog-index` says it is "the first of three stories", but five are drafted.

```findings
- file: .engineering/planning/epic/public-protocol-catalog.md
  line: 19
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the epic has no Acceptance section (headings: Outcome, What the catalogue shows, Depends on), so nothing states when the whole catalogue counts as done"
- file: .engineering/planning/story/catalog-recorded-runs.md
  line: 20
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the headline says the report that verify --report writes is accepted by rfcspec check and rendered by the catalogue, but no bullet performs that round trip: verify tests only assert pass/fail verdicts into a scratch dir and the check and render tests use hand-made fixtures"
- file: .engineering/planning/story/catalog-recorded-runs.md
  line: 20
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the headline says a report with a stale overlay digest fails `task check`, but the only test is a fixture profile through rfcspec check, and task check runs check only on profiles/time, which has no runs/"
```
