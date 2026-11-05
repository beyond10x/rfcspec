---
format: aep.planning-md/3
id: review-result:ntp-exchange-adversary-1
kind: review-result
status: active
title: Adversary on story:ntp-exchange-machine, pass 1
relations:
- reviews: story:ntp-exchange-machine
revision: 1
---
unit: story:ntp-exchange-machine, uncommitted working tree at the ntp-exchange-machine worktree (base b11d2aa)
verdict: NEEDS-CHANGE
cases: executed 238→245, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (deleted)
needs-coordinator: whether a zero origin is in scope for this story (the zero-origin refusal is in Appendix A.5.1.1, which is outside the profile's sections), or a backlog item with an overlay note

**1. Diff stat**

`git --no-pager diff --stat` is the same as the implementor's 10 tracked files (620+/28-). I changed none of them. My additions are two untracked test files, and both are test paths:
- `crates/rfcspec-ess/tests/adversary_ntp_exchange.rs` (new)
- `crates/rfcspec-model/tests/adversary_port.rs` (new)

**2. Cases added**

These ran alone, before any suite run.

`the ntp-exchange-machine worktree/crates/rfcspec-ess/tests/adversary_ntp_exchange.rs`:
- `a_server_packet_with_origin_zero_is_never_a_valid_packet`: **red**. It explores the committed model with the witness `replay {org: 0}` and asserts `passed`. After the first valid packet sets `xmt` to 0, the guard `org = xmt` accepts origin 0. That makes a third `valid-packet`. RFC 5905 A.5.1.1 (source lines 4545-4549, `if (r->org == 0) synch = FALSE`) refuses such a packet, and zeroing `xmt` only protects against replay because of that refusal. Red output:
  ```
  "explored_states": 339, ... "payload": {"org": 0.0} ...
  "reason": "property violated: {\"one-valid-packet-per-poll\"}",
  "verdict": "failed"
    left: String("failed")
   right: "passed"
  ```
- `a_mutant_that_keeps_xmt_after_the_second_check_fails_under_the_committed_witness`: **red**. It drops the `xmt := 0` reset from `accept-second` only. Under the committed witness the exploration still passes, so `task check` cannot see that R8-REPLAY-1 is missing for the second poll. Red output:
  ```
  "explored_states": 561, "explored_transitions": 1997, ... "payload": {"org": 1.0} ...
  "reason": "bounded model search completed for the supplied finite input witnesses; not implementation evidence",
  "verdict": "passed"
    left: String("passed")
   right: "failed"
  ```
- `the_second_check_mutant_fails_when_the_witness_replays_origin_two`: green, a control. The same mutant fails once the witness replays `org 2`, so the gap is in the witness, not the property.
- `the_committed_model_passes_a_replay_of_origin_two`: green, a control. A witness that adds `org 2` would not turn the correct model red.
- `single_transition_mutants_of_the_first_poll_fail_under_the_committed_witness`: green. Three single mutants are caught: no reset in `first`, no guard in `first`, no guard in `second`.

`the ntp-exchange-machine worktree/crates/rfcspec-model/tests/adversary_port.rs`:
- `absent_and_null_are_absent`: green.
- `the_bounds_are_1_and_65535`: green. It also refuses `0`, `65536`, `-1`, `123.0`, `true`, `[123]` and 2^32+123.

**3. Suite run, after the cases existed**

Command: `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 RUST_TEST_THREADS=4 nice -n 10 ionice -c3 cargo test --locked -p rfcspec-ess -p rfcspec-model -p rfcspec-docs -p rfcspec-cli --no-fail-fast`

Result: EXIT=101. 245 cases ran.
- The only failure: `error: 1 target failed: -p rfcspec-ess --test adversary_ntp_exchange`, with `test result: FAILED. 3 passed; 2 failed`.
- Every other binary was ok, including `ntp.rs` (30), the `rfcspec_ess` unit tests (32) and the `rfcspec_model` unit tests (25).
- "Before" is 238: the same run with my two binaries (7 cases) left out.

**4. Findings**

| file:line | verdict | origin | measured | what reaches it |
|---|---|---|---|---|
| profiles/ntp/overlay.yaml:126 (`accept-first` / `accept-second` guard) | NEEDS-CHANGE | introduced | adversary_ntp_exchange.rs:64: with witness org 0 the verdict is `failed` on `one-valid-packet-per-poll` | On the wire: any server or peer packet whose origin is 0, i.e. a sender that has not heard from us yet, or a spoofed packet. In the model: only a witness, because the committed one offers org 1 only. Section 8's text alone admits org 0 when xmt is 0, so R8-REPLAY-1 "satisfied" overstates what the model shows. The fix is the implementor's: record the zero-origin refusal as unsupported (G1 inequality) or as a finding, or model it, and say which. |
| profiles/ntp/witnesses/exchange.inputs.json:2 | NEEDS-CHANGE | introduced | adversary_ntp_exchange.rs:76: the mutant without the reset in `accept-second` gets `passed` (561 states) | `task check` explores with this witness only. The two mutant tests in `crates/rfcspec-ess/src/lib.rs` strip both transitions at once (`assert_eq!(mutated, 2)`), which hides this gap. Fix: the witness also replays `org 2`. The control shows the correct model still passes with it and this mutant then fails. |

**5. Attacked and not broken**
- Quotes and spans against the locked text: BOGUS 1590-1591, REPLAY 1593-1595, DUP 1588-1590, LOSS 1434-1435 and FXMIT 1862-1863 all match line by line. `rfcspec check` checks the quotes verbatim and prints "overlay holds (9 requirements, …)".
- The PORT 123 citation at line 906 matches.
- Port parsing: no edge case got through.
- Drift in the other profiles: `lower --check` reports current for time, websocket, dns and ntp. `rfcspec-docs generate --check` reports 35 files current. `index.md` changes only the ntp row.
- The committed model still passes `ess protocol explore` (579 states).
- The four changed earlier tests still test what they did. The "no semantics" and "no machines" assertions became exact assertions of one udp layer and `[("exchange","udp")]`. The counts 6/1/2, 5 and 4 match the requirement statuses.
- Side note, not a finding: `crates/rfcspec-ess/src/lib.rs` changes only the `mod tests` section, as the story's "mutant tests only" allows.

**6. Paths written outside the worktree**
- One suite log at `a scratch log under the claude temp directory`, deleted after reading.
- `target/.keeper-hold` was touched and removed around each build.

**7. Findings block**
```findings
- file: profiles/ntp/overlay.yaml
  line: 126
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: once xmt is reset to 0 the guard org = xmt accepts a zero-origin packet, a third valid-packet that RFC 5905 A.5.1.1 refuses as unsynchronized, so R8-REPLAY-1 marked satisfied overstates the model
- file: profiles/ntp/witnesses/exchange.inputs.json
  line: 2
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the witness replays origin 1 only, so a mutant without the xmt reset in accept-second passes exploration; adding a replay of origin 2 catches it and the correct model still passes
```
