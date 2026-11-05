---
format: aep.planning-md/3
id: review-result:wire-scope-round-2
kind: review-result
status: active
title: Scope critic on epic:wire-layer, round 2
relations:
- reviews: epic:wire-layer
- reviews: story:frame-codec-vectors
revision: 1
---
needs-revision

story:frame-codec-vectors — reach beyond the parent: the body says "Every multi-field layout gets them, so the generated `rfc1035-dns` and `rfc5905-ntp` crates change too", and its scope and acceptance rewrite the DNS and NTP generated crates and site downloads. M1 asks for the codec vectors only for "6455 handshake grammar and frame bit ABNF + diagram", and neither the epic body nor §13/§14 mentions DNS or NTP. The body should limit the vectors to the websocket profile, or record the extension to other profiles as a decision, which is not recorded today — `.engineering/planning/story/frame-codec-vectors.md:36`; `docs/design/rfcspec.md:649`

Round 1 finding 1 is resolved. `story:frame-codec-vectors` takes the frame-side "round-trip and mutation vectors" with its own acceptance. `story:handshake-codec` and `story:wire-m1-record` depend on it, and `handshake-codec` and `frame-codec-vectors` both record the bits-dialect codec under "Not here".

Round 1 finding 2 is resolved. `story:diagram-layouts` now carries a "Layout checks" section in the new file `crates/rfcspec-wire/src/layout.rs`. It has five named checks and a per-check test under `cargo test -p rfcspec-wire layout`.

Nothing else is new. I found no gap and no two items claiming one outcome. The `rfcspec-wire/1` sidecar and grammar/layout persistence are deferred in the cleared blocker, which records them as later work with no story yet.

What I read: the epic, its design §8, §11, §13 and §14 and `review-result:wire-scope-round-1`, plus 11 stories and the blocker whole. I ran `aep plan artifact list`, `show` on each id, and `graph`. I grepped the generated websocket and DNS crates for `mutation`, `roundtrip` and `round_trip`, and found no match. I extracted 10 promises from the parent and traced 10 to an item. The two partial ones from round 1 are now covered.

What I could not establish:
- The epic body is unchanged at revision 1 and still says "Draft, not decomposed", while the round-1 outcomes say "fixed on epic:wire-layer". I could not tell whether the epic was meant to change; it is not a coverage defect.
- Incremental `More(state)` parsing and ambiguity analysis (§8.2) are named in "Not here" in `abnf-core` and `handshake-codec`. They still point to no later home. I treated this as an honest omission, as in round 1.
- Whether the frame mutation vectors (bit flips) match design §11's "single-byte mutations" is an acceptance question, out of my lane.

```findings
- file: .engineering/planning/story/frame-codec-vectors.md
  line: 36
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the body extends the round-trip and mutation vectors to every multi-field layout and rewrites the generated rfc1035-dns and rfc5905-ntp crates and their downloads, but milestone M1 (docs/design/rfcspec.md:649) asks for them only for the 6455 frame and handshake, and no sentence of the epic or design reaches DNS or NTP; limit the story to the websocket profile or record the extension as a decision"
```
