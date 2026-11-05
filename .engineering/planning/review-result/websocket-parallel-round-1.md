---
format: aep.planning-md/3
id: review-result:websocket-parallel-round-1
kind: review-result
status: active
title: parallel critic, round 1, WebSocket profile
relations:
- reviews: epic:websocket-profile
- reviews: story:websocket-source-catalogue
- reviews: story:websocket-close-machine
- reviews: story:websocket-frame-codec
- reviews: story:websocket-review-matrix
revision: 1
---
needs-revision

- story:websocket-review-matrix — its scope names `crates/rfcspec-cli/src/checks.rs` and the body says "`check.rs`/`checks.rs` scope rule", but the "every MUST-level in-scope requirement has a status" rule does not land there, and no `crates/rfcspec-cli/src/check.rs` exists. `rfcspec check` runs `rfcspec_extract::check(&overlay, &catalogue, &source)` from `fn check` in `crates/rfcspec-cli/src/main.rs:235-240`. That function takes no profile sections. The scope sections live in `profile.yaml`, so the rule lands in `main.rs` or in `crates/rfcspec-extract/src/check.rs` with a signature change. `checks.rs` holds only `semantics`, `runs` and `run_files`. `crates/rfcspec-extract/src/check.rs` is declared by the source, close and codec stories, so the matrix's real surface is undeclared (inferred, from reading `main.rs` and `checks.rs`). Today the chain orders it after all three, but the listed scope understates where it collides with them and with any other item on those files. The body should name the file the rule lands in.
  `.engineering/planning/story/websocket-review-matrix.md:88`, `:16`; `crates/rfcspec-cli/src/main.rs:240`

I found no other collision inside the set. Source, close, codec and matrix form a `depends_on` chain (`aep plan artifact graph`). Every shared surface the waves tool reports among them has an ordering edge, and the close, codec and matrix bodies each name the shared files as the reason. These surfaces are the overlay, `crates/rfcspec-model/src/lib.rs`, `crates/rfcspec-extract/src/check.rs`, `Taskfile.yml`, `Cargo.lock`, the catalogue pages and the downloads.

**What I read:** 5 artifacts (the epic and 4 stories), via `aep plan artifact show` on each, `aep plan artifact waves` and `aep plan artifact graph`. I read the critic procedure and the rubric. In the tree I read `crates/rfcspec-cli/src/main.rs`, `crates/rfcspec-cli/src/checks.rs`, `crates/rfcspec-extract/src/check.rs` and `Cargo.toml`.

**Surfaces:** 4 stories placed **cited**, 0 **inferred**, 0 unplaceable. The epic is a container with no scope. The one finding rests on an inferred reading of the code, as noted above. I traced 5 acceptance reads to a producer in the set:
- close reads the overlay and ids from source (direct edge).
- codec reads the overlay from close (direct edge).
- matrix reads the layout from codec (direct edge).
- matrix reads the machine elements from close (transitive).
- matrix reads the keyword counts and ids from source (transitive).

3 of the 5 have a direct `depends_on` edge, and the other 2 are ordered through the chain.

**What I could not establish:**
- `aep plan artifact waves` also reports collisions between these stories and `story:catalog-index`, `story:catalog-profile-page`, `story:catalog-downloads`, `story:catalog-semantics` and `story:catalog-recorded-runs`. They share `website/docs/catalog/index.md`, `website/docs/catalog/time.md`, `Taskfile.yml` and `crates/rfcspec-cli`. Those items are outside the set, so I make no finding. The operator may want that pairing reviewed.
- The numeric claims (214 keywords, line numbers) belong to `plan-critic-acceptance`, not to me.

```findings
- file: .engineering/planning/story/websocket-review-matrix.md
  line: 88
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the scope names crates/rfcspec-cli/src/checks.rs and a nonexistent check.rs for the in-scope-MUST-needs-a-status rule, but rfcspec check runs rfcspec_extract::check from crates/rfcspec-cli/src/main.rs:240 and profile sections are not an input to it, so the rule lands in main.rs or crates/rfcspec-extract/src/check.rs (declared by the source, close and codec stories); the real surface is undeclared (inferred from reading main.rs and checks.rs); name the file the rule lands in"
```
