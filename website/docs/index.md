---
title: RFCspec
sidebar_label: Overview
sidebar_position: 1
description: RFCspec turns an RFC into an executable specification, a generated protocol core and a conformance suite, with every item cited back to the RFC.
---

RFCspec turns an RFC into an executable specification. It reads the RFC as the RFC Editor
publishes it and produces a **catalogue**: the RFC's requirements, grammars, packet layouts, IANA
registries, defined procedures and limits, each tied to the exact text it came from. The catalogue
is then lowered into [ESS](https://beyond10x.github.io/ess/)
([GitHub](https://github.com/beyond10x/ess)) documents for the protocol's state machines and
types, plus a wire-format sidecar for its bytes and bits. From those, RFCspec generates a
protocol core (codec, state machine, limits, roles) and a conformance suite that any
implementation can be held to.

:::caution[Design stage]
Nothing of the pipeline is implemented yet. This site describes the design and the first worked
example, WebSocket (RFC 6455). The [status page](/docs/status) lists what is decided and what is
planned.
:::

## What RFCspec is not

- **Not a reader that understands RFCs alone.** Parts of an RFC need judgement: state machines
  written as numbered steps, constraints in grammar comments, "unless negotiated otherwise". AI
  proposes that structure; a reviewer decides; every decision is cited and replayable
  ([AI proposals](./concepts/ai-proposals.md)).
- **Not a proof.** Bounded exploration and conformance runs are evidence within their bounds.
- **Not a cryptography library.** TLS record protection, hashes and ciphers are treated as opaque
  functions with the RFC's test vectors.
- **Not an RFC editor or renderer.** RFCspec reads RFCs.

## Where it sits

| Component | Relation |
|---|---|
| [ESS](https://beyond10x.github.io/ess/) ([GitHub](https://github.com/beyond10x/ess)) | the target: RFCspec lowers machines to `ess-protospec/1` and types to an `ess/N` system, and checks them with `ess` |
| Spindle ([GitHub](https://github.com/beyond10x/spindle); no documentation site yet) | a consumer: RFCspec-generated codecs and protocol cores become Spindle ports, so Spindle applications stay protocol-free |
| [AEP](https://beyond10x.github.io/aep/) ([GitHub](https://github.com/beyond10x/aep)) | plans RFCspec's own work |

## Where to start

| You want | Read |
|---|---|
| What exists and how to look at it | [Getting started](./getting-started.md) |
| Which RFCs are worth modelling | [Kinds of RFC](./concepts/rfc-kinds.md) |
| What an RFC becomes | [The catalogue](./concepts/catalogue.md) |
| Request-response, sessions, duplex, streams | [Communication semantics](./concepts/communication-semantics.md) |
| What ESS can carry today, and the gaps | [Lowering to ESS](./concepts/lowering-to-ess.md) |
| The first protocol end to end | [WebSocket](./examples/websocket.md) |
| Every decision with its reasons | [Design](/docs/design) |
