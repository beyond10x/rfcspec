//! The bulk summary (`rfcspec-bulk/1`): one JSON line per `rfc-index.xml` entry with an XML or a
//! plain-text format, after a header line. A row is the index metadata of one RFC, the format and
//! SHA-256 of the source read and the counts of its mechanical catalogue requirements by BCP 14
//! level. The same rows always serialize to the same bytes.

use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::{SourceFormat, sha256_hex};

/// The summary format.
pub const BULK_FORMAT: &str = "rfcspec-bulk/1";

/// Line 1 of a summary.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkHeader {
    pub format: String,
    /// SHA-256 of the `rfc-index.xml` read.
    pub index_sha256: String,
    /// The extraction the counts came from (`rfcspec_extract::EXTRACTOR_VERSION`).
    pub extractor: u32,
    /// The number of rows.
    pub rows: usize,
    /// SHA-256 over every row line with its trailing newline, in file order.
    pub rows_sha256: String,
}

/// One RFC of the index with an XML or a plain-text format, read in bulk.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BulkRow {
    pub rfc: u32,
    pub title: String,
    /// The index `current-status`, verbatim, such as `PROPOSED STANDARD`.
    pub status: String,
    pub stream: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area: Option<String>,
    /// `YYYY-MM`.
    pub date: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obsoletes: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obsoleted_by: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub updates: Vec<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub updated_by: Vec<u32>,
    /// The format of the source read, written only as `"format":"xml"`; a row without it was read
    /// from plain text.
    #[serde(default, skip_serializing_if = "SourceFormat::is_text")]
    pub format: SourceFormat,
    /// SHA-256 of the source file's bytes, the RFCXML file for a row read from XML.
    pub sha256: String,
    /// Requirements by BCP 14 level, the level as the extractor spells it; no zeros.
    pub counts: BTreeMap<String, u64>,
    /// The text is not valid UTF-8 and was read with replacement characters.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub lossy: bool,
}

/// One summary line, without its newline.
pub fn bulk_line<T: Serialize>(value: &T) -> Result<String> {
    Ok(serde_json::to_string(value)?)
}

/// The `rows_sha256` of some row lines, each given without its newline.
pub fn rows_sha256<S: AsRef<str>>(lines: &[S]) -> String {
    let mut bytes = Vec::new();
    for line in lines {
        bytes.extend_from_slice(line.as_ref().as_bytes());
        bytes.push(b'\n');
    }
    sha256_hex(&bytes)
}

/// The whole summary of some rows: the header, then one line per row, each ending in a newline.
pub fn bulk_summary(index_sha256: &str, extractor: u32, rows: &[BulkRow]) -> Result<String> {
    let lines = rows.iter().map(bulk_line).collect::<Result<Vec<_>>>()?;
    let header = BulkHeader {
        format: BULK_FORMAT.into(),
        index_sha256: index_sha256.into(),
        extractor,
        rows: rows.len(),
        rows_sha256: rows_sha256(&lines),
    };
    let mut text = bulk_line(&header)?;
    text.push('\n');
    for line in lines {
        text.push_str(&line);
        text.push('\n');
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row() -> BulkRow {
        BulkRow {
            rfc: 6455,
            title: "The WebSocket Protocol".into(),
            status: "PROPOSED STANDARD".into(),
            stream: "IETF".into(),
            area: Some("art".into()),
            date: "2011-12".into(),
            obsoletes: vec![],
            obsoleted_by: vec![],
            updates: vec![],
            updated_by: vec![7936],
            sha256: "0".repeat(64),
            counts: BTreeMap::from([("MUST NOT".into(), 2), ("MUST".into(), 3)]),
            format: SourceFormat::Text,
            lossy: false,
        }
    }

    #[test]
    fn a_row_writes_its_fields_in_order_and_omits_empty_ones() {
        assert_eq!(
            bulk_line(&row()).unwrap(),
            format!(
                "{{\"rfc\":6455,\"title\":\"The WebSocket Protocol\",\"status\":\"PROPOSED STANDARD\",\"stream\":\"IETF\",\"area\":\"art\",\"date\":\"2011-12\",\"updated_by\":[7936],\"sha256\":\"{}\",\"counts\":{{\"MUST\":3,\"MUST NOT\":2}}}}",
                "0".repeat(64)
            )
        );
    }

    #[test]
    fn a_row_read_from_xml_writes_its_format_and_a_text_row_reads_back_as_text() {
        let xml = BulkRow {
            format: crate::SourceFormat::Xml,
            ..row()
        };
        let line = bulk_line(&xml).unwrap();
        assert!(
            line.contains(&format!(
                "\"updated_by\":[7936],\"format\":\"xml\",\"sha256\":\"{}\"",
                "0".repeat(64)
            )),
            "{line}"
        );
        assert_eq!(serde_json::from_str::<BulkRow>(&line).unwrap(), xml);
        let text: BulkRow = serde_json::from_str(&bulk_line(&row()).unwrap()).unwrap();
        assert_eq!(text.format, crate::SourceFormat::Text);
    }

    #[test]
    fn the_header_digests_the_row_lines_with_their_newlines() {
        let text = bulk_summary(&"1".repeat(64), 1, &[row()]).unwrap();
        let mut lines = text.lines();
        let header: BulkHeader = serde_json::from_str(lines.next().unwrap()).unwrap();
        let line = lines.next().unwrap();
        assert_eq!(header.rows, 1);
        assert_eq!(
            header.rows_sha256,
            sha256_hex(format!("{line}\n").as_bytes())
        );
        let back: BulkRow = serde_json::from_str(line).unwrap();
        assert_eq!(back, row());
    }
}
