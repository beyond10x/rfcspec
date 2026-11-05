---
format: aep.planning-md/3
id: review-result:websocket-scope-round-1
kind: review-result
status: active
title: scope critic, round 1, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
needs-revision

story:websocket-close-machine — the epic promises "one layer, `websocket` over `tcp` (messages, ordered, reliable, full-duplex, single, session, correlation none, lifetime connection)" and "the `websocket` semantics row", but the story declares a layer named `tcp` with unit `bytes` and `over` omitted, and nothing in the body records that this departs from the epic — .engineering/planning/epic/websocket-profile.md:32-33 and :47-48; story:websocket-close-machine "Acceptance", first bullet

The story gives reasons for omitting `over`: `rfcspec check` refuses an `over.profile` that names no profile directory. It also cites the cleared decision-blocker that makes each transport its own layer. Neither covers the changed layer name or the changed unit. Design §6.2 (docs/design/rfcspec.md:265) also says `bytes`, so the epic's "messages" and the design disagree.

The body needs one of two edits:
- State that the semantics row is `tcp` with unit `bytes`, and why this replaces the epic's `websocket` over `tcp` with unit `messages`.
- Mark the epic's wording as superseded so the operator sees the decision.

`story:websocket-review-matrix` then shows "the `tcp` semantics row" against the epic's "`websocket` semantics row" (epic :47-48). It needs no edit of its own once the close-machine story records the decision.

**Counts**
- The parent holds 11 promises: 6 under "In this wave", 4 under "Outcome", and the final Acceptance paragraph.
- Ten trace to an item.
  - Source and reader: `story:websocket-source-catalogue`.
  - Extraction with section and span: `story:websocket-source-catalogue`.
  - Overlay review of §5.2, §5.5 and §7, with the rest left unreviewed and shown as such: `story:websocket-review-matrix`.
  - Closing handshake, explore and G9: `story:websocket-close-machine`.
  - Layout and the §5.7 codec: `story:websocket-frame-codec`.
  - Index and page, including matrix, gaps and downloads: `story:websocket-source-catalogue`, `story:websocket-close-machine` and `story:websocket-review-matrix`.
  - The design §12 worked example becoming a real profile: `story:websocket-close-machine`.
  - The first BCP 14 RFC, exercising the mechanical extraction path: `story:websocket-source-catalogue`.
  - The pipeline-and-page outcome: all four.
  - The final Acceptance paragraph: `story:websocket-review-matrix`, which closes it.
- The "one layer, `websocket` over `tcp`" semantics promise is claimed but narrowed silently, as in the finding above.

**What I checked on the other three defects**
- **Gaps:** none beyond the narrowed semantics promise.
- **Reach beyond the parent:** none. The generalised machine model, the octet-example extraction, the catalogue-id rule and the unreviewed counts all serve a parent promise. The epic's exclusions (HTTP/1.1 parsing, RFC 7692, RFC 8441, RFC 9220, a verify probe) are untouched.
- **Duplicate claims:** none. The overlay, `Taskfile.yml` and generated pages are edited by several stories, but each story claims a different outcome, and the stories are serialised by `depends_on`.

**What I read:** 5 artifacts (the epic and the 4 stories), via `aep plan artifact show` on each and `aep plan artifact graph`. I also read `.engineering/planning/epic/websocket-profile.md` with line numbers, `docs/design/rfcspec.md` lines 265 and 316, and `decision-blocker:catalog-multi-transport-semantics`.

**What I could not establish**
- I did not check the counts (214, 65, 29, 149) against RFC 6455. That is the acceptance critic's lane, and the figures are consistent among themselves: 11 + 21 + 33 = 65 and 214 − 65 = 149.
- Out of my lane: the `AGENTS.md` State edit in `story:websocket-review-matrix` is not traceable to a parent sentence. It looks like repository convention and I did not flag it.

```findings
- file: .engineering/planning/epic/websocket-profile.md
  line: 32
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the epic promises one layer 'websocket' over 'tcp' with unit messages and a 'websocket' semantics row (lines 32-33, 47-48), but story:websocket-close-machine declares a layer named 'tcp' with unit bytes and no over, and its body does not record the departure from the epic"
```
