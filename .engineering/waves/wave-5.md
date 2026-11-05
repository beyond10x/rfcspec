# Wave 5 of the [plan of 2026-11-05](plan-2026-11-05.md)

Stage: 2, approved by the operator on 2026-11-05 (approval-record:wave-5-approval); all three units merged into `wave-5` and implemented.

## Units

| story | objective | scope | branch | worktree | build dir | stage |
|---|---|---|---|---|---|---|
| story:wire-scaffold | the `rfcspec-abnf` and `rfcspec-wire` crates, workspace manifests, `Dialect` enum, model types and collect stubs; nothing under `crates/rfcspec-extract/src/` | inferred | wave5-wire-scaffold | trees/b10x/rfcspec/wave5-wire-scaffold | that tree's `target/` | implemented |
| story:send-payload-type-check | `rfcspec check` refuses a `send` payload typed differently from its message field | cited | wave5-send-payload | trees/b10x/rfcspec/wave5-send-payload | that tree's `target/` | implemented |
| story:text-reader-structure | the text reader recognises appendix headings, lists, asides and table-of-contents lines; `rfcspec bulk compare`; `EXTRACTOR_VERSION` 2; bulk files regenerated offline | inferred | wave5-text-reader | trees/b10x/rfcspec/wave5-text-reader | that tree's `target/` | implemented |

Integration branch: `wave-5`, from `a8d86ab` (main `96d7c2d` plus the planning commit).

## Check

- Path: `aep plan artifact waves --kind story --status proposed` (aep 0.79.0); its lines are on the plan page. The three stories share no file.
- Implementable here: no credential, no network. text-reader-structure regenerates the bulk files offline from the local corpus cache; its index matches the committed `891d504b`.
- Blast radius: wire-scaffold adds two crates and edits `rfcspec-model`. send-payload-type-check touches `check.rs` only. text-reader-structure touches `rfcspec-doc`, `rfcspec-extract/src/lib.rs`, `rfcspec-cli` bulk, and the generated bulk and website data.
- Each story moves to `implemented` on a `test_result` record and a `review_outcome` from its adversary pass.

## Pre-flight, 2026-11-05

- Primary checkout: clean, on `main` at `96d7c2d`.
- Disk: 80G free on `/`; the floor is 25G.
- Builds: the primary `target/` holds 689M, and the 25 older linked trees hold 4K of `target/` each. One unit build measured about 760M (explorer-profile-layer, wave 4).
- Agents: 3 at once, the operator's limit. Implementor `aep:implementor`, adversary `aep:adversary`.
- Older trees: 26 linked worktrees of earlier waves are still registered. `worktree gc` is assessing them; none is used by this wave.

## Known cost before the PR

text-reader-structure rewrites `bulk/rfc-sections.jsonl` and `website/static/data/rfcs/09900.json`. Their gates exceptions bind the file content, so new exceptions from b10x-atlas are needed before the PR can pass `common / Security and privacy`.

## Commits approval authorises

- 3 unit commits, 3 merges into `wave-5`, the opening commit holding these pages, and 1 closing store commit, all by `b10x-bot[bot]`.
- 1 push of `wave-5`, 1 PR, and its merge into `main` after the gate is green.
- No tag and no release.
