//! Adversarial cases for the section sidecar: tampered sidecars `rfcspec bulk check --sections`
//! should refuse, and a wrapped section heading `rfcspec bulk summarise --sections` should keep
//! whole. RFC 9220 is the target of the tampering because no profile locks it, so only the
//! structural checks stand between an edit and the shards.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use rfcspec_model::sha256_hex;
use serde_json::{Value, json};
use tempfile::TempDir;

const INDEX: &str = "crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml";
const XML: &str = "crates/rfcspec-doc/tests/fixtures/rfc9220.xml";

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
        .prefix(&format!("sections-adversary-{name}-"))
        .tempdir_in(scratch)
        .unwrap()
}

/// The fixture inputs in `dir`, RFC 6455's text passed through `websocket`.
fn inputs(dir: &Path, websocket: impl FnOnce(String) -> String) -> (PathBuf, PathBuf) {
    let texts = dir.join("texts");
    let xml = dir.join("xml");
    fs::create_dir_all(&texts).unwrap();
    fs::create_dir_all(&xml).unwrap();
    for (profile, rfc) in [("time", 868), ("dns", 1035), ("ntp", 5905)] {
        fs::copy(
            repository().join(format!("profiles/{profile}/sources/rfc{rfc}.txt")),
            texts.join(format!("rfc{rfc}.txt")),
        )
        .unwrap();
    }
    let text =
        fs::read_to_string(repository().join("profiles/websocket/sources/rfc6455.txt")).unwrap();
    fs::write(texts.join("rfc6455.txt"), websocket(text)).unwrap();
    fs::copy(repository().join(XML), xml.join("rfc9220.xml")).unwrap();
    (texts, xml)
}

fn summarise(dir: &Path, websocket: impl FnOnce(String) -> String) -> (PathBuf, PathBuf) {
    let (texts, xml) = inputs(dir, websocket);
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

fn lines(path: &Path) -> Vec<String> {
    fs::read_to_string(path)
        .unwrap()
        .lines()
        .map(str::to_string)
        .collect()
}

fn reheader(lines: &mut [String]) {
    let rows: String = lines[1..].iter().map(|line| format!("{line}\n")).collect();
    let mut header: Value = serde_json::from_str(&lines[0]).unwrap();
    header["rows"] = json!(lines.len() - 1);
    header["rows_sha256"] = json!(sha256_hex(rows.as_bytes()));
    lines[0] = serde_json::to_string(&header).unwrap();
}

/// Check the fixture summary with RFC 9220's sidecar row changed by `change` and the header
/// recomputed; returns the exit code and stderr.
fn check_tampered(name: &str, change: impl FnOnce(&mut Value)) -> (Option<i32>, String) {
    let dir = scratch(name);
    let (summary, sections) = summarise(dir.path(), |text| text);
    let mut lines = lines(&sections);
    let at = lines
        .iter()
        .position(|line| line.starts_with("{\"rfc\":9220,"))
        .unwrap();
    let mut row: Value = serde_json::from_str(&lines[at]).unwrap();
    change(&mut row);
    lines[at] = serde_json::to_string(&row).unwrap();
    reheader(&mut lines);
    let mut text = lines.join("\n");
    text.push('\n');
    fs::write(&sections, text).unwrap();
    let output = rfcspec(&[
        "bulk",
        "check",
        "--summary",
        summary.to_str().unwrap(),
        "--profiles",
        "profiles",
        "--sections",
        sections.to_str().unwrap(),
    ]);
    (output.status.code(), stderr(&output))
}

#[test]
fn check_refuses_a_row_that_lists_one_section_number_twice() {
    let (code, error) = check_tampered("duplicate-section", |row| {
        row["sections"] = json!([
            {"number": "3", "title": "WebSockets Upgrade over HTTP/3", "counts": {"SHOULD": 1}},
            {"number": "3", "title": "WebSockets Upgrade over HTTP/3", "counts": {"MAY": 1}}
        ]);
    });
    assert_eq!(
        code,
        Some(1),
        "section 3 listed twice passes the check: {error}"
    );
}

#[test]
fn check_refuses_a_section_that_holds_no_requirement() {
    let (code, error) = check_tampered("zero-section", |row| {
        row["sections"]
            .as_array_mut()
            .unwrap()
            .push(json!({"number": "4", "title": "Security Considerations", "counts": {"MAY": 0}}));
    });
    assert_eq!(
        code,
        Some(1),
        "a section counting 0 MAY passes the check: {error}"
    );
}

#[test]
fn check_refuses_the_entry_without_a_number_anywhere_but_last_and_more_than_once() {
    let (code, error) = check_tampered("numberless-first", |row| {
        row["sections"] = json!([
            {"counts": {"SHOULD": 1}},
            {"number": "3", "title": "WebSockets Upgrade over HTTP/3", "counts": {"MAY": 1}}
        ]);
    });
    assert_eq!(
        code,
        Some(1),
        "the numberless entry first passes the check: {error}"
    );
    let (code, error) = check_tampered("numberless-twice", |row| {
        row["sections"] = json!([{"counts": {"SHOULD": 1}}, {"counts": {"MAY": 1}}]);
    });
    assert_eq!(
        code,
        Some(1),
        "two numberless entries pass the check: {error}"
    );
}

#[test]
fn summarise_keeps_a_section_heading_that_wraps_onto_a_second_line_whole() {
    let dir = scratch("wrapped-heading");
    let (_, sections) = summarise(dir.path(), |text| {
        let wrapped = text.replacen(
            "\n5.2.  Base Framing Protocol\n",
            "\n5.2.  Base Framing Protocol for Frames Sent Between a Client and a\n      Server\n",
            1,
        );
        assert_ne!(wrapped, text, "the heading of section 5.2 was found");
        wrapped
    });
    let row: Value = lines(&sections)
        .iter()
        .skip(1)
        .map(|line| serde_json::from_str::<Value>(line).unwrap())
        .find(|row| row["rfc"] == 6455)
        .unwrap();
    let framing = row["sections"]
        .as_array()
        .unwrap()
        .iter()
        .find(|section| section["number"] == "5.2")
        .unwrap()
        .clone();
    assert_eq!(
        framing["title"],
        "Base Framing Protocol for Frames Sent Between a Client and a Server"
    );
}
