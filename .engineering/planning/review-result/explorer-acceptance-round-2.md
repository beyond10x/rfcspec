---
format: aep.planning-md/3
id: review-result:explorer-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, round 2, RFC explorer
relations:
- reviews: epic:rfc-explorer
- reviews: story:xref-reference-extract
- reviews: story:explorer-rfc-view
- reviews: story:explorer-search
- reviews: story:explorer-profile-layer
revision: 1
---
needs-revision

xref-reference-extract — the acceptance sentence still joins two independently passable outcomes with "and", the 19-entry `references` list and the `reference_counts` object, so one can pass while the other fails; round 1 asked for one outcome with the rest under "Checked by", and only the 8441/9110 and text-document bullets moved. Quote: "the document JSON carries a `references` list of 19 entries, one per `<xref>` to another RFC, and `reference_counts: {self_references: 16, non_rfc: 0}`" — .engineering/planning/story/xref-reference-extract.md:44-46

What I read: 5 of 5 ids (epic:rfc-explorer, story:xref-reference-extract, story:explorer-rfc-view, story:explorer-search, story:explorer-profile-layer), plus review-result:explorer-acceptance-round-1, via `aep plan artifact show`. I checked the named figures in the tree:
- **Fixture counts:** `crates/rfcspec-doc/tests/fixtures/rfc9220.xml` has 35 `<xref>` elements. 16 target `section-*` or `name-*` anchors and 19 target the 8 `<reference>` elements, so the 19 and 16 hold.
- **References:** every `<reference>` has an RFC `seriesInfo`, so non_rfc is 0. The fixture also has the one `<references pn="section-6">` group, and the xref to RFC 9110 at section 15.6.2 is present.
- **Text-document check:** `WEBSOCKET_READ_SHA256` exists at `crates/rfcspec-cli/tests/rfcxml.rs:16`.
- **Summary data:** `bulk/rfc-summary.jsonl` has 9,847 rows spread over 11 thousand-ranges, and RFC 6455's `updated_by` is [7936, 8307, 8441].
- **Search count:** the `jq` filter for status INTERNET STANDARD with `.counts.MUST>0` returns 69, so the search acceptance is checkable.
- **Profile data:** the time profile overlay has no review or proposal entries, which matches "no decisions" for RFC 868. The websocket overlay has `proposal_id` entries and `profiles/websocket/proposals/` exists.
- **Round-1 findings:** the explorer-rfc-view and explorer-profile-layer acceptances are now one before/after sentence with the other outcomes under "Checked by", so those findings are fixed. The sentence and counts for 9220 and the epic's four-bullet acceptance carry no defect.

What I could not establish:
- Whether `task explorer`, `task docs-check` over `website/static/data/` and the `ess generate types --target typescript` target work: the explorer, the data directory and the generated types do not exist yet, so these are planned checks I could not run. Not a defect at draft.
- Out of my lane: the summary field is `counts`, while the search story's domain section reads the entity field `requirement_counts`. That is a mapping question for design or scope.

```findings
- file: .engineering/planning/story/xref-reference-extract.md
  line: 44
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance sentence still joins two independently passable outcomes with 'and', the 19-entry references list and the reference_counts object, so one can pass while the other fails; state the references list as the one outcome and move reference_counts into Checked by"
```
