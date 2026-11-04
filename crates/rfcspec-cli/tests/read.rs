//! `rfcspec fetch` and `rfcspec read` as a user runs them.

use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output},
    thread,
};

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

fn copy_profile(to: &Path) {
    let from = repository().join("profiles/time");
    for file in ["profile.yaml", "rfcspec.lock", "sources/rfc868.txt"] {
        fs::create_dir_all(to.join(file).parent().unwrap()).unwrap();
        fs::copy(from.join(file), to.join(file)).unwrap();
    }
}

#[test]
fn read_prints_the_normalized_document_of_the_committed_source() {
    let output = rfcspec(&["read", "--profile", "profiles/time", "--rfc", "868"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["rfc"], 868);
    assert_eq!(
        document["furniture"],
        serde_json::json!([56, 57, 60, 61, 114, 115])
    );
}

#[test]
fn read_refuses_a_source_that_differs_from_the_lock_naming_both_digests() {
    let profile = tempfile::tempdir().unwrap();
    copy_profile(profile.path());
    let source = profile.path().join("sources/rfc868.txt");
    let mut bytes = fs::read(&source).unwrap();
    bytes[200] = b'X';
    fs::write(&source, &bytes).unwrap();
    let locked = "42d96cd4e0743f947be4dfe69a507aa1ab23fa0a1846ae9be67164ebba6bc2f8";

    let output = rfcspec(&[
        "read",
        "--profile",
        profile.path().to_str().unwrap(),
        "--rfc",
        "868",
    ]);
    assert_eq!(output.status.code(), Some(1));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains(locked), "{stderr}");
    let actual = rfcspec_source::sha256_hex(&bytes);
    assert!(stderr.contains(&actual), "{stderr}");
}

#[test]
fn fetch_twice_from_a_local_server_changes_no_byte() {
    let body = fs::read(repository().join("profiles/time/sources/rfc868.txt")).unwrap();
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let base = format!("http://{}/", listener.local_addr().unwrap());
    let served = body.clone();
    thread::spawn(move || {
        for stream in listener.incoming().take(2) {
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
            let head = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                served.len()
            );
            stream.write_all(head.as_bytes()).unwrap();
            stream.write_all(&served).unwrap();
        }
    });
    let profile = tempfile::tempdir().unwrap();
    let dir = profile.path().to_str().unwrap();
    let args = [
        "fetch",
        "--rfc",
        "868",
        "--profile",
        dir,
        "--base-url",
        &base,
    ];

    assert!(rfcspec(&args).status.success());
    let lock = fs::read(profile.path().join("rfcspec.lock")).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&lock),
        fs::read_to_string(repository().join("profiles/time/rfcspec.lock")).unwrap()
    );
    let second = rfcspec(&args);
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("(unchanged)"));
    assert_eq!(fs::read(profile.path().join("rfcspec.lock")).unwrap(), lock);
    assert_eq!(
        fs::read(profile.path().join("sources/rfc868.txt")).unwrap(),
        body
    );
}
