---
format: aep.planning-md/3
id: story:channel-capacity
kind: story
status: implemented
title: Let an overlay machine declare its channel capacity and send every fragments frame in one run
relations:
- decomposes: epic:hardening-follow-ups
- depends_on: story:overlay-type-checks
scope:
- confidence: cited
  path: crates/rfcspec-ess/src/lib.rs
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: profiles/websocket/generated
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/witnesses/fragments.inputs.json
- confidence: cited
  path: website/docs/catalog/websocket.md
- confidence: cited
  path: website/static/downloads/websocket
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T11:45:39Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T11:45:39Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T11:53:08Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Story

The lowering writes every channel with the constant capacity 4 (`crates/rfcspec-ess/src/lib.rs` `CAPACITY`), so the websocket fragments client cannot send text, start, Pong, continue, finish and close in one run and takes one of three paths instead (implementation note of story:websocket-fragments-machine). An overlay machine may declare `capacity` (a positive integer) on its transport; absent, it stays 4, so every committed model lowers byte for byte as today.

## Acceptance

- `crates/rfcspec-model`: `capacity` on the machine transport, omitted when absent; `rfcspec check` refuses 0 with the message `machine <name>: capacity 0 is not a positive integer`, asserted by a unit test in `crates/rfcspec-extract/src/check.rs`.
- `crates/rfcspec-ess`: the lowered channels carry the declared capacity; a test lowers a machine with capacity 8 and finds it on both channels.
- `profiles/websocket/overlay.yaml`: the fragments machine declares capacity 8 and its client sends text, start, Pong, continue, finish and close in one run: the client has one path, `open -> text -> start -> pong -> continue -> finish -> close`, every transition of which a test asserts is reached in one explored trace (a test reads the generated model and fails if the client offers a choice between two of those transitions); `ess protocol explore` on the regenerated `fragments.protospec.yaml` prints `"verdict": "passed"`, and the mutant test still fails with `server-never-fails`.
- `rfcspec lower --check` passes for `time`, `dns` and the `close` and `ping` models of `websocket` with no byte changed; the implementation note in story:websocket-fragments-machine stays as history.
- `task check` exits 0.
