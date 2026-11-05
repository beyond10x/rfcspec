---
format: aep.planning-md/3
id: review-result:wire-design-round-1
kind: review-result
status: active
title: Design critic on epic:wire-layer, round 1
relations:
- reviews: decision-blocker:wire-grammar-layout-pairing
- reviews: story:frame-agreement
- reviews: epic:wire-layer
revision: 1
---
needs-revision

decision-blocker:wire-grammar-layout-pairing — its `blocks` edge targets `epic:wire-layer`, so `aep plan artifact blocked` reports the whole epic blocked. The body says "Nothing in wave 6 persists the pair", and every drafted story is built not to rest on the answer. What waits on the decision is later work (the `rfcspec-wire/1` sidecar, the agreement check without arguments, the ESS declaration for the agreement result). No artifact for that work exists yet. The drafter has two choices. Re-point `blocks` at an item that does wait on the decision. Or drop `blocks` and record the shaping as `informed_by` from story:frame-agreement, the one story that takes the pairing on its command line. The two earlier blockers of this kind blocked epics with "No story is drafted for it" (`decision-blocker:catalog-multi-transport-semantics`). — `aep plan artifact blocked`; `aep plan artifact graph` (`decision-blocker:wire-grammar-layout-pairing -> epic:wire-layer [blocks]`); `.engineering/planning/decision-blocker/wire-grammar-layout-pairing.md` § "What is drafted without the answer" and § "What waits for it"

**What you read.** I read 11 artifacts whole with `aep plan artifact show`. I also ran `aep plan artifact relations`, `graph`, `validate`, `waves` and `blocked`, and read `profiles/websocket/profile.yaml`, `ess/domains/catalogue.yaml` (Grammar, Layout, Rfc relations), design §13–14, and the extract and docs consumers of the catalogue.

- **Edges.** I walked the 26 declared edges that touch the set, 10 `depends_on` and the rest `decomposes`, `informed_by` and `blocks`. I read the whole store graph of 153 artifacts, so I also looked outside the set. The graph is acyclic, with a depth of five (scaffold, then catalogue-grammars, then websocket-grammar-review, then frame-agreement, then wire-m1-record). It is not a queue: abnf-core, catalogue-grammars and diagram-layouts are one fan-out, and frame-agreement and handshake-codec are another.
- **Acceptance reads.** I traced about 20 acceptance reads to producers in the set. Each has a `depends_on` path, direct or transitive. The transitive cases are:
  - frame-agreement reading the catalogue grammar through websocket-grammar-review;
  - handshake-codec reading the catalogue grammar through websocket-grammar-review;
  - wire-m1-record reading everything through frame-agreement and handshake-codec.
- **Validate.** `aep plan artifact validate` reports one unrelated item: `review-result:explorer-profile-adversary-1` has no findings block. It is not mine.
- **Waves.** `aep plan artifact waves` places the wire stories in order. `waves` also lists many collisions with unrelated draft stories, which belong to parallel-safety.

**What I could not establish.**
- Two dialect types are declared, `rfcspec-model::AbnfDialect` and `rfcspec-abnf::Dialect`. `rfcspec-abnf` has no dependency on `rfcspec-model`, and no story owns the mapping that frame-agreement and handshake-codec both need. This is an unease, not a finding: each can write a trivial match.
- `Grammar.uses` is left empty by story:catalogue-grammars, although abnf-core computes imports and `rfcspec-extract` reaches `rfcspec-abnf` through `rfcspec-wire`. It is stated as deliberate, so I do not count it. Scope may want to check it.
- Whether the `blocks` edge actually stops waves or story moves, as opposed to only showing in `blocked`, is not something I tested.

```findings
- file: aep plan artifact blocked
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the open blocker's `blocks` edge targets epic:wire-layer, so the whole epic reports blocked, while its body says no wave-6 story rests on the answer and what waits (sidecar binding, argument-free agreement, ESS declaration) is later work with no artifact; re-point `blocks` at an item that waits, or drop it and record the shaping as `informed_by` from story:frame-agreement"
```
