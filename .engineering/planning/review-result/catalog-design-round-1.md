---
format: aep.planning-md/3
id: review-result:catalog-design-round-1
kind: review-result
status: active
title: design critic, round 1, protocol catalogue
relations:
- reviews: epic:public-protocol-catalog
- reviews: story:catalog-index
- reviews: story:catalog-profile-page
- reviews: story:catalog-downloads
- reviews: story:catalog-semantics
- reviews: story:catalog-recorded-runs
revision: 1
---
needs-revision

- story:catalog-index — the index story links each row to the profile page route `/docs/catalog/<name>`, a page that only `story:catalog-profile-page` creates, and `website/docusaurus.config.ts` sets `onBrokenLinks: 'throw'`. Its own `task website` acceptance therefore cannot pass when it lands alone. The graph's only edge runs the other way, and the link is also claimed by `story:catalog-profile-page`: "The index row of each profile links to its page". Revision: render the name as plain text in the index story and leave the link to the profile-page story. Citations: `.engineering/planning/story/catalog-index.md:27`, `.engineering/planning/story/catalog-index.md:49`, `website/docusaurus.config.ts:21`, `.engineering/planning/story/catalog-profile-page.md:34`.
- story:catalog-semantics — the set is one queue of five (index, profile-page, downloads, semantics, recorded-runs) and every edge is justified only by a shared file. `story:catalog-semantics` and `story:catalog-downloads` share only `catalog.rs`: semantics adds a model field and a `check` rule, downloads copies files. There are two options. Keep the edge and accept the serial order, as drafted. Or have `story:catalog-profile-page` put one section per module (for example `catalog/downloads.rs` and `catalog/semantics.rs`) so the downloads → semantics edge can drop. This is a trade-off, not a request to remove the edge, and it is a warning. Citations: `.engineering/planning/story/catalog-semantics.md:67`, `aep plan artifact graph`.

**What I read.** Six artifacts, in full (the epic and the five stories), plus the two cleared decision-blockers. Commands: `aep plan artifact show` for each, `relations`, `graph` and `validate`, which reported valid with 25 artifacts. I also read `website/docusaurus.config.ts` and `crates/rfcspec-docs/src/main.rs`.

**Graph and reads.** I walked 12 edges among the set, the epic and the two blockers, and the `epic:public-protocol-catalog` → `epic:rfc868-thin-slice` edge outside the set. There is no cycle. About 9 acceptance reads were traced to a producer inside the set, and 8 are recorded by a `depends_on` edge. The one exception is the index-to-profile-page link above, where the edge runs the wrong way. Reads of `profiles/time/generated/` and of the `story:rfc868-verify-probe` server go to implemented artifacts outside the set, so I did not count them.

**What I could not establish.**
- Out of my lane (scope): `story:catalog-index` says "the first of three stories" and "the two later stories", but the set has five.
- Out of my lane (scope): the epic wants a conformance verdict on the index, which is delivered only by `story:catalog-recorded-runs`.
- I did not check whether Docusaurus counts a link from `index.md` to an absent route as broken. I took `onBrokenLinks: 'throw'` at face value.

```findings
- file: .engineering/planning/story/catalog-index.md
  line: 27
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the index links each row to the profile page route that only story:catalog-profile-page creates, so its own task website acceptance cannot pass under onBrokenLinks throw, no edge runs that way, and story:catalog-profile-page claims the same link; render the name as plain text here and leave the link to the profile-page story"
- file: .engineering/planning/story/catalog-semantics.md
  line: 67
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the depends_on edge to story:catalog-downloads exists only because both edit crates/rfcspec-docs/src/catalog.rs, which leaves the five stories one queue; either keep the order or have story:catalog-profile-page split catalog.rs into one module per page section so this edge can drop"
```
