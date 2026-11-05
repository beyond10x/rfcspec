//! The RFCXML v3 reader over RFC 9220, the XML-path test case (design section 12.1), compared with
//! the text reader over the same RFC.

use std::{fs, path::PathBuf};

use rfcspec_model::{
    Block, BlockKind, Document, Reference, ReferenceCounts, ReferenceKind, SourceFormat, Span,
};

const XML_SHA256: &str = "8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae";
const TEXT_SHA256: &str = "4bac5e1368db199468345dbd2d31ebb4abb7509732b1ae08108cc4860e8967db";

fn fixture(name: &str) -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures")
            .join(name),
    )
    .unwrap()
}

fn xml() -> Document {
    rfcspec_doc::xml::read(9220, XML_SHA256, &fixture("rfc9220.xml")).unwrap()
}

fn text() -> Document {
    rfcspec_doc::read(9220, TEXT_SHA256, &fixture("rfc9220.txt"))
}

fn by_pn<'a>(document: &'a Document, source: &str, pn: &str) -> &'a Block {
    let line = source
        .lines()
        .position(|line| line.contains(&format!("pn=\"{pn}\"")))
        .unwrap_or_else(|| panic!("no element {pn}"))
        + 1;
    document
        .blocks
        .iter()
        .find(|block| block.span.start == line)
        .unwrap_or_else(|| panic!("no block starts at line {line} ({pn})"))
}

fn top_level(block: &Block) -> Option<u32> {
    block.section.as_deref()?.split('.').next()?.parse().ok()
}

fn collapsed(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A text-reader block's text, its lines joined with spaces except after a word the text format
/// hyphenated at a line end (`pseudo-` / `header`), which RFCXML holds unbroken.
fn text_reader_text(block: &Block) -> String {
    let mut joined = String::new();
    for line in block.lines.iter().filter(|line| !line.is_empty()) {
        let broken = joined.ends_with('-')
            && joined[..joined.len() - 1].ends_with(|c: char| c.is_ascii_alphabetic());
        if !joined.is_empty() && !broken {
            joined.push(' ');
        }
        joined.push_str(line);
    }
    collapsed(&joined)
}

#[test]
fn the_fixtures_are_the_files_the_story_names() {
    let size = |name: &str| {
        let bytes = fs::read(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("tests/fixtures")
                .join(name),
        )
        .unwrap();
        bytes.len()
    };
    assert_eq!(size("rfc9220.xml"), 21906);
    assert_eq!(fixture("rfc9220.xml").lines().count(), 331);
    assert_eq!(fixture("rfc9220.txt").lines().count(), 168);
}

#[test]
fn the_document_is_marked_xml_has_no_furniture_and_spans_are_xml_source_lines() {
    let document = xml();
    assert_eq!(document.format, SourceFormat::Xml);
    assert_eq!(document.rfc, 9220);
    assert_eq!(document.sha256, XML_SHA256);
    assert!(document.furniture.is_empty());
    let source = fixture("rfc9220.xml");
    let paragraph = by_pn(&document, &source, "section-3-3");
    assert_eq!(paragraph.kind, BlockKind::Paragraph);
    assert_eq!(
        paragraph.span,
        Span {
            start: 130,
            end: 134
        }
    );
    for block in &document.blocks {
        assert_eq!(
            block.lines.len(),
            block.span.end - block.span.start + 1,
            "{:?}",
            block.span
        );
    }
    let text = rfcspec_doc::read(9220, TEXT_SHA256, &fixture("rfc9220.txt"));
    assert_eq!(text.format, SourceFormat::Text);
}

#[test]
fn section_blocks_carry_their_number_from_pn_and_their_anchor() {
    let document = xml();
    let source = fixture("rfc9220.xml");
    let heading = document
        .blocks
        .iter()
        .find(|block| block.kind == BlockKind::Heading && block.section.as_deref() == Some("3"))
        .unwrap();
    assert_eq!(heading.text(), "3.  WebSockets Upgrade over HTTP/3");
    assert_eq!(
        heading.span,
        Span {
            start: 119,
            end: 119
        }
    );
    for block in document
        .blocks
        .iter()
        .filter(|block| block.section.as_deref() == Some("3"))
    {
        assert_eq!(
            block.anchor.as_deref(),
            Some("websockets-upgrade-over-http3"),
            "{:?}",
            block.span
        );
    }
    let dl = by_pn(&document, &source, "section-5-2");
    assert_eq!(dl.section.as_deref(), Some("5"));
    assert_eq!(dl.anchor.as_deref(), Some("iana-considerations"));
    for anchor in [
        "status-of-memo",
        "copyright",
        "acknowledgments",
        "authors-addresses",
    ] {
        let blocks: Vec<&Block> = document
            .blocks
            .iter()
            .filter(|block| block.anchor.as_deref() == Some(anchor))
            .collect();
        assert!(!blocks.is_empty(), "{anchor}");
        assert!(
            blocks.iter().all(|block| block.section.is_none()),
            "{anchor}"
        );
    }
    let abstract_paragraph = by_pn(&document, &source, "section-abstract-1");
    assert!(abstract_paragraph.anchor.is_some());
    assert_eq!(abstract_paragraph.section, None);
    let references = document
        .blocks
        .iter()
        .find(|block| block.kind == BlockKind::Heading && block.section.as_deref() == Some("6"))
        .unwrap();
    assert_eq!(references.text(), "6.  Normative References");
    let headings: Vec<String> = document
        .blocks
        .iter()
        .filter(|block| block.kind == BlockKind::Heading && block.section.is_some())
        .map(Block::text)
        .collect();
    assert_eq!(
        headings,
        [
            "1.  Introduction",
            "2.  Conventions and Definitions",
            "3.  WebSockets Upgrade over HTTP/3",
            "4.  Security Considerations",
            "5.  IANA Considerations",
            "6.  Normative References",
        ]
    );
    assert!(text().blocks.iter().all(|block| block.anchor.is_none()));
}

#[test]
fn markup_is_removed_and_references_render_as_xml2rfc_renders_them() {
    let document = xml();
    let source = fixture("rfc9220.xml");
    let intro = by_pn(&document, &source, "section-1-1").text();
    assert!(
        intro
            .starts_with("\"Bootstrapping WebSockets with HTTP/2\" [RFC8441] defines an extension"),
        "{intro}"
    );
    assert!(intro.contains("to HTTP/2 [HTTP/2] that"), "{intro}");
    assert!(intro.contains("Appendix A.3 of [HTTP/3]"), "{intro}");
    let section_3_3 = by_pn(&document, &source, "section-3-3").text();
    assert!(
        section_3_3.contains("(Section 15.6.2 of [HTTP])."),
        "{section_3_3}"
    );
    assert!(section_3_3.contains("response [RFC7807]."), "{section_3_3}");
    let status = by_pn(&document, &source, "section-boilerplate.1-3").text();
    assert!(
        status.ends_with("obtained at https://www.rfc-editor.org/info/rfc9220."),
        "{status}"
    );
    let acknowledgments = by_pn(&document, &source, "section-appendix.a-1").text();
    assert!(
        acknowledgments.contains("from David Schinazi, Martin Thomson,"),
        "{acknowledgments}"
    );
    for block in &document.blocks {
        let text = block.text();
        assert!(!text.contains('<'), "{text}");
        assert!(!text.contains("derivedContent"), "{text}");
    }
    let dl = by_pn(&document, &source, "section-5-2");
    assert_eq!(dl.kind, BlockKind::Paragraph);
    let pairs: Vec<&str> = dl
        .lines
        .iter()
        .filter(|line| !line.is_empty())
        .map(String::as_str)
        .collect();
    assert_eq!(
        pairs,
        [
            "Value: 0x08",
            "Setting Name: SETTINGS_ENABLE_CONNECT_PROTOCOL",
            "Default: 0",
            "Status: permanent",
            "Specification: This document",
            "Change Controller: IETF",
            "Contact: HTTP Working Group (ietf-http-wg@w3.org)",
        ]
    );
}

#[test]
fn the_14_blocks_of_sections_1_to_5_equal_the_text_readers() {
    let pick = |document: &Document, text: &dyn Fn(&Block) -> String| {
        document
            .blocks
            .iter()
            .filter(|block| top_level(block).is_some_and(|n| (1..=5).contains(&n)))
            .map(|block| (block.kind, block.section.clone(), text(block)))
            .collect::<Vec<_>>()
    };
    let from_xml = pick(&xml(), &|block| collapsed(&block.text()));
    let from_text = pick(&text(), &text_reader_text);
    assert_eq!(from_text.len(), 14, "{from_text:#?}");
    assert_eq!(from_xml, from_text);
    assert!(
        from_xml
            .iter()
            .all(|(kind, _, _)| matches!(kind, BlockKind::Heading | BlockKind::Paragraph))
    );
}

#[test]
fn the_table_of_contents_and_the_cited_documents_give_no_block() {
    let document = xml();
    let excluded = [
        (65, 95),
        (244, 246),
        (262, 264),
        (279, 281),
        (293, 295),
        (308, 310),
    ];
    for block in &document.blocks {
        for (start, end) in excluded {
            assert!(
                block.span.end < start || block.span.start > end,
                "{:?} overlaps lines {start}-{end}",
                block.span
            );
        }
        let text = block.text();
        for cited in [
            "Table of Contents",
            "Key words for use in RFCs to Indicate Requirement Levels",
            "HTTP Semantics",
            "The WebSocket Protocol enables two-way communication",
            "Problem Details for HTTP APIs",
            "Ambiguity of Uppercase vs Lowercase",
            "Fielding",
        ] {
            assert!(!text.contains(cited), "{cited} in {:?}", block.span);
        }
    }
}

#[test]
fn every_bcp14_mark_is_recorded_with_its_keyword_and_position() {
    let document = xml();
    let source = fixture("rfc9220.xml");
    let keywords = |block: &Block| {
        let text = block.text();
        for mark in &block.bcp14 {
            assert!(
                text[mark.offset..].starts_with(&mark.keyword),
                "{} at {} in {text}",
                mark.keyword,
                mark.offset
            );
        }
        block
            .bcp14
            .iter()
            .map(|mark| mark.keyword.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        keywords(by_pn(&document, &source, "section-2-1")),
        [
            "MUST",
            "MUST NOT",
            "REQUIRED",
            "SHALL",
            "SHALL NOT",
            "SHOULD",
            "SHOULD NOT",
            "RECOMMENDED",
            "NOT RECOMMENDED",
            "MAY",
            "OPTIONAL"
        ]
    );
    let section_3_3 = by_pn(&document, &source, "section-3-3");
    assert_eq!(keywords(section_3_3), ["SHOULD", "MAY"]);
    let rendered = section_3_3.text();
    assert_eq!(
        &rendered[section_3_3.bcp14[0].offset..section_3_3.bcp14[0].offset + 18],
        "SHOULD respond to "
    );
    let all: usize = document
        .blocks
        .iter()
        .map(|block| keywords(block).len())
        .sum();
    assert_eq!(all, 13);
    assert_eq!(source.matches("<bcp14>").count(), 13);
    assert!(text().blocks.iter().all(|block| block.bcp14.is_empty()));
}

#[test]
fn sourcecode_and_artwork_become_figures_with_their_lines_and_type() {
    let source = "<?xml version='1.0' encoding='utf-8'?>
<rfc version=\"3\" number=\"1\">
  <front><title>T</title></front>
  <middle>
    <section anchor=\"grammar\" numbered=\"true\" pn=\"section-1\">
      <name>Grammar</name>
      <sourcecode type=\"abnf\" pn=\"section-1-1\">
rule  = 1*DIGIT
  next = rule
</sourcecode>
      <figure pn=\"figure-1\">
        <name>A box</name>
        <artwork type=\"ascii-art\" pn=\"section-1-2.1\"><![CDATA[
+---+
| a |
+---+
]]></artwork>
      </figure>
    </section>
  </middle>
</rfc>
";
    let document = rfcspec_doc::xml::read(1, "digest", source).unwrap();
    let figures: Vec<&Block> = document
        .blocks
        .iter()
        .filter(|block| block.kind == BlockKind::Figure)
        .collect();
    assert_eq!(figures.len(), 2, "{:#?}", document.blocks);
    assert_eq!(figures[0].figure_type.as_deref(), Some("abnf"));
    assert_eq!(figures[0].span, Span { start: 7, end: 10 });
    assert_eq!(
        figures[0].lines,
        ["", "rule  = 1*DIGIT", "  next = rule", ""]
    );
    assert_eq!(figures[1].figure_type.as_deref(), Some("ascii-art"));
    assert_eq!(figures[1].span, Span { start: 13, end: 17 });
    assert_eq!(figures[1].lines, ["", "+---+", "| a |", "+---+", ""]);
    for figure in figures {
        assert_eq!(figure.section.as_deref(), Some("1"));
        assert_eq!(figure.anchor.as_deref(), Some("grammar"));
    }
    assert!(
        document
            .blocks
            .iter()
            .all(|block| !block.text().contains("A box")),
        "{:#?}",
        document.blocks
    );
    assert!(
        text()
            .blocks
            .iter()
            .all(|block| block.figure_type.is_none())
    );
}

#[test]
fn a_document_that_is_not_rfcxml_v3_is_refused() {
    assert!(rfcspec_doc::xml::read(1, "d", "<rfc version=\"2\"/>").is_err());
    assert!(rfcspec_doc::xml::read(1, "d", "<html/>").is_err());
    assert!(rfcspec_doc::xml::read(1, "d", "not xml").is_err());
}

#[test]
fn every_xref_to_another_rfc_is_a_reference_and_the_others_are_counted() {
    let document = xml();
    assert_eq!(fixture("rfc9220.xml").matches("<xref").count(), 35);
    assert_eq!(
        document.reference_counts,
        Some(ReferenceCounts {
            self_references: 16,
            non_rfc: 0,
            unresolved: 0
        })
    );
    let normative = |section: &str, target_rfc: u32, target_section: Option<&str>| Reference {
        section: Some(section.to_string()),
        target_rfc,
        target_section: target_section.map(String::from),
        kind: ReferenceKind::Normative,
    };
    let introduction = "introduction";
    let conventions = "conventions-and-definitions";
    let upgrade = "websockets-upgrade-over-http3";
    assert_eq!(
        document.references,
        [
            normative(introduction, 8441, None),
            normative(introduction, 8441, None),
            normative(introduction, 9113, None),
            normative(introduction, 9114, None),
            normative(introduction, 9114, Some("A.3")),
            normative(conventions, 2119, None),
            normative(conventions, 8174, None),
            normative(upgrade, 8441, None),
            normative(upgrade, 6455, None),
            normative(upgrade, 8441, None),
            normative(upgrade, 9114, Some("A.3")),
            normative(upgrade, 9110, Some("15.6.2")),
            normative(upgrade, 7807, None),
            normative(upgrade, 6455, None),
            normative(upgrade, 9114, Some("4.4")),
            normative(upgrade, 9114, Some("8")),
            normative(upgrade, 9114, Some("8.1")),
            normative("security-considerations", 8441, None),
            normative("iana-considerations", 9114, Some("11.2.2")),
        ]
    );
}

#[test]
fn a_text_document_has_no_references_and_no_reference_counts() {
    let document = text();
    assert!(document.references.is_empty());
    assert_eq!(document.reference_counts, None);
}
