---
format: aep.planning-md/3
id: story:dns-source-catalogue
kind: story
status: implemented
title: 'Lock, read and extract RFC 1035: a pre-BCP 14 RFC with column-0 body text, its headings and the header diagram'
relations:
- decomposes: epic:dns-profile
- depends_on: story:websocket-fragments-machine
scope:
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-doc
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
- confidence: cited
  path: profiles/dns/overlay.yaml
- confidence: cited
  path: profiles/dns/profile.yaml
- confidence: cited
  path: profiles/dns/rfcspec.lock
- confidence: cited
  path: profiles/dns/sources/rfc1035.txt
- confidence: cited
  path: website/docs/catalog/dns.md
- confidence: cited
  path: website/docs/catalog/index.md
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T08:01:49Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-05T08:01:49Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-05T08:25:27Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Story

`rfcspec fetch --rfc 1035` locks RFC 1035 into a new profile `profiles/dns`, and the reader and
extractor are shown to handle the first catalogue RFC that predates BCP 14: no capitals keyword
anywhere, body text at column 0 (RFC 868 and RFC 6455 indent theirs by 3), numbered headings with
one space after the number (`4.1.1. Header section format`), and the section 4.1.1 header diagram.
Its requirements are prose, as those of RFC 868 are: the catalogue lists lowercase `must`,
`should` and `may` sentences as prose candidates, and the reviewer writes `source: prose`
requirements in the later stories. The profile enters the protocol catalogue with an overlay that
reviews nothing yet.

Builds on the WebSocket wave (`epic:websocket-profile`): page-split paragraphs, section numbers
on blocks and figures come from there.

## Acceptance

`rfcspec extract --profile profiles/dns --rfc 1035` prints a catalogue with no BCP 14 requirement
whose blocks carry the section numbers of the 71 numbered headings of RFC 1035, and `rfcspec check
--all profiles` passes on `time`, `websocket` and `dns`.

Values below were read from `https://www.rfc-editor.org/rfc/rfc1035.txt` on 2026-11-05 (3077
lines, 122549 bytes, SHA-256 `d14ae809fc9b41bbcae26bb38c937c9515808b944f3252b00de9fe0e95f4fdfb`).
If the fetched file has another digest, the lock records what was fetched and every value here is
re-read before the story is accepted.

- `rfcspec fetch --rfc 1035 --profile profiles/dns` writes `sources/rfc1035.txt` and a lock
  entry `{rfc: 1035, file: sources/rfc1035.txt, sha256: <hex>}`; a second run changes no byte.
- Furniture: the reader removes the 55 `Mockapetris ... [Page N]` footers (pages 1 to 55), the
  55 form-feed lines and the 54 header lines `RFC 1035        Domain Implementation and
  Specification    November 1987` (the last form feed, after page 55, has no header): 164 lines.
  No block contains `[Page` or starts with `RFC 1035 `. The RFC 868 and RFC 6455 reader tests
  pass unchanged.
- Every block carries the number of the section it lies in, read from the 71 numbered headings (9
  top-level) from `1. STATUS OF THIS MEMO` (line 9) to `9. REFERENCES and BIBLIOGRAPHY` (line
  2748); `4.1.1. Header section format` is line 1401, `4.2.1. UDP usage` line 1752, `7.3.
  Processing responses` line 2526. Blocks before line 9 carry none.
- The section 4.1.1 header diagram (lines 1405-1419) is one figure. The field description at lines
  1473-1474 ("Z  Reserved for future use.  Must be zero in all queries and responses.") is a
  paragraph in section 4.1.1, and so is the `ID` description at lines 1423-1426.
- BCP 14: the file holds no capitals `MUST`, `SHOULD`, `MAY`, `SHALL`, `REQUIRED`,
  `RECOMMENDED` or `OPTIONAL`, so the catalogue has no requirement. A test asserts zero.
- Prose candidates: the file holds 23 lowercase `must`, 98 `should` and 46 `may` as whole
  words (case-sensitive; `Must` at line 1473 and `May` at line 2893 are not lowercase). Every
  paragraph sentence that carries one is a prose candidate (the existing rule). In the scoped
  sections the keyword lines are: `may` at 1447, 1464, 1501, 1503 (4.1.1); `may` at 1761, 1763
  and `should` at 1764, 1769, 1772, 1776 (4.2.1); `should` at 2529, 2557, 2565, 2566 and
  `must` at 2559 (7.3). A test asserts that each of these lines lies in the span of a prose
  candidate, and that the line 1473 sentence is no prose candidate (the reviewer adds it as a prose
  requirement in `story:dns-header-codec`).
- `profiles/dns/profile.yaml`: name `dns`, RFC 1035 "Domain Names - Implementation and
  Specification", sections `["4.1.1", "4.2.1", "7.3"]`, no semantics yet.
  `profiles/dns/overlay.yaml`: `rfcspec-overlay/1`, rfc 1035, `kind: K1`, no requirements,
  layouts, machines or findings.
- `rfcspec check --all profiles` checks `dns`, `time` and `websocket` and exits 0. On
  `profiles/time` and `profiles/websocket`, `check`, `lower --check` and `generate
  --check` pass with no change under either directory.
- `rfcspec-docs generate` lists `dns` (RFC 1035, K1, stage S4, no reviewed requirements) in
  `website/docs/catalog/index.md` beside `time` and `websocket`, and writes
  `website/docs/catalog/dns.md`; `rfcspec-docs generate --check` passes.
- The RFC 1035 catalogue is byte-identical across two runs, and `task check` exits 0.

## Domain relations

- Rfc -> Section, one-to-many, the RFC owns its sections: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Rfc`, relation `sections`.
- Section -> Section, zero or one parent: `ess/domains/catalogue.yaml`, entity
  `rfcspec.catalogue.Section`, relation `parent`; "a section includes its subsections" in the
  profile scope reads it.
- Profile -> Rfc, one-to-many, the profile references RFCs and owns none. Inferable (inferred from
  `ess/domains/profile.yaml`, type `rfcspec.profile.ProfileRfc`, a field typed
  `rfcspec.catalogue.RfcNumber` with no `relations:` entry, and
  `crates/rfcspec-model/src/lib.rs:45`).

## Scope

`profiles/dns/profile.yaml`, `profiles/dns/overlay.yaml`, `profiles/dns/rfcspec.lock`,
`profiles/dns/sources/rfc1035.txt`, tests in `crates/rfcspec-doc` and
`crates/rfcspec-extract` (a reader or extractor fix there only if a value above fails),
`crates/rfcspec-cli/tests`, `website/docs/catalog/index.md` and `website/docs/catalog/dns.md`
(regenerated).

Choices taken:

- The scope is sections 4.1.1, 4.2.1 and 7.3: the header, UDP usage, and the response-matching
  paragraph that names the ID field.
- Column-0 paragraphs continued across a page break (for example section 4.2.2, lines 1784-1793)
  are not joined: the existing rule needs an indented last line, and no scoped section has such a
  paragraph.

The RFC text names its author, his institute and example hosts (`F.ISI.ARPA`), as the committed
`rfc868.txt` names ISI and SRI; it is committed verbatim. If the Gates scan refuses a literal in
it, stop and report rather than edit the source.

First of three stories; the others depend on it through `profiles/dns/overlay.yaml`,
`profiles/dns/profile.yaml` and the generated catalogue pages.
