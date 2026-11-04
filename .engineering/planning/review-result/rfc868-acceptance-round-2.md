---
format: aep.planning-md/3
id: review-result:rfc868-acceptance-round-2
kind: review-result
status: active
title: acceptance critic, round 2, RFC 868 thin slice
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

epic:rfc868-thin-slice — the acceptance says "a correct in-process server passes every TCP row", but the verify story makes the "unable to determine the time" TCP row `inconclusive` on a healthy server, so the literal check can never hold — .engineering/planning/epic/rfc868-thin-slice.md:52 (contradicts .engineering/planning/story/rfc868-verify-probe.md:23-24)
story:rfc868-verify-probe — "The probe never writes to the target" names no observation, because none of the in-process servers is said to record the octets it receives, so only the author can confirm it — .engineering/planning/story/rfc868-verify-probe.md:30
story:rfc868-source-reader — the reader "removes both page footers and the page-2 header" (two lines, `RFC 868 May 1983` and `Time Protocol`), but the check only asserts that no paragraph starts with `RFC 868`, so a reader that leaves `Time Protocol` behind passes — .engineering/planning/story/rfc868-source-reader.md:23

What I read: 6 of 6 ids (the epic and the five stories), through `aep plan artifact show <id>`. I also read round 1's acceptance result (rfc868-acceptance-round-1.md), the RFC text, and the `event_count` examples in `~/beyond10x/ess/examples/protocols/`, and ran `ess protocol explore --help`.

Round 1's four findings did not recur:
- The epic now has an Acceptance section.
- The overlay finding is now owned by the extract story.
- The `event_count` property and its mutant test now exist.
- The verify matrix now has a row rule.

I checked in the RFC text:
- The counts are 11 steps (6 TCP, 5 UDP) and 5 examples, and the `should` quotes match.
- The four 32-bit examples reconcile with Unix time.

What I could not establish:
- `rfcspec fetch` is the only networked command and tests run offline, so its first bullet is not checked by `task check`. A live run does check it, so I did not file it.
- The epic's `task check` list names no workspace tests for the reader and extract stories. That is a coverage question for `plan-critic-scope`, out of my lane, and it did not set my verdict.
- Each story's acceptance is a bullet list rather than one sentence. I treated the bullets as separately checkable, as in round 1.
- No ESS conformance scenarios are named for the acceptances. That is out of my lane, as in round 1.

```findings
- file: .engineering/planning/epic/rfc868-thin-slice.md
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance says a correct in-process server \"passes every TCP row\", but the verify story makes the \"unable to determine the time\" TCP row inconclusive on a healthy server, so the literal check can never hold"
- file: .engineering/planning/story/rfc868-verify-probe.md
  line: 30
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "\"The probe never writes to the target\" names no observation, because none of the in-process servers is said to record the octets it receives, so only the author can confirm it"
- file: .engineering/planning/story/rfc868-source-reader.md
  line: 23
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the reader \"removes both page footers and the page-2 header\" (two lines, `RFC 868 May 1983` and `Time Protocol`), but the check only asserts that no paragraph starts with `RFC 868`, so a reader that leaves `Time Protocol` behind passes"
```
