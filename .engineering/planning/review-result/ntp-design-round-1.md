---
format: aep.planning-md/3
id: review-result:ntp-design-round-1
kind: review-result
status: active
title: NTP plan critic (design), round 1
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

What I read: 6 artifacts (`epic:ntp-profile` and its five stories), whole bodies, with `aep plan artifact show`. I also ran `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate`. Validate printed `valid`. I read the precedent `story:dns-header-codec` and `story:dns-query-machine` edges and spot-checked the repository: `crates/rfcspec-model/src/lib.rs:434` already has `Field.signed`, and the gap table in `docs/design/rfcspec.md` runs G1 to G10, so poll's G11 to G13 are new rows.

- **Edges:** I walked 10 declared edges among the set. Outside it, `epic:ntp-profile` depends on `epic:dns-profile`, which depends on `epic:websocket-profile`, and neither of those depends back on the set.
- **Cycles:** none. The order is `ntp-source-catalogue`, then `ntp-header-codec`, then `ntp-exchange-machine`, then `ntp-poll-machine`. `ntp-timer-model` has no outgoing dependency and feeds exchange and poll.
- **Chain:** not a defect. The header-to-catalogue edge exists because the codec needs the profile and overlay the catalogue creates. The exchange-to-header and poll-to-exchange edges each name the shared files beside them: `profiles/ntp/overlay.yaml`, `Taskfile.yml`, `crates/rfcspec-cli/tests` and the catalogue pages. Timer-model sits beside the others, so the set is not a queue.
- **Reads of produced state:** I traced 8 acceptance reads to a producer in the set.
  - Seven are recorded by an edge:
    - header reads the profile, lock and overlay from catalogue;
    - exchange reads `loss` from timer-model;
    - exchange reads the `ntp` Taskfile task from header;
    - exchange reads `profile.yaml` and `overlay.yaml` from catalogue, through header;
    - poll reads `backoff` and `loss` from timer-model;
    - poll reads the exchange overlay and Taskfile state from exchange.
  - The eighth, poll's read of the `ntp` Taskfile task that header creates, is not a direct edge but is covered by the poll-to-exchange-to-header path.
- **Split abstraction:** none. The codec story adds signed fields and the ntp layout together, so the generator change can be shown working through `cargo test -p rfc5905-ntp`. The timer-model story adds `backoff` and `loss` with synthetic overlays and exploration tests, so it can also be shown without a profile.

What I could not establish:
- Whether `approval-record:ntp-profile`, which the epic cites, exists. It is not in the graph. That is scope or acceptance territory, out of my lane, and did not set my verdict.
- Whether files such as `crates/rfcspec-cli/tests`, `Taskfile.yml` and `overlay.yaml` collide between concurrent items is parallel-safety's lane, and I did not judge it.

```findings
[]
```
