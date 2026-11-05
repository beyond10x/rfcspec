---
format: aep.planning-md/3
id: story:abnf-core
kind: story
status: proposed
title: rfcspec-abnf parses RFC 5234 and RFC 7405 ABNF, resolves references and core rules, and matches input
relations:
- decomposes: epic:wire-layer
- depends_on: story:wire-scaffold
scope:
- confidence: inferred
  path: crates/rfcspec-abnf/src/bits.rs
- confidence: inferred
  path: crates/rfcspec-abnf/src/core.rs
- confidence: inferred
  path: crates/rfcspec-abnf/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-abnf/src/matcher.rs
- confidence: inferred
  path: crates/rfcspec-abnf/src/parse.rs
- confidence: inferred
  path: crates/rfcspec-abnf/src/rfc2616.rs
- confidence: inferred
  path: crates/rfcspec-abnf/tests/rfc6455_core.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T20:58:31Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
## Story

Fill `crates/rfcspec-abnf` (design sections 8.1, 8.3, 13) for the `rfc5234` dialect.

- Parser: figure lines with their source line numbers in, a rule set out. RFC 5234: rulenames, `=` and `=/`, alternation `/`, concatenation, repetition (`n*m`, `n`, `*`), groups, options, char-vals, num-vals (`%b`, `%d`, `%x`, with ranges and `.` concatenation), prose-vals; RFC 7405: `%s` and `%i` on char-vals. Each comment stays attached to the line it is on (story:abnf-bits reads widths from them). An error names the source line and column.
- Resolution: a reference resolves to a rule of the same set; else to an RFC 5234 Appendix B core rule (ALPHA, BIT, CHAR, CR, CRLF, CTL, DIGIT, DQUOTE, HEXDIG, HTAB, LF, LWSP, OCTET, SP, VCHAR, WSP), built into the crate as code; else, for a prose-val of the form `<rule, defined in [RFCn], ...>`, a `RuleImport { rfc: n, rule }`; else it is listed as unresolved.
- Matcher: `matches(set, rule, bytes)` matches a complete input and gives the bytes consumed or `Invalid { rule, offset }`; a rule whose closure holds an import or an unresolved name is refused, naming it. Backtracking is fine; the input is bounded by its length.
- Dispatch on `Dialect`: `bits.rs` and `rfc2616.rs` get their entry points returning an `Unsupported` error, so story:abnf-bits and story:abnf-rfc2616 each fill one file.
- No figure is filtered here: a figure that does not parse is an error its caller reports.

Not here: ambiguity and boundedness analysis (design section 8.2); resolving an import against another RFC (it is not locked, and design section 8.3 needs a reviewed successor table).

## Acceptance

Before: `crates/rfcspec-abnf` has no parser and `cargo test -p rfcspec-abnf --test rfc6455_core` does not compile. After: it passes, reading `profiles/websocket/sources/rfc6455.txt` through `rfcspec-doc`:

- the two section 3 figures (lines 741-742 and 744-747) parse as `rfc5234` into the six rules `ws-URI`, `wss-URI`, `host`, `port`, `path`, `query`, with imports `{3986, host}`, `{3986, port}`, `{3986, path-abempty}`, `{3986, query}` and no unresolved name, and matching `ws-URI` is refused naming the import `host`;
- the section 5.3 figure (lines 1826-1827, `j = i MOD 4`) is refused with an error naming line 1826;
- comments stay on their lines: the rule `frame-fin` (lines 1701-1703) parsed as `rfc5234` carries the comment `more frames of this message follow` on line 1701, `final frame of this message` on line 1702 and `1 bit in length` on line 1703;
- the dispatch stubs answer: parsing the section 5.2 rule figures with `Dialect::Bits` returns the `Unsupported` error from `bits.rs` naming `bits`, and parsing the section 4.3 figures with `Dialect::Rfc2616` returns it from `rfc2616.rs` naming `rfc2616` (story:abnf-bits and story:abnf-rfc2616 take these as their Before);
- synthetic grammars written for the test cover `%s` and `%i`, `=/`, num-val ranges and concatenations, and each core rule.

## Scope

- inferred (new files in the crate story:wire-scaffold creates): `crates/rfcspec-abnf/src/lib.rs`, `crates/rfcspec-abnf/src/parse.rs`, `crates/rfcspec-abnf/src/core.rs`, `crates/rfcspec-abnf/src/matcher.rs`, `crates/rfcspec-abnf/src/bits.rs`, `crates/rfcspec-abnf/src/rfc2616.rs`, `crates/rfcspec-abnf/tests/rfc6455_core.rs`.
- read only: `profiles/websocket/sources/rfc6455.txt`, `crates/rfcspec-doc/src/lib.rs`.

## Domain relations

- Grammar → RuleImport: the imports the resolver reports have the shape of `rfcspec.catalogue.RuleImport` (`ess/domains/catalogue.yaml`: `rfc`, `rule`), the value list `rfcspec.catalogue.Grammar` field `uses` holds. This story stores nothing in the catalogue.
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
