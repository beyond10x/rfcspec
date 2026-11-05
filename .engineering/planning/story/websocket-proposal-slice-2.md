---
format: aep.planning-md/3
id: story:websocket-proposal-slice-2
kind: story
status: implemented
title: Propose and review statuses for the remaining unreviewed RFC 6455 requirements
relations:
- decomposes: epic:ai-proposals
scope:
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/proposals
- confidence: cited
  path: website/docs/catalog
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T12:01:18Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T12:01:18Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-11-05T14:08:38Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Every RFC 6455 catalogue requirement that the websocket overlay leaves without a status or a decision gets one recorded `status` proposal from the local model (`local/qwen3:14b`), and each proposal is settled in the overlay by `rfcspec review` (accept, edit, or reject with its reason). The `websocket` gate step replays every recording byte for byte.

## Acceptance

- `rfcspec propose --profile profiles/websocket --replay` exits 0 over every recorded proposal.
- `rfcspec check --profile profiles/websocket` exits 0, and the overlay holds a decision for every recorded proposal.
- The count of unreviewed requirements in the websocket catalogue page is 0, or each remaining one is named in the commit with the reason it has no proposal.
- `task check` exits 0.

## Scope

- `profiles/websocket/proposals/` (new recordings)
- `profiles/websocket/overlay.yaml` (decisions and entries)
- generated catalogue pages under `website/docs/catalog/` and `profiles/websocket/generated/` when lowering changes
