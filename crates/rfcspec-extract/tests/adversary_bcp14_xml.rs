//! Adversarial cases on extraction from `<bcp14>` markup (story:rfcxml-bcp14-extract) and on the
//! overlay check over an XML-locked catalogue.

use std::{fs, path::PathBuf};

use rfcspec_extract::{check, extract_reporting};
use rfcspec_model::{ABNF_COMMENT, BCP14_XML, Catalogue, Document, Overlay, from_yaml};

/// An RFCXML v3 document whose section 1 holds `body`, starting on source line 5.
fn xml_document(body: &str) -> Document {
    let text = format!(
        "<rfc version=\"3\">\n<middle>\n<section pn=\"section-1\">\n<name>Intro</name>\n{body}\n</section>\n</middle>\n</rfc>\n"
    );
    rfcspec_doc::xml::read(1, "digest", &text).unwrap()
}

fn levels(catalogue: &Catalogue) -> Vec<(&str, &str, &str)> {
    catalogue
        .requirements
        .iter()
        .map(|r| (r.id.as_str(), r.keyword.as_str(), r.source.as_str()))
        .collect()
}

/// The figure_type clause of the ABNF branch is not exercised by the unit's own ABNF test: its
/// only line is itself an ABNF rule, so the old line heuristic already takes the branch.
#[test]
fn the_units_own_abnf_test_line_is_a_rule_line_so_it_cannot_see_the_type_clause() {
    assert!(rfcspec_doc::abnf_rule("frame = %x00-FF ; MUST be zero"));
}

/// A `type="abnf"` figure that holds no rule-start line (a continuation of a rule defined in an
/// earlier figure) is ABNF by its type alone; its comment is an `abnf-comment` requirement.
#[test]
fn a_type_abnf_figure_without_a_rule_start_line_gives_its_comment_as_abnf_comment() {
    let (catalogue, problems) = extract_reporting(&xml_document(
        "<sourcecode type=\"abnf\">\n         / %x01 ; MUST be one\n</sourcecode>",
    ));
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(levels(&catalogue), [("R1-1", "MUST", ABNF_COMMENT)]);
    assert_eq!(catalogue.requirements[0].text, "MUST be one");
}

/// Two definitions of a `<dl>`, each one sentence with its own `<bcp14>`, are two requirements.
/// Shape of RFC 9742 section 8 (XML lines 1555 to 1565): the second term starts in lower case,
/// so the definition list read as one paragraph runs both definitions into one sentence, and the
/// `MUST NOT` of the second is lost behind the `MUST` of the first.
#[test]
fn two_definitions_of_a_dl_each_with_bcp14_are_two_requirements() {
    let (catalogue, problems) = extract_reporting(&xml_document(
        "<dl>\n<dt>facility-filter/pattern-match:</dt>\n<dd>When writing this node, implementations <bcp14>MUST</bcp14> ensure the pattern is safe.</dd>\n<dt>remote/destination/signing/cert-signer:</dt>\n<dd>When writing this subtree, implementations <bcp14>MUST NOT</bcp14> reuse a key.</dd>\n</dl>",
    ));
    assert!(problems.is_empty(), "{problems:?}");
    assert_eq!(
        levels(&catalogue),
        [("R1-1", "MUST", BCP14_XML), ("R1-2", "MUST NOT", BCP14_XML)],
        "{:?}",
        catalogue
            .requirements
            .iter()
            .map(|r| r.text.as_str())
            .collect::<Vec<_>>()
    );
}

/// A sentence whose first `<bcp14>` is a level and whose second is not gives a requirement at the
/// first; the problem about the second must not claim the sentence gives none.
#[test]
fn a_non_level_second_element_does_not_report_that_its_sentence_gives_no_requirement() {
    let (catalogue, problems) = extract_reporting(&xml_document(
        "<t>The client <bcp14>MUST</bcp14> send and <bcp14>must</bcp14> stop.</t>",
    ));
    assert_eq!(levels(&catalogue), [("R1-1", "MUST", BCP14_XML)]);
    assert_eq!(problems.len(), 1, "{problems:?}");
    assert!(
        !problems[0].contains("its sentence gives no requirement"),
        "the problem says the sentence gives no requirement, but the catalogue holds {:?}: {}",
        levels(&catalogue),
        problems[0]
    );
}

fn source_9220_xml() -> String {
    fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rfcspec-doc/tests/fixtures/rfc9220.xml"),
    )
    .unwrap()
}

fn catalogue_9220_xml() -> Catalogue {
    let (catalogue, problems) =
        extract_reporting(&rfcspec_doc::xml::read(9220, "digest", &source_9220_xml()).unwrap());
    assert!(problems.is_empty(), "{problems:?}");
    catalogue
}

fn overlay_with(requirement: &str) -> Overlay {
    from_yaml(&format!(
        "format: rfcspec-overlay/1\nrfc: 9220\nrequirements:\n{requirement}layouts: []\nmachines: []\nfindings: []\n"
    ))
    .unwrap()
}

/// An overlay requirement with `source: bcp14-xml` names a catalogue id that does not exist.
/// The same entry with `source: capitals-text` is refused; for the source the XML catalogue now
/// emits, nothing ties the entry to the catalogue.
#[test]
#[ignore = "story:xml-overlay-check"]
fn check_refuses_a_bcp14_xml_overlay_requirement_whose_id_is_no_catalogue_requirement() {
    let catalogue = catalogue_9220_xml();
    assert!(catalogue.requirements.iter().all(|r| r.id != "R3-9"));
    let entry = |source: &str| {
        format!(
            "- id: R3-9\n  source: {source}\n  level: MUST\n  transports: []\n  quote: A server\n  span:\n    start: 133\n    end: 133\n  status:\n    unsupported: probe\n"
        )
    };
    let as_capitals = check(
        &overlay_with(&entry("capitals-text")),
        &catalogue,
        &source_9220_xml(),
    );
    assert!(
        as_capitals
            .iter()
            .any(|p| p.contains("no catalogue requirement")),
        "{as_capitals:?}"
    );
    let as_xml = check(
        &overlay_with(&entry("bcp14-xml")),
        &catalogue,
        &source_9220_xml(),
    );
    assert!(
        !as_xml.is_empty(),
        "an overlay requirement R3-9 (bcp14-xml) passes the check against a catalogue without R3-9"
    );
}

/// An overlay requirement `R3-2` (`bcp14-xml`) whose span lies outside the catalogue span of
/// `R3-2` (lines 133 to 134) passes, where the same entry as `capitals-text` is refused.
#[test]
#[ignore = "story:xml-overlay-check"]
fn check_refuses_a_bcp14_xml_overlay_requirement_outside_its_catalogue_span() {
    let catalogue = catalogue_9220_xml();
    let r3_2 = catalogue
        .requirements
        .iter()
        .find(|r| r.id == "R3-2")
        .unwrap();
    assert_eq!((r3_2.span.start, r3_2.span.end), (133, 134));
    let entry = |source: &str| {
        format!(
            "- id: R3-2\n  source: {source}\n  level: MAY\n  transports: []\n  quote: HTTP/3 settings be registered separately for HTTP/3.\n  span:\n    start: 128\n    end: 128\n  status:\n    unsupported: probe\n"
        )
    };
    let as_capitals = check(
        &overlay_with(&entry("capitals-text")),
        &catalogue,
        &source_9220_xml(),
    );
    assert!(
        as_capitals
            .iter()
            .any(|p| p.contains("does not lie within")),
        "{as_capitals:?}"
    );
    let as_xml = check(
        &overlay_with(&entry("bcp14-xml")),
        &catalogue,
        &source_9220_xml(),
    );
    assert!(
        !as_xml.is_empty(),
        "R3-2 (bcp14-xml) quoting line 128 passes, outside the catalogue span 133 to 134"
    );
}

/// The faithful overlay entry for `R3-2`, quoting its catalogue text at its catalogue span, as
/// `rfcspec propose` and `review accept` would write it from this catalogue.
#[test]
#[ignore = "story:xml-overlay-check"]
fn check_accepts_the_catalogue_text_of_r3_2_quoted_at_its_catalogue_span() {
    let catalogue = catalogue_9220_xml();
    let r3_2 = catalogue
        .requirements
        .iter()
        .find(|r| r.id == "R3-2")
        .unwrap();
    let entry = format!(
        "- id: R3-2\n  source: bcp14-xml\n  level: MAY\n  transports: []\n  quote: '{}'\n  span:\n    start: {}\n    end: {}\n  status:\n    unsupported: probe\n",
        r3_2.text.replace('\'', "''"),
        r3_2.span.start,
        r3_2.span.end
    );
    let problems = check(&overlay_with(&entry), &catalogue, &source_9220_xml());
    assert_eq!(problems, Vec::<String>::new());
}
