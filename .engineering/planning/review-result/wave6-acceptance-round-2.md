---
format: aep.planning-md/3
id: review-result:wave6-acceptance-round-2
kind: review-result
status: active
title: 'Wave 6 plan critic, acceptance, round 2: one warning'
relations:
- reviews: story:abnf-core
- reviews: story:catalogue-grammars
- reviews: story:fixed-arm-bounds-check
- reviews: story:explorer-look
revision: 1
---
needs-revision

- story:catalogue-grammars — the synthetic-document line gives the section as "`2.1` (anchor `section-2.1`)" without saying the `<section>` carries `pn="section-2.1"`, but the XML reader takes the section number only from `pn` (a section with `anchor` and no `pn` has no number), so the expected id `rfc<N>/grammar/2.1` cannot come out of an anchor-only document — `.engineering/planning/story/catalogue-grammars.md:53`, `crates/rfcspec-doc/src/xml.rs:429-435`

Round 1 findings:
- **abnf-core, no result stated** — resolved. The synthetic grammars now state results (`.engineering/planning/story/abnf-core.md`, last acceptance bullet).
  - I checked each against RFC 5234 and RFC 7405.
  - `%s"Ab"` accepts `Ab` and refuses `ab` at 0. `%i"Ab"` and the plain form accept both.
  - `r = "a"` followed by `r =/ "b"` accepts `a` and `b` and refuses `c` at 0.
  - `%x30-39` accepts `5` and refuses `a` at 0. `%d13.10` accepts CR LF (2 bytes) and refuses CR SP at 1.
  - There are exactly 16 core rules in RFC 5234 Appendix B, matching the story's list.
  - No stated result is wrong for ABNF.
  - The first four bullets match the source: lines 741-747, 1701-1703 and 1826-1827 read as claimed, and `j = i MOD 4` ends in a bare repeat `4`, so it is a parse error.
- **catalogue-grammars, test name matches nothing** — resolved.
  - `--test grammars` fails before the work: `crates/rfcspec-wire/tests/grammars.rs` does not exist.
  - The expected grammar is now exact (id, span, rules, `uses`).
  - `stubs.rs` is in scope, and the update is stated.
  - The first test finding left one gap, the `pn` finding above.
- **catalogue-grammars, validate reads the same before and after** — resolved. The scenario file, the `ess-inputs.yaml` listing and the outcomes are named. The file does not exist today and `ess-inputs.yaml` lists six scenarios. On a scratch copy `ess specify validate` printed a scenario count, so the listing does change the output.
- **fixed-arm-bounds-check, drifting lines** — resolved.
  - The quoted strings are exact. `let head = format!("{item}: arm {} with backoff", ...)` is at `check.rs:286`, and the `CLOSE_START` constant at 1360 has the stated value.
  - The `close` machine's `max_time_ms: 30000` is at `overlay.yaml:1310`, and the first transition's fixed arm of `close-timeout` is `millis: 5000`.
  - `overlay.yaml:1310` is the one line number still cited. It is right at `bab7197`, and no wave 6 unit scopes `overlay.yaml`, so it holds for this wave.

story:explorer-look is unchanged and I did not widen it.

I read 4 of 4 stories with `aep plan artifact show`, plus the round 1 result, `wave-6.md`, and the tree files named above. I ran `ess specify validate` and `ess verify conform` against a scratch copy of `ess/` (deleted afterwards).

What I could not establish:
- **Scenario outcomes are checked by reading only.**
  - `ess specify validate` accepted a scratch scenario whose expected outcome I had made wrong (`recorded` instead of `already-exists`), so exit 0 does not test "expects `extracted`, then `already-exists`".
  - I did not find a command that runs it: `ess verify conform run --target interpreted` refused without `--report-format 2`. I raise no finding because I cannot name the fix.
- **Three lines pass before the work, as guards.** They are `millis: 30000` gives no problem, `rfcspec check --all profiles` exits 0 and the `bulk check`. Each sits beside a line that fails before. I did not count them as findings.
- **Core-rule inputs are not pinned.** abnf-core leaves the inputs for the 16 core rules to the test author. The RFC fixes the results, so a wrong pair would fail. For OCTET, the refused input has to be empty or longer than one byte.
- **Not run:** `task explorer`, `task website`, and the `rfcspec-wire` tests.
- Nothing outside my lane.

```findings
- file: .engineering/planning/story/catalogue-grammars.md
  line: 53
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the synthetic RFCXML section is given as '2.1 (anchor section-2.1)' without naming pn=\"section-2.1\", but the reader reads the section number only from pn (crates/rfcspec-doc/src/xml.rs:429-435), so an anchor-only section has no number and the expected id rfc<N>/grammar/2.1 cannot result"
```
