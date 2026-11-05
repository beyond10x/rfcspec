---
format: aep.planning-md/3
id: story:dns-header-codec
kind: story
status: implemented
title: Generate the RFC 1035 header codec, tested with the value section 4.1.1 fixes (Z = 0)
relations:
- decomposes: epic:dns-profile
- depends_on: story:dns-source-catalogue
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
  path: profiles/dns/generated/rust
- confidence: cited
  path: profiles/dns/overlay.yaml
- confidence: cited
  path: website/docs/catalog/dns.md
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/static/downloads/dns
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:49Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:49Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-05T08:29:30Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Story

The section 4.1.1 header becomes the overlay layout `header` of the `dns` profile, and
`rfcspec generate rust` writes the crate `rfc1035-dns`: a parser and printer of the fixed
12-octet DNS header. The multi-field generator of the WebSocket wave already packs unsigned
big-endian fields of 1 to 64 bits and checks `equals`, `range` and `one_of` constraints, but
it derives tests only from octet examples and from `range` constraints (design section 11, layout
vectors). RFC 1035 has no octet example of a header, and the one value it fixes is an equality (Z
"Must be zero"), so the generated crate would carry no header test. This story adds layout vectors
for `equals` constraints, so the header is tested with the value the RFC states.

## Acceptance

`cargo test -p rfc1035-dns` passes with generated tests that accept a 12-octet header with Z = 0
and refuse one with Z = 1, and `rfcspec generate --check` passes for `dns`, `time` and
`websocket` with no change under `profiles/time/generated/` or `profiles/websocket/generated/`.

Values below are from RFC 1035 as locked by `story:dns-source-catalogue` (SHA-256
`d14ae809fc9b41bbcae26bb38c937c9515808b944f3252b00de9fe0e95f4fdfb`).

- The overlay layout `header` quotes "The header contains the following fields:" (line 1403), no
  epoch, no `length`, all fields unsigned big-endian, in this order with these widths (section
  4.1.1 diagram, lines 1405-1419, and field texts, lines 1423-1528): `id` 16, `qr` 1,
  `opcode` 4, `aa` 1, `tc` 1, `rd` 1, `ra` 1, `z` 3 with constraint `equals: 0`,
  `rcode` 4, `qdcount` 16, `ancount` 16, `nscount` 16, `arcount` 16: 96 bits, 12 octets.
  A test asserts the widths sum to 96.
- `rfcspec-gen`: for each `equals: v` constraint, the layout vectors gain v accepted and v + 1
  and v - 1 refused where the field width holds them; for `z` that is 0 accepted and 1 refused.
  The vectors are built on the all-zero header when the RFC has no octet example (the existing
  `settle` base). A unit test in `crates/rfcspec-gen` asserts the two `z` vectors and that
  the websocket layout, which has no `equals` constraint, yields the same vectors as before.
- The crate has `Header`, `parse(&[u8]) -> Result<(Header, usize), Error>` and
  `print(&Header) -> Vec<u8>`. Generated tests:
  - `vector_z_0_accepted`: the octets `000000000000000000000000` parse to the all-zero header,
    12 octets used, and print back;
  - `vector_z_1_refused`: the octets `000000100000000000000000` (Z is bits 6 to 4 of the second
    16-bit word: QR 15, OPCODE 14-11, AA 10, TC 9, RD 8, RA 7, Z 6-4, RCODE 3-0) are
    `Error::Constraint { field: "z", value: 1 }`;
  - the existing `no_octets_are_incomplete`.
- The overlay gains one requirement: id `R-HDR-1`, source `prose`, level MUST, quote "Must be
  zero in all queries and responses." at lines 1473-1474, status satisfied by `layout.header`.
  `rfcspec check` passes with it.
- `profiles/dns/generated/rust` is a workspace member; `rfcspec generate --profile profiles/dns
  --check` exits 1 on drift. `Taskfile.yml` gains a `dns` task, run by `check`, with
  `rfcspec generate --profile profiles/dns --check` and `cargo test -p rfc1035-dns`.
- The index shows stage S6 for `dns`, the page lists the crate files under downloads and the
  reviewed requirement in its matrix, and `rfcspec-docs generate --check` passes. `task check`
  exits 0.

## Domain relations

- Layout -> LayoutField, one-to-many as a value list: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Layout`, field `fields` (`List<rfcspec.catalogue.LayoutField>`); the
  `equals` constraint is `rfcspec.catalogue.ConstraintKind` in the same document.
- Profile -> generated codec crate, one layout per crate. Inferable (inferred from
  `crates/rfcspec-gen/src/lib.rs:22`, which refuses an overlay without exactly one layout; no ESS
  document declares the generated crate).
- Profile -> Status, one-to-many, owned: `ess/domains/profile.yaml`, entity
  `rfcspec.profile.Profile`, relation `statuses`; Status -> Requirement, one: entity
  `rfcspec.profile.Status`, relation `requirement`.

## Scope

`crates/rfcspec-gen/src/fields.rs` (vectors for `equals`), `crates/rfcspec-cli/tests`,
`profiles/dns/overlay.yaml`, `profiles/dns/generated/rust/`, workspace `Cargo.toml`,
`Cargo.lock`, `Taskfile.yml`, `website/docs/catalog/index.md`,
`website/docs/catalog/dns.md`, `website/static/downloads/dns/` (regenerated). Reads
`profiles/time/` and `profiles/websocket/`, writes nothing in them.

Choices taken:

- OPCODE (values 3-15) and RCODE (values 6-15) are left unconstrained: RFC 1035 calls those values
  "reserved for future use", which tells a receiver nothing to refuse. No named-value enumeration
  is added to layouts.
- Equality vectors only; `one_of` constraints get no vectors here, because no profile uses one.
- Question, resource-record and compressed-name formats (sections 4.1.2 to 4.1.4) are not in this
  story: they need the variable-length layouts of design section 8 (`rfcspec-wire/1`, gap G3).

Depends on `story:dns-source-catalogue` (the profile and overlay it creates). The machine story
`story:dns-query-machine` depends on this one: both edit `profiles/dns/overlay.yaml`,
`Taskfile.yml`, `crates/rfcspec-cli/tests` and the generated catalogue pages.
