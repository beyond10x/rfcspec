---
format: aep.planning-md/3
id: review-result:wire-acceptance-round-1
kind: review-result
status: active
title: Acceptance critic on epic:wire-layer, round 1
relations:
- reviews: story:wire-m1-record
- reviews: story:abnf-core
- reviews: story:handshake-codec
- reviews: story:diagram-layouts
- reviews: story:wire-scaffold
revision: 1
---
needs-revision

- story:wire-m1-record — the acceptance joins the AGENTS.md record with the wave's full gate (`task check` passes on the integration branch), so the record can be right while the gate fails and the story is then neither done nor not done; "every path it names exists" and "the `websocket` task step" also name no command or string to check — .engineering/planning/story/wire-m1-record.md:29
- story:abnf-core — the acceptance never observes the `Dialect` dispatch returning `Unsupported` from `bits.rs` and `rfc2616.rs`, or each comment staying attached to its line, yet story:abnf-bits and story:abnf-rfc2616 take the `Unsupported` return as their "Before" and abnf-bits reads widths from those comments — .engineering/planning/story/abnf-core.md:41
- story:handshake-codec — the acceptance names neither the overlay finding `F-1` on `rfc6455/grammar/4.3` nor the refusal of left-recursive or empty-repetition rules, so both can be missing while every listed command passes — .engineering/planning/story/handshake-codec.md:54
- story:diagram-layouts — the acceptance has no unreadable-diagram case, though the story promises "A diagram the reader cannot read gives no layout and a reason in the notes it returns; it never panics"; "still exits 0" over dns, ntp and time cannot distinguish a reader that notes the refusal from one that silently drops it — .engineering/planning/story/diagram-layouts.md:36
- story:wire-scaffold — the acceptance checks only package names, so the `Dialect` enum, the `collect` stubs and the manifest dependency edges ("declared once so no later story edits a manifest") can be absent and the scaffold still marked done; the same `cargo metadata` output (`.packages[].dependencies`) could show the edges — .engineering/planning/story/wire-scaffold.md:59

What I read: all 10 ids, with `aep plan artifact show` on each plus `aep plan artifact show epic:wire-layer` and `decision-blocker:wire-grammar-layout-pairing`, and `aep plan artifact kinds` and `lifecycle story`. I checked every cited RFC 6455 line and count against `profiles/websocket/sources/rfc6455.txt`, and the figure spans against the reader's output from `rfcspec read`.

Verified correct, so not findings:
- The six grammar sections and their spans (3, 4.2.2, 4.3, 5.2, 5.3, 9.1), including 4.3 as 1384-1387, 1389-1415 and 1420-1423 with the prose at 1416-1419 excluded.
- The 19 rule figures in 5.2 and the 18 distinct rules in 4.3.
- The diagram at 1535-1552 and its eleven fields, with names and widths matching the overlay layout at `profiles/websocket/overlay.yaml` lines 1276-1305.
- Lines 1691, 1749-1761, 1826-1827, 248, 251, 258, 1467 and 1533, `R4.3-1` at 1408-1410, and the `Grammars`, `Layouts`, `ExtractGrammar` and `ExtractLayout` entries in `ess/domains/catalogue.yaml`.
- Every command and task named in the acceptances exists, and `grep -c rfcspec-abnf AGENTS.md` prints 0.

What I could not establish:
- I did not run any `cargo test`, so the "after" outcomes are unrun by design.
- The `Invalid { rule, offset }` semantics, which determine whether `01` as `version` is refused at offset 1, are undefined in story:abnf-core. This is a design matter, outside my lane.
- Whether the printer for `1#version` emits ", ", which "each accepted value prints back to the same bytes" requires for `13, 8, 7`, is not stated in story:handshake-codec. Also outside my lane.
- story:handshake-codec says each accepted value is read from the locked source, but `256` is not text in the RFC; it is derived from the `"2" DIGIT DIGIT` grammar. This is a wording point, not a finding.

```findings
- file: .engineering/planning/story/wire-m1-record.md
  line: 29
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance joins the AGENTS.md record with the wave's full gate (`task check` passes on the integration branch), so the record can be right while the gate fails; 'every path it names exists' and 'the websocket task step' name no command or string to check"
- file: .engineering/planning/story/abnf-core.md
  line: 41
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance never observes the Dialect dispatch returning Unsupported from bits.rs and rfc2616.rs, or comments staying attached to their lines, though story:abnf-bits and story:abnf-rfc2616 take the Unsupported return as their Before and abnf-bits reads widths from those comments"
- file: .engineering/planning/story/handshake-codec.md
  line: 54
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance names neither the overlay finding F-1 on rfc6455/grammar/4.3 nor the refusal of left-recursive or empty-repetition rules, so both can be absent while every listed command passes"
- file: .engineering/planning/story/diagram-layouts.md
  line: 36
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance has no unreadable-diagram case although the story promises 'A diagram the reader cannot read gives no layout and a reason in the notes it returns'; 'still exits 0' over dns, ntp and time cannot tell a noted refusal from a silent drop"
- file: .engineering/planning/story/wire-scaffold.md
  line: 59
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance checks only package names, so the Dialect enum, the collect stubs and the manifest dependency edges ('declared once so no later story edits a manifest') can be absent while the scaffold is marked done"
```
