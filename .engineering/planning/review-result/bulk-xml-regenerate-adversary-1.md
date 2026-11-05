---
format: aep.planning-md/3
id: review-result:bulk-xml-regenerate-adversary-1
kind: review-result
status: active
title: Adversary on story:bulk-xml-regenerate, pass 1
relations:
- reviews: story:bulk-xml-regenerate
revision: 1
---
```
unit: story:bulk-xml-regenerate, uncommitted working tree in the bulk-xml-regenerate worktree (base 096e2a6)
verdict: nothing found
cases: executed n/a→n/a, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: none
```

I didn't break it, and I added no tests.

**1. Diff stat.** `18 files changed, 11370 insertions(+), 11293 deletions(-)`. These are the same 18 implementation and generated files I was given. I wrote no file in the tree, test or otherwise.

**2. Cases added.** None. Every attack below held, so there was no red case to write.

**3. Suite run.** None, because no case of mine existed for it to run. I ran two gate steps directly, and both exit 0:
- `rfcspec bulk check --summary bulk/rfc-summary.jsonl --profiles profiles` printed "holds".
- `cargo run -p rfcspec-docs -- generate --check` printed "36 generated files current".

**4. Findings.** None.

**5. What I attacked and could not break:**
- **XML counts against an independent re-run:** I built a temporary profile that locks all 1,390 cached RFCXML files and ran `rfcspec extract` on each one. That goes through the profile path, not the bulk path. All 1,390 count maps equal the summary rows, and the extractor reported 0 problems on every one.
- **XML minus text distribution, total counts per RFC:** 1,193 RFCs show no difference. 145 differ in one direction (largest −15, RFC 9073), 52 in the other (largest +96, RFC 9904).
  - Large positives: XML counts BCP 14 keywords inside tables that the text reader misses. RFC 9904 has 65 more in section 3. Each of 9904, 9142, 8859 and 10004 has exactly as many `<bcp14>` elements as capital keywords in its text.
  - Large negatives: keywords that appear in capitals in the text but are not marked `<bcp14>` in the XML.
- **XML zero where text counted:** six RFCs: 8774, 8904, 8962, 9233, 9592, 9782, each with 1 to 4 text matches. I checked 8962, 8774 and 9233 by hand: the XML has no `<bcp14>` mark and the capitals are quotes or jokes, so these are not reader failures.
  - RFC 9948 has 11 `<bcp14>` elements but only in the RFC 2119 boilerplate. Its count is 0 on both paths.
  - Where the count exceeds the number of `<bcp14>` elements (for example 9169: 3 against 0), the extra comes from `capitals-text` sentences in unmarked figures. That is the extractor's documented behaviour.
- **Reader robustness:** across the cache every `<bcp14>` content is one of the 11 levels, including ones written with a non-breaking space or split over a line. RFCs 8651, 8657, 8659, 8860 and 9012 extract with 0 problems, and the oddly spaced `MUST NOT` resolves to `MUST NOT`.
- **Hashes:** every XML row's sha256 equals its cached file. RFC 9220 is `8a60d0b5…`, MAY 1 and SHOULD 1.
- **Header:** rows 9,847 and `rows_sha256` 5651b511… both match the row lines. The index sha 891d504b… equals the cached `rfc-index.xml`. The file is 2,774,259 bytes, under 8 MiB.
- **Text rows:** all 8,457 are byte-identical to HEAD. Metadata (everything except sha256, counts and format) is identical for all 9,847 rows.
- **Links:** the pages have 9,847 Source cells. None has a label that disagrees with its file extension. The set of `XML` links equals the set of XML rows. Links use the same `RFC_EDITOR` base and `rfc<N>.<ext>` pattern as `bulk fetch`. Every table row has 11 columns.
- **Page size:** 2,545,983 bytes in total, under the 4 MiB test.
- **Profile rows:** RFCs 868, 1035, 5905 and 6455 have no format (so text), and their sha256 equals each profile's `rfcspec.lock`.
- **Private content:** `b10x-gates scan-text` with the org policy reports "18 file(s) carry no private rule". The only `/home/` in a changed file is the existing rule text at `AGENTS.md:110`.
- **Stale wording:** README has no `bulk/` row to update. No hand-written page or ESS summary still says the index is plain text only.

**6. Paths written outside the worktree:** none. My scratch was `target/adv/` and the `target/.keeper-hold` marker in this worktree; I removed both. I built only `rfcspec-cli` and `rfcspec-docs`, in the tree's own `target/`.

**7. Findings block**

```findings
[]
```
