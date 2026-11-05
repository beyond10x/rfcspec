---
format: aep.planning-md/3
id: story:websocket-fragments-machine
kind: story
status: implemented
title: Model RFC 6455 data frames and fragmentation (sections 5.4 and 5.6) as the websocket fragments machine
relations:
- decomposes: epic:websocket-frames
- depends_on: story:websocket-machine-guards
- depends_on: story:websocket-ping-machine
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli/tests/websocket.rs
- confidence: cited
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: cited
  path: crates/rfcspec-ess/src/lib.rs
- confidence: cited
  path: profiles/websocket/generated/fragments.protospec.yaml
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/profile.yaml
- confidence: cited
  path: profiles/websocket/witnesses/fragments.inputs.json
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/websocket.md
- confidence: cited
  path: website/static/downloads/websocket
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:48Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:48Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T08:16:55Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Story

RFC 6455 sections 5.4 (Fragmentation) and 5.6 (Data Frames) enter the `websocket` profile scope and
become the reviewed machine `fragments` on the `tcp` layer, lowered to
`generated/fragments.protospec.yaml` and explored in `task check`. The server reassembles text
messages, accepts a control frame between fragments, and enters a `failed` state (Fail the
WebSocket Connection) on a fragmented control frame or a new message started inside a fragmented
one; a `never_state` property forbids `failed`, so a correct client passes and a wrong one fails.
The overlay reasons that say data frames, messages or fragmentation are not modelled are replaced
by statuses true of the profile.

## Acceptance

`ess protocol explore` on `generated/fragments.protospec.yaml` with `witnesses/fragments.inputs.json`
prints `"verdict": "passed"`, a mutant that sends a fragmented control frame in the middle of a
message is reported `failed`, all 13 requirements of sections 5.4 and 5.6 have a status, and the
websocket satisfied count rises from 21 to 29.

- `profiles/websocket/profile.yaml` sections become `["5.2", "5.4", "5.5", "5.6", "7"]`; the test in
  `crates/rfcspec-cli/tests/websocket.rs` that pins the list and the RFC row test in
  `crates/rfcspec-docs/src/catalog.rs` follow.
- The machine (simplest shape): messages `data-frame {opcode: Integer, fin: Boolean}`,
  `pong {fin: Boolean}`, `close-frame {code: Integer}`. Client inputs, each taken at most once per run:
  send one unfragmented text message (opcode 1, fin true); start a fragmented one (opcode 1, fin
  false); send an unsolicited Pong (fin true) only inside the fragmented message; continue (opcode 0,
  fin false); finish (opcode 0, fin true); `close {code}` only outside a message, sending
  `close-frame`, after which the client sends nothing. Server: `idle` and `in-message`, guards on
  `opcode` and `fin` literals; emits `message-received` on each complete message; receives `pong`
  with fin true in either state without reply; enters `failed` on a `pong` with fin false or on
  `opcode == 1` while `in-message`; receives `close-frame` into `closed`. Property
  `never_state: {participant: server, state: failed}`. Text only: binary (opcode 2) is left out.
- Overlay statuses, ids from the catalogue of the locked source (sentence rule of
  `crates/rfcspec-extract/src/lib.rs:294`, per-section ordinals):
  - R5.4-1 (MAY, lines 1894-1895, control frames injected mid-message) satisfied by the client
    in-message Pong transition;
  - R5.4-2 (MUST NOT, lines 1895-1896, control frames not fragmented) satisfied by that transition
    (fin true);
  - R5.4-3 (MUST, lines 1898-1899, fragments delivered in order) unsupported, no gap: order is the
    `tcp` layer delivery `ordered: true`, not a machine element;
  - R5.4-4 (MUST NOT, lines 1911-1913, no interleaving of messages) satisfied by the server transition
    that fails on a new message inside one;
  - R5.4-5 (MUST, lines 1915-1916, handle control frames mid-message) satisfied by the server
    in-message Pong transition;
  - R5.4-6 (MAY, lines 1918-1919, fragments of any size) `unsupported: ess-gap G3` (payload bytes and
    lengths);
  - R5.4-7 (MUST, lines 1921-1922, clients and servers receive both forms) unsupported, no gap: only
    the server receives data frames in the model;
  - R5.4-8 to R5.4-11 (MUST NOT, lines 1924-1925, 1927-1929, 1931-1934, 1934-1937) not-applicable:
    addressed to intermediaries, and the profile models two endpoints;
  - R5.4-12 (MUST, lines 1940-1943, all fragments text, binary or reserved) satisfied by the server
    transitions that start a message on opcode 1 and continue only on opcode 0;
  - R5.6-1 (MUST, lines 2096-2098, whole message valid UTF-8) `unsupported: ess-gap G3`;
  - R5.5.1-5 (no data frames after a Close) satisfied by the client `close` transition;
  - R5.5.1-8 (MAY finish the current message before the Close) satisfied by the client `close`
    transition, offered only outside a message;
  - R5.5.3-2 (Pong MAY be sent unsolicited) satisfied by the client in-message Pong transition;
  - R5.5-1, R5.2-3, R5.2-4 and R7.4.1-2 keep `unsupported`, with reasons rewritten to stay true: no
    reason in the overlay contains "data frames are not modelled", "carries no data frames",
    "fragmentation are not modelled" or "fragmentation is not modelled".
- If `rfcspec extract` on the locked source yields other ids, spans or levels for sections 5.4 and
  5.6 than the 13 above, the extraction wins and this list is corrected before the story moves on.
- `profiles/websocket/witnesses/fragments.inputs.json` holds the client input `close` (code 1000);
  the zero-field inputs need no witness.
- A test in `crates/rfcspec-ess` lowers a mutant overlay whose client in-message Pong sends fin false
  and sees explore report `failed` with "property violated" naming the `never_state` property. A
  second test asserts the committed model equals a fresh lowering and that explore passes it.
- `Taskfile.yml` `websocket` explores `fragments.protospec.yaml`; removing it makes `task check` exit 1.
- The profile page reads "Reviewed: 70 requirements, by level 26 MUST, 12 MUST NOT, 11 SHOULD, 3
  SHOULD NOT, 18 MAY; by status 29 satisfied, 33 unsupported, 8 not applicable." and 126 unreviewed
  (82 MUST, 11 MUST NOT, 7 SHOULD, 1 SHOULD NOT, 24 MAY, 1 OPTIONAL); the index row matches; downloads
  hold `fragments.protospec.yaml`; `rfcspec-docs generate --check` and `task check` pass.
- The pinned figures above assume `epic:ai-proposals` has not landed, which the recorded order
  guarantees (`story:websocket-proposal-slice` depends on this story).

## Domain relations

- Profile -> ProfileRfc, one-to-many as a value list, and ProfileRfc -> sections, a list of section
  ids where empty means the whole RFC: `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`,
  field `rfcs: List<rfcspec.profile.ProfileRfc>`, struct field `sections`.
- Profile -> Machine, one-to-many, owned: `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`,
  relation `machines`. `fragments` is the third machine of the profile.
- Machine -> LayerSemantics, many-to-one by transport name. Inferable (inferred from
  `crates/rfcspec-cli/src/checks.rs:67`; no `relations:` entry declares it).
- Profile -> Status, one-to-many, owned: relation `statuses`; Status -> Requirement, one: entity
  `rfcspec.profile.Status`, relation `requirement` (`ess/domains/profile.yaml`).
- Status -> machine element, zero or one, by name: field `element` of `rfcspec.profile.Status`; name
  resolution inferred from `crates/rfcspec-extract/src/check.rs:111`.

## Scope

`profiles/websocket/profile.yaml`, `profiles/websocket/overlay.yaml`,
`profiles/websocket/generated/fragments.protospec.yaml`,
`profiles/websocket/witnesses/fragments.inputs.json`, `crates/rfcspec-ess/src/lib.rs` (tests),
`crates/rfcspec-cli/tests/websocket.rs`, `crates/rfcspec-docs/src/catalog.rs` (pinned counts and
sections), `Taskfile.yml`, `website/docs/catalog/websocket.md`, `website/docs/catalog/index.md`,
`website/static/downloads/websocket/`, `AGENTS.md` (State row and gate steps).

Depends on `story:websocket-machine-guards` (guards, `never_state`) and on
`story:websocket-ping-machine`: both edit the overlay, `Taskfile.yml`, the pinned counts and the
same generated pages, and this story moves R5.5.3-2 from the reason that story leaves to satisfied.

## Implementation note

The lowered channel capacity is 4 (`crates/rfcspec-ess/src/lib.rs` `CAPACITY`). A client sending text, start, Pong, continue, finish and close in one run exhausts it (explore: "channel client-to-server capacity exhausted"). The client therefore takes one of three paths per run: text then close; start, Pong, finish, close; start, continue, finish, close. Every frame kind the story lists is still sent in some run and each input is taken at most once. A configurable capacity is not in this story.
