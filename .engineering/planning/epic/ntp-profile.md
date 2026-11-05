---
format: aep.planning-md/3
id: epic:ntp-profile
kind: epic
status: active
title: 'NTP (RFC 5905) as the fourth profile: a UDP association with poll timers'
summary: Header codec, on-wire exchange and poll-process machines for RFC 5905 sections 7.2, 7.3, 8, 9.2 and 13
relations:
- depends_on: epic:dns-profile
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T13:50:22Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T13:50:22Z", actor: "human:timo", revision: 3}
---
## Outcome

Operator approval 2026-11-05 (`approval-record:ntp-profile`). A fourth profile that exercises
what `time`, `websocket` and `dns` do not: a long-lived UDP association driven by timers, with
per-peer state (design section 6.2, NTP row: `lifetime association`, correlation by the origin
timestamp). RFC 5905 (NTPv4), first slice:

- the packet header (section 7.3, Figure 8, the basic 48-octet header up to the Transmit
  Timestamp) as a layout and a generated codec crate `rfc5905-ntp`;
- the on-wire exchange (section 8): a client response is accepted only when its origin
  timestamp equals the client's `xmt`, and `xmt` is zeroed after a successful check;
- the association's poll process (sections 9.2 and 13): a poll timer at 2^MINPOLL s, the reach
  register deciding reachable or unreachable, and the backoff that doubles the poll interval up to
  2^MAXPOLL s while the server stays unreachable, reset when it is heard again;
- the semantics of the `udp` layer, port 123 (section 7.2, Figure 6).

ESS 0.86.0 expresses the poll timer (`arm` fixed) and the doubling backoff (`arm` with
`backoff {timer, factor, ceiling_ms}`), not a duration read from a field (G7), not arithmetic on
the reach register or the unreach counter (G2), and no passing exploration of a timer that is
re-armed for ever. The slice scopes around those and records them as ESS gaps; it does not work
around them.

## Acceptance

`profiles/ntp` locked to RFC 5905; the header layout generates a codec tested with the version
range section 9.2 states and with signed Poll and Precision; the `exchange` and `poll` machines
pass `ess protocol explore` with their witnesses, and the named mutants (no origin guard, no `xmt`
reset, backoff replaced by a fixed interval) are reported; the semantics row is `udp`, port 123,
`correlation by-id`, `lifetime association`; the catalogue index lists `ntp`; the design's gap
table names the ESS gaps the slice hit; `task check` exits 0.

## Out of scope

Extension fields and the MAC (sections 7.5, 7.3 trailing fields), Kiss-o'-Death (7.4), the clock
filter, system, clock discipline and clock-adjust processes (10 to 12), symmetric, broadcast and
manycast modes, burst and iburst, mobilisation of more than one association (gap G4), SNTP (14),
the appendix skeleton, and a live verify probe against a server.
