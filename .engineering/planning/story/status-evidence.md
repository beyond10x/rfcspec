---
format: aep.planning-md/3
id: story:status-evidence
kind: story
status: draft
title: Record evidence for every satisfied requirement status in the profile overlays
relations:
- decomposes: epic:rfc-explorer
revision: 1
---
## Story

A satisfied requirement status in a profile overlay names its evidence: the conformance scenarios, protospec exploration or test that shows the element satisfies the requirement. Today no overlay records any (`grep evidence profiles/*/overlay.yaml` finds none), so 46 satisfied status rows across the four profiles have empty evidence, a state `rfcspec.profile.RecordSatisfiedStatus` refuses with `EvidenceMissing` (`ess/domains/profile.yaml`). Found by adversary pass 1 on `story:explorer-profile-layer` (`review-result:explorer-profile-adversary-1`).

## Acceptance

- Every satisfied status in `profiles/*/overlay.yaml` carries at least one evidence entry that names an existing test, scenario or witness.
- The profile gate refuses a satisfied status without evidence.
- The profile pages and the explorer's reviewed layer show the evidence.

## Open questions

- What counts as evidence for a requirement satisfied by a codec layout rather than a machine: the codec round-trip test, or a conformance scenario.
