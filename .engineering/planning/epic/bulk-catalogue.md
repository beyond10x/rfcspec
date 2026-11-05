---
format: aep.planning-md/3
id: epic:bulk-catalogue
kind: epic
status: active
title: Mechanical catalogue of every BCP 14 RFC
relations:
- depends_on: epic:websocket-profile
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:47Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:47Z", actor: "human:timo", revision: 3}
---
## Outcome

Timo, 2026-11-05: approved. The catalogue lists every RFC that uses BCP 14 keywords, mechanically
extracted (stage S2), without human review: one index row per RFC with its title, status, stream,
area, obsoleted/updated-by edges and requirement counts by level. It is the "insane amount of
specs" index the reviewed profiles grow out of.

## Constraints

- RFC texts are not committed in bulk (the common security scan refuses a candidate above 256 MiB
  and the repository should stay small). What is committed is a derived, digest-pinned summary:
  per RFC the source SHA-256, counts by level and the extractor version; reviewed profiles stay in
  `profiles/` as today.
- The summary is produced by an explicit networked command (rfc-index.xml plus each text, cached
  outside the repository) and checked offline by `task check` against itself (format, sort order,
  digests present); regeneration is a separate, recorded step.
- The site shows the bulk index without one page per RFC; reviewed profiles keep their pages and are
  marked in the index.

## Acceptance

`task check` exits 0 offline; the committed summary lists every rfc-index.xml entry with a TXT
format, BCP 14 counts for each (zero allowed), and the RFC 6455 row equals the counts of
`profiles/websocket`; the built site serves the bulk index and links the reviewed profiles.
