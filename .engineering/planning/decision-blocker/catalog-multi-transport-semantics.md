---
format: aep.planning-md/3
id: decision-blocker:catalog-multi-transport-semantics
kind: decision-blocker
status: cleared
title: Nobody has decided how a profile with several transports (RFC 868 over TCP and UDP) declares its communication semantics
relations:
- blocks: epic:public-protocol-catalog
revision: 4
transitions:
- {from: "open", to: "cleared", at: "2026-11-04T23:31:41Z", actor: "human:timo", revision: 4}
---
## Question

The epic asks each profile page to show its communication semantics (design section 6).
`ess/domains/profile.yaml` gives `rfcspec.profile.Profile` a `semantics` list with one
`LayerSemantics` per layer (invariant: distinct by layer). RFC 868 runs over TCP (unit bytes,
lifetime connection) and over UDP (unit datagrams, connectionless), with one machine each in
`profiles/time/overlay.yaml`; one entry per layer cannot hold both. Design section 6.2 writes
DNS the same way, as free text: "connectionless over UDP; connection over TCP".
`profiles/time/profile.yaml` declares no semantics, and `rfcspec-model` `Profile` has no field
for them.

How does a profile with several transports declare its semantics?

## Options seen

- A: semantics per machine (or per transport) instead of per profile; changes the ESS domain.
- B: one profile per transport (`time-tcp`, `time-udp`); changes the profile layout.
- C: a list of semantics variants per layer keyed by transport; changes the ESS invariant.

## What it stops

The semantics section of the profile page and any `profile.yaml` semantics field. No story is
drafted for it.

## Decision

Timo delegate, 2026-11-05, relayed by the coordinator: each transport is its own layer.
`profile.yaml` gains `semantics:`, a list with one entry per layer name (`tcp` and `udp` for
`time`), each with the design section 6 axes (unit, delivery, direction, multiplexing, exchange,
correlation, lifetime, layering, roles), matching the ESS `rfcspec.profile.LayerSemantics`
struct; layer names are distinct. Implemented by `story:catalog-semantics`.
