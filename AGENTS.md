# AGENTS.md — rfcspec

The contract for changing this repository. Humans start at [`README.md`](README.md); the design is
[`docs/design/rfcspec.md`](docs/design/rfcspec.md).

## Serves

- **O2 — decisions as data, with evidence.** An RFC becomes a cited requirement catalogue, ESS
  documents for its machines and types, and a wire-format sidecar for its bytes and bits; a
  conformance matrix gives every requirement a status backed by tests.
- **O4 — products run on the foundation.** The generated protocol cores become native libraries
  and Spindle ports, so a product's specification names an exchange, not a protocol.

## State

The pipeline runs every stage except S3 (AI proposals) end to end for two profiles,
`profiles/time` (RFC 868) and `profiles/websocket` (RFC 6455 sections 5.2, 5.5 and 7); everything
else is design. What exists:

| path | what |
|---|---|
| `docs/design/rfcspec.md` | the design: decisions (§0), pipeline, catalogue, communication semantics, ESS gaps, WebSocket example, milestones (§14) |
| `ess/` | RFCspec's own ESS specification: the catalogue, profile (semantics, statuses, recorded runs) and review nouns |
| `examples/websocket/` | the WebSocket closing-handshake model (`ess-protospec/1`) and its exploration witness |
| `crates/rfcspec-{model,source,doc,extract,ess,gen,conformance}/` | the pipeline stages S0–S2 and S4–S7 (design §13) for plain-text RFCs |
| `crates/rfcspec-cli/` | the `rfcspec` binary: `fetch`, `read`, `extract`, `check`, `lower`, `generate`, `verify` |
| `profiles/time/` | RFC 868: locked source, `profile.yaml` with the semantics of the `tcp` and `udp` layers, reviewed `overlay.yaml`, input witnesses, and `generated/` (protospec models and the `rfc868-time` codec crate); a profile may hold recorded runs (`runs/*.json`, `rfcspec-run/1`, written by `rfcspec verify --report`) |
| `profiles/websocket/` | RFC 6455: locked source, `profile.yaml` (one `tcp` layer, sections 5.2, 5.5 and 7 in scope), `overlay.yaml` giving every in-scope requirement a status (the rest of the catalogue counts as unreviewed), the `close` machine and its input witness, the `frame-header` layout, and `generated/` (`close.protospec.yaml` and the `rfc6455-websocket` frame codec crate) |
| `crates/rfcspec-docs/` | generates the site's derived pages (design, status, and the protocol catalogue `website/docs/catalog/` with its downloads `website/static/downloads/`, both from `profiles/`) and binds a built site to its commit |
| `website/` | the Docusaurus site published at `https://beyond10x.github.io/rfcspec/`; `website/static/pitch/` is a standalone slide deck |
| `.engineering/` | the AEP planning store (`aep.project/5`) |

## Gate

```console
task check
```

Steps (`Taskfile.yml`): `aep plan artifact validate`, `ess specify validate --path ess`, bounded
exploration of every model under `examples/` must report `"verdict": "passed"`,
the `time` task (`rfcspec check --all profiles`, which also fails on a stale recorded run or an
in-scope requirement without a status, `rfcspec lower --check`, exploration of both
Time Protocol models, `rfcspec generate --check`, `cargo test -p rfc868-time`),
the `websocket` task (`rfcspec lower --profile profiles/websocket --check`, exploration of the
`close` model with its witness, `rfcspec generate --profile profiles/websocket --check`,
`cargo test -p rfc6455-websocket`),
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
  `task docs-generate`. The same holds for `profiles/*/generated/`: change the overlay or
  the generator, then `rfcspec lower` and `rfcspec generate`.
- A new noun gets its ESS declaration in `ess/` before any AEP story is written around it.
- No comments in `Cargo.toml`, `Taskfile.yml`, workflow files or YAML specifications.
- Confidentiality: this repository is public. No name of an external company, customer, product or
  host, and no `/home/<name>/` path, in any file or commit message.
- Builds: `CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0`, the tree's own `target/`;
  check `df -h /` first.
- Commits and pushes are `b10x-bot[bot]` through `b10x-gates bot`; use managed worktrees for
  changes and keep the primary checkout clean.
