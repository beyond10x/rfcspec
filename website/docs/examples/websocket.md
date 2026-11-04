---
title: WebSocket
sidebar_position: 1
description: The first profile - RFC 6455 and its updates, what extraction finds in it, its communication semantics, the closing-handshake model in ESS, and how conformance is measured.
lede: WebSocket is the first protocol RFCspec takes end to end, and its hand-written closing-handshake model already found a gap in ESS.
source: design section 12; examples/websocket in the repository
source_url: https://github.com/beyond10x/rfcspec/tree/main/examples/websocket
status: planned
---

:::caution[Design stage]
The model on this page is written by hand and checked with `ess`. Extraction, proposals, the
generated crate and the Autobahn run are planned; see [Status](/docs/status).
:::

## Profile

| RFC | Title | Role in the profile |
|---|---|---|
| 6455 | The WebSocket Protocol | base; plain text only, so it tests the text reader |
| 7936 | Clarifying Registry Procedures for the WebSocket Subprotocol Name Registry | K5, catalogue only |
| 8307 | Well-Known URIs for the WebSocket Protocol | K4 |
| 8441 | Bootstrapping WebSockets with HTTP/2 | K4, layering over an HTTP/2 stream |
| 9220 | Bootstrapping WebSockets with HTTP/3 | K4, layering over an HTTP/3 stream; has RFCXML, so it tests the XML reader |
| 7692 | Compression Extensions for WebSocket | optional K4; DEFLATE itself is K3 |

The first milestone takes RFC 6455 alone, over TCP, client and server roles, no extensions.

## What extraction finds in RFC 6455

| Item | Where | Notes |
|---|---|---|
| requirements | whole text | the masking pair in section 5.1: "a client MUST mask all frames that it sends to the server" and "A server MUST NOT mask any frames that it sends to the client" |
| handshake grammar | section 4.3 | octet ABNF that imports HTTP rules |
| frame grammar | section 5.2 | bit-level ABNF, with page headers inside it and requirements in its comments |
| frame layout | section 5.2 figure | a packet diagram, checked against the bit ABNF |
| defined procedures | `_Fail the WebSocket Connection_`, `_Start the WebSocket Closing Handshake_`, `_The WebSocket Connection is Closed_` | each becomes a named group of transitions |
| connection states | `CONNECTING`, `CLOSING`, and OPEN and CLOSED implied by the procedures | the full set is proposed by AI and reviewed |
| registries | IANA's `websocket.xml`: opcode, close code, header bits, version, extension, subprotocol | enums; opcode and close code are open |
| limits | maximum frame size, message size, closing timeout | left to the implementation, no RFC minimum |

## Communication semantics

| Axis | Value |
|---|---|
| unit | bytes, then frames, then messages |
| delivery | ordered, reliable |
| direction | full-duplex |
| multiplexing | single |
| exchange | session |
| correlation | none |
| lifetime | connection |
| over | TCP; an HTTP/2 extended CONNECT stream (8441); an HTTP/3 stream (9220) |
| roles | client, server |

The opening handshake is a request-response exchange of HTTP; after it, the same connection
carries the WebSocket session. That switch is gap G5 on [Lowering to ESS](../concepts/lowering-to-ess.md).

## The closing-handshake model

[`examples/websocket/close.protospec.yaml`](https://github.com/beyond10x/rfcspec/blob/main/examples/websocket/close.protospec.yaml)
models the client starting the closing handshake and the server echoing the close frame. Frames are
messages here; their byte form belongs to the wire sidecar.

```yaml title="examples/websocket/close.protospec.yaml (excerpt)"
participants:
  - name: client
    initial: open
    states: [open, closing, closed]
    timers: [close-timeout]
    transitions:
      - name: start-closing-handshake
        from: open
        to: closing
        trigger: {kind: input, name: close}
        effects:
          - kind: send
            channel: c2s
            message: close-frame
            payload: {code: {kind: input, field: code}, masked: {kind: literal, value: true}}
          - {kind: arm, timer: close-timeout, after: {kind: fixed, millis: 5000}}
      - name: close-timeout
        from: closing
        to: closed
        trigger: {kind: timer, name: close-timeout}
        effects:
          - {kind: emit, name: connection-failed}
  - name: server
    transitions:
      - name: echo-close
        from: open
        to: closed
        trigger: {kind: receive, channel: c2s, message: close-frame}
        effects:
          - kind: send
            channel: s2c
            message: close-frame
            payload: {code: {kind: literal, value: 1000}, masked: {kind: literal, value: false}}
          - {kind: flush, channel: s2c}
          - {kind: close}
properties:
  - {kind: flush_before_close, name: server-flushes-close, participant: server, channel: s2c}
  - {kind: event_count, name: closed-once, participant: client, event: connection-closed, max: 1}
```

`ess protocol explore` with the witness in `close.inputs.json`, a client `close` with code 1000,
returns `"verdict": "passed"` with 10 states and 11 transitions (ess 0.86.0, 4 November 2026). The
full command and output are on [Getting started](../getting-started.md).

## What exploration found: gap G9

RFC 6455 section 7.1.7 has a client whose closing timeout fires drop the TCP connection. The first
draft of the model therefore had a `close` effect in the client's `close-timeout` transition.
Exploration returned `inconclusive`: "dead end with outstanding queue, timer or flush obligations".
The server's close frame was still queued on `s2c`, and a closed client can never receive it. On TCP
that frame is discarded with the connection; `ess-protospec/1` has no way to say that closing a
participant discards what is in flight to it. The model drops the client's `close` effect, and the
missing rule is gap G9. Exploration found it on the first run.

## Where the boundary is today

| RFC rule | Expressible in ESS today |
|---|---|
| "A server MUST NOT mask any frames" | yes: a literal `masked: false` in the server's send |
| a server receiving an unmasked client frame fails the connection | yes: a guard on the received message's `masked` field |
| the close code is in 1000 to 4999 and not reserved | no: needs gap G1, guards beyond equality |

## Generated crate (planned)

`rfcspec-websocket`, generated and never edited by hand:

| Module | Holds |
|---|---|
| `wire` | frame parse and print with the three length encodings and masking; handshake validation from section 4.3's grammar |
| `machine` | client and server step functions from the model |
| `limits` | maximum frame, maximum message, closing timeout |
| `registry` | opcodes and close codes |
| `roles` | client and server over a byte stream |

UTF-8 validation of text messages (section 8.1) is a K3 library call, linked to its requirements.

## Conformance (planned)

The independent suite is [Autobahn Testsuite](https://github.com/crossbario/autobahn-testsuite),
with over 500 cases for clients and servers. RFCspec runs it in a container against the generated
server and client, maps each case to the requirements it exercises, and reports two numbers per
run: Autobahn's verdicts, and which requirement rows only RFCspec's generated tests cover and which
only Autobahn covers.

The milestone is done when every MUST-level requirement of RFC 6455 in scope has a status, the
generated server passes Autobahn's framing, reserved bits, opcodes, fragmentation, UTF-8 and closing
categories without failures, and every MUST row is pass, a gap with its ESS gap id, or not
applicable with a reason.
