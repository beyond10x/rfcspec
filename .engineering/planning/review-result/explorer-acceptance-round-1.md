---
format: aep.planning-md/3
id: review-result:explorer-acceptance-round-1
kind: review-result
status: active
title: acceptance critic, round 1, RFC explorer
relations:
- reviews: epic:rfc-explorer
- reviews: story:xref-reference-extract
- reviews: story:explorer-rfc-view
- reviews: story:explorer-search
- reviews: story:explorer-profile-layer
revision: 1
---
needs-revision

xref-reference-extract — the acceptance counts self-references and non-RFC references in "the read summary", but `rfcspec read` prints only the document JSON and has no summary, so the counts have no named place and no expected value to check — "The read summary counts the self-references (the table of contents and in-text section links)" — .engineering/planning/story/xref-reference-extract.md:36 (`crates/rfcspec-cli/src/main.rs:316-319`: `Command::Read` does `print!("{}", to_json(&document)?)`)

xref-reference-extract — the acceptance holds four independent bullets (the references list, the counts, the 8441/9110 entries, the text-document identity), so one can pass while another fails and the story is neither done nor not done; state one outcome and move the rest into a "Checked by" list — .engineering/planning/story/xref-reference-extract.md:30-39

story:explorer-rfc-view — the single acceptance sentence joins three independent outcomes (11 drift-checked range files, the RFC 9220 view with its References, the RFC 6455 updated-by links), and "Checked by" adds a fourth, the `rfcspec bulk check` sidecar refusal, plus the range-page cap — .engineering/planning/story/explorer-rfc-view.md:42

story:explorer-profile-layer — the single acceptance sentence joins independent outcomes for RFC 6455 (requirement statuses, decisions and proposal links, three machines), for RFC 868 (two machines, no decisions) and for RFC 9220 (no reviewed layer), so any one can fail while the others pass — .engineering/planning/story/explorer-profile-layer.md:41

What I read: 5 of 5 ids (epic:rfc-explorer, story:xref-reference-extract, story:explorer-rfc-view, story:explorer-search, story:explorer-profile-layer) via `aep plan artifact show`, plus `aep plan artifact kinds` and `lifecycle epic` and `lifecycle story`. I checked the named figures in the tree:
- The RFC 9220 fixture cites 8441 and cites 9110 at section 15.6.2, both in the single Normative References group.
- The 6455 summary row has updated_by 7936, 8307 and 8441.
- `profiles/websocket/overlay.yaml` has 67 accept, 18 edit and 29 reject decisions, which match "85 accepted or edited, 29 rejected".
- The summary has 9,847 rows.
- The websocket witnesses are close, fragments and ping.
- The time witnesses are tcp and udp.
- `WEBSOCKET_READ_SHA256` exists in `crates/rfcspec-cli/tests/rfcxml.rs`, so the byte-identical text-document check is observable.

What I could not establish:
- The epic has only Outcome and "Open shape" sections and no acceptance. I left it unflagged because it is a draft epic whose stories carry the checks.
- `story:explorer-search` has a before state, an exact-count check against `jq` and named tests, so I found no acceptance defect in it.
- Whether the `jq`-equal count in `story:explorer-search` is stable for "level=MUST", whether a "number range" syntax is defined, and whether `ess generate types` 0.86.0 has a typescript target are outside the acceptance lane or not checkable here.
- Out of my lane, for the design and scope critics:
  - The sidecar header, the shard shapes and the References are specified without saying where `references` enter the per-range shards.
  - The summary names the area key `area`, while the search story reads `requirement_counts`.

```findings
- file: .engineering/planning/story/xref-reference-extract.md
  line: 36
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance counts self-references and non-RFC references in 'the read summary', but rfcspec read prints only the document JSON and has no summary, so the counts have no named location and no expected value for RFC 9220"
- file: .engineering/planning/story/xref-reference-extract.md
  line: 30
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance holds four independent bullets (references list, counts, 8441 and 9110 entries, text-document identity), so one can pass while another fails"
- file: .engineering/planning/story/explorer-rfc-view.md
  line: 42
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the single acceptance sentence joins three independent outcomes (11 drift-checked range files, the RFC 9220 view with References, the RFC 6455 updated-by links) and the Checked-by list adds the sidecar refusal and range-page cap as further outcomes"
- file: .engineering/planning/story/explorer-profile-layer.md
  line: 41
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the single acceptance sentence joins independent outcomes for RFC 6455 (statuses, decisions, proposal links, three machines), RFC 868 (two machines, no decisions) and RFC 9220 (no reviewed layer), so any one can fail while the others pass"
```
