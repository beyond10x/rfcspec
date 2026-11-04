---
format: aep.planning-md/3
id: review-result:rfc868-parallel-round-2
kind: review-result
status: active
title: parallel critic, round 2, RFC 868 thin slice
relations:
- reviews: epic:rfc868-thin-slice
- reviews: story:rfc868-source-reader
- reviews: story:rfc868-extract-overlay
- reviews: story:rfc868-lower-protospec
- reviews: story:rfc868-generate-codec
- reviews: story:rfc868-verify-probe
revision: 1
---
approve

Read 6 artifacts (the epic and its five stories) with `aep plan artifact show`, plus `aep plan artifact graph` and `aep plan artifact waves`. I also read the story bodies under `.engineering/planning/story/` and the repo's `Taskfile.yml`.

**Surfaces.** All 5 stories are placed from their own `## Scope` sections, so all 5 are cited, none inferred, none unplaceable. They all share `crates/rfcspec-cli` and `crates/rfcspec-model`, plus `Cargo.toml` and `Cargo.lock`, and `profiles/time/` is shared between several. Every pair that shares a file is ordered by a `depends_on` chain: reader, then extract, then lower, then generate, then verify. The lower and generate bodies name the shared files as the reason for the ordering. No pair can run concurrently, so there is no unnamed collision.

**Readers and producers.** I traced 6 acceptance reads to a producer in the set, and 6 are ordered by `depends_on`:
- extract reads the reader's normalized document (direct edge).
- lower reads `overlay.yaml` from extract (direct edge).
- generate reads the overlay's review finding from extract (ordered through lower, no direct edge).
- generate's reads of the lowering story's output directory are ordered by the generate-to-lower edge.
- verify reads the generated codec crate from generate (direct edge).
- verify reads the overlay's requirement count from extract (ordered through generate and lower, no direct edge).

Transitive ordering is enough here because the chain is a total order.

**What I could not establish.**
- `aep plan artifact waves` reports all 5 stories as unassessed ("nothing selected declares a scope"). The scopes live only in body text, not in `aep artifact scope`. A later wave planner cannot see them, but nothing runs in parallel today, so I did not count it as a finding.
- Only the lowering story lists `Taskfile.yml`. The epic's `task check` acceptance also needs rules for `rfcspec check`, `generate --check`, `cargo test -p rfc868-time` and the conformance tests, and no other story lists the file. The reader and extract stories also create crates without listing `Cargo.toml` or `Cargo.lock`. Both are scope omissions in a fully serial chain. Whether they are right is for the design and scope critics, and they did not set my verdict.
- `profiles/time/` and the new crates do not exist yet, so nothing there could be grepped. Only the `crates/rfcspec-docs` crate exists.

```findings
[]
```
