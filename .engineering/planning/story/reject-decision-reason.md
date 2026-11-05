---
format: aep.planning-md/3
id: story:reject-decision-reason
kind: story
status: draft
title: Store the reason of a reject decision in the overlay
relations:
- decomposes: epic:ai-proposals
revision: 1
---
## Story

`rfcspec review --proposal <id> reject --reason <text>` takes a reason and prints it, but the decision written to the overlay does not hold it, because `rfcspec.review.Decision` has no reason field. Add `reason` to the ESS `Decision` and `RecordDecision` (required for `reject`, absent otherwise), store it in `Overlay.decisions`, and show it on the profile page.

## Acceptance

`ess specify validate --path ess` exits 0 with the field; a reject decision written by `rfcspec review` reads back with its reason; `rfcspec check` refuses a reject decision with an empty reason; `task check` exits 0.

Found by story:proposal-review (implementation report, 2026-11-05).
