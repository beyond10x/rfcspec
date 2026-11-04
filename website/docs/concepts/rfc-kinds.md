---
title: Kinds of RFC
sidebar_position: 1
description: What the RFC corpus contains, in which formats, and which kinds of RFC RFCspec models.
lede: Of 9,854 RFCs, the ones worth modelling are wire protocols, data formats, extensions and registries; the rest get a requirement catalogue.
source: design sections 2.1 to 2.3 and decision D11; rfc-index.xml downloaded 4 November 2026
source_url: https://github.com/beyond10x/rfcspec/blob/main/docs/design/rfcspec.md
status: decided
---

## The corpus

Counted from the RFC Editor's index, `https://www.rfc-editor.org/rfc-index.xml`, on 4 November 2026:

| Measure | Value |
|---|---|
| RFC entries | 9,854 |
| with plain text | 9,847 |
| with RFCXML | 1,390; the lowest number with XML is RFC 8650 |
| with an errata link | 2,483 |

| Current status | Count |
|---|---|
| Proposed Standard | 4,434 |
| Informational | 3,013 |
| Unknown (mostly early RFCs) | 887 |
| Experimental | 559 |
| Historic | 353 |
| Best Current Practice | 337 |
| Draft Standard | 138 |
| Internet Standard | 133 |

RFCXML marks requirement keywords (`bcp14`), ABNF (`sourcecode type="abnf"`) and diagrams
(`artwork type="ascii-art"`). Older RFCs are plain text with page headers and footers, so RFCspec
reads both formats into one document model. RFC 6455, the WebSocket RFC, has no XML; RFC 9220,
WebSocket over HTTP/3, has.

## Kinds

The index has no field that says what an RFC specifies, so RFCspec classifies each RFC into one
primary kind.

| Kind | Examples | What RFCspec produces |
|---|---|---|
| K1 wire protocol | TCP (9293), UDP (768), IP (791), DNS (1035), NTP (5905), HTTP/1.1 (9112), HTTP/2 (9113), WebSocket (6455), QUIC (9000) | catalogue, wire sidecar, machines, communication semantics, limits |
| K2 data format or encoding | JSON (8259), CBOR (8949), URI (3986), base64 (4648), ABNF (5234) | grammar or layout, value types, round-trip and negative vectors |
| K3 algorithm or transform | HMAC test vectors (4231), hashes, ciphers, DEFLATE (1951) | a function signature and the RFC's test vectors; the body is a reviewed library |
| K4 extension or update | permessage-deflate (7692), WebSocket over HTTP/2 (8441) | an overlay on the base protocol |
| K5 registry procedure | WebSocket subprotocol registry (7936) | registry entries from IANA's XML; procedures catalogued only |
| K6 management data model | MIB modules, YANG modules | a requirement catalogue only |
| K7 architecture, requirements, process | frameworks, problem statements, BCPs | a requirement catalogue only |
| K8 not a specification | "Not Issued" placeholders, index summaries | nothing |

## What is modelled

Decided on 4 November 2026: K1, K2, K4 and K5 are modelled in full; K3 as signatures with the
RFC's test vectors; K6 and K7 get a requirement catalogue; K8 nothing.

K6 stays out because YANG (RFC 7950) and SMIv2 MIBs (RFC 2578) are already formal languages with
their own parsers. Turning them into ESS is an importer for those languages, not an RFC problem.
