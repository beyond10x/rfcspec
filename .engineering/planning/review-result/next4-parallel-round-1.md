---
format: aep.planning-md/3
id: review-result:next4-parallel-round-1
kind: review-result
status: active
title: parallel critic, round 1, four follow-up epics
relations:
- reviews: epic:bulk-catalogue
- reviews: epic:ai-proposals
- reviews: epic:websocket-frames
- reviews: epic:dns-profile
- reviews: story:bulk-summary-format
- reviews: story:bulk-fetch-corpus
- reviews: story:bulk-site-index
- reviews: story:proposal-record-replay
- reviews: story:proposal-review
- reviews: story:websocket-proposal-slice
- reviews: story:proposal-machine-layout
- reviews: story:websocket-machine-guards
- reviews: story:websocket-ping-machine
- reviews: story:websocket-fragments-machine
- reviews: story:dns-source-catalogue
- reviews: story:dns-header-codec
- reviews: story:dns-query-machine
revision: 1
---
needs-revision

No `depends_on` edge connects any of the four epics. Each epic depends only on `epic:websocket-profile` (`aep plan artifact graph`), and no story in one epic points at a story in another. The planned order is websocket-frames, dns-profile, ai-proposals, bulk-catalogue. `aep plan artifact waves` does not reproduce it: it puts `story:proposal-record-replay` in wave 1, `story:dns-source-catalogue` and `story:websocket-machine-guards` together in wave 2, and the bulk stories last.

Cross-epic file collisions, from `waves`:

| Epic pair | Shared paths |
|---|---|
| frames and dns | `website/docs/catalog/index.md`, `Taskfile.yml`, `AGENTS.md`, `crates/rfcspec-extract/src/check.rs`, `crates/rfcspec-model/src/lib.rs` |
| frames and ai | `profiles/websocket/overlay.yaml`, `website/docs/catalog/websocket.md`, `crates/rfcspec-docs/src/catalog.rs`, `crates/rfcspec-cli/tests/websocket.rs`, `check.rs`, `model/src/lib.rs`, `Taskfile.yml`, `AGENTS.md` |
| dns and ai | `crates/rfcspec-cli/tests`, `Cargo.toml`, `Cargo.lock`, `Taskfile.yml`, `AGENTS.md`, `check.rs`, `model/src/lib.rs` |
| bulk and each of the others | `Cargo.lock`, `crates/rfcspec-cli/src/main.rs`, `Taskfile.yml`, `AGENTS.md`, `website/docs/catalog/index.md` |

Only the first story of each later epic needs an edge to the last story of the epic before it, because the in-epic chains carry the rest. `story:bulk-summary-format` needs edges to both final ai stories, since `story:proposal-machine-layout` and `story:websocket-proposal-slice` are not ordered against each other.

Findings (all surfaces cited, all from the stories' own scope lists):

- story:dns-source-catalogue — it shares `website/docs/catalog/index.md` with `story:websocket-ping-machine` and `story:websocket-fragments-machine`, and nothing orders it after the frames epic. The planned order is "frames first", but `waves` puts it in wave 2 beside `story:websocket-machine-guards`. Remedy: a `depends_on` edge to `story:websocket-fragments-machine` that records the shared catalogue pages as its reason. The alternative is to split the index row out. — `.engineering/planning/story/dns-source-catalogue.md:26`, `.engineering/planning/story/websocket-fragments-machine.md:31`
- story:dns-query-machine — it re-adds what `story:websocket-machine-guards` adds. The same crates are touched: `crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-extract/src/check.rs` and `crates/rfcspec-ess`. Both add participant fields, `set` and guards, and no edge links them. Its body says the model "has no participant state … and no guards", which is false once the frames wave has landed. Remedy: an edge to `story:websocket-machine-guards` (or to the frames epic's last story) that names those files. The alternative is to narrow this story to what is new, the `state_implication` property. — `.engineering/planning/story/dns-query-machine.md:48-54`, `.engineering/planning/story/websocket-machine-guards.md:20-33`
- story:proposal-record-replay — it is the first ai story and shares `AGENTS.md`, `Cargo.toml`, `Cargo.lock`, `crates/rfcspec-cli/tests` and `check.rs` with the dns stories and the frames stories. Its body justifies "No `depends_on`" by reference to the websocket wave only, so `waves` puts it in wave 1. Remedy: an edge to `story:dns-query-machine` that records the shared files as its reason. The alternative is to split the surface. — `.engineering/planning/story/proposal-record-replay.md:116-117`, `.engineering/planning/story/dns-header-codec.md:11-18`
- story:websocket-proposal-slice — it lands on `profiles/websocket/overlay.yaml`, `website/docs/catalog/websocket.md`, `crates/rfcspec-docs/src/catalog.rs` and `crates/rfcspec-cli/tests/websocket.rs`, as do the ping and fragments stories. It is also a reader beside a producer. `story:websocket-fragments-machine` pins "126 unreviewed (82 MUST …)" and the websocket row of the index, and this story's accept decisions lower the same unreviewed figure. Run first, it would break that pinned number. Its only edge is to `story:proposal-review`. Remedy: an edge to `story:websocket-fragments-machine`. — `.engineering/planning/story/websocket-proposal-slice.md:99-101`, `.engineering/planning/story/websocket-fragments-machine.md:105-108`
- story:proposal-machine-layout — its test records, replays and accepts a Ping/Pong machine from §5.5.2 and §5.5.3 on a temporary copy of `profiles/websocket`. Its own rule refuses a machine whose name the overlay already holds (`AlreadyExists`). The frames epic's `ping` machine writes the overlay it reads, and no edge records that. Its answer shape, "a double answer for a Ping/Pong machine", does not say what name the machine takes. Remedy: an edge to `story:websocket-ping-machine`, or a body line that fixes a name other than `ping`. — `.engineering/planning/story/proposal-machine-layout.md:49-53,64-66`, `.engineering/planning/story/websocket-ping-machine.md:94-97`
- story:bulk-summary-format — it shares `Cargo.lock` and `crates/rfcspec-cli/src/main.rs` with all four ai stories. It also takes the whole `crates/rfcspec-model` and `crates/rfcspec-extract` directories, which hold the files the guards, dns-query-machine and proposal-review stories edit. The scope lists directories, so `waves` does not print those collisions. Its body names only `story:websocket-review-matrix` as its predecessor. It adds `EXTRACTOR_VERSION` at 1 and requires a bump on any extraction change, and `story:dns-source-catalogue` may change the extractor, so the order matters. Remedy: edges to `story:proposal-machine-layout` and `story:websocket-proposal-slice`, with the shared files as the reason. — `.engineering/planning/story/bulk-summary-format.md:22-26,105-108`
- story:bulk-fetch-corpus — its `task bulk` step runs `rfcspec bulk check --profiles profiles`. That check verifies every `profiles/*/rfcspec.lock`, including `profiles/dns/rfcspec.lock`, which `story:dns-source-catalogue` creates. Its acceptance names only the RFC 868 and RFC 6455 rows. This is a reader beside a producer with no edge. Remedy: add the edge to `story:dns-source-catalogue` and add the RFC 1035 row to the acceptance. — `.engineering/planning/story/bulk-fetch-corpus.md:59-66`, `.engineering/planning/story/bulk-summary-format.md:65-68`, `.engineering/planning/story/dns-source-catalogue.md:55-56`

**What I read:** 17 artifacts (4 epics, 13 stories) and the earlier `review-result:websocket-parallel-round-2`. Commands: `aep plan artifact show`, `graph` and `waves`, plus `cat -n` on each story file. I grouped the `waves` collisions by epic pair with a throwaway script.

**Surfaces:** 13 stories placed cited, 0 inferred, 0 unplaceable. The four epics are containers with no scope. Cross-epic acceptance reads traced to a producer: 4 (dns-query-machine on guards, the slice on fragments, machine-layout on ping, bulk-fetch on the dns lock), with 0 recorded by an edge. In-epic reads traced: 11, all 11 recorded by `depends_on`.

**What I could not establish:**
- `profiles/websocket` does not exist in the tree yet, because `epic:websocket-profile` is still active. I could not check any websocket path or symbol against code.
- Several scopes name a directory (`crates/rfcspec-ess`, `crates/rfcspec-extract`, `crates/rfcspec-model`, `crates/rfcspec-docs`, `crates/rfcspec-cli/tests`). `waves` matches paths literally and misses file-versus-directory overlaps, so the real collision set is larger than the `waves` output. The bulk stories take whole directories, and that is how a wide scope forbids everything.
- Out of my lane: whether dns-query-machine duplicating the guards story is the right split belongs to the design critic. The numeric claims (21, 29, 126, 9,847) and the RFC line numbers belong to the acceptance critic.

```findings
- file: .engineering/planning/story/dns-source-catalogue.md
  line: 26
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "shares website/docs/catalog/index.md (cited) with story:websocket-ping-machine and story:websocket-fragments-machine and has no depends_on edge to the frames epic, so waves places it in wave 2 beside story:websocket-machine-guards instead of after the frames wave; add an edge to story:websocket-fragments-machine recording the shared catalogue pages, or split the surface"
- file: .engineering/planning/story/dns-query-machine.md
  line: 48
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "adds participant fields, set and guards to crates/rfcspec-model/src/lib.rs, crates/rfcspec-extract/src/check.rs and crates/rfcspec-ess (cited), the same constructs story:websocket-machine-guards adds, with no depends_on edge, and its claim that the model has no guards is false after the frames wave; add the edge naming those files, or narrow the story to state_implication"
- file: .engineering/planning/story/proposal-record-replay.md
  line: 116
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "first ai story shares AGENTS.md, Cargo.toml, Cargo.lock, crates/rfcspec-cli/tests and crates/rfcspec-extract/src/check.rs (cited) with the dns stories and AGENTS.md and check.rs with the frames stories, and states 'No depends_on' on websocket-wave grounds only, so waves places it in wave 1; add an edge to story:dns-query-machine recording the shared files, or split the surface"
- file: .engineering/planning/story/websocket-proposal-slice.md
  line: 99
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "shares profiles/websocket/overlay.yaml, website/docs/catalog/websocket.md, crates/rfcspec-docs/src/catalog.rs and crates/rfcspec-cli/tests/websocket.rs (cited) with the ping and fragments stories, and its accept decisions change the unreviewed count that story:websocket-fragments-machine pins (126 unreviewed), a reader beside a producer; the only edge is to story:proposal-review, so add an edge to story:websocket-fragments-machine"
- file: .engineering/planning/story/proposal-machine-layout.md
  line: 49
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its test accepts a Ping/Pong machine into a temporary copy of profiles/websocket whose overlay already holds a ping machine after story:websocket-ping-machine, and its own rule refuses an existing machine name (AlreadyExists); no edge or fixed machine name records this, so add the edge or name the proposed machine other than ping"
- file: .engineering/planning/story/bulk-summary-format.md
  line: 105
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "shares Cargo.lock and crates/rfcspec-cli/src/main.rs (cited) with proposal-record-replay, proposal-review and proposal-machine-layout and takes whole crates/rfcspec-model and crates/rfcspec-extract directories that the guards, dns-query-machine and proposal-review stories edit, yet names only story:websocket-review-matrix as predecessor; EXTRACTOR_VERSION also depends on extractor changes made by story:dns-source-catalogue; add edges to story:proposal-machine-layout and story:websocket-proposal-slice recording the shared files"
- file: .engineering/planning/story/bulk-fetch-corpus.md
  line: 59
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "task bulk runs bulk check over --profiles profiles, which verifies profiles/dns/rfcspec.lock created by story:dns-source-catalogue, a reader beside its producer with no depends_on edge and an acceptance that names only the RFC 868 and RFC 6455 rows; add the edge and the RFC 1035 row"
```
