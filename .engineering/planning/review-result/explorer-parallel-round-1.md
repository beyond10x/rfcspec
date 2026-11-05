---
format: aep.planning-md/3
id: review-result:explorer-parallel-round-1
kind: review-result
status: active
title: parallel critic, round 1, RFC explorer
relations:
- reviews: epic:rfc-explorer
- reviews: story:xref-reference-extract
- reviews: story:explorer-rfc-view
- reviews: story:explorer-search
- reviews: story:explorer-profile-layer
revision: 1
---
needs-revision

story:explorer-profile-layer — it and story:explorer-search both extend `crates/rfcspec-docs/src/explorer.rs` and write under `explorer/src/`, and both depend only on story:explorer-rfc-view with no edge between them, so one wave would hold both and neither body says so (cited, both Scope sections). `explorer.rs` and `explorer/` do not exist yet. story:explorer-rfc-view creates both, so the collision is on code that builds on a new file. Two remedies, no choice made here: an ordering edge between the two stories that records the shared file as its reason, or splitting the surface so each owns its own module. For the split, that means a profile module apart from the index module in the docs crate, and named subdirectories under `explorer/src/` (list view and filters against the reviewed-layer section). — .engineering/planning/story/explorer-profile-layer.md:51, .engineering/planning/story/explorer-profile-layer.md:53, .engineering/planning/story/explorer-search.md:39, .engineering/planning/story/explorer-search.md:41

**What you read:** 5 artifacts (the epic and 4 stories) with `aep plan artifact show` on each, plus `aep plan artifact graph`. I read the tree for `ess/domains/catalogue.yaml`, `crates/rfcspec-docs/src/`, `Taskfile.yml` and `git log` (HEAD 345fafb sits on main df9f4b8). Surfaces: 4 cited, 0 inferred, 1 unplaced. The unplaced one is the epic, a container whose body names no files. Acceptance reads traced to a producer in the set: 3, and all 3 are recorded by `depends_on` (rfc-view to xref-reference-extract, search to rfc-view, profile-layer to rfc-view).

**What you could not establish:**
- Where `explorer/src/` splits into files. The bodies name only the directory, so I cannot tell whether the list view and the RFC view share a router or entry file.
- Whether `crates/rfcspec-docs/src/main.rs` wiring is touched by both search and profile-layer. Only rfc-view names it.
- Out of my lane, for the design and acceptance critics: story:explorer-rfc-view does not say whether the References in the per-range shards come from the `bulk/rfc-sections.jsonl` sidecar or from a second source.

```findings
- file: .engineering/planning/story/explorer-profile-layer.md
  line: 51
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "this story and story:explorer-search both extend crates/rfcspec-docs/src/explorer.rs (new, created by story:explorer-rfc-view) and write under explorer/src/, both depend only on story:explorer-rfc-view with no edge between them, and neither body names the other; add an ordering edge recording the shared file or split the surface (cited: explorer-search.md:39 and :41, explorer-profile-layer.md:51 and :53)"
```
