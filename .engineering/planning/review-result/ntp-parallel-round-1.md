---
format: aep.planning-md/3
id: review-result:ntp-parallel-round-1
kind: review-result
status: active
title: NTP plan critic (parallel), round 1
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

Nothing I found would change the drafted set, so there are no findings.

**What you read.** I read 6 artifacts (the epic and the five stories) with `aep plan artifact show` for each, plus `aep plan artifact graph` and `aep plan artifact waves`. I checked the cited paths against the worktree: `crates/rfcspec-gen/src/fields.rs` (the "must be unsigned" refusal), `crates/rfcspec-model/src/lib.rs` (`Field.signed`, `Arm`), the crate and `crates/rfcspec-cli/tests` listings, and `website/docs`.

**Surfaces.**
- 5 of 5 stories have surfaces established as cited; none are inferred or unplaceable.
- `aep plan artifact waves` prints a collision for every in-set pair on a shared path. All such pairs are ordered by `depends_on`, directly or through the chain.
  - Catalogue and header-codec are linked by a direct edge.
  - Header-codec and exchange are linked by a direct edge.
  - Exchange and poll are linked by a direct edge.
  - Timer-model has a direct edge to both exchange and poll.
  - The remaining pairs are ordered through that chain: catalogue with exchange and poll, and header-codec with poll.
- The timer-model is the only unordered pair partner. It sits on `crates/rfcspec-ess`, `crates/rfcspec-extract/src/check.rs` and `crates/rfcspec-model/src/lib.rs`, which none of catalogue and header-codec list. Its "independent, can run beside them" claim holds. The catalogue edits `crates/rfcspec-extract/src/lib.rs`, a different file in that crate.
- Each shared-path pair that is ordered says so in its body, with the shared paths named. For example, exchange and poll both state "both edit `profiles/ntp/overlay.yaml`, `Taskfile.yml`, `crates/rfcspec-cli/tests`".
- **Readers beside producers.** I traced 5 acceptance reads to a producer in the set, and a `depends_on` edge records all 5.
  - Header-codec reads the catalogue's lock and profile.
  - Exchange reads header-codec's `ntp` task and overlay.
  - Exchange reads the timer-model's `loss`.
  - Poll reads exchange's overlay and mutant tests.
  - Poll reads the timer-model's backoff.

**What I could not establish.**
- `profiles/ntp/rfcspec.lock` and `sources/rfc5905.txt` already exist untracked in the worktree. I did not check them against the catalogue story's digest, which is not my lane.
- Collisions with stories outside the set (for example the `websocket-*` and `dns-*` stories that `waves` prints against these paths) are outside what I was asked to judge.
- The poll story's scope names the whole `website/docs/` tree although its body names only the design page and the catalogue pages. It collides with nothing inside the set, so it does not affect the verdict.
- Whether the acceptances are checkable and whether the five-way split is right belong to the acceptance and design critics.

```findings
[]
```
