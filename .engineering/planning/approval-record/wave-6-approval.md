---
format: aep.planning-md/3
id: approval-record:wave-6-approval
kind: approval-record
status: approved
title: Wave 6 approved and its four stories accepted, on sys-mon's word for the operator
relations:
- decides: story:abnf-core
- decides: story:catalogue-grammars
- decides: story:fixed-arm-bounds-check
- decides: story:explorer-look
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-11-05T23:08:41Z", actor: "human:timo", revision: 3}
---
## Decision

Wave 6 of the plan of 2026-11-05 (`.engineering/waves/plan-2026-11-05.md`, `.engineering/waves/wave-6.md`) is approved and its four stories are accepted.

The approval came from session `sys-mon` on 2026-11-06. Under the workspace instructions, a `sys-mon` message quoting the operator with a date counts as the operator's word for class C, and a wave is class C. It quoted the operator, 2026-11-04T23:24:57Z, verbatim: "keep everyone in here ... working all the time ... you have authority to decide stuff" (shortened here).

Its conditions:

- the plan-critic panel approves the proposal;
- the units stay as named;
- no release and no tag without a separate word.

The panel's verdicts are the review-results `wave6-acceptance-round-1`, `wave6-acceptance-round-2`, `wave6-parallel-round-1`, `wave6-design-round-1` and `wave6-scope-round-1`. Design, parallel safety and scope approved in round 1. Acceptance asked for revisions in round 1 (fixed in abnf-core revision 6, catalogue-grammars revision 10, fixed-arm-bounds-check revision 6) and for one wording fix in round 2 (fixed in catalogue-grammars revision 12).

## Stories accepted

- story:abnf-core
- story:catalogue-grammars
- story:fixed-arm-bounds-check
- story:explorer-look

## Commits authorised

All are made as `b10x-bot[bot]`:

- the opening commit with the wave pages;
- 4 unit commits;
- 4 merges into `wave-6`;
- 1 closing store commit;
- 1 push of `wave-6` and 1 PR, merged into `main` after the gate is green.

No tag and no release.
