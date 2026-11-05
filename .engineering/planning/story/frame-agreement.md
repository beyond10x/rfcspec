---
format: aep.planning-md/3
id: story:frame-agreement
kind: story
status: draft
title: rfcspec wire agree checks the RFC 6455 section 5.2 bit ABNF against the frame diagram
relations:
- decomposes: epic:wire-layer
- depends_on: story:abnf-bits
- depends_on: story:diagram-layouts
- depends_on: story:websocket-grammar-review
- informed_by: decision-blocker:wire-grammar-layout-pairing
scope:
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: inferred
  path: crates/rfcspec-cli/tests/wire.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/agree.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-wire/tests/agree.rs
revision: 6
---
## Story

Check that RFC 6455 section 5.2 bit ABNF and its frame diagram agree (design section 8.1: "RFCspec imports both and checks that they agree (field order, widths, conditions); a disagreement is a finding for review, not a silent choice").

- `crates/rfcspec-wire/src/agree.rs`: flatten a `bits` rule into fields and pair them in order with the fields of a catalogue layout.
  - An element of the concatenation is a field with its width (story:abnf-bits).
  - An element whose rule alternates on a leading terminal (`frame-payload-length = ( %x00-7D ) / ( %x7E frame-payload-length-16 ) / ( %x7F frame-payload-length-63 )`, lines 1749-1753) is a field of the leading width followed by one optional field per tagged alternative, present when the leading field equals the tag (126, 127).
  - An optional element is present under the condition its rule comment states as `present only if <rule> is <v>` (line 1761).
  - Each pair compares position, width and presence; names are not compared. A disagreement names the grammar element and its source line, and on the diagram side the layout id, the layout span and the field name: `CatalogueField` (story:wire-scaffold) carries no source line of its own, so the diagram side is named by span and field, not by a line. Chosen 2026-11-05; a per-field line would need a field on `CatalogueField` and in ESS `rfcspec.catalogue.LayoutField`.
  - A disagreement is unresolved unless an overlay finding with `item` = the layout id quotes it.
- `crates/rfcspec-cli/src/main.rs`: `rfcspec wire agree --profile <dir> --grammar <id> --rule <name> --layout <id>` (clap derive). It extracts the catalogue from the locked source, takes the dialect of the grammar from the overlay (it must be `bits`), prints one line per pair with the grammar line and the layout field, exits 0 when every pair agrees and 1 listing each unresolved disagreement.
- The RFC says the figure is authoritative in a conflict (line 1533). A reviewer records that in a finding; the check does not apply it.
- `Taskfile.yml`: the `websocket` task runs the check for the frame.
- The pairing is passed on the command line and stored nowhere, as decided on 2026-11-05 in decision-blocker:wire-grammar-layout-pairing (this story is `informed_by` it). An overlay finding may name a layout id once story:websocket-grammar-review has widened the finding items in `check.rs`.

## Acceptance

Before: `rfcspec wire agree --profile profiles/websocket --grammar rfc6455/grammar/5.2 --rule ws-frame --layout rfc6455/layout/5.2-1` fails as an unknown subcommand. After: it exits 0 and prints eleven agreeing pairs: `frame-fin`/`fin` 1, `frame-rsv1`/`rsv1` 1, `frame-rsv2`/`rsv2` 1, `frame-rsv3`/`rsv3` 1, `frame-opcode`/`opcode` 4, `frame-masked`/`mask` 1, `frame-payload-length`/`payload-len` 7, `frame-payload-length-16`/`extended-payload-length-16` 16 when 126, `frame-payload-length-63`/`extended-payload-length-64` 64 when 127, `frame-masking-key`/`masking-key` 32 when 1, `frame-payload-data`/`payload-data` whole octets. `cargo test -p rfcspec-wire --test agree` passes: the same grammar against a copy of the layout whose `opcode` is 3 bits gives exactly one disagreement, whose text names `frame-opcode` at line 1691 (4 bits) and field `opcode` of `rfc6455/layout/5.2-1` (lines 1535-1552, 3 bits), which an overlay finding on `rfc6455/layout/5.2-1` quoting it resolves. `cargo test -p rfcspec-cli --test wire` passes: a copy of the profile whose overlay gives section 5.2 the dialect `rfc5234` is refused, naming the dialect. `task websocket` passes with the new step.

## Scope

- cited (read for this plan): `crates/rfcspec-cli/src/main.rs`, `Taskfile.yml`.
- inferred (new files, and the module line): `crates/rfcspec-wire/src/agree.rs`, `crates/rfcspec-wire/src/lib.rs`, `crates/rfcspec-wire/tests/agree.rs`, `crates/rfcspec-cli/tests/wire.rs`.

## Domain relations

- Grammar ↔ Layout (the pair checked): no relation is stored. decision-blocker:wire-grammar-layout-pairing decided on 2026-11-05 that M1 stores no pairing; the command takes both ids. Persisting the pair, with its cardinality and owner, is later work with no story yet.
- Finding → item, many-to-one, the overlay owns its findings — inferable (inferred from `crates/rfcspec-model/src/lib.rs:601-611`, `Finding { id, item, quote, span, finding }`; no `ess/1` document declares Finding).
- LayoutField presence: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Presence` (`field`, `equals`), compared with the condition the grammar states.
- Grammar → AbnfDialect read from the overlay entry story:websocket-grammar-review records (inferred, as stated there).
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
