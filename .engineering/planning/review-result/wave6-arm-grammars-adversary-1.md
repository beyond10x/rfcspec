---
format: aep.planning-md/3
id: review-result:wave6-arm-grammars-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:fixed-arm-bounds-check and story:catalogue-grammars
relations:
- reviews: story:fixed-arm-bounds-check
- reviews: story:catalogue-grammars
revision: 1
---
# Adversary report: rfcspec wave 6, two units

Unit A (fixed-arm bounds check): nothing found; check and ESS agree at every boundary. Unit B (catalogue grammars): one warning. On RFCXML it drops every `abnf9110` figure, so RFC 9110 and 9112 lose almost all their grammars.

## A. story:fixed-arm-bounds-check

```
unit: story:fixed-arm-bounds-check, uncommitted working tree on 24ccbc4 in wave6-fixed-arm
verdict: nothing found
cases: executed 56→59, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none (the cargo test temp dirs live under the session TMPDIR and clean themselves up)
needs-coordinator: none
```

1. The `git diff --stat` shows only the unit's `crates/rfcspec-extract/src/check.rs` (+60). Untracked: `crates/rfcspec-cli/tests/adversary_fixed_arm.rs`, a test file. No implementation file was touched.
2. Three cases in `crates/rfcspec-cli/tests/adversary_fixed_arm.rs`. Each lowers the websocket `close` machine with a fixed arm of `close-timeout` and runs `ess protocol explore`. It asserts that `rfcspec check` refuses exactly when ESS refuses with "fixed interval":
   - declared bounds, at millis 0, 1, 29999, 30000 and 30001;
   - default bounds (`bounds: None`), at millis 0, 1, 999, 1000 and 1001;
   - with default bounds, millis 1001 is named as `... with millis 1001 above the machine's max_time_ms 1000`.

   The first run failed, but the failure was in my own harness: ESS reports the refusal as a non-zero exit, which arrives as `Err` (`[InvalidTimerDuration]: fixed interval must be positive and within max_time_ms`). After I fixed that, all three are green.
3. Suite runs, all exit 0:
   - `cargo test -p rfcspec-extract --lib check`: 44 passed.
   - `cargo test -p rfcspec-cli --test adversary_timer --test adversary_fixed_arm --test check`: 3, 3 and 9 passed.
   - `rfcspec check --all profiles`: all four profiles hold.
4. No judgement findings.
5. Attacked without a break:
   - **Default `max_time_ms`:** lowering always emits `machine.bounds()`, so the check and ESS use the same 1000.
   - **Other places an arm appears:** `Arm` only occurs in transition effects, and decisions are checked against `overlay.machines`. The other profiles' fixed arms (16000 and 48000 in ntp, 5000 in websocket) are within their bounds.
   - **Message format:** it matches the story's acceptance strings exactly.
   - **Mutants:** `>` against `>=` is pinned by the 30000 and 30001 cases.
6. Paths outside the worktree: none.

```findings
[]
```

## B. story:catalogue-grammars

```
unit: story:catalogue-grammars, uncommitted working tree on 24ccbc4 in wave6-catalogue-grammars
verdict: CONFIRMED
cases: executed 126→131, red 3
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: whether `abnf9110` (and any `abnf*` type) counts as ABNF; the story text says `abnf` literally
```

1. The `git diff --stat` shows only the unit's 8 files (+279/−8). Untracked: `crates/rfcspec-wire/tests/adversary_grammars.rs`, a test file. No implementation file was touched.
2. Five cases in `crates/rfcspec-wire/tests/adversary_grammars.rs`. They read `$HOME/.cache/rfcspec-corpus` read-only, and panic if a file is missing rather than passing.
   - **`rfc9110_text_and_rfcxml_give_grammars_for_the_same_sections` (red):** 64 sections have a grammar only from the text, among them `5.6.7` with 29 lines, `4.1` with 11 and `14.1.1` with 12. RFCXML adds no section.
   - **`rfc9112_text_and_rfcxml_give_grammars_for_the_same_sections` (red):** 18 sections have a grammar only from the text, among them `1.2` with 16 lines and `7.1` with 9.

     The cause is that RFC 9110, 9111 and 9112 type their ABNF `type="abnf9110"` (102 figures in the corpus). `is_abnf` (`crates/rfcspec-wire/src/grammars.rs:11-16`) accepts only `Some("abnf")`, and a typed figure never reaches the line test.
   - **`figures_rfcxml_types_as_not_abnf_are_not_abnf_by_their_lines` (red):** across the whole RFCXML corpus, figures typed as something other than ABNF have a line that defines a rule, so the text reader would collect them as ABNF:

     | Type | Figures |
     |---|---|
     | `""` | 265 |
     | `pseudocode` | 243 |
     | `sdp` | 214 |
     | `xml` | 196 |
     | `test-vectors` | 180 |
     | `cddl` | 166 |
     | `http-message` | 71 |
     | `ascii-art` | 55 |
     | `c` | 39 |

     This is the story's chosen mechanical behaviour, so I report it as a note: text and RFCXML give different grammar sets for the same RFC.
   - **`page_furniture_stays_out_of_the_rules_of_rfc3986_5234_7230_9110` (green):** no `[Page N]` footer, running header or form feed reaches the rules.
   - **`grammar_spans_hold_their_rules_and_have_a_section` (green):** every span covers its rule lines, and no grammar has the id `-` in 3986, 5234, 7230 or 9110.
3. Suite run: `cargo test --no-fail-fast -p rfcspec-wire -p rfcspec-extract -p rfcspec-model` passed every target except `adversary_grammars` (2 passed, 3 failed). That includes `grammars.rs` 2/2, `stubs.rs` 1/1 and `locked_profiles` 1/1; exit 101.
   - `rfcspec bulk check ...` reports "holds", exit 0.
   - `ess specify validate --path ess` reports 7 scenarios valid, exit 0.
4. Findings:
   - **The `abnf9110` gap** (`crates/rfcspec-wire/src/grammars.rs:13-14`): CONFIRMED, introduced, warning.
     - **What reaches it:** `rfcspec extract` on any profile with an RFCXML source of RFC 9110, 9111 or 9112. No committed profile does today; all four lock text sources.
     - **Fix:** accept a type starting with `abnf`. The same literal is in the ABNF-comment test at `crates/rfcspec-extract/src/lib.rs:154`, which predates this unit.
   - **Text and RFCXML disagree** on figures RFCXML types as not-ABNF (`crates/rfcspec-wire/src/grammars.rs:15`): CONFIRMED, introduced, note. The story accepts this by design ("a pseudo-code figure stays a catalogue grammar"). In practice, a text read of a C, CDDL, SDP or XML figure with `x = y` becomes a grammar.
   - **No test fails if a typed non-ABNF figure is accepted** (`crates/rfcspec-wire/tests/grammars.rs`): CONFIRMED, introduced, note. The synthetic untyped figure has no rule line, so the mutants "ignore `type`" and "`type` OR line test" both survive. A case to add: `<sourcecode type="pseudocode">x = 1</sourcecode>` must give no grammar.
   - **Grammars are unpinned outside RFC 6455** (`crates/rfcspec-extract/tests/locked_profiles.rs:26`): CONFIRMED, introduced, note. Clearing grammars before hashing means grammars for 1035, 5905 and 868 are pinned nowhere; RFC 5905's C appendix will produce grammars nobody checks. RFC 6455 is pinned by `stubs.rs`.
5. Attacked without a break:
   - **Grouping and order:** by first appearance, and non-contiguous figures merge.
   - **Id collisions:** keys are unique by construction. In text, `-` only appears before the first numbered section; appendices get a letter number. In XML, an unnumbered section gets its anchor.
   - **serde default and skip:** both present; no committed catalogue JSON exists.
   - **Bulk data:** unchanged, since bulk check passes and the summary counts requirements only.
   - **XML catalogue equality:** `rfcxml.rs` compares requirements and prose only.
   - **Empty type:** the `type=""` figures in the corpus are pseudo-code, not real ABNF.
   - **ESS scenario:** validates.

   I could not check whether website data changes, because it is not in this repository.
6. Paths outside the worktree: none. My builds went into the tree's own `target/`.

I took no worktree lease in either tree; I only added test files and ran builds.

```findings
- file: crates/rfcspec-wire/src/grammars.rs
  line: 13
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: is_abnf accepts only type "abnf", so RFCXML figures typed "abnf9110" (RFC 9110/9111/9112, 102 figures) give no grammar while the text read of the same RFC gives one for 64 (9110) and 18 (9112) sections
- file: crates/rfcspec-wire/src/grammars.rs
  line: 15
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the text rule-line test collects figures that RFCXML types as pseudocode, sdp, xml, cddl, test-vectors, http-message, c and others (over 1500 corpus figures), so text and RFCXML catalogues of one RFC disagree on grammars
- file: crates/rfcspec-wire/tests/grammars.rs
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no case has a typed non-ABNF figure with a rule line, so mutants that ignore type or OR it with the line test survive
- file: crates/rfcspec-extract/tests/locked_profiles.rs
  line: 26
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: clearing grammars before the catalogue digest leaves the grammars of RFC 1035, 5905 and 868 pinned by no test
```
