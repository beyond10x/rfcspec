---
format: aep.planning-md/3
id: story:constraint-ess-shape
kind: story
status: draft
title: Field constraints read and write the ESS FieldConstraint shape
scope:
- confidence: cited
  path: crates/rfcspec-model/src/lib.rs
- confidence: cited
  path: crates/rfcspec-model/tests/adversary_wire_ess.rs
revision: 2
---
## Story

The model's field `Constraint` (`crates/rfcspec-model/src/lib.rs:561-567`) serialises externally tagged (`{range: [lo, hi]}`), while ESS `rfcspec.catalogue.FieldConstraint` (`ess/domains/catalogue.yaml`) is `{kind, values}`. The overlay `Field` and the new `CatalogueField` both use it, so an ESS-shaped constraint is refused. Nothing maps it to ESS today. Found by adversary pass 1 on story:wire-scaffold (`review-result:wire-scaffold-adversary-1`) as pre-existing.

## Acceptance

- The ignored case `layout_field_reads_the_ess_field_constraint_shape` in `crates/rfcspec-model/tests/adversary_wire_ess.rs` is un-ignored and passes.
- Every committed overlay still reads: `rfcspec check --all profiles` exits 0, after the four overlays' constraints are rewritten to the ESS shape, or by accepting both shapes. Whichever is chosen is stated in this story before work starts.

## Scope

- `crates/rfcspec-model/src/lib.rs`
- `crates/rfcspec-model/tests/adversary_wire_ess.rs`
- `profiles/*/overlay.yaml`, if they are rewritten
