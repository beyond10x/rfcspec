# Wave 6 of the [plan of 2026-11-05](plan-2026-11-05.md)

Stage: 2, approved 2026-11-06 by session sys-mon on the operator's word (approval-record:wave-6-approval), after the plan-critic panel; all four units merged into `wave-6` and implemented.

## Units

| story | objective | scope | branch | worktree | build dir | stage |
|---|---|---|---|---|---|---|
| story:abnf-core | `rfcspec-abnf` parses RFC 5234 and RFC 7405 ABNF, resolves references and core rules, and matches input; `bits` and `rfc2616` dispatch stubs | inferred | wave6-abnf-core | trees/b10x/rfcspec/wave6-abnf-core | that tree's `target/` | implemented |
| story:catalogue-grammars | S2 collects each section's ABNF figures into a catalogue `Grammar`, from text and RFCXML | cited and inferred | wave6-catalogue-grammars | trees/b10x/rfcspec/wave6-catalogue-grammars | that tree's `target/` | implemented |
| story:fixed-arm-bounds-check | `rfcspec check` refuses a fixed timer `arm` of 0 ms or above `max_time_ms` | cited | wave6-fixed-arm | trees/b10x/rfcspec/wave6-fixed-arm | that tree's `target/` | implemented |
| story:explorer-look | the RFC explorer takes the RFCspec documentation site's colours, font, header and footer | cited | wave6-explorer-look | trees/b10x/rfcspec/wave6-explorer-look | that tree's `explorer/node_modules` and `target/` | implemented |

Integration branch: `wave-6`, from main `bab7197` (wave 5 merged).

## Check

- Path: `aep plan artifact waves --kind story --status proposed` (aep 0.79.0) on `bab7197`:

```
wave 1
  story:abnf-core (inferred)
  story:catalogue-grammars (inferred)
  story:explorer-look
  story:fixed-arm-bounds-check
1 wave(s), 0 collision(s), 0 unassessed
```

- Dependencies: every `depends_on` target (wire-scaffold, text-reader-structure, send-payload-type-check) is implemented and on main.
- Scope on main: the files cited exist; abnf-core's six new files sit in `crates/rfcspec-abnf/`, which wire-scaffold created with `src/lib.rs`; catalogue-grammars fills the `crates/rfcspec-wire/src/grammars.rs` stub and adds one scenario file.
- Implementable here: no credential, no network. explorer-look needs `pnpm` and the built site (`task website`).
- Blast radius: abnf-core is inside `rfcspec-abnf`. catalogue-grammars adds a serde-defaulted field to `Catalogue` and fills it in `extract_reporting`; requirement extraction and `EXTRACTOR_VERSION` do not change, so the bulk files stay as they are. fixed-arm-bounds-check touches `check.rs` only. explorer-look touches `explorer/src/` only.
- Each story moves to `implemented` on a `test_result` record and a `review_outcome` from its adversary pass.

## Pre-flight, 2026-11-06

- Primary checkout: clean, on `main` at `bab7197`.
- Disk: 70G free on `/`; the floor is 25G.
- Agents: 3 at once, the operator's limit. Implementor `aep:implementor`, adversary `aep:adversary`. The fourth unit starts when the first finishes.
- Wave 5's unit trees are finished.

## Commits approval authorises

- 4 unit commits, 4 merges into `wave-6`, the opening commit holding these pages, and 1 closing store commit, all by `b10x-bot[bot]`.
- 1 push of `wave-6`, 1 PR, and its merge into `main` after the gate is green.
- No tag and no release.
