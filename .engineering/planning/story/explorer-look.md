---
format: aep.planning-md/3
id: story:explorer-look
kind: story
status: active
title: The RFC explorer takes the documentation site's look and feel
relations:
- decomposes: epic:rfc-explorer
scope:
- confidence: cited
  path: explorer/src/App.vue
- confidence: cited
  path: explorer/src/components
- confidence: cited
  path: explorer/src/style.css
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T21:53:33Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-05T23:08:42Z", actor: "human:timo", revision: 6}
---
## Story

The RFC explorer takes the look of the RFCspec documentation site: the same colours, font, header and footer. Today it is plain black on white (`explorer/src/style.css`).

Operator request, 2026-11-05: "when this deploys later to pub docs, Yeah, then also make it the same look and feel. I mean, like this, it's a bit like pre-2000s, no?" and "RFC spec already has some like look and feel, right? But this Explorer you showed is just white and black, and so just take that."

## Acceptance

- `/rfcspec/explorer/?rfc=6455` and `/rfcspec/explorer/` in the built site use the documentation site's colours, font, header and footer.
- `task explorer` and `task website` exit 0.

## Scope

- `explorer/src/style.css`, `explorer/src/App.vue`, `explorer/src/components/*.vue`
