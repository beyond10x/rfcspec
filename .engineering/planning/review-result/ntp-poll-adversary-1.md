---
format: aep.planning-md/3
id: review-result:ntp-poll-adversary-1
kind: review-result
status: active
title: Adversary on story:ntp-poll-machine, pass 1
relations:
- reviews: story:ntp-poll-machine
revision: 1
---
```
unit: story:ntp-poll-machine, uncommitted working tree on base 3c591a5 in the ntp-poll-machine worktree
verdict: NEEDS-CHANGE
cases: executed 189→193, red 4
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: whether the two reach mutants should hold the unit, since the story asked only for the fixed-interval mutant
```

**1. `git --no-pager diff --stat`**

The stat lists only the implementor's 13 tracked paths (1204 insertions, 51 deletions). I changed none of them. My only addition is one untracked test file, which `--stat` does not list:

`?? crates/rfcspec-ess/tests/adversary_ntp_poll.rs`

**2. Cases added**

All four are in `the ntp-poll-machine worktree/crates/rfcspec-ess/tests/adversary_ntp_poll.rs`. Each was run alone before the suite, and all four are red now.

- **`a_transition_that_sends_a_message_is_refused_without_a_requirement_even_when_labelled_model` (:46).** Two overlays, each with one sending transition stripped of its requirement and relabelled with a `model-` prefix:
  - the ntp overlay, where `poll-reachable` (it sends `request`) becomes `model-poll-reachable`;
  - the websocket overlay, using its first transition that sends.

  The case asserts that `lower` refuses both with "implements no requirement". Red:
  ```
  panicked at crates/rfcspec-ess/tests/adversary_ntp_poll.rs:52:5:
  ntp: a sending transition labelled model- lowered without a requirement link
  test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out
  ```
- **`a_mutant_whose_register_never_empties_is_reported` (:84).** `poll-emptied` goes to `reachable` instead of `unreachable`. The case asserts that exploration fails. Red:
  ```
  "explored_states": 3137, ... "verdict": "passed"
    left: String("passed")
   right: "failed"
  ```
- **`a_mutant_whose_response_never_makes_the_server_reachable_is_reported` (:94).** `response-unreachable` goes to `unreachable` instead of `reachable`. The case asserts that exploration fails. Red:
  ```
  "explored_states": 2950, ... "verdict": "passed"
    left: String("passed")
   right: "failed"
  ```
- **`a_mutant_that_backs_off_to_a_fixed_17_s_is_reported_by_exploration` (:106).** Both backoff arms are fixed at 17000 ms. The case asserts that exploration fails on `backoff-doubles-interval`. Red:
  ```
  "explored_states": 3673, ... "verdict": "passed"
    left: String("passed")
   right: "failed"
  ```

**3. Suite run (after the cases existed)**

```
cargo test --locked --no-fail-fast -p rfcspec-cli -p rfcspec-ess   (nice/ionice, line-tables-only, INCREMENTAL=0, RUST_TEST_THREADS=4)
EXIT=101
passed 189 failed 4
test a_mutant_that_backs_off_to_a_fixed_17_s_is_reported_by_exploration ... FAILED
test a_mutant_whose_register_never_empties_is_reported ... FAILED
test a_mutant_whose_response_never_makes_the_server_reachable_is_reported ... FAILED
test a_transition_that_sends_a_message_is_refused_without_a_requirement_even_when_labelled_model ... FAILED
```

The "before" figure, 189, comes from the implementor's `target/green-all.log`: the same two packages, 19 test binaries. The full log of my run is `target/adversary-suite.log`.

**4. Findings**

| file:line | verdict / origin | what was measured | what reaches it |
|---|---|---|---|
| `crates/rfcspec-ess/src/lib.rs:312` | NEEDS-CHANGE / introduced | The exemption depends on the label prefix alone. In any profile, a transition that sends a protocol message lowers with no requirement link once it is labelled `model-…`. `rfcspec check` says nothing: `check.rs:139` only loops over the ids a transition does list, and all four profiles report "overlay holds". | Any overlay edit or `rfcspec review` accept. Nothing in the lowering, the check or the CLI test (which looks only at `machines[1]`) limits the exemption to model constructs. **Fix:** refuse a `model-` transition that has a `send` or `emit` effect. Only the cancel-only horizons and the no-effect discard would stay exempt. |
| `profiles/ntp/overlay.yaml:264` | NEEDS-CHANGE / introduced | `R9.2-REACH-1` is marked satisfied by `poll.client.R9.2-REACH-1-poll-emptied`. A mutant in which that transition stays `reachable` passes exploration and every existing test. The same holds for `response-unreachable` (:297) when it stays `unreachable`. | The whole reach half of the model has no behavioural or structural check. `event_count` has only a `max`, so no ESS property can catch these mutants. **Fix:** a lowering test that pins the `(from, to, trigger)` triple of every `poll` transition. |
| `profiles/ntp/overlay.yaml:351` | CONFIRMED / introduced | Within the 48 s horizon, `backoff-doubles-interval` fails only when the first two backoff intervals together are 32 s or less. A fixed 17 s backoff, which never doubles, passes exploration. Only the literal `after` pin at `ntp_poll.rs:108` rejects it. Under the horizon, exploration never sees a second doubling or the ceiling (the story already says the ceiling is not reached). | This is the documented design. Passing the exploration means "the first backoff interval is not 16 s", not that the interval doubles. The property's name and the AGENTS.md row ("a backoff doubling the poll interval up to 2^MAXPOLL s") read as more than that. |

**5. Attacked and not broken**

- **Quotes and spans:** all seven quotes are verbatim at their stated lines in `profiles/ntp/sources/rfc5905.txt`. The F-1 lines 2871 (24) and 5820 (12) are correct, and MINPOLL/MAXPOLL are at 909/910 as cited.
- **Wrong ceiling, uncapped backoff, factor change, missing reset arm, or only one backoff arm changed:** each is caught by the arm and count pins at `ntp_poll.rs:108`.
- **Unreach at the wrong count:** this cannot be tested. The model abstracts the count (G2), as the story chose.
- **Design citations:** the cited strings exist in the ess 0.86.0 source (`engine.rs:230`, `:96`; `monitor.rs:199`, `:233`). I re-ran the probes:
  - with duplication on: inconclusive, "state bound exhausted" (G12 as written; the story said step bound);
  - with capacity 2: inconclusive, "channel server-to-client capacity exhausted" (G13).
- **Generated output of the other profiles:** no drift. `rfcspec lower --check` and `generate --check` exit 0 for time, websocket, dns and ntp, `rfcspec-docs generate --check` exits 0, and the download copy and the page's size and hash match the generated model.
- **AGENTS.md:** the State, What exists and Gate sections are true. `README.md:8` still says "three profiles". That line was already stale at the base, is outside the story's scope, and is not raised as a finding.

**6. Paths written outside the worktree:** none. The probe copies under `target/adversary-probe/` were removed. `target/adversary-suite.log` stays in the tree. The lease was released and `target/.keeper-hold` removed.

```findings
- file: crates/rfcspec-ess/src/lib.rs
  line: 312
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the model- exemption depends on the label alone, so in any profile a transition that sends a protocol message lowers with no requirement link and rfcspec check reports nothing
- file: profiles/ntp/overlay.yaml
  line: 264
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: mutants that keep poll-emptied reachable or response-unreachable unreachable pass exploration and every test, so R9.2-REACH-1's satisfying transitions are pinned by nothing
- file: profiles/ntp/overlay.yaml
  line: 351
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: under the 48 s horizon backoff-doubles-interval only rejects backoff intervals summing to 32 s or less, so a fixed 17 s backoff passes exploration and only the lowering test's literal pin rejects it
```
