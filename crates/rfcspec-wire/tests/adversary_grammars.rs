//! Adversarial cases for story:catalogue-grammars, run against cached RFCs of the bulk corpus
//! (`$HOME/.cache/rfcspec-corpus`, read only). A case whose corpus file is absent panics rather
//! than passes, so it is never silently green.

use std::{collections::BTreeMap, fs, path::PathBuf};

use rfcspec_model::{Document, wire::Grammar};
use rfcspec_wire::grammars;

fn corpus(name: &str) -> String {
    let path = PathBuf::from(std::env::var("HOME").unwrap())
        .join(".cache/rfcspec-corpus")
        .join(name);
    fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn text(rfc: u32) -> Document {
    rfcspec_doc::read(rfc, "00", &corpus(&format!("txt/rfc{rfc}.txt")))
}

fn xml(rfc: u32) -> Document {
    rfcspec_doc::xml::read(rfc, "00", &corpus(&format!("xml/rfc{rfc}.xml"))).unwrap()
}

fn by_name(found: &[Grammar]) -> BTreeMap<String, usize> {
    found
        .iter()
        .map(|g| {
            (
                g.name.clone(),
                g.rules.lines().filter(|l| !l.trim().is_empty()).count(),
            )
        })
        .collect()
}

/// The text and the RFCXML of one RFC give grammars for the same sections.
fn text_and_xml_agree(rfc: u32) {
    let from_text = by_name(&grammars::collect(&text(rfc)));
    let from_xml = by_name(&grammars::collect(&xml(rfc)));
    let only_text: Vec<_> = from_text
        .iter()
        .filter(|(k, _)| !from_xml.contains_key(*k))
        .collect();
    let only_xml: Vec<_> = from_xml
        .iter()
        .filter(|(k, _)| !from_text.contains_key(*k))
        .collect();
    assert!(
        only_text.is_empty() && only_xml.is_empty(),
        "RFC {rfc}: sections with a grammar only from text (section, non-empty lines): {only_text:?}; only from RFCXML: {only_xml:?}"
    );
}

#[test]
#[ignore = "reads a corpus outside the repository"]
fn rfc9110_text_and_rfcxml_give_grammars_for_the_same_sections() {
    text_and_xml_agree(9110);
}

#[test]
#[ignore = "reads a corpus outside the repository"]
fn rfc9112_text_and_rfcxml_give_grammars_for_the_same_sections() {
    text_and_xml_agree(9112);
}

/// Page furniture never reaches a grammar's rules: no `[Page N]` footer and no running header.
fn no_furniture(rfc: u32) {
    let found = grammars::collect(&text(rfc));
    let header = format!("RFC {rfc} ");
    let leaked: Vec<(String, String)> = found
        .iter()
        .flat_map(|g| {
            g.rules
                .lines()
                .filter(|l| l.contains("[Page ") || l.starts_with(&header) || l.contains('\u{c}'))
                .map(|l| (g.id.clone(), l.to_owned()))
                .collect::<Vec<_>>()
        })
        .collect();
    assert!(leaked.is_empty(), "RFC {rfc}: {leaked:#?}");
}

#[test]
#[ignore = "reads a corpus outside the repository"]
fn page_furniture_stays_out_of_the_rules_of_rfc3986_5234_7230_9110() {
    for rfc in [3986, 5234, 7230, 9110] {
        no_furniture(rfc);
    }
}

/// A grammar's span covers every line of its rules: the span holds at least as many lines as the
/// rules do, and every grammar has a section once the RFC has numbered sections.
#[test]
#[ignore = "reads a corpus outside the repository"]
fn grammar_spans_hold_their_rules_and_have_a_section() {
    for rfc in [3986, 5234, 7230, 9110] {
        for g in grammars::collect(&text(rfc)) {
            let lines = g.rules.lines().count();
            assert!(
                g.span.end + 1 - g.span.start >= lines,
                "RFC {rfc} {}: span {:?} shorter than {lines} rule lines",
                g.id,
                g.span
            );
            assert!(
                g.section.is_some() && !g.id.ends_with("/-"),
                "RFC {rfc}: grammar {} has no section",
                g.id
            );
        }
    }
}

/// A figure the RFCXML types as something other than ABNF (`http-message`, `ascii-art`, `json`,
/// ...) is not ABNF, yet the text test (a line that defines a rule) counts it as one when the RFC
/// is read from plain text. Every such figure in the cached RFCXML corpus is listed.
#[test]
#[ignore = "accepted by story:catalogue-grammars: text has no figure type"]
fn figures_rfcxml_types_as_not_abnf_are_not_abnf_by_their_lines() {
    let dir = PathBuf::from(std::env::var("HOME").unwrap()).join(".cache/rfcspec-corpus/xml");
    let mut wrong: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for entry in fs::read_dir(&dir).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_stem().unwrap().to_string_lossy().into_owned();
        let Some(rfc) = name.strip_prefix("rfc").and_then(|n| n.parse::<u32>().ok()) else {
            continue;
        };
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        let Ok(document) = rfcspec_doc::xml::read(rfc, "00", &source) else {
            continue;
        };
        for block in &document.blocks {
            let Some(kind) = block.figure_type.as_deref() else {
                continue;
            };
            if kind.starts_with("abnf") {
                continue;
            }
            let mut untyped = block.clone();
            untyped.figure_type = None;
            if grammars::is_abnf(&untyped) {
                wrong.entry(kind.to_owned()).or_default().push(format!(
                    "{name} {}",
                    block.section.as_deref().unwrap_or("-")
                ));
            }
        }
    }
    let summary: Vec<(String, usize, Vec<String>)> = wrong
        .into_iter()
        .map(|(kind, at)| (kind, at.len(), at.into_iter().take(3).collect()))
        .collect();
    assert!(summary.is_empty(), "{summary:#?}");
}
