---
title: Getting started
sidebar_position: 2
description: "What exists today, and the one check you can run now: bounded exploration of the WebSocket closing-handshake model with ess."
---

RFCspec is at the design stage. There is no `rfcspec` command to install yet. What exists is the
design, RFCspec's own ESS specification, and one protocol model you can check with `ess` today.

## Read the design

The [design](/docs/design) holds every decision (section 0), the pipeline from an RFC to
generated code (section 4), and the milestones (section 14). The [WebSocket](./examples/websocket.md)
page walks the first worked example.

## Explore the WebSocket closing-handshake model

The repository holds a small `ess-protospec/1` model of WebSocket's closing handshake (RFC 6455
section 7) and one input witness: the client starts closing with status code 1000.

You need [ESS](https://beyond10x.github.io/ess/) ([GitHub](https://github.com/beyond10x/ess))
0.86.0 or newer on your `PATH`.

```console
git clone https://github.com/beyond10x/rfcspec
cd rfcspec
ess protocol explore --path examples/websocket/close.protospec.yaml --inputs examples/websocket/close.inputs.json
```

Output, recorded with `ess` 0.86.0:

```json
{
  "time_policy": "inputs_at_initial_time_and_timer_deadlines",
  "verdict": "passed",
  "reason": "bounded model search completed for the supplied finite input witnesses; not implementation evidence",
  "explored_states": 10,
  "explored_transitions": 11,
  "bounds": {
    "max_steps": 64,
    "max_states": 1024,
    "max_time_ms": 30000
  },
  "inputs": [
    {
      "kind": "input",
      "participant": "client",
      "name": "close",
      "payload": {
        "code": 1000.0
      }
    }
  ],
  "counterexample": null
}
```

`passed` is model evidence: every schedule ESS explored within the bounds ended with no queued
message, armed timer or open flush obligation. It says nothing yet about any implementation.

Without the input witness, nothing starts the handshake, and the same command exits 2 with
`"verdict": "inconclusive"` and `"reason": "no transitions exercised"`.

## Build the repository

`task check` in the repository validates the planning store and the ESS specification, explores
the example models, checks the generated documentation pages, and runs format, Clippy and tests.
It needs Rust, [go-task](https://taskfile.dev), `ess` and `aep`.
