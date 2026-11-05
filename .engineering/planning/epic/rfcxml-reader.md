---
format: aep.planning-md/3
id: epic:rfcxml-reader
kind: epic
status: draft
title: 'Read RFCXML v3: exact requirements, ABNF grammars and diagrams for RFC 8650 onward'
revision: 1
---
## Outcome

rfcspec reads RFCXML v3, the XML source the RFC Editor publishes for RFC 8650 onward (1,390 RFCs, design D7), into the same normalized document the text reader produces, and uses its markup where it removes guesswork:

- `<bcp14>` marks give each requirement exactly, one per sentence (design §5), with no capitals heuristic;
- `<section>` anchors give section ids and spans;
- `<sourcecode type="abnf">` gives ABNF grammars as text to parse into a grammar and a codec, with no AI (milestone M1);
- `<artwork type="ascii-art">` gives packet diagrams to read into layouts;
- `<xref>` gives the references between RFCs for the explorer.

Operator direction, 2026-11-05: "There seems to be some XML-based whatsoever format for some specs. I mean, we should handle them as well."

## Limits (design §3 table, RFCXML row)

The XML carries "nothing semantic beyond markup; ABNF and diagrams are still text". State machines, timers and roles stay prose, so S3 proposals and review remain the path for machines.

## Test case

RFC 9220 (design §12: "RFC 9220 is the XML-path test case").
