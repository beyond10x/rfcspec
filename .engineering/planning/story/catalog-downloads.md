---
format: aep.planning-md/3
id: story:catalog-downloads
kind: story
status: implemented
title: Publish each profile's ESS documents and generated artifacts as catalogue downloads
relations:
- decomposes: epic:public-protocol-catalog
- depends_on: story:catalog-profile-page
scope:
- confidence: cited
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: cited
  path: crates/rfcspec-docs/src/main.rs
- confidence: cited
  path: website/docs/catalog/time.md
- confidence: cited
  path: website/static/downloads
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T23:34:09Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-04T23:34:09Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-04T23:50:41Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Story

`rfcspec-docs generate` copies every file under each `profiles/<name>/generated/` (the lowered
`ess-protospec/1` documents and the generated Rust crate) into
`website/static/downloads/<name>/`, keeping the relative path, and the profile page lists them
as downloads.

## Acceptance

After `task website`, `website/build/downloads/time/` holds `tcp.protospec.yaml`,
`udp.protospec.yaml`, `rust/Cargo.toml` and `rust/src/lib.rs` byte-identical to
`profiles/time/generated/`, each linked from `/rfcspec/docs/catalog/time`, and `task check` fails
when a copy is stale, edited or orphaned.

- The profile page gains a "Downloads" section: one row per copied file with its path, what it
  is (`ess-protospec/1` model of machine `<m>` for `generated/<m>.protospec.yaml`, matched to the
  overlay machine of that name; "generated Rust crate" for files under `generated/rust/`), its
  size in bytes and its SHA-256, and a link to the copy that the site build resolves with
  `onBrokenLinks: throw` (for example a `pathname://` link under the site base URL).
- The copies are outputs of `generate` and drift-checked by `generate --check` like the pages:
  missing, stale or edited copies fail naming the path, and a file under
  `website/static/downloads/` with no source under `profiles/*/generated/` is reported
  `unexpected` and deleted by `generate`. A unit test covers each case on a scratch root.
- A profile with no `generated/` (the fixture of `story:catalog-index`) gets the sentence "No
  generated artifacts yet." and no download directory.
- `profiles/*/generated/` is only read; `rfcspec generate --check` and `rfcspec lower --check`
  are unaffected. A non-UTF-8 file under `generated/` makes `generate` fail naming it, rather
  than copy it lossily.

## Domain relations

- Profile -> Machine, one-to-many, owned: `ess/domains/profile.yaml`, entity
  `rfcspec.profile.Profile`, relation `machines`; the machine field `protospec_path` names the
  lowered document this story publishes.
- Profile -> generated artifact, one-to-many, the profile directory owns `generated/` and a
  generated file exists only through its overlay. Inferable (inferred from `AGENTS.md`, Rules,
  "The same holds for `profiles/*/generated/`", and the layout of `profiles/time/generated/`; no
  `ess/1` document declares a generated artifact).

## Scope

`crates/rfcspec-docs` (`catalog.rs`, `main.rs` drift and orphan check over
`website/static/downloads/`, tests), `website/static/downloads/time/` (generated),
`website/docs/catalog/time.md` (regenerated). Authored witnesses (`profiles/*/witnesses/`) and
locked sources are not published here.

Depends on `story:catalog-profile-page`: it adds a section to the page that story generates and
extends the same `crates/rfcspec-docs/src/catalog.rs` and the outputs and orphan logic in
`main.rs` that `story:catalog-index` introduced.
