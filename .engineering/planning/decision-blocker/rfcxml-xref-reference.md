---
format: aep.planning-md/3
id: decision-blocker:rfcxml-xref-reference
kind: decision-blocker
status: cleared
title: 'Nobody has decided what an RFCXML xref becomes in the catalogue: an Rfc-to-Rfc edge, a reference from a section or requirement, and who owns it'
relations:
- blocks: epic:rfc-explorer
revision: 5
transitions:
- {from: "open", to: "cleared", at: "2026-11-05T17:37:05Z", actor: "human:timo", revision: 5}
---
## Question

The epic lists `<xref>` as one of the markups RFCXML gives: "`<xref>` gives the references between
RFCs for the explorer". RFC 9220 alone holds 35 `<xref>` elements (8 of them in its table of contents), to other RFCs (`RFC8441`,
`RFC6455`), to a section of another RFC (`section="15.6.2"` of `HTTP`, RFC 9110) and to its own
sections (the table of contents). No `ess/1` document declares a reference: `ess/domains/catalogue.yaml`
has `rfcspec.catalogue.Rfc` with update-graph fields from the index (`obsoletes`, `updates`, ...), which
name RFC numbers, not catalogue entries, and no relation from an `Rfc`, `Section` or `Requirement` to
another RFC or section. Design section 4 lists "references" as S1 and S2 output; section 5 has no
noun for them.

What does an xref become, and how does it relate to what exists?

- Entities and direction: `Rfc -> Rfc`, `Section -> Rfc`, `Section -> Section` (a cited section of
  another RFC), or `Requirement -> Section` (the sentence that cites it)?
- Cardinality: many-to-many; may the far side be an RFC that was never read (as the update graph
  allows), or a non-RFC reference (`<reference>` without an RFC `seriesInfo`, an I-D, a W3C
  document)?
- Ownership and lifecycle: owned by the citing RFC (deleted with it), or a separate projection
  rebuilt from every read RFC; does a cited section going `stale` affect the citing item?
- Normative and informative references: one relation with a kind, or two?

## Options seen

- A: a new entity `rfcspec.catalogue.Reference` (citing rfc, citing section, target rfc, target
  section, normative or informative), owned by the citing `Rfc`.
- B: an `Rfc` field `cites: List<Integer>` beside the update graph, numbers only, no section.
- C: leave references out of the catalogue; the explorer (`epic:rfc-explorer`) reads them from
  the normalized document directly.

## What it stops

Any story that extracts `<xref>` into the catalogue, the bulk summary or the explorer. Per the
repository rule (AGENTS.md: "A new noun gets its ESS declaration in `ess/` before any AEP story is
written around it") the answer is an `ess/domains/catalogue.yaml` change first. No such story is
drafted. The reader story (`story:rfcxml-source-reader`) renders an xref into its text and nothing
more, so it does not depend on the answer.

## Decision (2026-11-05)

Option A, decided by the session sys-mon. The basis it gave, quoted from its message: "Timo's delegation, typed 2026-11-04T23:24:57Z, verbatim: \"if the others have issues deciding something, then you are deciding too\", and his grant line in the workspace CLAUDE.md of 2026-11-05 (sys-mon's message quoting his words counts for decisions a session cannot make itself)."

The decision as sys-mon's message gives it, verbatim:

> option A, a new entity `rfcspec.catalogue.Reference` owned by the citing `Rfc`.
> - Fields: citing rfc (required), citing section (optional), target rfc number as Integer (required; an unread RFC is allowed, as the update graph allows), target section as Text (optional), kind: normative or informative. One entity, one kind field.
> - Excluded for now: self-references (target rfc equals citing rfc, the table of contents) and non-RFC references (I-Ds, W3C, a reference without an RFC seriesInfo); count them in the read summary, do not store them.
> - Ownership: owned by the citing Rfc, rebuilt when that Rfc is re-read, deleted with it. A cited section going stale does not change the citing Reference.
> Why not B or C: B loses the section and the kind the explorer needs; C makes the explorer parse RFCXML a second time.
> Order: the ESS declaration in ess/domains/catalogue.yaml first (repository rule), then the extraction story, then epic:rfc-explorer; epic:wire-layer after it.

Declared in `ess/domains/catalogue.yaml`:

- entity `rfcspec.catalogue.Reference`, with fields `rfc`, `section_id` (optional), `target_rfc` (Integer), `target_section` (optional String) and `kind`, and a `section` relation to the citing section;
- enum `rfcspec.catalogue.ReferenceKind`, with values `normative` and `informative`;
- relation `references` on `rfcspec.catalogue.Rfc`, `owns`, many, via `rfc`.

The extraction is `story:xref-reference-extract`.
