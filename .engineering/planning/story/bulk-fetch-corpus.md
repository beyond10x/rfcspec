---
format: aep.planning-md/3
id: story:bulk-fetch-corpus
kind: story
status: active
title: Fetch every RFC text into a cache outside the repository and commit the bulk summary, checked by task check
relations:
- decomposes: epic:bulk-catalogue
- depends_on: story:bulk-summary-format
- depends_on: story:dns-source-catalogue
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: bulk/rfc-summary.jsonl
- confidence: cited
  path: crates/rfcspec-cli/src/main.rs
- confidence: cited
  path: crates/rfcspec-cli/tests/bulk.rs
- confidence: cited
  path: crates/rfcspec-source
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:52Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:53Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":2}}}
---
## Story

`rfcspec bulk fetch` downloads `rfc-index.xml` and the plain text of every RFC the index lists with
a TXT format into a cache outside the repository; `rfcspec bulk summarise` (from
`story:bulk-summary-format`) turns the cache into `bulk/rfc-summary.jsonl`, which is committed; and
`task check` validates it offline. Fetching and summarising is the separate, recorded regeneration
step the epic names; the gate never fetches.

## Acceptance

On a clean checkout without network `task check` exits 0, and the committed `bulk/rfc-summary.jsonl`
has one row per TXT entry of the index whose SHA-256 its header records (9,847 rows for the
4 November 2026 index), with the RFC 868, RFC 1035 and RFC 6455 rows matching `profiles/time`,
`profiles/dns` and `profiles/websocket`.

- `rfcspec bulk fetch --cache <DIR>` (default `$XDG_CACHE_HOME/rfcspec/bulk`, else
  `$HOME/.cache/rfcspec/bulk`) writes `<DIR>/rfc-index.xml` from
  `https://www.rfc-editor.org/rfc-index.xml` (`--index-url`) and `<DIR>/txt/rfc<N>.txt` from
  `https://www.rfc-editor.org/rfc/` (`--base-url`, the default of `rfcspec fetch`) for every TXT
  entry. It is sequential, uses the 60 s timeout and 16 MiB per-file limit of `rfcspec_source::fetch`
  (the index gets its own 64 MiB limit), skips a text already in the cache unless `--refresh`, and
  exits 1 on the first failed download naming the RFC and URL, so a rerun resumes. It refuses a
  `--cache` inside a Git working tree (naming the tree), so RFC texts cannot land in the repository.
  The network code lives in `crates/rfcspec-source`, the one crate that reaches the network.
- `rfcspec bulk summarise --cache <DIR> --out bulk/rfc-summary.jsonl` reads only the cache. A test
  serves a fixture index and two texts from a `127.0.0.1` HTTP server (as the `rfcspec-source` tests
  do), runs fetch then summarise, asserts two rows, then asserts a second fetch downloads nothing
  and a cache path inside a temporary Git repository is refused.
- The committed summary is produced by those two commands with the then-current index; the commit
  message names the index SHA-256, the row count and the time of the fetch. Its size stays under
  8 MiB (the test asserts it on the committed file), far below the 256 MiB scan limit, and no RFC
  text, `rfc-index.xml` or cache file is committed.
- `bulk check` passes on the committed file: the RFC 6455 row has SHA-256
  `765775326aee0ecca9b04bde3fd1f52932d498e33e34e428bd61b8a24da0fa3b` and the counts by level of
  `rfcspec extract --profile profiles/websocket --rfc 6455`; the RFC 868 row has the SHA-256 of
  `profiles/time/rfcspec.lock` and `counts: {}`; the RFC 1035 row has the SHA-256 of
  `profiles/dns/rfcspec.lock` and the counts by level of `rfcspec extract --profile profiles/dns
  --rfc 1035`. If the RFC Editor now serves different bytes for any of the three, the story stops
  and reports instead of editing a lock.
- The header's `rows_sha256` is the SHA-256 over all row lines of the committed file, and `bulk
  check` recomputes it; the rows of RFCs locked in a profile (868, 1035, 6455) are additionally
  recomputed from their locked text.
- `Taskfile.yml` gains a `bulk` task, `rfcspec bulk check --summary bulk/rfc-summary.jsonl --profiles
  profiles`, run by `check`, and a `bulk-regenerate` task (not run by `check`) running fetch then
  summarise. Removing a row, swapping two rows or editing the counts of any row, locked or not, makes
  `task bulk` exit 1 (the edit changes `rows_sha256`); a test asserts each on a copy of the committed
  file, editing the counts of a row whose RFC no profile locks.
- `AGENTS.md` State lists `bulk/` and the gate step; its Rules name `rfcspec bulk fetch` as the second
  networked command and say the summary is regenerated only by `task bulk-regenerate`, never by hand.

## Domain relations

- Summary row -> Rfc, one-to-one, a projection: `ess/domains/catalogue.yaml`, view
  `rfcspec.catalogue.Rfcs`, `source: rfcspec.catalogue.Rfc` (the fields added by
  `story:bulk-summary-format`).
- Summary -> index, many rows from one index copy: inferable (inferred from
  `ess/domains/catalogue.yaml`, entity `rfcspec.catalogue.Rfc`, summary "One RFC as the index
  describes it", and `docs/design/rfcspec.md` section 3, which makes the index entry a fetched
  source). The header records the digest of that one copy.
- Profile -> Rfc, one-to-many, referencing: inferable (inferred from `ess/domains/profile.yaml`, type
  `rfcspec.profile.ProfileRfc`, field `rfc`, no `relations:` entry).

## Scope

`crates/rfcspec-source` (index and bulk text download, tests), `crates/rfcspec-cli` (`bulk fetch` in
`src/main.rs`, `--cache` on `bulk summarise`, `tests/bulk.rs`), `bulk/rfc-summary.jsonl` (new,
generated), `Taskfile.yml`, `AGENTS.md`, `Cargo.lock`.

Depends on `story:bulk-summary-format`: it runs that story's `summarise` and `check`, and edits the
same `crates/rfcspec-cli/src/main.rs`, `tests/bulk.rs` and `Cargo.lock`. Depends on
`story:dns-source-catalogue`: `bulk check --profiles profiles` verifies `profiles/dns/rfcspec.lock`,
which that story creates, against the RFC 1035 row.
