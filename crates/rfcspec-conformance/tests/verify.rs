//! The probe against in-process TCP time servers on 127.0.0.1: one correct, four faulty. Every
//! server records the octets it receives; the probe must never write.

use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener},
    path::PathBuf,
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

use rfcspec_conformance::{
    Options, Row, Verdict, exit_code, local_1900, report, report_path, verify, write_report,
};
use rfcspec_model::{Overlay, RunReport, read_runs, read_yaml, sha256_hex};

#[derive(Clone, Copy)]
enum Server {
    Correct,
    FiveOctets,
    HourOff,
    KeepsOpen,
    NeverSends,
}

/// Start a server for one connection; the handle yields the octets it received.
fn start(server: Server) -> (String, JoinHandle<Vec<u8>>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let target = format!("tcp://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let now = local_1900();
        let sent: Vec<u8> = match server {
            Server::Correct | Server::KeepsOpen => now.to_be_bytes().to_vec(),
            Server::FiveOctets => [&now.to_be_bytes()[..], &[0]].concat(),
            Server::HourOff => now.wrapping_add(3600).to_be_bytes().to_vec(),
            Server::NeverSends => vec![],
        };
        stream.write_all(&sent).unwrap();
        if matches!(
            server,
            Server::Correct | Server::FiveOctets | Server::HourOff
        ) {
            stream.shutdown(Shutdown::Write).unwrap();
        }
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let mut received = Vec::new();
        let _ = stream.read_to_end(&mut received);
        received
    });
    (target, handle)
}

fn overlay() -> Overlay {
    read_yaml(&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/time/overlay.yaml"))
        .unwrap()
}

fn options() -> Options {
    Options {
        tolerance: Duration::from_secs(5),
        timeout: Duration::from_secs(1),
    }
}

/// Run the probe against a server; check the matrix shape and that the server received nothing.
fn run(server: Server) -> Vec<Row> {
    let overlay = overlay();
    let (target, handle) = start(server);
    let started = Instant::now();
    let rows = verify(&overlay, &target, options()).unwrap();
    assert!(started.elapsed() < Duration::from_secs(5), "the probe hung");
    assert_eq!(
        handle.join().unwrap(),
        Vec::<u8>::new(),
        "the probe wrote to the target"
    );
    assert_eq!(rows.len(), overlay.requirements.len());
    rows
}

fn verdict(rows: &[Row], requirement: &str) -> Verdict {
    rows.iter()
        .find(|row| row.requirement == requirement)
        .unwrap()
        .verdict
}

#[test]
fn a_correct_server_passes_every_observable_tcp_row() {
    let rows = run(Server::Correct);
    for requirement in ["R-TCP-1", "R-TCP-2", "R-TIME-1"] {
        assert_eq!(verdict(&rows, requirement), Verdict::Pass, "{rows:#?}");
    }
    assert_eq!(verdict(&rows, "R-TCP-3"), Verdict::Inconclusive);
    for requirement in ["R-UDP-1", "R-UDP-2"] {
        assert_eq!(verdict(&rows, requirement), Verdict::OutOfScope);
    }
    assert_eq!(exit_code(&rows), 0);
}

#[test]
fn a_server_sending_five_octets_fails_the_octet_row() {
    let rows = run(Server::FiveOctets);
    assert_eq!(verdict(&rows, "R-TCP-1"), Verdict::Fail, "{rows:#?}");
    assert_eq!(exit_code(&rows), 1);
}

#[test]
fn a_server_an_hour_off_fails_the_value_row() {
    let rows = run(Server::HourOff);
    assert_eq!(verdict(&rows, "R-TIME-1"), Verdict::Fail, "{rows:#?}");
    assert_eq!(verdict(&rows, "R-TCP-1"), Verdict::Pass);
    assert_eq!(verdict(&rows, "R-TCP-2"), Verdict::Pass);
    assert_eq!(exit_code(&rows), 1);
}

#[test]
fn a_server_keeping_the_connection_open_fails_the_close_row() {
    let rows = run(Server::KeepsOpen);
    assert_eq!(verdict(&rows, "R-TCP-2"), Verdict::Fail, "{rows:#?}");
    assert_eq!(verdict(&rows, "R-TCP-1"), Verdict::Pass);
    assert_eq!(exit_code(&rows), 1);
}

#[test]
fn a_server_that_never_sends_stops_the_probe_at_its_timeout_with_a_fail_row() {
    let started = Instant::now();
    let rows = run(Server::NeverSends);
    let elapsed = started.elapsed();
    assert!(elapsed >= options().timeout, "{elapsed:?}");
    assert_eq!(verdict(&rows, "R-TCP-1"), Verdict::Fail, "{rows:#?}");
    assert_eq!(exit_code(&rows), 1);
}

#[test]
fn the_matrix_has_one_row_per_overlay_requirement_in_order() {
    let overlay = overlay();
    let rows = run(Server::Correct);
    let ids: Vec<&str> = rows.iter().map(|row| row.requirement.as_str()).collect();
    let expected: Vec<&str> = overlay.requirements.iter().map(|r| r.id.as_str()).collect();
    assert_eq!(ids, expected);
}

#[test]
fn only_tcp_targets_are_verified() {
    let error = verify(&overlay(), "udp://127.0.0.1:37", options())
        .unwrap_err()
        .to_string();
    assert!(error.contains("only tcp://"), "{error}");
}

/// A fresh directory under the workspace's build directory, not the system temporary one.
fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/rfcspec-conformance")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn overlay_bytes() -> Vec<u8> {
    std::fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/time/overlay.yaml"),
    )
    .unwrap()
}

/// Run against a server, write the report into `runs/` of a scratch profile, read it back.
fn recorded(server: Server, name: &str) -> (String, RunReport) {
    let (target, _) = start(server);
    let rows = verify(&overlay(), &target, options()).unwrap();
    let profile = scratch(name);
    let path = report_path(&profile.join("runs"), &target);
    write_report(
        &path,
        &report(&target, 1_793_793_600, &overlay_bytes(), rows),
    )
    .unwrap();
    let mut runs = read_runs(&profile).unwrap();
    assert_eq!(runs.len(), 1);
    std::fs::remove_dir_all(profile).unwrap();
    let (file, report) = runs.remove(0);
    assert_eq!(report.target, target);
    (file, report)
}

#[test]
fn a_report_of_the_correct_server_is_an_overall_pass() {
    let (file, report) = recorded(Server::Correct, "report-correct");
    assert!(file.starts_with("tcp-127.0.0.1-"), "{file}");
    assert_eq!(report.verdict, Verdict::Pass);
    assert_eq!(report.time, "2026-11-04T12:00:00Z");
    assert_eq!(report.rfcspec, rfcspec_conformance::VERSION);
    assert_eq!(report.overlay_sha256, sha256_hex(&overlay_bytes()));
    assert_eq!(report.rows.len(), overlay().requirements.len());
}

#[test]
fn a_report_of_the_five_octet_server_is_an_overall_fail() {
    let (_, report) = recorded(Server::FiveOctets, "report-five");
    assert_eq!(report.verdict, Verdict::Fail);
}

#[test]
fn the_same_run_at_an_injected_time_writes_the_same_bytes() {
    let rows = run(Server::Correct);
    let target = "tcp://127.0.0.1:37";
    let first = report(target, 1_793_793_600, &overlay_bytes(), rows.clone())
        .to_json()
        .unwrap();
    let second = report(target, 1_793_793_600, &overlay_bytes(), rows)
        .to_json()
        .unwrap();
    assert_eq!(first, second);
    assert!(first.ends_with("}\n"));
    assert!(first.starts_with("{\n  \"format\": \"rfcspec-run/1\",\n  \"target\": \"tcp://127.0.0.1:37\",\n  \"time\": \"2026-11-04T12:00:00Z\",\n"), "{first}");
}

#[test]
fn a_report_path_is_a_json_file_or_a_directory_for_the_target_slug() {
    let target = "tcp://127.0.0.1:37";
    assert_eq!(
        report_path(&PathBuf::from("profiles/time/runs"), target),
        PathBuf::from("profiles/time/runs/tcp-127.0.0.1-37.json")
    );
    assert_eq!(
        report_path(&PathBuf::from("x/run.json"), target),
        PathBuf::from("x/run.json")
    );
}
