---
format: aep.planning-md/3
id: review-result:wave6-acceptance-round-1
kind: review-result
status: active
title: 'Wave 6 plan critic, acceptance, round 1: needs-revision'
relations:
- reviews: story:abnf-core
- reviews: story:catalogue-grammars
- reviews: story:fixed-arm-bounds-check
- reviews: story:explorer-look
revision: 1
---
needs-revision

- story:abnf-core — the last acceptance bullet, "synthetic grammars written for the test cover `%s` and `%i`, `=/`, num-val ranges and concatenations, and each core rule", names inputs but no result (accepted, rejected, bytes consumed, `Invalid` offset), so a test that only parses them would satisfy it — `.engineering/planning/story/abnf-core.md:49`
- story:catalogue-grammars — the line "`cargo test -p rfcspec-wire grammars` passes the same on a synthetic RFCXML document" names no test, and a name filter that matches nothing exits 0. No test path in `crates/rfcspec-wire` contains "grammars" today (`stubs.rs` holds `stubs_collect_nothing_from_rfc6455`), so the command already passes without any work. "The same" also leaves open what the document must produce — `.engineering/planning/story/catalogue-grammars.md:51`
- story:catalogue-grammars — "`ess specify validate --path ess` passes with the scenario" reads the same before and after. It already exits 0 on this tree (I ran it), and the line names neither the scenario file nor the outcomes in the story body (`extracted`, then `already-exists`, `Grammars` view holds one row) — `.engineering/planning/story/catalogue-grammars.md:52`
- story:fixed-arm-bounds-check — the acceptance cites wrong lines. It gives backoff arm 275-297, the existing message at 1380 and `CLOSE_START` at 1332. On this tree the backoff arm is about 276-309, the message is at 1383 and `CLOSE_START` is at 1360 in `crates/rfcspec-extract/src/check.rs`. The quoted strings still let a checker find the code, so this is a warning, not a blocker — `.engineering/planning/story/fixed-arm-bounds-check.md:24`

What I read: all four stories in full with `aep plan artifact show` (4 of 4), plus `wave-6.md`. On the tree I checked:
- RFC 6455 lines 741-747, 1384-1423, 1535, 1687-1783, 1826-1827 and 1701-1703.
- `crates/rfcspec-doc` `abnf_rule`, the `rfcspec-abnf` crate, the `rfcspec-wire` stubs, the `overlay.yaml` close machine and the Taskfile.
- The pre-existing binary: `rfcspec extract … | jq .grammars` prints `null`, and `rfcspec check --all profiles`, `bulk check` and `ess specify validate` all exit 0.

What I verified as checkable on this tree:
- abnf-core bullets 1-4 are checkable against the quoted lines. Line 1826 holds `j = i MOD 4` and the comments sit on lines 1701-1703.
- Every span and figure count in catalogue-grammars matches the source: 1384-1423, 1687-1783 and 19 rule figures after the page furniture at 1738-1740.
- The six sections named exist.
- explorer-look: I judged its two lines checkable by comparing computed colours, font, header and footer against a built docs page, per your instruction.

What I could not establish:
- I did not run `task explorer` or `task website` (pnpm and a site build).
- I did not run the `rfcspec-wire` tests.
- Out of my lane, for the scope and parallel-safety critics:
  - `crates/rfcspec-wire/tests/stubs.rs:22` asserts `grammars::collect(...) == Vec::new()` and will fail once catalogue-grammars lands. That file is not in the story's scope.
  - catalogue-grammars' `rfcspec-wire` test file is not named in its scope either.

```findings
- file: .engineering/planning/story/abnf-core.md
  line: 49
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance names synthetic grammars that 'cover' %s, %i, =/, num-val ranges, concatenations and each core rule but states no result (accepted, rejected, bytes consumed, Invalid offset), so a test that only parses them satisfies it"
- file: .engineering/planning/story/catalogue-grammars.md
  line: 51
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance 'cargo test -p rfcspec-wire grammars passes the same on a synthetic RFCXML document' names no test, so the name filter matches nothing today and the command already exits 0 before the work; 'the same' also leaves the expected output unstated"
- file: .engineering/planning/story/catalogue-grammars.md
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance 'ess specify validate --path ess passes with the scenario' reads the same before and after (it exits 0 now) and names neither the scenario file nor its outcomes (extracted, then already-exists, one Grammars row)"
- file: .engineering/planning/story/fixed-arm-bounds-check.md
  line: 24
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance cites check.rs lines 275-297, 1380 and 1332, but on this tree the backoff arm is about 276-309, the existing message is at 1383 and CLOSE_START is at 1360"
```
