---
title: The catalogue
sidebar_position: 2
description: What an RFC becomes in RFCspec - requirements, grammars, layouts, registries, procedures, limits and machines, each with a cited source span.
lede: Every item RFCspec takes from an RFC carries the span of text it came from and that text's digest, so nothing in a generated artifact is unexplained.
source: design sections 3 to 5 and decisions D1, D6, D7, D8
source_url: https://github.com/beyond10x/rfcspec/blob/main/docs/design/rfcspec.md
status: decided
---

:::caution[Decided, not built]
The catalogue format `rfcspec/1` is a design decision. No code reads or writes it yet.
:::

## From RFC to catalogue

| Stage | Reads | Writes | Deterministic |
|---|---|---|---|
| S0 fetch | a profile | locked sources and `rfcspec.lock` | network, recorded |
| S1 read | a locked RFC | one document model: sections, paragraphs, figures, references, defined terms | yes |
| S2 extract | the document | requirements, ABNF, layouts, registries, procedures | yes |
| S3 propose | catalogue and document | AI proposals with cited spans | recorded and replayed |
| S4 review | proposals | the overlay: accepted, edited, rejected | authored |
| S5 lower | catalogue and overlay | ESS documents and the wire sidecar | yes |
| S6 synthesize | lowered documents | a Rust crate and a WASM codec | yes |
| S7 verify | lowered documents and a target | the requirement matrix and test results | yes, given the target |

Everything after S0 reads only locked files and runs offline.

## What the catalogue holds

| Item | Holds |
|---|---|
| `Rfc` | number, title, status, stream, area, format read, digest, update graph |
| `Section` | anchor or number, title, parent, span |
| `Requirement` | level (one of the eleven BCP 14 keywords), party, text, span, and where it was found: an XML `bcp14` mark, capitals in plain text, an ABNF comment, or prose for RFCs before BCP 14 |
| `DefinedTerm` | a phrase the RFC defines and invokes, such as RFC 6455's `_Fail the WebSocket Connection_`; it becomes a named procedure in a machine |
| `Grammar` | ABNF rules verbatim, their imports, their dialect |
| `Layout` | a packet diagram as fields with bit widths, dependent lengths and conditions |
| `Registry` | IANA's entries, read from IANA's XML, never from the RFC's IANA section |
| `Limit` | a quantity the RFC leaves to the implementation, with any stated minimum |
| `Machine` | participants, states, triggers, transitions and timers, each transition linked to requirements |
| `Profile` | a set of RFCs and the sections and parties in scope |
| `Status` | per profile and requirement: satisfied by, verified by, unsupported with a reason, or not applicable with a reason |

## Provenance

Each item carries a span: the RFC, the section, a byte range in the normalized text, and the
SHA-256 of that text. Requirement ids are `R<section>-<ordinal>`, qualified by the RFC when they
cross (`6455:R5.1-3`). Published RFCs never change, so ids are stable; when an erratum changes a
text, the item keeps its id, is marked stale, and its review decisions wait for confirmation.

## Profiles and overlays

A profile names the RFCs that make up a protocol and what is in scope. WebSocket is RFC 6455 with
7936, 8307, 8441 and 9220, and optionally 7692. The overlay is the only other authored file per
profile: it holds every review decision, keyed by item id and the digest of the text judged.
Generated files are never edited by hand.
