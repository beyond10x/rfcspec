---
format: aep.planning-md/3
id: epic:ai-proposals
kind: epic
status: active
title: AI-proposed overlay entries with cited spans (stage S3)
relations:
- depends_on: epic:websocket-profile
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:47Z", actor: "human:timo", revision: 3}
---
## Outcome

Timo, 2026-11-05: approved. Stage S3 of the design (`docs/design/rfcspec.md` §9, D-list on AI
proposals): a model proposes overlay entries (status, level for prose requirements, machine and
layout elements) for unreviewed requirements, each with cited spans that `rfcspec check` verifies
verbatim, and a reviewer accepts, edits or rejects each proposal into the overlay. Review stops
being the bottleneck: RFC 6455 alone has 139 unreviewed requirements.

## Constraints

- Running code is Rust; model access through the beyond10x `llm` crates, credentials injected,
  never committed.
- Every proposal is recorded (task, model id, input digest, output, cited spans) under the
  profile, and tests replay recorded proposals offline; no test calls a model.
- A proposal changes nothing until a reviewer decision is recorded (the ESS review domain:
  Proposal, Decision).
- A live proposal run is a try-it-out step for Timo, not part of `task check`.

## Acceptance

With a recorded proposal set for a slice of RFC 6455's unreviewed requirements, `rfcspec propose
--replay` reproduces it byte for byte, every cited span passes `rfcspec check`, `rfcspec review`
applies accept/edit/reject decisions into the overlay, and the catalogue's unreviewed count drops
by the accepted number; `task check` exits 0 offline.
