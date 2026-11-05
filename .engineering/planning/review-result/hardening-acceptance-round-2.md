---
format: aep.planning-md/3
id: review-result:hardening-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, round 2, hardening follow-ups
relations:
- reviews: epic:hardening-follow-ups
- reviews: story:overlay-type-checks
- reviews: story:proposal-level-check
- reviews: story:channel-capacity
revision: 1
---
approve

I found nothing that would change what was drafted. The round-1 blocker on story:channel-capacity is fixed.

- **Round-1 finding, resolved.** The acceptance now names a check that tells the old model from the new one: a test on the generated model that fails if the client offers a choice between two of the six transitions (`.engineering/planning/story/channel-capacity.md:35`). The current model fails that test. The `open` state offers both `send-text` and `start-message`, and the overlay's own wording at `profiles/websocket/overlay.yaml:54` says the client takes three paths. The round-1 minor point is also settled: capacity 0 now has a message and a named unit test in `check.rs` (line 33).
- **story:overlay-type-checks.** The three errors can be checked. `crates/rfcspec-extract/src/check.rs` currently checks only whether a guard operand's name exists (around line 150) and has no type check. The acceptance names three overlays and a clean pass on the committed profiles.
- **story:proposal-level-check.** The test's subject exists: `rfc6455/R5.1-1` is MUST (`profiles/websocket/overlay.yaml:5`) and has a recorded proposal. The `profiles/websocket/proposals/` directory holds exactly ten files. `rfcspec propose --replay` exists and is already a step of `task websocket`.
- **epic:hardening-follow-ups.** Its acceptance is "each story passes and `task check` exits 0", which can be checked.

What I read: 4 of 4 ids (epic:hardening-follow-ups, story:overlay-type-checks, story:proposal-level-check, story:channel-capacity) with `aep plan artifact show`, plus review-result:hardening-acceptance-round-1. I also read the rubric, `Taskfile.yml`, `check.rs`, `crates/rfcspec-ess/src/lib.rs`, `profiles/websocket/overlay.yaml` and the fragments witness. I ran `ess protocol explore` on the current fragments model, which printed `"verdict": "passed"`.

What I could not establish:
- **Unobservable trace phrase.** `ess protocol explore` prints counts, not traces. The phrase "reached in one explored trace" has no observable of its own, so the structural no-choice test carries that bullet. I judged that enough to tell the two models apart and not a finding.
- **Replay and mutant not run.** I did not run `rfcspec propose --replay`, so I can't say whether the ten proposals replay unchanged after the level check. I did not run the mutant test.
- **Out of my lane (scope).** Moving to one path will make two things stale. One is the `overlay.yaml:54` status text, "three paths". The other is the test at `crates/rfcspec-ess/src/lib.rs:898`, which compares the witness byte for byte with the close-only input. Both files are in the story's scope list, so the scope critic can judge whether the plan accounts for them.

```findings
[]
```
