---
format: aep.planning-md/3
id: story:websocket-ping-machine
kind: story
status: implemented
title: Model RFC 6455 Ping and Pong (sections 5.5.2 and 5.5.3) as the websocket ping machine
relations:
- decomposes: epic:websocket-frames
- depends_on: story:websocket-machine-guards
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: cited
  path: crates/rfcspec-ess/src/lib.rs
- confidence: cited
  path: docs/design/rfcspec.md
- confidence: cited
  path: profiles/websocket/generated/ping.protospec.yaml
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/witnesses/ping.inputs.json
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/websocket.md
- confidence: cited
  path: website/docs/design.md
- confidence: cited
  path: website/static/downloads/websocket
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:48Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:48Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T08:10:33Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Story

RFC 6455 sections 5.5.2 (Ping) and 5.5.3 (Pong) become the `websocket` profile reviewed machine
`ping` on the `tcp` layer, lowered by `rfcspec lower` to `generated/ping.protospec.yaml` and explored
in `task check` beside `close`. Its six catalogue requirements, all in the overlay today as
`unsupported: "Ping and Pong frames are not modelled by this profile's machines"`, get a status that
is true of the model. The Pong echo of the Ping data (section 5.5.3, lines 2060-2062, lowercase
"must", a prose candidate, not a catalogue requirement) is what the mutant breaks.

## Acceptance

`ess protocol explore` on `generated/ping.protospec.yaml` with `witnesses/ping.inputs.json` prints
`"verdict": "passed"`, a mutant whose Pong does not echo the Ping data is reported, and the
websocket satisfied count rises from 17 to 21.

- The machine (simplest shape; the implementor may rename states): messages `ping {data: Integer}`,
  `pong {data: Integer}`, `close-frame {code: Integer}`. Client: local field `ping-data: Integer`
  (initial 0); input `ping {data}` from `open` sets `ping-data` and sends `ping`; receives `pong`
  under the guard `data == ping-data`; input `close {code}` from `open` or after the Pong sends
  `close-frame`; input `ping` after the Close is also offered, so a Ping can reach a server that
  already received the Close. Server: in `open` receives `ping` and sends `pong` with `data` from the
  received `data`; receives `close-frame` into `close-received`; in `close-received` receives `ping`
  and sends nothing. Each client input transition is taken at most once per run (see
  `story:websocket-machine-guards`, Notes). Every transition name leads with catalogue ids.
- Overlay statuses (ids, spans and quotes as in the overlay today, read from the RFC):
  - R5.5.2-1 (MAY, lines 2040-2040, "A Ping frame MAY include \"Application data\".") satisfied by
    the client Ping transition;
  - R5.5.2-2 (MUST, lines 2042-2043, Pong in response unless a Close was received) satisfied by the
    server Pong transition; the `close-received` Ping transition implements its "unless" clause;
  - R5.5.2-3 (SHOULD, lines 2043-2044, "as soon as is practical") satisfied by the server Pong
    transition, as R5.5.1-7 is by the immediate Close echo;
  - R5.5.2-4 (MAY, lines 2047-2048, Ping any time after establishment and before close) satisfied by
    the client Ping transitions;
  - R5.5.3-1 (MAY, lines 2064-2067, Pong only for the most recent Ping) `unsupported: ess-gap G10`:
    choosing to skip a Ping needs a guard on whether a later Ping is pending on the incoming channel;
  - R5.5.3-2 (MAY, line 2079, "A Pong frame MAY be sent unsolicited.") stays unsupported with a reason
    true of this model: the ping machine sends a Pong only in reply. `story:websocket-fragments-machine`
    satisfies it with its unsolicited Pong.
- `docs/design/rfcspec.md` section 7 gains gap G10: a guard on a pending message of an incoming
  channel, needed by WebSocket section 5.5.3, proposal `pending(channel, message)` as a guard operand.
  The profile page lists G10 with R5.5.3-1 under "ESS gaps"; `website/docs/design.md` regenerated.
- `profiles/websocket/witnesses/ping.inputs.json` holds the inputs `ping` (data 7) and `close` (code
  1000) for the client.
- A test in `crates/rfcspec-ess` lowers a mutant overlay whose server Pong sends the literal 0 as
  `data` and sees explore report `inconclusive` with "dead end with outstanding queue, timer or flush
  obligations". A second test asserts the committed `ping.protospec.yaml` equals a fresh lowering and
  that explore passes it.
- `Taskfile.yml` `websocket` explores `ping.protospec.yaml` with its witness; removing the model
  makes `task check` exit 1.
- The profile page reads "by status 21 satisfied, 32 unsupported, 4 not applicable" (reviewed stays
  57); the catalogue index row shows 21 satisfied and 32 unsupported; downloads hold
  `ping.protospec.yaml`; the test in `crates/rfcspec-docs/src/catalog.rs` that pins the reviewed line
  is updated; `rfcspec-docs generate --check` and `task check` pass.
- No overlay reason contains "Ping and Pong frames are not modelled".
- The pinned figures above assume `epic:ai-proposals` has not landed, which the recorded order
  guarantees (`story:websocket-proposal-slice` depends on `story:websocket-fragments-machine`).

## Domain relations

- Profile -> Machine, one-to-many, owned: `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`,
  relation `machines`. `ping` is a second machine of the `websocket` profile.
- Machine -> LayerSemantics, many-to-one by transport name; `ping` and `close` both run over `tcp`.
  Inferable (inferred from `crates/rfcspec-cli/src/checks.rs:67`; no `relations:` entry declares it).
- Profile -> Status, one-to-many, owned: relation `statuses`; Status -> Requirement, one: entity
  `rfcspec.profile.Status`, relation `requirement` (`ess/domains/profile.yaml`).
- Status -> machine element, zero or one, by the name `<machine>.<participant>.<transition>`: field
  `element` of `rfcspec.profile.Status`, invariant `any: [kind != Satisfied, defined(element)]`; the
  name resolution is inferred from `crates/rfcspec-extract/src/check.rs:111`.

## Scope

`profiles/websocket/overlay.yaml`, `profiles/websocket/generated/ping.protospec.yaml`,
`profiles/websocket/witnesses/ping.inputs.json`, `crates/rfcspec-ess/src/lib.rs` (tests),
`crates/rfcspec-docs/src/catalog.rs` (pinned counts), `Taskfile.yml`, `docs/design/rfcspec.md`
(G10), `website/docs/design.md`, `website/docs/catalog/websocket.md`, `website/docs/catalog/index.md`,
`website/static/downloads/websocket/`, `AGENTS.md` (State row and gate steps).

Depends on `story:websocket-machine-guards` (local field, `set`, guards).
