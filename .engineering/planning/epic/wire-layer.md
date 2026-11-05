---
format: aep.planning-md/3
id: epic:wire-layer
kind: epic
status: draft
title: 'Wire layer: ABNF to grammar and codec, ASCII-art diagrams to layouts (milestone M1)'
relations:
- informed_by: epic:rfcxml-reader
revision: 1
---
## Outcome

The wire layer of design §13 and milestone M1: `rfcspec-abnf` parses an RFC's ABNF (RFC 5234) into a grammar and generates a parser and printer from it, and `rfcspec-wire` reads an ASCII-art packet diagram into a layout the overlay can review, for text and RFCXML sources alike. The RFCXML reader (epic:rfcxml-reader) keeps `<sourcecode type="abnf">` and `<artwork type="ascii-art">` verbatim, so XML sources need no heuristic to find them.

Operator direction, 2026-11-05: "this would be like an awesome programmatic way of reading that and turning it into code, like into ESS specs directly."

## Status

Draft, not decomposed. Ordered after epic:ntp-profile, epic:rfcxml-reader and epic:rfc-explorer unless the operator moves it.
