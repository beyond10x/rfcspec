---
format: aep.planning-md/3
id: review-result:ntp-catalogue-adversary-1
kind: review-result
status: active
title: Adversary on story:ntp-source-catalogue, pass 1
relations:
- reviews: story:ntp-source-catalogue
revision: 1
---
unit: story:ntp-source-catalogue, uncommitted working tree on top of ec76054 in the ntp-source-catalogue worktree
verdict: CONFIRMED (judgement findings only, no red case)
cases: executed 353→353, red 0
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: 2 paths, both deleted
needs-coordinator: whether R9.2-2 and R9.2-3 should be `unsupported` rather than `not-applicable` (finding 1)

I did not break this change with a test. Every gate step you named passes, and so does the rest of `task check` that the diff touches. What I found is three judgement points: one on which status two requirements get, and two on how the story and a test name describe the work.

1. `git --no-pager diff --stat`: the same 10 files and 370 insertions / 26 deletions I was handed. I added nothing, so there are no test files of mine and no non-test paths.

2. Cases added: none. I could not find a defect that a test would show failing. The readings I took against the locked text:
   - **Uppercase keywords:** the RFC has 19, and they fold into exactly the 13 requirements. The first keyword in a sentence wins: R2-3 is MAY with a MUST after it, and R7.4-5 is MUST NOT with a MUST after it. The totals are 6 MUST, 1 MUST NOT, 3 SHOULD, 3 MAY.
   - **Furniture lines:** 110 + 110 + 109 = 329 lines. Every form feed is on a line of its own, so no line is counted twice.
   - **Overlay quotes:** all three match lines 1889-1890, 1924-1925 and 1926-1927 word for word. `rfcspec check` also verifies them (`crates/rfcspec-extract/src/check.rs:592`).

3. Suite run, after the probes above: `cargo test --workspace --locked` exited 0, with 353 passed, 0 failed and 0 ignored. The other steps:
   - `rfcspec check --all profiles` exits 0 and lists 4 profiles.
   - `rfcspec bulk check --summary bulk/rfc-summary.jsonl --profiles profiles` holds.
   - `rfcspec-docs generate --check` reports 32 generated files current.
   - `cargo fmt --all --check` exits 0.
   - `cargo clippy --workspace --all-targets --locked -- -D warnings` exits 0.
   - `aep plan artifact validate` reports valid.
   - `rfcspec lower --profile profiles/ntp --check` exits 0.
   - `rfcspec generate --profile profiles/ntp --check` exits 1 ("needs exactly one layout"). It is not a gate step and needs a layout, which is the next story's work, so this is not a finding.

4. Judgement findings (tree as above):
   - **`profiles/ntp/overlay.yaml:16,22` (CONFIRMED, introduced, warning).** R9.2-2 and R9.2-3 are marked `not-applicable` because "the profile models client/server mode only".
     - The websocket overlay has a recorded rule: "not-applicable needs a reason the requirement cannot arise" (`profiles/websocket/overlay.yaml:846`). It also marks a feature the model leaves out as `unsupported` (line 54).
     - A NEWBC packet can arrive at a real NTPv4 implementation. Broadcast mode is only out of the epic's first slice ("symmetric, broadcast and manycast modes" under Out of scope), so it is not something that cannot arise.
     - R9.2-2 (Autokey) rests on the same reason as R9.2-1, which is that authentication is not modelled. R9.2-1 got `unsupported` and R9.2-2 got `not-applicable`.
     - `profile.yaml` declares no modes, so nothing in the profile backs "client/server only".
     - **Effect:** the catalogue's "Unsupported, no ESS gap" table lists only R9.2-1, and the broadcast limitation disappears from it.
     - **Suggested fix:** mark both `unsupported`, or record why the websocket rule does not apply here.
     - **What reaches it:** the generated `website/docs/catalog/ntp.md` and the index row ("0 | 1 | 2").
   - **`.engineering/planning/story/ntp-source-catalogue.md:43` (CONFIRMED, pre-existing, note).** The story asks for two things that cannot both hold.
     - It wants "an empty reviewed overlay ... no requirements" and also wants `rfcspec check --all profiles` to exit 0.
     - `scope()` in `crates/rfcspec-cli/src/checks.rs:92-117` requires a status for every in-scope requirement at every level.
     - So the three overlay entries are forced, and the catalogue shows 3 reviewed where the story says "no reviewed requirements". The story text needs correcting, not the code.
   - **`crates/rfcspec-cli/tests/ntp.rs:309` (CONFIRMED, introduced, note).** The test is named `..._with_no_reviewed_requirement_...` but checks for a row with 3 reviewed requirements (1 unsupported, 2 not applicable). The name will mislead the next reader.
   - **Outside the story's scope:** AGENTS.md's "What exists" table has no `profiles/ntp/` row. I did not raise this as a finding.

5. What I attacked and could not break:
   - **Reader tests:** the expected heading list is written out as literals. The page numbers 1 to 110 make sure the footer count cannot pass for the wrong reason.
   - **Extractor tests:** they check that every requirement id starts with its section, and they pin the 13 ids and the three in-scope spans.
   - **Prose candidates:** only lowercase must, should and may make a candidate. The ten listed are all of them in the scoped sections.
   - **Fixture index entry for RFC 5905:** title, status, stream, area, date, obsoletes and updated-by match the committed summary row, which was built from the real index.
   - **Bulk totals:** they are the old totals plus 6, 1, 3, 0 and 3.
   - **Catalogue counts:** 3 reviewed and 10 unreviewed (6 MUST, 1 MUST NOT, 2 SHOULD, 1 MAY) agree with the overlay.
   - **time, websocket and dns profiles:** check, lower and generate with `--check` pass unchanged.

6. Paths written outside the worktree, both deleted afterwards: two scratch files under the session's temporary directory (`adv-ntp-extract.json`, `adv-ntp-test.log`).

   Inside the worktree I created and then removed `target/.keeper-hold`; the build output stays in the tree's own `target/`. I did not take a worktree lease.

```findings
- file: profiles/ntp/overlay.yaml
  line: 16
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: R9.2-2 and R9.2-3 are not-applicable because broadcast mode is out of the slice, but broadcast can arise for an NTPv4 implementation; the websocket review rule makes a left-out feature unsupported, and R9.2-1 with the same authentication reason as R9.2-2 is unsupported.
- file: .engineering/planning/story/ntp-source-catalogue.md
  line: 43
  category: acceptance
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: The story asks for an overlay with no requirements and also for check to exit 0, but checks.rs scope() requires a status for every in-scope requirement, so the three entries and the "3 reviewed" catalogue row are forced and the story text is what is wrong.
- file: crates/rfcspec-cli/tests/ntp.rs
  line: 309
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: The test name says no reviewed requirement while its assertion checks for a row with three reviewed requirements.
```
