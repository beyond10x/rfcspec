//! Adversarial cases for xref resolution: markup the RFC 9220 fixture does not hold, each taken
//! from a shape the RFCXML corpus holds.

use std::collections::{BTreeMap, BTreeSet};

use rfcspec_model::{Document, Reference, ReferenceCounts, ReferenceKind};

fn read(rfc: u32, middle: &str, back: &str) -> Document {
    let source = format!(
        "<?xml version='1.0' encoding='utf-8'?>\n<rfc version=\"3\" number=\"{rfc}\">\n<front><title>T</title>\n<abstract pn=\"section-abstract\"><t>A</t></abstract></front>\n<middle>\n<section anchor=\"s\" pn=\"section-1\">\n<name>S</name>\n{middle}\n</section>\n</middle>\n<back>\n{back}\n</back>\n</rfc>\n"
    );
    rfcspec_doc::xml::read(rfc, "d", &source).unwrap()
}

fn cites(section: &str, target_rfc: u32, kind: ReferenceKind) -> Reference {
    Reference {
        section: Some(section.to_string()),
        target_rfc,
        target_section: None,
        kind,
    }
}

/// RFC 9786 lists RFC 9785 as `<seriesInfo name="RFC" value="RFC9785"/>` beside its DOI
/// `10.17487/RFC9785`; its two xrefs to it cite an RFC, not a non-RFC document.
#[test]
fn an_rfc_series_info_written_with_its_prefix_still_names_the_rfc() {
    let document = read(
        9786,
        "<t>See <xref target=\"RFC9785\"/>.</t>",
        "<references pn=\"section-2\"><name>Normative References</name>
<reference anchor=\"RFC9785\" target=\"https://www.rfc-editor.org/info/rfc9785\"><front><title>X</title></front>
<seriesInfo name=\"RFC\" value=\"RFC9785\"/><seriesInfo name=\"DOI\" value=\"10.17487/RFC9785\"/></reference>
</references>",
    );
    assert_eq!(
        document.references,
        [cites("s", 9785, ReferenceKind::Normative)]
    );
    assert_eq!(
        document.reference_counts,
        Some(ReferenceCounts {
            self_references: 0,
            non_rfc: 0,
            unresolved: 0
        })
    );
}

/// `STD94` is a `<referencegroup>` whose one member is RFC 8949 (as 8 corpus RFCs list it). An
/// xref to it cites RFC 8949; counting it as a non-RFC document drops the edge.
#[test]
fn an_xref_to_a_reference_group_of_one_rfc_cites_that_rfc() {
    let document = read(
        9000,
        "<t>Encoded in <xref target=\"STD94\"/>.</t>",
        "<references pn=\"section-2\"><name>Normative References</name>
<referencegroup anchor=\"STD94\" target=\"https://www.rfc-editor.org/info/std94\">
<reference anchor=\"RFC8949\"><front><title>CBOR</title></front>
<seriesInfo name=\"STD\" value=\"94\"/><seriesInfo name=\"RFC\" value=\"8949\"/></reference>
</referencegroup>
</references>",
    );
    assert_eq!(
        document.references,
        [cites("s", 8949, ReferenceKind::Normative)]
    );
    assert_eq!(
        document.reference_counts,
        Some(ReferenceCounts {
            self_references: 0,
            non_rfc: 0,
            unresolved: 0
        })
    );
}

/// An xref whose target names no anchor in the document is a broken source, not a citation of
/// the citing RFC; counting it as a self-reference makes the error invisible.
#[test]
fn an_xref_to_an_anchor_that_does_not_exist_is_not_a_self_reference() {
    let document = read(9000, "<t>See <xref target=\"nowhere\"/>.</t>", "");
    assert_eq!(
        document.reference_counts,
        Some(ReferenceCounts {
            self_references: 0,
            non_rfc: 0,
            unresolved: 1
        })
    );
}

/// The default and the `non-normative` branch of the kind: no group names a kind, and a group
/// named "Non-Normative References". The existing suite names every group "Normative" or
/// "Informative", so flipping the default or dropping the `non-normative` test stays green.
#[test]
fn the_kind_defaults_to_informative_and_non_normative_is_informative() {
    let document = read(
        9000,
        "<t><xref target=\"RFC1\"/> <xref target=\"RFC2\"/></t>",
        "<references pn=\"section-2\"><name>References</name>
<reference anchor=\"RFC1\"><front><title>One</title></front><seriesInfo name=\"RFC\" value=\"1\"/></reference>
</references>
<references pn=\"section-3\"><name>Non-Normative References</name>
<reference anchor=\"RFC2\"><front><title>Two</title></front><seriesInfo name=\"RFC\" value=\"2\"/></reference>
</references>",
    );
    assert_eq!(
        document.references,
        [
            cites("s", 1, ReferenceKind::Informative),
            cites("s", 2, ReferenceKind::Informative),
        ]
    );
}

/// Xrefs inside a figure's name, a table cell, the abstract and a nested section take the
/// innermost section-like anchor.
#[test]
fn xrefs_in_a_figure_a_table_and_the_abstract_take_their_section() {
    let source = "<?xml version='1.0' encoding='utf-8'?>
<rfc version=\"3\" number=\"9000\">
<front><title>T</title><abstract pn=\"section-abstract\"><t>Per <xref target=\"RFC1\"/>.</t></abstract></front>
<middle>
<section anchor=\"s\" pn=\"section-1\"><name>S</name>
<figure anchor=\"f\"><name>From <xref target=\"RFC1\"/></name><artwork>x</artwork></figure>
<section pn=\"section-1.1\"><name>Sub</name>
<table><tbody><tr><td><xref target=\"RFC1\" section=\"3\"/></td></tr></tbody></table>
</section>
</section>
</middle>
<back><references pn=\"section-2\"><name>Normative References</name>
<reference anchor=\"RFC1\"><front><title>One</title></front><seriesInfo name=\"RFC\" value=\"1\"/></reference>
</references></back>
</rfc>
";
    let document = rfcspec_doc::xml::read(9000, "d", source).unwrap();
    let sections: Vec<Option<&str>> = document
        .references
        .iter()
        .map(|reference| reference.section.as_deref())
        .collect();
    assert_eq!(
        sections,
        [Some("section-abstract"), Some("s"), Some("section-1.1")]
    );
}

/// The RFC an RFCXML `<reference>` names, by any of the identifiers the corpus uses for one: a
/// `seriesInfo name="RFC"` with or without the `RFC` prefix, an RFC DOI, or the RFC Editor's
/// info URL.
fn rfc_of(reference: roxmltree::Node) -> Option<u32> {
    let digits = |text: &str| -> Option<u32> {
        let rest = text
            .trim()
            .trim_start_matches("RFC")
            .trim_start_matches("rfc");
        (!rest.is_empty() && rest.bytes().all(|b| b.is_ascii_digit()))
            .then(|| rest.parse().ok())
            .flatten()
    };
    for info in reference
        .children()
        .filter(|n| n.tag_name().name() == "seriesInfo")
    {
        let value = info.attribute("value").unwrap_or_default();
        match info.attribute("name") {
            Some("RFC") => return digits(value),
            Some("DOI") => {
                if let Some(rest) = value.strip_prefix("10.17487/")
                    && let Some(number) = digits(rest)
                {
                    return Some(number);
                }
            }
            _ => {}
        }
    }
    reference
        .attribute("target")
        .and_then(|url| url.strip_prefix("https://www.rfc-editor.org/info/"))
        .and_then(digits)
}

/// Read every RFCXML file of the corpus named by `RFCSPEC_XML_CORPUS` (a directory of
/// `rfc<N>.xml`) and print what the reader made of its xrefs beside an independent count. Run
/// with `--ignored --nocapture`; it reads no network and writes nothing.
#[test]
#[ignore = "reads a corpus outside the repository"]
fn corpus_probe() {
    let Ok(dir) = std::env::var("RFCSPEC_XML_CORPUS") else {
        return;
    };
    let mut files: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .collect();
    files.sort();
    let mut stats: BTreeMap<&str, usize> = BTreeMap::new();
    let mut zero_examples = Vec::new();
    let mut unresolved_examples = Vec::new();
    let mut default_files = BTreeSet::new();
    for path in &files {
        let stem = path.file_stem().unwrap().to_string_lossy().to_string();
        let Some(rfc) = stem.strip_prefix("rfc").and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let text = std::fs::read_to_string(path).unwrap();
        let document = match rfcspec_doc::xml::read(rfc, "d", &text) {
            Ok(document) => document,
            Err(_) => {
                *stats.entry("files refused").or_default() += 1;
                continue;
            }
        };
        *stats.entry("files read").or_default() += 1;
        let xml = roxmltree::Document::parse_with_options(
            &text,
            roxmltree::ParsingOptions {
                allow_dtd: true,
                ..Default::default()
            },
        )
        .unwrap();
        let root = xml.root_element();
        let mut anchors = BTreeSet::new();
        let mut targets: BTreeMap<&str, roxmltree::Node> = BTreeMap::new();
        for node in root.descendants() {
            for attr in ["anchor", "pn", "slugifiedName"] {
                if let Some(value) = node.attribute(attr) {
                    anchors.insert(value);
                }
            }
            if let Some(anchor) = node.attribute("anchor")
                && matches!(node.tag_name().name(), "reference" | "referencegroup")
            {
                targets.insert(anchor, node);
            }
        }
        let mut expected_rfc_xrefs = 0;
        let mut to_group = 0;
        let mut unresolved = 0;
        let mut prefixed = 0;
        for xref in root.descendants().filter(|n| n.tag_name().name() == "xref") {
            let target = xref.attribute("target").unwrap_or_default();
            match targets.get(target) {
                Some(node) if node.tag_name().name() == "reference" => {
                    if let Some(number) = rfc_of(*node)
                        && number != rfc
                    {
                        expected_rfc_xrefs += 1;
                        let plain = node.children().any(|n| {
                            n.tag_name().name() == "seriesInfo"
                                && n.attribute("name") == Some("RFC")
                                && n.attribute("value")
                                    .is_some_and(|v| v.trim().parse::<u32>().is_ok())
                        });
                        if !plain {
                            prefixed += 1;
                        }
                    }
                }
                Some(_) => to_group += 1,
                None => {
                    if !anchors.contains(target) {
                        unresolved += 1;
                        if unresolved_examples.len() < 10 {
                            unresolved_examples.push(format!("{stem}:{target}"));
                        }
                    }
                }
            }
        }
        for reference in root
            .descendants()
            .filter(|n| n.tag_name().name() == "reference")
        {
            let named = reference
                .ancestors()
                .filter(|n| n.tag_name().name() == "references")
                .any(|group| {
                    group
                        .children()
                        .find(|c| c.tag_name().name() == "name")
                        .map(|name| {
                            let title: String = name
                                .descendants()
                                .filter(|n| n.is_text())
                                .map(|n| n.text().unwrap_or_default())
                                .collect();
                            title.to_lowercase().contains("normative")
                                || title.to_lowercase().contains("informative")
                        })
                        .unwrap_or(false)
                });
            if !named && rfc_of(reference).is_some() {
                *stats
                    .entry("RFC <reference>s whose kind is the default")
                    .or_default() += 1;
                default_files.insert(stem.clone());
            }
        }
        *stats.entry("xrefs to an RFC (independent)").or_default() += expected_rfc_xrefs;
        *stats
            .entry("xrefs to an RFC via a prefixed seriesInfo, DOI or URL only")
            .or_default() += prefixed;
        *stats.entry("xrefs to a referencegroup").or_default() += to_group;
        if to_group > 0 {
            *stats
                .entry("files with xrefs to a referencegroup")
                .or_default() += 1;
        }
        *stats.entry("xrefs to no anchor at all").or_default() += unresolved;
        *stats.entry("references stored").or_default() += document.references.len();
        let counts = document.reference_counts.unwrap();
        *stats.entry("self_references").or_default() += counts.self_references;
        *stats.entry("non_rfc").or_default() += counts.non_rfc;
        *stats.entry("unresolved").or_default() += counts.unresolved;
        if document.references.is_empty() {
            *stats.entry("files with zero references").or_default() += 1;
            if expected_rfc_xrefs > 0 {
                *stats
                    .entry("files with zero references but xrefs to an RFC")
                    .or_default() += 1;
                zero_examples.push(format!("{stem}({expected_rfc_xrefs})"));
            }
        }
        if document.references.len() != expected_rfc_xrefs {
            *stats.entry("files whose stored count differs").or_default() += 1;
            println!(
                "differs: {stem} stored {} independent {expected_rfc_xrefs}",
                document.references.len()
            );
        }
    }
    for (key, value) in &stats {
        println!("{key}: {value}");
    }
    println!(
        "files with a defaulted kind: {} {:?}",
        default_files.len(),
        default_files.iter().take(10).collect::<Vec<_>>()
    );
    println!("zero-reference files with RFC xrefs: {zero_examples:?}");
    println!("unresolved examples: {unresolved_examples:?}");
}
