---
format: aep.planning-md/3
id: epic:rfc868-thin-slice
kind: epic
status: implemented
title: RFC 868 Time Protocol end to end
summary: 'The thin slice: one tiny RFC through fetch, read, extract, review, lowering to ESS, codec generation and a live conformance probe.'
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T22:38:39Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-04T22:38:39Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-11-04T22:38:39Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":3}}}
---
## Outcome

One RFC goes through every RFCspec stage end to end, in the smallest form that is still real:
RFC 868, the Time Protocol (May 1983, two pages, TCP and UDP, one 32-bit binary value, five worked
examples, four of them valid in 32 bits). It is the thin slice of design milestones M0–M3 (`docs/design/rfcspec.md` §14) on a
protocol small enough to finish in one wave.

Why RFC 868:

- It predates BCP 14, so its requirements are prose (`should refuse the connection or close it
  without sending anything`): the `prose` source path of design §5, with human review.
- Its text has page furniture (`Postel [Page 1]`, `RFC 868 May 1983`) that the reader must remove.
- It writes its exchanges as dialogue lines (`S: Listen on port 37`, `U: Connect to port 37`),
  which a reader can extract mechanically as machine steps.
- Its wire format is one 32-bit binary number, the smallest layout (design §8).
- It carries four test vectors (2,208,988,800 = 00:00 1 Jan 1970 GMT, …) and one that cannot be a
  32-bit unsigned value (-1,297,728,000 = 17 Nov 1858), a real finding for review.

## Stages covered

| stage | here |
|---|---|
| S0 fetch | `rfcspec fetch` writes `profiles/time/sources/rfc868.txt` and `rfcspec.lock` with SHA-256 |
| S1 read | the plain-text reader removes page furniture and yields paragraphs and indented figures with line ranges |
| S2 extract | BCP 14 sentences (none in RFC 868), lowercase prose candidates, dialogue steps, numeric examples |
| S3 propose | not in this wave: the overlay is written by a reviewer by hand, every quote checked against the source |
| S4 review | `profiles/time/overlay.yaml`: accepted requirements, the layout, the TCP and UDP machines, statuses |
| S5 lower | `rfcspec lower` writes two `ess-protospec/1` models that `ess protocol explore` passes |
| S6 synthesize | `rfcspec generate` writes a small Rust codec crate from the layout, with the RFC's examples as tests |
| S7 verify | `rfcspec verify` probes a TCP time server and prints a requirement matrix; a good and a faulty in-process server show pass and fail |

## Not in this wave

- AI proposals (S3), RFCXML, ABNF, IANA registries, errata.
- A generated server or client beyond the codec; UDP verification against a live target.

## Specification

The nouns are RFCspec's own ESS domains in `ess/domains/` (catalogue, profile, review).

## Acceptance

On a clean checkout, `task check` exits 0 and runs, for profile `profiles/time`: `rfcspec check`, `rfcspec lower --check` and `ess protocol explore` on both models (each `"verdict": "passed"`), `rfcspec generate --check` and `cargo test -p rfc868-time`, and the `rfcspec-conformance` tests in which a correct in-process server passes every observable TCP row ("unable to determine the time" is `inconclusive`) and three faulty servers each fail their row.
