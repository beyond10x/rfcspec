---
format: aep.planning-md/3
id: story:explorer-local-serve
kind: story
status: draft
title: The built explorer loads under docusaurus serve
revision: 1
---
## Story

The built explorer loads under `npm run serve` (`docusaurus serve`) as it does on GitHub Pages. With `trailingSlash: false`, the local server redirects `/rfcspec/explorer/` to `/rfcspec/explorer`, where the explorer's relative `./assets/…` script resolves to `/rfcspec/assets/…` and returns 404, so the page does not load. GitHub Pages keeps the trailing slash, so production is not affected. Found by review-result:wave6-explorer-look-adversary-1 (pre-existing), 2026-11-06.

## Acceptance

- After `task website`, `npm run serve` in `website/` serves `/rfcspec/explorer/` and `/rfcspec/explorer/?rfc=6455` with the explorer loaded (its script returns 200).
- `task explorer` and `task website` exit 0.

## Scope

- `explorer/vite.config.ts`
- `website/package.json`
