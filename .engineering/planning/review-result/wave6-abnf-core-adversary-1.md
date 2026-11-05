---
format: aep.planning-md/3
id: review-result:wave6-abnf-core-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:abnf-core
relations:
- reviews: story:abnf-core
revision: 1
---
unit: story:abnf-core, uncommitted working tree a local path (base 24ccbc4)
verdict: NEEDS-CHANGE
cases: executed 10→32, red 5
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: whether left recursion should be refused, and whether a non-import prose-val should count as unresolved (both are design calls)

**1. Diff.** `git --no-pager diff --stat` covers only tracked files, so it shows just the implementor's changes: `Cargo.lock`, `crates/rfcspec-abnf/Cargo.toml` and `src/lib.rs`. My additions are untracked and all are test files:
- `crates/rfcspec-abnf/tests/adversary_matcher.rs`
- `crates/rfcspec-abnf/tests/adversary_parse.rs`
- `crates/rfcspec-abnf/tests/adversary_depth.rs`

No implementation file was touched. I also wrote a throwaway probe, `tests/adversary_corpus_probe.rs`, that read `~/.cache/rfcspec-corpus` read-only; I deleted it. The stack-overflow runs left three `core.*` dumps in `crates/rfcspec-abnf/`; I deleted those too.

**2. Cases, each file run alone first**

`adversary_matcher.rs`: 12 cases, 4 red.
- `left_recursion_gives_the_same_answer_whichever_rule_the_match_starts_from`: grammar `a = a "x" / "y"`, `s = a`. Red: `left: Ok(2)  right: Err(Invalid { rule: "s", offset: 1 })`.
- `a_left_recursive_rule_is_matched_or_refused_never_reported_invalid_for_input_it_derives`: input `yxx`. Red: `got Err(Invalid { rule: "a", offset: 2 })`.
- `a_zero_repetition_of_a_prose_value_does_not_refuse_the_rule`: grammar `r = "a" / 0<anything at all>`. Red: `left: Err(Prose { rule: "r", text: "anything at all" })  right: Ok(1)`.
- `nested_inline_repetitions_finish_on_a_modest_input`: grammar `r = *(*(*(*(*"a"))))` on 300 `a` plus `b`. Red: `the matcher did not finish within 20 s`.
- Green: repetition forms (`*`, `n*m`, `n`, `*m`, `0`, `1*`), precedence and nested groups/options, num-vals (`%X`/`%x`, `%b`, `%d`, lowercase hex, dotted, `%x100-10FFFF`), empty `""`, LWSP/CRLF/OCTET on empty input, case-insensitive rulenames with `=/`, furthest-failure offset, and nullable loops (`*(*"a")`, `1000000000*("" / "a")`).

`adversary_parse.rs`: 8 cases, all green.
- They cover `;` inside a char-val, rule-like text inside a char-val, prose or comment, `=/` before `=`, duplicate `=` across cases, and the line/column of `)`, trailing `/` and an unclosed group.
- One case first went red because of a mistake in my test: I put a prose-val in the matched closure. I fixed the test. It is not a finding.

`adversary_depth.rs`: 2 cases.
- `nested_groups_two_hundred_deep_do_not_overflow_the_stack` is green.
- `nested_groups_two_thousand_deep_do_not_overflow_the_stack`, grammar `nest = "(" *( "x" / nest ) ")"` on 4000 bytes. Red: `thread '...' has overflowed its stack / fatal runtime error: stack overflow, aborting` (SIGABRT).

**3. Suite run**, after the cases existed: `env CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 RUST_TEST_THREADS=4 timeout 400 nice -n 10 ionice -c3 cargo test -p rfcspec-abnf --no-fail-fast`. EXIT=101.
- `adversary_depth`: aborted on the stack overflow.
- `adversary_matcher`: `FAILED. 8 passed; 4 failed`.
- `adversary_parse`: 8 passed.
- `dialect`: 2 passed.
- `rfc6455_core`: 8 passed.
- `error: 2 targets failed`.
- The "before" count of 10 is `dialect` plus `rfc6455_core` from this same run, with my three files left out.

**4. Findings.** All are in `crates/rfcspec-abnf/src/matcher.rs`. The base has no matcher, so every origin is `introduced`.

- **Left recursion gives inconsistent results** (`matcher.rs:58`, `:232`). The top rule is evaluated outside the memo, so a left-recursive rule matches one level deep when it is the entry rule and none when it is reached through a reference. So it does not "match nothing" as the implementor says, and it reports `Invalid` for input the grammar derives.
  - What reaches it: any left-recursive grammar a caller passes. None turned up in the corpus I probed.
  - Fix: refuse left recursion in `check_closure`, or seed the memo for the top rule too, which at least makes the result consistent.
  - NEEDS-CHANGE.
- **`0<prose>` refuses the whole rule** (`matcher.rs:99`). `check_closure` refuses any prose-val, even one under a zero repetition that can never be matched. RFC 3986 Appendix A defines `path-empty = 0<pchar>`, so `URI`, `URI-reference` and everything that references them cannot be matched at all. My probe got `Err(Prose { rule: "path-empty", text: "pchar" })`.
  - What reaches it: the RFC 3986 grammar, which RFC 6455 imports.
  - Fix: skip leaves under `max == Some(0)`.
  - NEEDS-CHANGE.
- **Inline repetitions nested in a rule body are not memoized** (`matcher.rs:188`). The doc comment at `:2` says the work is bounded by the input's length, but cost grows as roughly n to the power of the nesting depth. Five levels on 301 bytes did not finish in 20 s.
  - What reaches it: nothing found in real grammars, which nest through references, and those are memoized. Built by me.
  - INFEASIBLE.
- **Recursive descent overflows the stack** (`matcher.rs:127`). Group nesting 2000 deep aborts the whole process instead of returning an error.
  - What reaches it: recursive rules such as an HTTP `comment` matched against wire input. Unbounded depth is not part of the story's acceptance.
  - CONFIRMED.
- **Judgement (note), no test:** a non-import prose-val is in neither `imports()` nor `unresolved()` (`lib.rs:176`). A set can therefore look fully resolved and still be refused when matched. The story's wording "else it is listed as unresolved" can be read to cover this case.

**5. What I attacked and could not break**
- All the repetition forms listed above, precedence, and nested groups and options.
- Num-val case, dotted values, ranges above 0xFF and `%b`.
- Empty `""`.
- `;` and rule-like text inside char-vals, prose and comments.
- `=/` before `=`, duplicate `=`, and rulename case.
- LWSP, CRLF and OCTET on empty input.
- Nullable loops terminate.
- Line and column in errors.
- Indented continuations and a rule starting mid-line.
- The 6455 acceptance tests are green.
- RFC 5234 Appendix B.1 as parsed matches the built-in core rules on all 256 single bytes and 125 multi-byte whitespace inputs: 0 diffs.
- RFC 3986 Appendix A parses into 36 rules with 0 unresolved, and IPv6 matching is fast.
- The rest of the corpus fails to parse only on things outside this unit:
  - `#` lists in RFC 7230, 9110 and 9112 belong to the rfc2616 dialect;
  - regexes and examples that are not ABNF (RFC 3986 B, RFC 9110 14.1.2, RFC 5322 prose);
  - the deliberately partial `=/` examples in RFC 5234 3.3;
  - rules split at a blank line or a page break (RFC 5234 3.4 line 409, RFC 5322 `obs-zone`), which is how rfcspec-doc cuts figure blocks.

**6. Paths written outside the worktree:** none.

```findings
- file: crates/rfcspec-abnf/src/matcher.rs
  line: 58
  category: property
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a left-recursive rule matches one level deep when it is the entry rule and nothing when reached by reference, so matches reports Invalid for input the grammar derives instead of refusing the rule
- file: crates/rfcspec-abnf/src/matcher.rs
  line: 99
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: check_closure refuses a prose-val under a zero repetition such as RFC 3986 path-empty = 0<pchar>, so URI and URI-reference can never be matched
- file: crates/rfcspec-abnf/src/matcher.rs
  line: 188
  category: property
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: repetitions nested inline are not memoized, so cost grows as n to the nesting depth and r = *(*(*(*(*"a")))) on 301 bytes runs past 20 s, contrary to the module doc's bound
- file: crates/rfcspec-abnf/src/matcher.rs
  line: 127
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the recursive matcher overflows the stack and aborts the process on a recursive rule nested 2000 deep instead of returning an error
- file: crates/rfcspec-abnf/src/lib.rs
  line: 176
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a prose-val that is not an import appears in neither imports() nor unresolved(), so a set can look fully resolved while matching refuses it
```
