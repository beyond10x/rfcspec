---
format: aep.planning-md/3
id: review-result:ntp-scope-round-1
kind: review-result
status: active
title: NTP plan critic (scope), round 1
relations:
- reviews: epic:ntp-profile
- reviews: story:ntp-source-catalogue
- reviews: story:ntp-header-codec
- reviews: story:ntp-timer-model
- reviews: story:ntp-exchange-machine
- reviews: story:ntp-poll-machine
revision: 1
---
needs-revision
story:ntp-poll-machine — the epic promises the design's gap table "names the ESS gaps the slice hit", but the gap-table edit (G2 and G7 gain NTP rows, new G11 to G13) leaves out G1, which the exchange story hits for the origin/duplicate inequality (R8-DUP-1 `unsupported: ess-gap G1`). The G1 row in the design lists only WebSocket, so nothing in the set claims adding NTP to it. Fix: add G1 to the poll story's gap-table bullet (the only story that edits the table), or say in a body that the omission is deliberate — `.engineering/planning/epic/ntp-profile.md:40-41` ("the design's gap table names the ESS gaps the slice hit"); `.engineering/planning/story/ntp-poll-machine.md:109`; `docs/design/rfcspec.md:308`

**What you read.** I read 6 artifacts in full (the epic and the five stories) with `aep plan artifact show` and `aep plan artifact graph`. I also read the precedent `epic:dns-profile` and its three stories, and the gap table in `docs/design/rfcspec.md`. I extracted 12 promises from the epic: 4 outcome bullets, the ESS-gap recording paragraph, and 7 acceptance clauses, with the `task check` clause counted as shared. All 12 trace to an item, except the part of the gap-table clause covering G1, which is the finding above. No item reaches into the epic's out-of-scope list. No two items claim one outcome. The `arm` backoff and fixed-interval claims, the mutants and the witnesses each have a single owner.

**What I could not establish.**
- Out of my lane (design): the epic cites `approval-record:ntp-profile`, and no such artifact exists in the store. `.engineering/planning/approval-record/` holds only `delete-rfc868-thin-slice.md`.
- Out of my lane (design): the `loss: true` machine flag in `story:ntp-timer-model` has no sentence in the epic. I counted it as enabling work for the udp exchange and did not raise it as reach beyond the parent.
- Not a finding: the `AGENTS.md` edit in `story:ntp-poll-machine` matches the dns precedent (`story:dns-query-machine`).

```findings
- file: .engineering/planning/story/ntp-poll-machine.md
  line: 109
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the epic promises the design's gap table names the ESS gaps the slice hit, but the gap-table edit adds NTP to G2 and G7 and new G11 to G13 only; G1 (the inequality behind R8-DUP-1 in story:ntp-exchange-machine) is hit by the slice and no story adds it to the table"
```
