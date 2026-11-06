---
title: Lowering to ESS
sidebar_position: 4
description: Which ESS document each catalogue item becomes, what ESS cannot carry yet, the wire-format sidecar, and the native and Spindle synthesis paths.
lede: RFCspec lowers what ESS can carry and lists the rest as named gaps, so a specification never looks complete while an RFC rule is lost.
source: design sections 7, 8 and 10 and decisions D2, D3, D9; ess-protospec/1 as of ESS 0.86.0
source_url: https://github.com/beyond10x/rfcspec/blob/main/docs/design/rfcspec.md
status: decided
---

:::note[Built in part, v0.2.0]
`rfcspec lower` turns each profile's overlay machines into `ess-protospec/1` models, eight across
the four profiles, and each passes `ess protocol explore` with its input witness in the gate.
`rfcspec generate` writes a Rust codec crate per profile from its layout. Types and registries in
an `ess/N` system, the wire sidecar and the Spindle path are not built yet.
:::

## What becomes what

| Catalogue item | ESS form |
|---|---|
| machine | `ess-protospec/1` participants and transitions |
| message and frame types | protospec messages with typed fields, and the same types in an `ess/N` system for applications |
| registry | an `ess/N` enum; an open registry also keeps a path for values IANA assigns later |
| limit | a typed configuration value, with the RFC's minimum as a guard |
| bytes and bits | the `rfcspec-wire/1` sidecar |
| requirement statuses | RFCspec's requirement matrix; ESS has no requirement noun |

## Gaps

Each gap is a proposal to ESS, raised only when a named RFC needs it. Until it is closed, the
lowering marks the requirement unsupported with the gap's id, so the matrix shows it.

| Gap | Needed by |
|---|---|
| G1 guards beyond equality: ordering, ranges, set membership | WebSocket close-code ranges, payload length classes |
| G2 modular arithmetic | TCP sequence numbers, NTP timestamps |
| G3 byte-string fields with length bounds | every payload; the WebSocket masking key |
| G4 participants created at run time, keyed by an id | HTTP/2 and QUIC streams, concurrent DNS queries |
| G5 layering one protocol over another's stream | WebSocket over HTTP/2 or HTTP/3 |
| G6 a half-duplex turn rule | HTTP/1.1 exchanges, request-response in general |
| G7 timers whose duration comes from a field | NTP poll interval, TCP retransmission timeout |
| G8 the wire layer | every wire protocol and data format |
| G9 a close that discards what is in flight to the closed side | WebSocket's closing timeout; every connection teardown |
| G10 a guard on a message pending on an incoming channel | WebSocket's Pong for only the most recent Ping |
| G11 a timer re-armed on every expiry that still passes exploration | the NTP poll process, which runs continuously |
| G12 duplication on a channel that does not exhaust exploration | NTP over UDP |
| G13 a send to a full channel on a lossy layer that drops instead of stopping | NTP over UDP |
| G14 a property that an event or state is reached | the NTP reach register |

G9 was found by `ess protocol explore` on the first draft of the WebSocket model; the
[WebSocket](./../examples/websocket.md) page tells how.

## The wire sidecar

ESS's protocol format excludes binary parsing. RFCspec keeps bytes and bits in `rfcspec-wire/1`:
ABNF verbatim (RFC 5234, RFC 7405, and RFC 2616's older notation that RFC 6455 still uses),
bit-level ABNF such as RFC 6455 section 5.2, and packet layouts. Each rule bound to a type gets an
incremental parser that resumes across reads and a printer that emits only what the parser
accepts, both bounded by the protocol's limits. Built so far: `rfcspec-abnf` parses RFC 5234 and
RFC 7405 ABNF, resolves references to rules, core rules and imports from other RFCs, and matches
input against a rule; `rfcspec-wire` collects each section's ABNF figures into a catalogue
grammar. The sidecar moves to ESS as a proposal once two
protocols with different notations, WebSocket and DNS, prove it.

## Synthesis

**Native.** One generated sans-IO Rust crate per profile: `wire` (parsers and printers), `machine`
(a pure step function), `limits`, `registry`, and `roles` over a byte-stream or message-stream
trait. Sans-IO lets the same core run under Tokio, in a test harness, in WASM and in Spindle.

**Spindle** ([GitHub](https://github.com/beyond10x/spindle)) runs a compiled ESS model directly and
keeps protocols out of application specifications. So the layers split:

| Layer | Runs where |
|---|---|
| wire codec | native, or as a pure WASM function through Spindle's `call_pure` instruction |
| protocol machine | native now; on Spindle once it executes `ess-protospec/1` |
| the application over the protocol | Spindle; its environment binds the exchange to an RFCspec-generated port |
