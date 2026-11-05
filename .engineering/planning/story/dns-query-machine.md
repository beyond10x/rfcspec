---
format: aep.planning-md/3
id: story:dns-query-machine
kind: story
status: implemented
title: Review and lower a UDP query/response machine that matches responses by ID, adding state_implication to the overlay model
relations:
- decomposes: epic:dns-profile
- depends_on: story:dns-header-codec
- depends_on: story:websocket-machine-guards
scope:
- confidence: cited
  path: AGENTS.md
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
  path: profiles/dns/generated/query.protospec.yaml
- confidence: cited
  path: profiles/dns/overlay.yaml
- confidence: cited
  path: profiles/dns/profile.yaml
- confidence: cited
  path: profiles/dns/witnesses/query.inputs.json
- confidence: cited
  path: website/docs/catalog/dns.md
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/static/downloads/dns
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:49Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:50Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "active", to: "implemented", at: "2026-11-05T08:35:44Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":5}}}
---
## Story

The `dns` profile gets a reviewed UDP query/response machine `query` in which the client
accepts a response only when its ID equals the ID of the query it sent (RFC 1035 sections 4.1.1 and
7.3), lowered by `rfcspec lower` to `ess-protospec/1` and explored in `task check`, and the
profile declares the semantics of its `udp` layer with correlation `by-id`. This is the axis
neither `time` nor `websocket` exercises: a reply found by an identifier, not by order or by
the connection.

The overlay machine model this story starts from is the one `story:websocket-machine-guards`
leaves: typed inputs, literals and input fields as payload values, send, emit, close, timers,
flush, participant local fields with an initial literal, the `set` effect, transition `guards` as
a conjunction of equalities over literal, `{local:}` and `{input:}` operands, and the
`event_count`, `flush_before_close` and `never_state` properties. The query machine uses those as
delivered and adds nothing to them. What it still lacks is the `ess-protospec/1` property
`state_implication` (ess 0.86.0, in the ess repository `crates/specify/ess-domain/src/protocol.rs`):
one participant in a state implies another participant in a state. This story carries exactly
that property into the overlay model, the check and the lowering, and adds its own machine.

## Acceptance

`rfcspec lower --profile profiles/dns` writes `generated/query.protospec.yaml`, `ess protocol
explore` on it with the committed witness prints `"verdict": "passed"`, and the same exploration
of a mutant whose client accepts a response without the ID guard reports the property
`answer-matches-query` violated.

- `profiles/dns/profile.yaml` gains one layer, `udp`: unit `datagrams`, delivery `{ordered:
  false, reliable: false, duplicates: true}`, direction `half-duplex`, multiplexing
  `sessions`, exchange `request-response`, correlation `by-id`, lifetime
  `connectionless`, roles `[client, server]` (design section 6.2, DNS row, for the UDP
  transport). The profile page shows exactly this one semantics row.
- Overlay model (`crates/rfcspec-model`): a property may be `{name, participant, state, other,
  other_state}` (`state_implication`), lowered by `crates/rfcspec-ess` to the `ess-protospec/1`
  `state_implication` property. Participant fields, `set`, guards and `{local:}`/`{input:}`
  operands are used as `story:websocket-machine-guards` delivers them, with its refusals; this
  story changes none of them. Unknown keys stay parse errors. `rfcspec check` refuses, naming the
  machine and the property, a `state_implication` whose `participant` or `other` names no
  participant of the machine, or whose `state` or `other_state` names no state of that
  participant; one test each.
- The overlay accepts these requirements, each `source: prose` with its quote verbatim:
  - `R-ID-1`, MUST, "This identifier is copied the corresponding reply" (lines 1424-1425),
    satisfied by the server transition that answers a query with its ID;
  - `R-MATCH-1`, SHOULD, "The recommended strategy is to do a preliminary matching using the ID
    field in the domain header" (lines 2544-2545), satisfied by the client transition that accepts
    a matching response;
  - `R-MATCH-2`, SHOULD, "and then to verify that the question section corresponds to the
    information currently desired." (lines 2545-2546), status `unsupported`: the messages carry
    the ID only, the question section (4.1.2) is not modelled;
  - `R-UDP-1`, SHOULD NOT, "Queries or their responses may be reordered by the network, or by
    processing in name servers, so resolvers should not depend on them being returned in order."
    (lines 1762-1764), satisfied by the server transition that sends a late response;
  - `R-UDP-2`, MUST, "Queries sent using UDP may be lost, and hence a retransmission strategy is
    required." (lines 1761-1762), status `unsupported`: retransmission is not modelled and the
    lowered channels are reliable.
- The lowered model `rfc1035-dns-query`: messages `query {id: Integer}` and `response {id:
  Integer}`. Client states `idle`, `awaiting`, `answered`; local field `qid: Integer`,
  initial 0; input `query {id: Integer}`; transitions: send the query (`set qid` from the input,
  send `query` with the input ID) idle to awaiting; accept a response awaiting to answered,
  guarded by `{input: id} = {local: qid}`; discard a response awaiting to awaiting and answered to
  answered, unguarded (the inequality guard is gap G1; the nondeterministic discard
  over-approximates it). Server states `idle`, `late`, `replied`; on receiving a query, send a response
  with the query ID, from `idle` or `late` to `replied`; input `late-response {id: Integer}` from
  `idle` only sends a response with that ID and goes to `late` (a reply to an earlier query,
  reordered by the network), so it is taken at most once per run and exploration has no input
  cycle (websocket-machine-guards Notes). Property
  `answer-matches-query`: client `answered` implies server `replied`. Every transition name
  leads with the requirement ids it implements.
- `profiles/dns/witnesses/query.inputs.json`: client `query` with id 1, server
  `late-response` with id 2. `ess protocol explore` with it prints `"verdict": "passed"`.
- A test lowers a mutant overlay whose accepting transition has no guard and sees `ess protocol
  explore` with the same witness print a verdict other than `passed` naming
  `answer-matches-query`: the late response with id 2 is accepted while the server is still
  `idle`.
- `rfcspec lower --check` passes for `time` and `websocket` with no change to their
  `generated/` models (new keys are omitted when empty).
- `Taskfile.yml` task `dns` also runs `rfcspec lower --profile profiles/dns --check` and the
  exploration of `query.protospec.yaml` with its witness; removing the generated model makes
  `task check` exit 1.
- The profile page lists `query.protospec.yaml` under downloads and the five requirements in its
  matrix; `rfcspec-docs generate --check` passes. `AGENTS.md` (State, Gate) names three
  profiles and the `dns` task. `task check` exits 0.

## Domain relations

- Profile -> Machine, one-to-many, the profile owns its machines: `ess/domains/profile.yaml`,
  entity `rfcspec.profile.Profile`, relation `machines`.
- Profile -> LayerSemantics, one-to-many as a value list, one per distinct layer:
  `ess/domains/profile.yaml`, entity `rfcspec.profile.Profile`, field `semantics`, invariant
  `distinct: {in: semantics, as: s, by: s.layer}`; layer = transport per
  `decision-blocker:catalog-multi-transport-semantics`.
- Machine -> LayerSemantics, many-to-one by transport name. Inferable (inferred from
  `crates/rfcspec-cli/src/checks.rs:69`; no `relations:` entry declares it).
- Profile -> Status, one-to-many, owned: `ess/domains/profile.yaml`, entity
  `rfcspec.profile.Profile`, relation `statuses`; Status -> Requirement, one: entity
  `rfcspec.profile.Status`, relation `requirement`.

## Scope

`crates/rfcspec-model/src/lib.rs` (`state_implication` only), `crates/rfcspec-ess` (its
lowering, mutant test), `crates/rfcspec-extract/src/check.rs` (the `state_implication`
refusals), `crates/rfcspec-cli/tests`,
`profiles/dns/profile.yaml`, `profiles/dns/overlay.yaml`,
`profiles/dns/generated/query.protospec.yaml`, `profiles/dns/witnesses/query.inputs.json`,
`Taskfile.yml`, `AGENTS.md`, `website/docs/catalog/index.md`, `website/docs/catalog/dns.md`,
`website/static/downloads/dns/` (regenerated).

Choices taken:

- Channels stay as lowered today (FIFO, reliable, no duplication, capacity 4), as for the `time`
  UDP model; lowering `delivery` from the layer semantics (design section 6.1) would change the
  `time` models and is left out. Reordering is modelled by the server `late-response` input.
- One outstanding query: concurrent queries are participant families keyed by ID (gap G4). The
  layer still says `multiplexing: sessions`, as design section 6.2 does.
- `unit: datagrams` for the `udp` layer, as the `time` profile declares it, not
  `messages` from the design table, whose row describes DNS over either transport.
- No `rfcspec check` rule ties `correlation: by-id` to a guarded transition.
- DNS over TCP (section 4.2.2), retransmission and the question section are not in this story.

Depends on `story:dns-header-codec`: both edit `profiles/dns/overlay.yaml`, `Taskfile.yml`,
`crates/rfcspec-cli/tests` and the generated catalogue pages, so they cannot run in parallel.

Depends on `story:websocket-machine-guards`: the query machine's local field `qid`, its `set` and
its ID guard are the constructs that story adds to `crates/rfcspec-model/src/lib.rs`,
`crates/rfcspec-extract/src/check.rs` and `crates/rfcspec-ess`, used here as delivered; this
story adds only `state_implication` to the same three, so it lands after that story.
