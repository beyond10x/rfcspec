---
format: aep.planning-md/3
id: story:rfc868-source-reader
kind: story
status: implemented
title: Fetch, lock and read an RFC's plain text
relations:
- decomposes: epic:rfc868-thin-slice
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-04T21:52:55Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-04T22:38:34Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Story

`rfcspec fetch` downloads one RFC's plain text into a profile's `sources/` and records its SHA-256
in the profile's `rfcspec.lock`; `rfcspec read` turns a locked text into a normalized document.

## Acceptance

- `rfcspec fetch --rfc 868 --profile profiles/time` writes `sources/rfc868.txt` and a lock entry
  `{rfc: 868, file: sources/rfc868.txt, sha256: <hex>}`; a second run with an unchanged file
  changes no byte.
- `rfcspec read --profile profiles/time --rfc 868` refuses when the file's digest differs from the
  lock, naming both digests.
- On the committed `rfc868.txt`, the reader removes both page footers and the page-2 header: no
  output paragraph contains `[Page` or starts with `RFC 868`, and source lines 56, 60, 61 and 114
  (the furniture lines of the committed file) lie in no paragraph's line range.
- Each paragraph and each indented figure keeps the 1-based line range it came from in the source;
  the dialogue block `S: Listen on port 37 (45 octal).` … `S: Close the connection.` is a figure.
- Tests run offline on the committed source; only `fetch` touches the network.

## Scope

`crates/rfcspec-model` (lock, profile and document types), `crates/rfcspec-source` (fetch, lock),
`crates/rfcspec-doc` (text reader), `crates/rfcspec-cli` (`fetch`, `read`),
`profiles/time/profile.yaml` (name, RFCs, sections in scope), `profiles/time/sources/`,
`profiles/time/rfcspec.lock`.
