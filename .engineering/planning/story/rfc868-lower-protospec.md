---
format: aep.planning-md/3
id: story:rfc868-lower-protospec
kind: story
status: implemented
title: Lower the Time Protocol machines to ess-protospec/1
relations:
- decomposes: epic:rfc868-thin-slice
- depends_on: story:rfc868-extract-overlay
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-04T22:38:35Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

`rfcspec lower` turns the overlay's machines into `ess-protospec/1` models, one per transport.

## Acceptance

- `rfcspec lower --profile profiles/time --out profiles/time/generated` writes
  `tcp.protospec.yaml` and `udp.protospec.yaml`; every transition name carries the requirement or
  dialogue-step id it implements.
- Each model declares an `event_count` property: the server emits `time-sent` at most once per
  session. `ess protocol explore` on each, with the committed input witnesses, prints
  `"verdict": "passed"`; `task check` runs both and fails otherwise.
- A mutant lowering in the tests that lets the TCP server send `time` twice makes `ess protocol
  explore` report the `event_count` violation, so the property is seen to fail.
- The TCP model has a server path "unable to determine the time" that closes without sending; the
  UDP model has a `time` reply or no reply.
- `rfcspec lower --check` exits 1 when a generated file differs from a fresh lowering.
- `Taskfile.yml` `check` runs `rfcspec check --profile profiles/time`, `rfcspec lower --check` and
  `ess protocol explore` on both models; removing one generated model makes `task check` exit 1.

## Scope

`crates/rfcspec-ess`, `crates/rfcspec-cli` (`lower`), `profiles/time/generated/*.protospec.yaml`,
`profiles/time/witnesses/`, `Taskfile.yml`, workspace `Cargo.toml`, `Cargo.lock`. Reads
`profiles/time/overlay.yaml`, writes nothing in it.

Ordering: the codec and probe stories depend on this one, because all three edit
`crates/rfcspec-cli`, `Cargo.toml` and `Cargo.lock`.

Depends on the extract story.
