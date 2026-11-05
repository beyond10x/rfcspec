---
format: aep.planning-md/3
id: review-result:websocket-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, round 2, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
approve

**What I read:** 6 artifacts, whole bodies, via `aep plan artifact show` for `epic:websocket-profile`, the four stories and `review-result:websocket-acceptance-round-1`. I did not run `aep plan artifact kinds` or `lifecycle`. I checked the acceptances against `.engineering/drafts/rfc6455.txt` and the tree.

Both round-1 findings are fixed:
- **1010 span:** `story:websocket-source-catalogue` now says 2572-2583. Line 2571 is "more extension, but the server didn't return them…", and the sentence starts mid-line 2572 and ends at line 2583.
- **Unreviewed count:** `story:websocket-review-matrix` (revision 6) now has the unit test compute the expected `unreviewed` count from the `rfcspec extract` output and assert the index cell equals it. It also asserts 0 unreviewed in scope. `rfcspec check` now refuses a missing status at every level, MUST and MAY each tested, so the headline "every requirement of sections 5.2, 5.5 and 7 has a status" is observable.

Checked against the RFC text and all hold:
- SHA-256 and the 3979 lines.
- Furniture counts of 71 page footers, 71 form feeds and 70 header lines.
- Heading bounds at lines 175 and 3856.
- Whole-file keyword counts, which give 224 in all, so 214 outside the boilerplate. The per-keyword splits (113/25/24/4/47/1) match once the 10 boilerplate occurrences are removed.
- Boilerplate at lines 658-660.
- The figure bounds, and the ABNF-comment lines 1409, 1706, 1710, 1714 and 2678.
- The 8 octet-example lines, and the 1005/1006/1015 sentences at lines 2533, 2540 and 2595.
- Every quoted requirement span.
- 65 in-scope occurrences (11 + 21 + 33), 29 of them MUST or MUST NOT, and 149 outside scope.
- Taskfile steps and crates named (`checks.rs`, `catalog.rs`, `fn check` at `main.rs:235`, `ess/domains/*.yaml`) exist in the tree.

**What I could not establish:**
- I did not recount the per-section keyword splits for 5.2, 5.5 and 7. I relied on round 1 and re-verified only their sums against the stories (65, 29, 149).
- I did not confirm that no keyword sits outside paragraphs and ABNF comments, for example in the table of contents. The whole-file total of 224 does hold.
- The stories' multi-bullet acceptances (a headline sentence plus several independent bullets) are a style the drafter chose. I treated them as in round 1, not as a defect.
- Out of my lane, not counted: the strict dependency chain of the four stories (parallel-safety), and the design claims about `crates/rfcspec-cli/src/checks.rs:68` and `crates/rfcspec-gen/src/lib.rs:15` (design).

```findings
[]
```
