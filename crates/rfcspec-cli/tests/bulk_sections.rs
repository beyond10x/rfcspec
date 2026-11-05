//! The section sidecar (`rfcspec-sections/1`) that `rfcspec bulk summarise --sections` writes
//! beside the summary and `rfcspec bulk check --sections` checks offline: over the fixture index,
//! the texts the profiles lock and RFC 9220's RFCXML, and over the committed `bulk/` files.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rfcspec_model::sha256_hex;
use serde_json::{Value, json};
use tempfile::TempDir;

const INDEX: &str = "crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml";
const XML: &str = "crates/rfcspec-doc/tests/fixtures/rfc9220.xml";
const COMMITTED_SUMMARY: &str = "bulk/rfc-summary.jsonl";
const COMMITTED_SECTIONS: &str = "bulk/rfc-sections.jsonl";

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn rfcspec(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rfcspec"))
        .args(args)
        .current_dir(repository())
        .output()
        .unwrap()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

fn scratch(name: &str) -> TempDir {
    let scratch = repository().join("target/rfcspec-cli");
    fs::create_dir_all(&scratch).unwrap();
    tempfile::Builder::new()
        .prefix(&format!("sections-{name}-"))
        .tempdir_in(scratch)
        .unwrap()
}

/// The fixture inputs copied into `dir`: the four locked texts and RFC 9220's XML.
fn inputs(dir: &Path) -> (PathBuf, PathBuf) {
    let texts = dir.join("texts");
    let xml = dir.join("xml");
    fs::create_dir_all(&texts).unwrap();
    fs::create_dir_all(&xml).unwrap();
    for (profile, rfc) in [
        ("time", 868),
        ("dns", 1035),
        ("ntp", 5905),
        ("websocket", 6455),
    ] {
        fs::copy(
            repository().join(format!("profiles/{profile}/sources/rfc{rfc}.txt")),
            texts.join(format!("rfc{rfc}.txt")),
        )
        .unwrap();
    }
    fs::copy(repository().join(XML), xml.join("rfc9220.xml")).unwrap();
    (texts, xml)
}

/// The summary and sidecar of the fixture inputs, written into `dir`.
fn summarise(dir: &Path) -> (PathBuf, PathBuf) {
    let (texts, xml) = inputs(dir);
    let summary = dir.join("rfc-summary.jsonl");
    let sections = dir.join("rfc-sections.jsonl");
    let output = rfcspec(&[
        "bulk",
        "summarise",
        "--index",
        repository().join(INDEX).to_str().unwrap(),
        "--texts",
        texts.to_str().unwrap(),
        "--xml",
        xml.to_str().unwrap(),
        "--out",
        summary.to_str().unwrap(),
        "--sections",
        sections.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    (summary, sections)
}

fn check(summary: &Path, sections: &Path) -> Output {
    rfcspec(&[
        "bulk",
        "check",
        "--summary",
        summary.to_str().unwrap(),
        "--profiles",
        "profiles",
        "--sections",
        sections.to_str().unwrap(),
    ])
}

fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

fn write_lines(path: &Path, lines: &[String]) {
    let mut text = lines.join("\n");
    text.push('\n');
    fs::write(path, text).unwrap();
}

/// The header with `rows` and `rows_sha256` recomputed over the row lines.
fn reheader(lines: &mut [String]) {
    let rows: String = lines[1..].iter().map(|line| format!("{line}\n")).collect();
    let mut header: Value = serde_json::from_str(&lines[0]).unwrap();
    header["rows"] = json!(lines.len() - 1);
    header["rows_sha256"] = json!(sha256_hex(rows.as_bytes()));
    lines[0] = serde_json::to_string(&header).unwrap();
}

fn row(lines: &[String], rfc: u64) -> (usize, Value) {
    lines
        .iter()
        .enumerate()
        .skip(1)
        .map(|(at, line)| (at, serde_json::from_str::<Value>(line).unwrap()))
        .find(|(_, row)| row["rfc"] == rfc)
        .unwrap()
}

fn set_row(lines: &mut [String], rfc: u64, change: impl FnOnce(&mut Value)) {
    let (at, mut value) = row(lines, rfc);
    change(&mut value);
    lines[at] = serde_json::to_string(&value).unwrap();
}

/// The levels of every section of a sidecar row added up.
fn summed(row: &Value) -> Value {
    let mut sum: BTreeMap<String, u64> = BTreeMap::new();
    for section in row["sections"].as_array().into_iter().flatten() {
        for (level, n) in section["counts"].as_object().unwrap() {
            *sum.entry(level.clone()).or_default() += n.as_u64().unwrap();
        }
    }
    serde_json::to_value(sum).unwrap()
}

/// What `rfcspec read` gives as RFC 9220's References, from a profile locking the XML fixture.
fn read_references(dir: &Path) -> Value {
    let profile = dir.join("h3");
    rfcspec_source::store_format(
        &profile,
        9220,
        rfcspec_model::SourceFormat::Xml,
        &fs::read(repository().join(XML)).unwrap(),
    )
    .unwrap();
    fs::write(
        profile.join("profile.yaml"),
        "name: h3\nrfcs:\n- number: 9220\n  title: Bootstrapping WebSockets with HTTP/3\n  sections:\n  - '3'\n",
    )
    .unwrap();
    let output = rfcspec(&[
        "read",
        "--profile",
        profile.to_str().unwrap(),
        "--rfc",
        "9220",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    document["references"].clone()
}

#[test]
fn summarise_writes_a_sidecar_beside_the_summary_that_check_accepts() {
    let dir = scratch("summarise");
    let (summary, sections) = summarise(dir.path());
    let output = check(&summary, &sections);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let summary_lines = lines(&summary);
    let lines = lines(&sections);
    let header: Value = serde_json::from_str(&lines[0]).unwrap();
    let summary_header: Value = serde_json::from_str(&summary_lines[0]).unwrap();
    assert_eq!(header["format"], "rfcspec-sections/1");
    assert_eq!(header["index_sha256"], summary_header["index_sha256"]);
    assert_eq!(header["extractor"], summary_header["extractor"]);
    assert_eq!(header["rows"], 5);
    let numbers: Vec<Value> = lines[1..]
        .iter()
        .map(|line| serde_json::from_str::<Value>(line).unwrap()["rfc"].clone())
        .collect();
    assert_eq!(numbers, [868, 1035, 5905, 6455, 9220]);

    for rfc in [868, 1035, 5905, 6455, 9220] {
        let (_, summary_row) = row(&summary_lines, rfc);
        let (_, sections_row) = row(&lines, rfc);
        assert_eq!(summed(&sections_row), summary_row["counts"], "RFC {rfc}");
    }

    let (_, time) = row(&lines, 868);
    assert_eq!(time, json!({"rfc": 868}));

    let (_, websocket) = row(&lines, 6455);
    assert!(websocket.get("references").is_none(), "{websocket}");
    let framing = websocket["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|section| section["number"] == "5.2")
        .unwrap();
    assert_eq!(framing["title"], "Base Framing Protocol");
    assert!(framing.get("anchor").is_none(), "{framing}");

    let (_, h3) = row(&lines, 9220);
    assert_eq!(h3["references"], read_references(dir.path()));
    assert!(
        h3["references"]
            .as_array()
            .unwrap()
            .contains(&json!({"section": "introduction", "target_rfc": 8441, "kind": "normative"})),
        "{h3}"
    );
    let upgrade = h3["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|section| section["number"] == "3")
        .unwrap();
    assert_eq!(upgrade["anchor"], "websockets-upgrade-over-http3");
    assert_eq!(upgrade["title"], "WebSockets Upgrade over HTTP/3");
}

/// Check the fixture summary with its sidecar changed by `change`; it must exit 1.
fn refused(name: &str, change: impl FnOnce(&mut Vec<String>)) -> String {
    let dir = scratch(name);
    let (summary, sections) = summarise(dir.path());
    let mut lines = lines(&sections);
    change(&mut lines);
    write_lines(&sections, &lines);
    let output = check(&summary, &sections);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    stderr(&output)
}

#[test]
fn check_refuses_a_sidecar_row_edited_under_its_header() {
    let error = refused("edited", |lines| {
        set_row(lines, 9220, |row| {
            row["sections"][0]["title"] = json!("Edited")
        });
    });
    assert!(error.contains("rfc-sections.jsonl"), "{error}");
    assert!(error.contains("line 1"), "{error}");
    assert!(error.contains("rows_sha256"), "{error}");
}

#[test]
fn check_refuses_a_sidecar_whose_row_count_differs_from_its_header() {
    let error = refused("rows", |lines| {
        let mut header: Value = serde_json::from_str(&lines[0]).unwrap();
        header["rows"] = json!(6);
        lines[0] = serde_json::to_string(&header).unwrap();
    });
    assert!(error.contains("rows 6"), "{error}");
}

#[test]
fn check_refuses_another_sidecar_format() {
    let error = refused("format", |lines| {
        let mut header: Value = serde_json::from_str(&lines[0]).unwrap();
        header["format"] = json!("rfcspec-bulk/1");
        lines[0] = serde_json::to_string(&header).unwrap();
    });
    assert!(error.contains("rfcspec-sections/1"), "{error}");
}

#[test]
fn check_refuses_a_sidecar_of_another_index() {
    let error = refused("index", |lines| {
        let mut header: Value = serde_json::from_str(&lines[0]).unwrap();
        header["index_sha256"] = json!("a".repeat(64));
        lines[0] = serde_json::to_string(&header).unwrap();
    });
    assert!(error.contains("index_sha256"), "{error}");
    assert!(error.contains(&"a".repeat(64)), "{error}");
}

#[test]
fn check_refuses_a_sidecar_without_a_summary_rfc_naming_it() {
    let error = refused("missing", |lines| {
        let (at, _) = row(lines, 5905);
        lines.remove(at);
        reheader(lines);
    });
    assert!(error.contains("RFC 5905"), "{error}");
    assert!(error.contains("no row"), "{error}");
}

#[test]
fn check_refuses_a_sidecar_with_an_rfc_the_summary_lacks_naming_it() {
    let error = refused("extra", |lines| {
        lines.insert(5, "{\"rfc\":7000}".into());
        reheader(lines);
    });
    assert!(error.contains("RFC 7000"), "{error}");
    assert!(error.contains("not in the summary"), "{error}");
}

#[test]
fn check_refuses_sidecar_rows_out_of_summary_order() {
    let error = refused("order", |lines| {
        lines.swap(2, 3);
        reheader(lines);
    });
    assert!(error.contains("ascending"), "{error}");
}

#[test]
fn check_refuses_section_counts_that_do_not_add_up_to_the_summary_counts() {
    let error = refused("sum", |lines| {
        set_row(lines, 9220, |row| {
            row["sections"][0]["counts"]["MAY"] = json!(5)
        });
        reheader(lines);
    });
    assert!(error.contains("RFC 9220"), "{error}");
    assert!(error.contains("add up"), "{error}");
}

#[test]
fn check_refuses_references_on_a_row_read_from_text() {
    let error = refused("text-references", |lines| {
        set_row(lines, 868, |row| {
            row["references"] = json!([{"target_rfc": 793, "kind": "normative"}]);
        });
        reheader(lines);
    });
    assert!(error.contains("RFC 868"), "{error}");
    assert!(error.contains("plain text"), "{error}");
}

#[test]
fn check_refuses_locked_sections_that_differ_from_reading_the_locked_source() {
    let error = refused("locked", |lines| {
        set_row(lines, 6455, |row| {
            row["sections"][0]["title"] = json!("Renamed");
        });
        reheader(lines);
    });
    assert!(error.contains("RFC 6455"), "{error}");
    assert!(error.contains("websocket/rfcspec.lock"), "{error}");
}

#[test]
fn the_committed_sidecar_holds_against_the_committed_summary() {
    let output = check(
        &repository().join(COMMITTED_SUMMARY),
        &repository().join(COMMITTED_SECTIONS),
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
}
