---
format: aep.planning-md/3
id: review-result:wave6-scope-round-1
kind: review-result
status: active
title: 'Wave 6 plan critic, scope, round 1: approve'
relations:
- reviews: story:abnf-core
- reviews: story:catalogue-grammars
- reviews: story:fixed-arm-bounds-check
- reviews: story:explorer-look
revision: 1
---
approve

I read all four stories, both parents, the operator approval record and the review result. I also read `epic:wire-layer`'s eleven stories, `story:wire-scaffold`, `docs/design/rfcspec.md` and `ess/domains/catalogue.yaml`. I ran `aep plan artifact show`, `graph` and `relations`, and checked `check.rs:258-300` at main.

- **Promises traced (part 3):** I extracted 9 promises from the parents and traced 9 to an item. The first six are from `epic:wire-layer`, then three from the other parents.
  - Six wire-layer promises:
    - ABNF parsed into a grammar (`story:abnf-core`).
    - Parser and printer generated (`story:handshake-codec` and others, outside wave 6).
    - ASCII-art diagram read into a layout (`story:diagram-layouts`, outside wave 6).
    - "for text and RFCXML sources alike" (`story:catalogue-grammars`).
    - "Waves 5 and 6 start it: wire-scaffold, then abnf-core and catalogue-grammars" (matches the wave).
    - 4.3 and 5.2 handled per section (`story:abnf-core` and `story:catalogue-grammars`).
  - One from the review result: finding 3, the fixed arm admitted at 0 or above `max_time_ms` (`story:fixed-arm-bounds-check`).
  - Two from the explorer request: the same colours and font, and the same header and footer, in the documentation site's look (`story:explorer-look`).
- **Reach beyond the parent:** none.
  - `story:abnf-core` goes slightly past the epic text. The 7405, core-rule, import and matcher parts trace to design §13 (`rfcspec-abnf`: "RFC 5234 and 7405 parser, bit-level mode, import resolution") and §8.2 (`matches`). They stay inside the epic's own design reference, and the story's "Not here" line keeps boundedness analysis and cross-RFC import resolution out.
  - `story:catalogue-grammars` is S2 collection per design §12.2, and it names `EXTRACTOR_VERSION` as unchanged.
  - `story:fixed-arm-bounds-check` takes only finding 3. It leaves the never-armed backoff and the null-key parse, which are findings 1 and 2 of the same review, out.
  - `story:explorer-look` stays inside `explorer/src`. I found nothing in it beyond the request.
- **Overlap:** the two ABNF stories split cleanly. `abnf-core` refuses the 5.3 pseudo-code figure and `catalogue-grammars` keeps it as an unreviewed grammar, and the two bodies state this consistently.
- **`Grammar.uses`:**
  - The epic does not expect a bridge. `epic:wire-layer` says nothing of imports, and its only mention of `uses` is none at all (`ess/domains/catalogue.yaml:712` and `:1739` carry the field).
  - No story among the eleven decomposing the epic fills `uses`. `abnf-rfc2616` keeps `token` and `quoted-string` as unresolved names, so no M1 story needs a catalogue import.
  - The promise lives only in design §8.1 and §13, so the gap sits below the epic and the drafter named it ("`uses` empty"). That is a named omission, not a coverage gap.
  - If the operator wants `uses` filled, that is a new story: `abnf-core`'s `RuleImport` (rfc, rule) maps to `rfcspec_model::wire::RuleImport`, with a call into `collect`.

**What I could not establish**
- Whether "header and footer" in `story:explorer-look` falls within "same look and feel". I read it as part of the site's look, so I did not flag it. If the operator meant colours and font only, it is the one candidate for removal.
- `story:fixed-arm-bounds-check` carries its source, `review-result:ntp-timer-adversary-1`, only as prose ("Source:"). The graph has no edge from it, and I could not tell whether the CLI allows `derived_from` to a review-result.
- The `.gitignore` the approval record says `story:explorer-look` touches is not in the story's scope. That is missing, not extra, so it is not flagged.
- Outside my lane: the duplicate `RuleImport` types in `rfcspec-abnf` and `rfcspec-model` belong to design. The wave-6 design critic raised it.

```findings
[]
```
