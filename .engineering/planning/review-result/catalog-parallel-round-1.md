---
format: aep.planning-md/3
id: review-result:catalog-parallel-round-1
kind: review-result
status: active
title: parallel critic, round 1, protocol catalogue
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

All five stories chain by `depends_on`, so no pair can run at the same time, and every reader of another story's state is ordered behind its producer. One defect remains. The structured `scope` field is empty on all five stories, so `aep plan artifact waves` cannot assess them.

- story:catalog-index — its structured `scope` field is empty (`aep plan artifact show story:catalog-index --format json` prints `"scope": []`), so `aep plan artifact waves` lists it as unassessed even though its `## Scope` prose names `crates/rfcspec-docs/src/catalog.rs`, `main.rs`, `crates/rfcspec-model`, `website/sidebars.ts` and `website/docusaurus.config.ts` (cited) — .engineering/planning/story/catalog-index.md:70
- story:catalog-profile-page — empty structured `scope`. The prose names `crates/rfcspec-docs` `catalog.rs`, but its body also needs `main.rs` for the orphan rule over profile pages and leaves it out of `## Scope` (cited) — .engineering/planning/story/catalog-profile-page.md:58
- story:catalog-downloads — empty structured `scope`. The prose names `catalog.rs`, `main.rs`, `website/static/downloads/` and `website/docs/catalog/time.md` (cited) — .engineering/planning/story/catalog-downloads.md:51
- story:catalog-semantics — empty structured `scope`. The prose names `crates/rfcspec-model`, `crates/rfcspec-cli`, `ess/domains/profile.yaml`, `profiles/time/profile.yaml` and `catalog.rs` (cited) — .engineering/planning/story/catalog-semantics.md:62
- story:catalog-recorded-runs — empty structured `scope`. The prose names `crates/rfcspec-conformance`, `crates/rfcspec-cli`, `ess/domains/profile.yaml`, `catalog.rs` and the regenerated pages (cited) — .engineering/planning/story/catalog-recorded-runs.md:60

Each fix is `aep plan artifact scope <id> --add <path>`, marking every path cited. The collisions already named in the prose are real and already ordered by the chain: `catalog.rs` and `main.rs` are shared by index, profile-page and downloads; `ess/domains/profile.yaml`, `crates/rfcspec-cli` `check` and `catalog.rs` are shared by semantics and recorded-runs. Once the scopes are recorded, `waves` should report them as ordered collisions rather than as unassessed stories.

**What I read.** 6 artifacts (the epic and 5 stories) with `aep plan artifact show <id>`, plus `aep plan artifact graph`, `relations`, `waves` and `show --format json`. I also listed `profiles/time`, `crates/` and `ess/` to confirm the surfaces exist. Items with a surface established: 5 of 5 cited, 0 inferred, 0 unplaceable. Acceptance reads traced to a producer in the set: 4, namely the profile page reading the index's fixture and `Overlay.kind`, downloads reading `time.md`, semantics reading `time.md` and `catalog.rs`, and recorded-runs reading the `check` rule and `profile.yaml`. A `depends_on` edge records 3 directly (profile-page to index, downloads to profile-page, recorded-runs to semantics) and the fourth through the chain (semantics to downloads to profile-page).

**What I could not establish.**
- `rfcspec-docs` has no `catalog.rs` yet, and `website/docs/catalog/` and `website/static/downloads/` do not exist. All three are created by the set, so they are creations, not collisions.
- Out of my lane, acceptance: `story:catalog-index` says it is "the first of three stories" and "the two later stories", but five stories are drafted (.engineering/planning/story/catalog-index.md:7).
- Out of my lane, scope: `story:catalog-profile-page` needs `main.rs` and its own fixtures but does not list `main.rs` in its Scope section (see its finding above).

```findings
- file: .engineering/planning/story/catalog-index.md
  line: 70
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the structured scope field is empty (aep plan artifact show story:catalog-index --format json prints scope []), so aep plan artifact waves reports the story unassessed although its Scope prose cites catalog.rs, main.rs, rfcspec-model, sidebars.ts and docusaurus.config.ts; record them with aep plan artifact scope (cited)"
- file: .engineering/planning/story/catalog-profile-page.md
  line: 58
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the structured scope field is empty so waves reports the story unassessed; record catalog.rs and website/docs/catalog/time.md, and add crates/rfcspec-docs/src/main.rs, which the body's orphan-rule acceptance needs but Scope omits (cited)"
- file: .engineering/planning/story/catalog-downloads.md
  line: 51
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the structured scope field is empty so waves reports the story unassessed; record catalog.rs, main.rs, website/static/downloads/ and website/docs/catalog/time.md (cited)"
- file: .engineering/planning/story/catalog-semantics.md
  line: 62
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the structured scope field is empty so waves reports the story unassessed; record rfcspec-model, rfcspec-cli, ess/domains/profile.yaml, profiles/time/profile.yaml and catalog.rs (cited)"
- file: .engineering/planning/story/catalog-recorded-runs.md
  line: 60
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the structured scope field is empty so waves reports the story unassessed; record rfcspec-conformance, rfcspec-cli, ess/domains/profile.yaml, catalog.rs and the regenerated pages (cited)"
```
