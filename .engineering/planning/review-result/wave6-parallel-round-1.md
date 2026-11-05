---
format: aep.planning-md/3
id: review-result:wave6-parallel-round-1
kind: review-result
status: active
title: 'Wave 6 plan critic, parallel safety, round 1: approve'
relations:
- reviews: story:abnf-core
- reviews: story:catalogue-grammars
- reviews: story:fixed-arm-bounds-check
- reviews: story:explorer-look
revision: 1
---
approve

I checked the four stories on tree `wave-plan` (main `bab7197`) and found no shared file or state that stops them running at the same time.

- **Cargo.toml and Cargo.lock.** Neither is touched. `rfcspec-wire` already depends on `rfcspec-abnf`, and `rfcspec-extract` already depends on `rfcspec-wire` (`crates/rfcspec-wire/Cargo.toml`, `crates/rfcspec-extract/Cargo.toml`). `rfcspec-abnf` already has `rfcspec-doc` as a dev-dependency, which abnf-core's test needs. `rfcspec-wire` has nothing that names an `rfcspec-abnf` item, so wire's `grammars.rs` and the abnf crate share no source line.
- **`Catalogue` struct literals.** There is one literal, at `crates/rfcspec-extract/src/lib.rs:277`, plus the declaration at `crates/rfcspec-model/src/lib.rs:264`. Every other `Catalogue {` hit is a function return type. Only catalogue-grammars edits these. fixed-arm touches `check.rs`, which builds no `Catalogue` literal.
- **Same-crate files.** fixed-arm edits `crates/rfcspec-extract/src/check.rs`; catalogue-grammars edits `crates/rfcspec-extract/src/lib.rs`. They compile together but share no line.
- **Generated website data.** `website/docs/catalog/` pages are rendered from catalogue fields by `rfcspec-docs`, which never reads `grammars` (`crates/rfcspec-docs/src/catalog.rs:143-149`). `website/data/status.json` is static text. `bulk/rfc-summary.jsonl` counts requirements only. No story changes any of these.
- **Explorer build output.** `/website/static/explorer/`, `/explorer/node_modules/` and `/target/` are all gitignored, and each tree has its own. explorer-look touches only `explorer/src/` and reads the site's CSS without editing it.
- **Planning store.** Evidence is kept per story (`.engineering/evidence/story/<id>/`), and there is no shared journal file. The tree has uncommitted changes to `.engineering/waves/plan-2026-11-05.md` and the new `wave-6.md` (opening-commit material, not unit scope).

**What I read.** Four stories via `aep plan artifact show`, the wave proposal, and these files: root and crate `Cargo.toml`, `Taskfile.yml`, `.gitignore`, `rfcspec-wire/src` and `rfcspec-wire/tests`, `rfcspec-abnf/src` and `rfcspec-abnf/tests`, and the extract, model, docs and propose call sites of `Catalogue`.

**Surfaces.**
- Cited: fixed-arm and explorer-look.
- Cited and inferred: catalogue-grammars.
- Inferred only: abnf-core, whose files are new in an existing crate.
- Not placeable: 0.
- Acceptance reads traced to a producer inside the set: 0 of 0. abnf-core and catalogue-grammars read only RFC 6455 text and existing crates, and catalogue-grammars leaves `uses` empty.

**Not established.**
- Two items are outside my lane, since they are one story's scope and not a collision. Neither sets the verdict.
  - catalogue-grammars: `crates/rfcspec-wire/tests/stubs.rs:20` asserts `grammars::collect(&document) == Vec::new()` on RFC 6455, and that fails once `collect` is filled. The file is not in the story's scope. The story's `cargo test -p rfcspec-wire grammars` filter would not run the test, so it would surface only at the wave gate.
  - catalogue-grammars: the new scenario file must also be listed under `scenarios:` in `ess/ess-inputs.yaml`, which the scope does not name. No other unit touches that file.
- Whether abnf-core adds a crate dependency is unknown, since the story names none. If it does, Cargo.lock becomes shared with no other unit.

```findings
[]
```
