---
format: aep.planning-md/3
id: review-result:ntp-scope-round-2
kind: review-result
status: active
title: NTP plan critic (scope), round 2
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

**What I read.** I read 6 artifacts in full: `epic:ntp-profile` first, then the five stories. I ran `aep plan artifact show` on each, `aep plan artifact graph`, and `aep plan artifact show review-result:ntp-scope-round-1`. I also read the gap table at `docs/design/rfcspec.md:308-317`.

**Promises.** I extracted 12 promises from the epic and traced all 12 to an item.
- The four outcome bullets go to `story:ntp-header-codec`, `story:ntp-exchange-machine` (two bullets) and `story:ntp-poll-machine`.
- The ESS-gap recording paragraph goes to `story:ntp-timer-model` and `story:ntp-poll-machine`.
- Seven acceptance clauses (the shared `task check` clause counts as one):

| Acceptance clause | Claimed by |
|---|---|
| `profiles/ntp` locked, catalogue index lists `ntp` | `story:ntp-source-catalogue` |
| Codec tested with version range and signed Poll and Precision | `story:ntp-header-codec` |
| Exchange and poll pass `explore` with witnesses | `story:ntp-exchange-machine`, `story:ntp-poll-machine` |
| Mutants: no origin guard and no `xmt` reset | `story:ntp-exchange-machine` |
| Mutant: fixed interval instead of backoff | `story:ntp-poll-machine` |
| Semantics row `udp`, port 123 | `story:ntp-exchange-machine` |
| Design gap table names the gaps | `story:ntp-poll-machine` |

**Round 1 finding.** It is fixed. The gap-table bullet in `story:ntp-poll-machine` (revision 3) now reads "G1 adds 'NTP origin and duplicate check (RFC 5905 section 8, R8-DUP-1)'". That matches the G1 row, which lists only WebSocket at `docs/design/rfcspec.md:308`. G2 and G7 already name NTP at lines 309 and 314, and the bullet extends them.

**Other checks.**
- No item reaches into the epic's out-of-scope list.
- No two items claim one outcome.
- No promise is silently narrowed. The reach register and unreach counter are abstracted, and R9.2-REACH-2, R13-UNREACH-1 and R13-RESET-1 are recorded as `unsupported` with named gaps.

**What I could not establish.**
- Unease, not a finding: `story:ntp-exchange-machine` mentions G4 and G3 in its "Choices taken". The design's G4 row lists only HTTP/2, QUIC and DNS. The epic names G4 as out of scope rather than as a gap the slice hit, so I did not count it as a promise.
- Out of my lane (design): `loss: true` in `story:ntp-timer-model` has no sentence in the epic. It is enabling work for the UDP exchange.
- Out of my lane (design): the missing `approval-record:ntp-profile` is expected, as you stated.

```findings
[]
```
