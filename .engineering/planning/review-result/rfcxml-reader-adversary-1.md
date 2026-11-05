---
format: aep.planning-md/3
id: review-result:rfcxml-reader-adversary-1
kind: review-result
status: active
title: Adversary on story:rfcxml-source-reader, pass 1
relations:
- reviews: story:rfcxml-source-reader
revision: 1
---
unit: story:rfcxml-source-reader, uncommitted working tree on base 05c8426, in the rfcxml-source-reader worktree
verdict: NEEDS-CHANGE
cases: executed 203→211, red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (deleted)
needs-coordinator: decide whether text-only callers should refuse an XML lock or send it to the XML reader

**1. Diff**

`git diff --stat` shows only the implementor's 9 tracked files, unchanged by me. Both of my files are new and untracked, and both are test files:
- `crates/rfcspec-doc/tests/rfcxml_adversary.rs` (7 cases)
- `crates/rfcspec-propose/tests/xml_lock.rs` (1 case)

No implementation file was touched.

**2. Cases added.** Each was run alone before the suite.

- **Red:** `an_artset_gives_one_figure_its_ascii_art_and_no_empty_svg_figure`. An `<artset>` (SVG plus ascii-art) gives 2 figures. One is `type: svg` with lines `["", "", ""]`.
  ```
  assertion `left == right` failed
    left: 2
   right: 1
  ```
- **Red:** `a_br_in_a_table_cell_does_not_join_the_words_either_side`. `<td>first<br/>second</td>` reads as one word.
  ```
  rfcxml_adversary.rs:108:5
    left: ["firstsecond"]
   right: ["first", "second"]
  ```
- **Red:** `loading_a_profile_whose_source_is_locked_as_xml_refuses_it`. `rfcspec_propose::Workspace::load` on a profile whose lock names `sources/rfc9220.xml` returns Ok: it runs the plain-text reader over XML markup.
  ```
  xml_lock.rs:30:5:
  a source locked as XML was loaded through the plain-text reader
  ```
- **Green:** BCP 14 marks inside `<em>`, `<li>`, `<li><t>`, `<dd>` (spanning lines) and `<td>`, with non-ASCII text. Every byte offset points at its keyword in `Block::text()`.
- **Green:** CRLF line ends give the same blocks as LF, CDATA included.
- **Green:** entities and CDATA in `<sourcecode>` are kept verbatim.
- **Green:** an external `SYSTEM` entity is not resolved. roxmltree has no default entity resolver.
- **Green:** six levels of tenfold entity nesting (about 600 B expanding to 3 MB) were refused or stayed bounded. roxmltree caps entity depth at 10 and 255 references per reference.

**3. Suite**

Command: `cargo test -p rfcspec-doc -p rfcspec-source -p rfcspec-cli -p rfcspec-propose --no-fail-fast`, with nice/ionice and the brief's environment variables.

Result: EXIT=101, 208 passed, 3 failed. The 3 failures are exactly the cases above. All other targets passed, including `rfcspec-cli` `tests/rfcxml.rs` 5/5, `rfcspec-doc` `tests/rfcxml.rs` 9/9 and `rfcspec-source` 9/9. The before count of 203 is 211 minus my 8 cases, not a separate run.

**4. Findings**

| file:line | verdict | origin | what was measured | what reaches it |
|---|---|---|---|---|
| `crates/rfcspec-propose/src/lib.rs:207` | NEEDS-CHANGE | introduced | `Workspace::load` returns Ok for an XML lock and builds its catalogue with the text reader over XML. `crates/rfcspec-docs/src/catalog.rs:146` does the same without a test. `bulk.rs:212` is less exposed because its sha256 check against the summary row reports a mismatch first. | `rfcspec fetch --format xml` into any profile with an overlay, then `rfcspec propose` or `rfcspec-docs generate`. No committed profile is in that state today. |
| `crates/rfcspec-doc/src/xml.rs:309` | CONFIRMED | introduced | Every `<artwork>` becomes a figure, so an SVG member of an `<artset>` becomes an empty figure next to its ascii-art alternative. | `rfcspec read` on any RFCXML that holds an `<artset>`. RFC 9220 has none. |
| `crates/rfcspec-doc/src/xml.rs:462` | CONFIRMED | introduced | `<br>` is dropped without a separator, so the words either side of it join. | Tables with `<br/>` in cells. Not in RFC 9220. |

Suggested fixes, not applied:
- **Text-only callers:** check `locked.format`, and either bail naming the file or dispatch to `rfcspec_doc::xml::read`.
- **SVG artwork:** skip an artwork whose `type` is `svg`, or one that has element children.
- **`<br>`:** push a space or a line break for it.

**5. Attacked and could not break**
- BCP 14 byte offsets with multibyte text, and after entities.
- CRLF spans.
- CDATA and entities in figures.
- XXE.
- Entity blowup, within roxmltree's limits.
- Lock-format refusal: `format_of` refuses `.html`, an upper-case `.XML` and a bare `.xml` filename.
- Byte-identical text documents: every new field and `format` is `skip_serializing_if`, so text output cannot change. I did not rebuild the base to compare hashes for 868, 1035 and 5905.
- Nested sections and `<references>` context follow the recursion.

Two things I did not settle:
- Whether xml2rfc renders an `<xref>` that has content as the content alone or as content plus `[ref]`. I could not verify it offline.
- `fetch --format xml` over an existing text lock leaves `sources/rfc<N>.txt` behind as an orphan. It is harmless to `read`, so I did not raise it.

**6. Paths written outside the worktree**

`a log file in the worktree trees directory`. It was a suite log written by mistake through a relative `../` path, into a directory nobody assigned me. I deleted it. I also removed my `target/.keeper-hold`.

```findings
- file: crates/rfcspec-propose/src/lib.rs
  line: 207
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a profile whose lock names an .xml source is loaded by Workspace::load (and rfcspec-docs catalog.rs:146) through the plain-text reader over XML markup instead of being refused
- file: crates/rfcspec-doc/src/xml.rs
  line: 309
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: an svg artwork in an artset becomes an empty figure block beside its ascii-art alternative
- file: crates/rfcspec-doc/src/xml.rs
  line: 462
  category: boundary
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: br is dropped without a separator, so the words either side of it in a table cell run together
```
