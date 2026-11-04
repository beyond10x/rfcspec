---
format: aep.planning-md/3
id: review-result:rfc868-acceptance-r1
kind: review-result
status: active
title: acceptance critic, round 1, RFC 868 thin slice
relations:
- reviews: epic:rfc868-thin-slice
- reviews: story:rfc868-source-reader
- reviews: story:rfc868-extract-overlay
- reviews: story:rfc868-lower-protospec
- reviews: story:rfc868-generate-codec
- reviews: story:rfc868-verify-probe
revision: 1
---
needs-revision

epic:rfc868-thin-slice — the body has no Acceptance section (headings are Outcome, Stages covered, Not in this wave, Specification), so nothing states the observable condition under which the epic is done, such as a named command run that passes — .engineering/planning/epic/rfc868-thin-slice.md:10

story:rfc868-verify-probe — the story promises "one row per requirement of the profile" but the acceptance defines only three observable rows plus the "unable to determine the time" row, leaving the UDP requirements and the other overlay requirements with no verdict, so the matrix can never be shown complete — .engineering/planning/story/rfc868-verify-probe.md:14

story:rfc868-generate-codec — the acceptance requires the fifth example "recorded in the overlay as a review finding", yet no story's acceptance creates that record and this story's scope omits `profiles/time/overlay.yaml`, so the before and after state has no named cause — .engineering/planning/story/rfc868-generate-codec.md:25

story:rfc868-lower-protospec — "The TCP model has the server send exactly one `time` message then close…" names no command or assertion that shows it, because `"verdict": "passed"` on the committed witnesses does not distinguish a model that sends two messages from one that sends one — .engineering/planning/story/rfc868-lower-protospec.md:23

**What I read:** 6 of 6 ids (epic:rfc868-thin-slice and the five stories), through `aep plan artifact show <id>`, plus `aep plan artifact kinds` and `lifecycle epic|story`. I checked the acceptances against `.engineering/drafts/rfc868.txt` and design §11. In the RFC text:
- the counts match: 11 dialogue steps (6 TCP, 5 UDP), 5 numeric examples, and two lowercase `should` sentences;
- the four 32-bit example dates reconcile with the Unix epoch;
- `ess protocol explore --path --inputs` exists.

**What I could not establish:**
- `rfcspec fetch` is the only networked command and "Tests run offline", so the first source-reader bullet has no stated way to be checked in CI. It can still be checked by a live run, so I did not file it.
- "Quote occurs verbatim at the span" does not say whether the RFC's line wraps are normalized. The one-character-change test works either way, so I did not file it.
- Each story's acceptance is a bullet list of several outcomes, not one sentence. I treated the bullets as separately checkable and filed nothing. The repository rule (the workspace CLAUDE.md) asks for "named conformance scenarios", and none are named. That falls under ESS coverage, not acceptance, so it is out of my lane.
- Out of my lane: the missing overlay-finding ownership is also a scope and coverage question for `plan-critic-scope`. It did not set my verdict, but the generate-codec finding above is filed as acceptance.

```findings
- file: .engineering/planning/epic/rfc868-thin-slice.md
  line: 10
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the body has no Acceptance section (headings are Outcome, Stages covered, Not in this wave, Specification), so nothing states the observable condition under which the epic is done"
- file: .engineering/planning/story/rfc868-verify-probe.md
  line: 14
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the story promises one row per requirement of the profile but the acceptance defines only three observable rows plus the unable-to-determine row, leaving the UDP requirements and the other overlay requirements with no verdict, so the matrix can never be shown complete"
- file: .engineering/planning/story/rfc868-generate-codec.md
  line: 25
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance requires the fifth example recorded in the overlay as a review finding, yet no story's acceptance creates that record and this story's scope omits profiles/time/overlay.yaml, so the before and after state has no named cause"
- file: .engineering/planning/story/rfc868-lower-protospec.md
  line: 23
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the bullet 'The TCP model has the server send exactly one time message then close' names no command or assertion that shows it, because a passed verdict on the committed witnesses does not distinguish one message from two"
```
