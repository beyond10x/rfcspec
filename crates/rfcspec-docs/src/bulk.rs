//! The RFC index under `website/docs/catalog/rfcs/`: the bulk summary `bulk/rfc-summary.jsonl`
//! (`rfcspec-bulk/1`) rendered as an overview and one table page per thousand RFC numbers, never
//! one page per RFC. An RFC a profile locks is marked and linked to its profile page. Everything
//! here is a projection of the summary and the profiles' locks; it reads no network.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, bail, ensure};
use rfcspec_model::{BULK_FORMAT, BulkHeader, BulkRow, LOCK_FILE, Lock, SourceFormat, read_yaml};
use rfcspec_source::RFC_EDITOR;

use crate::{
    catalog::{self, PROFILES, counted},
    page::{Meta, SOURCE, cell, front_matter},
};

/// The bulk summary, relative to the repository root.
pub const SUMMARY: &str = "bulk/rfc-summary.jsonl";
/// Where the RFC index pages land, relative to the repository root.
pub const DIR: &str = "website/docs/catalog/rfcs";
/// The overview page.
pub const OVERVIEW: &str = "website/docs/catalog/rfcs/index.md";
/// The route of the overview, under the site's base URL.
pub const ROUTE: &str = "/docs/catalog/rfcs";

/// A bulk summary, read.
#[derive(Debug)]
pub struct Summary {
    pub header: BulkHeader,
    pub rows: Vec<BulkRow>,
}

/// Read a summary's text: the header line, then one row per line. Refused, naming the line, when
/// a line does not parse, the format is another or the header counts other rows; the full check
/// is `rfcspec bulk check`.
pub fn parse(text: &str) -> Result<Summary> {
    let mut lines = text.lines().enumerate();
    let Some((_, first)) = lines.next() else {
        bail!("the summary is empty");
    };
    let header: BulkHeader =
        serde_json::from_str(first).context("line 1: not an rfcspec-bulk/1 header")?;
    ensure!(
        header.format == BULK_FORMAT,
        "line 1: format {} is not {BULK_FORMAT}",
        header.format
    );
    let rows = lines
        .map(|(n, line)| {
            serde_json::from_str(line).with_context(|| format!("line {}: not a summary row", n + 1))
        })
        .collect::<Result<Vec<BulkRow>>>()?;
    ensure!(
        header.rows == rows.len(),
        "line 1: the header counts {} rows, the summary holds {}",
        header.rows,
        rows.len()
    );
    Ok(Summary { header, rows })
}

/// Each profile under `root/profiles/` that has a lock, by name, with the RFCs it locks.
pub fn locked(root: &Path) -> Result<Vec<(String, Vec<u32>)>> {
    let dir = root.join(PROFILES);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().join(LOCK_FILE).is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let lock: Lock = read_yaml(&dir.join(&name).join(LOCK_FILE))?;
            let rfcs: BTreeSet<u32> = lock.sources.iter().map(|s| s.rfc).collect();
            Ok((name, rfcs.into_iter().collect()))
        })
        .collect()
}

/// The first number of the range an RFC's row is listed in.
fn range(rfc: u32) -> u32 {
    rfc / 1000 * 1000
}

/// The route of a range page, by its first number.
fn route(start: u32) -> String {
    format!("{ROUTE}/{start:05}")
}

/// The numbers a range page lists, `RFC 6000 to 6999`; the first range starts at RFC 1.
fn label(start: u32) -> String {
    format!("RFC {} to {}", start.max(1), start + 999)
}

/// The name a page gives a source format.
fn format_name(format: SourceFormat) -> &'static str {
    match format {
        SourceFormat::Xml => "XML",
        SourceFormat::Text => "text",
    }
}

/// A row's Source cell: the format its counts were read from, linked to that RFC Editor file.
fn source(row: &BulkRow) -> String {
    format!(
        "[{}]({RFC_EDITOR}rfc{}.{})",
        format_name(row.format),
        row.rfc,
        row.format.extension()
    )
}

/// What a page needs to link an RFC: the numbers with a row and the profiles locking each.
struct Links<'a> {
    rows: BTreeSet<u32>,
    profiles: BTreeMap<u32, Vec<&'a str>>,
}

impl Links<'_> {
    /// An RFC number, linked to its range page when it has a row.
    fn rfc(&self, rfc: u32, text: &str) -> String {
        if self.rows.contains(&rfc) {
            format!("[{text}]({})", route(range(rfc)))
        } else {
            text.to_owned()
        }
    }

    fn numbers(&self, numbers: &[u32]) -> String {
        numbers
            .iter()
            .map(|n| self.rfc(*n, &n.to_string()))
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// Each profile locking an RFC, linked to its page.
    fn profiles(&self, rfc: u32) -> String {
        self.profiles
            .get(&rfc)
            .map(|names| {
                names
                    .iter()
                    .map(|name| format!("[{}]({}/{name})", cell(name), catalog::ROUTE))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default()
    }
}

/// The overview and range pages of a summary, by path relative to the repository root.
pub fn render(summary: &Summary, profiles: &[(String, Vec<u32>)]) -> BTreeMap<String, String> {
    let mut ranges: BTreeMap<u32, Vec<&BulkRow>> = BTreeMap::new();
    for row in &summary.rows {
        ranges.entry(range(row.rfc)).or_default().push(row);
    }
    for rows in ranges.values_mut() {
        rows.sort_by_key(|row| row.rfc);
    }
    let mut links = Links {
        rows: summary.rows.iter().map(|row| row.rfc).collect(),
        profiles: BTreeMap::new(),
    };
    for (name, rfcs) in profiles {
        for rfc in rfcs {
            links.profiles.entry(*rfc).or_default().push(name);
        }
    }
    let mut out = BTreeMap::from([(
        OVERVIEW.to_owned(),
        overview(summary, profiles, &ranges, &links),
    )]);
    for (start, rows) in &ranges {
        out.insert(
            format!("{DIR}/{start:05}.md"),
            range_page(*start, rows, &links),
        );
    }
    out
}

fn overview(
    summary: &Summary,
    profiles: &[(String, Vec<u32>)],
    ranges: &BTreeMap<u32, Vec<&BulkRow>>,
    links: &Links<'_>,
) -> String {
    let mut page = front_matter(&Meta {
        title: "RFC index",
        label: "RFC index",
        description: "Every RFC the RFC Editor publishes as plain text or RFCXML, with its BCP 14 requirement counts by level, read from its RFCXML where it exists, extracted mechanically and unreviewed, one table per thousand RFC numbers; the RFCs a profile holds are marked.",
        position: 20,
    });
    page.push_str(&format!(
        ":::note[Generated]\nThis page is generated by `rfcspec-docs` from the bulk summary [`{SUMMARY}`]({SOURCE}/{SUMMARY}) on `main`.\n:::\n\n"
    ));
    page.push_str(&format!(
        "One row per RFC that the [RFC Editor's index](https://www.rfc-editor.org/rfc-index.xml) lists as plain text or RFCXML, read in bulk at stage S2: its index metadata and how many BCP 14 requirements the mechanical extraction finds in it, by level, counted in its RFCXML where the index lists one, else in its plain text. Nothing here is reviewed. RFC links each number to the RFC Editor and to its view in the RFC explorer, which shows its requirements by section and its References; Source names the format read, linked to the RFC Editor file the counts came from; Obsoleted by and Updated by link each RFC that has a row; Requirements reads `none` when the extraction finds no BCP 14 keyword; Profile names each profile of the [protocol catalogue]({}) whose lock holds the RFC, linked to its page, where its requirements are reviewed.\n\n",
        catalog::ROUTE
    ));
    let read_from = |format: SourceFormat| {
        summary
            .rows
            .iter()
            .filter(|row| row.format == format)
            .count()
    };
    let mut totals: BTreeMap<String, u64> = BTreeMap::new();
    for row in &summary.rows {
        for (level, n) in &row.counts {
            *totals.entry(level.clone()).or_default() += n;
        }
    }
    page.push_str(&format!(
        "## Summary\n\n- Index SHA-256: `{}`\n- Extractor version: {}\n- RFCs: {}\n- RFCs with at least one requirement: {}\n- Rows read from RFCXML: {}\n- Rows read from plain text: {}\n- Requirements by level: {}\n\n",
        summary.header.index_sha256,
        summary.header.extractor,
        summary.rows.len(),
        summary
            .rows
            .iter()
            .filter(|row| row.counts.values().any(|n| *n > 0))
            .count(),
        read_from(SourceFormat::Xml),
        read_from(SourceFormat::Text),
        counted(&totals),
    ));
    page.push_str("## Reviewed profiles\n\n");
    if profiles.is_empty() {
        page.push_str("No profile locks an RFC yet.\n\n");
    } else {
        page.push_str("| Profile | RFCs |\n|---|---|\n");
        for (name, rfcs) in profiles {
            let rfcs = rfcs
                .iter()
                .map(|rfc| links.rfc(*rfc, &format!("RFC {rfc}")))
                .collect::<Vec<_>>()
                .join(", ");
            page.push_str(&format!(
                "| [{}]({}/{name}) | {rfcs} |\n",
                cell(name),
                catalog::ROUTE
            ));
        }
        page.push('\n');
    }
    page.push_str("## Ranges\n\n| Range | RFCs |\n|---|---|\n");
    for (start, rows) in ranges {
        page.push_str(&format!(
            "| [{}]({}) | {} |\n",
            label(*start),
            route(*start),
            rows.len()
        ));
    }
    page
}

/// A range page; its sidebar position follows from its first number alone, so a change to another
/// range leaves it as it is.
fn range_page(start: u32, rows: &[&BulkRow], links: &Links<'_>) -> String {
    let label = label(start);
    let mut page = front_matter(&Meta {
        title: &format!("RFC index: {label}"),
        label: &label,
        description: &format!(
            "The {} RFCs from {label} that the RFC Editor publishes as plain text or RFCXML, with their BCP 14 requirement counts by level, read from their RFCXML where it exists, extracted mechanically and unreviewed.",
            rows.len()
        ),
        position: start / 1000 + 1,
    });
    page.push_str(&format!(
        ":::note[Generated]\nThis page is generated by `rfcspec-docs` from the bulk summary [`{SUMMARY}`]({SOURCE}/{SUMMARY}) on `main`. The [RFC index]({ROUTE}) describes the columns.\n:::\n\n"
    ));
    page.push_str("| RFC | Title | Status | Stream | Area | Date | Obsoleted by | Updated by | Requirements | Source | Profile |\n|---|---|---|---|---|---|---|---|---|---|---|\n");
    for row in rows {
        page.push_str(&format!(
            "| [{n}]({RFC_EDITOR}rfc{n}) [explorer]({}) | {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            crate::explorer::link(row.rfc),
            cell(&row.title),
            cell(&row.status),
            cell(&row.stream),
            row.area.as_deref().map_or_else(|| "-".to_owned(), cell),
            cell(&row.date),
            links.numbers(&row.obsoleted_by),
            links.numbers(&row.updated_by),
            counted(&row.counts),
            source(row),
            links.profiles(row.rfc),
            n = row.rfc,
        ));
    }
    page
}

/// Every RFC index page, by path relative to the repository root; none without a summary.
pub fn outputs(root: &Path) -> Result<BTreeMap<String, String>> {
    let path = root.join(SUMMARY);
    if !path.is_file() {
        return Ok(BTreeMap::new());
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {SUMMARY}"))?;
    let summary = parse(&text).with_context(|| format!("reading {SUMMARY}"))?;
    Ok(render(&summary, &locked(root)?))
}

#[cfg(test)]
mod tests {
    use std::{
        collections::{BTreeMap, BTreeSet},
        fs,
        path::{Path, PathBuf},
    };

    use rfcspec_extract::{EXTRACTOR_VERSION, extract, level_counts};
    use rfcspec_model::{BulkRow, SourceFormat, bulk_summary, sha256_hex};
    use rfcspec_source::{corpus::bulk_format, index::parse_index};

    use super::*;
    use crate::page::passive_problems;

    fn repository() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    /// The `story:bulk-summary-format` fixture: the summary `rfcspec bulk summarise` writes for
    /// the fixture index, the profiles' locked texts and RFC 9220's XML fixture, built here
    /// through the same functions, plus the rows given.
    fn fixture_summary(extra: &[BulkRow]) -> String {
        let repository = repository();
        let xml = fs::read_to_string(
            repository.join("crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml"),
        )
        .unwrap();
        let sources = [
            (868, "profiles/time/sources/rfc868.txt"),
            (1035, "profiles/dns/sources/rfc1035.txt"),
            (5905, "profiles/ntp/sources/rfc5905.txt"),
            (6455, "profiles/websocket/sources/rfc6455.txt"),
            (9220, "crates/rfcspec-doc/tests/fixtures/rfc9220.xml"),
        ];
        let mut rows: Vec<BulkRow> = parse_index(&xml)
            .unwrap()
            .into_iter()
            .filter_map(|entry| bulk_format(&entry).map(|format| (format, entry)))
            .map(|(format, entry)| {
                let source = sources.iter().find(|(n, _)| *n == entry.rfc).unwrap().1;
                let bytes = fs::read(repository.join(source)).unwrap();
                let sha256 = sha256_hex(&bytes);
                let text = String::from_utf8(bytes).unwrap();
                let document = rfcspec_doc::read_format(entry.rfc, format, &sha256, &text).unwrap();
                BulkRow {
                    format,
                    counts: level_counts(&extract(&document)),
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
                    lossy: false,
                }
            })
            .collect();
        rows.extend(extra.iter().cloned());
        rows.sort_by_key(|row| row.rfc);
        bulk_summary(&sha256_hex(xml.as_bytes()), EXTRACTOR_VERSION, &rows).unwrap()
    }

    /// A synthetic row: RFC 8441, which updates RFC 6455, with a title an MDX reader would execute.
    fn braced() -> BulkRow {
        BulkRow {
            rfc: 8441,
            title: "A {braced} <Fixture> | Protocol".into(),
            status: "PROPOSED STANDARD".into(),
            stream: "IETF".into(),
            area: None,
            date: "2018-09".into(),
            obsoletes: vec![],
            obsoleted_by: vec![],
            updates: vec![6455],
            updated_by: vec![],
            sha256: "0".repeat(64),
            counts: BTreeMap::from([("MAY".into(), 1), ("MUST".into(), 2)]),
            format: SourceFormat::Text,
            lossy: false,
        }
    }

    fn profiles() -> Vec<(String, Vec<u32>)> {
        locked(&repository()).unwrap()
    }

    /// The table rows of a page, header and separator dropped.
    fn rows(page: &str) -> Vec<&str> {
        page.lines()
            .filter(|l| l.starts_with('|'))
            .skip(2)
            .collect()
    }

    /// The RFC number of a range-page row.
    fn number(row: &str) -> u32 {
        row.trim_start_matches("| [")
            .split(']')
            .next()
            .unwrap()
            .parse()
            .unwrap()
    }

    #[test]
    fn the_profiles_lock_868_1035_5905_and_6455() {
        assert_eq!(
            profiles(),
            [
                ("dns".to_owned(), vec![1035]),
                ("ntp".to_owned(), vec![5905]),
                ("time".to_owned(), vec![868]),
                ("websocket".to_owned(), vec![6455]),
            ]
        );
    }

    #[test]
    fn the_fixture_summary_renders_an_overview_and_one_page_per_thousand() {
        let summary = parse(&fixture_summary(&[braced()])).unwrap();
        let pages = render(&summary, &profiles());
        assert_eq!(
            pages.keys().collect::<Vec<_>>(),
            [
                "website/docs/catalog/rfcs/00000.md",
                "website/docs/catalog/rfcs/01000.md",
                "website/docs/catalog/rfcs/05000.md",
                "website/docs/catalog/rfcs/06000.md",
                "website/docs/catalog/rfcs/08000.md",
                "website/docs/catalog/rfcs/09000.md",
                "website/docs/catalog/rfcs/index.md",
            ]
        );
        assert_eq!(
            rows(&pages["website/docs/catalog/rfcs/05000.md"]),
            [
                "| [5905](https://www.rfc-editor.org/rfc/rfc5905) [explorer](pathname:///rfcspec/explorer/?rfc=5905) | Network Time Protocol Version 4: Protocol and Algorithms Specification | PROPOSED STANDARD | IETF | int | 2010-06 |  | 7822, 8573, 9109, 9748, 9769 | 6 MUST, 1 MUST NOT, 3 SHOULD, 3 MAY | [text](https://www.rfc-editor.org/rfc/rfc5905.txt) | [ntp](/docs/catalog/ntp) |"
            ]
        );

        let range = &pages["website/docs/catalog/rfcs/06000.md"];
        assert!(range.starts_with("---\n# generated by rfcspec-docs, do not edit\n"));
        assert!(
            range.contains("\n| RFC | Title | Status | Stream | Area | Date | Obsoleted by | Updated by | Requirements | Source | Profile |\n|---|---|---|---|---|---|---|---|---|---|---|\n"),
            "{range}"
        );
        assert_eq!(
            rows(range),
            [
                "| [6455](https://www.rfc-editor.org/rfc/rfc6455) [explorer](pathname:///rfcspec/explorer/?rfc=6455) | The WebSocket Protocol | PROPOSED STANDARD | IETF | art | 2011-12 |  | 7936, 8307, [8441](/docs/catalog/rfcs/08000) | 108 MUST, 23 MUST NOT, 18 SHOULD, 4 SHOULD NOT, 42 MAY, 1 OPTIONAL | [text](https://www.rfc-editor.org/rfc/rfc6455.txt) | [websocket](/docs/catalog/websocket) |"
            ]
        );
        assert_eq!(
            rows(&pages["website/docs/catalog/rfcs/00000.md"]),
            [
                "| [868](https://www.rfc-editor.org/rfc/rfc868) [explorer](pathname:///rfcspec/explorer/?rfc=868) | Time Protocol | INTERNET STANDARD | Legacy | - | 1983-05 |  |  | none | [text](https://www.rfc-editor.org/rfc/rfc868.txt) | [time](/docs/catalog/time) |"
            ]
        );
        assert_eq!(
            rows(&pages["website/docs/catalog/rfcs/08000.md"]),
            [
                "| [8441](https://www.rfc-editor.org/rfc/rfc8441) [explorer](pathname:///rfcspec/explorer/?rfc=8441) | A \\{braced\\} &lt;Fixture> \\| Protocol | PROPOSED STANDARD | IETF | - | 2018-09 |  |  | 2 MUST, 1 MAY | [text](https://www.rfc-editor.org/rfc/rfc8441.txt) |  |"
            ]
        );
        assert_eq!(
            rows(&pages["website/docs/catalog/rfcs/09000.md"]),
            [
                "| [9220](https://www.rfc-editor.org/rfc/rfc9220) [explorer](pathname:///rfcspec/explorer/?rfc=9220) | Bootstrapping WebSockets with HTTP/3 | PROPOSED STANDARD | IETF | wit | 2022-06 |  |  | 1 SHOULD, 1 MAY | [XML](https://www.rfc-editor.org/rfc/rfc9220.xml) |  |"
            ]
        );

        let overview = &pages["website/docs/catalog/rfcs/index.md"];
        let index_sha256 = &summary.header.index_sha256;
        for line in [
            format!("- Index SHA-256: `{index_sha256}`"),
            format!("- Extractor version: {EXTRACTOR_VERSION}"),
            "- RFCs: 6".to_owned(),
            "- RFCs with at least one requirement: 4".to_owned(),
            "- Rows read from RFCXML: 1".to_owned(),
            "- Rows read from plain text: 5".to_owned(),
            "- Requirements by level: 116 MUST, 24 MUST NOT, 22 SHOULD, 4 SHOULD NOT, 47 MAY, 1 OPTIONAL".to_owned(),
            "| [dns](/docs/catalog/dns) | [RFC 1035](/docs/catalog/rfcs/01000) |".to_owned(),
            "| [ntp](/docs/catalog/ntp) | [RFC 5905](/docs/catalog/rfcs/05000) |".to_owned(),
            "| [time](/docs/catalog/time) | [RFC 868](/docs/catalog/rfcs/00000) |".to_owned(),
            "| [websocket](/docs/catalog/websocket) | [RFC 6455](/docs/catalog/rfcs/06000) |".to_owned(),
            "| [RFC 1 to 999](/docs/catalog/rfcs/00000) | 1 |".to_owned(),
            "| [RFC 1000 to 1999](/docs/catalog/rfcs/01000) | 1 |".to_owned(),
            "| [RFC 5000 to 5999](/docs/catalog/rfcs/05000) | 1 |".to_owned(),
            "| [RFC 6000 to 6999](/docs/catalog/rfcs/06000) | 1 |".to_owned(),
            "| [RFC 8000 to 8999](/docs/catalog/rfcs/08000) | 1 |".to_owned(),
            "| [RFC 9000 to 9999](/docs/catalog/rfcs/09000) | 1 |".to_owned(),
        ] {
            assert!(
                overview.lines().any(|l| l == line),
                "{line} not in\n{overview}"
            );
        }
        for prose in [
            "lists as plain text or RFCXML",
            "counted in its RFCXML where the index lists one, else in its plain text",
            "Source names the format read, linked to the RFC Editor file the counts came from",
        ] {
            assert!(overview.contains(prose), "{prose} not in\n{overview}");
        }
        assert!(!overview.contains("with a plain-text format"), "{overview}");
        for page in pages.values() {
            assert!(
                !page.contains("publishes as plain text,"),
                "a description still names plain text alone\n{page}"
            );
        }

        for (path, page) in &pages {
            assert!(passive_problems(page).is_empty(), "{path}\n{page}");
        }
    }

    #[test]
    fn a_summary_whose_header_does_not_match_is_refused() {
        let text = fixture_summary(&[]);
        assert!(parse(&text).is_ok());
        assert!(parse(&text.replacen("rfcspec-bulk/1", "rfcspec-bulk/2", 1)).is_err());
        assert!(text.contains("\"rows\":5"), "{text}");
        assert!(parse(&text.replacen("\"rows\":5", "\"rows\":6", 1)).is_err());
        let error = parse(&format!("{text}not json\n")).unwrap_err().to_string();
        assert!(error.contains("line 7"), "{error}");
    }

    #[test]
    fn the_repository_rfc_index_has_eleven_ranges_covering_every_row_under_4_mib() {
        let root = repository();
        let pages = outputs(&root).unwrap();
        let summary = parse(&fs::read_to_string(root.join(SUMMARY)).unwrap()).unwrap();
        let ranges: Vec<&String> = pages.keys().filter(|p| *p != OVERVIEW).collect();
        assert_eq!(ranges.len(), 11, "{ranges:?}");
        assert_eq!(ranges[0], "website/docs/catalog/rfcs/00000.md");
        assert_eq!(ranges[10], "website/docs/catalog/rfcs/10000.md");
        let mut seen = Vec::new();
        for path in &ranges {
            let start: u32 = path
                .trim_start_matches("website/docs/catalog/rfcs/")
                .trim_end_matches(".md")
                .parse()
                .unwrap();
            let numbers: Vec<u32> = rows(&pages[*path]).into_iter().map(number).collect();
            assert!(numbers.windows(2).all(|w| w[0] < w[1]), "{path}");
            assert!(numbers.iter().all(|n| n / 1000 * 1000 == start), "{path}");
            seen.extend(numbers);
        }
        let expected: Vec<u32> = summary.rows.iter().map(|r| r.rfc).collect();
        assert_eq!(seen, expected);
        assert_eq!(
            seen.iter().collect::<BTreeSet<_>>().len(),
            summary.rows.len()
        );
        for (rfc, profile) in [(6455, "websocket"), (1035, "dns"), (868, "time")] {
            let page = &pages[&format!("website/docs/catalog/rfcs/{:05}.md", rfc / 1000 * 1000)];
            let row = rows(page).into_iter().find(|r| number(r) == rfc).unwrap();
            assert!(
                row.ends_with(&format!("| [{profile}](/docs/catalog/{profile}) |")),
                "{row}"
            );
        }
        let mut read = BTreeMap::new();
        for row in &summary.rows {
            let page = &pages[&format!("website/docs/catalog/rfcs/{:05}.md", range(row.rfc))];
            let line = rows(page)
                .into_iter()
                .find(|r| number(r) == row.rfc)
                .unwrap();
            let (name, extension) = match row.format {
                SourceFormat::Xml => ("XML", "xml"),
                SourceFormat::Text => ("text", "txt"),
            };
            assert!(
                line.contains(&format!(
                    " | [{name}](https://www.rfc-editor.org/rfc/rfc{}.{extension}) | ",
                    row.rfc
                )),
                "{line}"
            );
            assert!(
                line.starts_with(&format!(
                    "| [{n}](https://www.rfc-editor.org/rfc/rfc{n}) [explorer](pathname:///rfcspec/explorer/?rfc={n}) | ",
                    n = row.rfc
                )),
                "{line}"
            );
            *read.entry(name).or_insert(0usize) += 1;
        }
        let overview = &pages[OVERVIEW];
        for (name, label) in [("XML", "RFCXML"), ("text", "plain text")] {
            let line = format!("- Rows read from {label}: {}", read.get(name).unwrap_or(&0));
            assert!(overview.lines().any(|l| l == line), "{line}");
        }
        let rfc9220 = summary.rows.iter().find(|r| r.rfc == 9220).unwrap();
        assert_eq!(rfc9220.format, SourceFormat::Xml);
        assert_eq!(
            rfc9220.sha256,
            "8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae"
        );
        let total: usize = pages.values().map(String::len).sum();
        assert!(total < 4 * 1024 * 1024, "{total} bytes");
    }
}
