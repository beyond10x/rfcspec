---
format: aep.planning-md/3
id: review-result:rfc868-parallel-round-1
kind: review-result
status: active
title: parallel critic, round 1, RFC 868 thin slice
relations:
- reviews: epic:rfc868-thin-slice
- reviews: story:rfc868-source-reader
- reviews: story:rfc868-extract-overlay
- reviews: story:rfc868-lower-protospec
- reviews: story:rfc868-generate-codec
- reviews: story:rfc868-verify-probe
revision: 1
---
needs-revision

story:rfc868-lower-protospec — it and story:rfc868-generate-codec both land on `crates/rfcspec-cli` (cited, both Scope lines) and neither mentions the other. Both also write under `profiles/time/generated/` (`*.protospec.yaml` against `rust/`, separate paths in one directory) and both add crates, so both change the workspace `Cargo.toml` and `Cargo.lock`. Only generate names `Cargo.toml`. The `depends_on` edges leave them unordered. Remedies: an ordering edge recording `crates/rfcspec-cli` as its reason, or splitting the surface — .engineering/planning/story/rfc868-lower-protospec.md:29, .engineering/planning/story/rfc868-generate-codec.md:32

story:rfc868-lower-protospec — it and story:rfc868-verify-probe both land on `crates/rfcspec-cli` (cited), they are unordered (verify depends on generate, lower on extract) and neither body says so. Remedies: an ordering edge recording the shared file as its reason, or splitting the surface — .engineering/planning/story/rfc868-lower-protospec.md:29, .engineering/planning/story/rfc868-verify-probe.md:31

story:rfc868-generate-codec — its acceptance writes the fifth example into `profiles/time/overlay.yaml`, which its Scope omits. story:rfc868-extract-overlay owns that file and is ordered before generate. story:rfc868-lower-protospec reads the file for its machines, and lower and generate are unordered. Remedies: name the file in generate's Scope and order generate against lower, or move the finding into the overlay extract-overlay delivers so generate only reads it — .engineering/planning/story/rfc868-generate-codec.md:25, .engineering/planning/story/rfc868-generate-codec.md:32, .engineering/planning/story/rfc868-lower-protospec.md:14

What I read: 6 artifacts (the epic and 5 stories) with `aep plan artifact show` on each and `aep plan artifact graph`. I also read `Cargo.toml`, `Taskfile.yml` and `ls crates profiles` in the repository. The surface counts are:
- **Surfaces:** 5 of 5 stories cited, 0 inferred, 0 unplaceable. The epic has no scope of its own.
- **Acceptance reads:** 6 traced to a producer in the set. Five are recorded by `depends_on`, directly or through the chain, and the overlay read by lower is not (the third finding).

What I could not establish:
- `Cargo.lock` is named by no story. I count it as a shared surface by inference, because every new crate changes it.
- The reader and extract stories share `crates/rfcspec-model` and `crates/rfcspec-cli`, but a `depends_on` edge orders them, so I report no collision.
- Out of my lane: extract-overlay's acceptance does not list the fifth-example review finding that generate says it records in the overlay.

```findings
- file: .engineering/planning/story/rfc868-lower-protospec.md
  line: 29
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "story:rfc868-lower-protospec and story:rfc868-generate-codec both land on crates/rfcspec-cli (cited, both Scope lines, generate at .engineering/planning/story/rfc868-generate-codec.md:32) and both write under profiles/time/generated/ and change the workspace Cargo.toml and Cargo.lock; they are unordered by depends_on and neither body says so. Remedies: an ordering edge recording the shared file as its reason, or splitting the cli surface."
- file: .engineering/planning/story/rfc868-lower-protospec.md
  line: 29
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "story:rfc868-lower-protospec and story:rfc868-verify-probe both land on crates/rfcspec-cli (cited, verify at .engineering/planning/story/rfc868-verify-probe.md:31); they are unordered (verify depends on generate, lower on extract) and neither body says so. Remedies: an ordering edge recording the shared file as its reason, or splitting the cli surface."
- file: .engineering/planning/story/rfc868-generate-codec.md
  line: 25
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "story:rfc868-generate-codec's acceptance writes profiles/time/overlay.yaml (cited), which its Scope at line 32 omits; story:rfc868-extract-overlay owns that file and story:rfc868-lower-protospec reads it (.engineering/planning/story/rfc868-lower-protospec.md:14), and lower and generate are unordered. Remedies: name the file in Scope and order generate against lower, or move the finding into the overlay extract-overlay delivers."
```
