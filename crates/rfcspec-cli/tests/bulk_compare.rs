//! `rfcspec bulk compare` (story:text-reader-structure) as a user runs it, over a small cache
//! laid out as `rfcspec bulk fetch` writes one: `txt/rfc<N>.txt` and `xml/rfc<N>.xml`.

use std::{
    fs,
    path::PathBuf,
    process::{Command, Output},
};

use tempfile::TempDir;

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

fn text(sentences: &[&str]) -> String {
    let mut text = String::from("1.  Introduction\n");
    for sentence in sentences {
        text.push_str(&format!("\n   {sentence}\n   and it is done.\n"));
    }
    text
}

fn xml(levels: &[&str]) -> String {
    let mut body = String::new();
    for level in levels {
        body.push_str(&format!(
            "<t>A client <bcp14>{level}</bcp14> send the request and it is done.</t>\n"
        ));
    }
    format!(
        "<rfc version=\"3\">\n<middle>\n<section pn=\"section-1\">\n<name>Introduction</name>\n{body}</section>\n</middle>\n</rfc>\n"
    )
}

/// A cache under the workspace's build directory: RFC 1 the same on both paths, RFC 2 with one
/// level differing, RFC 3 with one requirement more in XML, RFC 4 only as text.
fn cache() -> TempDir {
    let scratch = repository().join("target/rfcspec-cli");
    fs::create_dir_all(&scratch).unwrap();
    let cache = tempfile::Builder::new()
        .prefix("bulk-compare-")
        .tempdir_in(scratch)
        .unwrap();
    fs::create_dir_all(cache.path().join("txt")).unwrap();
    fs::create_dir_all(cache.path().join("xml")).unwrap();
    let write = |name: &str, content: String| fs::write(cache.path().join(name), content).unwrap();
    write("txt/rfc1.txt", text(&["A client MUST send the request"]));
    write("xml/rfc1.xml", xml(&["MUST"]));
    write("txt/rfc2.txt", text(&["A client SHOULD send the request"]));
    write("xml/rfc2.xml", xml(&["MUST"]));
    write("txt/rfc3.txt", text(&["A client MAY send the request"]));
    write("xml/rfc3.xml", xml(&["MAY", "MUST"]));
    write("txt/rfc4.txt", text(&["A client MUST send the request"]));
    cache
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

#[test]
fn every_rfc_held_in_both_formats_is_compared_and_each_differing_one_listed() {
    let cache = cache();
    let dir = cache.path().to_str().unwrap();
    let output = rfcspec(&["bulk", "compare", "--cache", dir]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        stdout(&output),
        "rfc2: 1 ids differ\nrfc3: 1 ids differ\ndiffering: 2 of 3\n"
    );
}

#[test]
fn more_differing_rfcs_than_allowed_exit_1() {
    let cache = cache();
    let dir = cache.path().to_str().unwrap();
    let within = rfcspec(&["bulk", "compare", "--cache", dir, "--max-differing", "2"]);
    assert!(within.status.success(), "{within:?}");
    let over = rfcspec(&["bulk", "compare", "--cache", dir, "--max-differing", "1"]);
    assert_eq!(over.status.code(), Some(1), "{over:?}");
    assert!(stdout(&over).ends_with("differing: 2 of 3\n"), "{over:?}");
}

#[test]
fn rfcs_restricts_the_comparison_and_an_rfc_not_held_in_both_formats_is_refused() {
    let cache = cache();
    let dir = cache.path().to_str().unwrap();
    let output = rfcspec(&["bulk", "compare", "--cache", dir, "--rfcs", "1,3"]);
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "rfc3: 1 ids differ\ndiffering: 1 of 2\n");
    let missing = rfcspec(&["bulk", "compare", "--cache", dir, "--rfcs", "1,4"]);
    assert_eq!(missing.status.code(), Some(1), "{missing:?}");
    let stderr = String::from_utf8_lossy(&missing.stderr);
    assert!(
        stderr.contains("RFC 4") && stderr.contains("rfc4.xml"),
        "{stderr}"
    );
}
