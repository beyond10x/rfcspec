---
format: aep.planning-md/3
id: epic:dns-profile
kind: epic
status: implemented
title: 'DNS (RFC 1035) as the third profile: correlation by id over UDP'
relations:
- depends_on: epic:websocket-profile
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:46Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-05T11:29:54Z", actor: "human:timo", revision: 4}
---
## Outcome

Timo, 2026-11-05: approved. A third profile that exercises what time and websocket do not:
correlation by id and request-response over a datagram transport (design §6.2: DNS RFC 1035 and
NTP RFC 5905 rows, `correlation by-id`). Decision taken: DNS, RFC 1035, message header (§4.1.1) and
query/response over UDP, because its header is a fixed 12-octet layout the generalised generator
already handles and its id correlation is the axis to prove; NTP's association timers stay later.

## Acceptance

`profiles/dns` locked to RFC 1035; the header layout generates a codec tested with header values
the RFC states; a UDP query/response machine where the client matches a response by id passes
`ess protocol explore`, and a mutant that accepts a response with a different id is reported; the
semantics row is `udp` with `correlation by-id`; the catalogue index lists `dns`; `task check`
exits 0.
