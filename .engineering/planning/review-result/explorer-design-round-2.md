---
format: aep.planning-md/3
id: review-result:explorer-design-round-2
kind: review-result
status: active
title: design critic, round 2, RFC explorer
relations:
- reviews: epic:rfc-explorer
- reviews: story:xref-reference-extract
- reviews: story:explorer-rfc-view
- reviews: story:explorer-search
- reviews: story:explorer-profile-layer
revision: 1
---
needs-revision

- story:explorer-profile-layer — Its body never says how the RFC view learns which `profiles/<name>.json` to load for `?rfc=<n>`, and without that mapping `?rfc=9220` cannot show "no reviewed layer". Half of "which profile locks an RFC" sits in story:explorer-search (the `rfc-index.json` profile marks, read from `locked`), and the other half is in this story (per-profile JSON built from `Entry`). Close it by naming one source for the mapping in this body. If the source is the index marks, say so, which also gives the existing `depends_on story:explorer-search` a data reason beside its shared-file reason. If it is the four profile files' `rfcs` field, say that instead, so the two derivations cannot disagree. — `.engineering/planning/story/explorer-profile-layer.md:15`, `.engineering/planning/story/explorer-profile-layer.md:24`, `.engineering/planning/story/explorer-search.md:18`

The round-1 findings have all landed. The only open finding is new in this round and is a warning, not a blocker.

- **References path, was a blocker.** `explorer-rfc-view.md:34` now says the sidecar rows carry each RFCXML RFC's References, read from `Document.references`. `story:xref-reference-extract` says it changes no bulk output.
- **Rust types, was a warning.** `explorer-rfc-view.md:36` generates both Rust and TypeScript from ESS, and both are drift-checked.
- **Index-row type, was a warning.** `explorer-search.md:21` and `:41` add the index-row type and its generated TypeScript to the scope.
- **Shape.** The edges are xref-reference-extract, then explorer-rfc-view, then explorer-search, then explorer-profile-layer. The last edge has its shared-file reason written at `explorer-profile-layer.md:70`. There is no cycle and no horizontal slice: rfc-view carries data, types, shards and UI end to end.

**What you read:** five artifacts (the epic and four stories), plus review-result:explorer-design-round-1. I ran `show` on each, `aep plan artifact relations`, `graph` and `validate` (valid, 133 artifacts). I also read `ess/domains/catalogue.yaml`, `uilab/Taskfile.yml`, `ess generate types --help` and `crates/rfcspec-docs/src/bulk.rs`.

- **Edges.** I walked the 3 `depends_on` edges (rfc-view to xref; search to rfc-view; profile-layer to rfc-view and search) and the 4 `decomposes` edges, and the epic's 2 `informed_by` edges. I followed the `depends_on` edges outside the set and found none, so there is no cycle.
- **Acceptance reads.** I traced 6 reads to a producer in the set, and an edge records all 6.

**What I could not establish:**
- Whether the shard shapes named in rfc-view ("ESS value types over `Rfc`, `Section`, `LevelCount`, `Reference`") are new structs or the existing entities used as generation roots. In `ess/domains/catalogue.yaml`, `Rfc`, `Section` and `Reference` are entities, and `Section` has no level-count field. No root is named, so this is an acceptance question, not a design one.
- `story:websocket-proposal-slice-2`, named in the epic's Order, is not an artifact in this store. It is outside the set and I did not judge it.

**Out of my lane:**
- Scope: `story:explorer-profile-layer` generates view rows with `--root <view>.row` and says "drift-checked", but its Scope lists no `Taskfile.yml` change.

```findings
- file: .engineering/planning/story/explorer-profile-layer.md
  line: 24
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the body never says how the RFC view learns which profiles/<name>.json to load for an RFC number, a mapping that story:explorer-search's rfc-index.json profile marks also carry; name one source (the index marks or the profile files' rfcs field) so the two derivations of which profile locks an RFC cannot disagree"
```
