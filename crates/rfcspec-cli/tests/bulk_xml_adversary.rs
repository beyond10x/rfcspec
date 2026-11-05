//! Adversarial cases for story:bulk-xml-summary: row `format` values, unreadable XML in
//! `bulk summarise`, and a body that is not RFCXML served for `rfc<N>.xml` in `bulk fetch`.

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output},
    thread,
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
        .prefix(&format!("bulk-adv-{name}-"))
        .tempdir_in(scratch)
        .unwrap()
}

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

fn set_row(lines: &mut [String], rfc: u64, change: impl FnOnce(&mut Value)) -> usize {
    let at = lines
        .iter()
        .enumerate()
        .skip(1)
        .find(|(_, line)| serde_json::from_str::<Value>(line).unwrap()["rfc"] == rfc)
        .unwrap()
        .0;
    let mut value: Value = serde_json::from_str(&lines[at]).unwrap();
    change(&mut value);
    lines[at] = serde_json::to_string(&value).unwrap();
    at + 1
}

/// The fixture summary changed by `change`, checked against the repository's profiles.
fn checked(name: &str, change: impl FnOnce(&mut Vec<String>)) -> (Option<i32>, String) {
    let dir = scratch(name);
    let out = dir.path().join("summary.jsonl");
    let output = summarise(
        &repository().join(INDEX),
        &texts(dir.path()),
        &xml(dir.path()),
        &out,
    );
    assert_eq!(output.status.code(), Some(0), "{}", stderr(&output));
    let mut lines = lines(&out);
    change(&mut lines);
    reheader(&mut lines);
    write_lines(&out, &lines);
    let output = rfcspec(&[
        "bulk",
        "check",
        "--summary",
        out.to_str().unwrap(),
        "--profiles",
        "profiles",
    ]);
    (output.status.code(), stderr(&output))
}

#[test]
fn adv_check_refuses_format_xml_in_upper_case_naming_the_line() {
    let (code, error) = checked("format-upper", |lines| {
        set_row(lines, 9220, |row| row["format"] = json!("XML"));
    });
    assert_eq!(code, Some(1), "{error}");
    assert!(error.contains("line 6"), "{error}");
    assert!(error.contains("\"XML\""), "{error}");
}

#[test]
fn adv_check_refuses_format_null_naming_the_line() {
    let (code, error) = checked("format-null", |lines| {
        set_row(lines, 9220, |row| row["format"] = Value::Null);
    });
    assert_eq!(code, Some(1), "{error}");
    assert!(error.contains("line 6"), "{error}");
    assert!(error.contains("format"), "{error}");
}

/// `bulk summarise` refuses a non-UTF-8 XML instead of reading it lossily, so a row read from
/// XML is never lossy; `bulk check` validates a summary against itself and should refuse one.
#[test]
fn adv_check_refuses_a_row_read_from_xml_marked_lossy() {
    let (code, error) = checked("xml-lossy", |lines| {
        set_row(lines, 9220, |row| row["lossy"] = json!(true));
    });
    assert_eq!(code, Some(1), "{error}");
    assert!(error.contains("line 6"), "{error}");
    assert!(error.contains("lossy"), "{error}");
}

#[test]
fn adv_summarise_exits_1_naming_the_rfc_and_path_of_an_xml_that_is_not_utf8() {
    let dir = scratch("xml-not-utf8");
    let xml = xml(dir.path());
    let path = xml.join("rfc9220.xml");
    let mut bytes = fs::read(&path).unwrap();
    bytes.extend_from_slice(b"<!-- \xff\xfe -->\n");
    fs::write(&path, &bytes).unwrap();
    let out = dir.path().join("summary.jsonl");
    let output = summarise(&repository().join(INDEX), &texts(dir.path()), &xml, &out);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let error = stderr(&output);
    assert!(error.contains("RFC 9220"), "{error}");
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert!(!out.exists());
}

#[test]
fn adv_summarise_exits_1_naming_the_rfc_and_path_of_an_html_page_saved_as_xml() {
    let dir = scratch("xml-html");
    let xml = xml(dir.path());
    let path = xml.join("rfc9220.xml");
    fs::write(&path, "<html><body>Service unavailable</body></html>\n").unwrap();
    let out = dir.path().join("summary.jsonl");
    let output = summarise(&repository().join(INDEX), &texts(dir.path()), &xml, &out);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    let error = stderr(&output);
    assert!(error.contains("RFC 9220"), "{error}");
    assert!(error.contains(&path.display().to_string()), "{error}");
    assert!(!out.exists());
}

fn serve(files: BTreeMap<String, Vec<u8>>) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
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
            let path = request.split_whitespace().nth(1).unwrap_or_default();
            let (status, body) = match files.get(path) {
                Some(body) => ("200 OK", body.clone()),
                None => ("404 Not Found", b"not found".to_vec()),
            };
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                body.len()
            );
            let _ = stream.write_all(head.as_bytes());
            let _ = stream.write_all(&body);
        }
    });
    format!("http://{address}")
}

/// A 200 answer whose body is an HTML page, cached as `xml/rfc9220.xml`, is skipped by every
/// later fetch without `--refresh` and blocks every later summary; only a refresh of the whole
/// corpus repairs it. The fetch should refuse a body that is not RFCXML before caching it.
#[test]
fn adv_fetch_refuses_an_html_page_served_for_an_xml_and_caches_nothing() {
    let profile_text = |profile: &str, rfc: u32| {
        fs::read(repository().join(format!("profiles/{profile}/sources/rfc{rfc}.txt"))).unwrap()
    };
    let server = serve(BTreeMap::from([
        (
            "/rfc-index.xml".to_string(),
            fs::read(repository().join(INDEX)).unwrap(),
        ),
        ("/rfc/rfc868.txt".to_string(), profile_text("time", 868)),
        ("/rfc/rfc1035.txt".to_string(), profile_text("dns", 1035)),
        ("/rfc/rfc5905.txt".to_string(), profile_text("ntp", 5905)),
        (
            "/rfc/rfc6455.txt".to_string(),
            profile_text("websocket", 6455),
        ),
        (
            "/rfc/rfc9220.xml".to_string(),
            b"<!DOCTYPE html><html><body>Please log in</body></html>\n".to_vec(),
        ),
    ]));
    let cache = tempfile::Builder::new()
        .prefix("rfcspec-bulk-adv-cache-")
        .tempdir()
        .unwrap();
    let index_url = format!("{server}/rfc-index.xml");
    let base_url = format!("{server}/rfc/");
    let output = rfcspec(&[
        "bulk",
        "fetch",
        "--cache",
        cache.path().to_str().unwrap(),
        "--index-url",
        &index_url,
        "--base-url",
        &base_url,
    ]);
    assert_eq!(output.status.code(), Some(1), "{}", stderr(&output));
    assert!(stderr(&output).contains("RFC 9220"), "{}", stderr(&output));
    assert!(!cache.path().join("xml/rfc9220.xml").exists());
}
