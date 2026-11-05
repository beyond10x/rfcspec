---
format: aep.planning-md/3
id: story:ntp-poll-machine
kind: story
status: implemented
title: 'Review and lower the RFC 5905 poll process: poll timer, reach, capped backoff, and record the ESS timer gaps'
relations:
- decomposes: epic:ntp-profile
- depends_on: story:ntp-exchange-machine
- depends_on: story:ntp-timer-model
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
  path: docs/design/rfcspec.md
- confidence: cited
  path: profiles/ntp/generated/poll.protospec.yaml
- confidence: cited
  path: profiles/ntp/overlay.yaml
- confidence: cited
  path: profiles/ntp/witnesses/poll.inputs.json
- confidence: cited
  path: website/docs
- confidence: cited
  path: website/static/downloads/ntp
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T13:50:23Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T14:42:02Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T15:33:40Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

The `ntp` profile gets a reviewed machine `poll` for one client association's poll process
(sections 9.2 and 13): a poll timer at 2^MINPOLL s while the server is reachable or not yet
backed off, the reach register deciding reachable or unreachable, and the backoff that doubles the
poll interval up to 2^MAXPOLL s while the server stays unreachable and resets when a valid packet
arrives. This is the first machine whose timer interval changes at run time, and it is where the
slice meets what ess 0.86.0 cannot express. Those limits become rows in the design's gap table,
with the probes that showed them.

## Outcome

`profiles/ntp/generated/poll.protospec.yaml` is lowered from the reviewed overlay and explored in
`task check`; a mutant that polls at a fixed interval instead of backing off is reported; the
design names the ESS gaps the NTP slice hit, and `AGENTS.md` names four profiles.

## Acceptance

`rfcspec lower --profile profiles/ntp` writes `generated/poll.protospec.yaml`; `ess protocol
explore --path profiles/ntp/generated/poll.protospec.yaml --inputs
profiles/ntp/witnesses/poll.inputs.json` prints `"verdict": "passed"`; the same exploration of a
mutant whose backoff transitions arm `poll` fixed 16000 ms prints `"verdict": "failed"` naming
`backoff-doubles-interval`.

- The lowered model `rfc5905-ntp-poll`, machine `loss: true`, capacity 4, bounds `{max_steps:
  128, max_states: 8192, max_time_ms: 131072000}` (ESS requires a backoff ceiling within
  `max_time_ms`): messages `request {}` and `response {}`. Client states `idle`,
  `unreachable`, `reachable`, `backoff`, `ended`; timers `poll` and `horizon`; zero-field
  input `mobilize`. Transitions:
  - `mobilize` idle to unreachable (reach is 0 at mobilisation): send `request`, arm `poll`
    fixed 16000 (2^MINPOLL s, Figure 6 line 909), arm `horizon` fixed 48000;
  - on `poll`: reachable to reachable, reachable to unreachable (the register has emptied),
    unreachable to unreachable (unreach below UNREACH): send `request`, arm `poll` fixed 16000;
    unreachable to backoff, and backoff to backoff: send `request`, emit `backoff-poll` (from
    backoff only), arm `poll` with backoff factor 2, ceiling 131072000 (2^MAXPOLL s, Figure 6
    line 910);
  - on `response`: unreachable or reachable to reachable; backoff to reachable re-arming `poll`
    fixed 16000;
  - on `horizon`: unreachable, reachable or backoff to ended, cancelling `poll`; in `ended`
    a response is discarded.
  Server state `listening`; on a request, send `response`. Property
  `backoff-doubles-interval`: `event_count` of `backoff-poll` at the client, max 1 (within the
  48 s horizon a backed-off client polls at 16 s, then not before 48 s; a fixed interval polls at
  32 s and 48 s). Every transition name leads with the requirement ids it implements; the
  `horizon` transitions implement none and are labelled `model-horizon`.
- `profiles/ntp/witnesses/poll.inputs.json`: `[]` (the only input has no fields).
- Probe, 2026-11-05, ess 0.86.0, hand-written model of this shape without the property and emit:
  `passed`, 2743 states; with capacity 2 it is inconclusive ("channel server-to-client capacity
  exhausted"); with a 96 s horizon and capacity 7 it did not finish in 250 s. The bounds and
  horizon above are the probe's; the story may raise `max_states` but does not lengthen the
  horizon.
- A lowering test asserts that the backoff transitions lower to `after: {kind: backoff, timer:
  poll, factor: 2, ceiling_ms: 131072000}` and the others to `{kind: fixed, millis: 16000}`.
- The overlay accepts these requirements, each `source: prose`, quote verbatim, lines re-read
  from the locked source when written:
  - `R13-BACKOFF-1`, MUST, "Thereafter for each poll hpoll is increased by one, which doubles the
    poll interval up to the maximum MAXPOLL" (lines 2948-2950), satisfied by the backoff
    transitions;
  - `R13-RESET-1`, MUST, "When the server again becomes reachable, unreach is set to zero, hpoll
    is reset to the tc system variable" (lines 2951-2952), status `unsupported: ess-gap G7` (tc
    is a system variable; the model resets to 2^MINPOLL s);
  - `R9.2-REACH-1`, MUST, "If the register contains any nonzero bits, the server is considered
    reachable; otherwise, it is unreachable." (lines 2012-2014), satisfied by the
    reachable/unreachable transitions, which over-approximate the register;
  - `R9.2-REACH-2`, MUST, "The register is shifted left by one bit when a packet is sent and the
    rightmost bit is set to zero." (lines 2010-2011), status `unsupported: ess-gap G2` (no shift
    or count in `set`; the 8 polls are abstracted to a nondeterministic step);
  - `R13-UNREACH-1`, MUST, "otherwise, unreach is incremented by one for each poll to the maximum
    UNREACH." (lines 2947-2948), status `unsupported: ess-gap G2`, same reason (24 polls);
  - `R13-RUN-1`, MUST, "It runs continuously, whether or not servers are reachable" (lines
    2833-2834), status `unsupported: ess-gap G11` (the model ends at a 48 s horizon);
  - `R13-UPDATE-1`, MUST, the section 13.2 sentence that sets hpoll to the minimum of ppoll and
    hpoll within MINPOLL and MAXPOLL (lines 2999-3002), status `unsupported: ess-gap G7`.
- The overlay records one finding: Figure 29 sets UNREACH to 24 (line 2871), the appendix skeleton
  to 12 (line 5820).
- Design `docs/design/rfcspec.md` section 7, gap table: G1 adds "NTP origin and duplicate check (RFC 5905 section 8, R8-DUP-1)"; G2 and G7 add "NTP reach register,
  unreach counter, poll exponent (RFC 5905 sections 9.2, 13)"; new rows, each citing the ess
  0.86.0 source and the probe:
  - G11, a timer re-armed on every expiry (a periodic process) cannot pass `ess protocol explore`:
    advancing past `max_time_ms` is a bound error and a dead end with an armed timer is
    inconclusive (`ess-conformance/src/protocol/engine.rs` `successors`, `monitor.rs`); proposal:
    a horizon verdict that passes a schedule cut at `max_time_ms` with no violation;
  - G12, `duplication: true` makes every exploration end at the step bound (a message can be
    duplicated again after each delivery); proposal: a duplication budget per transmission;
  - G13, a send to a full channel refuses execution even where the layer's delivery is lossy;
    proposal: `on_full: drop` for channels declared `loss: true`.
  `rfcspec-docs generate` regenerates the design page; `rfcspec-docs generate --check` exits 0.
- `AGENTS.md` (State, What exists, Gate) names four profiles and the `ntp` task.
- `Taskfile.yml` task `ntp` also runs the poll exploration; removing the generated model makes
  `task check` exit 1. `rfcspec lower --check` passes for `time`, `websocket`, `dns`. The
  profile page lists `poll.protospec.yaml` and the requirements; `task check` exits 0.

## Scope

`crates/rfcspec-ess` (mutant and lowering tests only), `crates/rfcspec-cli/tests`,
`profiles/ntp/overlay.yaml`, `profiles/ntp/generated/poll.protospec.yaml`,
`profiles/ntp/witnesses/poll.inputs.json`, `Taskfile.yml`, `docs/design/rfcspec.md`,
`AGENTS.md`, `website/docs/` (regenerated design and catalogue pages),
`website/static/downloads/ntp/` (regenerated).

Choices taken:

- The reach register and unreach counter are abstracted to nondeterministic transitions rather
  than unrolled: probed on 2026-11-05, an exact unrolling with literal-guarded transitions grew to
  14560 states at a 12-poll counter and still exhausted channel capacity; the RFC's 8 + 24 polls
  is out of reach of bounded exploration in ess 0.86.0. RFC constants are not shrunk.
- The `horizon` timer is a model construct, not RFC behaviour; it exists only so that every
  schedule settles (G11), and the overlay says so.
- The 2^MAXPOLL ceiling is checked in the lowered model, not reached by exploration (13
  doublings exceed the horizon).
- One association, client mode only; burst, iburst, manycast and symmetric modes are out.

Depends on `story:ntp-exchange-machine` (both edit `profiles/ntp/overlay.yaml`, `Taskfile.yml`,
`crates/rfcspec-cli/tests` and the catalogue pages) and on `story:ntp-timer-model` (backoff,
`loss`).

