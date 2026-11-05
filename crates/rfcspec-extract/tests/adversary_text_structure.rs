//! Adversarial cases for story:text-reader-structure: layouts from the RFC corpus that the
//! text reader's new list and table-of-contents rules mishandle. Each text is a short excerpt
//! laid out as the cited RFC lays it out.

use rfcspec_extract::extract;

fn requirements(text: &str) -> Vec<(String, String, String)> {
    extract(&rfcspec_doc::read(1, "digest", text))
        .requirements
        .into_iter()
        .map(|r| (r.id, r.keyword, r.text))
        .collect()
}

/// RFC 3995 lines 2537-2539 (and RFC 2639 line 497, RFC 3996 line 904): a `*` item whose
/// wrapped lines sit at the mark's column. The sentence runs on with a lower-case word after
/// a line without closing punctuation, so it is one requirement, as the reader gave it before
/// this story; the new rule cuts it after the first line and drops its condition.
#[test]
fn a_list_item_wrapped_at_its_marks_column_keeps_its_sentence_whole() {
    let text = "\
1.  Introduction

   *  The Printer MUST deliver the \"job-impressions-completed\" attribute
   in an Event Notification only for the combinations of Events and
   Subscribed Events shown in Table 7.
";
    assert_eq!(
        requirements(text),
        [(
            "R1-1".to_owned(),
            "MUST".to_owned(),
            "*  The Printer MUST deliver the \"job-impressions-completed\" attribute in an \
             Event Notification only for the combinations of Events and Subscribed Events \
             shown in Table 7."
                .to_owned()
        )]
    );
}

/// RFC 2848 lines 40-146: a table of contents one of whose wrapped titles ends in an
/// abbreviation ("I.N."). One such line makes the whole table a paragraph, and its entry
/// "3.3. REQUIRED and OPTIONAL elements for PINT compliance" becomes requirement `R-1`.
#[test]
#[ignore = "story:text-reader-toc-coverage"]
fn a_table_of_contents_with_a_title_ending_in_an_abbreviation_gives_no_requirement() {
    let text = "\
Table of Contents

   1. Introduction .................................................  4
   2. PINT Milestone Services ......................................  6
   3.3. REQUIRED and OPTIONAL elements for PINT compliance ......... 11
   6. Deployment considerations and the Relationship PINT to I.N.
      ........................................................... 40

1.  Introduction

   A client MUST send the request.
";
    let found = requirements(text);
    assert_eq!(
        found
            .iter()
            .map(|(id, level, _)| (id.as_str(), level.as_str()))
            .collect::<Vec<_>>(),
        [("R1-1", "MUST")],
        "{found:#?}"
    );
}

/// The table-of-contents rule applies only before the first section heading. No case of the
/// suite checks that: with the `sections[start].is_none()` guard of `rfcspec_doc::read` removed,
/// every `rfcspec-doc` and `rfcspec-extract` test stays green. A numbered list without closing
/// punctuation in a section's body is a paragraph and gives its requirements.
#[test]
fn a_numbered_list_without_closing_punctuation_inside_a_section_is_no_table_of_contents() {
    let text = "\
1.  Introduction

   The exchange runs in two steps:

   1.  The client MUST send a ping
   2.  The server MUST answer with a pong
";
    assert_eq!(
        requirements(text)
            .into_iter()
            .map(|(id, level, _)| (id, level))
            .collect::<Vec<_>>(),
        [
            ("R1-1".to_owned(), "MUST".to_owned()),
            ("R1-2".to_owned(), "MUST".to_owned())
        ]
    );
}
