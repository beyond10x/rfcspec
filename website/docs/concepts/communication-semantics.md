---
title: Communication semantics
sidebar_position: 3
description: A fixed vocabulary for what kind of conversation a protocol is - request-response or session, half or full duplex, one stream or many - and how it maps to ESS channels.
lede: Every protocol declares one value per axis for each of its layers, and the lowering to ESS checks that each value survives.
source: design section 6 and decision D4
source_url: https://github.com/beyond10x/rfcspec/blob/main/docs/design/rfcspec.md
status: decided
---

:::caution[Decided, not built]
The vocabulary is a design decision. The values in the table below are design input, to be
confirmed against each RFC when it is imported.
:::

## The axes

| Axis | Values | What it decides |
|---|---|---|
| unit | `bytes`, `messages`, `datagrams` | whether framing is the protocol's job and a codec must resume across reads |
| delivery | ordered or not; reliable or lossy; duplicates possible or not | the ESS channel's `ordering`, `loss` and `duplication` |
| direction | `simplex`, `half-duplex`, `full-duplex` | one channel, two with a turn rule, or two independent channels |
| multiplexing | `single`, `streams`, `sessions` | whether participants are created at run time |
| exchange | `request-response`, `request-stream`, `session`, `publish`, `one-shot` | what the protocol offers an application |
| correlation | `none`, `by-order`, `by-id` | how a reply finds its request, and what a duplicate is |
| lifetime | `connectionless`, `connection`, `association` | the outer machine every protocol gets |
| layering | `over` another protocol's stream or exchange | which lower layer supplies the channel |
| roles | named parties: client, server, resolver, proxy, sender, recipient | requirement scope, and the generated entry points |

## Examples

| Protocol | Unit | Direction | Multiplexing | Exchange | Correlation | Lifetime | Over |
|---|---|---|---|---|---|---|---|
| IP (791) | datagrams | simplex | single | one-shot | none | connectionless | link |
| UDP (768) | datagrams | simplex | single | one-shot | none | connectionless | IP |
| TCP (9293) | bytes | full-duplex | single | session | none | connection | IP |
| DNS (1035) | messages | half-duplex | sessions | request-response | by id | connectionless over UDP; connection over TCP | UDP or TCP |
| NTP (5905) | datagrams | half-duplex | single | request-response | by id (the origin timestamp) | association | UDP |
| HTTP/1.1 (9112) | bytes | half-duplex | single | request-response | by order | connection | TCP or TLS |
| HTTP/2 (9113) | bytes | full-duplex | streams | request-response | by id | connection | TCP or TLS |
| WebSocket (6455) | bytes | full-duplex | single | session | none | connection | TCP; an HTTP/2 stream (8441); an HTTP/3 stream (9220) |

The unit is what the carrier hands the protocol; frames and messages a protocol builds itself belong
to its wire layer. A port number selects an endpoint, which is addressing, not multiplexing.

## What ESS can express today

`ess-protospec/1` has fixed participants, channels with `fifo` or `unordered` ordering, a capacity,
and optional loss and duplication, timers, and equality guards. So `messages` as the unit, every
delivery value, `simplex` and `full-duplex`, `single` multiplexing and connection lifetimes with
timers lower today. Run-time streams (HTTP/2, QUIC, concurrent DNS queries), layering, and a
half-duplex turn rule are gaps, listed on [Lowering to ESS](./lowering-to-ess.md).
