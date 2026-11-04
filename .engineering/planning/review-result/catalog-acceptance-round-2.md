---
format: aep.planning-md/3
id: review-result:catalog-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, round 2, protocol catalogue
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

**What I read:** 6 artifacts plus `review-result:catalog-acceptance-round-1`, read as whole bodies with `aep plan artifact show <id>` for each id. I ran `aep plan artifact validate`, which reported 29 artifacts and valid. I checked the tree: `Taskfile.yml`, `crates/rfcspec-docs/src/main.rs`, `crates/rfcspec-cli/src/main.rs`, `website/docusaurus.config.ts` and `profiles/time`.

**Round 1 findings:**
- **Epic acceptance:** `epic:public-protocol-catalog` now has an `## Acceptance` section. It is an end-to-end check with concrete values: the `time` row, the two semantics rows, the four downloads, and the one-byte edit making `task check` exit 1.
- **Round trip:** `story:catalog-recorded-runs` now has a bullet that copies `profiles/time` to a scratch directory, runs `verify --report` there, then checks that `rfcspec check` exits 0 and that the rendered page shows the verdict, target and date. After a one-byte overlay edit the same check exits 1 naming the report.
- **Stale reports in `task check`:** the story now covers `task check` through an `rfcspec check --all profiles` step and a test that `--all` visits each profile and fails when one fails. `Taskfile.yml` is in its scope. Today `task time` runs `check --profile profiles/time` only and the CLI has no `--all`, so this is a real before and after change.

**Tree checks:**
- `Provenance` is an existing `rfcspec-docs` subcommand that writes `.well-known/b10x-routes.json`, as `story:catalog-index` assumes.
- `onBrokenLinks: 'throw'` and `baseUrl: '/rfcspec/'` are set.
- `profiles/time/generated/` holds exactly the four files the downloads story names, and `website/docs/catalog/` does not exist yet, so the generated-page criteria describe a real transition.
- `profiles/time/profile.yaml` declares no semantics today, so the semantics acceptance changes something.

**Could not establish:**
- Whether `task website` (`npm ci`) builds offline and the built route list contains `/rfcspec/docs/catalog/`. I did not run the build.

**Out of my lane (not counted):** none new. The round-1 out-of-lane notes (scope: the epic's conformance and semantics promises sit in later stories; design: "first of three stories") are fixed in the bodies: `story:catalog-index` now says "first of five stories".

```findings
[]
```
