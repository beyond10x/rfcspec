---
format: aep.planning-md/3
id: review-result:wire-design-round-2
kind: review-result
status: active
title: Design critic on epic:wire-layer, round 2
relations:
- reviews: epic:wire-layer
- reviews: decision-blocker:wire-grammar-layout-pairing
- reviews: story:frame-agreement
revision: 1
---
approve

**What you read.** I read 17 artifacts whole: the 12 in the set, `epic:wire-layer`, `review-result:wire-design-round-1` and the three backlog stories. I ran `aep plan artifact show` on each, plus `relations`, `graph`, `validate`, `blocked` and `waves`.

- **Round-1 finding: resolved.**
  - `decision-blocker:wire-grammar-layout-pairing` is `cleared` and has no `blocks` edge any more. `graph` shows only three `blocks` edges, targeting `epic:public-protocol-catalog` (twice) and `epic:rfc-explorer`, none touching this set.
  - `aep plan artifact blocked` prints "nothing is blocked".
  - The shaping is now `story:frame-agreement -> decision-blocker:wire-grammar-layout-pairing [informed_by]`, which is the second remedy round 1 offered.
- **Edges.**
  - I walked all 39 edges leaving the set's stories and read the whole store graph, so I also looked outside the set. It is acyclic. The longest path is send-payload-type-check, fixed-arm-bounds-check, xml-overlay-check, websocket-grammar-review, frame-agreement, wire-m1-record.
  - The set is not a queue: abnf-bits, abnf-rfc2616, catalogue-grammars and the frame-codec-vectors branch fan out.
  - The `check.rs` chain of three backlog stories carries its shared-file reason in each body's `## Order` section, so it is the shared-file trade-off, not an unexplained chain. The alternative to the ordering would be splitting `crates/rfcspec-extract/src/check.rs`.
- **Acceptance reads.** I traced about 24 reads of state a sibling produces. All have a `depends_on` path to the producer, 14 of them direct and the rest transitive.
  - Transitive: frame-agreement and handshake-codec reading the catalogue grammar through websocket-grammar-review, handshake-codec reading abnf-core through abnf-rfc2616, and wire-m1-record reading `Section 4.3 in scope` through handshake-codec.
  - I traced these reads:
    - the `Overlay.grammars` and `Catalogue.grammars/layouts` struct fields
    - the widened finding items
    - the `rfc6455/layout/5.2-1` and `grammar/4.3` and `5.2` ids
    - the `Dialect` stubs
    - the `handshake` module
    - `F-1`
    - the `mutation_` tests
    - the `wire agree` command and its Taskfile step
- **Shared files.** Every file shared between stories in the set is ordered by an edge. The files are `crates/rfcspec-extract/src/lib.rs`, `crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-wire/src/lib.rs`, `crates/rfcspec-gen/src/lib.rs`, `crates/rfcspec-gen/src/fields.rs`, the generated websocket `lib.rs`, and `profiles/websocket/overlay.yaml`.
- **Validate.** It reports only `review-result:explorer-profile-adversary-1` with no findings block, which is not mine.

**What I could not establish.**
- The `Dialect` enum in `rfcspec-abnf` and `AbnfDialect` in `rfcspec-model` are still two types. No story says which side maps one to the other for frame-agreement and handshake-codec. `Dialect::from_wire` makes a trivial match possible, so I count it as unease. This is the same as round 1.
- abnf-core's body does not state the return type of the `Bits` and `Rfc2616` entry points, and abnf-bits must carry per-rule and per-element widths in it. I could not tell whether that is a split abstraction or a free choice for abnf-core.
- `story:frame-codec-vectors` depends on `story:diagram-layouts`, but its acceptance reads nothing that story produces. It only adds ordering and no extra depth, because handshake-codec already depends on diagram-layouts through websocket-grammar-review.
- `story:status-evidence` may cite the `wire agree` test as evidence for the frame layout, and it has no edge to frame-agreement. Its open question names only codec round-trip tests, so I cannot say.
- Out of my lane:
  - `aep plan artifact waves` lists many `collision:` lines for set members, including ones already ordered by an edge (abnf-core with abnf-bits, abnf-core with abnf-rfc2616). That belongs to parallel safety.
  - The `epic:wire-layer` body still says "Draft, not decomposed". That is wording, not shape.

```findings
[]
```
