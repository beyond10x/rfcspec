---
format: aep.planning-md/3
id: story:rfc868-extract-overlay
kind: story
status: implemented
title: Extract the mechanical catalogue and check a reviewed overlay
relations:
- decomposes: epic:rfc868-thin-slice
- depends_on: story:rfc868-source-reader
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-11-04T22:38:34Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Story

`rfcspec extract` writes the mechanical catalogue of one RFC, and `rfcspec check` verifies a
reviewer's overlay against it: every quote in the overlay must occur verbatim in the source at the
span it names.

## Acceptance

- On RFC 868, `extract` reports 0 BCP 14 requirements and lists every sentence containing a
  lowercase `must`, `should` or `may` as a prose candidate with its span; the two `should`
  sentences ("…it should either refuse the connection or close it without sending anything", "…it
  should discard the arriving datagram and make no reply") are among them.
- `extract` lists the 11 dialogue steps (6 TCP, 5 UDP) as `(party, text, span)`, party `S` or `U`.
- `extract` lists the 5 numeric examples as `(seconds, text, span)`, the last one -1297728000.
- `profiles/time/overlay.yaml` holds the reviewed requirements (prose source), the layout, the two
  machines, a status per requirement, and a review finding for the fifth example (-1297728000
  cannot be a 32-bit unsigned value), which later stories only read.
- `rfcspec check --profile profiles/time` exits 0, and exits
  1 naming the item when one quote is changed by one character.
- The catalogue output is byte-identical across two runs.

## Scope

`crates/rfcspec-extract`, `crates/rfcspec-model` (catalogue and overlay types),
`crates/rfcspec-cli` (`extract`, `check`), `profiles/time/overlay.yaml`.

Depends on the reader story.
