//! The bulk summary (`rfcspec-bulk/1`): [`summarise`] reads a copy of `rfc-index.xml`, a
//! directory of RFC plain texts and a directory of RFCXML sources, [`check`] validates a summary
//! offline against itself and against the RFCs the profiles lock.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rfcspec_extract::{EXTRACTOR_VERSION, LEVELS, extract, level_counts};
use rfcspec_model::{
    BULK_FORMAT, BulkHeader, BulkRow, Document, LOCK_FILE, SECTIONS_FORMAT, SectionCounts,
    SectionsRow, SourceFormat, bulk_summary, sections_sidecar, sha256_hex,
};
use rfcspec_source::corpus::bulk_format;
use serde_json::Value;

/// What the bulk files record of one source read by the reader of its format: its counts by
/// level, by the extraction `rfcspec extract` runs, and its sidecar row.
fn counts(
    rfc: u32,
    format: SourceFormat,
    sha256: &str,
    text: &str,
) -> Result<(BTreeMap<String, u64>, SectionsRow)> {
    let document = rfcspec_doc::read_format(rfc, format, sha256, text)?;
    let catalogue = extract(&document);
    let source: Vec<&str> = text.lines().collect();
    Ok((
        level_counts(&catalogue),
        sections_row(&document, &catalogue, &source),
    ))
}

/// Whether a source line continues a wrapped plain-text heading: it is indented deeper than the
/// three spaces of body text.
fn continues_heading(line: &str) -> bool {
    line.starts_with("    ") && !line.trim().is_empty()
}

/// A heading's title without its number: the first line of `block` read as `<number>.  Title`
/// or `Appendix <number>.  Title`, joined with the lines that continue a wrapped heading. In an
/// RFCXML heading every further line belongs to the title; in plain text only the lines of the
/// block that `source` (the source lines) shows indented deeper than body text do, so a body
/// paragraph run on without a blank line is not taken for the title.
fn heading_title(
    block: &rfcspec_model::Block,
    number: &str,
    format: SourceFormat,
    source: &[&str],
) -> Option<String> {
    let first = block.lines.iter().position(|line| !line.is_empty())?;
    let line = &block.lines[first];
    let line = line.strip_prefix("Appendix ").unwrap_or(line);
    let mut title = line
        .strip_prefix(number)?
        .strip_prefix(". ")?
        .trim()
        .to_owned();
    let rest = &block.lines[first + 1..];
    let wrapped = match format {
        SourceFormat::Xml => block.kind == rfcspec_model::BlockKind::Heading,
        SourceFormat::Text => {
            rest.len() <= MAX_HEADING_CONTINUATIONS
                && !block.lines.iter().any(|line| line.contains(DOT_LEADER))
                && rest.iter().enumerate().all(|(at, line)| {
                    !line.is_empty()
                        && source
                            .get(block.span.start + first + at)
                            .is_some_and(|source| continues_heading(source))
                })
        }
    };
    if wrapped {
        for line in rest.iter().take_while(|line| !line.is_empty()) {
            title.push(' ');
            title.push_str(line.trim());
        }
    }
    (!title.is_empty()).then_some(title)
}

/// The most lines a wrapped plain-text heading continues on. A numbered block running longer is
/// a numbered paragraph, and only its first line is read as the title.
const MAX_HEADING_CONTINUATIONS: usize = 2;

/// Dots that lead a table-of-contents entry to its page number; a block holding them is a table
/// of contents, not a wrapped heading.
const DOT_LEADER: &str = "....";

/// The sidecar row of a document: each numbered section holding a requirement, in document
/// order, with its anchor and title where a heading for it was read, then the requirements
/// outside every numbered section; and the document's References. `source` is the source text's
/// lines, which tell a wrapped heading's continuation from body text.
fn sections_row(
    document: &Document,
    catalogue: &rfcspec_model::Catalogue,
    source: &[&str],
) -> SectionsRow {
    let mut order: Vec<Option<String>> = Vec::new();
    let mut counts: BTreeMap<Option<String>, BTreeMap<String, u64>> = BTreeMap::new();
    for requirement in &catalogue.requirements {
        let key = requirement.section.clone();
        if !counts.contains_key(&key) {
            order.push(key.clone());
        }
        *counts
            .entry(key)
            .or_default()
            .entry(requirement.keyword.clone())
            .or_default() += 1;
    }
    order.sort_by_key(Option::is_none);
    let sections = order
        .into_iter()
        .map(|number| {
            let heading = number.as_deref().and_then(|number| {
                document
                    .blocks
                    .iter()
                    .filter(|block| block.section.as_deref() == Some(number))
                    .find_map(|block| {
                        heading_title(block, number, document.format, source)
                            .map(|title| (block, title))
                    })
            });
            SectionCounts {
                anchor: heading.as_ref().and_then(|(block, _)| block.anchor.clone()),
                title: heading.map(|(_, title)| title),
                counts: counts.remove(&number).unwrap_or_default(),
                number,
            }
        })
        .collect();
    SectionsRow {
        rfc: document.rfc,
        sections,
        references: document.references.clone(),
    }
}

/// The name of a format in a check message: the value a row's `format` holds, `text` for plain
/// text.
fn format_name(format: SourceFormat) -> &'static str {
    match format {
        SourceFormat::Xml => "xml",
        SourceFormat::Text => "text",
    }
}

/// The summary of every index entry with an XML or a TXT format: one with XML read from
/// `<xml>/rfc<N>.xml`, every other one from `<texts>/rfc<N>.txt`. Refused, naming each RFC and
/// its expected path, when a source is missing, and naming the RFC and the file when an XML
/// source does not read. Returns the summary and its section sidecar.
pub fn summarise(index: &Path, texts: &Path, xml: &Path) -> Result<(String, String)> {
    let bytes = fs::read(index).with_context(|| format!("reading {}", index.display()))?;
    let index_xml =
        std::str::from_utf8(&bytes).with_context(|| format!("{} is not UTF-8", index.display()))?;
    let mut entries: Vec<_> = rfcspec_source::index::parse_index(index_xml)
        .with_context(|| format!("reading {}", index.display()))?
        .into_iter()
        .filter_map(|entry| bulk_format(&entry).map(|format| (format, entry)))
        .collect();
    entries.sort_by_key(|(_, entry)| entry.rfc);
    let mut problems = Vec::new();
    let mut rows = Vec::new();
    let mut sidecar = Vec::new();
    for (format, entry) in entries {
        let path = match format {
            SourceFormat::Xml => xml.join(format!("rfc{}.xml", entry.rfc)),
            SourceFormat::Text => texts.join(format!("rfc{}.txt", entry.rfc)),
        };
        let Ok(source) = fs::read(&path) else {
            let kind = match format {
                SourceFormat::Xml => "XML",
                SourceFormat::Text => "text",
            };
            problems.push(format!(
                "RFC {}: no {kind} at {}",
                entry.rfc,
                path.display()
            ));
            continue;
        };
        let sha256 = sha256_hex(&source);
        let (text, lossy) = match (String::from_utf8(source), format) {
            (Ok(text), _) => (text, false),
            (Err(_), SourceFormat::Xml) => {
                problems.push(format!(
                    "RFC {}: {} is not UTF-8",
                    entry.rfc,
                    path.display()
                ));
                continue;
            }
            (Err(error), SourceFormat::Text) => {
                (String::from_utf8_lossy(error.as_bytes()).into_owned(), true)
            }
        };
        let (counts, sections) = match counts(entry.rfc, format, &sha256, &text) {
            Ok(read) => read,
            Err(error) => {
                problems.push(format!(
                    "RFC {}: reading {}: {error:#}",
                    entry.rfc,
                    path.display()
                ));
                continue;
            }
        };
        sidecar.push(sections);
        rows.push(BulkRow {
            counts,
            rfc: entry.rfc,
            title: entry.title,
            status: entry.status,
            stream: entry.stream,
            area: entry.area,
            date: entry.date,
            obsoletes: entry.obsoletes,
            obsoleted_by: entry.obsoleted_by,
            updates: entry.updates,
            updated_by: entry.updated_by,
            format,
            sha256,
            lossy,
        });
    }
    if !problems.is_empty() {
        bail!("{}", problems.join("; "));
    }
    let index_sha256 = sha256_hex(&bytes);
    Ok((
        bulk_summary(&index_sha256, EXTRACTOR_VERSION, &rows)?,
        sections_sidecar(&index_sha256, EXTRACTOR_VERSION, &sidecar)?,
    ))
}

/// Whether a value is 64 lower-case hexadecimal digits.
fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// A row as the check read it: its line number, source format, digest and counts.
struct Read {
    line: usize,
    format: SourceFormat,
    sha256: String,
    counts: BTreeMap<String, u64>,
}

/// The problems of one row line, numbered `line`; with none, the row.
fn row(line: usize, text: &str) -> (Vec<String>, Option<(u32, Read)>) {
    let mut problems = Vec::new();
    let value: Value = match serde_json::from_str(text) {
        Ok(value) => value,
        Err(error) => return (vec![format!("line {line}: not JSON: {error}")], None),
    };
    match value.get("sha256") {
        None => problems.push(format!("line {line}: sha256 is missing")),
        Some(Value::String(digest)) if is_digest(digest) => {}
        Some(other) => problems.push(format!(
            "line {line}: sha256 {other} is not 64 lower-case hex digits"
        )),
    }
    match value.get("counts") {
        Some(Value::Object(counts)) => {
            for (level, count) in counts {
                if !LEVELS.contains(&level.as_str()) {
                    problems.push(format!(
                        "line {line}: counts key {level:?} is not one of the eleven BCP 14 levels"
                    ));
                }
                if !count.as_u64().is_some_and(|count| count > 0) {
                    problems.push(format!(
                        "line {line}: counts {level:?} is {count}, not a positive integer"
                    ));
                }
            }
        }
        _ => problems.push(format!("line {line}: counts is missing or not an object")),
    }
    match value.get("format") {
        None => {}
        Some(Value::String(format)) if format == "xml" => {
            if value.get("lossy") == Some(&Value::Bool(true)) {
                problems.push(format!(
                    "line {line}: lossy is true on a row read from XML; an XML source that is not UTF-8 is refused, never read lossily"
                ));
            }
        }
        Some(other) => problems.push(format!(
            "line {line}: format {other} is not \"xml\", the only format a row records; a row without one was read from plain text"
        )),
    }
    if !problems.is_empty() {
        return (problems, None);
    }
    match serde_json::from_value::<BulkRow>(value) {
        Ok(row) => (
            vec![],
            Some((
                row.rfc,
                Read {
                    line,
                    format: row.format,
                    sha256: row.sha256,
                    counts: row.counts,
                },
            )),
        ),
        Err(error) => (vec![format!("line {line}: not a row: {error}")], None),
    }
}

/// Every problem of a summary, each naming its line or the lock it concerns; none when it holds.
pub fn check(summary: &Path, profiles: &Path) -> Result<Vec<String>> {
    let text =
        fs::read_to_string(summary).with_context(|| format!("reading {}", summary.display()))?;
    let Some((head, body)) = text.split_once('\n') else {
        return Ok(vec!["line 1: no header line".into()]);
    };
    let mut problems = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    match serde_json::from_str::<BulkHeader>(head) {
        Err(error) => problems.push(format!("line 1: not an {BULK_FORMAT} header: {error}")),
        Ok(header) => {
            if header.format != BULK_FORMAT {
                problems.push(format!(
                    "line 1: format {:?}, expected {BULK_FORMAT}",
                    header.format
                ));
            }
            if header.extractor != EXTRACTOR_VERSION {
                problems.push(format!(
                    "line 1: extractor {}, but this extraction is version {EXTRACTOR_VERSION}",
                    header.extractor
                ));
            }
            if header.rows != lines.len() {
                problems.push(format!(
                    "line 1: rows {}, but the summary has {} rows",
                    header.rows,
                    lines.len()
                ));
            }
            let actual = sha256_hex(body.as_bytes());
            if header.rows_sha256 != actual {
                problems.push(format!(
                    "line 1: rows_sha256 {} differs from {actual}, computed over the row lines",
                    header.rows_sha256
                ));
            }
        }
    }
    let mut rows: BTreeMap<u32, Read> = BTreeMap::new();
    let mut previous: Option<u32> = None;
    for (at, text) in lines.iter().enumerate() {
        let line = at + 2;
        let (found, read) = row(line, text);
        problems.extend(found);
        let Some((rfc, read)) = read else { continue };
        match previous {
            Some(before) if before == rfc => {
                problems.push(format!("line {line}: duplicate row for RFC {rfc}"));
            }
            Some(before) if before > rfc => problems.push(format!(
                "line {line}: RFC {rfc} after RFC {before}; numbers must be strictly ascending"
            )),
            _ => {}
        }
        previous = Some(rfc);
        rows.entry(rfc).or_insert(read);
    }
    for profile in locked_profiles(profiles)? {
        let lock_path = profile.join(LOCK_FILE);
        for entry in rfcspec_source::read_lock(&profile)?.sources {
            let rfc = entry.rfc;
            let Some(read) = rows.get(&rfc) else {
                problems.push(format!(
                    "{}: RFC {rfc} is locked but has no row",
                    lock_path.display()
                ));
                continue;
            };
            let line = read.line;
            if read.sha256 != entry.sha256 {
                problems.push(format!(
                    "line {line}: RFC {rfc} sha256 {} differs from {} locked in {}",
                    read.sha256,
                    entry.sha256,
                    lock_path.display()
                ));
            }
            let locked_format = rfcspec_source::format_of(&entry.file)?;
            if read.format != locked_format {
                problems.push(format!(
                    "line {line}: RFC {rfc} row format {} differs from format {} of {} locked in {}",
                    format_name(read.format),
                    format_name(locked_format),
                    entry.file,
                    lock_path.display()
                ));
                continue;
            }
            let locked = rfcspec_source::read_locked(&profile, rfc)?;
            let (expected, _) = counts(rfc, locked.format, &locked.entry.sha256, &locked.text)
                .with_context(|| format!("reading {}", profile.join(&entry.file).display()))?;
            if read.counts != expected {
                problems.push(format!(
                    "line {line}: RFC {rfc} counts {} differ from {} extracted from the source locked in {}",
                    serde_json::to_string(&read.counts)?,
                    serde_json::to_string(&expected)?,
                    lock_path.display()
                ));
            }
        }
    }
    Ok(problems)
}

/// The problems of one sidecar row's sections, which `summarise` never writes: a section number
/// listed twice; a section with no count, a count that is not positive or a level that is not
/// a BCP 14 level; an entry without a number that is not the last, comes twice, or carries an
/// anchor or a title.
fn section_problems(line: usize, row: &SectionsRow) -> Vec<String> {
    let rfc = row.rfc;
    let mut problems = Vec::new();
    let mut numbers = std::collections::BTreeSet::new();
    let last = row.sections.len().saturating_sub(1);
    for (at, section) in row.sections.iter().enumerate() {
        let name = section
            .number
            .as_deref()
            .map_or_else(|| "without a number".to_owned(), |n| format!("section {n}"));
        match &section.number {
            Some(number) if !numbers.insert(number.clone()) => problems.push(format!(
                "line {line}: RFC {rfc} lists section {number} more than once"
            )),
            Some(_) => {}
            None => {
                if at != last {
                    problems.push(format!(
                        "line {line}: RFC {rfc} lists the entry without a number at position {}, not last",
                        at + 1
                    ));
                }
                if section.anchor.is_some() || section.title.is_some() {
                    problems.push(format!(
                        "line {line}: RFC {rfc} gives the entry without a number an anchor or a title"
                    ));
                }
            }
        }
        if section.counts.is_empty() {
            problems.push(format!(
                "line {line}: RFC {rfc} {name} counts no requirement; only sections holding one are listed"
            ));
        }
        for (level, count) in &section.counts {
            if !LEVELS.contains(&level.as_str()) {
                problems.push(format!(
                    "line {line}: RFC {rfc} {name} counts key {level:?} is not one of the eleven BCP 14 levels"
                ));
            }
            if *count == 0 {
                problems.push(format!(
                    "line {line}: RFC {rfc} {name} counts {level:?} is 0, not a positive integer"
                ));
            }
        }
    }
    let numberless = row.sections.iter().filter(|s| s.number.is_none()).count();
    if numberless > 1 {
        problems.push(format!(
            "line {line}: RFC {rfc} lists {numberless} entries without a number; at most one counts the requirements outside every numbered section"
        ));
    }
    problems
}

/// Every problem of a section sidecar against its summary, each naming its sidecar line or the
/// RFC it concerns; none when it holds. The header must be an `rfcspec-sections/1` header of the
/// summary's index and extraction that counts and digests its rows; the rows must name the
/// summary's RFCs, in its order, each with section counts adding up to its summary counts and
/// References only where it was read from RFCXML; and the row of every RFC a profile locks must
/// be the one read from its locked source.
pub fn check_sections(sections: &Path, summary: &Path, profiles: &Path) -> Result<Vec<String>> {
    let text =
        fs::read_to_string(sections).with_context(|| format!("reading {}", sections.display()))?;
    let summary_text =
        fs::read_to_string(summary).with_context(|| format!("reading {}", summary.display()))?;
    let (summary_head, summary_body) = summary_text.split_once('\n').unwrap_or_default();
    let summary_header = serde_json::from_str::<BulkHeader>(summary_head).ok();
    let summary_rows: BTreeMap<u32, BulkRow> = summary_body
        .lines()
        .filter_map(|line| serde_json::from_str::<BulkRow>(line).ok())
        .map(|row| (row.rfc, row))
        .collect();
    let Some((head, body)) = text.split_once('\n') else {
        return Ok(vec!["line 1: no header line".into()]);
    };
    let mut problems = Vec::new();
    let lines: Vec<&str> = body.lines().collect();
    match serde_json::from_str::<BulkHeader>(head) {
        Err(error) => problems.push(format!("line 1: not an {SECTIONS_FORMAT} header: {error}")),
        Ok(header) => {
            if header.format != SECTIONS_FORMAT {
                problems.push(format!(
                    "line 1: format {:?}, expected {SECTIONS_FORMAT}",
                    header.format
                ));
            }
            if header.extractor != EXTRACTOR_VERSION {
                problems.push(format!(
                    "line 1: extractor {}, but this extraction is version {EXTRACTOR_VERSION}",
                    header.extractor
                ));
            }
            if let Some(summary_header) = &summary_header
                && header.index_sha256 != summary_header.index_sha256
            {
                problems.push(format!(
                    "line 1: index_sha256 {} differs from {} in {}",
                    header.index_sha256,
                    summary_header.index_sha256,
                    summary.display()
                ));
            }
            if header.rows != lines.len() {
                problems.push(format!(
                    "line 1: rows {}, but the sidecar has {} rows",
                    header.rows,
                    lines.len()
                ));
            }
            let actual = sha256_hex(body.as_bytes());
            if header.rows_sha256 != actual {
                problems.push(format!(
                    "line 1: rows_sha256 {} differs from {actual}, computed over the row lines",
                    header.rows_sha256
                ));
            }
        }
    }
    let mut seen: BTreeMap<u32, (usize, SectionsRow)> = BTreeMap::new();
    let mut previous: Option<u32> = None;
    for (at, text) in lines.iter().enumerate() {
        let line = at + 2;
        let row: SectionsRow = match serde_json::from_str(text) {
            Ok(row) => row,
            Err(error) => {
                problems.push(format!(
                    "line {line}: not an {SECTIONS_FORMAT} row: {error}"
                ));
                continue;
            }
        };
        let rfc = row.rfc;
        match previous {
            Some(before) if before == rfc => {
                problems.push(format!("line {line}: duplicate row for RFC {rfc}"));
            }
            Some(before) if before > rfc => problems.push(format!(
                "line {line}: RFC {rfc} after RFC {before}; numbers must be strictly ascending"
            )),
            _ => {}
        }
        previous = Some(rfc);
        problems.extend(section_problems(line, &row));
        match summary_rows.get(&rfc) {
            None => problems.push(format!(
                "line {line}: RFC {rfc} is not in the summary {}",
                summary.display()
            )),
            Some(summary_row) => {
                let mut summed: BTreeMap<String, u64> = BTreeMap::new();
                for section in &row.sections {
                    for (level, n) in &section.counts {
                        *summed.entry(level.clone()).or_default() += n;
                    }
                }
                if summed != summary_row.counts {
                    problems.push(format!(
                        "line {line}: RFC {rfc} section counts add up to {}, not to its summary counts {}",
                        serde_json::to_string(&summed)?,
                        serde_json::to_string(&summary_row.counts)?
                    ));
                }
                if summary_row.format == SourceFormat::Text && !row.references.is_empty() {
                    problems.push(format!(
                        "line {line}: RFC {rfc} has {} references, but its summary row was read from plain text, which records none",
                        row.references.len()
                    ));
                }
            }
        }
        seen.entry(rfc).or_insert((line, row));
    }
    for rfc in summary_rows.keys().filter(|rfc| !seen.contains_key(rfc)) {
        problems.push(format!(
            "RFC {rfc} has a row in the summary {} but no row in the sidecar",
            summary.display()
        ));
    }
    for profile in locked_profiles(profiles)? {
        let lock_path = profile.join(LOCK_FILE);
        for entry in rfcspec_source::read_lock(&profile)?.sources {
            let Some((line, row)) = seen.get(&entry.rfc) else {
                continue;
            };
            let locked = rfcspec_source::read_locked(&profile, entry.rfc)?;
            let (_, expected) =
                counts(entry.rfc, locked.format, &locked.entry.sha256, &locked.text)
                    .with_context(|| format!("reading {}", profile.join(&entry.file).display()))?;
            if *row != expected {
                problems.push(format!(
                    "line {line}: RFC {} sections and references differ from {}, read from the source locked in {}",
                    entry.rfc,
                    serde_json::to_string(&expected)?,
                    lock_path.display()
                ));
            }
        }
    }
    Ok(problems)
}

/// The directories under `dir` that hold a lock.
fn locked_profiles(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(crate::checks::profiles(dir)?
        .into_iter()
        .filter(|profile| profile.join(LOCK_FILE).is_file())
        .collect())
}

#[cfg(test)]
mod tests {
    use rfcspec_model::{Block, BlockKind, Span};

    use super::*;

    /// A plain-text block over `source`, its lines trimmed as the text reader trims them.
    fn block(source: &[&str]) -> Block {
        Block {
            kind: BlockKind::Paragraph,
            span: Span {
                start: 1,
                end: source.len(),
            },
            lines: source.iter().map(|line| line.trim().to_owned()).collect(),
            section: Some("3".into()),
            anchor: None,
            figure_type: None,
            bcp14: vec![],
            entries: vec![],
        }
    }

    fn title(source: &[&str]) -> Option<String> {
        heading_title(&block(source), "3", SourceFormat::Text, source)
    }

    #[test]
    fn a_heading_wrapped_onto_indented_lines_is_joined() {
        assert_eq!(
            title(&[
                "3.  Base Framing Protocol for Frames Sent Between a",
                "    Client and a",
                "    Server"
            ])
            .as_deref(),
            Some("Base Framing Protocol for Frames Sent Between a Client and a Server")
        );
    }

    #[test]
    fn a_numbered_paragraph_longer_than_a_wrapped_heading_keeps_only_its_first_line() {
        assert_eq!(
            title(&[
                "3.  The entry is mastered in a DSA which does not",
                "    follow this specification.  The entry in the EDB",
                "    will contain a master attribute, which holds a",
                "    subordinate reference."
            ])
            .as_deref(),
            Some("The entry is mastered in a DSA which does not")
        );
    }

    #[test]
    fn body_text_and_a_table_of_contents_are_not_joined() {
        assert_eq!(
            title(&["3.  Overview", "   Body text indented as body text."]).as_deref(),
            Some("Overview")
        );
        assert_eq!(
            title(&[
                "3.  Introduction.............2",
                "    3.1 Description..........2"
            ])
            .as_deref(),
            Some("Introduction.............2")
        );
    }
}
