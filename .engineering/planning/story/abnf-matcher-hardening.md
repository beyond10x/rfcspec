---
format: aep.planning-md/3
id: story:abnf-matcher-hardening
kind: story
status: draft
title: rfcspec-abnf limits recursion by stack and bounds inline repetition cost
revision: 1
---
## Story

`rfcspec-abnf` matches and parses without recursion limits tied to the stack, and without exponential cost on inline nesting. Left after story:abnf-core and review-result:wave6-abnf-core-adversary-1 (2026-11-06):

- `MATCH_DEPTH_LIMIT` (1000) counts nested nodes, not stack bytes: on a 1 MiB thread a 200-deep match still overflows and aborts the process.
- The parser has no depth limit: deeply nested groups in a figure recurse without bound.
- Repetitions nested inside one rule are not memoized; `r = *(*(*(*(*"a"))))` on 301 bytes runs past 20 s (`tests/adversary_matcher.rs`, `nested_inline_repetitions_finish_on_a_modest_input`, ignored).
- `unresolved()` lists a prose-val under a zero-maximum repetition (RFC 3986 `path-empty = 0<pchar>`), although `matches` ignores it.

## Acceptance

- A match or parse on a thread with `RUST_MIN_STACK=1048576` returns an error instead of aborting for nesting 2000 deep.
- `nested_inline_repetitions_finish_on_a_modest_input` passes with its `#[ignore]` removed.
- `unresolved()` on RFC 3986 Appendix A does not list `<pchar>`.

## Scope

- `crates/rfcspec-abnf/src/matcher.rs`
- `crates/rfcspec-abnf/src/parse.rs`
- `crates/rfcspec-abnf/src/lib.rs`
