---
format: aep.planning-md/3
id: story:rfc868-verify-probe
kind: story
status: implemented
title: Probe a live time server and print the requirement matrix
relations:
- decomposes: epic:rfc868-thin-slice
- depends_on: story:rfc868-generate-codec
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-04T21:52:56Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-04T22:38:35Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

`rfcspec verify` probes a live TCP time server and prints one row per requirement of the profile,
with the verdicts of design §11.

## Acceptance

- `rfcspec verify --profile profiles/time --target tcp://HOST:PORT` connects, reads until the
  server closes, and decodes with the generated codec.
- The matrix has exactly one row per requirement in `profiles/time/overlay.yaml`. TCP rows:
  exactly 4 octets sent (pass/fail), value within a tolerance (default 5 s, `--tolerance`) of the
  local clock (pass/fail), server closes after sending (pass/fail); "unable to determine the time"
  is `inconclusive` (not observable from a healthy server). Every UDP requirement row is
  `out of scope` (TCP target). A test fails if the row count differs from the overlay's
  requirement count.
- Tests start in-process servers on 127.0.0.1: a correct one gives every observable TCP row `pass`
  and exit 0; one that sends 5 octets, one that sends a value an hour off, and one that keeps the
  connection open past a timeout each give the matching row `fail` and exit 1.
- Every in-process server records the octets it receives; each test asserts 0 octets received
  (the probe never writes to the target). A server that never sends makes the probe stop at its
  read timeout (`--timeout`, default 10 s) with a `fail` row, not hang.

## Scope

`crates/rfcspec-conformance`, `crates/rfcspec-cli` (`verify`), workspace `Cargo.toml`,
`Cargo.lock`.

Depends on the codec story (and through it on the lowering story; shared `crates/rfcspec-cli`).
