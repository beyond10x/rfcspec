---
format: aep.planning-md/3
id: review-result:send-payload-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:send-payload-type-check
relations:
- reviews: story:send-payload-type-check
revision: 1
---
I found three red cases. The type check works on every input I tried, and the acceptance statement holds. What it misses: a payload whose field names don't exactly match the message's declared fields. `ess protocol explore` refuses such a payload with `[PayloadFieldsMismatch]`, but `rfcspec check` passes it.

```
unit: story:send-payload-type-check, uncommitted working tree on 60e0a56 in the wave5-send-payload worktree
verdict: NEEDS-CHANGE
cases: executed 77→83, red 3
origin: introduced 0 / pre-existing 3 / undecided 0
wrote-outside-worktree: 1 (scratch directory, 13 files)
needs-coordinator: whether field-name matching (PayloadFieldsMismatch) belongs in this story or a follow-up
```

**1. Diff stat**
`git --no-pager diff --stat`: ` crates/rfcspec-extract/src/check.rs | 54 ++++++++++++-----` (the implementor's change, not mine). My only write in the tree is one new untracked test file, `crates/rfcspec-extract/tests/adversary_send_payload.rs`. I changed no implementation file.

**2. Cases added** (`the wave5-send-payload worktree/crates/rfcspec-extract/tests/adversary_send_payload.rs`)

| case | asserts | now |
|---|---|---|
| `a_payload_field_the_message_does_not_declare_is_named` | ping server's pong payload gets an extra `extra: 1`; exactly one problem naming the transition, `pong` and `extra` | red |
| `a_misspelt_payload_field_of_the_wrong_type_is_named` | pong payload `{Data: true}` instead of `{data: …}`; at least one problem | red |
| `a_declared_message_field_the_payload_leaves_out_is_named` | pong payload empty; exactly one problem naming `data` | red |
| `an_input_of_the_wrong_type_is_named_with_both_types` | close-frame `code` (Integer) given `true`; named with both types | green |
| `an_undeclared_input_is_named_once` | `{input: nothing}` gives exactly the "no field of its trigger" problem, with no second type problem | green |
| `the_committed_websocket_overlay_holds` | committed websocket overlay gives no problems | green |

Red output from running this file alone (`cargo test -p rfcspec-extract --test adversary_send_payload`), before the suite ran:
```
---- a_misspelt_payload_field_of_the_wrong_type_is_named stdout ----
panicked at crates/rfcspec-extract/tests/adversary_send_payload.rs:92:5:
the check passes a payload ESS refuses
---- a_declared_message_field_the_payload_leaves_out_is_named stdout ----
panicked at crates/rfcspec-extract/tests/adversary_send_payload.rs:66:5:
assertion `left == right` failed: []
  left: 0
 right: 1
---- a_payload_field_the_message_does_not_declare_is_named stdout ----
panicked at crates/rfcspec-extract/tests/adversary_send_payload.rs:66:5:
assertion `left == right` failed: []
  left: 0
 right: 1
test result: FAILED. 3 passed; 3 failed
```

ESS refuses each of these. I hand-mutated `profiles/websocket/generated/ping.protospec.yaml` the same way the lowering would and ran `ess protocol explore` 0.86.0 with `witnesses/ping.inputs.json`:
- **Extra payload field** `extra`: exit 1, `[PayloadFieldsMismatch]: send payload must name exactly the message's declared fields`.
- **Payload field renamed** (`data` → `other`): exit 1, same `PayloadFieldsMismatch`.
- **`payload` key removed**: exit 1, `[MalformedProtocol]: missing field 'payload'`. An empty overlay payload lowers to `payload: {}`, which matches the renamed case instead. I did not run that exact variant.
- **Boolean literal into an Integer field**: exit 1, `[ValueTypeMismatch]: expected Integer, found Boolean`. This answers the story's open question: ESS does refuse a type mismatch, and the new check now catches it earlier.

**3. Suite run** (after the cases existed)
`CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 RUST_TEST_THREADS=4 nice -n 10 ionice -c3 cargo test -p rfcspec-extract --no-fail-fast`
```
unittests src/lib.rs          ok. 73 passed
tests/adversary_bcp14_xml.rs  ok. 4 passed; 3 ignored
tests/adversary_send_payload.rs  FAILED. 3 passed; 3 failed
error: 1 target failed:
exit=101
```
"Before" is 77: the same run with my file left out (73 + 4).

**4. Findings**

| file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|
| `crates/rfcspec-extract/src/check.rs:262` | NEEDS-CHANGE | pre-existing | A payload field the message does not declare is skipped, so `check` passes a send that ESS refuses with `PayloadFieldsMismatch` (two red cases above). A misspelt field name also skips the new type check. | A hand-written overlay; the overlay is the reviewed input of every profile. |
| `crates/rfcspec-extract/src/check.rs:262` | NEEDS-CHANGE | pre-existing | A declared message field missing from the payload is not checked; it lowers to a payload ESS refuses (red case above). | Same: hand-written overlay. |

Why pre-existing: the base `check.rs`, read through the diff, never compares payload keys with message fields. I did not run the cases against the base. The fix: inside `Effect::Send`, when the message is known, report every payload key not in `message_fields` and every `message_fields` key not in the payload. The tree covered is the uncommitted working tree on 60e0a56.

**5. What I attacked and could not break**
- **Boolean into Integer:** a Boolean literal or Boolean input sent into an Integer message field is named with both types.
- **Undeclared input:** `{input: x}` with `x` undeclared is reported once, with no second type problem.
- **Unknown message:** a send to an unknown message only gets the existing "unknown message" problem.
- **Both participants sending:** each send is checked on its own, so there is nothing extra to break.
- **Types outside your list:** `Value` has only Integer, Boolean and a trigger field, so payload expressions and stored-field values can't be written. No profile uses Optional, List, enums, newtypes or bounded integers, and the model only names Integer, String and Boolean.
- **Committed profiles:** `rfcspec check --all profiles` says "overlay holds" for dns, ntp, time and websocket, and the acceptance unit test asserts the exact message.

**6. Paths written outside the worktree**
- `a local path`: `base.yaml`, `in.json`, `m1.yaml` to `m4.yaml`, and `m1.out`/`.err` to `m4.out`/`.err`. These are the ESS probes, kept so the refusals can be reproduced.

**7. Findings block**
```findings
- file: crates/rfcspec-extract/src/check.rs
  line: 262
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: a send payload field its message does not declare is skipped (a misspelt name also escapes the type check), yet ess protocol explore refuses the lowered send with PayloadFieldsMismatch
- file: crates/rfcspec-extract/src/check.rs
  line: 262
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: a declared message field absent from a send payload passes rfcspec check but lowers to a send ESS refuses because the payload does not name exactly the declared fields
```
