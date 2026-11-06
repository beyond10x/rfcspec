---
title: Getting started
sidebar_position: 2
description: "What exists in v0.2.0 and how to look at it: the RFC explorer, the protocol catalogue, the rfcspec command on the four profiles, and the WebSocket closing-handshake model in ess."
---

RFCspec v0.2.0 takes four RFCs through the pipeline and reads every RFC in the index. Nothing is
installed from a registry: you browse the results on this site, or build `rfcspec` from the
repository.

## Browse

- The [RFC explorer](pathname:///rfcspec/explorer/) searches every RFC the index publishes as
  plain text or RFCXML, and shows one RFC's requirements by section and level, its update graph
  and its references. For an RFC a profile holds, it also shows the reviewed layer: each
  requirement's status, the review decisions, the machines and the layouts.
- The [protocol catalogue](/docs/catalog) has one page per profile: its RFCs, communication
  semantics, requirement matrix, ESS gaps and the downloads of its generated files.

## Run `rfcspec` on a profile

You need Rust, and [ESS](https://beyond10x.github.io/ess/)
([GitHub](https://github.com/beyond10x/ess)) 0.87.0 or newer on your `PATH` for the models.

```console
git clone https://github.com/beyond10x/rfcspec
cd rfcspec
cargo build --locked -p rfcspec-cli
target/debug/rfcspec check --all profiles
```

Each profile's overlay is checked against its locked RFC:

```text
profiles/dns: overlay holds (6 requirements, 1 layouts, 1 machines, 0 findings, 1 layers, 0 recorded runs)
profiles/ntp: overlay holds (16 requirements, 1 layouts, 2 machines, 1 findings, 1 layers, 0 recorded runs); 4 model constructs implement no requirement: poll.client.model-horizon-unreachable, poll.client.model-horizon-reachable, poll.client.model-horizon-backoff, poll.client.model-horizon-discard
profiles/time: overlay holds (6 requirements, 1 layouts, 2 machines, 1 findings, 2 layers, 0 recorded runs)
profiles/websocket: overlay holds (155 requirements, 1 layouts, 3 machines, 0 findings, 1 layers, 0 recorded runs)
```

`rfcspec extract` prints a locked RFC's mechanical catalogue. RFC 6455 gives 196 requirements and
one grammar for each section with ABNF figures:

```console
target/debug/rfcspec extract --profile profiles/websocket --rfc 6455 | jq -c '[.grammars[].id]'
```

```json
["rfc6455/grammar/3","rfc6455/grammar/4.2.2","rfc6455/grammar/4.3","rfc6455/grammar/5.2","rfc6455/grammar/5.3","rfc6455/grammar/9.1"]
```

`rfcspec lower --profile <profile> --check` and `rfcspec generate --profile <profile> --check`
fail when a profile's lowered models or generated codec differ from what its overlay determines.

## Explore the WebSocket closing-handshake model

The repository also holds a hand-written `ess-protospec/1` model of WebSocket's closing handshake
(RFC 6455 section 7) and one input witness: the client starts closing with status code 1000.

```console
ess protocol explore --path examples/websocket/close.protospec.yaml --inputs examples/websocket/close.inputs.json
```

Output, recorded with `ess` 0.87.0:

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
message, armed timer or open flush obligation. It says nothing about any implementation.

Without the input witness, nothing starts the handshake, and the same command exits 2 with
`"verdict": "inconclusive"` with `"reason": "no transitions exercised"`.

## Build the repository

`task check` validates the planning store and the ESS specification, checks the generated types,
every profile (overlay, lowered models and their exploration, generated codec and its tests) and
the bulk summary, checks the generated documentation pages, and runs format, Clippy and tests. It
needs Rust, [go-task](https://taskfile.dev), `ess` and `aep`. `task website` builds the RFC
explorer and this site, and also needs Node and pnpm.
