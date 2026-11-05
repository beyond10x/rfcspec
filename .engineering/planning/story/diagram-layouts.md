---
format: aep.planning-md/3
id: story:diagram-layouts
kind: story
status: draft
title: rfcspec-wire reads an ASCII-art packet diagram into a catalogue layout, from text and RFCXML
relations:
- decomposes: epic:wire-layer
- depends_on: story:catalogue-grammars
scope:
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/diagram.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/layout.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-wire/tests/rfc6455_diagram.rs
- confidence: inferred
  path: ess/scenarios/extract-layout-already-exists.yaml
revision: 8
---
## Story

S2 reads an ASCII-art packet diagram into a catalogue `Layout` (ESS `rfcspec.catalogue.ExtractLayout`) that the overlay can review, and `rfcspec-wire` runs the layout checks of design section 13 on it.

- `crates/rfcspec-wire/src/diagram.rs` (the stub story:wire-scaffold leaves) fills `collect(&Document)`, giving layouts and notes:
  - A figure is a packet diagram when it has a bit ruler (a row of tens digits over a row of unit digits at a regular pitch) and `+-` borders. On RFCXML it is an `<artwork type="ascii-art">` of the same shape (kept verbatim by the reader, `Block.figure_type`).
  - Geometry: the pitch comes from the ruler (two characters per bit in RFC 6455); cells lie between `|` and `+` boundaries; a solid `+-+` border closes a field and a dashed `+ - -` border continues it on the next row; a field width is its columns divided by the pitch, summed over the rows it occupies; a field that runs into `:` rows or a dashed open end has no width.
  - Names: the cell text read top to bottom (the vertical one-bit labels F, I, N give `FIN`), normalised to lower case, spaces to `-`, parenthesised text dropped.
  - Annotations:
    - a parenthesised width such as `(4)` must equal the geometry;
    - `(a/b)` with a condition `if <name> == x/y` (spaces optional) splits the field into `<name>-<a>`, present when `<name>` equals x, and `<name>-<b>`, present when it equals y, and the geometry must equal the larger;
    - `if <name> set to v` gives presence `{field, equals: v}`;
    - `continued` marks a continuation row.
  - A diagram the reader cannot read gives no layout and one note naming its first source line and the reason; it never panics. Id `rfc<N>/layout/<section>-<k>`, k counting the readable diagrams of the section from 1; span, section and digest as for grammars.
- Layout checks (design section 13, "layout checks"), `crates/rfcspec-wire/src/layout.rs`, run on every layout the reader gives; a failing check turns the diagram into a note instead of a layout:
  - each drawn row is covered exactly by its cells: their widths sum to the ruler width, and no two cells overlap;
  - field names are unique;
  - a presence condition names an earlier field, and its value fits the width of that field;
  - only the last field may have no width;
  - every constraint value fits the width of its field.
- `crates/rfcspec-model/src/lib.rs`: `Catalogue` gains `layouts: Vec<wire::CatalogueLayout>`, `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, with every `Catalogue` struct literal the compiler names. `crates/rfcspec-extract/src/lib.rs`: `extract_reporting` fills it from `rfcspec_wire::diagram::collect` and adds each note to the problems it returns, which `rfcspec extract` prints on stderr without failing (`crates/rfcspec-cli/src/main.rs:331-334`).
- ESS scenario `ess/scenarios/extract-layout-already-exists.yaml`: `rfcspec.catalogue.ExtractLayout` twice for one id gives `extracted`, then `already-exists`, and the `rfcspec.catalogue.Layouts` view holds one row.
- `EXTRACTOR_VERSION` stays, for the reason story:catalogue-grammars gives.

Order: this story depends on story:catalogue-grammars because both edit `crates/rfcspec-extract/src/lib.rs` and `crates/rfcspec-model/src/lib.rs`.

Not here: constraints, signedness and byte order, which the figure does not state and the overlay adds; comparing a catalogue layout with an overlay layout in `rfcspec check`.

## Acceptance

Before: `rfcspec extract --profile profiles/websocket --rfc 6455 | jq ".layouts"` prints `null`. After:

- it prints one layout `rfc6455/layout/5.2-1`, spanning lines 1535-1552, with eleven fields in order: `fin` 1, `rsv1` 1, `rsv2` 1, `rsv3` 1, `opcode` 4, `mask` 1, `payload-len` 7, `extended-payload-length-16` 16 present when `payload-len` = 126, `extended-payload-length-64` 64 present when `payload-len` = 127, `masking-key` 32 present when `mask` = 1, `payload-data` with no width;
- `cargo test -p rfcspec-wire --test rfc6455_diagram` passes asserting these from the locked source, and that the first ten equal the fields of the overlay layout `frame-header` (`profiles/websocket/overlay.yaml` lines 1276-1305) in name, width and presence; a synthetic RFCXML `<artwork type="ascii-art">` of the same shape gives the same fields;
- an unreadable diagram is noted: in the same test, a copy of the section 5.2 figure whose `opcode` cell is drawn one bit narrower, so that the first row covers 31 of 32 bits, gives no layout and exactly one note naming line 1535 and `31 of 32 bits`;
- `cargo test -p rfcspec-wire layout` passes one synthetic layout per check: a presence naming a later field, a presence value 128 on the 7-bit `payload-len`, two fields named `mask`, a field without width before the last, and a constraint value 16 on a 4-bit field each give exactly one problem naming the field; the RFC 6455 frame layout gives none;
- `rfcspec extract` over `profiles/dns` (RFC 1035), `profiles/ntp` (RFC 5905) and `profiles/time` (RFC 868) exits 0, and each figure with a bit ruler and `+-` borders there gives either a layout or a note on stderr.

## Scope

- cited (read for this plan): `crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-extract/src/lib.rs`.
- inferred (the stubs story:wire-scaffold creates, and new files): `crates/rfcspec-wire/src/diagram.rs`, `crates/rfcspec-wire/src/layout.rs`, `crates/rfcspec-wire/src/lib.rs`, `crates/rfcspec-wire/tests/rfc6455_diagram.rs`, `ess/scenarios/extract-layout-already-exists.yaml`.
- read only: `profiles/websocket/sources/rfc6455.txt`, `profiles/websocket/overlay.yaml`.

## Domain relations

- Rfc → Layout, one-to-many, Rfc owns via `rfc`: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc` relation `layouts`.
- Layout → LayoutField, embedded list: `rfcspec.catalogue.Layout` field `fields`. `rfcspec.catalogue.LayoutField` field `bits` is "absent where the length depends on another field", which is how `payload-data` is written.
- LayoutField → LayoutField through presence, many-to-one within one layout: `rfcspec.catalogue.Presence` ("present when another field of the layout equals the value"). The layout check that a condition names an earlier field enforces this.
- `rfcspec.catalogue.LayoutField` field `length` (`DependentLength`) is left unset: the figure does not say which field carries the length of `payload-data`; the prose does (RFC 6455 lines 1604-1618), and the overlay `length` list already records it.
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
