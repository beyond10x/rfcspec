---
format: aep.planning-md/3
id: story:websocket-grammar-review
kind: story
status: draft
title: The websocket overlay reviews its handshake and frame grammars, and section 4.3 joins the profile
relations:
- decomposes: epic:wire-layer
- depends_on: story:catalogue-grammars
- depends_on: story:xml-overlay-check
- depends_on: story:diagram-layouts
scope:
- confidence: cited
  path: crates/rfcspec-cli/src/checks.rs
- confidence: cited
  path: crates/rfcspec-cli/tests/adversary_timer.rs
- confidence: cited
  path: crates/rfcspec-cli/tests/check.rs
- confidence: inferred
  path: crates/rfcspec-docs/src/explorer_profile_tests.rs
- confidence: cited
  path: crates/rfcspec-ess/src/lib.rs
- confidence: cited
  path: crates/rfcspec-extract/src/check.rs
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: profiles/websocket/overlay.yaml
- confidence: cited
  path: profiles/websocket/profile.yaml
- confidence: cited
  path: website/docs/catalog/websocket.md
- confidence: cited
  path: website/static/data/profiles/websocket.json
revision: 7
---
## Story

Review the two RFC 6455 grammars M1 needs in the websocket overlay, and bring section 4.3 into the profile.

- `crates/rfcspec-model/src/lib.rs`: `Overlay` gains `grammars: Vec<wire::OverlayGrammar>`, `#[serde(default, skip_serializing_if = "Vec::is_empty")]`, and every `Overlay` struct literal gains it: `crates/rfcspec-extract/src/check.rs:731`, `crates/rfcspec-ess/src/lib.rs:1253` and `1735`, `crates/rfcspec-cli/tests/adversary_timer.rs:126`, and any other the compiler names.
- `profiles/websocket/profile.yaml`: `sections` gains `"4.3"`. A non-empty list narrows the scope to those sections (`ess/domains/profile.yaml:26-28`), and the handshake grammar lies in section 4.3.
- `profiles/websocket/overlay.yaml`:
  - `R4.3-1`, the abnf-comment MUST at lines 1408-1410 (the value after quoted-string unescaping must conform to token), gets status `unsupported`: `extension-param` rests on `token` and `quoted-string` from RFC 2616, which the profile does not lock.
  - `grammars:` gains `{id: rfc6455/grammar/4.3, dialect: rfc2616, text_digest}` and `{id: rfc6455/grammar/5.2, dialect: bits, text_digest}`, each dialect as the text states it (lines 1366-1367; lines 1525-1528).
- `crates/rfcspec-extract/src/check.rs`:
  - an overlay grammar is refused when its id names no catalogue grammar, or when its `text_digest` differs from `text_digest` of the grammar rules (it judged another text, design D6);
  - a finding may name a catalogue grammar id or layout id as its `item` (today only examples, prose candidates, requirements and dialogue steps count, `check.rs:388-403`); story:frame-agreement and story:handshake-codec record findings on them.
- `crates/rfcspec-cli/src/checks.rs`: an overlay grammar is refused when its section lies outside the profile scope.
- Regenerate the pages that read the profile and its overlay with `task docs-generate`.

Order, for shared files:
- this story depends on story:xml-overlay-check because both edit `crates/rfcspec-extract/src/check.rs`, the last in the chain send-payload-type-check → fixed-arm-bounds-check → xml-overlay-check → websocket-grammar-review;
- it depends on story:diagram-layouts because both edit `crates/rfcspec-model/src/lib.rs`, and the finding items need the catalogue layouts;
- it depends on story:catalogue-grammars for the catalogue grammars it checks;
- story:status-evidence, which rewrites `profiles/websocket/overlay.yaml`, `check.rs` and the websocket pages, runs after it.

## Acceptance

Before: `rfcspec check --profile profiles/websocket` passes with section 4.3 outside the profile and no grammar reviewed. After:

- it passes with section 4.3 in scope, `R4.3-1` holding a status and both overlay grammars checked;
- `cargo test -p rfcspec-cli --test check` passes cases on copies of `profiles/websocket`:
  - the `4.3` entry carrying another `text_digest` fails, naming `rfc6455/grammar/4.3`;
  - an entry naming `rfc6455/grammar/5.9` fails, naming it;
  - `profile.yaml` without `"4.3"` fails, naming the out-of-scope grammar;
  - a finding with `item: rfc6455/layout/5.2-1` passes, and one with `item: rfc6455/layout/9.9-1` fails as not in the catalogue;
- `task docs-check` passes.

## Scope

- cited (read for this plan): `crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-ess/src/lib.rs`, `crates/rfcspec-cli/tests/adversary_timer.rs`, `profiles/websocket/profile.yaml`, `profiles/websocket/overlay.yaml`, `crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-cli/src/checks.rs`, `crates/rfcspec-cli/tests/check.rs`, `website/docs/catalog/websocket.md`, `website/static/data/profiles/websocket.json`.
- inferred (may pin counts that change): `crates/rfcspec-docs/src/explorer_profile_tests.rs`.

## Domain relations

- Profile → sections in scope: `ess/domains/profile.yaml`, `rfcspec.profile.ProfileRfc` field `sections` ("Empty means the whole RFC; a non-empty list narrows the scope to those sections").
- Overlay grammar entry → Grammar, many-to-one by id, the profile overlay owns the entry — inferable (inferred from `crates/rfcspec-model/src/lib.rs:347-362`, the overlay owning authored entries keyed by item id; no `ess/1` document declares an overlay grammar entry). The value it reviews is `rfcspec.catalogue.Grammar` field `dialect` (`ess/domains/catalogue.yaml`); story:wire-scaffold says why it is not an ESS review decision yet.
- Finding → item, many-to-one, the overlay owns its findings — inferable (inferred from `crates/rfcspec-model/src/lib.rs:601-611` and `crates/rfcspec-extract/src/check.rs:388-403`; no `ess/1` document declares Finding). This story widens the items a finding may name to grammars and layouts, both owned by the RFC (`rfcspec.catalogue.Rfc` relations `grammars`, `layouts`).
- Profile → Status, one-to-many, Profile owns: `ess/domains/profile.yaml`, `rfcspec.profile.Profile` relation `statuses` (owns, many, via profile); Status → Requirement, one: `rfcspec.profile.Status` relation `requirement` (references, cardinality one). `R4.3-1` gets its status under this.
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
