---
format: aep.planning-md/3
id: epic:rfcxml-reader
kind: epic
status: implemented
title: 'Read RFCXML v3: exact requirements, ABNF grammars and diagrams for RFC 8650 onward'
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T15:46:56Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-05T15:46:56Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-05T17:32:07Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":3}}}
---
## Outcome

rfcspec reads RFCXML v3, the XML source the RFC Editor publishes for RFC 8650 onward (1,390 RFCs, design D7), into the same normalized document the text reader produces, and uses its markup where it removes guesswork:

- `<bcp14>` marks give each requirement exactly, one per sentence (design §5), with no capitals heuristic;
- `<section>` anchors give section ids and spans;
- `<sourcecode>` and `<artwork>` blocks are kept verbatim with their `type` (`abnf`, `ascii-art`), so later grammar and layout work starts from exact input;
- the bulk index reads the XML source wherever the index lists one, so the 1,390 RFCs it covers are counted from `<bcp14>` marks and each RFC index row says which source it was read from.

Operator direction, 2026-11-05: "There seems to be some XML-based whatsoever format for some specs. I mean, we should handle them as well." The bulk index is the place every RFC is read today, so handling XML there is part of this epic (decision taken by the coordinating session, 2026-11-05).

## Out of scope, and where it is held

- ABNF to a grammar and a codec, and ASCII-art diagrams to layouts: epic:wire-layer (draft). Neither exists for the text path either (design §13 `rfcspec-abnf`, `rfcspec-wire`, milestone M1), so it serves both formats and starts from the verbatim blocks this epic keeps.
- What an `<xref>` becomes: decision-blocker:rfcxml-xref-reference, which blocks epic:rfc-explorer, where the links are used.
- Choosing XML automatically in `rfcspec fetch` from the index, and XML-backed profile overlays.

## Acceptance

- `rfcspec fetch --rfc 9220 --format xml` locks the XML source and `rfcspec read` gives a document marked `xml` whose section blocks equal the text reader's after whitespace is collapsed, with the `<bcp14>` marks and figure types recorded (story:rfcxml-source-reader).
- `rfcspec extract` on the XML lists exactly the requirements the text catalogue lists for RFC 9220, each with source `bcp14-xml` (story:rfcxml-bcp14-extract).
- `rfcspec bulk fetch` and `summarise` read XML where the index lists it, and `bulk check` refuses a row whose format disagrees with its source (story:bulk-xml-summary).
- The committed `bulk/rfc-summary.jsonl` holds 9,847 rows of which 1,390 carry `"format":"xml"` (before: no row carries `format`), and the RFC index pages show a `Source` column (before: none) (story:bulk-xml-regenerate).
- `task check` exits 0.

## Limits (design §3 table, RFCXML row)

The XML carries "nothing semantic beyond markup; ABNF and diagrams are still text". State machines, timers and roles stay prose, so S3 proposals and review remain the path for machines.

## Test case

RFC 9220 (design §12: "RFC 9220 is the XML-path test case").

## Order

The waves of this epic start after the epic:ntp-profile pull request merges into `main`; the stories share files with it (each story names them under "Shared files and order"). `depends_on: epic:ntp-profile` is recorded on `story:rfcxml-source-reader` once that epic is in this store.
