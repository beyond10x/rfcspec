---
format: aep.planning-md/3
id: story:abnf-rfc2616
kind: story
status: draft
title: 'rfcspec-abnf reads the RFC 2616 dialect: the RFC 6455 section 4.3 handshake grammar'
relations:
- decomposes: epic:wire-layer
- depends_on: story:abnf-core
scope:
- confidence: inferred
  path: crates/rfcspec-abnf/src/rfc2616.rs
- confidence: inferred
  path: crates/rfcspec-abnf/tests/rfc6455_handshake.rs
revision: 3
---
## Story

Fill `crates/rfcspec-abnf/src/rfc2616.rs`: the `rfc2616` dialect (ESS `rfcspec.catalogue.AbnfDialect` `Rfc2616`; design section 8.3). RFC 6455 section 4.3 "is using ABNF syntax/rules from Section 2.1 of [RFC2616], including the implied *LWS rule" (lines 1366-1367); line 723 says the document uses both variants.

- `|` is alternation; `n#m element` is a list of elements separated by commas with optional SP or HTAB around each comma; null list elements are accepted on parse and not printed.
- A rule defined twice with identical elements is one rule with both source lines recorded (section 4.3 defines `Sec-WebSocket-Extensions` at lines 1385 and 1420); defined twice differently, it is refused naming both lines.
- Implied `*LWS`: this story admits linear white space only around `#` list separators. The general rule (LWS between any two adjacent words) needs the RFC 2616 definitions of `token` and `quoted-string`, which no profile locks; the crate documentation states the limitation.
- The matcher of story:abnf-core handles the dialect.

## Acceptance

Before: parsing with `Dialect::Rfc2616` returns `Unsupported`. After: `cargo test -p rfcspec-abnf --test rfc6455_handshake` passes, reading `profiles/websocket/sources/rfc6455.txt` through `rfcspec-doc`: the three section 4.3 figures (lines 1384-1387, 1389-1415, 1420-1423) parse into 18 distinct rules with `Sec-WebSocket-Extensions` recorded at lines 1385 and 1420; the unresolved names are exactly `token` and `quoted-string`; the matcher accepts `dGhlIHNhbXBsZSBub25jZQ==` (line 248) as `Sec-WebSocket-Key`, `13` (line 251) as `Sec-WebSocket-Version-Client` and `13, 8, 7` (line 1467) as `Sec-WebSocket-Version-Server`, refuses `01` as `version` at offset 1, and refuses `Sec-WebSocket-Protocol-Client` naming `token`.

## Scope

- inferred (the stub story:abnf-core leaves, and a new test): `crates/rfcspec-abnf/src/rfc2616.rs`, `crates/rfcspec-abnf/tests/rfc6455_handshake.rs`.
- read only: `profiles/websocket/sources/rfc6455.txt`.

## Domain relations

- Grammar → AbnfDialect: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Grammar` field `dialect`; story:websocket-grammar-review records `rfc2616` for the section 4.3 grammar.
- `token` and `quoted-string` stay unresolved names, not `rfcspec.catalogue.RuleImport` values: the text names RFC 2616 for the section (lines 1366-1367), but nothing reviewed attributes a given undefined name to it, and design section 8.3 maps such names through a reviewed successor table (RFC 2616 to RFC 9110), which is a later wave.
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
