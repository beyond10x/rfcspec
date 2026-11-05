//! `rfcspec bulk fetch`, `rfcspec bulk summarise` and `rfcspec bulk check` as a user runs them,
//! over a small committed copy of `rfc-index.xml`, the texts the profiles lock, a 127.0.0.1 HTTP
//! server and the committed `bulk/rfc-summary.jsonl`.

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output},
    sync::{Arc, Mutex},
    thread,
};

use rfcspec_model::sha256_hex;
use serde_json::{Value, json};
use tempfile::TempDir;

const INDEX: &str = "crates/rfcspec-cli/tests/fixtures/bulk/rfc-index.xml";

/// RFC 9220's RFCXML v3 source, the one entry of the fixture index with an XML format.
const XML: &str = "crates/rfcspec-doc/tests/fixtures/rfc9220.xml";

const XML_SHA256: &str = "8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae";

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

/// A fresh directory under the workspace's build directory, not the system temporary one,
/// removed when the guard drops.
fn scratch(name: &str) -> TempDir {
    let scratch = repository().join("target/rfcspec-cli");
    fs::create_dir_all(&scratch).unwrap();
    tempfile::Builder::new()
        .prefix(&format!("bulk-{name}-"))
        .tempdir_in(scratch)
        .unwrap()
}

/// The four texts the profiles lock, copied into `<dir>/texts` as `rfc<N>.txt`.
fn texts(dir: &Path) -> PathBuf {
    let texts = dir.join("texts");
    fs::create_dir_all(&texts).unwrap();
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
    texts
}

/// RFC 9220's XML source copied into `<dir>/xml` as `rfc9220.xml`.
fn xml(dir: &Path) -> PathBuf {
    let xml = dir.join("xml");
    fs::create_dir_all(&xml).unwrap();
    fs::copy(repository().join(XML), xml.join("rfc9220.xml")).unwrap();
    xml
}

fn summarise(index: &Path, texts: &Path, xml: &Path, out: &Path) -> Output {
    rfcspec(&[
        "bulk",
        "summarise",
        "--index",
        index.to_str().unwrap(),
        "--texts",
        texts.to_str().unwrap(),
        "--xml",
        xml.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ])
}

fn check(summary: &Path) -> Output {
    check_against(summary, Path::new("profiles"))
}

fn check_against(summary: &Path, profiles: &Path) -> Output {
    rfcspec(&[
        "bulk",
        "check",
        "--summary",
        summary.to_str().unwrap(),
        "--profiles",
        profiles.to_str().unwrap(),
    ])
}

/// The summary of the fixture index, texts and XML, written into `dir`.
fn summary(dir: &Path) -> PathBuf {
    let out = dir.join("summary.jsonl");
    let output = summarise(&repository().join(INDEX), &texts(dir), &xml(dir), &out);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    out
}

/// A profiles directory under `dir`: a copy of the repository's `profiles/time` lock and source,
/// and a profile `h3` locking RFC 9220 as `sources/rfc9220.xml`.
fn xml_profiles(dir: &Path) -> PathBuf {
    let profiles = dir.join("profiles");
    let time = profiles.join("time");
    fs::create_dir_all(&time).unwrap();
    rfcspec_source::store(&time, 868, &locked_text("time", 868)).unwrap();
    rfcspec_source::store_format(
        &profiles.join("h3"),
        9220,
        rfcspec_model::SourceFormat::Xml,
        &fs::read(repository().join(XML)).unwrap(),
    )
    .unwrap();
    profiles
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

/// The header line with `rows` and `rows_sha256` recomputed over the row lines.
fn reheader(lines: &mut [String]) {
    let rows = lines[1..]
        .iter()
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    let mut header: Value = serde_json::from_str(&lines[0]).unwrap();
    header["rows"] = json!(lines.len() - 1);
    header["rows_sha256"] = json!(sha256_hex(rows.as_bytes()));
    lines[0] = serde_json::to_string(&header).unwrap();
}

/// The counts by level of `rfcspec extract` on a profile.
fn extracted_counts(profile: &str, rfc: &str) -> Value {
    let output = rfcspec(&["extract", "--profile", profile, "--rfc", rfc]);
    assert!(output.status.success(), "{}", stderr(&output));
    let catalogue: Value = serde_json::from_slice(&output.stdout).unwrap();
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for requirement in catalogue["requirements"].as_array().unwrap() {
        *counts
            .entry(requirement["keyword"].as_str().unwrap().to_string())
            .or_default() += 1;
    }
    serde_json::to_value(counts).unwrap()
}

fn locked_sha256(profile: &str) -> String {
    let lock = fs::read_to_string(repository().join(profile).join("rfcspec.lock")).unwrap();
    lock.lines()
        .find_map(|line| line.trim().strip_prefix("sha256: "))
        .unwrap()
        .to_string()
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

/// The line of RFC `rfc`'s row, verbatim.
fn row_line(lines: &[String], rfc: u64) -> &str {
    &lines[row(lines, rfc).0]
}

#[test]
fn summarise_writes_a_summary_check_accepts_with_the_locked_digests_and_extracted_counts() {
    let dir = scratch("summary");
    let summary = summary(dir.path());
    let output = check(&summary);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let lines = lines(&summary);
    let header: Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(header["format"], "rfcspec-bulk/1");
    assert_eq!(header["extractor"], 2);
    assert_eq!(header["rows"], 5);
    assert_eq!(
        header["index_sha256"],
        sha256_hex(&fs::read(repository().join(INDEX)).unwrap())
    );
    let rows: Vec<Value> = lines[1..]
        .iter()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let numbers: Vec<&Value> = rows.iter().map(|row| &row["rfc"]).collect();
    assert_eq!(
        numbers,
        [
            &json!(868),
            &json!(1035),
            &json!(5905),
            &json!(6455),
            &json!(9220)
        ]
    );
    for text in &rows[..4] {
        assert!(text.get("format").is_none(), "{text}");
    }

    let h3 = &rows[4];
    assert_eq!(h3["format"], "xml");
    assert_eq!(h3["sha256"], XML_SHA256);
    assert_eq!(h3["counts"], json!({"MAY": 1, "SHOULD": 1}));
    assert_eq!(h3["title"], "Bootstrapping WebSockets with HTTP/3");
    assert_eq!(h3["area"], "wit");
    assert_eq!(h3["date"], "2022-06");

    let time = &rows[0];
    assert_eq!(time["counts"], json!({}));
    assert_eq!(time["status"], "INTERNET STANDARD");
    assert_eq!(time["stream"], "Legacy");
    assert!(time.get("area").is_none(), "{time}");
    assert!(time.get("lossy").is_none(), "{time}");
    assert_eq!(time["sha256"], locked_sha256("profiles/time"));

    let dns = &rows[1];
    assert_eq!(dns["sha256"], locked_sha256("profiles/dns"));
    assert_eq!(dns["counts"], extracted_counts("profiles/dns", "1035"));
    assert_eq!(dns["date"], "1987-11");
    assert_eq!(dns["obsoletes"], json!([882, 883, 973]));

    let ntp = &rows[2];
    assert_eq!(ntp["sha256"], locked_sha256("profiles/ntp"));
    assert_eq!(ntp["counts"], extracted_counts("profiles/ntp", "5905"));
    assert_eq!(
        ntp["counts"],
        json!({"MAY": 3, "MUST": 6, "MUST NOT": 1, "SHOULD": 3})
    );

    let websocket = &rows[3];
    assert!(
        websocket["sha256"]
            .as_str()
            .unwrap()
            .starts_with("765775326aee"),
        "{websocket}"
    );
    assert_eq!(websocket["sha256"], locked_sha256("profiles/websocket"));
    assert_eq!(
        websocket["counts"],
        extracted_counts("profiles/websocket", "6455")
    );
    assert_ne!(websocket["counts"], json!({}));
    assert_eq!(websocket["title"], "The WebSocket Protocol");
    assert_eq!(websocket["status"], "PROPOSED STANDARD");
    assert_eq!(websocket["stream"], "IETF");
    assert_eq!(websocket["area"], "art");
    assert_eq!(websocket["date"], "2011-12");
    assert_eq!(websocket["updated_by"], json!([7936, 8307, 8441]));
    assert!(websocket.get("obsoletes").is_none(), "{websocket}");

    let rows_text = lines[1..]
        .iter()
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    assert_eq!(header["rows_sha256"], sha256_hex(rows_text.as_bytes()));
}

#[test]
fn summarise_is_byte_identical_across_two_runs() {
    let dir = scratch("twice");
    let first = fs::read(summary(dir.path())).unwrap();
    let second = fs::read(summary(dir.path())).unwrap();
    assert_eq!(first, second);
}

#[test]
fn summarise_marks_a_text_that_is_not_utf8_lossy() {
    let dir = scratch("lossy");
    let texts = texts(dir.path());
    let path = texts.join("rfc868.txt");
    let mut bytes = fs::read(&path).unwrap();
    bytes.extend_from_slice(b"\xff\xfe\n");
    fs::write(&path, &bytes).unwrap();
    let out = dir.path().join("summary.jsonl");
    let output = summarise(&repository().join(INDEX), &texts, &xml(dir.path()), &out);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines = lines(&out);
    let (_, time) = row(&lines, 868);
    assert_eq!(time["lossy"], true);
    assert_eq!(time["sha256"], sha256_hex(&bytes));
    let (_, dns) = row(&lines, 1035);
    assert!(dns.get("lossy").is_none(), "{dns}");
}

#[test]
fn summarise_exits_1_naming_the_rfc_and_path_of_a_missing_text() {
    let dir = scratch("missing");
    let texts = texts(dir.path());
    fs::remove_file(texts.join("rfc1035.txt")).unwrap();
    let out = dir.path().join("summary.jsonl");
    let output = summarise(&repository().join(INDEX), &texts, &xml(dir.path()), &out);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("RFC 1035"), "{error}");
    assert!(
        error.contains(&texts.join("rfc1035.txt").display().to_string()),
        "{error}"
    );
    assert!(!out.exists());
}

#[test]
fn summarise_exits_1_naming_the_rfc_and_path_of_a_missing_xml() {
    let dir = scratch("missing-xml");
    let xml = xml(dir.path());
    fs::remove_file(xml.join("rfc9220.xml")).unwrap();
    let out = dir.path().join("summary.jsonl");
    let output = summarise(&repository().join(INDEX), &texts(dir.path()), &xml, &out);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("RFC 9220"), "{error}");
    assert!(
        error.contains(&xml.join("rfc9220.xml").display().to_string()),
        "{error}"
    );
    assert!(!out.exists());
}

#[test]
fn summarise_requires_xml_with_index() {
    let dir = scratch("no-xml-arg");
    let out = dir.path().join("summary.jsonl");
    let output = rfcspec(&[
        "bulk",
        "summarise",
        "--index",
        repository().join(INDEX).to_str().unwrap(),
        "--texts",
        texts(dir.path()).to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(2), "{}", stderr(&output));
    assert!(stderr(&output).contains("--xml"), "{}", stderr(&output));
    assert!(!out.exists());
}

#[test]
fn summarise_writes_a_row_for_an_entry_with_xml_and_no_text() {
    let dir = scratch("xml-only");
    let index = dir.path().join("rfc-index.xml");
    fs::write(
        &index,
        r#"<?xml version='1.0' encoding='utf-8'?>
<rfc-index xmlns="https://www.rfc-editor.org/rfc-index">
  <rfc-entry>
    <doc-id>RFC9220</doc-id>
    <title>Bootstrapping WebSockets with HTTP/3</title>
    <date><month>June</month><year>2022</year></date>
    <format><file-format>HTML</file-format><file-format>XML</file-format></format>
    <current-status>PROPOSED STANDARD</current-status>
    <stream>IETF</stream>
  </rfc-entry>
</rfc-index>"#,
    )
    .unwrap();
    let texts = dir.path().join("texts");
    fs::create_dir_all(&texts).unwrap();
    let out = dir.path().join("summary.jsonl");
    let output = summarise(&index, &texts, &xml(dir.path()), &out);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines = lines(&out);
    assert_eq!(lines.len(), 2, "{lines:?}");
    let (_, h3) = row(&lines, 9220);
    assert_eq!(h3["format"], "xml");
    assert_eq!(h3["sha256"], XML_SHA256);
    assert_eq!(h3["counts"], json!({"MAY": 1, "SHOULD": 1}));
}

#[test]
fn summarise_leaves_the_text_rows_byte_identical_to_the_committed_summary() {
    let dir = scratch("text-rows");
    let lines = lines(&summary(dir.path()));
    let committed = committed();
    for rfc in [868, 1035, 5905, 6455] {
        assert_eq!(
            row_line(&lines, rfc),
            row_line(&committed, rfc),
            "RFC {rfc}"
        );
    }
}

/// Mutate a valid summary with `change`, run the check and return its stderr; it must exit 1.
fn refused(name: &str, change: impl FnOnce(&mut Vec<String>)) -> String {
    let dir = scratch(name);
    let summary = summary(dir.path());
    let mut lines = lines(&summary);
    change(&mut lines);
    write_lines(&summary, &lines);
    let output = check(&summary);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    stderr(&output)
}

fn set_header(lines: &mut [String], field: &str, value: Value) {
    let mut header: Value = serde_json::from_str(&lines[0]).unwrap();
    header[field] = value;
    lines[0] = serde_json::to_string(&header).unwrap();
}

fn set_row(lines: &mut [String], rfc: u64, change: impl FnOnce(&mut Value)) -> usize {
    let (at, mut value) = row(lines, rfc);
    change(&mut value);
    lines[at] = serde_json::to_string(&value).unwrap();
    at + 1
}

#[test]
fn check_refuses_another_format() {
    let error = refused("format", |lines| {
        set_header(lines, "format", json!("rfcspec-bulk/2"))
    });
    assert!(error.contains("line 1"), "{error}");
    assert!(error.contains("rfcspec-bulk/2"), "{error}");
}

#[test]
fn check_refuses_a_row_count_that_differs() {
    let error = refused("rows", |lines| set_header(lines, "rows", json!(6)));
    assert!(error.contains("line 1"), "{error}");
    assert!(error.contains("rows 6"), "{error}");
}

#[test]
fn check_refuses_an_edited_row_whose_rfc_no_profile_locks_by_rows_sha256() {
    let mut line = 0;
    let error = refused("rows-sha256", |lines| {
        let (after_6455, _) = row(lines, 6455);
        lines.insert(
            after_6455 + 1,
            json!({"rfc": 7000, "title": "Synthetic", "status": "INFORMATIONAL", "stream": "IETF",
                "date": "2013-01", "sha256": "0".repeat(64), "counts": {"MUST": 1}})
            .to_string(),
        );
        reheader(lines);
        line = set_row(lines, 7000, |row| row["counts"]["MUST"] = json!(2));
    });
    assert_eq!(line, 6);
    assert!(error.contains("line 1"), "{error}");
    assert!(error.contains("rows_sha256"), "{error}");
    assert_eq!(error.lines().count(), 1, "{error}");
}

#[test]
fn check_refuses_numbers_that_are_not_ascending() {
    let error = refused("order", |lines| {
        lines.swap(1, 2);
        reheader(lines);
    });
    assert!(error.contains("line 3"), "{error}");
    assert!(error.contains("ascending"), "{error}");
}

#[test]
fn check_names_a_duplicate_number_as_such() {
    let error = refused("duplicate", |lines| {
        let copy = lines[2].clone();
        lines.insert(3, copy);
        reheader(lines);
    });
    assert!(error.contains("line 4"), "{error}");
    assert!(error.contains("duplicate"), "{error}");
}

#[test]
fn check_refuses_a_missing_sha256() {
    let error = refused("sha-missing", |lines| {
        let line = set_row(lines, 868, |row| {
            row.as_object_mut().unwrap().remove("sha256");
        });
        assert_eq!(line, 2);
        reheader(lines);
    });
    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("sha256"), "{error}");
}

#[test]
fn check_refuses_a_sha256_that_is_not_64_lower_case_hex_digits() {
    let error = refused("sha-form", |lines| {
        set_row(lines, 868, |row| {
            row["sha256"] = json!(row["sha256"].as_str().unwrap().to_uppercase())
        });
        reheader(lines);
    });
    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("64 lower-case hex"), "{error}");
}

#[test]
fn check_refuses_a_counts_key_that_is_not_a_level() {
    let error = refused("counts-key", |lines| {
        set_row(lines, 868, |row| row["counts"] = json!({"MUSTN'T": 1}));
        reheader(lines);
    });
    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("MUSTN'T"), "{error}");
}

#[test]
fn check_refuses_a_count_that_is_not_a_positive_integer() {
    let error = refused("counts-value", |lines| {
        set_row(lines, 868, |row| row["counts"] = json!({"MUST": 0}));
        reheader(lines);
    });
    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("positive integer"), "{error}");
}

#[test]
fn check_refuses_another_extractor() {
    let error = refused("extractor", |lines| {
        set_header(lines, "extractor", json!(0))
    });
    assert!(error.contains("line 1"), "{error}");
    assert!(error.contains("extractor 0"), "{error}");
}

#[test]
fn check_refuses_a_locked_rfc_without_a_row() {
    let error = refused("locked-missing", |lines| {
        set_row(lines, 868, |row| row["rfc"] = json!(869));
        reheader(lines);
    });
    assert!(error.contains("RFC 868"), "{error}");
    assert!(error.contains("profiles/time/rfcspec.lock"), "{error}");
}

#[test]
fn check_refuses_a_locked_rfc_whose_sha256_differs_from_the_lock() {
    let error = refused("locked-sha", |lines| {
        set_row(lines, 1035, |row| row["sha256"] = json!("a".repeat(64)));
        reheader(lines);
    });
    assert!(error.contains("line 3"), "{error}");
    assert!(error.contains(&locked_sha256("profiles/dns")), "{error}");
}

#[test]
fn check_refuses_locked_counts_that_differ_from_extracting_the_locked_source() {
    let error = refused("locked-counts", |lines| {
        set_row(lines, 6455, |row| {
            let must = row["counts"]["MUST"].as_u64().unwrap();
            row["counts"]["MUST"] = json!(must + 1);
        });
        reheader(lines);
    });
    assert!(error.contains("line 5"), "{error}");
    assert!(error.contains("counts"), "{error}");
}

#[test]
fn check_refuses_a_row_format_other_than_xml_naming_the_line() {
    let error = refused("format-text", |lines| {
        set_row(lines, 868, |row| row["format"] = json!("text"));
        reheader(lines);
    });
    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("format"), "{error}");
    assert!(error.contains("\"text\""), "{error}");
}

/// Check the fixture summary changed by `change` against [`xml_profiles`]; returns the exit code
/// and stderr.
fn checked_with_xml_lock(name: &str, change: impl FnOnce(&mut Vec<String>)) -> (i32, String) {
    let dir = scratch(name);
    let summary = summary(dir.path());
    let mut lines = lines(&summary);
    change(&mut lines);
    write_lines(&summary, &lines);
    let output = check_against(&summary, &xml_profiles(dir.path()));
    (output.status.code().unwrap(), stderr(&output))
}

#[test]
fn check_accepts_an_xml_lock_whose_row_was_read_from_xml() {
    let (code, error) = checked_with_xml_lock("xml-lock", |_| {});
    assert_eq!(code, 0, "{error}");
}

#[test]
fn check_refuses_an_xml_lock_whose_row_was_read_from_text_naming_the_lock_and_both_formats() {
    let (code, error) = checked_with_xml_lock("xml-lock-text-row", |lines| {
        set_row(lines, 9220, |row| {
            row.as_object_mut().unwrap().remove("format");
        });
        reheader(lines);
    });
    assert_eq!(code, 1, "{error}");
    assert!(error.contains("line 6"), "{error}");
    assert!(error.contains("h3/rfcspec.lock"), "{error}");
    assert!(error.contains("sources/rfc9220.xml"), "{error}");
    assert!(error.contains("format text"), "{error}");
    assert!(error.contains("format xml"), "{error}");
}

#[test]
fn check_refuses_a_text_lock_whose_row_was_read_from_xml_naming_the_lock_and_both_formats() {
    let (code, error) = checked_with_xml_lock("text-lock-xml-row", |lines| {
        set_row(lines, 868, |row| row["format"] = json!("xml"));
        reheader(lines);
    });
    assert_eq!(code, 1, "{error}");
    assert!(error.contains("line 2"), "{error}");
    assert!(error.contains("time/rfcspec.lock"), "{error}");
    assert!(error.contains("sources/rfc868.txt"), "{error}");
    assert!(error.contains("format xml"), "{error}");
    assert!(error.contains("format text"), "{error}");
}

#[test]
fn check_refuses_xml_locked_counts_that_differ_from_reading_the_locked_xml() {
    let (code, error) = checked_with_xml_lock("xml-lock-counts", |lines| {
        set_row(lines, 9220, |row| row["counts"]["MAY"] = json!(2));
        reheader(lines);
    });
    assert_eq!(code, 1, "{error}");
    assert!(error.contains("line 6"), "{error}");
    assert!(error.contains(r#"{"MAY":1,"SHOULD":1}"#), "{error}");
}

#[test]
fn check_refuses_an_xml_locked_digest_that_differs_from_the_lock() {
    let (code, error) = checked_with_xml_lock("xml-lock-sha", |lines| {
        set_row(lines, 9220, |row| row["sha256"] = json!("a".repeat(64)));
        reheader(lines);
    });
    assert_eq!(code, 1, "{error}");
    assert!(error.contains("line 6"), "{error}");
    assert!(error.contains(XML_SHA256), "{error}");
}

/// The committed summary, relative to the repository.
const COMMITTED: &str = "bulk/rfc-summary.jsonl";

/// An index of three entries, two of them with a plain text.
const TWO_TEXTS: &str = r#"<?xml version='1.0' encoding='utf-8'?>
<rfc-index xmlns="https://www.rfc-editor.org/rfc-index">
  <rfc-entry>
    <doc-id>RFC8</doc-id>
    <title>ARPA Network Functional Specifications</title>
    <date><month>May</month><year>1969</year></date>
    <format><file-format>PDF</file-format></format>
    <current-status>UNKNOWN</current-status>
    <stream>Legacy</stream>
  </rfc-entry>
  <rfc-entry>
    <doc-id>RFC868</doc-id>
    <title>Time Protocol</title>
    <date><month>May</month><year>1983</year></date>
    <format><file-format>TXT</file-format></format>
    <current-status>INTERNET STANDARD</current-status>
    <stream>Legacy</stream>
  </rfc-entry>
  <rfc-entry>
    <doc-id>RFC1035</doc-id>
    <title>Domain names - implementation and specification</title>
    <date><month>November</month><year>1987</year></date>
    <format><file-format>TXT</file-format></format>
    <current-status>INTERNET STANDARD</current-status>
    <stream>Legacy</stream>
  </rfc-entry>
</rfc-index>"#;

/// An HTTP server on 127.0.0.1 answering `GET <path>` from `files` and 404 otherwise; it
/// returns its address and the paths requested so far.
fn serve(files: BTreeMap<String, Vec<u8>>) -> (String, Arc<Mutex<Vec<String>>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    let requested = Arc::new(Mutex::new(Vec::new()));
    let log = Arc::clone(&requested);
    thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let mut request = Vec::new();
            let mut buffer = [0u8; 1024];
            while !request.ends_with(b"\r\n\r\n") {
                let Ok(read) = stream.read(&mut buffer) else {
                    break;
                };
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
            }
            let request = String::from_utf8_lossy(&request).into_owned();
            let path = request
                .split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .to_string();
            log.lock().unwrap().push(path.clone());
            let (status, body) = match files.get(&path) {
                Some(body) => ("200 OK", body.clone()),
                None => ("404 Not Found", b"not found".to_vec()),
            };
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
    });
    (format!("http://{address}"), requested)
}

fn locked_text(profile: &str, rfc: u32) -> Vec<u8> {
    fs::read(repository().join(format!("profiles/{profile}/sources/rfc{rfc}.txt"))).unwrap()
}

/// The fixture server: the two-text index and the texts of RFC 868 and RFC 1035.
fn corpus_server() -> (String, Arc<Mutex<Vec<String>>>) {
    serve(BTreeMap::from([
        ("/rfc-index.xml".to_string(), TWO_TEXTS.as_bytes().to_vec()),
        ("/rfc/rfc868.txt".to_string(), locked_text("time", 868)),
        ("/rfc/rfc1035.txt".to_string(), locked_text("dns", 1035)),
    ]))
}

fn fetch(server: &str, cache: &Path, extra: &[&str]) -> Output {
    let index_url = format!("{server}/rfc-index.xml");
    let base_url = format!("{server}/rfc/");
    let mut args = vec![
        "bulk",
        "fetch",
        "--cache",
        cache.to_str().unwrap(),
        "--index-url",
        &index_url,
        "--base-url",
        &base_url,
    ];
    args.extend_from_slice(extra);
    rfcspec(&args)
}

/// A directory outside every Git working tree: under the system temporary directory, since the
/// repository's own `target/` is inside one.
fn outside_cache() -> TempDir {
    tempfile::Builder::new()
        .prefix("rfcspec-bulk-cache-")
        .tempdir()
        .unwrap()
}

#[test]
fn fetch_then_summarise_writes_two_rows_and_a_second_fetch_downloads_no_text() {
    let (server, requested) = corpus_server();
    let cache = outside_cache();
    let output = fetch(&server, cache.path(), &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        fs::read(cache.path().join("rfc-index.xml")).unwrap(),
        TWO_TEXTS.as_bytes()
    );
    assert_eq!(
        fs::read(cache.path().join("txt/rfc868.txt")).unwrap(),
        locked_text("time", 868)
    );
    assert_eq!(
        fs::read(cache.path().join("txt/rfc1035.txt")).unwrap(),
        locked_text("dns", 1035)
    );
    assert_eq!(
        *requested.lock().unwrap(),
        ["/rfc-index.xml", "/rfc/rfc868.txt", "/rfc/rfc1035.txt"]
    );

    let out = cache.path().join("summary.jsonl");
    let output = rfcspec(&[
        "bulk",
        "summarise",
        "--cache",
        cache.path().to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines = lines(&out);
    assert_eq!(lines.len(), 3, "{lines:?}");
    let header: Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(header["rows"], 2);
    assert_eq!(header["index_sha256"], sha256_hex(TWO_TEXTS.as_bytes()));
    assert_eq!(row(&lines, 868).1["sha256"], locked_sha256("profiles/time"));
    assert_eq!(row(&lines, 1035).1["sha256"], locked_sha256("profiles/dns"));

    requested.lock().unwrap().clear();
    let output = fetch(&server, cache.path(), &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(*requested.lock().unwrap(), ["/rfc-index.xml"]);

    requested.lock().unwrap().clear();
    let output = fetch(&server, cache.path(), &["--refresh"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        *requested.lock().unwrap(),
        ["/rfc-index.xml", "/rfc/rfc868.txt", "/rfc/rfc1035.txt"]
    );
}

#[test]
fn fetch_exits_1_naming_the_rfc_and_url_of_a_failed_download_and_a_rerun_resumes() {
    let (server, _) = serve(BTreeMap::from([
        ("/rfc-index.xml".to_string(), TWO_TEXTS.as_bytes().to_vec()),
        ("/rfc/rfc868.txt".to_string(), locked_text("time", 868)),
    ]));
    let cache = outside_cache();
    let output = fetch(&server, cache.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("RFC 1035"), "{error}");
    assert!(
        error.contains(&format!("{server}/rfc/rfc1035.txt")),
        "{error}"
    );
    assert!(cache.path().join("txt/rfc868.txt").is_file());
    assert!(!cache.path().join("txt/rfc1035.txt").exists());

    let (server, requested) = corpus_server();
    let output = fetch(&server, cache.path(), &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(
        *requested.lock().unwrap(),
        ["/rfc-index.xml", "/rfc/rfc1035.txt"]
    );
    assert_eq!(
        fs::read(cache.path().join("txt/rfc1035.txt")).unwrap(),
        locked_text("dns", 1035)
    );
}

/// The fixture index, the four locked texts and, when `with_xml`, RFC 9220's XML, served.
fn fixture_server(with_xml: bool) -> (String, Arc<Mutex<Vec<String>>>) {
    let mut files = BTreeMap::from([
        (
            "/rfc-index.xml".to_string(),
            fs::read(repository().join(INDEX)).unwrap(),
        ),
        ("/rfc/rfc868.txt".to_string(), locked_text("time", 868)),
        ("/rfc/rfc1035.txt".to_string(), locked_text("dns", 1035)),
        ("/rfc/rfc5905.txt".to_string(), locked_text("ntp", 5905)),
        (
            "/rfc/rfc6455.txt".to_string(),
            locked_text("websocket", 6455),
        ),
    ]);
    if with_xml {
        files.insert(
            "/rfc/rfc9220.xml".to_string(),
            fs::read(repository().join(XML)).unwrap(),
        );
    }
    serve(files)
}

const FIXTURE_REQUESTS: [&str; 6] = [
    "/rfc-index.xml",
    "/rfc/rfc868.txt",
    "/rfc/rfc1035.txt",
    "/rfc/rfc5905.txt",
    "/rfc/rfc6455.txt",
    "/rfc/rfc9220.xml",
];

#[test]
fn fetch_downloads_xml_where_the_index_lists_it_and_summarise_reads_it() {
    let (server, requested) = fixture_server(true);
    let cache = outside_cache();
    let output = fetch(&server, cache.path(), &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(*requested.lock().unwrap(), FIXTURE_REQUESTS);
    assert_eq!(
        fs::read(cache.path().join("xml/rfc9220.xml")).unwrap(),
        fs::read(repository().join(XML)).unwrap()
    );
    assert!(!cache.path().join("txt/rfc9220.txt").exists());
    for (profile, rfc) in [
        ("time", 868),
        ("dns", 1035),
        ("ntp", 5905),
        ("websocket", 6455),
    ] {
        assert_eq!(
            fs::read(cache.path().join(format!("txt/rfc{rfc}.txt"))).unwrap(),
            locked_text(profile, rfc)
        );
    }

    let out = cache.path().join("summary.jsonl");
    let output = rfcspec(&[
        "bulk",
        "summarise",
        "--cache",
        cache.path().to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let lines = lines(&out);
    assert_eq!(lines.len(), 6, "{lines:?}");
    let (_, h3) = row(&lines, 9220);
    assert_eq!(h3["format"], "xml");
    assert_eq!(h3["sha256"], XML_SHA256);
    assert_eq!(h3["counts"], json!({"MAY": 1, "SHOULD": 1}));

    requested.lock().unwrap().clear();
    let output = fetch(&server, cache.path(), &[]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(*requested.lock().unwrap(), ["/rfc-index.xml"]);

    requested.lock().unwrap().clear();
    let output = fetch(&server, cache.path(), &["--refresh"]);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    assert_eq!(*requested.lock().unwrap(), FIXTURE_REQUESTS);
}

#[test]
fn fetch_exits_1_naming_the_rfc_and_xml_url_of_a_failed_xml_download() {
    let (server, _) = fixture_server(false);
    let cache = outside_cache();
    let output = fetch(&server, cache.path(), &[]);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("RFC 9220"), "{error}");
    assert!(
        error.contains(&format!("{server}/rfc/rfc9220.xml")),
        "{error}"
    );
    assert!(!cache.path().join("xml/rfc9220.xml").exists());
    assert!(!cache.path().join("txt/rfc9220.txt").exists());
}

#[test]
fn fetch_refuses_a_cache_inside_a_git_working_tree_naming_the_tree() {
    let (server, requested) = corpus_server();
    let repository = outside_cache();
    let status = Command::new("git")
        .args(["init", "-q"])
        .current_dir(repository.path())
        .status()
        .unwrap();
    assert!(status.success());
    let cache = repository.path().join("nested/cache");
    let output = fetch(&server, &cache, &[]);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("Git working tree"), "{error}");
    let tree = repository.path().canonicalize().unwrap();
    assert!(error.contains(&tree.display().to_string()), "{error}");
    assert!(!cache.exists());
    assert!(requested.lock().unwrap().is_empty());
}

fn committed() -> Vec<String> {
    lines(&repository().join(COMMITTED))
}

#[test]
fn the_committed_summary_holds_and_stays_under_8_mib() {
    let size = fs::metadata(repository().join(COMMITTED)).unwrap().len();
    assert!(size < 8 * 1024 * 1024, "{COMMITTED} is {size} bytes");
    let output = check(&repository().join(COMMITTED));
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let lines = committed();
    let header: Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(header["rows"], lines.len() - 1);
    let (_, time) = row(&lines, 868);
    assert_eq!(time["sha256"], locked_sha256("profiles/time"));
    assert_eq!(time["counts"], json!({}));
    let (_, dns) = row(&lines, 1035);
    assert_eq!(dns["sha256"], locked_sha256("profiles/dns"));
    assert_eq!(dns["counts"], extracted_counts("profiles/dns", "1035"));
    let (_, ntp) = row(&lines, 5905);
    assert_eq!(
        ntp["sha256"],
        "8b7abd903c60202e2953ee012fefa6916666c3b0c71825c748cdd262024fb268"
    );
    assert_eq!(ntp["sha256"], locked_sha256("profiles/ntp"));
    assert_eq!(ntp["counts"], extracted_counts("profiles/ntp", "5905"));
    let (_, websocket) = row(&lines, 6455);
    assert_eq!(
        websocket["sha256"],
        "765775326aee0ecca9b04bde3fd1f52932d498e33e34e428bd61b8a24da0fa3b"
    );
    assert_eq!(
        websocket["counts"],
        extracted_counts("profiles/websocket", "6455")
    );
}

/// Check a copy of the committed summary changed by `change`, its header left as committed; it
/// must exit 1 naming `rows_sha256`.
fn committed_refused(name: &str, change: impl FnOnce(&mut Vec<String>)) {
    let dir = scratch(name);
    let copy = dir.path().join("rfc-summary.jsonl");
    let mut lines = committed();
    change(&mut lines);
    write_lines(&copy, &lines);
    let output = check(&copy);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let error = stderr(&output);
    assert!(error.contains("rows_sha256"), "{error}");
}

#[test]
fn check_refuses_the_committed_summary_without_one_row() {
    committed_refused("committed-removed", |lines| {
        let (at, _) = row(lines, 1000);
        lines.remove(at);
    });
}

#[test]
fn check_refuses_the_committed_summary_with_two_rows_swapped() {
    committed_refused("committed-swapped", |lines| lines.swap(10, 11));
}

#[test]
fn check_refuses_the_committed_summary_with_the_counts_of_an_unlocked_row_edited() {
    committed_refused("committed-counts", |lines| {
        let (_, unlocked) = row(lines, 2119);
        assert!(
            unlocked["counts"]
                .as_object()
                .is_some_and(|counts| !counts.is_empty()),
            "{unlocked}"
        );
        set_row(lines, 2119, |row| {
            let must = row["counts"]["MUST"].as_u64().unwrap_or(0);
            row["counts"]["MUST"] = json!(must + 1);
        });
    });
}
