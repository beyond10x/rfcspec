---
format: aep.planning-md/3
id: story:text-reader-structure
kind: story
status: active
title: The text reader recognises appendix headings, lists and asides
relations:
- informed_by: epic:rfcxml-reader
scope:
- confidence: cited
  path: bulk/rfc-sections.jsonl
- confidence: cited
  path: bulk/rfc-summary.jsonl
- confidence: cited
  path: crates/rfcspec-cli/src/bulk.rs
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: cited
  path: crates/rfcspec-doc/src/lib.rs
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
- confidence: inferred
  path: crates/rfcspec-extract/tests/locked_profiles.rs
- confidence: inferred
  path: website/docs/catalog/rfcs
- confidence: inferred
  path: website/static/data/rfc-index.json
- confidence: inferred
  path: website/static/data/rfcs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T20:58:31Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T21:24:57Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
---
## Story

The plain-text reader recognises appendix headings, keeps list items and asides as their own paragraphs, and does not read table-of-contents lines as paragraphs, so its requirement ids match those the RFCXML reader gives for the same RFC.

Found on 2026-11-05 while comparing both paths over 158 cached RFCs for `story:rfcxml-bcp14-extract`: 25 give different (id, level) lists. Causes, classified from the rows:

- An `Appendix A.` heading is not recognised, so the text reader keeps the last numbered section (`R13.2-1` where XML gives `RA-1`): RFC 9588, 9609 and 9644 (checked), 8944 and 9231 (from ids).
- `*` bullet lists and `|` asides are joined into one paragraph: 8860, 8951, 9077, 9399, 9441, 9504, 9651, 9700.
- Table-of-contents lines such as "REQUIRED Attributes 5.2." are read as a paragraph: 8881 (lines 101-108 of its text).
- Not classified: 8839, 9028, 9140, 9175, 9210, 9329, 9350, 9483, 9560, 9595, 9665.

No command compares the two paths today; the comparison of 2026-11-05 was ad hoc. This story adds one: `rfcspec bulk compare --cache <dir> [--rfcs <n,...>] [--max-differing <n>]` (clap derive, `crates/rfcspec-cli/src/bulk.rs`). For every RFC the cache holds as both `txt/rfc<N>.txt` and `xml/rfc<N>.xml` (or only those `--rfcs` names), it extracts both and compares their (id, level) lists. It prints `rfc<N>: <k> ids differ` per differing RFC, then `differing: <d> of <n>`, and exits 1 when d exceeds `--max-differing`. It reads only the cache and runs offline.

This changes text-path catalogues, so `EXTRACTOR_VERSION` (`crates/rfcspec-extract/src/lib.rs:38`) moves from 1 to 2, and the committed bulk summary and its section sidecar are regenerated. Rows read from RFCXML keep their counts; the text-only RFCs change.

Gates exceptions: the bytes of `bulk/rfc-sections.jsonl` and `website/static/data/rfcs/09900.json` are pinned by exception digests in the private Gates policy. Regenerating them needs new exception digests, requested from the steward session `b10x-atlas` before the PR, or the `common / Security and privacy` check fails on the PR.

## Acceptance

Values read on 2026-11-05: text ids from the `rfcspec` binary over the corpus cache, XML counts per section from the committed `bulk/rfc-sections.jsonl`.

- Appendix (RFC 9644): before, the text path gives `R7.2-1` (MUST NOT) for the requirement that XML places in Appendix A (section `A`, one MUST NOT). After, the text path gives `RA-1` for it, and no `R7.2-` id.
- Bullet list (RFC 9700 section 4.9.3, three `*` items with SHOULD, SHOULD and MUST): before, the text path gives `R4.9.3-1` and `R4.9.3-2`, both SHOULD; XML gives three (MUST 1, SHOULD 2). After, the text path gives `R4.9.3-1` to `R4.9.3-3` with those three levels.
- Aside (RFC 8860 section 5.1, quoted text in `|` asides): before, the text path gives eight ids `R5.1-1` to `R5.1-8` (SHOULD 2, SHOULD NOT 1, SHALL NOT 4, MAY 1); XML gives ten (SHOULD 2, SHOULD NOT 2, SHALL NOT 4, MAY 2). After, the text path gives `R5.1-1` to `R5.1-10` with the XML levels.
- Table of contents (RFC 8881): before, the text path gives `R-1` to `R-4` (REQUIRED, RECOMMENDED, REQUIRED, RECOMMENDED) from table-of-contents lines 101-108, such as "REQUIRED Attributes 5.2."; XML has one requirement outside numbered sections. After, no text requirement spans lines 101-108, and the text path gives one id under `R-`.
- Comparison: `rfcspec bulk compare --cache <corpus cache> --rfcs 9588,9609,9644,8944,9231,8860,8951,9077,9399,9441,9504,9651,9700,8881,8839,9028,9140,9175,9210,9329,9350,9483,9560,9595,9665 --max-differing 11` exits 0. None of the 14 classified RFCs is listed, and each RFC still listed gets a cause line in this body before the story closes. The same command without `--rfcs` over the whole cache (1390 RFCs held in both formats on 2026-11-05) records its `differing:` line in the implementation report, with no threshold.
- Version and bulk:
  - `grep -n "EXTRACTOR_VERSION: u32 = 2" crates/rfcspec-extract/src/lib.rs` matches;
  - `bulk summarise --cache <corpus cache> --out bulk/rfc-summary.jsonl --sections bulk/rfc-sections.jsonl` is run without a new fetch, from the cache whose `rfc-index.xml` has the committed `index_sha256` `891d504b…`;
  - line 1 of both files then says `"extractor":2`;
  - `task bulk` passes;
  - `rfcspec-docs generate --check` passes on the regenerated explorer data.

- Locked profiles keep their catalogue ids. RFC 868, 1035, 5905 and 6455 are locked by `profiles/time`, `profiles/dns`, `profiles/ntp` and `profiles/websocket`, whose overlays key statuses by catalogue id and whose generated code carries source line numbers. Two places reach them, found by the parallel-safety critic (review-result:wire-parallel-round-2, finding 2): the `*` bullets of RFC 6455 at `profiles/websocket/sources/rfc6455.txt:1581`, and the heading "Appendix A.  Code Skeleton" of RFC 5905 at `profiles/ntp/sources/rfc5905.txt:3367`.
  - `cargo test -p rfcspec-extract --test locked_profiles` passes. The test is new and written first, against the unchanged reader. For each of the four locked RFCs it pins the SHA-256 of the text-path catalogue requirement list (id, level, span), and it must still pass after the change.
  - `rfcspec check --all profiles` exits 0, `rfcspec generate --check` passes for `profiles/time`, `profiles/dns`, `profiles/ntp` and `profiles/websocket`, and `git diff --stat` shows no file under `profiles/`.
  - If the change would move any catalogue id of a locked profile, the implementor stops and reports, naming the RFC and the ids. The implementor does not edit an overlay or a generated file to follow it.

## Scope

- cited: `crates/rfcspec-doc/src/lib.rs` (text reader); `crates/rfcspec-extract/src/lib.rs` (`EXTRACTOR_VERSION`); `crates/rfcspec-cli/src/bulk.rs` and `crates/rfcspec-cli/src/main.rs` (`bulk compare`); `bulk/rfc-summary.jsonl` and `bulk/rfc-sections.jsonl` (regenerated).
- inferred: `website/docs/catalog/rfcs`, `website/static/data/rfc-index.json`, `website/static/data/rfcs` (regenerated from the summary); `crates/rfcspec-extract/tests/locked_profiles.rs` (new). Nothing under `profiles/` is in scope.

## Order

This story runs in wave 5 (coordinator plan, 2026-11-05). No ordering edge to story:send-payload-type-check, beside it in wave 5, is needed: this story keeps every catalogue id of the locked profiles (see Acceptance) and edits no file under `profiles/`, so the committed profiles that story checks stay as they are. story:catalogue-grammars depends on it because both edit `crates/rfcspec-extract/src/lib.rs`. story:wire-scaffold, beside it in wave 5, does not touch that file. story:frame-agreement, which also edits `crates/rfcspec-cli/src/main.rs`, comes after it through catalogue-grammars.

## Constraints

- Running code is Rust with clap derive; no Python. The corpus cache lies outside every Git working tree, and no RFC text, RFCXML or cache file is committed.
- No `/home/<name>/` path in any file, artifact or commit message; the cache is named by its option, not its path.
- Run only the targeted tests and commands above; the full gate runs at the wave close.
