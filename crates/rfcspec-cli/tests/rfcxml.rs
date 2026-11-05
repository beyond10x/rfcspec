//! `rfcspec fetch --format xml` and `rfcspec read` over RFC 9220's RFCXML v3 source, as a user runs
//! them against a profile in a temporary directory.

use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output},
    thread,
};

const XML_SHA256: &str = "8a60d0b566bfb252ff18a3dc3df9fe94e90b201b807002a47d45386616b09aae";

/// The SHA-256 of `rfcspec read --profile profiles/websocket --rfc 6455` before the XML reader.
const WEBSOCKET_READ_SHA256: &str =
    "2b7033351dba1343ddc8092b8370d4d10cd91569f16d67c4d08a907aa792aa10";

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

fn fixture() -> Vec<u8> {
    fs::read(repository().join("crates/rfcspec-doc/tests/fixtures/rfc9220.xml")).unwrap()
}

/// An HTTP server on 127.0.0.1 that answers `requests` requests with `body`.
fn serve(body: Vec<u8>, requests: usize) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}/", listener.local_addr().unwrap());
    thread::spawn(move || {
        for stream in listener.incoming().take(requests) {
            let mut stream = stream.unwrap();
            let mut request = Vec::new();
            let mut buffer = [0u8; 512];
            while !request.ends_with(b"\r\n\r\n") {
                let read = stream.read(&mut buffer).unwrap();
                if read == 0 {
                    break;
                }
                request.extend_from_slice(&buffer[..read]);
            }
            let path = String::from_utf8_lossy(&request)
                .split_whitespace()
                .nth(1)
                .unwrap_or_default()
                .to_string();
            let (status, served) = if path.ends_with("/rfc9220.xml") {
                ("200 OK", body.clone())
            } else {
                ("404 Not Found", Vec::new())
            };
            let head = format!(
                "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                served.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(&served).unwrap();
        }
    });
    base
}

/// A profile naming RFC 9220 with its XML source fetched and locked through `rfcspec fetch`.
fn fetched_profile() -> tempfile::TempDir {
    let profile = tempfile::tempdir().unwrap();
    fs::write(
        profile.path().join("profile.yaml"),
        "name: h3-websockets\nrfcs:\n- number: 9220\n  title: Bootstrapping WebSockets with HTTP/3\n  sections:\n  - '3'\n",
    )
    .unwrap();
    let base = serve(fixture(), 1);
    let output = fetch(profile.path(), &base);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    profile
}

fn fetch(profile: &Path, base: &str) -> Output {
    rfcspec(&[
        "fetch",
        "--rfc",
        "9220",
        "--profile",
        profile.to_str().unwrap(),
        "--format",
        "xml",
        "--base-url",
        base,
    ])
}

fn read(profile: &Path) -> Output {
    rfcspec(&[
        "read",
        "--profile",
        profile.to_str().unwrap(),
        "--rfc",
        "9220",
    ])
}

#[test]
fn fetch_with_format_xml_locks_the_xml_source_and_a_second_run_changes_no_byte() {
    let profile = tempfile::tempdir().unwrap();
    let base = serve(fixture(), 2);

    let first = fetch(profile.path(), &base);
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    assert_eq!(
        fs::read(profile.path().join("sources/rfc9220.xml")).unwrap(),
        fixture()
    );
    let lock = fs::read_to_string(profile.path().join("rfcspec.lock")).unwrap();
    assert_eq!(
        lock,
        format!(
            "format: rfcspec-lock/1\nsources:\n- rfc: 9220\n  file: sources/rfc9220.xml\n  sha256: {XML_SHA256}\n"
        )
    );
    assert!(!profile.path().join("sources/rfc9220.txt").exists());

    let second = fetch(profile.path(), &base);
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("(unchanged)"));
    assert_eq!(
        fs::read_to_string(profile.path().join("rfcspec.lock")).unwrap(),
        lock
    );
    assert_eq!(
        fs::read(profile.path().join("sources/rfc9220.xml")).unwrap(),
        fixture()
    );
}

#[test]
fn read_prints_the_xml_document_with_anchors_bcp14_marks_and_xml_line_spans() {
    let profile = fetched_profile();
    let output = read(profile.path());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["format"], "xml");
    assert_eq!(document["rfc"], 9220);
    assert_eq!(document["sha256"], XML_SHA256);
    assert_eq!(document["furniture"], serde_json::json!([]));
    let blocks = document["blocks"].as_array().unwrap();
    let heading = blocks
        .iter()
        .find(|block| block["kind"] == "heading" && block["section"] == "3")
        .unwrap();
    assert_eq!(heading["anchor"], "websockets-upgrade-over-http3");
    assert_eq!(
        heading["lines"],
        serde_json::json!(["3.  WebSockets Upgrade over HTTP/3"])
    );
    let paragraph = blocks
        .iter()
        .find(|block| block["span"] == serde_json::json!({"start": 130, "end": 134}))
        .unwrap();
    assert_eq!(paragraph["section"], "3");
    let keywords: Vec<&str> = paragraph["bcp14"]
        .as_array()
        .unwrap()
        .iter()
        .map(|mark| mark["keyword"].as_str().unwrap())
        .collect();
    assert_eq!(keywords, ["SHOULD", "MAY"]);
    let marks: usize = blocks
        .iter()
        .filter_map(|block| block["bcp14"].as_array())
        .map(Vec::len)
        .sum();
    assert_eq!(marks, 13);
}

#[test]
fn read_refuses_a_changed_xml_byte_naming_both_digests() {
    let profile = fetched_profile();
    let source = profile.path().join("sources/rfc9220.xml");
    let mut bytes = fs::read(&source).unwrap();
    bytes[500] ^= 1;
    fs::write(&source, &bytes).unwrap();

    let output = read(profile.path());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(XML_SHA256), "{stderr}");
    assert!(
        stderr.contains(&rfcspec_source::sha256_hex(&bytes)),
        "{stderr}"
    );
}

#[test]
fn read_refuses_a_lock_entry_that_is_neither_txt_nor_xml_naming_the_file() {
    let profile = fetched_profile();
    fs::rename(
        profile.path().join("sources/rfc9220.xml"),
        profile.path().join("sources/rfc9220.html"),
    )
    .unwrap();
    let lock = profile.path().join("rfcspec.lock");
    let text = fs::read_to_string(&lock).unwrap();
    fs::write(&lock, text.replace("rfc9220.xml", "rfc9220.html")).unwrap();

    let output = read(profile.path());
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("sources/rfc9220.html"), "{stderr}");
}

#[test]
fn a_text_document_reads_byte_for_byte_as_before_and_has_no_format_key() {
    let output = rfcspec(&["read", "--profile", "profiles/websocket", "--rfc", "6455"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        rfcspec_source::sha256_hex(&output.stdout),
        WEBSOCKET_READ_SHA256
    );
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(document.get("format").is_none());
}

fn extract(profile: &Path) -> Output {
    rfcspec(&[
        "extract",
        "--profile",
        profile.to_str().unwrap(),
        "--rfc",
        "9220",
    ])
}

/// A profile naming RFC 9220 with `bytes` locked as its source in `format`.
fn locked_profile(format: rfcspec_model::SourceFormat, bytes: &[u8]) -> tempfile::TempDir {
    let profile = tempfile::tempdir().unwrap();
    fs::write(
        profile.path().join("profile.yaml"),
        "name: h3-websockets\nrfcs:\n- number: 9220\n  title: Bootstrapping WebSockets with HTTP/3\n  sections:\n  - '3'\n",
    )
    .unwrap();
    rfcspec_source::store_format(profile.path(), 9220, format, bytes).unwrap();
    profile
}

fn catalogue_of(output: &Output) -> serde_json::Value {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn collapsed(value: &serde_json::Value) -> String {
    value
        .as_str()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn extract_over_the_xml_gives_r3_1_should_and_r3_2_may_from_bcp14_as_the_text_gives_them() {
    let profile = fetched_profile();
    let output = extract(profile.path());
    let xml = catalogue_of(&output);
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text_profile = locked_profile(
        rfcspec_model::SourceFormat::Text,
        &fs::read(repository().join("crates/rfcspec-doc/tests/fixtures/rfc9220.txt")).unwrap(),
    );
    let text = catalogue_of(&extract(text_profile.path()));

    let rows = |catalogue: &serde_json::Value| -> Vec<(String, String, String, String)> {
        catalogue["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["id"].as_str().unwrap().to_string(),
                    r["keyword"].as_str().unwrap().to_string(),
                    r["section"].as_str().unwrap().to_string(),
                    collapsed(&r["text"]),
                )
            })
            .collect()
    };
    assert_eq!(
        rows(&xml),
        [
            (
                "R3-1".to_string(),
                "SHOULD".to_string(),
                "3".to_string(),
                "If a server advertises support for Extended CONNECT but receives an Extended CONNECT request with a \":protocol\" value that is unknown or is not supported, the server SHOULD respond to the request with a 501 (Not Implemented) status code (Section 15.6.2 of [HTTP]).".to_string()
            ),
            (
                "R3-2".to_string(),
                "MAY".to_string(),
                "3".to_string(),
                "A server MAY provide more information via a \"problem details\" response [RFC7807].".to_string()
            ),
        ]
    );
    assert_eq!(rows(&xml), rows(&text));
    let sources_and_spans = |catalogue: &serde_json::Value| -> Vec<(String, u64, u64)> {
        catalogue["requirements"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| {
                (
                    r["source"].as_str().unwrap().to_string(),
                    r["span"]["start"].as_u64().unwrap(),
                    r["span"]["end"].as_u64().unwrap(),
                )
            })
            .collect()
    };
    assert_eq!(
        sources_and_spans(&xml),
        [
            ("bcp14-xml".to_string(), 130, 133),
            ("bcp14-xml".to_string(), 133, 134)
        ]
    );
    assert_eq!(
        sources_and_spans(&text),
        [
            ("capitals-text".to_string(), 91, 94),
            ("capitals-text".to_string(), 94, 96)
        ]
    );
    let prose = |catalogue: &serde_json::Value| -> Vec<String> {
        catalogue["prose_candidates"]
            .as_array()
            .unwrap()
            .iter()
            .map(|c| collapsed(&c["text"]))
            .collect()
    };
    assert_eq!(prose(&xml), prose(&text));
    assert_eq!(prose(&xml).len(), 2, "{:?}", prose(&xml));
    assert!(prose(&xml)[0].contains("may be obtained at"));
    assert!(prose(&xml)[1].contains("must include"));
}

#[test]
fn extract_names_the_line_of_a_bcp14_element_holding_no_level_and_still_exits_0() {
    let source = String::from_utf8(fixture()).unwrap();
    let changed = source.replace(
        "<bcp14>MAY</bcp14>\nprovide",
        "<bcp14>must</bcp14>\nprovide",
    );
    assert_ne!(changed, source);
    let profile = locked_profile(rfcspec_model::SourceFormat::Xml, changed.as_bytes());

    let output = extract(profile.path());
    let catalogue = catalogue_of(&output);
    let ids: Vec<&str> = catalogue["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["R3-1"]);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(stderr.lines().count(), 1, "{stderr}");
    assert!(stderr.contains("line 133"), "{stderr}");
}

#[test]
fn bulk_check_refuses_an_xml_lock_naming_the_file_and_the_story_that_reads_it() {
    let profiles = tempfile::tempdir().unwrap();
    let profile = profiles.path().join("time");
    fs::create_dir_all(&profile).unwrap();
    fs::copy(
        repository().join("profiles/time/profile.yaml"),
        profile.join("profile.yaml"),
    )
    .unwrap();
    rfcspec_source::store_format(&profile, 868, rfcspec_model::SourceFormat::Xml, &fixture())
        .unwrap();

    let output = rfcspec(&[
        "bulk",
        "check",
        "--summary",
        "bulk/rfc-summary.jsonl",
        "--profiles",
        profiles.path().to_str().unwrap(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("sources/rfc868.xml"), "{stderr}");
    assert!(stderr.contains("story:bulk-xml-summary"), "{stderr}");
}
