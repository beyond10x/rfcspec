---
format: aep.planning-md/3
id: story:ntp-header-codec
kind: story
status: proposed
title: Generate the RFC 5905 header codec with signed Poll and Precision, tested with the version range 1-4
relations:
- decomposes: epic:ntp-profile
- depends_on: story:ntp-source-catalogue
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-gen/src/fields.rs
- confidence: cited
  path: profiles/ntp/generated/rust
- confidence: cited
  path: profiles/ntp/overlay.yaml
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/ntp.md
- confidence: cited
  path: website/static/downloads/ntp
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T13:50:22Z", actor: "human:timo", revision: 5}
---
## Story

The section 7.3 basic header becomes the overlay layout `header` of the `ntp` profile, and
`rfcspec generate rust` writes the crate `rfc5905-ntp`: a parser and printer of the fixed
48-octet NTP header. The multi-field generator packs unsigned big-endian fields of 1 to 64 bits
and refuses a signed field (`crates/rfcspec-gen/src/fields.rs`, "must be unsigned and
big-endian"); RFC 5905 defines Poll and Precision as "8-bit signed integer" (lines 1164 and 1168).
This story adds two's-complement signed fields to the generator, and makes the base header of the
layout vectors satisfy every constraint, because the all-zero base `settle` builds today has
version 0, which the version range refuses.

## Outcome

`profiles/ntp/generated/rust` holds the generated `rfc5905-ntp` crate; its tests cover the version
range section 9.2 states and the signed Poll and Precision fields, and the `ntp` gate task runs
them.

## Acceptance

`cargo test -p rfc5905-ntp` exits 0 with generated tests that accept versions 1 and 4, refuse 0
and 5, and round-trip Poll and Precision at -128, -1 and 127; `rfcspec generate --check` exits 0
for `dns`, `ntp`, `time` and `websocket` with no change under the other three profiles'
`generated/`.

Values below are from RFC 5905 as locked by `story:ntp-source-catalogue` (SHA-256
`8b7abd903c60202e2953ee012fefa6916666c3b0c71825c748cdd262024fb268`).

- The overlay layout `header` quotes "The basic header extends from the beginning of the packet
  to the end of the Transmit Timestamp field." (lines 1083-1084), no epoch, no `length`, all
  fields big-endian, in this order with these widths (Figure 8, lines 1015-1059, and the field
  texts, lines 1089-1257): `leap` 2, `version` 3 with constraint `range: [1, 4]`, `mode` 3,
  `stratum` 8, `poll` 8 signed, `precision` 8 signed, `rootdelay` 32, `rootdisp` 32, `refid`
  32, `reftime` 64, `org` 64, `rec` 64, `xmt` 64; the others unsigned. 384 bits, 48 octets. A
  test asserts the widths sum to 384.
- `rfcspec-gen`, signed fields: a field with `signed: true` of 2 to 64 bits parses as a
  two's-complement value into the narrowest of `i8`, `i16`, `i32`, `i64` that holds it and
  prints back; a 1-bit signed field is refused by `rfcspec generate` naming the layout and field (in `fields.rs`, beside the existing width refusal, so this story leaves `crates/rfcspec-extract/src/check.rs` to `story:ntp-timer-model`).
  For each signed field the layout vectors gain the width's minimum, -1 and maximum, accepted
  and printed back byte for byte. A unit test asserts the `poll` vectors (octet 2 of the header
  is `80`, `ff`, `7f`).
- `rfcspec-gen`, base header: the base the vectors are built on sets each constrained field to
  its smallest admitted value (`range` low, `equals` value) instead of 0. For `version` that
  is 1. A unit test asserts the `version` vectors: 1 and 4 accepted, 0 and 5 refused as
  `Error::Constraint { field: "version", value: 0 }` and `value: 5` (version is bits 5 to 3 of
  octet 0: LI 7-6, VN 5-3, Mode 2-0; the first octet of the base is `08`). The websocket and dns
  layouts yield the same vectors as before (their constrained fields admit 0, or the generated
  tests are byte-identical); `rfcspec generate --check` on them shows no drift.
- The crate has `Header`, `parse(&[u8]) -> Result<(Header, usize), Error>` and
  `print(&Header) -> Vec<u8>`; 47 octets are `Incomplete`.
- The overlay gains one requirement: id `R-VN-1`, source `prose`, level MUST, quote "Format
  checks require correct field length and alignment, acceptable version number (1-4)" (lines
  1812-1814), status satisfied by `layout.header`. `rfcspec check` exits 0 with it.
- `profiles/ntp/generated/rust` is a workspace member; `rfcspec generate --profile profiles/ntp
  --check` exits 1 on drift. `Taskfile.yml` gains an `ntp` task, run by `check`, with
  `rfcspec generate --profile profiles/ntp --check` and `cargo test -p rfc5905-ntp`.
- The index shows stage S6 for `ntp`, the page lists the crate files under downloads and the
  reviewed requirement in its matrix; `rfcspec-docs generate --check` exits 0. `task check`
  exits 0.

## Scope

`crates/rfcspec-gen/src/fields.rs` (signed fields, settled base, the 1-bit signed refusal),
`crates/rfcspec-cli/tests`, `profiles/ntp/overlay.yaml`, `profiles/ntp/generated/rust/`,
workspace `Cargo.toml`, `Cargo.lock`, `Taskfile.yml`, `website/docs/catalog/index.md`,
`website/docs/catalog/ntp.md`, `website/static/downloads/ntp/` (regenerated). Reads `profiles/time/`,
`profiles/websocket/` and `profiles/dns/`, writes nothing in them.

Choices taken:

- `version` is constrained to 1-4 as section 9.2 states, not to 4 ("currently 4", line 1105).
- `leap`, `mode` and `stratum` are left unconstrained: every value of their width is named in
  Figures 9, 10 and 11 (reserved values included). No named-value enumeration is added.
- Root delay and root dispersion stay 32-bit unsigned integers; their NTP short format (16.16
  fixed point, section 6) and the 64-bit timestamp format are not decoded.
- Extension fields, Key Identifier and Message Digest (section 7.5 and the optional MAC) are not in
  this story: they are variable-length (`rfcspec-wire/1`, gap G3).

Depends on `story:ntp-source-catalogue` (the profile and overlay it creates).
