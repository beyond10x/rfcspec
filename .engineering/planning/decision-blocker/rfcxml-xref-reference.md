---
format: aep.planning-md/3
id: decision-blocker:rfcxml-xref-reference
kind: decision-blocker
status: open
title: 'Nobody has decided what an RFCXML xref becomes in the catalogue: an Rfc-to-Rfc edge, a reference from a section or requirement, and who owns it'
relations:
- blocks: epic:rfc-explorer
revision: 3
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
