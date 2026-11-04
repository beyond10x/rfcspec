---
format: aep.planning-md/3
id: decision-blocker:catalog-conformance-verdict
kind: decision-blocker
status: cleared
title: Nobody has decided which implementation and which recorded run back the conformance verdict a catalogue entry shows
relations:
- blocks: epic:public-protocol-catalog
revision: 4
transitions:
- {from: "open", to: "cleared", at: "2026-11-04T23:31:41Z", actor: "human:timo", revision: 4}
---
## Question

The epic asks every catalogue entry to show a "conformance verdict". Design section 11 defines a
verdict for "implementation X conforms to profile P": it needs an implementation and a run
against it. `rfcspec verify` probes a live target, and no gate may reach the network
(`AGENTS.md`, Rules), so `task check` cannot produce it. Nothing records a run today: the
overlay statuses carry no evidence, though `ess/domains/profile.yaml` gives
`rfcspec.profile.Status` an optional `evidence` list.

Which implementation does the catalogue verdict speak for (the generated crate, a reference
server, any recorded target), and what checked-in record backs it?

## Options seen

- A: a verdict computed offline from the overlay alone (gap, deviation, inconclusive for
  satisfied without evidence), labelled as a specification verdict, not an implementation one.
- B: a recorded `rfcspec verify` report checked into the profile (a new S7 artifact), read by
  `rfcspec-docs`.
- C: no verdict column until S7 records exist.

## What it stops

The verdict column of the index and page. `story:catalog-index` and `story:catalog-profile-page`
are drafted without it.

## Decision

Timo delegate, 2026-11-05, relayed by the coordinator: option B. A profile verdict is the latest
committed recorded run. `rfcspec verify ... --report profiles/<name>/runs/<target-slug>.json`
writes a deterministic report: the target as given, UTC time, rfcspec version, overlay digest, one
row per requirement with its verdict, and the overall verdict. The index and the page show the
newest report overall verdict, target and date, or "no recorded run". `task check` fails when a
committed report overlay digest differs from the current overlay (a stale run). Implemented by
`story:catalog-recorded-runs`.
