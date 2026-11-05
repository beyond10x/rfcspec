---
format: aep.planning-md/3
id: review-result:wire-parallel-round-1
kind: review-result
status: active
title: Parallel-safety critic on epic:wire-layer and the backlog, round 1
relations:
- reviews: story:fixed-arm-bounds-check
- reviews: story:send-payload-type-check
- reviews: story:xml-overlay-check
- reviews: story:wire-scaffold
- reviews: story:websocket-grammar-review
- reviews: story:status-evidence
- reviews: story:wire-m1-record
revision: 1
---
needs-revision

Seven findings, all collisions the plan does not name. The `depends_on` edges inside the wire set hold up; the gaps are the shared files with the five non-wire stories.

`aep plan artifact waves --status draft` does order these items into eight waves. It does so from the scope collisions alone, and no body or edge records why. If the work is dispatched as two waves instead of those eight, items 1 to 4 and 6 run concurrently.

1. story:fixed-arm-bounds-check — it shares `crates/rfcspec-extract/src/check.rs` (cited, all bodies) with send-payload-type-check, xml-overlay-check, wire-scaffold (the `Overlay` literal at 731) and websocket-grammar-review (cited), and with status-evidence (inferred); none of these bodies says so and no `depends_on` edge records it, so the remedy is an ordering edge naming `check.rs` as its reason, or splitting the surface — `.engineering/planning/story/fixed-arm-bounds-check.md:9`
2. story:send-payload-type-check — its unit test and fix sit in `check.rs`, shared (cited) with fixed-arm-bounds-check, xml-overlay-check, wire-scaffold and websocket-grammar-review and (inferred) with status-evidence, none admitted; the remedy is an ordering edge or a split surface — `.engineering/planning/story/send-payload-type-check.md:11,20`
3. story:xml-overlay-check — it edits `check.rs` at lines 24, 358 and 655 (cited) while fixed-arm-bounds-check, send-payload-type-check, wire-scaffold and websocket-grammar-review edit the same file, and its body names none of them; the remedy is an ordering edge or a split surface — `.engineering/planning/story/xml-overlay-check.md:13,20-29`
4. story:wire-scaffold — the edge `depends_on story:catalogue-grammars` ordering websocket-grammar-review after it is only transitive, so `check.rs` (the `Overlay` literal at 731) is unordered against the three check.rs hardening stories and status-evidence (inferred), and `crates/rfcspec-extract/src/lib.rs` (`extract_reporting` at 146, the `Catalogue` literal at 277) is shared with text-reader-structure's `EXTRACTOR_VERSION` edit (cited, both bodies), with no edge or mention either way; the remedy is an ordering edge or a split surface — `.engineering/planning/story/wire-scaffold.md:25,31,63`
5. story:websocket-grammar-review — it adds checks to `check.rs` (cited) beside fixed-arm-bounds-check, send-payload-type-check and xml-overlay-check (cited) and status-evidence (inferred), and `profiles/websocket/overlay.yaml` is also edited by status-evidence (inferred); no edge or mention covers any of these; the remedy is an ordering edge or a split surface — `.engineering/planning/story/websocket-grammar-review.md:18,20,37`
6. story:status-evidence — its scope is inferred and names two whole directories, `website/docs/catalog` and `website/static/data/profiles`. Those contain `website/docs/catalog/websocket.md` and `website/static/data/profiles/websocket.json`, which websocket-grammar-review and handshake-codec regenerate (cited), and `website/docs/catalog/rfcs`, which text-reader-structure regenerates. The `waves` output lists no such collision, because it compares file to file. The story also shares `overlay.yaml` and `check.rs` with those items, and its scope is too wide to place without narrowing; the remedy is an ordering edge, or naming the generated files it actually rewrites — `.engineering/planning/story/status-evidence.md:15-29`
7. story:wire-m1-record — its acceptance reads "`task check` passes on the integration branch of the wave", which holds the work of fixed-arm-bounds-check, send-payload-type-check, xml-overlay-check, status-evidence and text-reader-structure, but its only edges run to frame-agreement and handshake-codec; the missing edges are `depends_on` from this story to those five, or merging the stories — `.engineering/planning/story/wire-m1-record.md:8-9,30`

**What I read:** 15 artifacts via `aep plan artifact show`, plus `epic:wire-layer`, `decision-blocker:wire-grammar-layout-pairing`, `aep plan artifact waves --status draft` and `aep plan artifact graph`. I grepped the tree at the wave-plan worktree for `EXTRACTOR_VERSION`, the `bulk check` code in `crates/rfcspec-cli/src/bulk.rs`, `Taskfile.yml` and the docs generators.

Surfaces placed: 9 cited, 6 inferred (abnf-core, abnf-bits, abnf-rfc2616, catalogue-grammars, diagram-layouts and status-evidence, which are mostly new files), 0 unplaceable.

Acceptance reads traced to a producer in the set: 14, of which 13 are recorded by a `depends_on` edge. The unrecorded one is the gate in wire-m1-record (finding 7).

**What I could not establish:**
- Where fixed-arm-bounds-check puts its two test overlays. If they go in `crates/rfcspec-cli/tests/adversary_timer.rs`, wire-scaffold also edits that file (the `Overlay` literal at 126).
- Whether catalogue-grammars and diagram-layouts, which change `rfcspec extract` output, also need an `EXTRACTOR_VERSION` bump. That belongs to the design critic, not this one.
- xml-overlay-check un-ignores tests in `crates/rfcspec-extract/tests/adversary_bcp14_xml.rs`, but its scope omits that file. It collides with nothing in this set, so it is not a finding; the scope critic may want it.
- Whether the docs generators' output for `websocket.md` and `websocket.json` changes with catalogue-grammars or diagram-layouts. I found no rendering of the new catalogue fields, so I raised no collision.

```findings
- file: .engineering/planning/story/fixed-arm-bounds-check.md
  line: 9
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it shares crates/rfcspec-extract/src/check.rs (cited) with send-payload-type-check, xml-overlay-check, wire-scaffold and websocket-grammar-review, and (inferred) with status-evidence; none of these bodies says so and no depends_on edge records it, so the remedy is an ordering edge naming check.rs as its reason, or splitting the surface"
- file: .engineering/planning/story/send-payload-type-check.md
  line: 11
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its fix and unit test sit in crates/rfcspec-extract/src/check.rs, shared (cited) with fixed-arm-bounds-check, xml-overlay-check, wire-scaffold and websocket-grammar-review and (inferred) with status-evidence, none admitted; the remedy is an ordering edge or a split surface"
- file: .engineering/planning/story/xml-overlay-check.md
  line: 13
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it edits check.rs at lines 24, 358 and 655 (cited) while fixed-arm-bounds-check, send-payload-type-check, wire-scaffold and websocket-grammar-review edit the same file and its body names none of them; the remedy is an ordering edge or a split surface"
- file: .engineering/planning/story/wire-scaffold.md
  line: 25
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "check.rs (Overlay literal at 731) is unordered against fixed-arm-bounds-check, send-payload-type-check, xml-overlay-check and (inferred) status-evidence, and crates/rfcspec-extract/src/lib.rs (extract_reporting at 146, Catalogue literal at 277) is shared with text-reader-structure's EXTRACTOR_VERSION edit (cited, both bodies); no edge or mention covers either; the remedy is an ordering edge or a split surface"
- file: .engineering/planning/story/websocket-grammar-review.md
  line: 18
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it adds checks to crates/rfcspec-extract/src/check.rs (cited) beside fixed-arm-bounds-check, send-payload-type-check and xml-overlay-check, and (inferred) status-evidence, which also edits profiles/websocket/overlay.yaml; no edge or mention covers these; the remedy is an ordering edge or a split surface"
- file: .engineering/planning/story/status-evidence.md
  line: 27
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its inferred scope names the directories website/docs/catalog and website/static/data/profiles, which contain websocket.md and websocket.json regenerated by websocket-grammar-review and handshake-codec and the rfcs pages regenerated by text-reader-structure, a collision the waves command cannot see; the remedy is an ordering edge or naming the generated files it actually rewrites"
- file: .engineering/planning/story/wire-m1-record.md
  line: 30
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its acceptance reads task check passing on the wave's integration branch, which holds fixed-arm-bounds-check, send-payload-type-check, xml-overlay-check, status-evidence and text-reader-structure, yet its only depends_on edges run to frame-agreement and handshake-codec; the missing edges run from this story to those five, or merge the stories"
```
