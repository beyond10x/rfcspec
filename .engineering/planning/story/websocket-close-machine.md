---
format: aep.planning-md/3
id: story:websocket-close-machine
kind: story
status: implemented
title: Review and lower the RFC 6455 closing handshake, with timers and flush in the overlay machine model
relations:
- decomposes: epic:websocket-profile
- depends_on: story:websocket-source-catalogue
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-ess
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: profiles/websocket/generated/close.protospec.yaml
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/profile.yaml
- confidence: cited
  path: profiles/websocket/witnesses/close.inputs.json
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/websocket.md
- confidence: cited
  path: website/static/downloads/websocket
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T00:11:32Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-05T00:11:32Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-05T00:42:15Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Story

The closing handshake of RFC 6455 (sections 5.5.1 and 7) becomes the `websocket` profile reviewed
machine `close`, lowered by `rfcspec lower` to `ess-protospec/1` and explored in `task check`, and
the profile declares its communication semantics. The overlay machine model and the lowering only
carry what the Time Protocol needed (zero-field inputs, integer literals, send, emit and close, one
`event_count` property, fixed bounds); this story generalises them to what the design section 12.4
model uses: timers (`arm` with a fixed delay, `cancel`, a timer trigger), inputs with typed fields
and payload values read from them, Boolean literals, `flush` and the `flush_before_close` property,
exploration bounds per machine, and participants without a dialogue party. The Time Protocol
models lower byte-identically.

## Acceptance

`rfcspec lower --profile profiles/websocket` writes `generated/close.protospec.yaml`, `ess protocol
explore` on it with the committed witness prints `"verdict": "passed"`, and `task check` runs both.

- `profiles/websocket/profile.yaml` gains one layer, `tcp`: unit `bytes`, ordered, reliable, no
  duplicates, `full-duplex`, `single`, exchange `session`, correlation `none`, lifetime
  `connection`, roles `[client, server]` (design section 6.2, WebSocket row). `over` is omitted:
  `rfcspec check` refuses an `over.profile` that names no profile directory, and no `tcp` profile
  exists. The profile page shows exactly this one semantics row.
- The overlay accepts, with source `capitals-text`, the catalogue id as its id and the quote verbatim:
  - section 7.1.2, lines 2305-2309, "To _Start the WebSocket Closing Handshake_ ... an endpoint MUST
    send a Close control frame ...", satisfied by the client transition that starts closing;
  - section 5.5.1, lines 2005-2006, "If an endpoint receives a Close frame and did not previously
    send a Close frame, the endpoint MUST send a Close frame in response.", satisfied by the server
    echo;
  - section 5.5.1, lines 2025-2029, "The server MUST close the underlying TCP connection
    immediately; ...", satisfied by the server echo (its `close` effect);
  - section 7.1.7, lines 2386-2388, "To do so, the client MUST _Close the WebSocket Connection_,
    ...", status `unsupported: ess-gap G9`: the client close on the closing timeout is left out of
    the model, because protospec keeps a message in flight to a closed participant as an obligation
    (design sections 7 and 12.4).
- `rfcspec check` refuses, naming the requirement, an overlay requirement with source `capitals-text`
  or `abnf-comment` whose id is no catalogue requirement, or whose span does not lie within that
  catalogue requirement span; one test each. The level stays the reviewer (ESS
  `rfcspec.catalogue.SetRequirementLevel`).
- The lowered model: client states `open`, `closing`, `closed`; server `open`, `closed`; client
  input `close` with an `Integer` field `code`, sent in the close frame; timer `close-timeout` armed
  for 5000 ms when closing starts and cancelled when the echo arrives; the server sends, flushes and
  closes; properties `flush_before_close` (server, server-to-client channel) and `event_count`
  (client emits `connection-closed` at most once). Every transition name leads with the requirement
  ids it implements; a transition implementing nothing is refused, as today.
- `ess protocol explore` with `profiles/websocket/witnesses/close.inputs.json` (one client `close`
  input, code 1000) prints `"verdict": "passed"`. A test lowers a mutant overlay whose client
  `close-timeout` transition also closes the client and sees explore report `inconclusive` with
  "dead end with outstanding queue, timer or flush obligations": gap G9 is seen to fire.
- `rfcspec lower --profile profiles/time --check` passes with no change to `profiles/time/generated/`.
- `Taskfile.yml` `check` runs `rfcspec lower --profile profiles/websocket --check` and the exploration
  of `close.protospec.yaml`; removing the generated model makes `task check` exit 1.
- The profile page shows G9 under "ESS gaps" with the section 7.1.7 requirement, and
  `close.protospec.yaml` under downloads; the index shows stage S5; `rfcspec-docs generate --check`
  passes.

## Domain relations

- Profile -> Machine, one-to-many, the profile owns its machines: `ess/domains/profile.yaml`, entity
  `rfcspec.profile.Profile`, relation `machines`.
- Profile -> LayerSemantics, one-to-many as a value list, one per distinct layer:
  `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`, field `semantics`, invariant
  `distinct: {in: semantics, as: s, by: s.layer}`; layer = transport per
  `decision-blocker:catalog-multi-transport-semantics`.
- Machine -> LayerSemantics, many-to-one by transport name. Inferable (inferred from
  `crates/rfcspec-cli/src/checks.rs:68` and `crates/rfcspec-model/src/lib.rs:319`; no `relations:`
  entry declares it).
- Profile -> Status, one-to-many, owned: `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`,
  relation `statuses`. Status -> Requirement, one: entity `rfcspec.profile.Status`, relation
  `requirement`.

## Scope

`crates/rfcspec-model` (machine, participant, trigger, effect, property and bounds types),
`crates/rfcspec-ess` (lowering), `crates/rfcspec-extract/src/check.rs` (machine elements, the
catalogue-id rule), `crates/rfcspec-cli` (tests), `profiles/websocket/profile.yaml`,
`profiles/websocket/overlay.yaml`, `profiles/websocket/generated/close.protospec.yaml`,
`profiles/websocket/witnesses/close.inputs.json`, `Taskfile.yml`, `website/docs/catalog/index.md`,
`website/docs/catalog/websocket.md`, `website/static/downloads/websocket/` (regenerated).
`examples/websocket/` stays as it is: the design section 12.4 illustration and its `examples` task.

Depends on `story:websocket-source-catalogue`: it edits the overlay and profile that story creates,
`crates/rfcspec-model`, `crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-cli` and the same
generated pages, and its requirement ids are the catalogue ids that story introduces.
