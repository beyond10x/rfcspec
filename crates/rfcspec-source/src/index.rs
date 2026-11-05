//! The RFC Editor's `rfc-index.xml`, parsed: one [`IndexEntry`] per `rfc-entry`. Entries for
//! numbers never issued (`rfc-not-issued-entry`) and every other kind of entry (BCP, FYI, STD)
//! are skipped. Parsing only: nothing here reaches the network.

use anyhow::{Context, Result, bail};
use roxmltree::Node;

/// One published RFC as the index describes it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IndexEntry {
    pub rfc: u32,
    pub title: String,
    /// The `current-status`, verbatim, such as `PROPOSED STANDARD`.
    pub status: String,
    pub stream: String,
    pub area: Option<String>,
    /// `YYYY-MM`.
    pub date: String,
    /// The `file-format`s the RFC is published in, such as `TXT`, `HTML`, `PDF` or `XML`.
    pub formats: Vec<String>,
    pub obsoletes: Vec<u32>,
    pub obsoleted_by: Vec<u32>,
    pub updates: Vec<u32>,
    pub updated_by: Vec<u32>,
}

impl IndexEntry {
    /// Whether the RFC is published as plain text.
    pub fn has_text(&self) -> bool {
        self.formats.iter().any(|format| format == "TXT")
    }

    /// Whether the RFC is published as RFCXML.
    pub fn has_xml(&self) -> bool {
        self.formats.iter().any(|format| format == "XML")
    }
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Parse an `rfc-index.xml` into its RFC entries, in document order.
pub fn parse_index(xml: &str) -> Result<Vec<IndexEntry>> {
    let document = roxmltree::Document::parse(xml).context("parsing rfc-index.xml")?;
    let root = document.root_element();
    if root.tag_name().name() != "rfc-index" {
        bail!(
            "the root element is {}, expected rfc-index",
            root.tag_name().name()
        );
    }
    root.children()
        .filter(|node| node.is_element() && node.tag_name().name() == "rfc-entry")
        .map(|node| {
            let id = text(node, "doc-id").unwrap_or_default();
            entry(node).with_context(|| format!("rfc-index.xml entry {id:?}"))
        })
        .collect()
}

fn entry(node: Node) -> Result<IndexEntry> {
    let rfc = rfc_number(&required(node, "doc-id")?)?;
    let date = child(node, "date").context("no date")?;
    let month = required(date, "month")?;
    let year = required(date, "year")?;
    let month = MONTHS
        .iter()
        .position(|name| *name == month)
        .with_context(|| format!("month {month:?}"))?
        + 1;
    if year.len() != 4 || !year.bytes().all(|b| b.is_ascii_digit()) {
        bail!("year {year:?}");
    }
    let formats = child(node, "format")
        .map(|format| {
            elements(format, "file-format")
                .map(|node| node.text().unwrap_or_default().trim().to_string())
                .collect()
        })
        .unwrap_or_default();
    Ok(IndexEntry {
        rfc,
        title: required(node, "title")?,
        status: required(node, "current-status")?,
        stream: required(node, "stream")?,
        area: text(node, "area"),
        date: format!("{year}-{month:02}"),
        formats,
        obsoletes: numbers(node, "obsoletes")?,
        obsoleted_by: numbers(node, "obsoleted-by")?,
        updates: numbers(node, "updates")?,
        updated_by: numbers(node, "updated-by")?,
    })
}

fn elements<'a, 'input>(
    node: Node<'a, 'input>,
    name: &'static str,
) -> impl Iterator<Item = Node<'a, 'input>> {
    node.children()
        .filter(move |child| child.is_element() && child.tag_name().name() == name)
}

fn child<'a, 'input>(node: Node<'a, 'input>, name: &'static str) -> Option<Node<'a, 'input>> {
    elements(node, name).next()
}

fn text(node: Node, name: &'static str) -> Option<String> {
    child(node, name).map(|child| child.text().unwrap_or_default().trim().to_string())
}

fn required(node: Node, name: &'static str) -> Result<String> {
    text(node, name).with_context(|| format!("no {name}"))
}

/// The number of an `RFC<N>` document id.
fn rfc_number(id: &str) -> Result<u32> {
    id.strip_prefix("RFC")
        .and_then(|digits| digits.parse().ok())
        .with_context(|| format!("{id:?} is not an RFC document id"))
}

/// The RFC numbers of an update edge list (`obsoletes`, `updated-by`, ...); none when absent.
fn numbers(node: Node, name: &'static str) -> Result<Vec<u32>> {
    let Some(list) = child(node, name) else {
        return Ok(vec![]);
    };
    elements(list, "doc-id")
        .map(|id| rfc_number(id.text().unwrap_or_default().trim()))
        .collect::<Result<_>>()
        .with_context(|| format!("in {name}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX: &str = r#"<?xml version='1.0' encoding='utf-8'?>
<rfc-index xmlns="https://www.rfc-editor.org/rfc-index">
  <bcp-entry><doc-id>BCP1</doc-id></bcp-entry>
  <rfc-entry>
    <doc-id>RFC8</doc-id>
    <title>Functional specifications for the ARPA Network</title>
    <date><month>May</month><year>1969</year></date>
    <format><file-format>PDF</file-format></format>
    <current-status>UNKNOWN</current-status>
    <stream>Legacy</stream>
  </rfc-entry>
  <rfc-not-issued-entry><doc-id>RFC14</doc-id></rfc-not-issued-entry>
  <rfc-entry>
    <doc-id>RFC6455</doc-id>
    <title>The WebSocket Protocol</title>
    <date><month>December</month><year>2011</year></date>
    <format><file-format>TXT</file-format><file-format>HTML</file-format></format>
    <updated-by><doc-id>RFC7936</doc-id><doc-id>RFC8307</doc-id></updated-by>
    <current-status>PROPOSED STANDARD</current-status>
    <stream>IETF</stream>
    <area>art</area>
  </rfc-entry>
</rfc-index>"#;

    #[test]
    fn parses_rfc_entries_and_skips_every_other_kind() {
        let entries = parse_index(INDEX).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].rfc, 8);
        assert!(!entries[0].has_text());
        assert_eq!(entries[0].area, None);
        assert_eq!(entries[0].date, "1969-05");
        let websocket = &entries[1];
        assert!(websocket.has_text());
        assert_eq!(websocket.status, "PROPOSED STANDARD");
        assert_eq!(websocket.area.as_deref(), Some("art"));
        assert_eq!(websocket.date, "2011-12");
        assert_eq!(websocket.updated_by, vec![7936, 8307]);
        assert!(websocket.obsoletes.is_empty());
    }

    #[test]
    fn an_entry_has_xml_when_its_formats_include_xml() {
        let entries = parse_index(&INDEX.replace(
            "<file-format>HTML</file-format>",
            "<file-format>HTML</file-format><file-format>XML</file-format>",
        ))
        .unwrap();
        assert!(!entries[0].has_xml());
        assert!(entries[1].has_xml());
        assert!(entries[1].has_text());
        assert!(!parse_index(INDEX).unwrap()[1].has_xml());
    }

    #[test]
    fn refuses_an_entry_with_an_unknown_month_naming_it() {
        let error = parse_index(&INDEX.replace("December", "Decembre"))
            .unwrap_err()
            .to_string();
        assert!(error.contains("RFC6455"), "{error}");
    }
}
