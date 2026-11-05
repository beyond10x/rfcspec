---
format: aep.planning-md/3
id: review-result:ntp-acceptance-round-1
kind: review-result
status: active
title: NTP plan critic (acceptance), round 1
relations:
- reviews: epic:ntp-profile
- reviews: story:ntp-source-catalogue
- reviews: story:ntp-header-codec
- reviews: story:ntp-timer-model
- reviews: story:ntp-exchange-machine
- reviews: story:ntp-poll-machine
revision: 1
---
approve

Findings: none. Every acceptance is checkable by a command or a file, and each moves something from not-there to there.

What I read: 6 of 6 ids (`epic:ntp-profile`, `story:ntp-source-catalogue`, `story:ntp-header-codec`, `story:ntp-timer-model`, `story:ntp-exchange-machine`, `story:ntp-poll-machine`), whole bodies via `aep plan artifact show`. I also read `aep plan artifact lifecycle story`, the `epic:dns-profile` and `story:dns-query-machine` bodies as precedent, and `AGENTS.md`, `docs/design/rfcspec.md` (gap table G1–G10), `Taskfile.yml` and `crates/rfcspec-docs/src/catalog.rs` in the worktree. I checked the source-catalogue facts against `profiles/ntp/sources/rfc5905.txt` and `rfcspec.lock`. The following all match:
- the SHA-256, 6163 lines and 241096 bytes;
- 110 `[Page` lines, 110 form-feed lines and 109 `RFC 5905` header lines;
- 37 numbered headings, and the five section start lines;
- the Figure 6 lines 906, 909 and 910, and Figure 8 at line 1061;
- the quoted requirement sentences and their line ranges, for `R-VN-1`, `R8-*`, `R9.2-*` and `R13-*`.

What I could not establish:
- I did not run `rfcspec` (not installed), so I did not check the BCP 14 count of 13, the 329 furniture total as reader output, the prose ids `P-14` to `P-29`, or the mutant verdicts.
- Only the exchange story records a probe for its mutants: the `xmt`-reset mutant is recorded as naming `one-valid-packet-per-poll`; the no-guard mutant is recorded only as `failed`. The poll story records no mutant probe. These are claims the implementation will confirm, not unobservable acceptances.
- Out of my lane (design or parallel-safety): the header-codec bullet "their constrained fields admit 0, or the generated tests are byte-identical" is false for the websocket ranges `[126, 65535]` and `[65536, …]` at `profiles/websocket/overlay.yaml:513,520`. If the new "smallest admitted value" base reaches presence-gated fields, websocket vectors could drift. The `generate --check` acceptance would catch it. Also, header-codec, exchange-machine and poll-machine all edit `profiles/ntp/overlay.yaml`, `Taskfile.yml` and the catalogue pages, and the dependency chain serialises them.

```findings
[]
```
