# AGENTS.md — rfcspec

The contract for changing this repository. Humans start at [`README.md`](README.md); the design is
[`docs/design/rfcspec.md`](docs/design/rfcspec.md).

## Serves

- **RFCs as executable specifications.** An RFC becomes a cited requirement catalogue, ESS
  documents for its machines and types, a wire-format sidecar for its bytes and bits, and from
  those a generated protocol core and a conformance suite.

## State

Design stage. Nothing of the pipeline (design §4, stages S0–S7) is implemented. What exists:

| path | what |
|---|---|
| `docs/design/rfcspec.md` | the design: decisions (§0), pipeline, catalogue, communication semantics, ESS gaps, WebSocket example, milestones (§14) |
| `ess/` | RFCspec's own ESS specification: the catalogue and review nouns |
| `examples/websocket/` | the WebSocket closing-handshake model (`ess-protospec/1`) and its exploration witness |
| `crates/rfcspec-docs/` | generates the site's derived pages (design, status) and binds a built site to its commit |
| `website/` | the Docusaurus site published at `https://beyond10x.github.io/rfcspec/`; `website/static/pitch/` is a standalone slide deck |
| `.engineering/` | the AEP planning store (`aep.project/5`) |

## Gate

```console
task check
```

Steps (`Taskfile.yml`): `aep plan artifact validate`, `ess specify validate --path ess`, bounded
exploration of every model under `examples/` must report `"verdict": "passed"`,
`rfcspec-docs generate --check`, `cargo fmt --check`, strict Clippy, `cargo test --workspace`.
CI (`.github/workflows/check.yml`) pins `ess` 0.86.0 and `aep` 0.79.0. The site build is separate:
`task website` (`npm ci --allow-git=root` locally; CI's Node 22 needs no flag).

## Rules

- Running code is Rust, clap derive for the command line. No Python, no shell helpers.
- Only the source crate (design §13, `rfcspec-source`) will reach the network. Every later stage
  reads locked sources and runs offline; no gate calls a model or a website.
- AI output is a recorded proposal with cited spans. It changes nothing until a review decision is
  written to a profile's overlay.
- ESS is read and written through its crates at a released tag; never re-implement ESS parsing or
  validation.
- Generated files are never edited by hand: change the generator in `crates/rfcspec-docs/`, then
  `task docs-generate`.
- A new noun gets its ESS declaration in `ess/` before any AEP story is written around it.
- No comments in `Cargo.toml`, `Taskfile.yml`, workflow files or YAML specifications.
- Confidentiality: this repository is public. No name of an external company, customer, product or
  host, and no `/home/<name>/` path, in any file or commit message.
- Builds: `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0`, the tree's own `target/`;
  check `df -h /` first.
- Commits and pushes are `b10x-bot[bot]` through `b10x-gates bot`; use managed worktrees for
  changes and keep the primary checkout clean.
