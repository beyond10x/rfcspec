---
format: aep.planning-md/3
id: epic:websocket-profile
kind: epic
status: implemented
title: WebSocket (RFC 6455) as the catalogue's second profile
relations:
- depends_on: epic:public-protocol-catalog
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T00:11:33Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-05T00:11:33Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-05T00:42:15Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

The catalogue's second entry: `profiles/websocket`, RFC 6455 (The WebSocket Protocol), carried
through the same pipeline as `profiles/time` and shown at `/rfcspec/docs/catalog/websocket`.
RFC 6455 is the first BCP 14 RFC in the catalogue (capitalised MUST/SHOULD/MAY), so it exercises
the mechanical extraction path that RFC 868 (prose only) did not, and the design's worked example
(`docs/design/rfcspec.md` §12, `examples/websocket/close.protospec.yaml`) becomes a real profile.

## In this wave

- Source: `rfcspec fetch --rfc 6455`, locked; the reader handles its page furniture, section
  headings and ABNF/figure blocks.
- Catalogue: every BCP 14 requirement of RFC 6455 extracted with its section and span; the
  overlay reviews the requirements of §5.2 (base framing), §5.5 (control frames) and §7 (closing
  the connection) and marks the rest `not-applicable` with a reason or leaves them unreviewed,
  shown as such.
- Machines: the closing handshake of §7 lowered to `ess-protospec/1`; `ess protocol explore`
  passes; the G9 gap of design §7 recorded where the model needs it.
- Layout and codec: the §5.2 base frame header (FIN, RSV1–3, opcode, MASK, payload length 7/16/64
  bit, masking key) as a layout; `rfcspec generate rust` writes a frame-header codec crate tested
  with the RFC's own §5.7 examples.
- Semantics: one layer named `tcp` (the transport, per `decision-blocker:catalog-multi-transport-semantics`),
  unit `bytes` (design §6.2: frames and messages belong to the wire layer), ordered, reliable,
  full-duplex, single, session, correlation none, lifetime connection; no `over`, because no `tcp`
  profile exists.
- Catalogue: the index lists `time` and `websocket`; the WebSocket page shows its requirement
  matrix, ESS gaps and downloads.

## Not in this wave

- The opening handshake's HTTP/1.1 parsing (RFC 2616 ABNF, RFC 7230), extensions (RFC 7692),
  WebSocket over HTTP/2 or HTTP/3 (RFC 8441, RFC 9220), a live verify probe.

## Acceptance

On a clean checkout, `task check` exits 0 and runs `rfcspec check --all profiles` (now two
profiles), `lower --check`, `ess protocol explore` on the closing model (`"verdict": "passed"`),
`generate --check` and `cargo test` of the generated frame codec; the built site's index lists
`time` and `websocket`, and `/rfcspec/docs/catalog/websocket` shows RFC 6455 linked to
rfc-editor.org, the `tcp` semantics row, the requirement counts by level and status, and the
downloads byte-identical to `profiles/websocket/generated/`.
