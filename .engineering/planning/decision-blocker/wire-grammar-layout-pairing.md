---
format: aep.planning-md/3
id: decision-blocker:wire-grammar-layout-pairing
kind: decision-blocker
status: cleared
title: Nobody has decided whether the catalogue records which bit-level grammar a layout is checked against, and who owns the agreement result
revision: 4
transitions:
- {from: "open", to: "cleared", at: "2026-11-05T20:49:37Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"review_outcome":1}}}
---
## Question

Does the catalogue (or the `rfcspec-wire/1` sidecar) record which bit-level grammar a packet layout is checked against, and if so: what is the cardinality (one grammar rule per layout, or many to many), which side owns the pairing, and who owns the result of the agreement check?

## Why nobody could read the answer

- `ess/domains/catalogue.yaml` declares `rfcspec.catalogue.Grammar` and `rfcspec.catalogue.Layout`, each owned by `rfcspec.catalogue.Rfc` (relations `grammars` and `layouts`), and no relation between a Grammar and a Layout.
- Design section 8.1 says RFCspec "imports both and checks that they agree (field order, widths, conditions); a disagreement is a finding for review", without saying where the pair is declared.
- An overlay `Finding` (`crates/rfcspec-model/src/lib.rs:601-611`) names one item; no `ess/1` document declares Finding.

## Decision

2026-11-05, decided by the rfcspec session under the operator's delegation: milestone M1 stores no grammar-layout pairing. story:frame-agreement takes the grammar id, the rule and the layout id as command arguments, stores nothing, and reports a disagreement against an overlay finding whose item is the layout id.

Persisting the pair is later work and has no story yet: lowering grammars and layouts into an `rfcspec-wire/1` sidecar that carries the binding, running the agreement check for every paired grammar and layout of a profile without naming them on the command line, and an ESS declaration for the agreement result. Whoever drafts that work answers the cardinality and ownership questions above in an `ess/1` document first.
