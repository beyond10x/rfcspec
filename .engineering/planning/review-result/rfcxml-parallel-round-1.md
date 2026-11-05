---
format: aep.planning-md/3
id: review-result:rfcxml-parallel-round-1
kind: review-result
status: active
title: parallel critic, round 1, RFCXML reader
relations:
- reviews: epic:rfcxml-reader
- reviews: story:rfcxml-source-reader
- reviews: story:rfcxml-bcp14-extract
- reviews: story:bulk-xml-summary
- reviews: story:bulk-xml-regenerate
revision: 1
---
needs-revision
story:rfcxml-source-reader — it edits `crates/rfcspec-model/src/lib.rs` (cited, scope line 117) and adds XML-dispatch tests under `crates/rfcspec-doc` (cited), while the ntp-profile branch edits both: ntp-timer-model rewrites `Arm`/`loss` in the model file and ntp-source-catalogue appends 124 lines of tests to `crates/rfcspec-doc/src/lib.rs`; the body names neither story, so say they share these files and record an ordering edge, or split the surface (the AGENTS.md edit also meets ntp-timer-model's) — .engineering/planning/story/rfcxml-source-reader.md:113-120
story:rfcxml-bcp14-extract — it edits `crates/rfcspec-extract/src/lib.rs` and `crates/rfcspec-model/src/lib.rs` (cited, scope line 78), where ntp-source-catalogue adds 117 lines to the extract file (reader or extractor fixes allowed there) and ntp-timer-model rewrites the model file, and the body does not say so; add an ordering edge that records the shared files, or split the surface — .engineering/planning/story/rfcxml-bcp14-extract.md:78-83
story:bulk-xml-summary — it edits `crates/rfcspec-cli/tests/bulk.rs`, `crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml` and `crates/rfcspec-docs/src/bulk.rs` (cited), all three changed by ntp-source-catalogue; that story's fixture insert is RFC 5905 after RFC 1035, its `texts()` helper locks four texts and its header asserts `rows` 4, so the acceptance's "the three locked texts" and "rows byte-identical" no longer hold unchanged; the body must name that story, with an ordering edge (or split the files), and re-read its row counts — .engineering/planning/story/bulk-xml-summary.md:53-58, :92-100
story:bulk-xml-regenerate — it regenerates `website/docs/catalog/rfcs/` and edits `crates/rfcspec-docs/src/bulk.rs`, `Taskfile.yml` and `AGENTS.md` (cited); ntp-source-catalogue changes `rfcs/05000.md` (the 5905 row gains an `[ntp]` link) and `rfcs/index.md` plus `bulk.rs`, and ntp-header-codec, ntp-exchange-machine and ntp-poll-machine edit `Taskfile.yml`, and ntp-timer-model and ntp-poll-machine edit `AGENTS.md`; the body says nothing about this, so record the ordering edge or split the surface, and the pages must be regenerated after the ntp merge to carry the profile marker — .engineering/planning/story/bulk-xml-regenerate.md:59-64, :75-81

Collision with epic:ntp-profile, which I read from branches `ntp-profile`, `ntp-source-catalogue` and `ntp-timer-model` (their diff against the merge base, e763e20):
- **Collides:**
  - `crates/rfcspec-model/src/lib.rs`
  - `crates/rfcspec-extract/src/lib.rs` (a different file from the `check.rs` you named)
  - `crates/rfcspec-doc/src/lib.rs`
  - `crates/rfcspec-cli/tests` (ntp edits `bulk.rs`, `dns.rs`, `websocket.rs` and adds `ntp.rs`; this set edits `bulk.rs` and adds `rfcxml.rs`, which does not collide)
  - `crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml`
  - `crates/rfcspec-docs/src/bulk.rs`
  - `website/docs/catalog/rfcs`
  - `Taskfile.yml`
  - `AGENTS.md`
- **Does not collide:** `crates/rfcspec-gen`, `crates/rfcspec-ess`, `crates/rfcspec-extract/src/check.rs`, `profiles/ntp`.

Inside the set the four stories form one `depends_on` chain (reader, extract, summary, regenerate), so no pair is unordered.

What I read: 5 artifacts (`aep plan artifact show`, `graph` and `list`), plus the ntp stories and `git diff` against the merge base on the three ntp branches. Placement: 4 stories cited by body scope sections (the store records every scope entry as inferred) and 0 unplaced. Acceptance reads traced to a producer in the set: 5, of which 5 are covered by a `depends_on` path. 4 are direct edges and 1 is transitive (summary reads the reader's fixture through extract).

Could not establish:
- Which lines of `crates/rfcspec-model/src/lib.rs` and `crates/rfcspec-doc/src/lib.rs` each side touches, so whether a textual merge conflicts is unknown. The shared file is certain.
- Which ntp story merges first, since sequencing is not mine to choose.

Out of your lane, not counted in the verdict:
- The epic promises ABNF parsing, packet-diagram layouts and `<xref>` references (the xref edge is blocked by `decision-blocker:rfcxml-xref-reference`), but no story in this set delivers them (scope or design).

```findings
- file: .engineering/planning/story/rfcxml-source-reader.md
  line: 113
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it edits crates/rfcspec-model/src/lib.rs and crates/rfcspec-doc, which the ntp-profile branch also edits (ntp-timer-model rewrites Arm/loss in the model file; ntp-source-catalogue appends tests to crates/rfcspec-doc/src/lib.rs), and AGENTS.md (ntp-timer-model); the body names none of this, so state the shared files and record an ordering edge or split the surface"
- file: .engineering/planning/story/rfcxml-bcp14-extract.md
  line: 78
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it edits crates/rfcspec-extract/src/lib.rs and crates/rfcspec-model/src/lib.rs, which ntp-source-catalogue (117 lines added to the extract file) and ntp-timer-model (model file) also edit on the ntp-profile branch, and the body does not say so; record an ordering edge or split the surface"
- file: .engineering/planning/story/bulk-xml-summary.md
  line: 92
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "it edits crates/rfcspec-cli/tests/bulk.rs, tests/fixtures/bulk/rfc-index.xml and crates/rfcspec-docs/src/bulk.rs, all changed by ntp-source-catalogue, which adds RFC 5905 to the fixture and raises the summarise test to four locked texts and rows 4, so the acceptance 'the three locked texts' no longer holds; the body must name that story, record an ordering edge or split the files, and re-read its row counts"
- file: .engineering/planning/story/bulk-xml-regenerate.md
  line: 75
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "it regenerates website/docs/catalog/rfcs and edits crates/rfcspec-docs/src/bulk.rs, Taskfile.yml and AGENTS.md, all touched by epic:ntp-profile stories (ntp-source-catalogue changes rfcs/05000.md, rfcs/index.md and bulk.rs; ntp-header-codec, ntp-exchange-machine and ntp-poll-machine edit Taskfile.yml; ntp-timer-model and ntp-poll-machine edit AGENTS.md); the body is silent, so record an ordering edge or split the surface, and regenerate the pages after the ntp merge"
```
