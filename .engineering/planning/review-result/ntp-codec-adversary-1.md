---
format: aep.planning-md/3
id: review-result:ntp-codec-adversary-1
kind: review-result
status: active
title: Adversary on story:ntp-header-codec, pass 1
relations:
- reviews: story:ntp-header-codec
revision: 1
---
I found one real defect, and nothing in the repository reaches it today. The NTP header crate itself held up against everything I tried.

```
unit: story:ntp-header-codec, uncommitted working tree on 07edfa1 in the ntp-header-codec worktree
verdict: INFEASIBLE
cases: executed 34→41, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none
```

**1. What I changed.** `git --no-pager diff --stat` shows only the implementor's 9 files, unchanged. My one addition is untracked: the test file `crates/rfcspec-gen/tests/adversary_signed_fields.rs`. I touched no other file.

**2. Cases added** (all in that file, all drive `rfcspec_gen::rust` from the ntp overlay with its fields replaced):

**Red now:** `a_signed_field_with_a_long_name_generates_rustfmt_clean_code`. A signed field named `poll-interval-in-log2-seconds` produces a parse line that rustfmt would rewrap. The unsigned path wraps long `reader.field(...)` calls once the arguments pass 60 characters; the new signed path at `crates/rfcspec-gen/src/fields.rs:507` never wraps. Output when the case first ran alone:
```
generated:     let poll_interval_in_log2_seconds = signed(reader.field("poll-interval-in-log2-seconds", 8, any)?, 8) as i8;
rustfmt:       let poll_interval_in_log2_seconds =
test result: FAILED. 1 passed; 1 failed
```
- **What reaches it:** nothing found. ntp's signed fields are `poll` and `precision`, and no other profile has a signed field. `cargo fmt --all --check` in `task check` would catch it once a profile does.
- **Suggested fix:** give the signed branch the same wrapping as the unsigned one.

**Green now:**
- `signed_fields_of_widths_2_12_16_17_33_63_and_64_generate_a_crate_whose_tests_pass` compiles the generated crate with `rustc --test`, runs it, and checks rustfmt. It covers the minimum, -1 and maximum vectors for every width, including `i64::MIN` and 2^62 at 63 bits.
- Four refusal cases. A signed field is refused when it has a constraint, has presence, selects another field, or carries the length, and each refusal names the layout and field. Before these, no test in the repository asserted that refusal, so removing that check would have stayed green.
- `a_settled_base_whose_selector_refuses_0_generates_a_crate_whose_tests_pass` uses a selector with range 1–3 that switches dependent fields on and off, plus `equals` and `one_of` fields that refuse 0. Its generated crate compiles and all 16 of its tests pass.

**3. Suite run, after the cases existed:**
- `cargo test --locked -p rfcspec-gen -p rfc5905-ntp`: rfc5905_ntp 11 ok; rfcspec_gen unit tests 23 ok; adversary_signed_fields 6 passed, 1 failed (the long-name case). Exit 101.
- The same with `--lib`, which leaves out my file: 11 + 23 ok, so 34 before my cases.
- `cargo test --locked -p rfcspec-cli --test ntp`: 27 ok, exit 0.
- `rfcspec generate --check` exits 0 for dns, ntp, time and websocket, and `git status` shows no changes under the other three profiles.
- `cargo fmt --all --check` exits 0, and `rfcspec check --profile profiles/ntp` says the overlay holds.

**4. Judgement findings** (covering the same working tree):
- **Misleading test name** (`crates/rfcspec-gen/src/fields.rs:1121`). `a_signed_field_of_12_bits_parses_its_twos_complement_and_prints_back` only checks the generator's vector octets; no generated parser runs in it. My widths case now does what the name claims. CONFIRMED, introduced.
- **Out of scope:** the story's scope list doesn't include `crates/rfcspec-gen/src/lib.rs`, but the diff changes it (+85: tests and a module doc). This is not in the findings block because it has no line.

**5. What I attacked and could not break:**
- Two's complement for widths 2–64, at their minimum, -1, maximum and 0.
- Octet positions: poll is octet 2 and precision octet 3.
- The settled base when it interacts with presence, `equals` and `one_of`.
- The 48-octet layout against Figure 8, and the 384-bit sum.
- The overlay's quotes and spans (lines 1083–1084 and 1812–1814) against the locked RFC text; its SHA-256 matches.
- `octets_line`: the ntp crate is rustfmt-clean, and the other profiles' generated output is unchanged.
- Version 0 and 5–7 refused; 47 octets give `Incomplete`.
- Taskfile: the `ntp` task holds both commands and `check` runs it.
- Catalogue: 1 MUST, 1 SHOULD, 2 MAY; 1 satisfied, 10 unreviewed. The downloads table's byte counts and SHA-256 values match the files.

**6. Paths written outside the worktree:** none. My probes built under the worktree's `target/tmp/adversary-signed-widths/` and `target/tmp/adversary-settled-base/`. I removed `target/.keeper-hold` after the builds. I did not acquire a worktree lease.

```findings
- file: crates/rfcspec-gen/src/fields.rs
  line: 507
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the signed-field parse line is never wrapped, so a signed field with a name of about 24 or more characters generates a crate rustfmt would reformat; no current profile has one (red case a_signed_field_with_a_long_name_generates_rustfmt_clean_code)
- file: crates/rfcspec-gen/src/fields.rs
  line: 1121
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a_signed_field_of_12_bits_parses_its_twos_complement_and_prints_back checks only generator vector octets and never runs a generated parser or printer, so its name claims more than it tests
```
