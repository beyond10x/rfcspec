---
format: aep.planning-md/3
id: review-result:explorer-profile-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:explorer-profile-layer
relations:
- reviews: story:explorer-profile-layer
revision: 1
---
unit: story:explorer-profile-layer, uncommitted working tree on f524ed6 in the explorer-profile-layer worktree
verdict: NEEDS-CHANGE
cases: executed 56→58 (cargo, rfcspec-docs) and 67→75 (vitest), red 4
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: whether "satisfied with its element and evidence" can be met at all, since the overlay records no evidence

1. **Diff stat.** `git --no-pager diff --stat` is unchanged from the tree I was handed: `23 files changed, 674 insertions(+), 50 deletions(-)`. All of that is the implementor's. I added two untracked test files and nothing else:
   - `?? crates/rfcspec-docs/tests/explorer_profile_adversary.rs`
   - `?? explorer/tests/adversary-profile.test.ts`

2. **Cases added.** Each was run alone before any suite run.

   In `the explorer-profile-layer worktree/crates/rfcspec-docs/tests/explorer_profile_adversary.rs`:
   - `every_satisfied_status_row_names_its_test_evidence` is **RED**. It checks that every row of kind satisfied in the four committed profile JSONs has non-empty `evidence`. The ESS operation `RecordSatisfiedStatus` refuses an empty list with `EvidenceMissing`, and the acceptance says "satisfied with its element and evidence".
     ```
     thread 'every_satisfied_status_row_names_its_test_evidence' panicked at crates/rfcspec-docs/tests/explorer_profile_adversary.rs:35:5:
     46 satisfied status rows without evidence, first ["rfc1035/R-HDR-1", "rfc1035/R-ID-1", "rfc1035/R-MATCH-1"]
     test result: FAILED. 1 passed; 1 failed
     ```
   - `a_row_gaining_an_optional_field_is_drift_the_reread_does_not_see` is green. It is a rewrite of the implementor's drift test: it compares each copy with its view row in `source.schema.json`, property by property including types and `required`. Inside the test I add an optional `superseded_by` to a copy of `Statuses.row`. My comparison reports the drift, but every committed status value still fits the mutated row, so `reread` would stay green.

   In `the explorer-profile-layer worktree/explorer/tests/adversary-profile.test.ts`:
   - The counts lines match the page for dns, ntp, time and websocket: 4 cases, all green.
   - `shows the reviewed layer of a mark whose file loads when an earlier mark has none` is **RED**: `expected [] to have a length of 1 but got +0` (:77).
   - `reports no failure for a mark of a profile without an overlay` is **RED**: the page ends with `The reviewed layer could not be loaded: profile draft: HTTP 404` (:89).
   - `still shows the reviewed layer when the RFC has no shard` is **RED**: `expected [] to have a length of 1 but got +0` (:100).
   - A quote containing `<img onerror>` renders as text: green.

3. **Suite runs**, made after the cases existed.
   - `cargo test --locked -p rfcspec-docs`: lib 40 ok, main 14 ok, `explorer_adversary` 2 ok, `explorer_profile_adversary` 1 passed, 1 failed. EXIT=101.
   - `npx vitest run`: `Test Files 1 failed | 9 passed (10)`, `Tests 3 failed | 72 passed (75)`. EXIT=1.
   - With my file excluded: `Tests 67 passed (67)`.
   - Hygiene: `npm run typecheck` exits 0, clippy `-D warnings` exits 0, and `cargo fmt --check` passes on my file.

4. **Findings.**

| file:line | finding | verdict | origin | what reaches it |
|---|---|---|---|---|
| crates/rfcspec-docs/src/explorer.rs:396 | `evidence` is always absent. 46 published `Statuses` rows are satisfied with no evidence, which the domain's `RecordSatisfiedStatus` refuses. The acceptance's "satisfied with its element and evidence" is not met. The overlay records no evidence, so the fix is either to drop evidence from the acceptance or to source it from runs. | NEEDS-CHANGE | introduced | every committed profile JSON; seen in every `?rfc=` view of a profiled RFC |
| explorer/src/components/RfcView.vue:42 | `ProfileLayer` and the layer error sit inside the `v-else` article. A profiled RFC whose shard is missing, or missing from its shard, loses its reviewed layer silently, even though App fetched it. | INFEASIBLE | introduced | nothing found: all four profiled RFCs have shard rows today |
| explorer/src/App.vue:57 | A 404 on one profile file returns early. Later marks are never loaded, and a profile without an overlay is shown as an error (the profile page says "No reviewed overlay yet"). | INFEASIBLE | introduced | nothing found: no RFC has two marks, and all four marked profiles have overlays. It is reached once a profile is locked before it has an overlay. |
| crates/rfcspec-docs/src/explorer_profile_tests.rs:194 | The copied-row drift test cannot fail when a row gains an optional field. It also never exercises the always-absent fields (`evidence`, `edit`, `errata_*`, `corrected_text`, `confirms`). `types-check` regenerates both sides, so neither the test nor the check catches drift in them. Fix: compare `source.schema.json` definitions, as my case does. | CONFIRMED | introduced | any future change to `rfcspec.profile.Statuses` / `rfcspec.review.Decisions` in `ess/` |
| explorer/src/App.vue:60 | A profile file whose `rfc` differs from the marked RFC is dropped without any message. `ExplorerMachine` is also not read back through any `Machines` row. | INFEASIBLE | introduced | nothing found: each profile locks exactly one RFC |

5. **Attacked and could not break:**
   - I re-read all four overlays independently with yq. Ids, levels, quotes, spans, status kinds and reasons, decision kinds, reasons and proposal ids, machine names, and layout fields with bits and presence all match the JSON. The 67 accept + 18 edit + 29 reject decisions all map to the right ids.
   - All 114 proposal files exist, and the model id is `local/qwen3:14b`.
   - Every quote, element and reason in the JSON appears on its page with the page's cell escaping (395 strings checked, 0 missing), so no extra RFC text is published.
   - The unreviewed and reviewed counts the explorer shows equal what each of the four pages prints.
   - There is no `v-html`. Quotes render as text. Hrefs are prefixed with `https://` or `../`, so they cannot become `javascript:`. Proposal ids are hex.
   - `types-check` diffs both new output directories. docs-check owns `PROFILES_DIR`.
   - There are no `/home/` paths, no comments in the new Taskfile or YAML lines, and no Python.

6. **Paths written outside the worktree:** none. Builds used only this tree's `target/`.

7. ```findings
- file: crates/rfcspec-docs/src/explorer.rs
  line: 396
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: every satisfied Statuses row is published without evidence, a row RecordSatisfiedStatus refuses, and the acceptance's "satisfied with its element and evidence" is unmet
- file: explorer/src/components/RfcView.vue
  line: 42
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the reviewed layer and its error render only inside the v-else article, so a profiled RFC without a shard row silently loses its fetched layer
- file: explorer/src/App.vue
  line: 57
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a 404 on one profile file aborts the remaining marks and reports a profile without an overlay as a load failure
- file: crates/rfcspec-docs/src/explorer_profile_tests.rs
  line: 194
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the reread drift test stays green when a view row gains an optional field or changes a field the writer always leaves absent
- file: explorer/src/App.vue
  line: 60
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: a profile file whose rfc differs from the mark is dropped silently, and ExplorerMachine is never read back through a Machines row
```
