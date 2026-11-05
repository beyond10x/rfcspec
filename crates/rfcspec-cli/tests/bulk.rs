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

/// The three texts the profiles lock, copied into `<dir>/texts` as `rfc<N>.txt`.
fn texts(dir: &Path) -> PathBuf {
    let texts = dir.join("texts");
    fs::create_dir_all(&texts).unwrap();
    for (profile, rfc) in [("time", 868), ("dns", 1035), ("websocket", 6455)] {
        fs::copy(
            repository().join(format!("profiles/{profile}/sources/rfc{rfc}.txt")),
            texts.join(format!("rfc{rfc}.txt")),
        )
        .unwrap();
    }
    texts
}

fn summarise(index: &Path, texts: &Path, out: &Path) -> Output {
    rfcspec(&[
        "bulk",
        "summarise",
        "--index",
        index.to_str().unwrap(),
        "--texts",
        texts.to_str().unwrap(),
        "--out",
        out.to_str().unwrap(),
    ])
}

fn check(summary: &Path) -> Output {
    rfcspec(&[
        "bulk",
        "check",
        "--summary",
        summary.to_str().unwrap(),
        "--profiles",
        "profiles",
    ])
}

/// The summary of the fixture index and texts, written into `dir`.
fn summary(dir: &Path) -> PathBuf {
    let out = dir.join("summary.jsonl");
    let output = summarise(&repository().join(INDEX), &texts(dir), &out);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    out
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

#[test]
fn summarise_writes_a_summary_check_accepts_with_the_locked_digests_and_extracted_counts() {
    let dir = scratch("summary");
    let summary = summary(dir.path());
    let output = check(&summary);
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));

    let lines = lines(&summary);
    let header: Value = serde_json::from_str(&lines[0]).unwrap();
    assert_eq!(header["format"], "rfcspec-bulk/1");
    assert_eq!(header["extractor"], 1);
    assert_eq!(header["rows"], 3);
    assert_eq!(
        header["index_sha256"],
        sha256_hex(&fs::read(repository().join(INDEX)).unwrap())
    );
    let rows: Vec<Value> = lines[1..]
        .iter()
        .map(|line| serde_json::from_str(line).unwrap())
        .collect();
    let numbers: Vec<&Value> = rows.iter().map(|row| &row["rfc"]).collect();
    assert_eq!(numbers, [&json!(868), &json!(1035), &json!(6455)]);

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

    let websocket = &rows[2];
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
    let output = summarise(&repository().join(INDEX), &texts, &out);
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
    let output = summarise(&repository().join(INDEX), &texts, &out);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr(&output);
    assert!(error.contains("RFC 1035"), "{error}");
    assert!(
        error.contains(&texts.join("rfc1035.txt").display().to_string()),
        "{error}"
    );
    assert!(!out.exists());
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
    let error = refused("rows", |lines| set_header(lines, "rows", json!(4)));
    assert!(error.contains("line 1"), "{error}");
    assert!(error.contains("rows 4"), "{error}");
}

#[test]
fn check_refuses_an_edited_row_whose_rfc_no_profile_locks_by_rows_sha256() {
    let mut line = 0;
    let error = refused("rows-sha256", |lines| {
        lines.push(
            json!({"rfc": 7000, "title": "Synthetic", "status": "INFORMATIONAL", "stream": "IETF",
                "date": "2013-01", "sha256": "0".repeat(64), "counts": {"MUST": 1}})
            .to_string(),
        );
        reheader(lines);
        line = set_row(lines, 7000, |row| row["counts"]["MUST"] = json!(2));
    });
    assert_eq!(line, 5);
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
    assert!(error.contains("line 4"), "{error}");
    assert!(error.contains("counts"), "{error}");
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
