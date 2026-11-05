---
format: aep.planning-md/3
id: story:wire-m1-record
kind: story
status: draft
title: Record milestone M1 in AGENTS.md
relations:
- decomposes: epic:wire-layer
- depends_on: story:frame-agreement
- depends_on: story:handshake-codec
- depends_on: story:frame-codec-vectors
scope:
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: README.md
revision: 6
---
## Story

Record milestone M1 (design section 14) in `AGENTS.md`. The full gate (`task check`) is the wave close the coordinator runs on the integration branch, not part of this story.

- `AGENTS.md` State table:
  - a row for `crates/rfcspec-abnf`: dialects `rfc5234` with RFC 7405, `rfc2616` with the `#` list limitation, and `bits`; core rules; prose-val imports; the matcher;
  - a row for `crates/rfcspec-wire`: grammar collection, the diagram reader and layout checks, the agreement check;
  - `rfcspec extract` now also giving grammars and layouts;
  - the `wire agree` command in the `crates/rfcspec-cli/` row;
  - the `profiles/websocket/` row and the State paragraph: section 4.3 in scope, the reviewed grammars, the generated `handshake` module with finding `F-1`, and the frame codec mutation vectors.
- `AGENTS.md` Gate: the `websocket` task step that runs `rfcspec wire agree`.
- `README.md` only where it lists crates or commands.
- The generated status page keeps M1 as planned: `crates/rfcspec-docs/src/status.rs` knows only `decided` and `planned` rows (lines 31 and 46), and M0 shows planned as well. A built state is later work.

## Acceptance

Before, re-read from `AGENTS.md` on 2026-11-05 at the head of this worktree: the first six commands below print 0, `grep -c "sections 4.3, 5.2, 5.4, 5.5, 5.6 and 7" AGENTS.md` prints 0, and `grep -c "sections 5.2, 5.4" AGENTS.md` prints 2 (the State paragraph, line 17, and the `profiles/websocket/` row, line 42). After:

- `grep -c "crates/rfcspec-abnf" AGENTS.md` prints at least 1;
- `grep -c "crates/rfcspec-wire" AGENTS.md` prints at least 1;
- `grep -c "rfcspec wire agree" AGENTS.md` prints at least 2 (the CLI row and the Gate);
- `grep -c "src/handshake.rs" AGENTS.md` prints at least 1;
- `grep -c "F-1" AGENTS.md` prints at least 1;
- `grep -c "mutation_" AGENTS.md` prints at least 1;
- `grep -c "sections 4.3, 5.2, 5.4, 5.5, 5.6 and 7" AGENTS.md` prints 2, and `grep -c "sections 5.2, 5.4" AGENTS.md` prints 0: the word "sections" anchors the second command, so the new text "sections 4.3, 5.2, 5.4" does not match it.

## Scope

- cited (read for this plan): `AGENTS.md`.
- inferred: `README.md`.

## Order

This story depends on story:frame-agreement, story:handshake-codec and story:frame-codec-vectors, whose results it records.

## Domain relations

None: this story changes documentation only.
## Constraints

- Running code is Rust; any command line uses clap derive. No Python, no shell helpers.
- No RFC text is committed beyond what the profiles already lock: tests read RFC 6455 values from `profiles/websocket/sources/rfc6455.txt` by line number; generated code carries rule names and source line numbers, not rule text; synthetic fixtures are written for the test, not copied from an RFC.
- No comments in `Cargo.toml`, `Taskfile.yml` or YAML files.
- No `/home/<name>/` path in any file, artifact or commit message.
- Builds run with `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0` in the `target/` of the worktree itself, after `df -h /`. Run only the targeted tests named under Acceptance; the full gate (`task check`) runs once at the wave close.
