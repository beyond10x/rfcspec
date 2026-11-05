---
format: aep.planning-md/3
id: story:status-evidence
kind: story
status: draft
title: Record evidence for every satisfied requirement status in the profile overlays
relations:
- decomposes: epic:rfc-explorer
- depends_on: story:websocket-grammar-review
- depends_on: story:handshake-codec
scope:
- confidence: inferred
  path: crates/rfcspec-cli/src/checks.rs
- confidence: inferred
  path: crates/rfcspec-cli/tests/check.rs
- confidence: inferred
  path: crates/rfcspec-docs/src/catalog.rs
- confidence: inferred
  path: crates/rfcspec-docs/src/explorer.rs
- confidence: inferred
  path: crates/rfcspec-extract/src/check.rs
- confidence: inferred
  path: crates/rfcspec-model/src/lib.rs
- confidence: inferred
  path: explorer/src/components/ProfileLayer.vue
- confidence: inferred
  path: profiles/dns/overlay.yaml
- confidence: inferred
  path: profiles/ntp/overlay.yaml
- confidence: inferred
  path: profiles/time/overlay.yaml
- confidence: inferred
  path: profiles/websocket/overlay.yaml
- confidence: inferred
  path: website/docs/catalog/dns.md
- confidence: inferred
  path: website/docs/catalog/ntp.md
- confidence: inferred
  path: website/docs/catalog/time.md
- confidence: inferred
  path: website/docs/catalog/websocket.md
- confidence: inferred
  path: website/static/data/profiles/dns.json
- confidence: inferred
  path: website/static/data/profiles/ntp.json
- confidence: inferred
  path: website/static/data/profiles/time.json
- confidence: inferred
  path: website/static/data/profiles/websocket.json
revision: 9
---
## Story

A satisfied requirement status in a profile overlay names its evidence. Today no overlay records any (`grep evidence profiles/*/overlay.yaml` finds none), so 46 satisfied status rows across the four profiles have empty evidence: websocket 29, ntp 7, time 6, dns 4. `rfcspec.profile.RecordSatisfiedStatus` refuses that state with `EvidenceMissing` (`ess/domains/profile.yaml:947-954`). Found by adversary pass 1 on `story:explorer-profile-layer` (`review-result:explorer-profile-adversary-1`).

Evidence has the ESS shape `rfcspec.profile.Evidence { reference, digest }`: a test name or a file path, and the SHA-256 of what it names (`ess/domains/profile.yaml:217-226`). In the overlay it is an `evidence:` list beside `status` on each satisfied requirement entry.

What counts as evidence (decided 2026-11-05 by the coordinator; this was the open question):
- A requirement satisfied by a codec layout (`layout.<name>`) cites the generated round-trip test of that codec:
  - in `rfc6455-websocket`, a test whose name starts with `roundtrip_` (story:frame-codec-vectors);
  - in `rfc1035-dns`, `rfc5905-ntp` and `rfc868-time`, which have no `roundtrip_` tests, an accepted vector or example test (`vector_*_accepted`, `example_*`) that asserts printing gives the parsed octets back.
  - The reference is `<crate>::tests::<name>`, and the digest is that of the generated `profiles/<p>/generated/rust/src/lib.rs`.
- A requirement satisfied by a machine transition (`<machine>.<participant>.<transition>`) cites the protospec witness of that machine, `profiles/<p>/witnesses/<machine>.inputs.json`, or a conformance scenario file under `ess/scenarios/`. The digest is that of the file.

Changes:
- `crates/rfcspec-model/src/lib.rs`: the evidence list on an overlay requirement.
- `crates/rfcspec-extract/src/check.rs`: refuses a satisfied status without evidence.
- `crates/rfcspec-cli/src/checks.rs`, which knows the profile directory: refuses evidence whose reference names no file and no test, or whose digest differs.
- `crates/rfcspec-docs/src/catalog.rs` (the profile page) and `crates/rfcspec-docs/src/explorer.rs` (the `evidence` field the generated `ExplorerStatus` already declares, `ess/domains/profile.yaml:294`) render it, and `explorer/src/components/ProfileLayer.vue` shows it.
- The four overlays gain their 46 evidence entries, and the pages are regenerated with `task docs-generate`.

## Acceptance

- Refusal: `cargo test -p rfcspec-extract a_satisfied_status_without_evidence_is_named` passes. An overlay whose `R5.2-1` is satisfied by `layout.frame-header` with no evidence gives exactly the problem `requirement R5.2-1: satisfied by layout.frame-header, but names no evidence`, in the form of the existing `requirement {id}: satisfied by {element}, which is no layout or transition` (`crates/rfcspec-extract/src/check.rs:382`).
- Existence: `cargo test -p rfcspec-cli --test check evidence` passes on copies of `profiles/websocket`:
  - evidence `rfc6455-websocket::tests::roundtrip_missing` gives `requirement R5.2-1: evidence rfc6455-websocket::tests::roundtrip_missing names no file and no test`; a test exists when `grep -c "fn roundtrip_missing()" profiles/websocket/generated/rust/src/lib.rs` would print 1;
  - evidence `witnesses/close.inputs.json` with a changed digest gives `requirement R7.1.2-1: evidence witnesses/close.inputs.json has digest <d>, but what it names hashes to <actual>`.
- Coverage: `grep -c "evidence:" profiles/websocket/overlay.yaml profiles/ntp/overlay.yaml profiles/time/overlay.yaml profiles/dns/overlay.yaml` prints 29, 7, 6 and 4, and `rfcspec check --all profiles` exits 0.
- Shown:
  - `grep -c "| Element or reason | Evidence |" website/docs/catalog/websocket.md` prints 1, and the row of `R5.2-1` ends with its `roundtrip_` reference;
  - `jq "[.requirements[] | select(.status.kind == \"satisfied\") | .status.evidence | length > 0] | all" website/static/data/profiles/websocket.json` prints `true`;
  - `task docs-check` and `task explorer` pass.

## Scope

- inferred:
  - `profiles/dns/overlay.yaml`, `profiles/ntp/overlay.yaml`, `profiles/time/overlay.yaml`, `profiles/websocket/overlay.yaml`;
  - `crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-cli/src/checks.rs`, `crates/rfcspec-cli/tests/check.rs`, `crates/rfcspec-model/src/lib.rs`;
  - `crates/rfcspec-docs/src/catalog.rs`, `crates/rfcspec-docs/src/explorer.rs`, `explorer/src/components/ProfileLayer.vue`;
  - the regenerated pages `website/docs/catalog/{dns,ntp,time,websocket}.md` and `website/static/data/profiles/{dns,ntp,time,websocket}.json`.
- No other generated page changes: `website/docs/catalog/rfcs/` and `website/static/data/rfcs/` do not read overlays.
- The profile page is rendered by `catalog.rs` (`status_cells`, line 452), not `page.rs`, so `page.rs` left the scope.

## Order

This story depends on story:websocket-grammar-review and story:handshake-codec, because both rewrite `profiles/websocket/overlay.yaml` and the websocket pages, and story:websocket-grammar-review also edits `crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-cli/src/checks.rs` and `crates/rfcspec-model/src/lib.rs` (coordinator plan, 2026-11-05). It is the last of the `check.rs` chain send-payload-type-check → fixed-arm-bounds-check → xml-overlay-check → websocket-grammar-review. Through story:handshake-codec it also follows story:frame-codec-vectors, whose `roundtrip_` tests it cites.

## Constraints

- Running code is Rust; browser code is Vue 3 with TypeScript. No Python.
- No comments in YAML. No `/home/<name>/` path in any file, artifact or commit message.
- Run only the targeted tests above; the full gate runs at the wave close.
