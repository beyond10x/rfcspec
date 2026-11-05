//! The bulk summary (`rfcspec-bulk/1`): [`summarise`] reads a copy of `rfc-index.xml` and a
//! directory of RFC plain texts, [`check`] validates a summary offline against itself and against
//! the RFCs the profiles lock.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rfcspec_extract::{EXTRACTOR_VERSION, LEVELS, extract, level_counts};
use rfcspec_model::{
    BULK_FORMAT, BulkHeader, BulkRow, LOCK_FILE, SourceFormat, bulk_summary, sha256_hex,
};
use serde_json::Value;

/// The counts by level of a text, by the extraction `rfcspec extract` runs.
fn counts(rfc: u32, sha256: &str, text: &str) -> BTreeMap<String, u64> {
    level_counts(&extract(&rfcspec_doc::read(rfc, sha256, text)))
}

/// The summary of every index entry with a plain text, read from `<texts>/rfc<N>.txt`; refused,
/// naming each RFC and its expected path, when a text is missing.
pub fn summarise(index: &Path, texts: &Path) -> Result<String> {
    let bytes = fs::read(index).with_context(|| format!("reading {}", index.display()))?;
    let xml =
        std::str::from_utf8(&bytes).with_context(|| format!("{} is not UTF-8", index.display()))?;
    let mut entries: Vec<_> = rfcspec_source::index::parse_index(xml)
        .with_context(|| format!("reading {}", index.display()))?
        .into_iter()
        .filter(|entry| entry.has_text())
        .collect();
    entries.sort_by_key(|entry| entry.rfc);
    let mut missing = Vec::new();
    let mut rows = Vec::new();
    for entry in entries {
        let path = texts.join(format!("rfc{}.txt", entry.rfc));
        let Ok(bytes) = fs::read(&path) else {
            missing.push(format!("RFC {}: no text at {}", entry.rfc, path.display()));
            continue;
        };
        let sha256 = sha256_hex(&bytes);
        let (text, lossy) = match String::from_utf8(bytes) {
            Ok(text) => (text, false),
            Err(error) => (String::from_utf8_lossy(error.as_bytes()).into_owned(), true),
        };
        rows.push(BulkRow {
            counts: counts(entry.rfc, &sha256, &text),
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
            sha256,
            lossy,
        });
    }
    if !missing.is_empty() {
        bail!("{}", missing.join("; "));
    }
    bulk_summary(&sha256_hex(&bytes), EXTRACTOR_VERSION, &rows)
}

/// Whether a value is 64 lower-case hexadecimal digits.
fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'))
}

/// A row as the check read it: its line number, digest and counts.
struct Read {
    line: usize,
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
            if matches!(
                rfcspec_source::format_of(&entry.file),
                Ok(SourceFormat::Xml)
            ) {
                problems.push(format!(
                    "{}: RFC {rfc} is locked as RFCXML ({}), and the bulk summary counts plain text only; story:bulk-xml-summary makes it read XML",
                    lock_path.display(),
                    entry.file
                ));
                continue;
            }
            let locked = rfcspec_source::read_locked(&profile, rfc)?;
            let expected = counts(rfc, &locked.entry.sha256, &locked.text);
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

/// The directories under `dir` that hold a lock.
fn locked_profiles(dir: &Path) -> Result<Vec<PathBuf>> {
    Ok(crate::checks::profiles(dir)?
        .into_iter()
        .filter(|profile| profile.join(LOCK_FILE).is_file())
        .collect())
}
