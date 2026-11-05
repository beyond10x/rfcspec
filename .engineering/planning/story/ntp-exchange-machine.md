---
format: aep.planning-md/3
id: story:ntp-exchange-machine
kind: story
status: proposed
title: 'Review and lower the RFC 5905 on-wire exchange: accept by origin timestamp, zero xmt after the check, udp layer on port 123'
relations:
- decomposes: epic:ntp-profile
- depends_on: story:ntp-header-codec
- depends_on: story:ntp-timer-model
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-docs
- confidence: cited
  path: crates/rfcspec-ess
- confidence: cited
  path: crates/rfcspec-model/src/semantics.rs
- confidence: cited
  path: ess/domains/profile.yaml
- confidence: cited
  path: profiles/ntp/generated/exchange.protospec.yaml
- confidence: cited
  path: profiles/ntp/overlay.yaml
- confidence: cited
  path: profiles/ntp/profile.yaml
- confidence: cited
  path: profiles/ntp/witnesses/exchange.inputs.json
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/ntp.md
- confidence: cited
  path: website/static/downloads/ntp
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T13:50:23Z", actor: "human:timo", revision: 3}
---
## Story

The `ntp` profile gets a reviewed UDP machine `exchange` for the on-wire protocol of section 8
between a client association (mode 3) and a stateless server (mode 4, FXMIT in section 9.2): the
client accepts a server packet only when its origin timestamp equals the client's `xmt`
("A packet is bogus if the origin timestamp t1 in the packet does not match the xmt state variable
T1"), and sets `xmt` to zero after a successful check, so a replay of the last response is
discarded. This is correlation `by-id` where the id is a timestamp the client chose, and the first
machine whose channels may lose packets. The profile declares the semantics of its `udp` layer,
including port 123.

## Outcome

`profiles/ntp/generated/exchange.protospec.yaml` is lowered from the reviewed overlay, explored in
`task check` with its witness, and two named mutants are reported; the profile page shows one
`udp` semantics row with port 123.

## Acceptance

`rfcspec lower --profile profiles/ntp` writes `generated/exchange.protospec.yaml`; `ess protocol
explore --path profiles/ntp/generated/exchange.protospec.yaml --inputs
profiles/ntp/witnesses/exchange.inputs.json` prints `"verdict": "passed"`; the same exploration of
a mutant without the origin guard, and of a mutant without the `xmt` reset, each prints
`"verdict": "failed"` naming `one-valid-packet-per-poll`.

- Layer port: `ess/domains/profile.yaml` gives `rfcspec.profile.LayerSemantics` an optional
  `port` (`Optional<Integer>`, summary: the well-known port of the layer, addressing only, design
  section 6.2) before any code reads it; `ess specify validate --path ess` exits 0.
  `crates/rfcspec-model/src/semantics.rs` reads it (absent stays absent, 0 and above 65535 are
  refused), and the profile page shows it in the semantics row. The other profiles declare none and
  their pages do not change.
- `profiles/ntp/profile.yaml` gains one layer, `udp`: port 123 (Figure 6, line 906), unit
  `datagrams`, delivery `{ordered: false, reliable: false, duplicates: true}`, direction
  `half-duplex`, multiplexing `single`, exchange `request-response`, correlation `by-id`,
  lifetime `association`, roles `[client, server]` (design section 6.2, NTP row).
- The lowered model `rfc5905-ntp-exchange`, machine `loss: true`, capacity 3, bounds
  `{max_steps: 64, max_states: 2048, max_time_ms: 60000}`: messages `request {xmt: Integer}` and
  `response {org: Integer}`. Client states `idle`, `first`, `second`, `ended`; local field
  `xmt: Integer`, initial 0; timer `poll`; zero-field input `mobilize`. Transitions: `mobilize`
  idle to first sets `xmt` 1, sends `request {xmt: 1}`, arms `poll` fixed 16000 ms (2^MINPOLL
  s, Figure 6 line 909); on `poll` first to second sets `xmt` 2, sends `request {xmt: 2}`, arms
  `poll` fixed 16000; on `poll` second to ended; in `first` and in `second`, a response guarded
  by `{input: org} = {local: xmt}` sets `xmt` 0 and emits `valid-packet`; in `first`,
  `second` and `ended` an unguarded discard of a response (the inequality is gap G1; the
  nondeterministic discard over-approximates it). Server states `listening`, `replayed`; on a
  request, from either state, send `response` with `org` copied from the request's `xmt`; input
  `replay {org: Integer}` from `listening` only sends a response with that `org` and goes to
  `replayed` (a replayed or duplicated server packet; taken at most once per run, so exploration
  has no input cycle). Property `one-valid-packet-per-poll`: `event_count` of `valid-packet` at
  the client, max 2. Every transition name leads with the requirement ids it implements.
- `profiles/ntp/witnesses/exchange.inputs.json`: server `replay` with `org` 1.
- Probe, 2026-11-05, ess 0.86.0, hand-written model of this shape: correct model `passed` (570
  states); without the `xmt` reset `failed`, `one-valid-packet-per-poll` (330 states); without
  the guard `failed` (387 states). The story's tests reproduce the two mutant verdicts from the
  lowered overlay (`crates/rfcspec-ess` mutant tests, as the dns query mutant test does).
- The overlay accepts these requirements, each `source: prose`, quote verbatim:
  - `R8-BOGUS-1`, MUST, "A packet is bogus if the origin timestamp t1 in the packet does not match
    the xmt state variable T1." (lines 1590-1592), satisfied by the client's guarded accept;
  - `R8-REPLAY-1`, MUST, "To protect against replay of the last transmitted packet, the xmt state
    variable is set to zero immediately after a successful bogus check." (lines 1594-1596),
    satisfied by the same transition;
  - `R8-DUP-1`, MUST, "a packet is duplicate or replay if the transmit timestamp t3 in the packet
    matches the org state variable T3." (lines 1588-1591), status `unsupported: ess-gap G1`
    (accepting requires t3 differing from org, an inequality);
  - `R8-LOSS-1`, MUST (a descriptive sentence; the reviewer confirms the level), "It is inherently resistant to lost or duplicate packets." (lines 1434-1435), satisfied
    by the lossy channels and the server `replay` input;
  - `R9.2-FXMIT-1`, MUST, "the server constructs a server (mode 4) packet and returns it to the
    client without retaining state." (lines 1862-1863), satisfied by the server's answer.
  Line numbers are re-read from the locked source when the overlay is written; levels for prose
  are the reviewer's, as in `dns`.
- `rfcspec lower --check` passes for `time`, `websocket` and `dns` with no change.
- `Taskfile.yml` task `ntp` also runs `rfcspec lower --profile profiles/ntp --check` and the
  exploration above; removing the generated model makes `task check` exit 1.
- The profile page lists `exchange.protospec.yaml` under downloads and the requirements in its
  matrix; `rfcspec-docs generate --check` exits 0; `task check` exits 0.

## Scope

`ess/domains/profile.yaml` (`port`), `crates/rfcspec-model/src/semantics.rs`,
`crates/rfcspec-docs` (port in the semantics row), `crates/rfcspec-ess` (mutant tests only),
`crates/rfcspec-cli/tests`, `profiles/ntp/profile.yaml`, `profiles/ntp/overlay.yaml`,
`profiles/ntp/generated/exchange.protospec.yaml`, `profiles/ntp/witnesses/exchange.inputs.json`,
`Taskfile.yml`, `website/docs/catalog/index.md`, `website/docs/catalog/ntp.md`,
`website/static/downloads/ntp/` (regenerated).

Choices taken:

- Timestamps are small `Integer` stand-ins (1 and 2), not 64-bit NTP timestamps: ESS `Integer`
  is signed 64-bit and has no arithmetic (gaps G2, G3). The offset and delay formulas are not
  modelled.
- Two polls, then the association ends: a passing exploration needs every schedule to settle
  (see `story:ntp-poll-machine`). The poll timer here is fixed; backoff belongs to the poll
  machine.
- One association and one server: more associations are participant families (gap G4).
- Server `xmt`/`rec` and the duplicate check on them are not modelled (R8-DUP-1 unsupported).

Depends on `story:ntp-header-codec` (both edit `profiles/ntp/overlay.yaml`, `Taskfile.yml`,
`crates/rfcspec-cli/tests` and the catalogue pages) and on `story:ntp-timer-model` (machine
`loss`).
