---
format: aep.planning-md/3
id: story:ntp-source-catalogue
kind: story
status: active
title: 'Lock, read and extract RFC 5905 into profiles/ntp: 110 pages, 37 numbered sections, 13 BCP 14 requirements'
relations:
- decomposes: epic:ntp-profile
scope:
- confidence: cited
  path: crates/rfcspec-cli/tests
- confidence: cited
  path: crates/rfcspec-doc
- confidence: cited
  path: crates/rfcspec-extract/src/lib.rs
- confidence: cited
  path: profiles/ntp/overlay.yaml
- confidence: cited
  path: profiles/ntp/profile.yaml
- confidence: cited
  path: profiles/ntp/rfcspec.lock
- confidence: cited
  path: profiles/ntp/sources/rfc5905.txt
- confidence: cited
  path: website/docs/catalog/index.md
- confidence: cited
  path: website/docs/catalog/ntp.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-05T13:50:22Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-05T13:50:49Z", actor: "human:timo", revision: 4}
---
## Story

`rfcspec fetch --rfc 5905` locks RFC 5905 into a new profile `profiles/ntp`, and the reader and
extractor are shown to handle it: a 110-page BCP 14 RFC (unlike RFC 1035) whose scoped sections
carry almost no capitals keyword, so most of what the machines implement is prose the reviewer
writes as `source: prose` requirements in the later stories. The profile enters the protocol
catalogue with an overlay that reviews nothing yet.

## Outcome

`profiles/ntp` exists with a locked RFC 5905, a profile scoped to sections 7.2, 7.3, 8, 9.2 and
13, an overlay holding a status for each of the three in-scope requirements (R9.2-1 to R9.2-3, all unsupported: authentication and broadcast mode are not modelled; `rfcspec check` requires a status for every in-scope requirement), and a catalogue page; reader and extractor values for RFC 5905 are
pinned by tests.

## Acceptance

`rfcspec extract --profile profiles/ntp --rfc 5905` exits 0 and prints a catalogue whose 13 BCP 14
requirements carry section ids, and `rfcspec check --all profiles` exits 0 on `dns`, `ntp`, `time`
and `websocket`.

Values below were read from `https://www.rfc-editor.org/rfc/rfc5905.txt` on 2026-11-05 (6163
lines, 241096 bytes, SHA-256 `8b7abd903c60202e2953ee012fefa6916666c3b0c71825c748cdd262024fb268`)
with the reader and extractor of `origin/main` at the time of drafting. If the fetched file has
another digest, the lock records what was fetched and every value here is re-read before the story
is accepted.

- `rfcspec fetch --rfc 5905 --profile profiles/ntp` writes `sources/rfc5905.txt` and a lock entry
  `{rfc: 5905, file: sources/rfc5905.txt, sha256: <hex>}`; a second run changes no byte.
- Furniture: `rfcspec read` reports 329 furniture lines: the 110 `Mills, et al. ... [Page N]`
  footers (pages 1 to 110), the 110 form-feed lines and the 109 header lines `RFC 5905 NTPv4
  Specification June 2010`. No block contains `[Page` or starts with `RFC 5905 `. A test asserts
  the three counts. The RFC 868, RFC 6455 and RFC 1035 reader tests pass unchanged.
- Sections: blocks carry the ids of the 37 numbered headings from `1. Introduction` (line 175) to
  `18.2. Informative References` (line 3295); `7.2. Global Parameters` is line 889, `7.3. Packet
  Header Variables` line 938, `8. On-Wire Protocol` line 1431, `9.2. Peer Process Operations`
  line 1804, `13. Poll Process` line 2829. Figure captions (`Figure 8: Packet Header Format`, line
  1061) start no section. A test asserts the 37 ids and these five lines.
- The Figure 8 diagram (lines 1015-1059) is one figure in section 7.3; the Figure 6 table (lines
  903-915, PORT 123, MINPOLL 4, MAXPOLL 17) is one figure in section 7.2.
- BCP 14: the catalogue has 13 requirements, `R2-1` to `R2-3`, `R7.4-1` to `R7.4-5`, `R9.2-1`
  to `R9.2-3`, `R11.2.3-1` and `R11.2.3-2`. In scope are `R9.2-1` (MAY, lines 1889-1890),
  `R9.2-2` (MAY, 1924-1925) and `R9.2-3` (SHOULD, 1926-1927). A test asserts the 13 ids.
- Prose candidates in the scoped sections: `P-14` (`may`, 932), `P-15` (`should`, 1265),
  `P-16` (`should`, 1283), `P-17` (`should`, 1440), `P-18` (`may`, 1451), `P-19` (`must`, 1613),
  `P-20` (`should`, 1639), `P-24` (`must`, 1816), `P-25` (`must`, 1866), `P-29` (`must`, 2958).
  A test asserts these ten ids and start lines.
- `profiles/ntp/profile.yaml`: name `ntp`, RFC 5905 "Network Time Protocol Version 4: Protocol
  and Algorithms Specification" (line 17), sections `["7.2", "7.3", "8", "9.2", "13"]`, no
  semantics yet. `profiles/ntp/overlay.yaml`: `rfcspec-overlay/1`, rfc 5905, `kind: K1`, no
  requirements, layouts, machines or findings. Requirements outside the scoped sections count as
  unreviewed, as for `websocket`.
- `rfcspec check --all profiles` exits 0. On `time`, `websocket` and `dns`, `check`, `lower
  --check` and `generate --check` pass with no change under their directories.
- `rfcspec-docs generate` lists `ntp` (RFC 5905, K1, stage S4, three reviewed requirements, all unsupported) in
  `website/docs/catalog/index.md` and writes `website/docs/catalog/ntp.md`;
  `rfcspec-docs generate --check` exits 0.
- The RFC 5905 catalogue is byte-identical across two runs, and `task check` exits 0.

## Scope

`profiles/ntp/profile.yaml`, `profiles/ntp/overlay.yaml`, `profiles/ntp/rfcspec.lock`,
`profiles/ntp/sources/rfc5905.txt`, tests in `crates/rfcspec-doc` and `crates/rfcspec-extract`
(a reader or extractor fix there only if a value above fails), `crates/rfcspec-cli/tests`,
`website/docs/catalog/index.md` and `website/docs/catalog/ntp.md` (regenerated).

Choices taken:

- Section 7.2 is in scope only for Figure 6 (PORT, MINPOLL, MAXPOLL), which the timer values and
  the layer port cite. Section 9.1 (peer variables) is not in scope; 9.2 and 13 name what the
  machines need.
- The appendix (`A.` headings from line 3396) is not in scope. Its blocks carry the id of the last
  numbered section before them; giving appendix headings their own ids is left for a later slice.
- The RFC text names its authors, their institutions and reference-clock sources (Figure 12); it is
  committed verbatim, as `rfc1035.txt` is. If the Gates scan refuses a literal in it, stop and
  report rather than edit the source.

First of five stories; the others depend on it through `profiles/ntp/overlay.yaml`,
`profiles/ntp/profile.yaml` and the generated catalogue pages.
