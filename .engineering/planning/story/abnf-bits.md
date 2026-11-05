---
format: aep.planning-md/3
id: story:abnf-bits
kind: story
status: draft
title: 'rfcspec-abnf reads the bits dialect: RFC 6455 section 5.2 with each width from its comment'
relations:
- decomposes: epic:wire-layer
- depends_on: story:abnf-core
scope:
- confidence: inferred
  path: crates/rfcspec-abnf/src/bits.rs
- confidence: inferred
  path: crates/rfcspec-abnf/tests/rfc6455_bits.rs
revision: 3
---
## Story

Fill `crates/rfcspec-abnf/src/bits.rs`: the `bits` dialect (ESS `rfcspec.catalogue.AbnfDialect` `Bits`, "the bit-level case, as in RFC 6455 section 5.2"; design sections 8.1 and 8.3).

- It parses as `rfc5234` (story:abnf-core) and adds widths read from the comments, as the RFC announces: "The length of each group of bits is indicated in a comment" (RFC 6455 lines 1527-1528).
- Width comment forms: `N bit in length` / `N bits in length`; `either A, B, or C bits in length` and `A, B, or C bits in length, respectively`, where a term may be a sum such as `7+16`; `n*8 bits in length` (a whole number of octets, no fixed width). A width comment may continue over comment-only lines.
- An element of a concatenation takes the width comment on its own line; a rule takes the width comment inside its own definition. Alternative widths align with the alternation of the rule. A terminal (num-val or range) wider than its width is refused. A comment that is no width (`; frame continuation`) stays a comment.
- Output per rule: its width (fixed, alternatives, or a multiple of 8), and per element of a concatenation its width and whether it is optional.
- An element whose comment is not one of these forms gets no width and is reported, never guessed. Design section 8.1 gives reviewed widths to S3 (the `bit-widths` task) for those.

## Acceptance

Before: parsing with `Dialect::Bits` returns `Unsupported`. After: `cargo test -p rfcspec-abnf --test rfc6455_bits` passes: the section 5.2 rule figures (lines 1687-1783 of `profiles/websocket/sources/rfc6455.txt`, read through `rfcspec-doc`) parse into 19 rules; `ws-frame` has nine elements, in order `frame-fin` 1, `frame-rsv1` 1, `frame-rsv2` 1, `frame-rsv3` 1, `frame-opcode` 4, `frame-masked` 1, `frame-payload-length` one of 7, 23 or 71, `frame-masking-key` 32 and optional, `frame-payload-data` a multiple of 8 (lines 1687-1699); `frame-payload-length-16` is 16 bits (line 1755) and `frame-payload-length-63` 64 bits (lines 1757-1758); and a synthetic 4-bit rule holding `%x10` is refused.

## Scope

- inferred (the stub story:abnf-core leaves, and a new test): `crates/rfcspec-abnf/src/bits.rs`, `crates/rfcspec-abnf/tests/rfc6455_bits.rs`.
- read only: `profiles/websocket/sources/rfc6455.txt`.

## Domain relations

- Grammar → AbnfDialect, zero or one value: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Grammar` field `dialect` (`Optional<rfcspec.catalogue.AbnfDialect>`). The dialect is the input of this parse; story:websocket-grammar-review records it for the section 5.2 grammar.
- Widths are not persisted: no `ess/1` field holds a rule width, so they live only in the parse result. Persisting reviewed widths comes with the S3 `bit-widths` task (`ess/domains/review.yaml` proposal kind `BitWidths`).
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
