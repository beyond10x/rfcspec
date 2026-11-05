---
format: aep.planning-md/3
id: review-result:wire-parallel-round-2
kind: review-result
status: active
title: Parallel-safety critic on epic:wire-layer and the backlog, round 2
relations:
- reviews: story:frame-codec-vectors
- reviews: story:websocket-grammar-review
- reviews: story:text-reader-structure
- reviews: story:send-payload-type-check
revision: 1
---
needs-revision

Round 1 is resolved. Finding 7 (wire-m1-record) is resolved because the full gate is now the coordinator's wave close, not that story's acceptance. Findings 1 to 6 are resolved by the edges send-payload-type-check → fixed-arm-bounds-check → xml-overlay-check → websocket-grammar-review → status-evidence and by `Order` sections in each body. I checked all 42 collisions printed by `aep plan artifact waves --status draft` against the `depends_on` edges, direct or transitive. Every one is ordered.

Waves 5 and 6 hold. The pairs inside wave 5 are disjoint, and so are the pairs inside wave 6.
- **Wave 5:** wire-scaffold edits Cargo manifests, the model crate's `lib.rs`, `crates/rfcspec-model/src/wire.rs` and new files, and says it touches nothing under `crates/rfcspec-extract/src/`. send-payload-type-check edits only `check.rs`. text-reader-structure edits `crates/rfcspec-doc/src/lib.rs` and `crates/rfcspec-extract/src/lib.rs`.
- **Wave 6:** abnf-core edits `crates/rfcspec-abnf/*`. catalogue-grammars edits the extract and model `lib.rs` plus `crates/rfcspec-wire/src/grammars.rs`. Its "every `Catalogue` literal" has exactly one site, `crates/rfcspec-extract/src/lib.rs:277`, so it does not reach `check.rs`. fixed-arm-bounds-check edits only `check.rs`.
- **Wave 6 edges:** each wave-6 story's edges run into wave 5.
- **Two more checks:** `ExtractGrammar`, `ExtractLayout` and `Grammars` already exist in `ess/domains/catalogue.yaml`, so the two ESS scenarios touch no domain file. The website data does not render the new `grammars` field.

Two new findings, one outside waves 5 and 6 and one inside wave 5.

story:frame-codec-vectors — it collides with story:websocket-grammar-review on `profiles/websocket/profile.yaml` (cited, both bodies) and on `crates/rfcspec-model/src/lib.rs` (inferred). No `depends_on` edge or sentence orders them, and `aep plan artifact waves` printed no collision for the pair. The two are the dispatch order's waves 4's pair, "frame-codec-vectors, websocket-grammar-review". frame-codec-vectors needs `codec_vectors: true` in `profile.yaml`. `Profile` is `deny_unknown_fields` (`crates/rfcspec-model/src/lib.rs:38-45`), and `grep codec_vectors` over `crates`, `profiles` and `Taskfile.yml` finds nothing. So this story must add a `Profile` field in `crates/rfcspec-model/src/lib.rs`, which its scope omits. websocket-grammar-review edits `profile.yaml` (adds `"4.3"` to `sections`) and the model `lib.rs` (`Overlay.grammars`). Both also run `task docs-generate`, which writes the websocket pages. The remedy is an ordering edge naming `profile.yaml` and the model `lib.rs` as the reason, or putting the flag somewhere that does not touch the two shared files. The model `lib.rs` also has to be added to this story's scope — `.engineering/planning/story/frame-codec-vectors.md:18,27`, `.engineering/planning/story/websocket-grammar-review.md:42,72`

story:text-reader-structure — its scope names no profile files, yet its change to the text reader reaches the locked RFC 6455 and RFC 5905 text (inferred). Both contain what the story changes: `*  %x0 denotes…` bullets at `profiles/websocket/sources/rfc6455.txt:1581`, and an `Appendix A.` heading at `profiles/ntp/sources/rfc5905.txt:3367`. The overlays (`profiles/websocket/overlay.yaml`, `profiles/ntp/overlay.yaml`) key statuses by catalogue id. The generated models and codecs carry source line numbers. send-payload-type-check, which runs beside it in wave 5, has the acceptance "the committed profiles still check clean". That reads catalogue ids this story may renumber, and no edge or sentence orders the two. Either the ids of the four profile RFCs do not move and the story should say so, or it edits the overlays and generated files and must list them. Those files are then shared with wire-scaffold's later neighbours. The remedy is a stated check in the acceptance that `rfcspec check --all profiles` and `rfcspec generate --check` stay clean, plus a scope entry for any file it rewrites, or an ordering edge from send-payload-type-check — `.engineering/planning/story/text-reader-structure.md:31-49`, `.engineering/planning/story/send-payload-type-check.md:20`

Parts 3 and 4 of the rubric:
- **Surfaces:** I placed 11 of the 16 items as cited and 5 as inferred (abnf-core, abnf-bits, abnf-rfc2616, wire-scaffold's new files, status-evidence). None are unplaceable.
- **Acceptance reads:** I traced 15 to a producer in the set, and 14 are recorded by a `depends_on` edge. The unrecorded one is the second finding.

Out of my lane or not established:
- Whether `codec_vectors` could live outside `Profile` is a design question, not mine.
- Where fixed-arm-bounds-check puts its two test overlays is still not stated. If it uses `crates/rfcspec-cli/tests/adversary_timer.rs`, that file belongs to websocket-grammar-review, which runs after it in the chain, so there is no collision.
- Whether catalogue-grammars and diagram-layouts leaving `EXTRACTOR_VERSION` alone is right belongs to the design critic.

```findings
- file: .engineering/planning/story/frame-codec-vectors.md
  line: 27
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "collides with story:websocket-grammar-review on profiles/websocket/profile.yaml (cited, both bodies) and crates/rfcspec-model/src/lib.rs (inferred: the codec_vectors flag needs a Profile field, Profile is deny_unknown_fields at lib.rs:38-45, and grep finds codec_vectors nowhere); no depends_on edge or sentence orders them and waves does not report it; both also run task docs-generate; the remedy is an ordering edge naming profile.yaml and model lib.rs as its reason, or a split surface; model lib.rs must be added to this story's scope"
- file: .engineering/planning/story/text-reader-structure.md
  line: 31
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "inferred: the text-reader change reaches locked RFC 6455 (bullets at rfc6455.txt:1581) and RFC 5905 (Appendix A at rfc5905.txt:3367), whose overlays key statuses by catalogue id and whose generated code carries source line numbers, yet the scope names no profile file; story:send-payload-type-check, in the same wave, accepts 'the committed profiles still check clean' (send-payload-type-check.md:20) against catalogue ids this story may renumber, with no edge; the remedy is a stated check that rfcspec check --all profiles and rfcspec generate --check stay clean plus a scope entry for any file it rewrites, or an ordering edge from send-payload-type-check"
```
