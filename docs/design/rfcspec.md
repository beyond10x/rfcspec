# RFCspec: RFCs as executable specifications

**Status:** design proposal; nothing is implemented. **Date:** 4 November 2026.
**Scope:** taking any RFC published by the RFC Editor, turning what it specifies into ESS documents
plus a wire-format sidecar, and from those generating clients, servers, codecs and conformance
suites, natively or on Spindle. Worked example: WebSocket (RFC 6455 and the RFCs that update it).

Origin (Timo, 4 November 2026): look at ESS, at Ignite's RFC binding, and at RFCs in general; find
"some programmatic way of taking any RFC from the website and turn it into ESS", "even with the help
of AI", "into rules and such", because this "would allow us to generate clients automatically,
generate conformance tests". In a follow-up: "we need a way to specify or express the semantics of
transports, protocols, things like request response, session, half duplex, full duplex, parallel
streams"; the target is "mostly communication things between two services", such as "TCP, IP,
DNS, this time protocol"; look at which classes of RFC exist and decide together whether other
kinds are worth modelling; and "can this be turned into ESS and then reliably either synthesized
into a native target language or turned into Spindle".

Citations: `ess:<path>` is a file in `beyond10x/ess` at `origin/main` (0.86.0, read 4 November
2026); `spindle:<path>` is a file in `beyond10x/spindle`; `ignite:protocols.md §n` is Ignite's
protocol design (1 November 2026); `[Pn]` is an external source in section 17. Every corpus number
in section 2 was counted from `rfc-index.xml` downloaded on 4 November 2026 (9,854 entries).

## 0 Decisions at a glance

| # | Question | Decision |
|---|---|---|
| D1 | What an RFC becomes | A **catalogue** (`rfcspec/1`): the RFC's requirements, grammars, layouts, registries, defined procedures and limits, each with a source span and a digest. The catalogue is data about the RFC; it is not yet a specification of a system. |
| D2 | What ESS receives | The catalogue is **lowered**: the protocol's machines to `ess-protospec/1`, its message and value types, enums and registries to an `ess/N` system, its limits to typed configuration. Nothing is lowered that ESS cannot carry; what it cannot carry is listed as a gap (section 7). |
| D3 | Wire formats | Bytes and bits live in an RFCspec sidecar, `rfcspec-wire/1`: RFC 5234 ABNF verbatim, bit-level ABNF, and packet layouts. ESS's protocol format excludes "binary parsing" (`ess:docs/design/protocol-specification.md`). The sidecar moves to ESS as a proposal once two protocols with different notations (WebSocket text and bits, DNS binary) prove it. |
| D4 | Communication semantics | A small closed vocabulary (section 6): channel kind, delivery, duplex, multiplexing, exchange pattern, layering, lifetime. Each RFC protocol declares one value per axis; the lowering checks each against what `ess-protospec/1` can express and refuses the rest. |
| D5 | Determinism | Stages S0 to S2 and S5 to S7 are deterministic functions of locked sources and reviewed decisions. AI runs only in S3, produces proposals with cited spans, and is recorded and replayed. No gate calls a model. |
| D6 | Review | Human and AI decisions live in an authored **overlay** next to the generated catalogue, keyed by stable id and the digest of the text they judged. Re-import never loses them; a changed source text marks them stale. |
| D7 | Sources | RFCXML v3 where it exists (RFC 8650 onward; 1,390 RFCs), plain text otherwise (all 9,854). IANA registries from IANA's XML, never from the RFC's IANA section. Errata are applied only when verified, as overlay entries. |
| D8 | Composition | A **profile** names a set of RFCs and the parts of each in scope (WebSocket = 6455 + 7936 + 8307 + 8441 + 9220, optionally 7692). Requirement statuses are per profile. |
| D9 | Synthesis | Native first: a generated sans-IO Rust crate per profile (codec, machine, limits, roles). Spindle consumes the same artifacts: the codec as a `call_pure` WASM module or a native port, the session machine as a protospec program once Spindle runs one (section 10). |
| D10 | Conformance | A requirement matrix with Ignite's verdicts (pass, fail, gap, deviation, inconclusive, n/a); test sources are generated vectors, protospec traces, fuzzing, and an independent external suite where one exists (Autobahn for WebSocket). |
| D11 | Which RFCs | Decided by Timo, 4 November 2026 (option A): wire protocols (K1), data formats (K2), extensions (K4) and registries (K5) are modelled; algorithms (K3) as signatures with the RFC's test vectors; management models (K6: YANG, MIB), architecture and process documents (K7) get a requirement catalogue only; K8 nothing. Section 2.3. |
| D12 | Repository | `beyond10x/rfcspec`, a Rust workspace of about ten crates (section 13); binary `rfcspec`; RFCspec is itself specified in ESS (`ess/`) and planned in AEP. |

## 1 Goal and non-goals

**Goal.** For an RFC (or a profile of several), produce, with a recorded trail from every artifact
back to a sentence or a figure of the RFC:

1. a requirement catalogue with a status per requirement for a given implementation;
2. ESS documents a reviewer can read and `ess specify validate` accepts;
3. a generated implementation core (codec, machine, limits) whose every MUST in scope is either
   enforced by construction, covered by a generated test, or marked unsupported with a reason;
4. a conformance suite that can be pointed at any implementation, generated or not.

**Non-goals.**

- Understanding an RFC without review. An RFC is prose written for implementers; parts of it
  (state machines described as numbered steps, field constraints in comments, "unless negotiated
  otherwise") need judgement. RFCspec makes that judgement explicit, cited and replayable; it does
  not remove it.
- Proving an implementation correct. Bounded exploration and conformance are evidence within their
  bounds, as ESS says of its own protocol checker (`ess:docs/design/protocol-specification.md`,
  "Checking and evidence").
- Cryptographic semantics in machines. TLS, QUIC packet protection and similar are treated as
  opaque transforms with test vectors (section 2.3, kind K3), as Ignite delegates TLS to a sans-IO
  engine (`ignite:protocols.md` D9).
- Rendering or editing RFCs. RFCspec reads them.

## 2 What RFCs are

### 2.1 The corpus

Counted from `rfc-index.xml` (`https://www.rfc-editor.org/rfc-index.xml`, 4 November 2026):

| measure | value |
|---|---|
| RFC entries | 9,854 |
| with plain text | 9,847 |
| with RFCXML (`XML` format) | 1,390; the lowest number with XML is RFC 8650 |
| with PDF | 1,522 |
| entries with an `<errata-url>` | 2,483; RFC 6455's info page shows 18 under "Errata" |

| current status | count |
|---|---|
| Proposed Standard | 4,434 |
| Informational | 3,013 |
| Unknown (mostly early RFCs) | 887 |
| Experimental | 559 |
| Historic | 353 |
| Best Current Practice | 337 |
| Draft Standard | 138 |
| Internet Standard | 133 |

| stream | count |
|---|---|
| IETF | 7,261 |
| Legacy | 1,893 |
| Independent | 435 |
| IAB | 134 |
| IRTF | 128 |
| Editorial | 3 |

Areas with the most RFCs: routing (rtg) 1,393; operations (ops) 927; internet (int) 904; security
(sec) 807; applications (app) 544; art 430; rai 415; wit 380; tsv 357. Many entries carry no area.

Each entry also carries `obsoletes`, `obsoleted-by`, `updates`, `updated-by`, keywords, abstract,
working group and DOI. RFCspec reads these to build profiles (D8) and to follow ABNF imports through
obsoleted RFCs (section 8.3).

### 2.2 Source formats

| format | from | what it gives | what it lacks |
|---|---|---|---|
| RFCXML v3 [P3] | RFC 8650 onward | `<section>` tree with anchors; `<bcp14>` marks on keywords (RFC 7991 §2.9); `<sourcecode type="abnf">` (RFC 7991 §2.48.4, "abnf" is the first preferred value); `<artwork type="ascii-art">` for diagrams (§2.5.7); `<xref>` references; `<iref>` index terms | nothing semantic beyond markup; ABNF and diagrams are still text |
| plain text | every RFC | the text, paginated: a page footer and header every page (RFC 6455 has 71 `[Page n]` footers) | sections are recovered from the table of contents and numbering; keywords from capitalisation (RFC 8174 [P4]: they count "only when they are in all capitals"); ABNF and diagrams from indentation and shape |
| `rfc-index.xml` | all | metadata and the update graph | content |
| IANA registry XML | registries | authoritative current values: e.g. `https://www.iana.org/assignments/websocket/websocket.xml` holds six registries (`extension-name`, `subprotocol-name`, `version-number`, `close-code-number`, `opcode`, `framing-header-bits`) | which RFC sentence governs a value; that comes from the registry's reference column |
| errata | `https://www.rfc-editor.org/errata/rfcNNNN` | reported, verified, held and rejected corrections | a machine-readable diff; an erratum is applied by a reviewed overlay entry |

The pagination of the text format reaches into figures: RFC 6455's framing ABNF in §5.2 is broken
by a page footer and header between `frame-opcode-control` and `frame-masked` (lines 1738 and 1740
of `rfc6455.txt`). The text reader removes page furniture before anything else reads the text.

### 2.3 Kinds of RFC, and which are worth modelling

The index has no field for what an RFC specifies. RFCspec classifies each RFC into one primary kind
(S2, partly AI-proposed and reviewed). The title-keyword hits below are a rough first look, not a
classification: the patterns overlap, and many RFCs match none.

| title pattern (case-sensitive) | titles matching |
|---|---|
| `Protocol` | 2,337 |
| `Extension` or `Option` | 925 |
| `Framework`, `Architecture`, `Requirements`, `Problem Statement`, `Use Cases` | 719 |
| `Format`, `Encoding`, `Syntax`, `Schema`, `Media Type`, `JSON`, `CBOR`, `URI`, `ASN` | 710 |
| `Minutes`, `Status`, `Report`, `Survey`, `Guideline`, `Considerations`, … | 509 |
| `Algorithm`, `Cipher`, `Hash`, `Signature`, `Encryption`, `Cryptograph` | 463 |
| `IANA`, `Registry`, `Registration` | 297 |
| `MIB`, `Management Information Base` | 250 |
| `YANG` | 137 |

| kind | examples | what RFCspec produces | ESS target | worth it |
|---|---|---|---|---|
| K1 wire protocol | TCP (9293), UDP (768), IP (791), DNS (1035), NTP (5905), HTTP/1.1 (9112), HTTP/2 (9113), WebSocket (6455), QUIC (9000) | catalogue, wire sidecar, machines, communication semantics, limits | `ess-protospec/1` + `ess/N` types | yes: the main case |
| K2 data format or encoding | JSON (8259), CBOR (8949), URI (3986), base64 (4648), ABNF itself (5234) | grammar or layout, value types, round-trip and negative vectors | `ess/N` types; the grammar in the sidecar | yes: every K1 protocol imports some |
| K3 algorithm or transform | HMAC test vectors (4231), hashes, ciphers, compression (DEFLATE 1951) | a pure function signature, the RFC's test vectors as conformance cases; the body is a reviewed native library, never generated | an `ess/N` operation with vectors | yes, as signatures plus vectors |
| K4 extension or update | permessage-deflate (7692), WebSocket over HTTP/2 (8441), TCP options | an overlay on the base profile: new registry values, new grammar alternatives, new transitions, changed requirements | same as the base | yes: a profile is incomplete without them |
| K5 registry procedure | 7936 (WebSocket subprotocol registry procedures) | registry entries from IANA XML; procedural requirements are catalogued, not executed | enums in `ess/N` | registries yes; procedures catalogue only |
| K6 management data model | MIBs (SMIv2), YANG modules | the module itself is already machine-readable; a separate importer from YANG to `ess/N` | `ess/N` | later, separate importer |
| K7 architecture, requirements, BCP, informational | frameworks, problem statements, operational guidance, RFC 2119 itself | a requirement catalogue only, for traceability | none | catalogue only |
| K8 not a specification | "Not Issued" placeholders, index summaries, minutes, April 1 RFCs | nothing | none | no |

Decided (D11, option A): K6 stays out. YANG (RFC 7950) and SMIv2 MIBs (RFC 2578) are already
formal languages with their own parsers; their value is a YANG→ESS importer, which is not an RFC
problem. Revisit if a protocol profile needs a YANG model (NETCONF, RESTCONF).

## 3 Sources, lock and provenance

`rfcspec fetch <profile>` resolves a profile to RFC numbers, follows `updated-by` to list candidates
(it never adds them silently; a profile names what it includes), and downloads:

- `rfcNNNN.xml` if the index lists XML, else `rfcNNNN.txt`;
- the index entry;
- the errata page;
- every IANA registry the RFC's IANA section or the profile names.

Each file is stored under `sources/` with its SHA-256 in `rfcspec.lock`. Everything after S0 reads
only locked files and runs offline; CI never fetches. `rfcspec check-sources` compares the lock with
the live sites and reports new errata, new `updated-by` entries and changed registries as drift; it
changes nothing.

**Provenance.** Every catalogue item carries a `Span`: the RFC, the section anchor (XML) or number
(text), a byte range in the normalized text, and the SHA-256 of the spanned text. An item whose
span no longer matches after re-import keeps its id and is marked `stale`; its overlay decisions
are not applied until a reviewer confirms them.

## 4 Pipeline

| stage | input | output | deterministic | tool |
|---|---|---|---|---|
| S0 fetch | profile | `sources/`, `rfcspec.lock` | network; recorded | `rfcspec fetch` |
| S1 read | locked source | normalized document: section tree, paragraphs, lists, figures, references, defined terms | yes | `rfcspec-doc` |
| S2 extract | document | catalogue items found mechanically: requirements, ABNF rules, layouts, registries, defined procedures, references | yes | `rfcspec-extract` |
| S3 propose | catalogue + document | proposals: party of each requirement, kind of the RFC, machines, constraints from prose, links requirement → model element, communication semantics | no; recorded and replayed | `rfcspec-propose` |
| S4 review | proposals | the overlay: accepted, edited, rejected; statuses (`unsupported`, `not_applicable`) | authored | `rfcspec review` |
| S5 lower | catalogue + overlay | `ess-protospec/1`, `ess/N` system, `ess-inputs.yaml`, `rfcspec-wire/1` | yes | `rfcspec-ess` |
| S6 synthesize | lowered documents | Rust crate (codec, machine, limits, roles); WASM codec module | yes | `rfcspec-gen` |
| S7 verify | lowered documents + a target | requirement matrix, vectors, traces, external suite results | yes, given the target | `rfcspec-conformance`, `ess verify` |

The overlay is the only authored file per profile besides the profile itself. Generated files are
never edited by hand; `task check` regenerates them and fails on drift, as ESS projections do.

## 5 The catalogue, `rfcspec/1`

Proposed nouns (RFCspec's own ESS domain specifies them, D12):

| noun | identity | holds |
|---|---|---|
| `Rfc` | number | title, status, stream, area, date, format read, digest, update graph edges |
| `Section` | rfc, anchor or number | title, parent, span |
| `Requirement` | rfc, section, ordinal (`R5.3-2`) | level (the eleven BCP 14 keywords [P2]; negation is part of the level, `MUST NOT`), party (proposed), text, span, source (`bcp14-xml`, `capitals-text`, `abnf-comment`, `prose` for RFCs before BCP 14) |
| `DefinedTerm` | rfc, name | a phrase the RFC defines and invokes, such as RFC 6455's `_Fail the WebSocket Connection_` (§7.1.7); such terms become named procedures in machines |
| `Grammar` | rfc, name | ABNF rules verbatim, their imports (`uses` another RFC's rules), bit-level flag (RFC 6455 §5.2), span |
| `Layout` | rfc, name | a packet diagram parsed to fields with bit widths, dependent lengths, presence conditions, constraints |
| `Registry` | IANA id | entries (value, name, reference, status), lowered to an enum with an open or closed policy |
| `Limit` | rfc, name | a quantity the RFC leaves to the implementation, with any stated minimum or maximum (Ignite's `Limits`, `ignite:protocols.md` §3.6) |
| `Machine` | profile, name | participants, states, triggers, transitions, timers, with a requirement link per transition |
| `Profile` | name | RFCs and the sections and parties in scope, the communication semantics (section 6) |
| `Status` | profile, requirement | one of `satisfied <element>`, `unsupported <reason>`, `not-applicable <reason>`; tests that verify a satisfied requirement are evidence attached to it, read by the conformance matrix (section 11), not a fourth status; MUST-level items in scope need one, as Ignite's E090 rule (`ignite:protocols.md` §3.2) |

A `Limit` also holds the reaction the RFC states for exceeding it, verbatim and cited by the limit's
span, which the limit pairs of section 11 read. That every MUST-level requirement in scope has a
status spans every requirement of a profile, so `rfcspec check` enforces it, not a single command of
the ESS domain.

**Requirements, mechanically.**

- XML: one requirement per sentence containing a `<bcp14>` element.
- Text: one per sentence containing a BCP 14 keyword in capitals, outside the boilerplate paragraph
  that cites RFC 2119. RFC 6455's text has 140 `MUST` occurrences (26 of them in `MUST NOT`), 30
  `SHOULD` (5 in `SHOULD NOT`), 48 `MAY`, 1 `REQUIRED`, 1 `RECOMMENDED`, 2 `OPTIONAL`, counted with
  `grep -o` on the downloaded text, the boilerplate paragraph included.
- ABNF comments: RFC 6455 §5.2 puts a requirement inside the grammar: `frame-rsv1 = %x0 / %x1 ;
  1 bit in length, MUST be 0 unless negotiated otherwise`. Comments are scanned too.
- RFCs before BCP 14 (Telnet, early TCP): S3 proposes requirements from prose, `source = prose`,
  and every one needs review, as Ignite hand-writes its Telnet catalogues (`ignite:protocols.md`
  §3.1).

**Ids.** `R<section>-<ordinal>` within an RFC, qualified by the RFC when crossing (`6455:R5.1-3`).
The ordinal counts requirements in document order within the section. A new RFC revision is a new
RFC number, so ids never shift under a published RFC; only errata can change text, and that is the
`stale` path of section 3.

## 6 Communication semantics

Timo's follow-up asks for a way to say what kind of conversation a protocol is. Ignite separates
transport, stream, connection, protocol, exchange and application (`ignite:protocols.md` D1, §1.1).
RFCspec uses the same stack and adds a fixed set of axes. Every profile declares one value per axis
for each layer; S3 proposes the values with cited sentences, a reviewer accepts them, and S5 checks
that the lowering preserves them.

### 6.1 Axes

| axis | values | what it decides |
|---|---|---|
| unit | `bytes` (a stream without message boundaries), `messages` (boundaries preserved), `datagrams` (bounded, independent) | whether a codec must resume across reads; whether framing is the protocol's job |
| delivery | ordered or not; reliable or lossy; duplicates possible or not | protospec channel `ordering`, `loss`, `duplication` |
| direction | `simplex`, `half-duplex` (one side speaks at a time, with a turn rule), `full-duplex` | one channel, two channels with a turn property, or two independent channels |
| multiplexing | `single`, `streams` (ids, opened by either side, independently ordered), `sessions` (many conversations over one association, e.g. DNS ids over UDP) | whether participants are created at run time |
| exchange | `request-response`, `request-stream`, `session` (long-lived, both sides send), `publish`, `one-shot` | the application-facing type: Ignite's `Exchange<Q, R, S>` or a session record |
| correlation | `none`, `by-order` (HTTP/1.1 pipelining), `by-id` (DNS id, HTTP/2 stream id, JSON-RPC id) | how a reply finds its request; what a duplicate is |
| lifetime | `connectionless`, `connection` (open, established, closing, closed), `association` with timers | the outer machine every profile gets |
| layering | `over <profile>.<exchange or stream>` | which lower layer supplies the channel (WebSocket over TCP; over an HTTP/2 stream for RFC 8441; over an HTTP/3 stream for RFC 9220) |
| roles | named parties (`client`, `server`, `resolver`, `proxy`, `sender`, `recipient`) | requirement scope by party; generated role entry points |

### 6.2 Examples

Proposed values, to be confirmed against each RFC in S3; they are design input, not extracted.

| protocol | unit | delivery | direction | multiplexing | exchange | correlation | lifetime | over |
|---|---|---|---|---|---|---|---|---|
| IP (791) | datagrams | unordered, lossy, duplicates | simplex | single | one-shot | none | connectionless | link |
| UDP (768) | datagrams | unordered, lossy, duplicates | simplex | single | one-shot | none | connectionless | IP |
| TCP (9293) | bytes | ordered, reliable | full-duplex | single | session | none | connection | IP |
| DNS (1035) | messages | as the carrier | half-duplex | sessions | request-response | by-id | connectionless over UDP; connection over TCP | UDP or TCP |
| NTP (5905) | datagrams | unordered, lossy | half-duplex | single | request-response | by-id (the origin timestamp) | association | UDP |
| HTTP/1.1 (9112) | bytes | ordered, reliable | half-duplex | single | request-response | by-order | connection | TCP or TLS |
| HTTP/2 (9113) | bytes | ordered, reliable | full-duplex | streams | request-response | by-id | connection | TCP or TLS |
| WebSocket (6455) | bytes | ordered, reliable | full-duplex | single | session | none | connection | TCP; HTTP/2 stream (8441); HTTP/3 stream (9220) |

`unit` is what the carrier hands the protocol. Units the protocol builds itself (HTTP/2 frames,
WebSocket frames and messages) belong to its wire layer (section 8), not to this axis. A port number
selects an endpoint; it is addressing, not multiplexing, so UDP and TCP are `single`.

### 6.3 What `ess-protospec/1` can carry today

Read from `ess:crates/specify/ess-domain/src/protocol.rs` and
`ess:docs/design/protocol-specification.md`:

| construct | protospec today |
|---|---|
| participants | fixed set, each with states, typed local fields, inputs, timers, transitions |
| channels | `from`, `to`, `ordering: fifo \| unordered`, `capacity`, `loss`, `duplication` |
| triggers | `input`, `receive(channel, message)`, `timer` |
| guards | equality only, between a literal, a local field and an input field; an input field is a field of the triggering local input or of the received message |
| effects | `set`, `send` (with exchange and logical identity), `arm` (fixed or capped backoff), `cancel`, `emit`, `flush`, `close` |
| field types | `String`, `Boolean`, signed 64-bit `Integer` |
| properties | `never_state`, `event_count`, `flush_before_close`, `state_implication` |
| execution | pure; `run`, `replay` of `ess-prototrace/1`, bounded `explore` |
| excluded | "arbitrary executable predicates, unbounded model checking, binary parsing, cryptographic semantics, durable restarts"; dynamic role creation and session epochs are listed as future extensions |

So: unit `messages`, every delivery value, `simplex` and `full-duplex`, `single` multiplexing,
`connection` lifetime with timers, and fixed roles lower today. The rest are the gaps of section 7.

## 7 Lowering to ESS, and the gaps

| catalogue item | ESS form |
|---|---|
| machine | `ess-protospec/1` participants and transitions; each transition carries the requirement ids it implements in its name or a sidecar map (ESS has no annotation field; RFCspec keeps the map) |
| message and frame types | protospec `messages` with typed fields; the same types as `ess/N` value types for applications |
| registry | `ess/N` enum; an open registry (new values assigned by IANA later) also keeps an `unassigned(Integer)` path |
| limit | a typed configuration value with its minimum as a guard |
| requirement statuses | the RFCspec matrix; not an ESS document |
| bytes and bits | `rfcspec-wire/1` (D3), referenced from protospec message names |

**Gaps.** Each is a proposal to ESS, raised only when the named RFC needs it. ESS's rule is to "add
concrete Rust types only when an importer or projector establishes their required semantics"
(`ess/AGENTS.md`, Boundaries); RFCspec is such an importer.

| # | gap | needed by | proposal |
|---|---|---|---|
| G1 | guards beyond equality: `<`, `<=`, ranges, set membership | WebSocket close codes (1000–4999 ranges), payload length classes | ordered comparisons on `Integer`, membership in an enum subset |
| G2 | modular and bounded arithmetic in effects | TCP sequence numbers (2^32 wraparound), windows; NTP timestamps | `add`/`sub` modulo a declared width; checked, no silent wrap |
| G3 | byte-string fields and lengths | every K1 payload; WebSocket masking key, close reason | a `Bytes` field type with length bounds; content stays opaque to guards except length and equality |
| G4 | dynamic participants and ids | HTTP/2 and QUIC streams, DNS concurrent queries | participant families keyed by an id, created by a transition, bounded by a limit; protospec lists "dynamic role creation" as future |
| G5 | layering | WebSocket over HTTP/2 or HTTP/3; HTTP over TLS over TCP | a channel of one protocol bound to an exchange or stream of another, checked by unit and delivery |
| G6 | half-duplex turn rule | HTTP/1.1 per exchange, SMTP, request-response in general | a property `turn(channel_a, channel_b)` or derive it from the machine |
| G7 | timers with values from fields | NTP poll interval, TCP RTO from measurements | `arm(after: field)` with checked bounds |
| G8 | wire layer | all of K1 and K2 | `rfcspec-wire/1` first (D3); upstream proposal after WebSocket and DNS |
| G9 | a `close` that discards transmissions still in flight to the closed participant, on a reliable channel | WebSocket §7.1.7 (client drops TCP after the closing timeout); every connection teardown | channel option `on_close: discard \| obligation`; today a queued message to a closed participant is an outstanding obligation (section 12.4) |
| G10 | a guard on a message pending on an incoming channel | WebSocket §5.5.3 (a Pong for only the most recent Ping, skipping a Ping when a later one is pending) | `pending(channel, message)` as a guard operand |

Until a gap is closed, the lowering refuses or abstracts: it drops the guard and marks the
requirement's status `unsupported: ess-gap G1`, so the matrix shows the gap. It never emits a
specification that looks complete while the RFC's rule is lost.

## 8 The wire layer, `rfcspec-wire/1`

### 8.1 Notations

| RFC notation | example | form in the sidecar |
|---|---|---|
| ABNF on octets (RFC 5234 [P5], RFC 7405 case-sensitive strings [P6]) | RFC 6455 §4.3 handshake header fields; RFC 9112 message head | `grammar` with rules verbatim, `uses` for imports |
| ABNF on bits | RFC 6455 §5.2: "the ABNF in this section is operating on groups of bits" | `grammar` with `unit = bit`; rule widths from the comments (`; 1 bit in length`) proposed in S3 and reviewed, because ABNF has no width syntax |
| packet diagrams | RFC 6455 §5.2 figure; TCP header; DNS header | `layout`: fields, widths, dependent lengths, presence conditions, in the style of EverParse 3D [P10] |
| prose about framing | Telnet IAC escaping; HTTP/1.1 body length (RFC 9112 §6.3) | a pure transition function in the machine, not a grammar (`ignite:protocols.md` §3.4) |
| TLV and length-prefixed lists | DNS resource records, TLS extensions | `layout` with `repeat until length` |

WebSocket has both a bit-level ABNF and a packet diagram for one frame. RFCspec imports both and
checks that they agree (field order, widths, conditions); a disagreement is a finding for review,
not a silent choice.

### 8.2 What is generated

For each grammar rule or layout bound to a type: an incremental parser that resumes across reads
(`Done(value, consumed) | More(state) | Invalid(rule, offset)`), a printer that emits only what the
parser accepts, and both bounded by the profile's limits. This is Ignite's design for `grammar` and
`matches` (`ignite:protocols.md` §3.4), with the same accepted subset: deterministic, bounded
lookahead, every repetition limited. An ambiguous or unbounded rule is refused with both
alternatives named.

### 8.3 Imports across RFCs

RFC 6455 is from December 2011 and says "this document uses both [RFC5234] and [RFC2616] variants of
ABNF in different sections" (text line 723); its handshake rules use RFC 2616's notation and HTTP
rules, and RFC 9110 and 9112 have since replaced RFC 2616. So the ABNF reader has dialects
(`rfc5234`, `rfc2616` with its `#rule` lists and implied whitespace, `bits` for §5.2), and a grammar
records its dialect, proposed in S3 per figure and reviewed. Imports resolve against the RFC
named in the text first; if the profile pins a successor, the resolver maps rule names through a
reviewed table and records each mapping. Core rules (`ALPHA`, `DIGIT`, `OCTET`, …) come from RFC
5234 Appendix B.

## 9 AI in the pipeline

AI is useful exactly where S2 stops: prose to structure. It is never trusted, only checked.

| task | input | output | mechanical check before a human sees it |
|---|---|---|---|
| classify kind (2.3) | index entry, abstract, table of contents | kind and confidence | none beyond schema |
| party of a requirement | the sentence and its section | `client`, `server`, `sender`, … | the party appears in the profile's roles |
| communication semantics (6) | the RFC's overview sections | one value per axis, each with a quoted sentence | every quote occurs verbatim in the locked source |
| machines | sections with numbered steps and defined terms | protospec participants and transitions, each transition citing a span and requirement ids | `ess` validates it; every cited span exists; every MUST in scope is linked or listed as unlinked |
| constraints from prose | "MUST be 0 unless negotiated otherwise" | a guard on a field, conditioned on a negotiated extension | the field exists in the layout |
| bit widths from ABNF comments | `; 1 bit in length` | widths | sum against the packet diagram |
| import mappings | rule names across RFC revisions | a mapping table | both rules exist |

**Contract.** A proposal is a typed document (schema-checked) with the model id, a digest of the
prompt and inputs, the response, and the spans it cites. Proposals are stored in
`proposals/` and replayed: S3 with the same inputs and a recorded response returns the recording,
so CI and every later stage are offline and deterministic. Calls go through the `beyond10x/llm`
crates. A proposal changes nothing until S4 records a decision in the overlay. Accepted
communication semantics reach a profile when a reviewer, under a current decision of that profile's
overlay, replaces the profile's semantics block with them (`ReviseProfile`).

**Prior art.** RFCNLP extracted finite state machines from the TCP and DCCP RFCs with hybrid NLP
and used them for attack synthesis [P11]. SAGE extracted logical forms from RFC text and generated
an interoperable ICMP implementation, and is reported to have struggled with TCP's state [P12, as
summarized by later work]. PROSPER used LLMs for FSM extraction; RFSeek reports better recovery of
diagrammed states and edges than PROSPER and ties each transition to its RFC passage [P13].
FlowFSM splits extraction across chained prompts to reduce invented transitions [P14]. The common
lesson these papers report: extraction is partial and needs a cited trail. RFCspec's contribution
is the trail plus the downstream use: the extracted machine becomes an ESS document that is
validated, explored and conformance-tested, not an end product.

## 10 Synthesis: native and Spindle

### 10.1 Native

`rfcspec generate rust <profile>` writes one crate per profile:

| module | from | shape |
|---|---|---|
| `wire` | `rfcspec-wire/1` | incremental parsers and printers; no I/O |
| `machine` | `ess-protospec/1` | `step(state, event) -> (state, Vec<Action>)`, sans-IO; timers as actions |
| `limits` | catalogue limits | one `Limits` struct with RFC minimums as checked defaults |
| `roles` | profile roles | `Client` and `Server` drivers over a `ByteStream` or `MessageStream` trait; Tokio adapter in a separate crate |
| `registry` | IANA registries | enums with `Unassigned(u…)` for open registries |

Sans-IO is chosen so the same core runs under Tokio, in a test harness, in WASM, and inside
Spindle. Other languages (Go, TypeScript) follow ESS's realization targets later; the wire module
is the expensive one and is generated from the same sidecar.

**Reliability** comes from four independent checks, none of which trusts the generator:

1. parse/print round trips and grammar-derived positive and negative vectors (section 11);
2. replay of `ess-prototrace/1` traces recorded from the generated machine against the protospec
   model (`ess protocol replay`), and bounded `ess protocol explore` of the model;
3. fuzzing of every parser entry point with the limits at their maxima;
4. an independent external suite where one exists (Autobahn for WebSocket, section 12.6) and
   interoperability runs against an existing implementation.

### 10.2 Spindle

Spindle runs a compiled ESS model directly: handlers are trees of instructions, each message is
one step, and "the result is one record … appended durably; then the outbox is executed"
(`spindle:docs/design/spindle.md` §5). Its rule is that a specification never names "a database, a
protocol or a queue"; those live in an environment or a runtime implementation
(`spindle:AGENTS.md`, Rules).

Consequences for RFCspec:

| layer | runs where | why |
|---|---|---|
| wire codec | native, or WASM through Spindle's `call_pure(module, fn, args)` instruction ("WASM function, no imports, fuel-limited, deterministic") | a durable record per frame is not viable at line rate; the codec is pure and fits `call_pure` |
| protocol machine (WebSocket session, DNS transaction) | native now; on Spindle once it executes `ess-protospec/1` with a non-durable cell kind | protospec's step semantics (pure, explicit timers, ordered effects) match Spindle's step; durability per protocol step is optional and must be declared |
| application over the protocol | Spindle | the application's spec names an exchange (`session`, `request-response`), not "WebSocket"; the environment binds it to an RFCspec-generated port |

So the answer to "native or Spindle" is both, at different layers: RFCspec-generated crates become
Spindle **port and channel implementations** (`channel = websocket` in an environment document),
and Spindle applications stay protocol-free, as its rules require. Running a protospec machine
inside Spindle is a later Spindle milestone and a proposal to that repository, not a precondition.

## 11 Conformance

**Matrix.** One row per requirement of each RFC in the profile, verdicts as Ignite defines them
(`ignite:protocols.md` §3.3): pass, fail, gap (MUST-level unsupported), deviation (SHOULD-level
unsupported), inconclusive (satisfied without a passing test, or a test skipped), n/a or out of
scope. The claim "implementation X conforms to profile P" holds when no MUST-level row in scope is
fail, gap or inconclusive. Skips count as inconclusive, the rule Ignite takes from ESS's runners.

**Test sources.**

| source | generated from | catches |
|---|---|---|
| grammar vectors | each rule: minimal and boundary samples; single-byte mutations that leave the language | parser acceptance and rejection |
| layout vectors | each field at minimum, maximum, maximum + 1; each length encoding (WebSocket 7, 7+16, 7+64 bits) | length handling, non-minimal encodings |
| limit pairs | each limit at `limit` and `limit + 1` with the declared reaction | resource bounds |
| machine traces | `ess protocol explore` schedules replayed against the target through an adapter | ordering, timers, duplicate handling |
| requirement tests | per MUST: a scenario built from the requirement's linked elements (e.g. an unmasked client frame → the server fails the connection) | behaviour the grammar cannot see |
| external suite | Autobahn, h2spec-style suites, interop | everything above, from an independent author |

A target is driven through a small adapter: open a connection to a client or server under test,
send bytes, read bytes, observe close. The adapter never uses the generated code to produce
expected output, as ESS requires of target adapters (`ess:docs/design/protocol-specification.md`).

## 12 Worked example: WebSocket

### 12.1 Profile

| RFC | title (from the index) | status | format | role in the profile |
|---|---|---|---|---|
| 6455 | The WebSocket Protocol | Proposed Standard | TXT, HTML | base |
| 7936 | Clarifying Registry Procedures for the WebSocket Subprotocol Name Registry | Proposed Standard | TXT, HTML | K5, catalogue only |
| 8307 | Well-Known URIs for the WebSocket Protocol | Proposed Standard | TXT, HTML | K4 |
| 8441 | Bootstrapping WebSockets with HTTP/2 | Proposed Standard | TXT, HTML | K4, layering over an HTTP/2 stream |
| 9220 | Bootstrapping WebSockets with HTTP/3 | Proposed Standard | HTML, TXT, PDF, XML | K4, layering over an HTTP/3 stream |
| 7692 | Compression Extensions for WebSocket | Proposed Standard | TXT, HTML | optional K4 (permessage-deflate); DEFLATE itself is K3 |

`updated-by` of 6455 in the index: 7936, 8307, 8441. The first milestone takes 6455 alone, over
TCP, client and server roles, no extensions.

RFC 6455 has no RFCXML; it is the text-path test case. RFC 9220 is the XML-path test case within
the same profile.

### 12.2 What S2 extracts from RFC 6455

| item | where | notes |
|---|---|---|
| requirements | whole text | keyword counts in section 5; the masking pair is in §5.1: "a client MUST mask all frames that it sends to the server" and "A server MUST NOT mask any frames that it sends to the client" |
| handshake grammar | §4.3 "Collected ABNF for New Header Fields Used in Handshake" | octet ABNF; imports HTTP rules |
| frame grammar | §5.2 "Base Framing Protocol" | bit ABNF; page furniture inside it; requirements in comments |
| frame layout | §5.2 figure | packet diagram; checked against the bit ABNF |
| defined procedures | `_Fail the WebSocket Connection_` (§7.1.7), `_Start the WebSocket Closing Handshake_` (§7.1.2), `_The WebSocket Connection is Closed_` (§7.1.4), `_The WebSocket Connection is Established_` | each becomes a named transition group in the machine |
| connection states | `CONNECTING` (text line 777), `CLOSING` (line 2317), and the OPEN and CLOSED states implied by the defined procedures | S3 proposes the full state set; reviewed |
| registries | IANA `websocket.xml`: opcode, close-code-number, framing-header-bits, version-number, extension-name, subprotocol-name | enums; `close-code-number` and `opcode` open |
| limits | the RFC leaves maximum frame size, message size and closing timeout to the implementation | each a `Limit` with no RFC minimum |

### 12.3 Communication semantics

`unit = bytes → frames → messages`, `delivery = ordered reliable`, `direction = full-duplex`,
`multiplexing = single`, `exchange = session`, `correlation = none`, `lifetime = connection`,
`over = tcp` (8441: an HTTP/2 extended CONNECT stream; 9220: an HTTP/3 stream), roles `client`,
`server`. The opening handshake is a `request-response` exchange of the underlying HTTP profile;
after it, the same connection carries the WebSocket session. That switch is the layering gap G5.

### 12.4 Machine (illustrative lowering)

Illustrative, in the syntax of `ess:examples/protocols/lost-ack.yaml`; S3 produces the real one.
Frames are messages; their byte form is the sidecar's.

```yaml
format: ess-protospec/1
name: websocket-close
bounds: {max_steps: 64, max_states: 1024, max_time_ms: 30000}
participants:
  - name: client
    initial: open
    states: [open, closing, closed]
    inputs: [{name: close, fields: [{name: code, type: Integer}]}]
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
            exchange: {kind: literal, value: close}
            logical: {kind: literal, value: client-close}
            payload: {code: {kind: input, field: code}, masked: {kind: literal, value: true}}
          - {kind: arm, timer: close-timeout, after: {kind: fixed, millis: 5000}}
      - name: peer-close-echoed
        from: closing
        to: closed
        trigger: {kind: receive, channel: s2c, message: close-frame}
        effects:
          - {kind: cancel, timer: close-timeout}
          - {kind: emit, name: connection-closed}
      - name: close-timeout
        from: closing
        to: closed
        trigger: {kind: timer, name: close-timeout}
        effects:
          - {kind: emit, name: connection-failed}
      - name: late-close-after-timeout
        from: closed
        to: closed
        trigger: {kind: receive, channel: s2c, message: close-frame}
  - name: server
    initial: open
    states: [open, closed]
    transitions:
      - name: echo-close
        from: open
        to: closed
        trigger: {kind: receive, channel: c2s, message: close-frame}
        effects:
          - kind: send
            channel: s2c
            message: close-frame
            exchange: {kind: literal, value: close}
            logical: {kind: literal, value: server-close}
            payload: {code: {kind: literal, value: 1000}, masked: {kind: literal, value: false}}
          - {kind: flush, channel: s2c}
          - {kind: close}
messages:
  - {name: close-frame, fields: [{name: code, type: Integer}, {name: masked, type: Boolean}]}
channels:
  - {name: c2s, from: client, to: server, ordering: fifo, capacity: 8, loss: false, duplication: false}
  - {name: s2c, from: server, to: client, ordering: fifo, capacity: 8, loss: false, duplication: false}
properties:
  - {kind: flush_before_close, name: server-flushes-close, participant: server, channel: s2c}
  - {kind: event_count, name: closed-once, participant: client, event: connection-closed, max: 1}
```

Checked with `ess` 0.85.0 on 4 November 2026: `ess protocol explore --path websocket-close.yaml
--inputs close-inputs.json`, with one witness `{"kind":"input","participant":"client","name":"close",
"payload":{"code":1000}}`, returns `"verdict": "passed"`, 10 states, 11 transitions. A first draft
had `{kind: close}` in the client's `close-timeout` transition, as §7.1.7 has the client drop the TCP
connection; explore returned `inconclusive`, "dead end with outstanding queue, timer or flush
obligations": the server's close frame was still queued on `s2c` and a closed client can never
receive it. On TCP that frame is discarded with the connection. Protospec has no "closing a
participant discards what is in flight to it", so the model drops the client's `close` effect; this
is gap G9 (section 7). Exploration found it on the first run, which is the reason to lower into ESS
at all.

The `masked` field shows where the boundary is. "A server MUST NOT mask any frames" and "a server
receiving an unmasked frame from a client fails the connection" are both expressible today: an
`input` value source reads the fields of the received message as well as of a local input
(`ess:crates/specify/ess-compiler/src/protocol.rs`, the `Trigger::Receive` arm), so a
`fail-unmasked` transition guards on `{left: {kind: input, field: masked}, right: {kind: literal,
value: false}}`. "The close code is in 1000–4999 and not reserved" is not: it needs G1.

### 12.5 Generated crate

`rfcspec-websocket` (generated, not hand-written): `wire::Frame` parse/print with the three length
encodings and masking; `wire::handshake` request and response validation from §4.3's grammar;
`machine::{client, server}` from the protospec; `limits::Limits { max_frame, max_message,
close_timeout }`; `registry::{Opcode, CloseCode}`; `roles::{Client, Server}` over a `ByteStream`.
UTF-8 validation of text messages (§8.1) is a K3 library call, linked to its requirements.

### 12.6 Conformance

The independent suite is Autobahn|Testsuite [P15]: "over 500 test cases" for clients and servers,
grouped as framing, pings and pongs, reserved bits, opcodes, fragmentation, UTF-8 handling,
limits and performance, closing handshake, opening handshake (marked "under development") and
permessage-deflate. Its last commit on `master` is dated 10 October 2026; the Docker image is
"frozen intentionally on pypy:2-7-bullseye". RFCspec runs it in a container against the generated
server and client, maps each Autobahn case to the requirements it exercises (reviewed table), and
reports two numbers per run: Autobahn's own verdicts, and the requirement-matrix rows that only
RFCspec's generated tests cover and the rows only Autobahn covers. The second number measures the
generator's test coverage against an independent author.

**Exit criterion for the WebSocket milestone:** every MUST-level requirement of RFC 6455 in scope
for client and server has a status; the generated server passes the Autobahn categories that cover
framing, reserved bits, opcodes, fragmentation, UTF-8 and closing without failures; every MUST row
is pass, gap with an ESS gap id, or not applicable with a reason.

## 13 Repository

`beyond10x/rfcspec`, Rust, clap derive, `Taskfile.yml`, AEP store, ESS specification of its own
nouns in `ess/`.

| crate | does |
|---|---|
| `rfcspec-model` | `rfcspec/1` types, ids, spans, overlay, profile; canonical serialization |
| `rfcspec-source` | fetch, lock, index reading, errata and IANA registry readers; the only crate with network access |
| `rfcspec-doc` | RFCXML v3 and plain-text readers into one normalized document (page furniture removed, sections, figures, terms) |
| `rfcspec-extract` | requirements, ABNF collection, packet-diagram parsing, registry binding, defined terms |
| `rfcspec-abnf` | RFC 5234 and 7405 parser, bit-level mode, import resolution, determinism and boundedness analysis |
| `rfcspec-wire` | `rfcspec-wire/1` model, layout checks, agreement check between bit ABNF and diagrams |
| `rfcspec-propose` | S3: prompts, proposal schema, record and replay, span verification; uses `beyond10x/llm` |
| `rfcspec-ess` | S5: lowering to `ess-protospec/1`, `ess/N`, `ess-inputs.yaml`; gap reporting; depends on ESS crates at a released tag |
| `rfcspec-gen` | S6: Rust crate generation, WASM build of the codec |
| `rfcspec-conformance` | S7: vector generation, target adapter protocol, matrix, external-suite runners and case maps |
| `rfcspec-cli` | the `rfcspec` binary: `fetch`, `read`, `extract`, `propose`, `review`, `lower`, `generate`, `verify`, `matrix`, `check-sources`, `survey` |

Directory per profile: `profiles/websocket/{profile.yaml, overlay.yaml, rfcspec.lock, sources/,
proposals/, generated/}`.

## 14 Milestones

| # | deliverable | proves |
|---|---|---|
| M0 | `fetch`, `read`, `extract` for RFC 6455 (text) and RFC 9220 (XML): sections, requirements, ABNF, defined terms; `survey` over the whole index classifying every RFC by kind with counts | the corpus can be read, and the kinds of section 2.3 can be counted rather than guessed |
| M1 | `rfcspec-abnf` and `rfcspec-wire`: 6455 handshake grammar and frame bit ABNF + diagram, agreement check, generated codec with round-trip and mutation vectors | the wire layer |
| M2 | S3 and S4: proposals for parties, semantics and the close and open machines, recorded; overlay review; lowering to `ess-protospec/1` that `ess specify validate` and `ess protocol explore` accept | AI with a cited trail, and ESS as the target |
| M3 | generated `rfcspec-websocket` crate, Tokio roles, Autobahn run, requirement matrix | "generate clients", "generate conformance tests" |
| M4 | DNS (1035) over UDP: binary layouts, `by-id` correlation, `sessions` multiplexing; the ESS gap proposals G1, G3, G4 with WebSocket and DNS as evidence | a second, binary protocol; upstreaming the wire format (D3) |
| M5 | Spindle: the WebSocket codec as a `call_pure` module and a `channel = websocket` port in a Spindle environment | the Spindle path of section 10.2 |
| M6 | TCP (9293) as the hard case: modular arithmetic, windows, timers from measurements (G2, G7) | whether the approach scales to a transport |

## 15 Risks and open questions

| risk or question | default |
|---|---|
| ESS declines the gap proposals | RFCspec keeps the extended constructs in its sidecar and lowers what ESS accepts; the matrix shows the rest as gaps |
| AI-proposed machines are wrong in ways a reviewer misses | every transition cites a span; exploration and the external suite check behaviour; a transition without a requirement link is flagged |
| text-format RFCs vary in layout (pre-1990 documents) | the reader targets RFCs with a table of contents first; older ones get `source = prose` and review |
| the ABNF in an RFC is wrong or ambiguous | verified errata apply through the overlay; ambiguity is refused with both alternatives named |
| profile scope creep (HTTP/2 under WebSocket 8441) | layered profiles reference another profile's exchange as an abstract channel until that profile exists |
| where the requirement matrix lives long-term: RFCspec or ESS | RFCspec; ESS has no requirement noun and the ESS boundaries discourage generic additions |

## 16 Relation to Ignite

Ignite binds protocol code written in Ignite to RFC requirement catalogues (`std/rfc/rfcNNNN`) and
checks the binding (`ignite:protocols.md` §3). RFCspec produces the same catalogues, language
neutral, and the ESS documents and generated cores that Ignite's standard library could consume.
The catalogue fields in section 5 are a superset of Ignite's `Requirement` and `Spec` types, so
`rfcspec export ignite <profile>` is a later projection, not a separate extraction.

## 17 References

- [P1] RFC Editor index, `https://www.rfc-editor.org/rfc-index.xml` (downloaded 4 November 2026).
- [P2] RFC 2119 and RFC 8174 (BCP 14), the requirement keywords.
- [P3] RFC 7991, The "xml2rfc" Version 3 Vocabulary, `https://www.rfc-editor.org/rfc/rfc7991.html`.
- [P4] RFC 8174, Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words.
- [P5] RFC 5234, Augmented BNF for Syntax Specifications: ABNF.
- [P6] RFC 7405, Case-Sensitive String Support in ABNF.
- [P7] RFC 6455, The WebSocket Protocol, `https://www.rfc-editor.org/rfc/rfc6455.txt`.
- [P8] RFC 9220, Bootstrapping WebSockets with HTTP/3; RFC 8441, Bootstrapping WebSockets with HTTP/2.
- [P9] IANA WebSocket registries, `https://www.iana.org/assignments/websocket/websocket.xml`.
- [P10] EverParse 3D, as cited in `ignite:protocols.md` §9.
- [P11] Pacheco et al., Automated Attack Synthesis by Extracting Finite State Machines from Protocol
  Specification Documents (RFCNLP), `https://arxiv.org/abs/2202.09470`.
- [P12] Yen et al., SAGE, semi-automated protocol disambiguation and code generation; characterized
  here from the summaries in [P11] and [P13], not read directly.
- [P13] RFSeek and Ye Shall Find, `https://arxiv.org/pdf/2509.10216` (compares against PROSPER).
- [P14] An Agentic Flow for Finite State Machine Extraction using Prompt Chaining (FlowFSM),
  `https://arxiv.org/pdf/2507.11222`.
- [P15] Autobahn|Testsuite, `https://github.com/crossbario/autobahn-testsuite`.
- ESS protocol format: `ess:docs/design/protocol-specification.md`,
  `ess:crates/specify/ess-domain/src/protocol.rs`, `ess:examples/protocols/lost-ack.yaml`.
- Spindle: `spindle:docs/design/spindle.md` §5, §6, §11; `spindle:AGENTS.md`.
- Ignite: `ignite:protocols.md` (Ignite language repository, `docs/protocols.md`, 1 November 2026).
