---
format: aep.planning-md/3
id: story:wire-scaffold
kind: story
status: active
title: 'Scaffold the wire layer: rfcspec-abnf and rfcspec-wire crates, catalogue and overlay wire fields, empty collectors'
relations:
- decomposes: epic:wire-layer
scope:
- confidence: cited
  path: Cargo.lock
- confidence: inferred
  path: crates/rfcspec-abnf/Cargo.toml
- confidence: inferred
  path: crates/rfcspec-abnf/src/lib.rs
- confidence: cited
  path: crates/rfcspec-cli/Cargo.toml
- confidence: cited
  path: crates/rfcspec-extract/Cargo.toml
- confidence: cited
  path: crates/rfcspec-gen/Cargo.toml
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-model/src/wire.rs
- confidence: inferred
  path: crates/rfcspec-wire/Cargo.toml
- confidence: inferred
  path: crates/rfcspec-wire/src/diagram.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/grammars.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/lib.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T20:58:30Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T21:24:56Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
---
## Story

Prefactoring for epic:wire-layer, so that later stories of the epic land on disjoint files. Nothing is filled in here, no existing output changes, and this story touches neither `crates/rfcspec-extract/src/lib.rs` nor `crates/rfcspec-extract/src/check.rs` (coordinator plan, 2026-11-05: it runs in wave 5 beside story:send-payload-type-check, which edits `check.rs`, and story:text-reader-structure, which edits `rfcspec-extract/src/lib.rs`).

- `crates/rfcspec-abnf` (design section 13): a library crate whose `lib.rs` declares `pub enum Dialect { Rfc5234, Rfc2616, Bits }` with wire names `rfc5234`, `rfc2616`, `bits` (as ESS `rfcspec.catalogue.AbnfDialect`), read and written by `Dialect::from_wire` and `Dialect::wire`. story:abnf-core adds the parser.
- `crates/rfcspec-wire` (design section 13): a library crate with two collect stubs that return empty lists:
  - `grammars::collect(&Document) -> Vec<Grammar>` (story:catalogue-grammars fills it);
  - `diagram::collect(&Document) -> (Vec<CatalogueLayout>, Vec<String>)`, layouts and notes (story:diagram-layouts fills it).
- `crates/rfcspec-model/src/wire.rs`, declared `pub mod wire;` in the model `lib.rs`: the types `Grammar { id, rfc, name, section, rules, uses: Vec<RuleImport>, span, digest }`, `RuleImport { rfc, rule }`, `CatalogueLayout { id, rfc, name, section, fields: Vec<CatalogueField>, span, digest }`, `CatalogueField { name, bits: Option<u32>, presence: Option<Presence>, constraints: Vec<Constraint> }`, `AbnfDialect` and `OverlayGrammar { id, dialect, text_digest }`. No field is added to `Catalogue` or `Overlay` here: story:catalogue-grammars adds `Catalogue.grammars`, story:diagram-layouts adds `Catalogue.layouts` and story:websocket-grammar-review adds `Overlay.grammars`, each with the struct literals it forces.
- Manifests and `Cargo.lock`, declared once so no later story edits a manifest:
  - `rfcspec-abnf`: dev-dependency `rfcspec-doc`;
  - `rfcspec-wire`: dependencies `rfcspec-model`, `rfcspec-doc`, `rfcspec-abnf`;
  - `rfcspec-extract`: adds `rfcspec-wire`;
  - `rfcspec-gen`: adds `rfcspec-abnf`, and dev-dependency `rfc6455-websocket` (path `profiles/websocket/generated/rust`);
  - `rfcspec-cli`: adds `rfcspec-abnf` and `rfcspec-wire`.
  The workspace already takes `crates/*` as members (`Cargo.toml`).

## Acceptance

Before: `cargo metadata --no-deps --format-version 1 | jq -r ".packages[].name"` lists neither `rfcspec-abnf` nor `rfcspec-wire`. After:

- `cargo metadata --no-deps --format-version 1 | jq -r ".packages[] | select(.name == \"rfcspec-wire\") | .dependencies[].name"` prints `rfcspec-abnf`, `rfcspec-doc`, `rfcspec-model`; the same query prints `rfcspec-wire` among the dependencies of `rfcspec-extract`, `rfcspec-abnf` and `rfc6455-websocket` among those of `rfcspec-gen`, `rfcspec-abnf` and `rfcspec-wire` among those of `rfcspec-cli`, and `rfcspec-doc` among those of `rfcspec-abnf`;
- `cargo test -p rfcspec-abnf dialect` passes a test that reads and writes `rfc5234`, `rfc2616` and `bits` through `Dialect` and refuses `abnf`;
- `cargo test -p rfcspec-wire stubs` passes a test that runs `grammars::collect` and `diagram::collect` over RFC 6455 read from `profiles/websocket/sources/rfc6455.txt` and gets empty lists;
- `git diff --stat` on the branch names no file under `crates/rfcspec-extract/src/`.

## Scope

- cited (read for this plan): `Cargo.lock`, `crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-extract/Cargo.toml`, `crates/rfcspec-gen/Cargo.toml`, `crates/rfcspec-cli/Cargo.toml`.
- inferred (new files): `crates/rfcspec-abnf/Cargo.toml`, `crates/rfcspec-abnf/src/lib.rs`, `crates/rfcspec-wire/Cargo.toml`, `crates/rfcspec-wire/src/lib.rs`, `crates/rfcspec-wire/src/grammars.rs`, `crates/rfcspec-wire/src/diagram.rs`, `crates/rfcspec-model/src/wire.rs`.

## Domain relations

- Rfc → Grammar, one-to-many (zero allowed), Rfc owns its grammars: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc` relation `grammars` (kind owns, cardinality many, via rfc).
- Rfc → Layout, one-to-many, Rfc owns: same file, relation `layouts`.
- Grammar → RuleImport: a value list, not a relation (`rfcspec.catalogue.Grammar` field `uses`, `rfcspec.catalogue.RuleImport` with `rfc` and `rule`).
- Layout → LayoutField, and LayoutField → Presence / FieldConstraint: embedded values (`rfcspec.catalogue.Layout` field `fields`; `rfcspec.catalogue.LayoutField`; `rfcspec.catalogue.Presence` names another field of the same layout).
- Overlay grammar entry → Grammar, many-to-one by grammar id, the profile overlay owns the entry — inferable (inferred from `crates/rfcspec-model/src/lib.rs:347-362`, where the overlay owns its authored layouts, findings and decisions keyed by item id; no `ess/1` document declares an overlay grammar entry). The value it carries is ESS `rfcspec.catalogue.Grammar` field `dialect` (`Optional<AbnfDialect>`, "Proposed in S3 and reviewed"). ESS `rfcspec.catalogue.SetGrammarDialect` takes a Current review decision, and an overlay decision must name a recorded proposal (`crates/rfcspec-cli/src/checks.rs:174`), which no S3 `dialect` task records yet; so the epic records the dialect as an authored overlay entry, as overlay layouts are authored. The decision path comes with the S3 dialect task.
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
