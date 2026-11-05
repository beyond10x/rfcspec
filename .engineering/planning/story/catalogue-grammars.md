---
format: aep.planning-md/3
id: story:catalogue-grammars
kind: story
status: proposed
title: S2 collects the ABNF figures of each section into a catalogue grammar, from text and RFCXML
relations:
- decomposes: epic:wire-layer
- depends_on: story:wire-scaffold
- depends_on: story:text-reader-structure
scope:
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-wire/src/grammars.rs
- confidence: inferred
  path: ess/scenarios/extract-grammar-already-exists.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T20:58:31Z", actor: "human:timo", revision: 8}
---
## Story

S2 collects, per numbered section, the ABNF figures into one catalogue `Grammar` (design sections 4 and 5).

- `crates/rfcspec-wire/src/grammars.rs` (the stub story:wire-scaffold leaves) fills `collect(&Document)`:
  - A figure is ABNF by its `type` on RFCXML (`abnf`, kept verbatim by the reader, `Block.figure_type`), else when one of its lines defines a rule (`rfcspec_doc::abnf_rule`). This is the same test `extract_reporting` applies to ABNF comments (`crates/rfcspec-extract/src/lib.rs:152-156`).
  - Id `rfc<N>/grammar/<section number>`; `name` the section number; `rules` the figure lines verbatim, joined by newlines, with page furniture dropped as the reader gives it; span from the first to the last figure line; `section` the anchor (XML) or the number (text), as ESS `rfcspec.catalogue.Span` field `section` says; `digest` = `text_digest(rules)`; `uses` empty (story:abnf-core reports imports at parse time; ESS `ExtractGrammar` takes `uses` as input, and an empty list is valid).
  - Collection is mechanical, so no figure is dropped for failing to parse. A pseudo-code figure such as RFC 6455 section 5.3 stays a catalogue grammar that no overlay reviews.
  - One grammar per section follows design section 12.2 (handshake grammar section 4.3, frame grammar section 5.2) and the text itself, which states the dialect per section (RFC 6455 lines 1364-1367 and 1525-1528).
- `crates/rfcspec-model/src/lib.rs`: `Catalogue` gains `grammars: Vec<wire::Grammar>`, `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, and every `Catalogue` struct literal the compiler names gains it.
- `crates/rfcspec-extract/src/lib.rs`: `extract_reporting` fills it from `rfcspec_wire::grammars::collect`.
- ESS scenario `ess/scenarios/extract-grammar-already-exists.yaml`: `rfcspec.catalogue.ExtractGrammar` twice for one id gives `extracted`, then `already-exists`, and the `rfcspec.catalogue.Grammars` view holds one row.
- `EXTRACTOR_VERSION` (`crates/rfcspec-extract/src/lib.rs:38`) stays: it versions the bulk summary rows (`crates/rfcspec-cli/src/bulk.rs:236-237, 339`), which count requirements only, and requirement extraction does not change.

Order: this story depends on story:text-reader-structure because both edit `crates/rfcspec-extract/src/lib.rs`, and on story:wire-scaffold for the stub and the model types.

## Acceptance

Before: `rfcspec extract --profile profiles/websocket --rfc 6455 | jq ".grammars"` prints `null`. After:

- it prints six grammars in document order, with ids `rfc6455/grammar/3`, `rfc6455/grammar/4.2.2`, `rfc6455/grammar/4.3`, `rfc6455/grammar/5.2`, `rfc6455/grammar/5.3`, `rfc6455/grammar/9.1`;
- `4.3` spans lines 1384-1423, and its rules hold the three figures (1384-1387, 1389-1415, 1420-1423) and none of the prose lines 1416-1419;
- `5.2` spans lines 1687-1783 and holds the 19 rule figures, not the diagram at 1535-1552;
- `cargo test -p rfcspec-wire grammars` passes the same on a synthetic RFCXML document with a `<sourcecode type="abnf">` (id from the section number, span section from the anchor);
- `ess specify validate --path ess` passes with the scenario;
- `cargo run -q -p rfcspec-cli -- bulk check --summary bulk/rfc-summary.jsonl --profiles profiles --sections bulk/rfc-sections.jsonl` still passes.

## Scope

- cited (read for this plan): `crates/rfcspec-model/src/lib.rs` (`Catalogue` at 263), `crates/rfcspec-extract/src/lib.rs` (`extract_reporting` at 146, the `Catalogue` literal at 277).
- inferred (the stub story:wire-scaffold creates, and a new file): `crates/rfcspec-wire/src/grammars.rs`, `ess/scenarios/extract-grammar-already-exists.yaml`.

## Domain relations

- Rfc → Grammar, one-to-many, Rfc owns via `rfc`: `ess/domains/catalogue.yaml`, `rfcspec.catalogue.Rfc` relation `grammars`. A grammar lives and goes stale with its RFC (`rfcspec.catalogue.Grammar` lifecycle Read and Stale; the digest is what `MarkGrammarStale` compares later).
- Section → Grammar: no `ess/1` relation; a grammar records its section through `rfcspec.catalogue.Span` field `section`. One grammar per section is an extraction rule (design section 12.2), not a stored relation.
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
