---
format: aep.planning-md/3
id: epic:rfc-explorer
kind: epic
status: active
title: 'RFC explorer: browse every RFC as structured data, profiles as the deepest layer'
relations:
- informed_by: epic:public-protocol-catalog
- informed_by: epic:rfcxml-reader
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T17:48:52Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T17:48:52Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

The public site becomes an RFC explorer rather than a catalogue listing: any RFC the index publishes can be opened and read as structured data, and the reviewed profiles are its deepest layer.

Operator direction, 2026-11-05: "what I earlier named catalog. I think what we actually want is more like a RFC explorer".

## Open shape (to settle at decomposition)

- One page per RFC from `bulk/rfc-summary.jsonl`: metadata, its BCP 14 requirements by section and level, and the RFCs it updates or obsoletes and is updated or obsoleted by, as links.
- Search and filter across the index (number, title, keyword level counts, status).
- For a profiled RFC: each requirement with its overlay status, its review decision and the recorded proposal, the machines and layouts, and their exploration witnesses.
- For an RFC read from RFCXML, the view also links the RFCs it cites, normative and informative, read from its `<xref>` elements as `rfcspec.catalogue.Reference` (`ess/domains/catalogue.yaml`). Sources: the original outcome of `epic:rfcxml-reader` (revision 1, 2026-11-05: "`<xref>` gives the references between RFCs for the explorer"), and `decision-blocker:rfcxml-xref-reference`, cleared 2026-11-05 with option A, decided by the session sys-mon under the operator's delegation, with the order: the ESS declaration, then the extraction story, then this epic.

## Acceptance

- `story:xref-reference-extract`: `rfcspec read` on the RFC 9220 XML lists the RFCs it cites as References.
- `story:explorer-rfc-view`: `/rfcspec/explorer/?rfc=9220` shows RFC 9220 from the generated per-range data, its References linked.
- `story:explorer-search`: `/rfcspec/explorer/` filters the whole index in the browser, and the filtered count equals a `jq` count over the summary.
- `story:explorer-profile-layer`: `/rfcspec/explorer/?rfc=6455` shows the reviewed layer of the websocket profile.

## Order

After story:websocket-proposal-slice-2 and the NTP profile epic (approval-record:ntp-profile).
