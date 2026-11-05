---
format: aep.planning-md/3
id: review-result:frames-adversary-1
kind: review-result
status: active
title: Adversary, frames epic
relations:
- reviews: story:websocket-machine-guards
- reviews: story:websocket-ping-machine
- reviews: story:websocket-fragments-machine
revision: 2
---
unit: story:websocket-machine-guards, story:websocket-ping-machine, story:websocket-fragments-machine. Covers the uncommitted working tree on HEAD 0e4e26e, `.engineering/` excluded.
verdict: CONFIRMED (note only; no defect against the acceptance statements)
cases: executed 141→141, red 0
origin: introduced 1 / pre-existing 0 / undecided 1
wrote-outside-worktree: 1 path (scratch directory, listed in section 6)
needs-coordinator: none

**1. `git --no-pager diff --stat HEAD -- . ':!.engineering'`**
```
 13 files changed, 1275 insertions(+), 81 deletions(-)
```
This is the implementor's diff, not mine. I added no files and changed none in the worktree.

**2. Cases added:** none. Nothing I found is a defect against the acceptance statements, so there was no red case to write. Two things I could not prove failing are in section 4.

**3. Suite run** (after the probes; no cases of mine exist):
- `cargo test -q --locked -p rfcspec-ess -p rfcspec-extract -p rfcspec-model -p rfcspec-docs -p rfcspec-cli` (line-tables-only, incremental off): every result line `ok`, 141 passed, EXIT=0.
- `rfcspec check --profile profiles/websocket` → `overlay holds (70 requirements, 1 layouts, 3 machines, ...)`, EXIT=0.
- `rfcspec lower --profile profiles/websocket --check` → `profiles/websocket/generated is current`; the same for `profiles/time`.
- `ess protocol explore` on close, ping and fragments, each with its witness → `"verdict": "passed"` for all three.
- `rfcspec-docs generate --check` → `website: 15 generated files current`.
- The downloads for `ping` and `fragments` are byte-identical to `generated/`.
- No `/home/` path appears in the untracked generated files, witnesses or downloads.

**4. Judgement findings**
- **Mutant the check tests miss** — `crates/rfcspec-extract/src/check.rs:171`, verdict CONFIRMED, origin introduced.
  - What was measured: I copied the repository to scratch and changed this line to `flat_map(|guard| [&guard.left])`. `cargo test -p rfcspec-extract --lib` still gave `26 passed; 0 failed`.
  - Why: every refusal test built by `guarded()` puts the bad operand on the left, and the right side is always the declared `token`. Nothing checks a bad right-hand operand.
  - What reaches it: any overlay guard that reads an undeclared field on the right.
  - Suggested fix (not applied): one test whose right-hand operand is `Operand::Local("absent")` or `Operand::Input("reason")`.
- **`rfcspec check` lets type errors through** — `crates/rfcspec-extract/src/check.rs:168-206`, verdict INFEASIBLE, origin undecided.
  - What was measured: in a scratch copy of the profile, three bad overlays all pass `rfcspec check`, and `ess protocol explore` refuses each one:
    - a guard comparing Boolean `fin` with `1`: `[GuardTypeMismatch]`
    - `set` of the Integer `ping-data` from `true`: `[ValueTypeMismatch]`
    - a duplicate local field: `[DuplicateIdentity]`
  - What reaches it: `task check` runs explore on every model, so CI still catches these. The cost is only that the refusal comes later and names the protospec path instead of the overlay. The acceptance lists none of these refusals.

**5. Attacked and could not break**
- The committed `close`, `ping` and `fragments` models and both profiles' `generated/` match a fresh lowering byte for byte.
- In the catalogue extracted from the locked source, the ids and spans for R5.4-1..12, R5.6-1, R5.5.2-1..4 and R5.5.3-1..2 match the overlay exactly.
- I read the statuses for 5.4, 5.5.2 and 5.5.3 against the RFC text (lines 1894-1943, 2040-2079) and found them consistent.
- The fragments mutant test can fail, and it fails through the property. I also tried, in scratch, a client that sends a second opcode-1 frame in the middle of a message (the R5.4-4 case). Explore returns `failed` with `property violated: {"server-never-fails"}`.
- The ping mutant test depends on the guard. With the guard removed or the operands swapped, the mutant would pass explore and the test would go red. The witness value (7) differs from the initial value (0), and the witness file is pinned.
- The lowering unit tests pin literal JSON for `fields`, `set`, `guards` and `never_state`. None of them would pass if the functions returned defaults.
- The `never_state` refusal is checked per participant, so a state belonging to another participant does not satisfy it.
- The docs figures are 70 reviewed, 29/33/8 by status and 126 unreviewed. G10 and the G3 rows are present, and the overlay and pages contain none of the stale reason strings.
- Side observation, not a defect: the fragments server's idle `R5.5.3-2-pong` transition is never reached, because the client sends a Pong only inside a message.

**6. Paths written outside the worktree**
- `~/.cache/scratch/adversary-frames-wave/` holds `cat.json`, the profile copy `ws/`, and the repository copy `mut/`, whose `target/` I already deleted. The whole directory can be removed.

**7.**
```findings
- file: crates/rfcspec-extract/src/check.rs
  line: 171
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: Changing the guard check to read only the left operand keeps all 26 rfcspec-extract tests green, because every refusal test puts the bad operand on the left.
- file: crates/rfcspec-extract/src/check.rs
  line: 168
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: undecided
  message: rfcspec check accepts a guard comparing Boolean with Integer, a set with the wrong type and a duplicate local field; ess explore refuses each later, so task check still fails, but the error names the protospec instead of the overlay.
```
