---
format: aep.planning-md/3
id: approval-record:ntp-profile
kind: approval-record
status: approved
title: 'Fourth profile: NTP (RFC 5905) with association timers'
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-05T12:08:00Z", actor: "human:timo", revision: 2}
---
## Decision

Add a fourth profile: NTP (RFC 5905), with association timers: a protocol with timers and per-peer state, as one more epic.

- Approved by the operator on 2026-11-05, verbatim: "A".
- The options answered: "A | NTP, RFC 5905, with association timers | a protocol with timers and per-peer state | one more epic", "B | HTTP/1.1 framing", "C | none for now".
- Relayed by the session sys-mon under the workspace grant of 2026-11-05.
- Order: the remaining WebSocket proposals first (story:websocket-proposal-slice-2), then the NTP epic, its plan-critic panel and its first wave.
