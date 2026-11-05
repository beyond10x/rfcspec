---
format: aep.planning-md/3
id: review-result:wave6-design-round-1
kind: review-result
status: active
title: 'Wave 6 plan critic, design, round 1: approve'
relations:
- reviews: story:abnf-core
- reviews: story:catalogue-grammars
- reviews: story:fixed-arm-bounds-check
- reviews: story:explorer-look
revision: 1
---
`approve`

No findings. I read all four stories, `aep plan artifact relations` and `aep plan artifact graph`, and ran `aep plan artifact validate`. It reports `valid`, with one note on `review-result:explorer-profile-adversary-1`; that note is not mine to report.

**Part 3: what I walked**
- I walked every `depends_on` and `decomposes` edge touching the four stories, in both directions, and followed them outside the set:
  - Out of the set: `story:wire-scaffold`, `story:text-reader-structure`, `story:send-payload-type-check`. All three are `implemented`.
  - Into the set: `story:abnf-bits`, `story:abnf-rfc2616`, `story:diagram-layouts`, `story:websocket-grammar-review`, `story:xml-overlay-check`.
- There are no cycles. The set is four independent items with no edge between any two. The chain `send-payload-type-check → fixed-arm-bounds-check → xml-overlay-check → websocket-grammar-review` exists because those stories share `check.rs`. Its reason is written in the story body, so it is a recorded ordering and not a serialising chain.
- I traced 7 acceptance reads. Each resolves to something already on main or inside the story itself. Zero read state produced by a sibling in this set, so no edge is missing.
  - `abnf-core` reads only `rfc6455.txt` through `rfcspec-doc`.
  - `catalogue-grammars` reads `extract_reporting`, the wire stub, `profiles/websocket/sources/rfc6455.txt`, `bulk/rfc-summary.jsonl` and ESS.
  - `fixed-arm-bounds-check` reads `check.rs` and `profiles/websocket/overlay.yaml`.
  - `explorer-look` reads the built site.

**The `abnf-core` / `catalogue-grammars` split holds.** It cuts the wire layer by capability, with no layer split.
- `abnf-core` is parse, resolve and match. It lives wholly in `crates/rfcspec-abnf` and is tested by its own `tests/rfc6455_core.rs`.
- `catalogue-grammars` is verbatim collection of figures per section. It fills the `crates/rfcspec-wire/src/grammars.rs` stub, adds the `Catalogue` field and fills it in `extract_reporting`.
- Neither calls the other:
  - `grammars::collect` does not use `rfcspec-abnf`, although `crates/rfcspec-wire/Cargo.toml` already links it.
  - `abnf-core` has no dependency on `rfcspec-model`; it stores nothing in the catalogue.
- They share no file. Both use the existing `rfcspec_doc::abnf_rule` helper.
- Their handling of the RFC 6455 section 5.3 pseudo-code figure is consistent: `catalogue-grammars` keeps it as a grammar and `abnf-core` refuses to parse it.
- Downstream edges are one-directional. `abnf-bits` and `abnf-rfc2616` depend on `abnf-core`. `diagram-layouts` and `websocket-grammar-review` depend on `catalogue-grammars`.

**Other surfaces.** `fixed-arm-bounds-check` is confined to `check.rs`, and `explorer-look` to `explorer/src/`.

**Out of my lane, and not setting the verdict**
- Nothing in the store fills `Grammar.uses` yet. `catalogue-grammars` leaves it empty by design. `abnf-core` reports imports only at parse time, using its own `RuleImport` type, which duplicates `rfcspec_model::wire::RuleImport`. The ESS domain's own text says dialect is proposed in S3, which explains the empty field at S2. Whether a later story must bridge the two is a question for the scope critic.
- Parallel-safety and acceptance quality were not assessed.
