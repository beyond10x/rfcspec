---
format: aep.planning-md/3
id: review-result:websocket-acceptance-round-1
kind: review-result
status: active
title: acceptance critic, round 1, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
needs-revision

- story:websocket-source-catalogue — the acceptance gives the 1010 sentence "The list of extensions that are needed SHOULD appear in the /reason/ part of the Close frame." the span `2571-2583`, but in `.engineering/drafts/rfc6455.txt` line 2571 is "more extension, but the server didn't return them in the response" and the sentence begins mid-line 2572, so a correct extractor fails the stated span; it should read 2572-2583 — `.engineering/planning/story/websocket-source-catalogue.md:61`
- story:websocket-review-matrix — the acceptance states the expected `unreviewed` count for `websocket` as "the other 149 keyword occurrences" while the index and page count requirements, so the number a unit test must assert is not stated; nothing observable shows that the non-MUST requirements in scope (36 of the 65 occurrences) have a status, because the check enforces only MUST-level — `.engineering/planning/story/websocket-review-matrix.md:64` (also line 43, "every requirement of sections 5.2, 5.5 and 7 has a status")

**What I read:** 5 artifacts (the epic and the 4 stories), whole bodies, via `aep plan artifact show <id>` for each. I checked against `.engineering/drafts/rfc6455.txt`:
- the SHA-256, the 3979 lines and the furniture counts (71 page footers, 71 form feeds, 70 headers);
- the 88 headings (14 top-level);
- the 140/26/30/5/48/2/1/1/2 whole-file keyword counts;
- the 113/25/24/4/47/1 total of 214;
- the per-section counts for 5.2 (10 MUST, 1 MAY), 5.5 (6 MUST, 3 MUST NOT, 3 SHOULD, 9 MAY) and 7 (6 MUST, 4 MUST NOT, 11 SHOULD, 3 SHOULD NOT, 9 MAY), which sum to 65 (29 MUST or MUST NOT), leaving 149;
- the figure bounds and the five ABNF-comment lines;
- the eight octet-example lines in 5.7;
- the quoted requirement line spans.
All of these hold except the 2571 span above.

**What I could not establish:**
- I did not run `aep plan artifact kinds` or `aep plan artifact lifecycle`.
- I did not check the tree's code or tasks that the acceptances name (`rfcspec-*` crates, `Taskfile.yml`), because only the drafts and store were in scope.
- I did not confirm that no keyword occurs outside paragraphs and ABNF comments, for example in the table of contents, so the "exactly 5 `abnf-comment`" and 214 figures hold only for what I checked.
- I did not flag the multi-outcome shape of the acceptances (a headline sentence plus 5 to 9 independent bullets in each story, and four outcomes in one sentence in the epic). I treated it as a style the drafter chose, not a defect. The coordinator may disagree.
- Out of my lane, not counted in the verdict:
  - The close-machine and frame-codec acceptances cite `crates/rfcspec-cli/src/checks.rs:68` and `crates/rfcspec-gen/src/lib.rs:15`, which belong to design or scope.
  - The four stories form a strict dependency chain, so none can run in parallel (parallel-safety).

```findings
- file: .engineering/planning/story/websocket-source-catalogue.md
  line: 61
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance gives the section 7.4.1 status-1010 sentence the span 2571-2583, but the sentence starts mid-line 2572 of rfc6455.txt (line 2571 is 'more extension, but the server didn't return them in the response'), so a correct extractor fails the stated span; it should read 2572-2583"
- file: .engineering/planning/story/websocket-review-matrix.md
  line: 64
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the expected websocket unreviewed count is stated as '149 keyword occurrences' though the index and page count requirements, so the asserted number is undetermined, and nothing observable shows that non-MUST in-scope requirements have a status (the check enforces only MUST-level) although the headline says every requirement of sections 5.2, 5.5 and 7 has one"
```
