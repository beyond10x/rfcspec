---
format: aep.planning-md/3
id: review-result:websocket-scope-round-2
kind: review-result
status: active
title: scope critic, round 2, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
approve

**What I read:** 6 artifacts. I ran `aep plan artifact show` on the epic, the four stories and `review-result:websocket-scope-round-1`, plus `aep plan artifact graph`. I read the epic with line numbers from `.engineering/planning/epic/websocket-profile.md` (revision 2).

**Round 1 finding:** it no longer holds. Revision 2 of the epic now says the semantics row is one layer named `tcp` with unit `bytes` and no `over`, citing `decision-blocker:catalog-multi-transport-semantics` and design §6.2 (epic lines 32-35, acceptance line 50). `story:websocket-close-machine` (first acceptance bullet) and `story:websocket-review-matrix` (page bullet) match that wording.

**Promises (10 extracted, 10 traced):**
- Source locked, reader handles furniture, headings and figures: `story:websocket-source-catalogue`.
- Every BCP 14 requirement extracted with section and span: `story:websocket-source-catalogue`.
- Overlay reviews §5.2, §5.5 and §7, and the rest is unreviewed and shown as such: `story:websocket-review-matrix`.
- Closing handshake lowered, `ess protocol explore` passes, G9 recorded: `story:websocket-close-machine`.
- §5.2 layout and the codec tested with the §5.7 examples: `story:websocket-frame-codec`.
- The `tcp` semantics row: `story:websocket-close-machine`.
- Index lists `time` and `websocket`, and the page shows matrix, gaps and downloads: all four stories, each claiming a different increment.
- RFC 6455 as the first BCP 14 RFC, exercising the mechanical extraction path: `story:websocket-source-catalogue`.
- The design §12 worked example becomes a real profile: `story:websocket-close-machine`.
- The final Acceptance paragraph: `story:websocket-review-matrix`, which closes it.

**Checks on the other defects:**
- **Gaps and narrowing:** none. Nothing is dropped silently. The epic's "or leaves them unreviewed" is claimed as the unreviewed count in `story:websocket-review-matrix`.
- **Reach beyond the parent:** none. The generalised machine model, octet-example extraction, catalogue-id rule, layout generalisation and unreviewed counts each serve a parent promise. None touches the epic's exclusions (HTTP/1.1 parsing, RFC 7692, RFC 8441/9220, live probe).
- **Duplicate claims:** none. The overlay, `Taskfile.yml` and generated pages are edited by several stories, but each claims a different outcome and `depends_on` serialises them.

**What I could not establish:**
- I did not check the RFC counts (214, 65, 29, 149, the line numbers) against RFC 6455. That is `plan-critic-acceptance`'s lane.
- Out of my lane: the `AGENTS.md` State edit in `story:websocket-review-matrix` has no parent sentence. I treat it as repository convention and it does not set my verdict.

```findings
[]
```
