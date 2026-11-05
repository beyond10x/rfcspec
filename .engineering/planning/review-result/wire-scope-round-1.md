---
format: aep.planning-md/3
id: review-result:wire-scope-round-1
kind: review-result
status: active
title: Scope critic on epic:wire-layer, round 1
relations:
- reviews: epic:wire-layer
- reviews: story:handshake-codec
- reviews: story:diagram-layouts
revision: 1
---
needs-revision

epic:wire-layer — "generated codec with round-trip and mutation vectors" is promised for M1's "6455 handshake grammar and frame bit ABNF + diagram", and story:handshake-codec claims it only for the handshake grammar. The frame side is not covered: the existing frame codec has no round-trip or mutation vectors (no match in `profiles/websocket/generated/rust/src`), and handshake-codec's "Not here" drops the bits-dialect codec without recording that decision against the epic or in a decision-blocker. A story, or a decision-blocker like the pairing one, should take frame-codec vectors, or the narrowing should be recorded on the epic — `docs/design/rfcspec.md:649`; `.engineering/planning/epic/wire-layer.md:13`; `.engineering/planning/story/handshake-codec.md:50`

epic:wire-layer — "layout checks" is promised for `rfcspec-wire` in design §13 and no drafted item claims it. The only related line is diagram-layouts' "Not here: comparing a catalogue layout with an overlay layout in `rfcspec check`", which is narrower than "layout checks". Its test only asserts equality with `frame-header`. The `rfcspec-wire/1` model half of the same row is deferred by decision-blocker:wire-grammar-layout-pairing, but the layout checks are not named there. The natural owner is story:diagram-layouts, or a line on the blocker — `docs/design/rfcspec.md:634`; `.engineering/planning/story/diagram-layouts.md:32`; `.engineering/planning/story/diagram-layouts.md:26`

I found no reach beyond the parent and no two items claiming one outcome.

**What I read:** 11 drafted artifacts plus `epic:wire-layer`. I ran `aep plan artifact list`, `show` on each id and `graph`, and read `docs/design/rfcspec.md` §8, §13, §14 and `story:websocket-frame-codec`. I extracted 10 promises from the epic body, §13 and M1. I traced 8 to an item (ABNF parse, parser and printer, diagram to layout, text and RFCXML, handshake grammar, frame bit ABNF plus diagram, agreement check, §13 dialects and import resolution). Two are only partly covered: the codec vectors and the layout checks above.

**What I could not establish:**
- The operator quote "into ESS specs directly" reads as direction, not a promise, so I did not require a lowering item. The `rfcspec-wire/1` sidecar is deferred by the blocker.
- Ambiguity and determinism analysis (§8.2, §13) and incremental `More(state)` parsing are named under "Not here" in abnf-core and handshake-codec. I treated them as honest omissions, but they point to no later home.
- Whether blocking the whole epic on the pairing decision is right is out of my lane (design critic).
- The overlapping files between abnf-core, abnf-bits and abnf-rfc2616 are out of my lane (parallel-safety critic).

```findings
- file: .engineering/planning/epic/wire-layer.md
  line: 13
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "\"generated codec with round-trip and mutation vectors\" is promised for the handshake grammar and the frame bit ABNF + diagram (docs/design/rfcspec.md:649), but story:handshake-codec claims vectors only for the handshake grammar and excludes a bits-dialect codec in a Not-here line (handshake-codec.md:50) with no story or decision-blocker recording the narrowing; the existing frame codec has no round-trip or mutation vectors"
- file: docs/design/rfcspec.md
  line: 634
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "design section 13 promises \"layout checks\" for rfcspec-wire and no drafted item claims them; diagram-layouts excludes only comparing a catalogue layout with an overlay layout in rfcspec check, and the blocker defers the rfcspec-wire/1 model but not layout checks; story:diagram-layouts would most naturally take it"
```
