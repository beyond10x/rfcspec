---
format: aep.planning-md/3
id: review-result:rfcxml-design-round-1
kind: review-result
status: active
title: design critic, round 1, RFCXML reader
relations:
- reviews: epic:rfcxml-reader
- reviews: story:rfcxml-source-reader
- reviews: story:rfcxml-bcp14-extract
- reviews: story:bulk-xml-summary
- reviews: story:bulk-xml-regenerate
revision: 1
---
approve

What I read: 5 artifacts (the four stories and the epic), whole bodies via `aep plan artifact show`. I also ran `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate`, which reports valid for all 94 artifacts. I checked the catalogue claims in `ess/domains/catalogue.yaml` and the crate dependencies (`rfcspec-extract` depends on `rfcspec-doc` and `rfcspec-model`).

Shape:
- **Chain:** the graph has one chain, `story:bulk-xml-regenerate` → `story:bulk-xml-summary` → `story:rfcxml-bcp14-extract` → `story:rfcxml-source-reader`. Each edge is a `depends_on` that matches a real data dependency, and each story ends with a "Depends on … because" sentence. That is a recorded ordering, not a hidden one.
- **Cycles:** none. Walking out from the set (9 declared edges inside it, plus the edges to `epic:rfc-explorer` and `decision-blocker:rfcxml-xref-reference`) finds no path back.
- **Acceptance reads:** I traced 6 reads of state another item in the set produces, and the graph records 6 of 6. Examples are the fixture and `format` field the summary story takes from the earlier stories, and the regenerated summary and `format_read` view field the regenerate story takes from the summary story. Reads of pre-existing state (`bulk/rfc-summary.jsonl`, `rfc9220.txt`, the 868/1035/6455 rows) have no producer in the set.
- **Split abstraction:** none. The reader stops at S1 and is demonstrable alone (13 marks in RFC 9220), so extraction consuming them is a clean seam. The same holds between summary and regenerate: `bulk.rs` in `rfcspec-docs` is touched by both, but the first only reads `format`, "pages unchanged here".
- **Horizontal slice:** none. Each story ends on a runnable command with its own fixture-backed acceptance.

What I could not establish:
- Out of my lane: the epic outcome lists ABNF-to-grammar, artwork-to-layout and `<xref>` for the explorer, and no story in the set delivers them (scope critic). Four stories sharing `AGENTS.md`, `README.md`, `main.rs` and `rfcxml.rs` is covered by the recorded chain (parallel-safety critic).

```findings
[]
```
