---
format: aep.planning-md/3
id: story:rfc868-generate-codec
kind: story
status: implemented
title: Generate the Time Protocol codec with the RFC's examples as tests
relations:
- decomposes: epic:rfc868-thin-slice
- depends_on: story:rfc868-lower-protospec
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-04T22:38:35Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

`rfcspec generate rust` writes a small codec crate from the overlay's layout, with the RFC's own
examples as tests.

## Acceptance

- `rfcspec generate rust --profile profiles/time --out profiles/time/generated/rust` writes a crate
  `rfc868-time` with `Time(u32)`, `parse(&[u8]) -> Result<Time, Error>` (exactly 4 octets, big
  endian; any other length is an error) and `print(Time) -> [u8; 4]`.
- The crate's generated tests assert the four examples that fit 32 bits (2208988800 = 1970-01-01,
  2398291200 = 1976-01-01, 2524521600 = 1980-01-01, 2629584000 = 1983-05-01) against Unix time,
  and `parse(print(x)) == x` for them.
- The fifth example, which the overlay already marks as a review finding (extract story), generates
  no test; the generator reads the finding and skips the example by it.
- The generated crate is a workspace member; `cargo test -p rfc868-time` passes; `rfcspec generate
  --check` exits 1 on drift. `Taskfile.yml` `check` runs `rfcspec generate --check` and
  `cargo test -p rfc868-time`.

## Scope

`crates/rfcspec-gen`, `crates/rfcspec-cli` (`generate`), `Taskfile.yml`, `profiles/time/generated/rust/`,
workspace `Cargo.toml`, `Cargo.lock`. Reads `profiles/time/overlay.yaml`, writes nothing in it.

Depends on the lowering story (shared `crates/rfcspec-cli`, `Cargo.toml`, `Cargo.lock`).
