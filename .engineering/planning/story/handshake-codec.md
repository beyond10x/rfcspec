---
format: aep.planning-md/3
id: story:handshake-codec
kind: story
status: draft
title: Generate the RFC 6455 handshake header-value codec from the section 4.3 grammar, with round-trip and mutation vectors
relations:
- decomposes: epic:wire-layer
- depends_on: story:abnf-rfc2616
- depends_on: story:websocket-grammar-review
- depends_on: story:frame-codec-vectors
scope:
- confidence: inferred
  path: crates/rfcspec-gen/src/fields.rs
- confidence: inferred
  path: crates/rfcspec-gen/src/grammar.rs
- confidence: cited
  path: crates/rfcspec-gen/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-gen/tests/handshake_rfc6455.rs
- confidence: inferred
  path: profiles/websocket/generated/rust/src/handshake.rs
- confidence: cited
  path: profiles/websocket/generated/rust/src/lib.rs
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: website/docs/catalog/websocket.md
- confidence: inferred
  path: website/static/downloads/websocket/rust/src/handshake.rs
- confidence: cited
  path: website/static/downloads/websocket/rust/src/lib.rs
revision: 6
---
## Story

Generate a codec for the RFC 6455 handshake header-field values from the reviewed section 4.3 grammar, with the grammar vectors of design section 11 (S6, design sections 8.2 and 12.5).

- `crates/rfcspec-gen/src/grammar.rs`, called from `rfcspec_gen::rust` when the overlay reviews a grammar of dialect `rfc5234` or `rfc2616`. For each rule of the grammar whose closure resolves (the same grammar and the core rules), it generates, into the profile crate, a module `handshake` (`src/handshake.rs`, and `pub mod handshake;` in the generated `lib.rs`):
  - `parse_<rule>(&[u8])`: a complete header-field value in; a value of the rule, or `Invalid { rule, offset }`.
  - `print_<rule>(&value) -> Vec<u8>`: emits only what the parser accepts.
- Rules whose closure holds an unresolved name are not generated; the module documentation lists each with the name (`Sec-WebSocket-Extensions`, `Sec-WebSocket-Protocol-Client`, `Sec-WebSocket-Protocol-Server` and the extension rules, on `token` and `quoted-string`).
- Boundedness, as far as a terminating parser needs it (design section 8.2): generation is refused for a rule with left recursion, or with a repetition whose element can match the empty string.
- Generated tests (grammar vectors, design section 11), per generated rule:
  - the minimal sample and boundary samples (each alternative; a repetition at its minimum and at its maximum where bounded) round-trip byte for byte through print and parse;
  - single-byte mutations of each sample that the `rfcspec-abnf` matcher places outside the language are refused at the offset the matcher gives.
- Generated code carries rule names and source line numbers, no rule text. Profiles without overlay grammars (`time`, `dns`, `ntp`) generate byte-identical output.
- `profiles/websocket/overlay.yaml` gains finding `F-1` on `rfc6455/grammar/4.3`: `version` admits `"2" DIGIT DIGIT`, up to 299 (lines 1413-1414), while its comment says "Limited to 0-255 range, with no leading zeros" (line 1415). The generated parser follows the grammar.
- Regenerate `profiles/websocket/generated/rust` (`rfcspec generate`) and the site downloads (`task docs-generate`).

Not here: incremental parsing across reads (`More(state)`, design section 8.2); a codec generated from the `bits` dialect (the frame codec stays the layout codec; story:frame-agreement ties it to the bit ABNF); ambiguity analysis; header-field examples extracted mechanically from the RFC.

## Acceptance

Before: `profiles/websocket/generated/rust` has no `handshake` module. After:

- `rfcspec generate --profile profiles/websocket --check` passes with `src/handshake.rs` present, and the same check passes unchanged for `profiles/time`, `profiles/dns` and `profiles/ntp`;
- `cargo test -p rfc6455-websocket handshake` passes the generated round-trip and mutation vectors;
- `cargo test -p rfcspec-gen --test handshake_rfc6455` passes:
  - reading each RFC value from `profiles/websocket/sources/rfc6455.txt`: `parse_sec_websocket_key` accepts `dGhlIHNhbXBsZSBub25jZQ==` (line 248); `parse_sec_websocket_accept` accepts `s3pPLMBiTxaQ9kYGzzhZRbK+xOo=` (line 258); `parse_sec_websocket_version_client` accepts `13` (line 251); `parse_sec_websocket_version_server` accepts `13, 8, 7` (line 1467); each prints back to the same bytes (the list printer writes `, ` between elements);
  - on two values derived from the grammar rather than read from the RFC: `parse_sec_websocket_version_client` accepts `256`, which `"2" DIGIT DIGIT` admits (lines 1413-1414), and refuses `01` at offset 1;
- `cargo test -p rfcspec-gen grammar::refuses` passes: generation from a synthetic grammar with `a = a "x"` is refused naming `a` as left-recursive, and from one with `b = *( [ "x" ] )` is refused naming `b` as a repetition whose element matches the empty string;
- `grep -n -A1 "id: F-1" profiles/websocket/overlay.yaml` shows the finding with `item: rfc6455/grammar/4.3`, and `rfcspec check --profile profiles/websocket` exits 0 with it, which checks that the item is in the catalogue and the quote stands at its span (line 1415).

## Scope

- cited (read for this plan): `crates/rfcspec-gen/src/lib.rs`, `profiles/websocket/generated/rust/src/lib.rs`, `profiles/websocket/overlay.yaml`, `website/static/downloads/websocket/rust/src/lib.rs`, `website/docs/catalog/websocket.md`.
- inferred (new files, and where the generated `lib.rs` is written): `crates/rfcspec-gen/src/grammar.rs`, `crates/rfcspec-gen/src/fields.rs`, `crates/rfcspec-gen/tests/handshake_rfc6455.rs`, `profiles/websocket/generated/rust/src/handshake.rs`, `website/static/downloads/websocket/rust/src/handshake.rs`.

## Domain relations

- Overlay grammar entry → Grammar, many-to-one by id — inferred, as story:wire-scaffold states (`crates/rfcspec-model/src/lib.rs:347-362`); the generator reads the dialect from it.
- Finding → item `rfc6455/grammar/4.3`, many-to-one, the overlay owns it — inferable (inferred from `crates/rfcspec-model/src/lib.rs:601-611`; no `ess/1` document declares Finding).
- Profile scope holds section 4.3 after story:websocket-grammar-review (`ess/domains/profile.yaml`, `rfcspec.profile.ProfileRfc` field `sections`).
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.

## Order

- this story depends on story:frame-codec-vectors, because both edit `crates/rfcspec-gen/src/lib.rs`, `crates/rfcspec-gen/src/fields.rs`, `profiles/websocket/generated/rust/src/lib.rs` and its download copy;
- it depends on story:websocket-grammar-review for the reviewed dialect, the profile scope holding section 4.3, and findings on grammar ids;
- story:status-evidence, which rewrites `profiles/websocket/overlay.yaml` and the websocket pages, runs after it.
