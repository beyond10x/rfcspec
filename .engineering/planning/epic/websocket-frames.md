---
format: aep.planning-md/3
id: epic:websocket-frames
kind: epic
status: active
title: WebSocket Ping/Pong and data-frame machines
relations:
- depends_on: epic:websocket-profile
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:46Z", actor: "human:timo", revision: 3}
---
## Outcome

Timo, 2026-11-05: approved. `profiles/websocket` models Ping/Pong (§5.5.2, §5.5.3) and data frames
with fragmentation (§5.4, §5.6) as ess-protospec/1 machines beside the closing handshake, so
requirements now `unsupported` because "Ping and Pong frames are not modelled" or "data frames are
not modelled" become satisfied by a machine element or carry an ESS gap.

## Acceptance

`ess protocol explore` prints `"verdict": "passed"` on each new model with committed witnesses; a
mutant per model (a Pong not echoing the Ping's application data; a control frame interleaved
wrongly into a fragmented message) is reported by explore; every requirement of §5.4, §5.5.2,
§5.5.3 and §5.6 has a status; the catalogue's satisfied count for `websocket` rises by the number
of requirements those machines satisfy; `task check` exits 0.
