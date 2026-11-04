---
format: aep.planning-md/3
id: review-result:rfc868-scope-round-2
kind: review-result
status: active
title: scope critic, round 2, RFC 868 thin slice
relations:
- reviews: epic:rfc868-thin-slice
- reviews: story:rfc868-source-reader
- reviews: story:rfc868-extract-overlay
- reviews: story:rfc868-lower-protospec
- reviews: story:rfc868-generate-codec
- reviews: story:rfc868-verify-probe
revision: 2
---
needs-revision

```
epic:rfc868-thin-slice — "task check exits 0 and runs ... rfcspec check, rfcspec lower --check and ess protocol explore on both models ... rfcspec generate --check" is promised and no story claims putting `rfcspec check`, `rfcspec lower --check` or `rfcspec generate --check` into Taskfile.yml's `check` task: lower-protospec (the only story with Taskfile.yml in scope) claims only that "task check runs both" explore runs, and extract-overlay and generate-codec list no Taskfile.yml in scope; lower-protospec would most naturally take it — .engineering/planning/epic/rfc868-thin-slice.md:52
```

What I read: 6 artifacts (epic and 5 stories) plus review-result:rfc868-scope-round-1, via `aep plan artifact show` on each and `aep plan artifact graph`. I also read `.engineering/drafts/rfc868.txt`, `Taskfile.yml` and `ls ess/domains`.

Promises: I extracted 11 from the epic. I traced 10 to an item or a stated exclusion. The one I could not trace is the `task check` wiring above. The `ess/domains` nouns (catalogue, profile, review) exist, so that statement holds.

Round 1 comparison: I flagged the missing `profiles/time/profile.yaml`. source-reader's scope now lists it, so that is fixed.

Not established or outside my lane:
- `cargo test -p rfc868-time` and the `rfcspec-conformance` tests are covered once the new crates are workspace members, because `task check` already runs `cargo test --workspace`.
- I did not judge whether acceptance lines are checkable (acceptance lane) or whether the stories run in the right order (parallel-safety lane).
- Out of lane (design): the epic says "four worked examples", while extract-overlay and the RFC list five numeric examples. The fifth is the one the epic calls a review finding.

```findings
- file: .engineering/planning/epic/rfc868-thin-slice.md
  line: 52
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "\"task check exits 0 and runs ... rfcspec check, rfcspec lower --check and ess protocol explore on both models ... rfcspec generate --check\" is promised and no story claims wiring `rfcspec check`, `rfcspec lower --check` or `rfcspec generate --check` into Taskfile.yml's check task: lower-protospec, the only story with Taskfile.yml in scope, claims only the explore runs, and extract-overlay and generate-codec list no Taskfile.yml; lower-protospec would most naturally take it"
```
