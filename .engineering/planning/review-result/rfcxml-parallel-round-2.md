---
format: aep.planning-md/3
id: review-result:rfcxml-parallel-round-2
kind: review-result
status: active
title: parallel critic, round 2, RFCXML reader
relations:
- reviews: epic:rfcxml-reader
- reviews: story:rfcxml-source-reader
- reviews: story:rfcxml-bcp14-extract
- reviews: story:bulk-xml-summary
- reviews: story:bulk-xml-regenerate
revision: 1
---
approve

The ordering as written makes the set safe. The RFCXML waves start only after the `epic:ntp-profile` pull request merges into `main`. That removes every collision I recorded in round 1.

- **Each story names the gate.** All four stories carry the same sentence, e.g. `.engineering/planning/story/rfcxml-source-reader.md:118`, `.engineering/planning/story/bulk-xml-summary.md:100`, `.engineering/planning/story/bulk-xml-regenerate.md:85`.
- **The edge lands in one place.** `depends_on: epic:ntp-profile` goes on `story:rfcxml-source-reader`, the root of the in-set chain. `aep plan artifact graph` shows the chain reader, then extract, then summary, then regenerate as `depends_on` edges. One edge on the root therefore covers all four stories.
- **The round-1 collisions are now named in the story bodies.**
  - `crates/rfcspec-model/src/lib.rs`
  - `crates/rfcspec-doc/src/lib.rs`
  - `crates/rfcspec-extract/src/lib.rs`
  - `crates/rfcspec-cli/tests/bulk.rs`
  - `crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml`
  - `crates/rfcspec-docs/src/bulk.rs`
  - `website/docs/catalog/rfcs/`
  - `Taskfile.yml`
  - `AGENTS.md`
- **The row counts are fixed.** The old "three locked texts" wording is replaced at `bulk-xml-summary.md:39-42` and `:96-98` by four texts (RFC 868, 1035, 5905, 6455) and `rows` 4, "as `story:ntp-source-catalogue` leaves the test". That matches the `ntp-profile` branch, whose `crates/rfcspec-cli/tests/bulk.rs` asserts `rows` 4 and whose fixture index has 5 `<rfc-entry>` elements.
- **Regeneration happens after the merge.** `bulk-xml-regenerate.md:82-83` regenerates the pages from `main` after the NTP merge, so the RFC 5905 row carries its `[ntp]` link.
- **`Cargo.lock` is not named, and it does not matter.**
  - The `ntp-profile` branch changes `Cargo.lock`. `story:rfcxml-source-reader` also edits it, because it adds `roxmltree` (scope line 6 and body line 110).
  - No shared-files section lists it.
  - The gate covers it whether or not it is listed.

What I read: 5 artifacts (`epic:rfcxml-reader` and its four stories) and the round-1 record `review-result:rfcxml-parallel-round-1`. I ran `aep plan artifact show` on each, plus `graph` and `waves`. On the NTP side I ran `git diff --stat main...ntp-profile` in the `ntp-draft` tree and read `bulk.rs` and the fixture index from the `ntp-profile` branch.

Surfaces: 4 stories cited by their body Scope sections (the store marks every scope entry inferred), 0 inferred in my sense, 0 unplaced. In-set acceptance reads traced to a producer: 5, and a `depends_on` edge records all 5 (4 direct, 1 transitive). Two further reads (the four locked texts and `rows` 4; the `[ntp]` marker) have their producer outside the set. That is a gate in prose, not an edge, by design.

What I could not establish:
- **Enforcement of the gate.** Only prose holds it until the NTP merge. `aep plan artifact waves` prints `story:rfcxml-source-reader (inferred)` in wave 1 beside five unrelated stories. It cannot see the gate, because the `ntp-profile` stories are not in this store. The gate sits in the story bodies, and `epic:rfcxml-reader` does not mention it (its body has no `ntp` or `merge` text). A driver working from `waves` or the epic alone could dispatch the reader early.
  - This is an unease, not a finding: the bodies state the gate and an implementor reads them.
  - An open `decision-blocker` that `blocks` the reader story until the merge would make it visible in the store, if you want that. I am not choosing it for you.
- **The branch is still moving.** The `ntp-profile` branch is unmerged, so its file list may still change. The ordering holds whatever it ends up touching.

Out of lane, not counted in the verdict: none new. The round-1 note that the epic promises ABNF, layouts and `<xref>` which no story in the set delivers is still a scope or design matter.

```findings
[]
```
