---
format: aep.planning-md/3
id: review-result:bulk-xml-summary-adversary-1
kind: review-result
status: active
title: Adversary on story:bulk-xml-summary, pass 1
relations:
- reviews: story:bulk-xml-summary
revision: 1
---
```
unit: story:bulk-xml-summary, uncommitted working tree on base f60b446 at the bulk-xml-summary worktree
verdict: INFEASIBLE
cases: executed 243→249, red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none
```

Two of my six new cases fail, and neither failure is a blocker. Both build states that I could not show anyone reaching. Every acceptance clause holds.

**1. Diff stat**

```
 AGENTS.md                                          |   8 +-
 crates/rfcspec-cli/src/bulk.rs                     | 130 ++++++--
 crates/rfcspec-cli/src/main.rs                     |  61 ++--
 crates/rfcspec-cli/tests/bulk.rs                   | 346 ++++++++++++++++++++-
 .../rfcspec-cli/tests/fixtures/bulk/rfc-index.xml  |  32 ++
 crates/rfcspec-cli/tests/rfcxml.rs                 |   6 +-
 crates/rfcspec-docs/src/bulk.rs                    |  55 ++--
 crates/rfcspec-model/src/bulk.rs                   |  38 ++-
 crates/rfcspec-source/src/corpus.rs                |  74 ++++-
 crates/rfcspec-source/src/index.rs                 |  20 +-
 ess/domains/catalogue.yaml                         |   6 +-
 11 files changed, 651 insertions(+), 125 deletions(-)
?? crates/rfcspec-cli/tests/bulk_xml_adversary.rs
```

All the tracked changes are the implementor's. My only addition is the new untracked test file. I changed no implementation file.

**2. Cases added** in `the bulk-xml-summary worktree/crates/rfcspec-cli/tests/bulk_xml_adversary.rs`, each run alone first:

- `adv_check_refuses_a_row_read_from_xml_marked_lossy`: **red.** `bulk check` should refuse a row that has both `"format":"xml"` and `"lossy":true`, because `summarise` never writes that pair.
  ```
  panicked at crates/rfcspec-cli/tests/bulk_xml_adversary.rs:178:5:
  assertion `left == right` failed:
    left: Some(0)
   right: Some(1)
  ```
- `adv_fetch_refuses_an_html_page_served_for_an_xml_and_caches_nothing`: **red.** The server answers 200 with an HTML page for `/rfc/rfc9220.xml`. `bulk fetch` caches the page as `xml/rfc9220.xml` and exits 0.
  ```
  panicked at crates/rfcspec-cli/tests/bulk_xml_adversary.rs:290:5:
  assertion `left == right` failed: rfcspec bulk: fetched RFC 868
  rfcspec bulk: fetched RFC 1035
  rfcspec bulk: fetched RFC 5905
  rfcspec bulk: fetched RFC 6455
  rfcspec bulk: fetched RFC 9220
    left: Some(0)
   right: Some(1)
  ```
- **Green:**
  - `adv_check_refuses_format_xml_in_upper_case_naming_the_line`
  - `adv_check_refuses_format_null_naming_the_line`
  - `adv_summarise_exits_1_naming_the_rfc_and_path_of_an_xml_that_is_not_utf8`
  - `adv_summarise_exits_1_naming_the_rfc_and_path_of_an_html_page_saved_as_xml`

  The last two cover two branches the implementor's suite never reached: the refusal of non-UTF-8 XML and the refusal of an XML file that does not read. Before these cases, replacing the XML non-UTF-8 refusal with the text path's lossy fallback would have left the suite green.

**3. Suite run** (after the cases existed)

Command: `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 RUST_TEST_THREADS=4 nice -n 10 ionice -c3 cargo test -p rfcspec-source -p rfcspec-model -p rfcspec-cli -p rfcspec-docs --no-fail-fast`, exit 101.

```
tests/bulk.rs                 test result: ok. 36 passed; 0 failed
tests/bulk_xml_adversary.rs   test result: FAILED. 4 passed; 2 failed
  adv_fetch_refuses_an_html_page_served_for_an_xml_and_caches_nothing ... FAILED
  adv_check_refuses_a_row_read_from_xml_marked_lossy ... FAILED
every other target            ok (rfcxml 8, websocket 22, ntp 33, rfcspec_docs 25+14, rfcspec_model 27, rfcspec_source 10, …)
```

The before count of 243 comes from this same run with my 6 cases subtracted. I did not run the suite before writing the cases.

**4. Judgement findings** (they cover the working tree on f60b446)

- **`crates/rfcspec-cli/src/bulk.rs:171`: INFEASIBLE / introduced.** `row()` accepts `"format":"xml"` together with `"lossy":true`, a pair `summarise` cannot produce.
  - **What reaches it:** only a hand-edited summary. The fix is one more check in `row()`.
- **`crates/rfcspec-source/src/corpus.rs:151`: INFEASIBLE / introduced.** `fetch_corpus` caches any 200 body as `rfc<N>.xml` without checking that it is XML. After that, a rerun skips the file because it exists, and every `summarise` exits 1 at `bulk.rs:176`. The only repair is `--refresh`, which downloads all 9,847 sources again, or deleting the file by hand.
  - **What reaches it:** a proxy or captive portal answering 200. I did not show the RFC Editor doing that.
  - **Fix:** parse the root element before `write`. Plain-text files have the same caching gap, but there the HTML is counted silently instead of blocking the summary.
- **`crates/rfcspec-docs/src/bulk.rs:187`: CONFIRMED / introduced, note.** The overview page text still says "lists with a plain-text format … finds in its text". It is true today and becomes false when `story:bulk-xml-regenerate` writes XML rows. That story should own the edit.
- **"One bad XML blocks the whole summary": acceptable.**
  - The reader at `crates/rfcspec-doc/src/xml.rs:48-68` refuses only three things: XML that is not well-formed, a root other than `<rfc>`, and `version` not `3`. Unknown elements pass.
  - The local 2026-11-05 index has 1,390 XML entries, all from RFC 8650 onward, published v3.
  - All problems are reported in one run, joined by `; `.
  - So the risk falls on `story:bulk-xml-regenerate`. Not finding

**5. Attacked and not broken**

- **Text-row byte identity:** `format` is skipped when it is text and placed before `sha256`. The test against the committed summary compares real committed bytes, not a fixture with itself.
- **`format` values:** `"text"`, `"XML"` and `null` are all refused with the line named.
- **XML entry without TXT:** gets a row, and the test exists. The real index has none.
- **Entry with neither format:** skipped, as before. The real index has 7 (RFC 8, 9, 51, 418, 500, 530, 598).
- **Duplicate index entries:** the real index has none. Handling is unchanged from the text path, and check names a duplicate row.
- **`--refresh` and a second run:** asserted by the exact list of requests.
- **Partial download:** `.part` file then rename.
- **404 on XML:** exits 1 naming the RFC and the `.xml` URL, and the test exists.
- **Cache traversal:** the RFC number is a parsed `u32`, so odd input cannot reach a path.
- **Check with an XML lock:** row format, digest and recount from the locked XML are all enforced. A format mismatch in either direction is named.
- **RFC 9220 fixture entry:** identical to the 2026-11-05 index.
- **Size cap:** 16 MiB against a largest text of 1.6 MB.
- **rfcspec-docs reading rows with `format`:** `parse` is documented as lenient, so it accepts `"text"`, and `rfcspec bulk check` is the full check.
- **ESS `format_read`:** consistent with the entity field.

**6. Paths written outside the worktree:** none. The test temporary directories live under the worktree's `target/rfcspec-cli/` and `$TMPDIR`, and are removed on drop. I created `target/.keeper-hold` and removed it.

```findings
- file: crates/rfcspec-cli/src/bulk.rs
  line: 171
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: bulk check accepts a row with "format":"xml" and "lossy":true, a combination summarise never writes, reachable only by hand edit
- file: crates/rfcspec-source/src/corpus.rs
  line: 151
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: bulk fetch caches any 200 body as rfc<N>.xml unchecked, so an HTML page poisons the cache, later fetches skip it and every summarise exits 1 until a full --refresh
- file: crates/rfcspec-docs/src/bulk.rs
  line: 187
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the overview page prose still says rows are RFCs listed with a plain-text format counted from their text, false once story:bulk-xml-regenerate writes XML rows
```
