---
format: aep.planning-md/3
id: review-result:explorer-view-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:explorer-rfc-view
relations:
- reviews: story:explorer-rfc-view
revision: 1
---
I made four kinds of case fail. `rfcspec bulk check --sections` passes three kinds of tampered sidecar. `bulk summarise` cuts any section heading that wraps onto a second line. And `?rfc=` quietly opens a different number when the one asked for is too large for JavaScript to hold exactly.

```
unit: story:explorer-rfc-view, uncommitted working tree on f616eef in the explorer-rfc-view worktree
verdict: NEEDS-CHANGE
cases: executed 268→277, red 5
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path
needs-coordinator: none
```

1. **`git --no-pager diff --stat`**: `28 files changed, 10472 insertions(+), 9903 deletions(-)`. This is the same as when I started, so I changed no tracked file. My three files are new and untracked, and all are test files:
   - `crates/rfcspec-cli/tests/bulk_sections_adversary.rs`
   - `crates/rfcspec-docs/tests/explorer_adversary.rs`
   - `explorer/tests/adversary.test.ts`

   I touched no implementation file.

2. **Cases added.** Each was run alone first:
   - **`bulk_sections_adversary.rs`**: each check case tampers RFC 9220's sidecar row, recomputes the header, and expects `bulk check --sections` to exit 1. No profile locks 9220, so only the structural checks stand between an edit and the shards.
     - **`check_refuses_a_row_that_lists_one_section_number_twice`** (red): `left: Some(0) right: Some(1)`, "section 3 listed twice passes the check".
     - **`check_refuses_a_section_that_holds_no_requirement`** (red): `{"MAY":0}` in section 4 gives `left: Some(0) right: Some(1)`.
     - **`check_refuses_the_entry_without_a_number_anywhere_but_last_and_more_than_once`** (red): `left: Some(0) right: Some(1)`, "the numberless entry first passes the check".
     - **`summarise_keeps_a_section_heading_that_wraps_onto_a_second_line_whole`** (red): RFC 6455's 5.2 heading is wrapped onto a second line. The result was `left: String("Base Framing Protocol for Frames Sent Between a Client and a")`, `right: "...a Client and a Server"`.
   - **`explorer_adversary.rs`** (green, 2 cases):
     - The writer's `path()` and the app's `shardFile` agree at 1, 99, 100, 9999, 10000, 10099 and 10100.
     - Over all 101 committed shards, every number the app would link has a row, and every number in `not_in_index` has none.
   - **`adversary.test.ts`** (3 cases, 1 red):
     - **Green:** leading zeros, `-1`, `+`, `1e3` and `0x10`.
     - **Red:** "never answers with another number than the one ?rfc= names" fails with `AssertionError: asked 9007199254740993, got 9007199254740992`.

3. **Suite runs**, after the cases existed:
   - **Rust:** `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 RUST_TEST_THREADS=4 nice -n 10 ionice -c3 cargo test --no-fail-fast --locked -p rfcspec-model -p rfcspec-cli -p rfcspec-docs` exited 101. 264 cases ran: 260 passed and 4 failed, all in `bulk_sections_adversary`.
   - **Explorer:** `pnpm exec vitest run` in `explorer/` exited 1. 13 tests ran, 12 passed and 1 failed (`adversary.test.ts`).
   - **Count:** before = after minus my 9 cases from the same runs, so 258 + 10 = 268.
   - **Types:** `task types-check` exited 0.

4. **Findings**, all on the working tree above:

   | file:line | finding | verdict | origin | what reaches it |
   |---|---|---|---|---|
   | `crates/rfcspec-cli/src/bulk.rs:465` | `check_sections` only checks that each row's section counts add up to its summary counts. It accepts a repeated section number, a zero count, and the numberless entry first or twice, all of which `SectionsRow`'s doc and AGENTS.md rule out. | NEEDS-CHANGE | introduced | Hand-editing the committed sidecar and recomputing its header. `docs-generate` then puts the edit into the shards, and a repeated number gives duplicate Vue `:key`s in `RfcView.vue`. The committed sidecar has none of these today (checked with jq). |
   | `crates/rfcspec-cli/src/bulk.rs:38` | `heading_title` takes only the first line of a heading, so a heading that wraps is cut. | NEEDS-CHANGE | introduced | `task bulk-regenerate`. The committed sidecar has it: RFC 2180 §3.1 reads "...of a multi-accessed" (missing "mailbox"), RFC 2557 §12 "...previous version of this proposed", and RFC 5091 §5.5.1 ends in a comma. I did not measure the full count; my scan was rough and caught 1 of these 3. |
   | `explorer/src/lib/shard.ts:21` | `Number(value)` rounds digit strings past 2^53, so the view opens and names a different RFC. 400 nines become `Infinity`, and longer values show as `1e+23`. | CONFIRMED (note) | introduced | Only a typed or crafted URL. No link the site generates reaches it. |

   **Judgement only, no case written:**
   - The app's test fixtures (`explorer/tests/fixtures/06400.json`, `09200.json`) are hand-cut, cast with `as ExplorerShard`, and not compared with the generated shards. A shape drift in the shards would leave vitest green.
   - The References "Citing section" column shows the anchor (`websockets-upgrade-over-http3`), not the section number.
   - The shards total 16,069,836 bytes against the 16 MiB (16,777,216) cap, so about 4% headroom.

   **Fixes, named but not applied:**
   - **Check:** have `check_sections` enforce unique section numbers, no zero or empty counts, and at most one numberless entry, last.
   - **Wrapped headings:** have `heading_title` join the indented lines that continue the heading.
   - **`?rfc=`:** have `requestedRfc` refuse a value above `Number.MAX_SAFE_INTEGER`, or above 5 to 6 digits.

5. **Attacked and could not break:**
   - **Header tampering:** a changed header SHA, row count, format, index or extractor is refused, and so are rows out of order, duplicated, missing or extra, and References on a text row (existing tests).
   - **Shard boundaries:** 99/100 and 9999/10000; a hundred with no rows returns 404 and reads "not in the index".
   - **Links:** a number is linked only when it has a row (property over all 101 shards).
   - **RFC 9220:** its References in the shard equal `rfcspec read` on the fixture.
   - **XSS:** none possible; there is no `v-html`, and every `href` is a fixed prefix plus data.
   - **`?rfc=` parsing:** leading zeros, signs, exponents, hex, `rfc9220` and empty values.
   - **Fetch failures:** 404 reads "not in the index"; other status codes and thrown errors read "could not be loaded".
   - **`types-check`:** regenerates both targets and compares them with `diff -r`, so an extra or missing file is reported.
   - **Shard drift:** `docs-check` reports a stale, extra or missing shard.
   - **Committed content:** no `/home/` paths, no Python, no comments in the config files. The sidecar titles are genuine headings: none starts at a list item.
   - **`pages.yml`:** it builds the explorer with `corepack enable` and pnpm before Docusaurus, under Node 22.
   - **Not done:** I did not acquire a worktree lease; the `worktree` CLI shows no lease command.

6. **Paths written outside the worktree:** `the session scratch directory`, holding test logs and probe TSVs. Inside the worktree I also wrote `target/types-drift/` (from `task types-check`), and the tests used tempdirs under `target/rfcspec-cli/`, which removed themselves.

```findings
- file: crates/rfcspec-cli/src/bulk.rs
  line: 465
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: bulk check --sections accepts a re-headered sidecar row with a repeated section number, a zero-count section, or the numberless entry first or twice, because it only compares summed counts
- file: crates/rfcspec-cli/src/bulk.rs
  line: 38
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: heading_title keeps only the first line of a wrapped heading, so the committed sidecar and shards carry cut section titles (RFC 2180 3.1, 2557 12, 5091 5.5.1)
- file: explorer/src/lib/shard.ts
  line: 21
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: requestedRfc rounds digit strings past 2^53 and returns another number (9007199254740993 becomes 9007199254740992), so the view names an RFC that was not asked for
```
