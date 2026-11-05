---
format: aep.planning-md/3
id: review-result:hardening-acceptance-round-1
kind: review-result
status: active
title: acceptance critic, round 1, hardening follow-ups
relations:
- reviews: epic:hardening-follow-ups
- reviews: story:overlay-type-checks
- reviews: story:proposal-level-check
- reviews: story:channel-capacity
revision: 1
---
needs-revision

- story:channel-capacity — the acceptance for "its client sends text, start, Pong, continue, finish and close in one run" names no check that tells the old three-path model from the new one, because `"verdict": "passed"` is already printed for the current fragments model and the witness holds only a `close` input — `.engineering/planning/story/channel-capacity.md:35`; see `crates/rfcspec-ess/src/lib.rs:899` and `profiles/websocket/witnesses/fragments.inputs.json:2`.

Ids read: epic:hardening-follow-ups, story:overlay-type-checks and story:proposal-level-check were fine. In story:channel-capacity the other bullets, including the capacity-8 test on both channels and `lower --check` with no byte changed, are observable and show a change.

Minor point in the same story, not a separate finding: the `rfcspec check` refusal of capacity 0 (line 33) gives no test or message to check it against.

What I read: 4 of 4 ids, with `aep plan artifact show` on each. I also read `Taskfile.yml`, `crates/rfcspec-extract/src/check.rs` around line 168, `crates/rfcspec-propose/src/lib.rs` around line 611, `profiles/websocket/overlay.yaml`, the fragments witness, and the ess mutant test at `lib.rs:899`.

What I could not establish: whether the ten recorded proposals replay unchanged after the level check, because I did not run `rfcspec propose --replay`. Not mine: `profiles/websocket/witnesses/fragments.inputs.json` is not in story:channel-capacity's scope list (scope lane).

```findings
- file: .engineering/planning/story/channel-capacity.md
  line: 35
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance that the fragments client sends text, start, Pong, continue, finish and close in one run names no check that differs between the old three-path model and the new one; explore already prints passed on the current model and the witness (profiles/websocket/witnesses/fragments.inputs.json) holds only a close input, so it reads the same before and after the work"
```
