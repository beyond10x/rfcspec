---
format: aep.planning-md/3
id: review-result:wire-acceptance-round-2
kind: review-result
status: active
title: Acceptance critic on epic:wire-layer and the backlog, round 2
relations:
- reviews: story:wire-m1-record
- reviews: story:frame-agreement
- reviews: story:text-reader-structure
- reviews: story:status-evidence
- reviews: story:send-payload-type-check
- reviews: story:fixed-arm-bounds-check
revision: 1
---
needs-revision

- story:wire-m1-record — the last acceptance bullet cannot pass: after the edit "4.3, 5.2, 5.4, 5.5, 5.6 and 7" still contains the substring " 5.2, 5.4, 5.5, 5.6 and 7" (space before 5.2), so `grep -c " 5.2, 5.4, 5.5, 5.6 and 7" AGENTS.md` prints 2, not the required 0; a grep that distinguishes old from new text must anchor on what precedes it, for example `sections 5.2, 5.4` — .engineering/planning/story/wire-m1-record.md:43
- story:frame-agreement — the disagreement is to name "line 1691 and the diagram line", but no diagram line number is given and `CatalogueField` carries no per-field span (`{ name, bits, presence, constraints }`; the layout holds one span, 1535-1552), so the test has no value to assert for the second line — .engineering/planning/story/frame-agreement.md:45
- story:text-reader-structure — the acceptance observes only the appendix cause (RFC 9644), so the list/aside and heading fixes can be absent while it passes; "the comparison over the same 158 RFCs reports fewer differing RFCs, each remaining one classified" names no command and no threshold (25 to 24 passes); and the "`EXTRACTOR_VERSION` moves and the committed bulk summary is regenerated" outcome has no check such as `bulk check` — .engineering/planning/story/text-reader-structure.md:47
- story:status-evidence — "The profile gate refuses a satisfied status without evidence" and "The profile pages and the explorer's reviewed layer show the evidence" name no command, test or output string, and "names an existing test, scenario or witness" names nothing that checks existence; the open question on what counts as evidence for codec-layout requirements leaves the first bullet undecidable for websocket rows — .engineering/planning/story/status-evidence.md:52
- story:send-payload-type-check — the acceptance joins the unit test with "the committed profiles still check clean; `task check` exits 0", so the test can pass while the full gate fails, and the full gate runs once at wave close for the whole `check.rs` chain, so it cannot be attributed to this story; the same joined clause is in story:fixed-arm-bounds-check, whose "refused ... with the named message" also gives no message text — .engineering/planning/story/send-payload-type-check.md:20
- story:fixed-arm-bounds-check — the acceptance gives no message text ("the named message"; the Outcome says only "naming machine, participant, transition and timer"), and ends with "`task check` exits 0", a wave-close gate that cannot be attributed to this story — .engineering/planning/story/fixed-arm-bounds-check.md:22

Round 1 status, each checked against the current bodies:
- Resolved: story:abnf-core now observes the `Unsupported` dispatch from `bits.rs` and `rfc2616.rs`, and the comments on `frame-fin` lines 1701-1703.
- Resolved: story:handshake-codec names `F-1` on `rfc6455/grammar/4.3` and the two refusals (`a = a "x"`, `b = *( [ "x" ] )`).
- Resolved: story:diagram-layouts has the one-bit-narrower `opcode` case (`31 of 32 bits`, line 1535).
- Resolved: story:wire-scaffold's acceptance now lists the dependency edges and the `Dialect` and `stubs` tests.
- Partly resolved: story:wire-m1-record split off the gate and now uses grep counts, but the final grep is unsatisfiable (first finding).

What I read: all 16 ids (the 11 wire-layer stories, the 5 backlog stories) with `aep plan artifact show`, plus `review-result:wire-acceptance-round-1`, `epic:wire-layer` and `aep plan artifact kinds`. I checked the cited RFC 6455 lines in `profiles/websocket/sources/rfc6455.txt`, the five grep counts against `AGENTS.md`, and the 46 satisfied statuses in `profiles/*/overlay.yaml` (4+29+7+6). I also ran `git diff` on the five backlog stories: only `Order`, `Scope` and relations changed, so their acceptance defects are pre-existing.

What I could not establish:
- Whether `ess specify validate --path ess` observes the two new `ess/scenarios` files in story:catalogue-grammars and story:diagram-layouts, since it passes with or without them. I did not run it.
- Which line the diagram-layouts "31 of 32 bits" note names, 1535 (the ruler) or the drawn row 1537. The story says "first source line", so I took 1535.
- I ran no `cargo test`.
- Out of my lane, not counted:
  - The `Invalid { rule, offset }` semantics for `01` at offset 1 are still undefined in story:abnf-core (round 1 also noted this).
  - Story:frame-agreement needs the layout to carry a per-field line; that is a design matter.

```findings
- file: .engineering/planning/story/wire-m1-record.md
  line: 43
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the last bullet cannot pass: after the edit '4.3, 5.2, 5.4, 5.5, 5.6 and 7' still contains the substring ' 5.2, 5.4, 5.5, 5.6 and 7', so the second grep prints 2, not 0; the grep must anchor on what precedes 5.2"
- file: .engineering/planning/story/frame-agreement.md
  line: 45
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the disagreement is to name 'line 1691 and the diagram line' but no diagram line number is given and CatalogueField carries no per-field span, so the test has no value to assert for the second line"
- file: .engineering/planning/story/text-reader-structure.md
  line: 47
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: pre-existing
  message: "the acceptance observes only the appendix cause (RFC 9644), so the list/aside and heading fixes can be absent while it passes; 'the comparison ... reports fewer differing RFCs, each remaining one classified' names no command and no threshold; the EXTRACTOR_VERSION move and regenerated bulk summary have no check"
- file: .engineering/planning/story/status-evidence.md
  line: 52
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: pre-existing
  message: "'The profile gate refuses a satisfied status without evidence' and 'The profile pages and the explorer's reviewed layer show the evidence' name no command, test or output string, and 'names an existing test, scenario or witness' names no check of existence; the open question on codec-layout evidence leaves the first bullet undecidable for websocket rows"
- file: .engineering/planning/story/send-payload-type-check.md
  line: 20
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: pre-existing
  message: "the acceptance joins the unit test with 'the committed profiles still check clean; task check exits 0', so the test can pass while the full gate fails, and the full gate runs once at wave close for the whole check.rs chain, so it cannot be attributed to this story"
- file: .engineering/planning/story/fixed-arm-bounds-check.md
  line: 22
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: pre-existing
  message: "'refused by rfcspec check with the named message' gives no message text (the Outcome says only 'naming machine, participant, transition and timer'), and 'task check exits 0' is a wave-close gate that cannot be attributed to this story"
```
