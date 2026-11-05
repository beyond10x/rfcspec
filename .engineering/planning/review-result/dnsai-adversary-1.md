---
format: aep.planning-md/3
id: review-result:dnsai-adversary-1
kind: review-result
status: active
title: Adversary, dns and ai stories
relations:
- reviews: story:dns-source-catalogue
- reviews: story:dns-header-codec
- reviews: story:dns-query-machine
- reviews: story:proposal-record-replay
- reviews: story:proposal-review
- reviews: story:proposal-machine-layout
revision: 2
---
unit: the uncommitted working tree at ~/.local/state/worktree/trees/b10x/rfcspec/frames-wave (base 0e4e26e, `.engineering/` ignored), covering stories dns-source-catalogue, dns-header-codec, dns-query-machine, proposal-record-replay, proposal-review and proposal-machine-layout
verdict: NEEDS-CHANGE
cases: executed 194→195, red 1
origin: introduced 3 / pre-existing 0 / undecided 1
wrote-outside-worktree: 3 paths, all deleted
needs-coordinator: none

1. **Diff stat.** `git --no-pager diff --stat HEAD -- . ':!.engineering'` prints `25 files changed, 4491 insertions(+), 346 deletions(-)`, the same as before my pass. My only edit is in an untracked file, so the diff stat cannot show it. That file is `crates/rfcspec-propose/src/elements_tests.rs`, a test-only module. I appended one test at line 452. I changed no implementation file.

2. **Case added.** `crates/rfcspec-propose/src/elements_tests.rs:452`, `adversary_an_equals_or_one_of_value_outside_the_field_width_is_refused`. It feeds the in-process model double a `bit-widths` answer: the fixture's `frame-header` layout with a 16-bit field given `{equals: 65536}`, and in a second case `{one_of: [126, 70000]}`. It asserts the mechanical check refuses each answer and no proposal file is written. It is red now. The red output from running it alone:
   ```
   $ CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 cargo test -p rfcspec-propose --lib adversary_
   test elements_tests::adversary_an_equals_or_one_of_value_outside_the_field_width_is_refused ... FAILED
   panicked at crates/rfcspec-propose/src/elements_tests.rs:147:18:
   frame-header-equals: Ok(Recorded(".../target/rfcspec-propose/no-machines-H5y2Pp/no-machines/proposals/c40ca616...f78f4.json"))
   test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 12 filtered out
   ```
   - **Measured:** the answer is recorded, although the story:proposal-machine-layout acceptance requires "constraints within width". `rfcspec_extract::layout_problems` (`crates/rfcspec-extract/src/check.rs:310`) checks only `range` against the width. `rfcspec-gen`'s `validate` (`crates/rfcspec-gen/src/fields.rs:75-79`) checks every constraint value against the field's bounds.
   - **What reaches it:** `rfcspec propose --task bit-widths` records the answer, and `rfcspec review accept` passes it, because the overlay check uses the same function. After that, `rfcspec check` passes and `rfcspec generate` fails.
   - **Fix to name:** in `layout_problems`, check every value of `constraint.values()` against `field.bounds()`, not only `Range`.
   - **Origin:** introduced. The range-only rule was already in base `check`, but the `bit-widths` check that relies on it, and the acceptance line it breaks, are new in this tree.

3. **Suite, run after the case existed.** Exit 101, 194 passed and 1 failed. The 194 are every case except mine.
   ```
   $ CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 cargo test -p rfcspec-propose -p rfcspec-extract -p rfcspec-gen -p rfcspec-ess -p rfcspec-model -p rfcspec-cli --no-fail-fast
   test elements_tests::adversary_an_equals_or_one_of_value_outside_the_field_width_is_refused ... FAILED
   test result: FAILED. 12 passed; 1 failed; ... (rfcspec_propose lib)
   error: 1 target failed:
   EXIT=101
   ```

4. **Judgement findings** (on the same working tree):
   - **`crates/rfcspec-propose/src/lib.rs:229` (`pending`).** CONFIRMED, introduced, warning. Prose candidates are pending unless the overlay holds an entry under the same `P-n` id. The DNS overlay reviews P-56 as `R-UDP-2` and P-57 as `R-UDP-1` (I read this from `rfcspec extract --profile profiles/dns`: spans 1761-1762 and 1762-1764). So `propose --task requirement-from-prose --section 4.2.1` would ask the model about both sentences again. The refusal in `decide` that keeps a hand-reviewed requirement from being overwritten is also by id, so accepting the answer adds a second entry with the same quote. Reached by that documented command on the committed `profiles/dns`. I wrote no case, because no acceptance line says how `P-n` ids and hand-chosen ids relate.
   - **`crates/rfcspec-propose/src/lib.rs:611` (`entry`).** CONFIRMED, introduced, note. For `status` and `requirement-link` answers, the level is not compared with the catalogue keyword. The quote also only has to occur somewhere in the span. A model that answers MAY for a MUST requirement, quoting one word, passes record, replay, review and `rfcspec check`, and nothing in `check.rs` compares levels. The acceptance does say "the level as you read it", so this rests on human review.
   - **`crates/rfcspec-propose/src/tests.rs:22`.** CONFIRMED, introduced, note. The tests create fixed-name directories under `target/rfcspec-propose/` that are never removed. After this run there were 83, such as `decided-<pid>` and `empty-reason-<pid>`. They grow with every run. I did not delete them, because they are under `target/`.
   - **`crates/rfcspec-propose/src/review.rs:260`.** INFEASIBLE, undecided, note. `reject` of a layout or machine proposal is refused as AlreadyExists when the overlay already holds that name. The acceptance does not say whether reject is meant to be covered.

5. **Attacked and not broken:**
   - DNS overlay quotes and spans: all six requirement quotes and the header quote match RFC 1035 at the lines given, and the lock's SHA-256 matches the source file.
   - Z vector bit position: octets `00 00 00 10 …` put Z = 1 at bits 6-4 of the second word, as the story states.
   - `equals` vectors: neighbours stay within the field's range, the websocket vectors are unchanged, and an out-of-width value is refused by `validate` in `rfcspec-gen`.
   - `state_implication`: it is not silently parsed as `never_state`, because both have `deny_unknown_fields`. The mutant test would go red if the property were dropped or the guard not lowered, and a swapped participant/other would break the committed-model test.
   - Replay: the context is taken from the recording and a hand edit changes the digest, so the file name no longer matches. Adding `inputs.context` to a `status` proposal changes the bytes. Keys are sorted at every level.
   - `proposal_id` and `input_digest`: both cover task, model id, instructions, tool schema and the source SHA.
   - Credentials: only `env:`/`file:` references are used, no argv secret, and the proposal file stores only the model id.
   - Review: the double-decision checks hold by proposal and by (item, digest). An edited entry whose id differs from the item is refused through the "requirements holds no X" rule. Requirement edits are checked for equality with the overlay entry. I found no realistic way for the "refuse only new problems" filter to hide a new problem.

6. **Paths written outside the worktree** (all deleted):
   - `~/.cache/claude-tmp/adv-rfc-stories.txt`
   - `~/.cache/claude-tmp/adv-rfc-dns-cat.json`
   - `~/.cache/claude-tmp/adv-rfc-suite.log`

   Test scratch went to the tree's own `target/rfcspec-propose/`.

7. **Findings block:**
```findings
- file: crates/rfcspec-extract/src/check.rs
  line: 310
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: layout_problems checks only range constraints against the field width, so a bit-widths answer with an equals or one_of value the field cannot hold is recorded and accepted, then refused by rfcspec generate (red case elements_tests.rs:452).
- file: crates/rfcspec-propose/src/lib.rs
  line: 229
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: pending and the AlreadyExists refusal compare prose candidates by P-id only, so P-56 and P-57 of profiles/dns, already reviewed as R-UDP-2 and R-UDP-1, are proposed again and can be accepted as duplicate entries.
- file: crates/rfcspec-propose/src/lib.rs
  line: 611
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no mechanical check ties a status or link answer's level to the catalogue keyword or requires more than a one-word quote, so a MUST answered as MAY passes record, review and rfcspec check.
- file: crates/rfcspec-propose/src/tests.rs
  line: 22
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the tests create fixed-name directories under target/rfcspec-propose that are never removed (83 after one run).
- file: crates/rfcspec-propose/src/review.rs
  line: 260
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: undecided
  message: a reject decision on a layout or machine proposal is refused as AlreadyExists when the overlay already holds that name, and the acceptance does not say whether reject is meant to be covered.
```
