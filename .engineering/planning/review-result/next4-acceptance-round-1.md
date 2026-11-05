---
format: aep.planning-md/3
id: review-result:next4-acceptance-round-1
kind: review-result
status: active
title: acceptance critic, round 1, four follow-up epics
relations:
- reviews: epic:bulk-catalogue
- reviews: epic:ai-proposals
- reviews: epic:websocket-frames
- reviews: epic:dns-profile
- reviews: story:bulk-summary-format
- reviews: story:bulk-fetch-corpus
- reviews: story:bulk-site-index
- reviews: story:proposal-record-replay
- reviews: story:proposal-review
- reviews: story:websocket-proposal-slice
- reviews: story:proposal-machine-layout
- reviews: story:websocket-machine-guards
- reviews: story:websocket-ping-machine
- reviews: story:websocket-fragments-machine
- reviews: story:dns-source-catalogue
- reviews: story:dns-header-codec
- reviews: story:dns-query-machine
revision: 1
---
needs-revision

- story:bulk-fetch-corpus — the acceptance says "editing one row's counts makes `task bulk` exit 1", but `bulk check` recomputes counts only for RFCs locked in a profile, so editing the counts of any other row passes — .engineering/planning/story/bulk-fetch-corpus.md:67 (check rules at .engineering/planning/story/bulk-summary-format.md:66-69)
- story:proposal-record-replay — the story promises prompts for two tasks, `status` and `requirement-from-prose`, but every checked behaviour is `status` only (checks, five failure answers, the three-proposal replay test, the `--task status` command). `requirement-from-prose` has no observable outcome, so the story can be closed with half of it unverified — .engineering/planning/story/proposal-record-replay.md:62
- story:dns-query-machine — the "Overlay model" bullet (participant fields, `set`, guards, `local` operands) is the same outcome that story:websocket-machine-guards delivers. Nothing orders the two stories, so whichever lands second finds the bullet already true and the acceptance reads the same before and after its work — .engineering/planning/story/dns-query-machine.md:69
- story:websocket-ping-machine — "reviewed stays 57", "32 unsupported, 4 not applicable" and the pinned `catalog.rs` test hold only if story:websocket-proposal-slice has not landed. The slice adds overlay entries, and the page's Reviewed figure is `overlay.requirements.len()` (`crates/rfcspec-docs/src/catalog.rs:534`). No relation orders the two stories — .engineering/planning/story/websocket-ping-machine.md:86
- story:websocket-fragments-machine — the absolute figures "70 requirements … 33 unsupported, 8 not applicable" and "126 unreviewed (82 MUST, …)" are false if story:websocket-proposal-slice lands first. The slice lowers the unreviewed count, adds Reviewed entries, and nothing orders them — .engineering/planning/story/websocket-fragments-machine.md:105
- story:websocket-proposal-slice — "keeps its sections (5.2, 5.5, 7)" is a literal list that story:websocket-fragments-machine changes to 5.2, 5.4, 5.5, 5.6, 7. The acceptance is false if the fragments story lands first, and nothing orders them — .engineering/planning/story/websocket-proposal-slice.md:64

**What I read:** all 16 ids (4 epics, 12 stories) with `aep plan artifact show`, plus `aep plan artifact kinds` and `aep plan artifact lifecycle story`. In the tree and the RFC texts I checked these against what the acceptances cite:
- The websocket counts: 17/36/4, 139 unreviewed (87 MUST, 17 MUST NOT, 7 SHOULD, 1 SHOULD NOT, 26 MAY, 1 OPTIONAL), and the 21 → 29 and 57 → 70 arithmetic.
- The 13 requirements of §5.4 and §5.6, the §5.5.2 and §5.5.3 lines, and the 11 keywords in §5.1 and §5.3.
- The RFC 1035 lines, counts and digest, and the RFC 6455 and RFC 868 lock digests.
- The ESS constructs (`NeverState`, `StateImplication`, `Equality`, `LocalField`, `Set`) and the "dead end" and "property violated" messages.
- The `ProposalTask` variants, `ItemId` spellings, `rfcspec-gen` vector code, `passive_check`, and the 11 BCP 14 levels.

**What I could not establish:**
- No `rfcspec` or `ess` binary was available, so I did not run `extract` or `explore`; the explore verdicts and mutant outcomes are unchecked.
- The index-dependent figures (9,847 rows, the 7 RFCs without a TXT format, RFC 6455 `updated_by` 7936/8307/8441, 11 range pages) need the live `rfc-index.xml`, which I did not fetch.
- Out of my lane:
  - The dns-query-machine finding above is mainly a design/scope overlap with websocket-machine-guards.
  - The ordering gaps behind the ping, fragments and slice findings are a shared-surface problem: the overlay, `Taskfile.yml`, `catalog.rs` and the generated pages. Parallel-safety should see them.

```findings
- file: .engineering/planning/story/bulk-fetch-corpus.md
  line: 67
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance says editing one row's counts makes `task bulk` exit 1, but `bulk check` recomputes counts only for RFCs locked in a profile, so a counts edit on any other row passes"
- file: .engineering/planning/story/proposal-record-replay.md
  line: 62
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the story promises prompts for `status` and `requirement-from-prose`, but every checked behaviour in the acceptance is `status` only, so the second task has no observable outcome"
- file: .engineering/planning/story/dns-query-machine.md
  line: 69
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Overlay model bullet (fields, set, guards, local operands) is the outcome of story:websocket-machine-guards and nothing orders the two, so it reads the same before and after this story's work if that story lands first"
- file: .engineering/planning/story/websocket-ping-machine.md
  line: 86
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the pinned 'reviewed stays 57', '32 unsupported, 4 not applicable' figures hold only if story:websocket-proposal-slice has not landed, because Reviewed is the overlay requirement count and no relation orders the two stories"
- file: .engineering/planning/story/websocket-fragments-machine.md
  line: 105
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the absolute figures '70 requirements ... 33 unsupported, 8 not applicable' and '126 unreviewed (82 MUST, ...)' are false if story:websocket-proposal-slice lands first and nothing orders the two stories"
- file: .engineering/planning/story/websocket-proposal-slice.md
  line: 64
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "'keeps its sections (5.2, 5.5, 7)' is a literal list that story:websocket-fragments-machine changes to 5.2, 5.4, 5.5, 5.6, 7, so the acceptance is false if that story lands first and nothing orders them"
```
