---
format: aep.planning-md/3
id: story:frame-codec-vectors
kind: story
status: draft
title: Round-trip and mutation vectors for the generated WebSocket frame codec, from its layout
relations:
- decomposes: epic:wire-layer
- depends_on: story:diagram-layouts
- depends_on: story:websocket-grammar-review
scope:
- confidence: cited
  path: crates/rfcspec-gen/src/fields.rs
- confidence: cited
  path: crates/rfcspec-gen/src/lib.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: profiles/websocket/generated/rust/src/lib.rs
- confidence: cited
  path: profiles/websocket/profile.yaml
- confidence: cited
  path: website/static/downloads/websocket/rust/src/lib.rs
revision: 10
---
## Story

Give the existing generated frame codec (`rfc6455-websocket`, generated from the overlay layout `frame-header`) the round-trip and mutation vectors milestone M1 promises for the frame side ("generated codec with round-trip and mutation vectors", design section 14; scope critic, review-result:wire-scope-round-1 finding 1). Today the crate has one test per octet example and the boundary vectors of design section 11, and no mutation vector.

- `crates/rfcspec-gen/src/fields.rs`, the multi-field path, generates, for the layouts of a profile whose `profile.yaml` sets `codec_vectors: true` (only `profiles/websocket` in M1):
  - a round-trip test per accepted header (each octet example and each accepted layout vector): parsing the printed header gives the same header, and printing it gives the same octets;
  - mutation vectors: for each accepted layout vector, one test per bit of its header octets with that bit flipped. The expected outcome is computed by the independent decoder of `fields.rs` (as for the existing vectors, never by the generated code): accepted with the changed field value and a byte-for-byte round-trip, `Error::Incomplete`, or `Error::Constraint` naming the field. Test names `mutation_<vector>_bit_<n>`, bits counted from the most significant bit of the first octet.
- Only the websocket profile opts in, so the generated `rfc1035-dns`, `rfc5905-ntp` and `rfc868-time` crates and their downloads stay byte-identical. Extending the vectors to DNS and NTP is outside M1 (design §14 names only RFC 6455); a later story may set the flag there. Revised 2026-11-05 after review-result:wire-scope-round-2.
- Regenerate the profile crates (`rfcspec generate`) and the site downloads (`task docs-generate`).

Order:
- this story depends on story:diagram-layouts, which shows the overlay `frame-header` the vectors are generated from equal to the diagram field for field;
- story:handshake-codec depends on this story, because both edit `crates/rfcspec-gen/src/lib.rs`, `crates/rfcspec-gen/src/fields.rs` and the generated websocket `lib.rs`.

Not here: a codec generated from the `bits` dialect of the frame grammar; story:frame-agreement ties the layout to the bit ABNF instead.

## Acceptance

Before: `cargo test -p rfc6455-websocket mutation` runs 0 tests. After:

- it runs one test per accepted layout vector and header bit, and passes. Among them, on `vector_payload_len_125` (octets `81 7d`, built on the first example, RFC 6455 line 2111):
  - `mutation_payload_len_125_bit_8` flips the MASK bit, giving `81 fd`, and expects `Error::Incomplete`, because a masking key is now due;
  - `mutation_payload_len_125_bit_9` gives `81 3d` and expects `payload_len` 61, printed back to `81 3d`;
- `cargo test -p rfc6455-websocket roundtrip` passes one test per accepted header;
- `rfcspec generate --check` passes for `profiles/websocket`, `profiles/dns`, `profiles/ntp` and `profiles/time`, and `git diff --stat profiles/dns profiles/ntp profiles/time website/static/downloads/dns website/static/downloads/ntp website/static/downloads/time` prints nothing.

## Scope

- cited (read for this plan): `crates/rfcspec-gen/src/fields.rs`, `crates/rfcspec-gen/src/lib.rs`, `profiles/websocket/generated/rust/src/lib.rs`, `website/static/downloads/websocket/rust/src/lib.rs`, `profiles/websocket/profile.yaml` (the `codec_vectors: true` flag), `crates/rfcspec-model/src/lib.rs` (the `codec_vectors` field on `Profile`, lines 38-45).

## Domain relations

- Layout → LayoutField, embedded, with Presence and FieldConstraint: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Layout` field `fields`, `rfcspec.catalogue.Presence`, `rfcspec.catalogue.FieldConstraint`. The vectors exercise each.
- The overlay layout `frame-header` is an authored overlay entry (`crates/rfcspec-model/src/lib.rs:519-533`, inferred: no `ess/1` document declares an overlay layout apart from the catalogue `Layout`).
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.

## Order

This story depends on story:websocket-grammar-review, because both edit `profiles/websocket/profile.yaml` and `crates/rfcspec-model/src/lib.rs`. That story adds section 4.3 to the profile and the `Overlay.grammars` field; this one adds `codec_vectors: true` to the profile, which needs a `codec_vectors` field on `Profile`, a struct with `#[serde(deny_unknown_fields)]` (`crates/rfcspec-model/src/lib.rs:38-45`). Both also run `task docs-generate`. It still depends on story:diagram-layouts (see the Order list above), and story:handshake-codec still depends on it.
