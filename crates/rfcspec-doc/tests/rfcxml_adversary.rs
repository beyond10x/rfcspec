//! Adversarial cases for the RFCXML v3 reader: markup the RFC 9220 fixture does not hold.

use rfcspec_model::{Block, BlockKind, Document};

fn read(body: &str) -> Document {
    let source = format!(
        "<?xml version='1.0' encoding='utf-8'?>\n<rfc version=\"3\" number=\"1\">\n<front><title>T</title></front>\n<middle>\n<section anchor=\"s\" numbered=\"true\" pn=\"section-1\">\n<name>S</name>\n{body}\n</section>\n</middle>\n</rfc>\n"
    );
    rfcspec_doc::xml::read(1, "digest", &source).unwrap()
}

fn assert_marks_point_at_their_keywords(document: &Document) {
    for block in &document.blocks {
        let text = block.text();
        for mark in &block.bcp14 {
            assert!(
                text.get(mark.offset..)
                    .is_some_and(|rest| rest.starts_with(&mark.keyword)),
                "mark {mark:?} does not point at its keyword in {text:?}"
            );
        }
    }
}

#[test]
fn bcp14_marks_in_em_li_dd_and_table_cells_point_at_their_keywords_with_non_ascii_text() {
    let document = read(
        "<t>Él <em>façade <bcp14>MUST</bcp14></em> go.</t>
<ul><li>Ünï <bcp14>SHOULD</bcp14> x</li><li><t>ä <bcp14>MAY</bcp14></t></li></ul>
<dl><dt>Tërm:</dt><dd>çé
  <bcp14>MUST NOT</bcp14> y</dd></dl>
<table><thead><tr><th>Ä</th></tr></thead><tbody><tr><td>ö <bcp14>REQUIRED</bcp14></td></tr></tbody></table>",
    );
    let marks: Vec<&str> = document
        .blocks
        .iter()
        .flat_map(|block| block.bcp14.iter().map(|mark| mark.keyword.as_str()))
        .collect();
    assert_eq!(marks, ["MUST", "SHOULD", "MAY", "MUST NOT", "REQUIRED"]);
    assert_marks_point_at_their_keywords(&document);
}

#[test]
fn crlf_line_ends_give_the_same_spans_lines_and_marks_as_lf() {
    let body = "<t>The client\n  <bcp14>MUST</bcp14> close.</t>\n<sourcecode type=\"abnf\"><![CDATA[\na = \"<\" b\n]]></sourcecode>";
    let source = format!(
        "<?xml version='1.0' encoding='utf-8'?>\n<rfc version=\"3\" number=\"1\">\n<middle>\n<section anchor=\"s\" pn=\"section-1\">\n<name>S</name>\n{body}\n</section>\n</middle>\n</rfc>\n"
    );
    let lf = rfcspec_doc::xml::read(1, "d", &source).unwrap();
    let crlf = rfcspec_doc::xml::read(1, "d", &source.replace('\n', "\r\n")).unwrap();
    assert_eq!(crlf.blocks, lf.blocks);
    let figure = lf
        .blocks
        .iter()
        .find(|block| block.kind == BlockKind::Figure)
        .unwrap();
    assert_eq!(figure.lines, ["", "a = \"<\" b", ""]);
}

#[test]
fn entities_and_cdata_in_sourcecode_are_kept_verbatim() {
    let document = read(
        "<sourcecode type=\"c\">if (a &lt; b &amp;&amp; c) {}\n<![CDATA[x = y < z && w;]]>\n</sourcecode>",
    );
    let figure = document
        .blocks
        .iter()
        .find(|block| block.kind == BlockKind::Figure)
        .unwrap();
    assert_eq!(figure.lines, ["if (a < b && c) {}", "x = y < z && w;", ""]);
}

/// xml2rfc renders an `<artset>` in text through its ascii-art member; an SVG artwork has no
/// text lines, so it must not become a figure of empty lines.
#[test]
fn an_artset_gives_one_figure_its_ascii_art_and_no_empty_svg_figure() {
    let document = read(
        "<figure><name>F</name><artset>
<artwork type=\"svg\"><svg xmlns=\"http://www.w3.org/2000/svg\">
<text>a</text>
</svg></artwork>
<artwork type=\"ascii-art\"><![CDATA[
+---+
]]></artwork>
</artset></figure>",
    );
    let figures: Vec<&Block> = document
        .blocks
        .iter()
        .filter(|block| block.kind == BlockKind::Figure)
        .collect();
    assert_eq!(figures.len(), 1, "{figures:#?}");
    assert_eq!(figures[0].figure_type.as_deref(), Some("ascii-art"));
}

/// `<br/>` in a table cell is a line break in xml2rfc's text output; the words either side of
/// it must not run together.
#[test]
fn a_br_in_a_table_cell_does_not_join_the_words_either_side() {
    let document = read("<table><tbody><tr><td>first<br/>second</td></tr></tbody></table>");
    let cell = document
        .blocks
        .iter()
        .find(|block| block.text().contains("first"))
        .unwrap();
    assert_eq!(
        cell.text().split_whitespace().collect::<Vec<_>>(),
        ["first", "second"]
    );
}

#[test]
fn an_external_entity_is_not_resolved() {
    let target = concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml");
    let source = format!(
        "<?xml version='1.0'?>\n<!DOCTYPE rfc [<!ENTITY x SYSTEM \"file://{target}\">]>\n<rfc version=\"3\">\n<middle>\n<section pn=\"section-1\">\n<name>S</name>\n<t>a &x; b</t>\n</section>\n</middle>\n</rfc>\n"
    );
    if let Ok(document) = rfcspec_doc::xml::read(1, "d", &source) {
        assert!(
            document
                .blocks
                .iter()
                .all(|block| !block.text().contains("roxmltree")),
            "{:#?}",
            document.blocks
        );
    }
}

/// Six levels of tenfold entity expansion: a 600-byte source that expands to 3 MB of text. A
/// locked source is bounded by its fetch limit; its expansion must not be unbounded.
#[test]
fn nested_entity_expansion_is_refused_or_bounded() {
    let mut dtd = String::from("<!ENTITY l0 \"lol\">");
    for level in 1..=6 {
        let previous = format!("&l{};", level - 1);
        dtd.push_str(&format!("<!ENTITY l{level} \"{}\">", previous.repeat(10)));
    }
    let source = format!(
        "<?xml version='1.0'?>\n<!DOCTYPE rfc [{dtd}]>\n<rfc version=\"3\">\n<middle>\n<section pn=\"section-1\">\n<name>S</name>\n<t>&l6;</t>\n</section>\n</middle>\n</rfc>\n"
    );
    if let Ok(document) = rfcspec_doc::xml::read(1, "d", &source) {
        let text: usize = document.blocks.iter().map(|block| block.text().len()).sum();
        assert!(
            text <= 100 * source.len(),
            "a {}-byte source expanded to {text} bytes of block text",
            source.len()
        );
    }
}
