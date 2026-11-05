---
format: aep.planning-md/3
id: review-result:wire-scaffold-adversary-1
kind: review-result
status: active
title: Adversary pass 1 on story:wire-scaffold
relations:
- reviews: story:wire-scaffold
revision: 1
---
unit: story:wire-scaffold, uncommitted working tree on 60e0a56 in the wave5-wire-scaffold worktree
verdict: NEEDS-CHANGE
cases: executed 34→37, red 2
origin: introduced 4 / pre-existing 1 / undecided 0
wrote-outside-worktree: none
needs-coordinator: whether `CatalogueField` should carry the ESS `length` field now, or ESS `LayoutField.length` be marked as deferred. The story left `length` out on purpose, and story:diagram-layouts says it stays unset.

**1. Diff stat.** `git --no-pager diff --stat` is unchanged from the tree I was handed (the implementor's 5 tracked files, 27 insertions). I edited no implementation file. I added two untracked test files:
- `crates/rfcspec-model/tests/adversary_wire_ess.rs`
- `crates/rfcspec-wire/tests/adversary_dialects_agree.rs`

**2. Cases added.**

`crates/rfcspec-model/tests/adversary_wire_ess.rs` holds two cases, both red now and red on their first run (`cargo test -p rfcspec-model --test adversary_wire_ess`, EXIT=101):
- `layout_field_reads_the_ess_dependent_length`: an ESS `LayoutField` with `length: {field, unit}` should deserialize into `CatalogueField`.
  ```
  ESS rfcspec.catalogue.LayoutField declares length: Optional<DependentLength>: Err(Error("unknown field `length`, expected one of `name`, `bits`, `presence`, `constraints`", line: 3, column: 16))
  ```
- `layout_field_reads_the_ess_field_constraint_shape`: an ESS `FieldConstraint` `{kind: range, values: [0, 15]}` should be accepted.
  ```
  ESS rfcspec.catalogue.FieldConstraint is {kind, values}: Err(Error("unknown variant `kind`, expected one of `equals`, `range`, `one_of`", line: 4, column: 31))
  ```

`crates/rfcspec-wire/tests/adversary_dialects_agree.rs` holds one case, green:
- `abnf_dialect_and_model_dialect_share_every_wire_name`: every `rfcspec_model::wire::AbnfDialect` serializes to the same name as the matching `rfcspec_abnf::Dialect::wire()`, and `from_wire` reads it back. It guards against the two enums drifting apart.

**3. Suite run** (after the cases existed):
```
CARGO_PROFILE_DEV_DEBUG=line-tables-only CARGO_INCREMENTAL=0 RUST_TEST_THREADS=4 nice -n 10 ionice -c3 cargo test --no-fail-fast -p rfcspec-abnf -p rfcspec-wire -p rfcspec-model
```
- abnf dialect.rs: 2 ok
- model unit tests: 29 ok
- adversary_port: 2 ok
- adversary_wire_ess: 0 passed, 2 FAILED
- wire adversary_dialects_agree: 1 ok
- stubs: 1 ok
- result: EXIT=101

37 cases ran. The before figure of 34 is this same run minus my three cases.

**4. Findings.**

1. **`length` missing from the layout field.**
   - Where: `crates/rfcspec-model/src/wire.rs:52-61`.
   - Measured: `CatalogueField` has `deny_unknown_fields` and no `length` field, so a field written in ESS `LayoutField` form is refused (`adversary_wire_ess.rs:10`). The doc comment on `bits` says the width is "absent where the length depends on another field", yet nothing records which field.
   - What reaches it: nothing today. story:diagram-layouts says `length` "is left unset", and story:frame-agreement's "payload-data whole octets" would need it.
   - Verdict: INFEASIBLE, introduced, note. It is ESS drift that no current producer hits.
   - Fix: either add `#[serde(default, skip_serializing_if = "Option::is_none")] pub length: Option<DependentLength>` now, so that wire.rs stays untouched by later stories, or record the deferral in ESS.
2. **Constraint encoding differs from ESS.**
   - Where: `crates/rfcspec-model/src/lib.rs:561-567`, reused at `wire.rs:60`.
   - Measured: `Constraint` is written as an externally tagged enum (`{range: [lo, hi]}`), while ESS `FieldConstraint` is `{kind, values}` (`adversary_wire_ess.rs:24`).
   - Origin: pre-existing. `Constraint` and the overlay `Field` (lib.rs:546-548, which already cites ESS `FieldConstraint`) are unchanged from the base. The new type inherits the mismatch.
   - What reaches it: nothing found; `rfcspec-ess` does not map `Constraint`.
   - Verdict: INFEASIBLE, note.
3. **`dialect` left off `Grammar`.**
   - Where: `crates/rfcspec-model/src/wire.rs:9-24`.
   - ESS `Grammar` has `dialect: Optional<AbnfDialect>`. The model leaves it out, and `deny_unknown_fields` refuses it.
   - This is the story's own decision: the dialect lives in `OverlayGrammar` (story § Domain relations).
   - Verdict: CONFIRMED as a decision, introduced, note. No test written.
4. **`section` doc comments disagree with the plan.**
   - Where: `crates/rfcspec-model/src/wire.rs:15` and `:41`.
   - The comments say `section` is "the number of the numbered section". story:catalogue-grammars and ESS `Span.section` say it is the anchor when read from XML and the number when read from text.
   - The field is also a required `String`, while `Block.section` and the requirement sections are `Option` ("absent before the first numbered heading and in an RFC without numbered sections", lib.rs:224-227). A figure outside a numbered section has no representable value.
   - Verdict: NEEDS-CHANGE, introduced, warning, for the doc comment at least.
5. **Two dialect enums with no conversion.**
   - Where: `crates/rfcspec-abnf/src/lib.rs:8` and `crates/rfcspec-model/src/wire.rs:67`.
   - The two enums agree today (my green case shows it). Nothing converts between them, and neither crate can hold a `From` impl without a manifest edit, which the story rules out.
   - story:frame-agreement's CLI must map the overlay `AbnfDialect` to `Dialect`; going through the wire name works with the current dependencies.
   - Verdict: CONFIRMED, introduced, note.

**5. Attacked and could not break.**
- Dialect wire names: `rfc5234`, `rfc2616` and `bits` match on both enums and in ESS, and `abnf` is refused.
- `RuleImport`, `Presence` (`equals` as i64 for an ESS Integer) and `bits` (`Option<u32>`) match ESS.
- The stubs test is real for a stub. It checks the source against the lock, checks that the reader produced blocks, and calls both collectors on RFC 6455.
- Dependency edges: `cargo metadata` matches the acceptance exactly.
  - The edges are acyclic: model ← doc ← wire ← extract, and abnf has no dependencies (only rfcspec-doc as a dev-dependency).
  - The helpers the later stories call (`text_digest`, `abnf_rule`, `sha256_hex`) live in model or doc, not in extract, so wire never needs extract.
- Nothing changed under `crates/rfcspec-extract/src/`.
- No comments in any `Cargo.toml`, no `/home/` paths and no Python.

**6. Paths written outside the worktree:** none. A temporary metadata file in `~/.cache/claude-tmp/` was deleted. The build went to the tree's own `target/`.

```findings
- file: crates/rfcspec-model/src/wire.rs
  line: 52
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: CatalogueField omits ESS LayoutField.length (Optional<DependentLength>) and deny_unknown_fields refuses it; no current producer writes it.
- file: crates/rfcspec-model/src/lib.rs
  line: 561
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: Constraint serializes externally tagged, not as ESS FieldConstraint {kind, values}, and CatalogueField inherits the mismatch; nothing maps it to ESS today.
- file: crates/rfcspec-model/src/wire.rs
  line: 11
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: Grammar drops the ESS dialect field by design (it lives in OverlayGrammar), so an ESS-shaped grammar carrying dialect is refused.
- file: crates/rfcspec-model/src/wire.rs
  line: 15
  category: judgement
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Grammar and CatalogueLayout document section as a section number while ESS Span and story:catalogue-grammars use the anchor for XML, and the required String cannot express the absent section that Block.section allows.
- file: crates/rfcspec-abnf/src/lib.rs
  line: 8
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: rfcspec_abnf::Dialect and rfcspec_model::wire::AbnfDialect are parallel enums with no conversion, and the frozen manifests leave no crate that can host a From impl.
```
