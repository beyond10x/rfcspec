---
format: aep.planning-md/3
id: review-result:ntp-timer-adversary-1
kind: review-result
status: active
title: Adversary on story:ntp-timer-model, pass 1
relations:
- reviews: story:ntp-timer-model
revision: 1
---
unit: story:ntp-timer-model, uncommitted working tree on top of ec76054 in the ntp-timer-model worktree
verdict: NEEDS-CHANGE
cases: executed 217→221, red 3
origin: introduced 2 / pre-existing 1 / undecided 0
wrote-outside-worktree: 2 paths (listed in part 6)
needs-coordinator: whether "the check refuses a backoff of a timer that has no earlier interval" belongs in this story. The acceptance lists three refusals and this is not one of them.

**1. Diff stat**

```
 crates/rfcspec-ess/src/lib.rs       | 173 +++++++++++++++++++++++++++++++-----
 crates/rfcspec-extract/src/check.rs | 128 ++++++++++++++++++++++++--
 crates/rfcspec-model/src/lib.rs     | 158 ++++++++++++++++++++++++++++++--
?? crates/rfcspec-cli/tests/adversary_timer.rs
```

The three changed files are the implementor's. My only file is the new, untracked test file `crates/rfcspec-cli/tests/adversary_timer.rs`. I changed no implementation file.

**2. Cases added** (all in `adversary_timer.rs`; first run: `cargo test -p rfcspec-cli --test adversary_timer`)

- **`a_backoff_of_a_timer_never_armed_before_is_refused_by_the_check_or_explores`: red.**
  - It takes the committed WebSocket `close` client, which arms `close-timeout` only once, and makes that one arm a backoff (factor 2, ceiling 8000).
  - It asserts that `rfcspec check` refuses this or that `ess protocol explore` passes it.
  - Output: `rfcspec check admitted it ([]) and ess protocol explore reported: ... "explored_states": 1, "explored_transitions": 0, "reason": "no transitions exercised", "verdict": "inconclusive"`
  - Cause: ess 0.86.0 fails the arm with "backoff has no prior interval" (`ess-conformance/src/protocol/engine.rs:413` at tag 0.86.0), and `ess` validation does not catch it. So the check admits a model that cannot run. The unit's own test `a_backoff_within_the_machine_bounds_holds` (check.rs:1295) asserts that this same overlay is clean.
- **`an_arm_naming_both_keys_is_a_parse_error_even_when_one_is_null`: red.**
  - Output: `parsed: ["{timer: t, millis: null, backoff: {factor: 2, ceiling_ms: 4}}", "{timer: t, millis: 1000, backoff: null}"]`
  - Both keys are present in each input, which the acceptance says is a parse error. Each `Option` reads a null as absent (`rfcspec-model/src/lib.rs:825`).
- **`a_fixed_arm_of_zero_or_above_max_time_is_refused_by_the_check`: red.**
  - Output: `check admitted fixed millis [0, 30001]`
  - ess refuses both values ("fixed interval must be positive and within max_time_ms"). The check refuses the backoff version of this and not the fixed one.
  - At the base, check.rs:258 only did `timers.push(&arm.timer)`, so this was already true before the change.
- **`the_uncancelled_retry_is_inconclusive_for_its_outstanding_timer`: green.**
  - It makes the unit's `assert_ne!(... "passed")` (rfcspec-ess/src/lib.rs:1558) more specific: verdict `inconclusive`, reason "dead end with outstanding queue, timer or flush obligations", 7 transitions explored.
  - My first draft of this case expected `failed`, and that run showed ess actually returns `inconclusive`. I corrected the case to match ess. It is not a finding.

**3. Suite run** (after the cases existed): `cargo test --no-fail-fast -p rfcspec-model -p rfcspec-ess -p rfcspec-extract -p rfcspec-cli`, exit 101.

- adversary_timer: `FAILED. 1 passed; 3 failed`
- Every other binary passed:
  - rfcspec-cli tests: bulk 24, check 9, dns 14, elements 4, extract 4, generate 5, lower 5, propose 6, read 3, review 10, verify 2, websocket 22, main 1
  - rfcspec_ess 29, rfcspec_extract 56, rfcspec_model 23
- The before count, 217, is this same run with `adversary_timer.rs` taken out; the implementor reported no case count.

**4. Findings**

| file:line | verdict | origin | what reaches it |
|---|---|---|---|
| crates/rfcspec-extract/src/check.rs:260 | NEEDS-CHANGE | introduced | The check admits a backoff on a timer that has no earlier fixed arm, and ess cannot run it. The unit's test at :1295 asserts this case is clean. NTP poll overlays (story:ntp-poll-machine) will write backoff arms. Smallest fix: refuse a backoff of a timer the participant never arms with `millis` anywhere. |
| crates/rfcspec-model/src/lib.rs:825 | INFEASIBLE | introduced | `millis: null` next to `backoff`, or `backoff: null` next to `millis`, parses and silently picks one. Nothing in profiles/ writes a null today. Fix: `#[serde(deserialize_with)]` or `Option<Option<_>>`, so that a present key counts. |
| crates/rfcspec-extract/src/check.rs:260 | CONFIRMED | pre-existing | The check admits a fixed `millis` of 0 or above `max_time_ms`, which ess refuses. The committed overlays use values ess accepts. |

These cover the working tree on ec76054.

**5. Attacked and could not break**

- Bytes of existing overlays: `ArmFields` writes `timer, millis` in the old order, and `loss` is left out when false.
- Default `loss` reaching existing lowered models: none; lowering still writes `loss: false`.
- Overflow:
  - `factor` above u32 or negative does not parse.
  - A huge `ceiling_ms` is refused against `u64::from(max_time_ms)`.
  - ess multiplies with `checked_mul` on an interval of at most a u32.
- Factor 1: ess accepts it, and so does the check.
- Undeclared timer: refused through the existing "unknown timer" path.
- The check compares the ceiling against the lowered bounds, because both read `Machine::bounds()`.

**6. Paths written outside the worktree**

- The harness task directory holds the outputs of the two background test runs; the harness wrote them.
- `tempfile::tempdir()` directories under the session's temporary directory were removed when they dropped.
- I took no worktree lease. `target/.keeper-hold` and my suite log are removed.

**7.**

```findings
- file: crates/rfcspec-extract/src/check.rs
  line: 260
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: rfcspec check admits a backoff arm of a timer the participant never arms with a fixed interval, which ess 0.86.0 cannot execute ("backoff has no prior interval", explore verdict inconclusive with no transitions), and the unit's test a_backoff_within_the_machine_bounds_holds asserts that overlay is clean
- file: crates/rfcspec-model/src/lib.rs
  line: 825
  category: acceptance
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: an arm naming both millis and backoff parses when either is null, against the acceptance's "both keys together is a parse error"; no committed overlay writes a null
- file: crates/rfcspec-extract/src/check.rs
  line: 260
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: rfcspec check admits a fixed arm of 0 ms or above max_time_ms, which ess specify validate refuses, while the new code refuses the backoff analogue
```
