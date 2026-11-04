---
format: aep.planning-md/3
id: review-result:rfcspec-spec-hardening-1
kind: review-result
status: active
title: 'hardening, round 1: design review, spec diff, guard analysis'
relations:
- reviews: executable-system-specification:rfcspec-spec
revision: 1
---
I ran three hardening techniques. Design review (8) found real problems in the spec. Guard analysis (6) found three gaps that matter: a re-import or replay hits a creating command with no declared answer for an id that already exists, `MarkDecisionStale` has no answer for an unknown key, and a few commands accept inputs they should check. The spec diff (7) found no spec defects, only two places where `ess` under-rates a narrowing. Nothing in `/home/timo/beyond10x/rfcspec` changed, and `git status` is the same as at the start. Scratch files are in `/home/timo/.cache/scratch/53dbf269-1cf2-493b-81e3-12cdded0d72f/harden/`.

## 8 Design review (one sub-agent, the skill's brief, run on a planted copy)
- **Planted defect:** in `review-copy/ess/domains/profile.yaml` I deleted `Status.reason`, its invariant, its guard conjunct, the input, the `sets:` entry and the view field. I put blank lines in their place so line numbers stay the same, and the copy still validated. The agent was not told what was planted. It reported it as row 2, `missing`, quoting design `rfcspec.md:205` "`unsupported <reason>`, `not-applicable <reason>`" against `profile.yaml:278-280`. So the review catches a deleted rule.
- **Green result:** the real spec has `reason`, so I dropped row 2. The other 24 rows apply to the real spec at the same lines.
- **Findings:**
  - **18 `missing`.** The firmest:
    - Only Requirement can go Stale; Section, DefinedTerm, Grammar, Layout and Limit cannot.
    - No command ever sets `Rfc.kind` or `Grammar.dialect`.
    - A Status or Machine id is not tied to its profile and requirement fields, so one profile and requirement can get two statuses.
    - Accept, Reject, SetRequirementLevel and RecordStatus name no Decision.
    - A Stale requirement can be accepted without `ConfirmDecision`.
  - **3 `contradicts`:**
    - The Decision key `(item, text_digest)` has no profile, but overlays are per profile.
    - The Proposer (S3) creates and moves Requirements before any decision.
    - The Reviewer (S4) is granted `RecordMachine`, which records an S5 artifact.
  - **3 `unclear`**, questions for the design's owner:
    - Which stage records `satisfied`?
    - How do accepted semantics reach an authored, immutable profile?
    - Where is a limit's reaction declared?
  - **2 rows I count as false positives:**
    - Per-transition spans are dropped on purpose under the Machine decision of 2026-11-04.
    - The views `Rfcs` and `Sections` exist so the suite can observe things.
- The spec contradicts no 2026-11-04 decision. Two rows (5, 13) say a decision is applied unevenly: the id spelling is not tied to the fields, and `SetRequirementLevel` has no guard.

## 7 Spec diff
- **Commands:** `ess verify diff --from <scratch>/old/ess --to ess --compatibility --format json`, then the same with `--fail-on breaking-or-unknown`.
- **Planted defect:** removing the `ProposalTask.ImportMapping` variant in `diffplant/` failed the gate with exit 4: `type/rfcspec.review.ProposalTask/variant-removed/ImportMapping`, breaking for callers and history.
- **Green result:** comparing the current spec with itself passes with exit 0.
- **Real diff from b50992d:** 279 changes, of which 69 are compatible, 17 breaking (13 are required inputs added) and 193 unknown. The gate fails with exit 4 on 210 unacknowledged changes. All of them are the intended 2026-11-04 remodel. There is no tag, so b50992d was never released and nobody consumes it. Acknowledgements are only needed if b50992d is ever treated as a baseline.
- **Two `ess-limit` notes:**
  - The new `party-not-a-role` refusal on `DeclareProfile` is rated `unknown`, not `breaking` as an added refusal.
  - `Rfc.date` changing from `String` to `YearMonth` is a narrowing but is rated `unknown`.

## 6 Guard analysis
- **Command:** a throwaway probe, `guards.py`, over `ess specify compile --path ess --format json`. It enumerates each command's input guards over enum variants, Optional present or absent, small party and role lists, and related-row states, 334 combinations in all. It reports guards that are dead, overlapping, leave a gap, or let an entity invariant break after `sets:`. It also flags a missing `unknown_instance` or `existing_instance` outcome.
- **Planted defects:** on `guardplant/`, which still validated, I planted two:
  - `RecordStatus` guard `kind == Satisfied` became `kind != Unsupported`. The probe reported an invariant break: NotApplicable with no reason passes.
  - `DecideProposal` `edited` became `verdict == Accept`. The probe reported the overlap and the dead `edited` branch.
- **Green result:** both planted results are gone. What remains:
  - `MarkDecisionStale` has no `unknown_instance` outcome, so an absent key has no declared answer. The old version had one.
  - 15 creating commands declare no `existing_instance` outcome, although design §3 says re-import keeps ids.
  - `ConfirmDecision`: `incomplete` overlaps `missing` and `not-stale`. Declaration order settles it and all three refuse, so it is a false positive.
- **Weak guards found by hand**, beyond what the probe enumerates; none breaks a declared invariant:
  - `ConfirmDecision` does not check `key.item == confirms.item` or that the digest changed.
  - `DecideProposal` does not tie the verdict to the decision's existence or kind.
  - `ExtractRequirement` (S2) accepts `source: Prose`.
  - `SetRequirementLevel` has no guard on source or state.
  - `Limit` has no `minimum <= maximum` invariant.

Not run: 1 mutation, 2 random sequences, 3 caller replay, 4 determinism and 5 metamorphic relations, because there is no implementation yet.

```findings
- file: ess/domains/review.yaml
  line: 450
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: MarkDecisionStale declares no unknown_instance outcome, so marking an absent decision key has no declared answer
- file: ess/domains/catalogue.yaml
  line: 643
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: none of the 15 creating commands (ReadRfc, ReadSection, Extract*, ProposeRequirement, ReadRegistry, DeclareProfile, RecordMachine, RecordStatus, RecordProposal, RecordDecision, ConfirmDecision) declares an existing_instance outcome, though re-import keeps ids and replay re-records proposals
- file: ess/domains/review.yaml
  line: 515
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: ConfirmDecision does not require key.item to equal confirms.item or the text digest to differ, so a confirmation can create a decision for another item
- file: ess/domains/review.yaml
  line: 344
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: DecideProposal does not check that the decision key exists, matches the proposal's item, or has a kind matching the verdict
- file: ess/domains/catalogue.yaml
  line: 721
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: ExtractRequirement (S2, mechanical) accepts source Prose, bypassing the S3 proposal and review path the design gives prose requirements
- file: ess/domains/catalogue.yaml
  line: 853
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: SetRequirementLevel has no guard on source Prose or review state, so a mechanically extracted BCP 14 level can be overwritten
- file: ess/domains/review.yaml
  line: 168
  category: spec-gap
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the Decision key (item, text_digest) carries no profile, so two profiles' overlays cannot each hold a decision on the same item and digest
- file: ess/domains/catalogue.yaml
  line: 396
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: only Requirement can become Stale; Section, DefinedTerm, Grammar, Layout and Limit carry spans but have no stale state or re-import command (design section 3)
- file: ess/domains/profile.yaml
  line: 268
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: Status and Machine ids are not tied to their profile and requirement fields, so two statuses can exist for one (profile, requirement) pair
- file: ess/domains/catalogue.yaml
  line: 815
  category: spec-gap
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: AcceptRequirement, RejectRequirement, SetRequirementLevel and RecordStatus name no Decision, so the rule that changes reach the catalogue only through an overlay entry is not enforced
- file: ess/domains/catalogue.yaml
  line: 463
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: a Stale requirement is accepted directly, without ConfirmDecision, though the design applies its decisions only after a reviewer confirms them
- file: ess/domains/review.yaml
  line: 233
  category: spec-gap
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: the Proposer (S3) may create and move Requirements, contradicting "a proposal changes nothing until S4 records a decision"
- file: ess/domains/catalogue.yaml
  line: 321
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: no command sets Rfc.kind, so classification (design 2.3 and 9) has no declared path
- file: ess/domains/catalogue.yaml
  line: 509
  category: spec-gap
  severity: warning
  verdict: CONFIRMED
  origin: undecided
  message: no command sets Grammar.dialect, though design 8.3 has it proposed in S3 and reviewed
- file: ess/domains/review.yaml
  line: 250
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: the Reviewer (S4) is granted RecordMachine for a lowered protospec document, which design section 4 assigns to S5
- file: ess/domains/review.yaml
  line: 91
  category: spec-gap
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: nothing records or guards that an erratum is verified, though the summary and design D7 say only verified errata apply
- file: ess/domains/profile.yaml
  line: 263
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: Status has nowhere to attach the test evidence the design says the conformance matrix reads
- file: ess/domains/profile.yaml
  line: 266
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: the rule that every MUST-level requirement in scope needs a status is stated as out of scope and declared nowhere
- file: ess/domains/profile.yaml
  line: 389
  category: spec-gap
  severity: note
  verdict: CONFIRMED
  origin: undecided
  message: RecordMachine accepts transitions with empty requirement lists and holds no unlinked-MUST list, which design 9 and 15 require to be flagged
- file: ess/domains/profile.yaml
  line: 218
  category: spec-gap
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: semantics may name the same layer twice, though the design declares one value per axis for each layer
- file: ess/domains/catalogue.yaml
  line: 570
  category: spec-gap
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: Limit declares no invariant that minimum does not exceed maximum
- file: ess/domains/profile.yaml
  line: 306
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: unclear in the design (rfcspec.md:181 vs :314) which stage records a satisfied status; both Lowering and Reviewer may record any kind
- file: ess/domains/profile.yaml
  line: 231
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: unclear in the design how semantics accepted in S3/S4 reach a profile that is authored and has no update command
- file: ess/domains/catalogue.yaml
  line: 559
  category: spec-gap
  severity: note
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: unclear in the design (rfcspec.md:445) where a limit's declared reaction lives; Limit holds none
- file: ess/domains/profile.yaml
  line: 180
  category: false-positive
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: TransitionLink holds no per-transition span, which the 2026-11-04 Machine decision intends
- file: ess/domains/catalogue.yaml
  line: 1157
  category: false-positive
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: views Rfcs and Sections have no design statement but exist for conformance observability
- file: ess/domains/review.yaml
  line: 490
  category: false-positive
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: ConfirmDecision incomplete overlaps missing and not-stale, but declaration order resolves it and all three refuse with nothing written
- file: ess/domains/profile.yaml
  line: 364
  category: ess-limit
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: ess verify diff rates the added party-not-a-role refusal on DeclareProfile unknown rather than breaking (refusal-added) because the earlier branch was an unguarded creates
- file: ess/domains/catalogue.yaml
  line: 315
  category: ess-limit
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: ess verify diff rates Rfc.date String to YearMonth (alphabet and length narrowing) unknown, whereas an alphabet added to Digest is rated breaking
```
