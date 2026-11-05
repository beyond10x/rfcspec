---
format: aep.planning-md/3
id: epic:rfc-explorer
kind: epic
status: draft
title: 'RFC explorer: browse every RFC as structured data, profiles as the deepest layer'
relations:
- informed_by: epic:public-protocol-catalog
- informed_by: epic:rfcxml-reader
revision: 1
---
## Outcome

The public site becomes an RFC explorer rather than a catalogue listing: any RFC the index publishes can be opened and read as structured data, and the reviewed profiles are its deepest layer.

Operator direction, 2026-11-05: "what I earlier named catalog. I think what we actually want is more like a RFC explorer".

## Open shape (to settle at decomposition)

- One page per RFC from `bulk/rfc-summary.jsonl`: metadata, its BCP 14 requirements by section and level, and the RFCs it updates or obsoletes and is updated or obsoleted by, as links.
- Search and filter across the index (number, title, keyword level counts, status).
- For a profiled RFC: each requirement with its overlay status, its review decision and the recorded proposal, the machines and layouts, and their exploration witnesses.

## Order

After story:websocket-proposal-slice-2 and the NTP profile epic (approval-record:ntp-profile).
