//! The plain-text reader's structure as the catalogue sees it (story:text-reader-structure):
//! appendix headings, `*` list items, `|` asides and tables of contents, each over a small text
//! laid out as an RFC renders it.

use rfcspec_extract::extract;
use rfcspec_model::BlockKind;

fn ids(text: &str) -> Vec<(String, String)> {
    extract(&rfcspec_doc::read(1, "digest", text))
        .requirements
        .into_iter()
        .map(|r| (r.id, r.keyword))
        .collect()
}

fn pairs(expected: &[(&str, &str)]) -> Vec<(String, String)> {
    expected
        .iter()
        .map(|(id, level)| ((*id).to_owned(), (*level).to_owned()))
        .collect()
}

#[test]
fn an_appendix_heading_starts_its_own_section_and_its_subsections_follow() {
    let text = "\
1.  Introduction

   A client MUST send the request
   at once.

Appendix A.  Code

   The listing MUST NOT be read as
   normative text.

A.1.  Helpers

   A helper SHOULD be
   small.
";
    assert_eq!(
        ids(text),
        pairs(&[("R1-1", "MUST"), ("RA-1", "MUST NOT"), ("RA.1-1", "SHOULD")])
    );
}

#[test]
fn a_capital_letter_line_without_the_appendix_prefix_or_a_numeric_part_is_no_heading() {
    assert_eq!(rfcspec_doc::section_heading("A.  Title"), None);
    assert_eq!(rfcspec_doc::section_heading("AB.1.  Title"), None);
    assert_eq!(rfcspec_doc::section_heading("   A.1.  Title"), None);
    assert_eq!(
        rfcspec_doc::section_heading("Appendix A.  Title"),
        Some("A")
    );
    assert_eq!(
        rfcspec_doc::section_heading("Appendix B.2.  Title"),
        Some("B.2")
    );
    assert_eq!(
        rfcspec_doc::section_heading("C.10.3.  Title"),
        Some("C.10.3")
    );
    assert_eq!(rfcspec_doc::section_heading("4.2.  Title"), Some("4.2"));
}

#[test]
fn each_bullet_item_is_its_own_paragraph_and_so_is_the_text_after_the_list() {
    let text = "\
1.  Introduction

   The following measures apply:

   *  Bound tokens SHOULD be used to prevent replay.  If the logs
      leak, bound tokens may also prevent replay on the system
   *  Audience restriction SHOULD be used to prevent replay on
      other servers.
   *  The server MUST treat tokens as secrets.
   Text after the list MAY follow without a blank line.
";
    assert_eq!(
        ids(text),
        pairs(&[
            ("R1-1", "SHOULD"),
            ("R1-2", "SHOULD"),
            ("R1-3", "MUST"),
            ("R1-4", "MAY")
        ])
    );
    let document = rfcspec_doc::read(1, "digest", text);
    let paragraphs: Vec<(usize, usize)> = document
        .blocks
        .iter()
        .filter(|block| block.kind == BlockKind::Paragraph)
        .map(|block| (block.span.start, block.span.end))
        .collect();
    assert_eq!(paragraphs, [(3, 3), (5, 6), (7, 8), (9, 9), (10, 10)]);
}

#[test]
fn an_aside_is_read_without_its_bars_and_a_bar_alone_ends_its_paragraph() {
    let text = "\
1.  Introduction

   The earlier specification states:

   |  Each medium SHOULD be carried in its own session.
   |
   |  Two media SHOULD NOT share one session.  A source MAY change
   |  types.
   |  A sender MAY stop.
";
    assert_eq!(
        ids(text),
        pairs(&[
            ("R1-1", "SHOULD"),
            ("R1-2", "SHOULD NOT"),
            ("R1-3", "MAY"),
            ("R1-4", "MAY")
        ])
    );
    let document = rfcspec_doc::read(1, "digest", text);
    let aside = document
        .blocks
        .iter()
        .find(|block| block.span.start == 7)
        .unwrap();
    assert_eq!(aside.kind, BlockKind::Paragraph);
    assert_eq!(aside.span.end, 9);
    assert_eq!(
        aside.lines[0],
        "Two media SHOULD NOT share one session.  A source MAY change"
    );
}

#[test]
fn a_table_of_contents_is_no_paragraph_but_a_numbered_list_before_section_one_is() {
    let text = "\
   Clients MUST read
   this.

Table of Contents

   1.  Introduction
   2.  File Attributes
     2.1.  REQUIRED Attributes
     2.2.  RECOMMENDED Attributes
   Appendix A.  OPTIONAL Extras

   1.  Clients MUST send the request.
   2.  Servers MUST answer it.

1.  Introduction

   A client MUST send the request
   at once.
";
    assert_eq!(
        ids(text),
        pairs(&[
            ("R-1", "MUST"),
            ("R-2", "MUST"),
            ("R-3", "MUST"),
            ("R1-1", "MUST")
        ])
    );
}

#[test]
fn a_one_line_list_item_without_closing_punctuation_is_a_paragraph_not_a_heading() {
    let text = "\
1.  Introduction

   A profile MUST pick one of the following two
   options:

   *  The tile size MUST be the regular size, or

   *  the tile size MUST be one word smaller than the regular
      size.
";
    assert_eq!(
        ids(text),
        pairs(&[("R1-1", "MUST"), ("R1-2", "MUST"), ("R1-3", "MUST")])
    );
}

#[test]
fn nested_list_items_marked_with_a_dash_o_or_plus_are_paragraphs_too() {
    let text = "\
1.  Introduction

   On receiving an acknowledgement, the sender acts
   as follows:

   *  if the window is the last one:

      -  if the bit is set, the sender MAY exit successfully.

      -  otherwise:

         o  the sender MUST send the last tile

         +  the receiver SHOULD wait

   A dash in running text - such as this one - MUST NOT start an item.
";
    assert_eq!(
        ids(text),
        pairs(&[
            ("R1-1", "MAY"),
            ("R1-2", "MUST"),
            ("R1-3", "SHOULD"),
            ("R1-4", "MUST NOT")
        ])
    );
}
