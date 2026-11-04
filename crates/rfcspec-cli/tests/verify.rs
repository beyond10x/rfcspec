//! `rfcspec verify` against in-process time servers: the matrix and the exit status.

use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener},
    path::PathBuf,
    process::{Command, Output},
    thread::{self, JoinHandle},
    time::Duration,
};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A server that sends `offset` seconds from the local clock and closes; yields what it received.
fn server(offset: u32) -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let target = format!("tcp://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let now = rfcspec_conformance::local_1900().wrapping_add(offset);
        stream.write_all(&now.to_be_bytes()).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut received = Vec::new();
        let _ = stream.read_to_end(&mut received);
        received
    });
    (target, handle)
}

fn verify(target: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rfcspec"))
        .args(["verify", "--profile", "profiles/time", "--target", target])
        .args(["--timeout", "2"])
        .current_dir(repository())
        .output()
        .unwrap()
}

#[test]
fn a_correct_server_exits_zero_with_one_row_per_requirement() {
    let (target, handle) = server(0);
    let output = verify(&target);
    assert_eq!(handle.join().unwrap(), Vec::<u8>::new());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    let rows: Vec<&str> = stdout.lines().skip(1).collect();
    assert_eq!(rows.len(), 6, "{stdout}");
    assert!(rows[0].starts_with("R-TCP-1      MUST    pass"), "{stdout}");
    assert!(
        stdout.contains("R-TCP-3      SHOULD  inconclusive"),
        "{stdout}"
    );
    assert!(
        stdout.contains("R-UDP-1      MUST    out of scope"),
        "{stdout}"
    );
}

#[test]
fn a_server_an_hour_off_exits_one() {
    let (target, handle) = server(3600);
    let output = verify(&target);
    assert_eq!(handle.join().unwrap(), Vec::<u8>::new());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(1), "{stdout}");
    assert!(stdout.contains("R-TIME-1     MUST    fail"), "{stdout}");
}
