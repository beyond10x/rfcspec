---
format: aep.planning-md/3
id: epic:hardening-follow-ups
kind: epic
status: active
title: Close the adversary and implementation gaps of the follow-up epics
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T11:45:37Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T11:45:37Z", actor: "human:timo", revision: 3}
---
## Outcome

The gaps the two adversary passes and the frames implementation left open in the four follow-up epics are closed, so `rfcspec check` and `rfcspec propose` refuse what a later stage would otherwise refuse with a worse message, and the fragments machine can send every frame kind in one run.

Sources: review-result:frames-adversary-1 (finding 2, `check.rs:168`), review-result:dnsai-adversary-1 (finding 3, `lib.rs:611`), and the implementation note in story:websocket-fragments-machine (lowered channel capacity 4).

## Acceptance

Each story below is implemented with its own acceptance; `task check` exits 0.
