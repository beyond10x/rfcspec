---
format: aep.planning-md/3
id: story:websocket-frame-codec
kind: story
status: implemented
title: Generate the RFC 6455 frame-header codec from a multi-field layout, tested with the section 5.7 examples
relations:
- decomposes: epic:websocket-profile
- depends_on: story:websocket-close-machine
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
- confidence: cited
  path: crates/rfcspec-gen
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: profiles/websocket/generated/rust
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/websocket.md
- confidence: cited
  path: website/static/downloads/websocket
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T00:11:32Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-05T00:11:33Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-05T00:42:15Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Story

The section 5.2 base frame header becomes the overlay layout `frame-header`, and `rfcspec generate
rust` writes the crate `rfc6455-websocket`: a header parser and printer tested with the section 5.7
example frames, which the extractor now reads from the RFC. The generator today accepts exactly one
layout of exactly one 8, 16 or 32-bit integer field counting seconds from an epoch (RFC 868); the
frame header needs layout generalisation: several fields of 1 to 64 bits packed most significant bit
first, a field present only when another field has a given value, and value constraints, as ESS
`rfcspec.catalogue.LayoutField` already declares them (`bits`, `presence`, `constraints`). The
single-field epoch path stays, so `profiles/time/generated/rust/` does not change. The payload, its
masking (section 5.3), the bit-ABNF agreement check and `rfcspec-wire/1` (design section 8) are not
in this story.

## Acceptance

`cargo test -p rfc6455-websocket` passes with one generated test per section 5.7 example frame, and
`rfcspec generate --profile profiles/websocket --check` and `rfcspec generate --check` (time) exit 0.

- `rfcspec-model`: `Layout.epoch` becomes optional; `Field` gains an optional `presence: {field,
  equals}` and a list `constraints` of `{equals | range | one_of: [values]}` (ESS
  `rfcspec.catalogue.Presence`, `FieldConstraint`, `ConstraintKind`); an unknown key is a parse
  error. `profiles/time/overlay.yaml` parses unchanged.
- The overlay layout `frame-header` quotes "Payload length:  7 bits, 7+16 bits, or 7+64 bits" (line
  1604), all fields unsigned big-endian: `fin` 1, `rsv1` 1, `rsv2` 1, `rsv3` 1, `opcode` 4, `mask` 1,
  `payload-len` 7; `extended-payload-length-16` 16, present when `payload-len` = 126, range
  126-65535; `extended-payload-length-64` 64, present when `payload-len` = 127, range
  65536-9223372036854775807; `masking-key` 32, present when `mask` = 1.
- `rfcspec check` refuses a layout whose `presence` names a field that is not earlier in the same
  layout, and a range whose bounds do not fit the field width; one test each.
- `rfcspec extract` lists octet examples: a bulleted line whose text starts with `0x` tokens, a
  multi-octet token (`0x0100`, `0x0000000000010000`) read big-endian by its hex digits, a trailing
  `[N bytes of ...]` recorded as N elided octets, a continuation line joined. On RFC 6455 there are
  exactly 8, at lines 2111, 2115-2116, 2120, 2122, 2137-2138, 2140-2141, 2145 and 2149. RFC 868
  yields none, and its 5 numeric examples are unchanged.
- The crate has `FrameHeader`, `parse(&[u8]) -> Result<(FrameHeader, usize), Error>` (the header and
  the octets it used; too few octets is `Error::Incomplete`, a violated constraint
  `Error::Constraint` naming the field), `print(&FrameHeader) -> Vec<u8>` and the effective payload
  length. Generated tests, one per example, each also asserting that printing the parsed header
  gives back the header octets:
  - `81 05 48 65 6c 6c 6f`: fin 1, opcode 1, mask 0, length 5, header 2 octets, the remaining 5
    octets are `Hello`;
  - `81 85 37 fa 21 3d 7f 9f 4d 51 58`: mask 1, masking key `37 fa 21 3d`, length 5, header 6 octets;
  - `01 03 48 65 6c`: fin 0, opcode 1, length 3; `80 02 6c 6f`: fin 1, opcode 0, length 2;
  - `89 05 48 65 6c 6c 6f`: opcode 9, length 5; `8a 85 37 fa 21 3d 7f 9f 4d 51 58`: opcode 10,
    masked, length 5;
  - `82 7e 01 00` and 256 elided octets: opcode 2, `payload-len` 126, length 256, header 4 octets;
  - `82 7f 00 00 00 00 00 01 00 00` and 65536 elided octets: `payload-len` 127, length 65536, header
    10 octets.
- Generated constraint tests (design section 11, layout vectors): `81 7e 00 7c`, the encoding of
  length 124 as 126, 0, 124 that section 5.2 (lines 1611-1614) forbids, is `Error::Constraint` on
  `extended-payload-length-16`; 65535 in the 64-bit field and 9223372036854775808 (most significant
  bit set) are refused; 125 in `payload-len` is accepted.
- The overlay accepts, satisfied by `layout.frame-header`: section 5.2 "If 127, the following 8 bytes
  interpreted as a 64-bit unsigned integer (the most significant bit MUST be 0) are the payload
  length." (lines 1608-1610) and the sentence at lines 1611-1614 that the minimal number of bytes
  MUST be used to encode the length.
- `profiles/websocket/generated/rust` is a workspace member; `rfcspec generate --check` exits 1 on
  drift for either profile. `Taskfile.yml` `check` runs `rfcspec generate --profile
  profiles/websocket --check` and `cargo test -p rfc6455-websocket` beside the time steps.
- The index shows stage S6 for `websocket`, the page lists the crate files under downloads, and
  `rfcspec-docs generate --check` passes.

## Domain relations

- Layout -> LayoutField, one-to-many as a value list: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Layout`, field `fields: List<rfcspec.catalogue.LayoutField>`.
- LayoutField -> LayoutField through `presence`, zero or one, by field name within the same layout:
  `ess/domains/catalogue.yaml`, type `rfcspec.catalogue.Presence` ("present when another field of
  the layout equals the value"); a field, no `relations:` entry.
- Profile -> generated codec crate, one layout per crate. Inferable (inferred from
  `crates/rfcspec-gen/src/lib.rs:15`, which refuses any overlay without exactly one layout; no ESS
  document declares the generated crate).
- Rfc -> example, one-to-many, the catalogue of one RFC holds its examples. Inferable (inferred from
  `crates/rfcspec-model/src/lib.rs`, `Catalogue.examples`; no `ess/1` document declares an example
  noun).

## Scope

`crates/rfcspec-model` (`Layout`, `Field`, octet examples on `Catalogue`), `crates/rfcspec-extract`
(octet examples; `check.rs` layout rules), `crates/rfcspec-gen` (multi-field path),
`crates/rfcspec-cli` (`generate`, tests), `profiles/websocket/overlay.yaml`,
`profiles/websocket/generated/rust/`, workspace `Cargo.toml`, `Cargo.lock`, `Taskfile.yml`,
`website/docs/catalog/index.md`, `website/docs/catalog/websocket.md`,
`website/static/downloads/websocket/` (regenerated). Reads `profiles/time/`, writes nothing in it.

Depends on `story:websocket-close-machine`: both edit `profiles/websocket/overlay.yaml`,
`crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-cli`,
`Taskfile.yml`, `Cargo.lock` and the generated catalogue pages, so they cannot run in parallel.
