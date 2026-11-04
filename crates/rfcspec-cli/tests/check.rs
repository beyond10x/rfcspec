//! `rfcspec check`: communication semantics, recorded runs and `--all`, on copies of
//! `profiles/time` in scratch directories under the workspace's `target/`.

use std::{
    fs,
    io::{Read, Write},
    net::{Shutdown, TcpListener},
    path::{Path, PathBuf},
    process::{Command, Output},
    thread::{self, JoinHandle},
    time::Duration,
};

use rfcspec_docs::catalog;
use rfcspec_model::{RunReport, RunRow, Verdict};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// A fresh directory under the workspace's build directory, not the system temporary one.
fn scratch(name: &str) -> PathBuf {
    let dir = repository()
        .join("target/rfcspec-cli")
        .join(format!("{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Copy every file of `profiles/time` to `to`.
fn copy_time(to: &Path) {
    let from = repository().join("profiles/time");
    for file in catalog::files(&from).unwrap() {
        fs::create_dir_all(to.join(&file).parent().unwrap()).unwrap();
        fs::copy(from.join(&file), to.join(&file)).unwrap();
    }
}

fn rfcspec(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rfcspec"))
        .args(args)
        .current_dir(repository())
        .output()
        .unwrap()
}

fn check(profile: &Path) -> (Option<i32>, String) {
    let output = rfcspec(&["check", "--profile", profile.to_str().unwrap()]);
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// A copy of `profiles/time` whose `profile.yaml` has `from` replaced by `to`.
fn edited(name: &str, from: &str, to: &str) -> (PathBuf, PathBuf) {
    let root = scratch(name);
    let profile = root.join("time");
    copy_time(&profile);
    let text = fs::read_to_string(profile.join("profile.yaml")).unwrap();
    assert!(text.contains(from), "{from}");
    fs::write(profile.join("profile.yaml"), text.replacen(from, to, 1)).unwrap();
    (root, profile)
}

#[test]
fn the_time_profile_passes() {
    let (code, stderr) = check(&repository().join("profiles/time"));
    assert_eq!(code, Some(0), "{stderr}");
}

#[test]
fn a_layer_named_twice_is_refused() {
    let (root, profile) = edited("twice", "  - layer: udp", "  - layer: tcp");
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("profile time: layer tcp is declared twice"),
        "{stderr}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn machines_without_semantics_are_refused() {
    let root = scratch("no-semantics");
    let profile = root.join("time");
    copy_time(&profile);
    let text = fs::read_to_string(profile.join("profile.yaml")).unwrap();
    let head = text.split("semantics:").next().unwrap();
    fs::write(profile.join("profile.yaml"), head).unwrap();
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(
            "profile time: the overlay has machines (tcp, udp) but profile.yaml declares no semantics"
        ),
        "{stderr}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_machine_transport_without_its_layer_is_refused() {
    let (root, profile) = edited("transport", "  - layer: udp", "  - layer: quic");
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(
            "profile time: machine udp runs over transport udp, but no layer udp is declared"
        ),
        "{stderr}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn a_layer_over_a_missing_profile_is_refused_and_over_a_sibling_passes() {
    let line = "  - layer: udp\n";
    let (root, profile) = edited(
        "over",
        line,
        "  - layer: udp\n    over: {profile: nope, endpoint: stream, name: datagrams}\n",
    );
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("profile time: layer udp is over profile nope, but"),
        "{stderr}"
    );
    let text = fs::read_to_string(profile.join("profile.yaml")).unwrap();
    fs::write(
        profile.join("profile.yaml"),
        text.replace("profile: nope", "profile: time"),
    )
    .unwrap();
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(0), "{stderr}");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn an_unknown_semantics_value_is_refused() {
    let (root, profile) = edited("unknown", "direction: simplex", "direction: sideways");
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(stderr.contains("profile.yaml"), "{stderr}");
    fs::remove_dir_all(root).unwrap();
}

fn report(overlay: &[u8]) -> RunReport {
    RunReport::new(
        "tcp://127.0.0.1:37",
        1_793_793_600,
        "0.0.0",
        overlay,
        vec![RunRow {
            requirement: "R-TCP-1".into(),
            level: "MUST".into(),
            verdict: Verdict::Pass,
            detail: "received 4 octets".into(),
        }],
    )
}

#[test]
fn a_stale_or_unreadable_report_fails_check_naming_the_file() {
    let root = scratch("stale");
    let profile = root.join("time");
    copy_time(&profile);
    fs::create_dir_all(profile.join("runs")).unwrap();
    let overlay = fs::read(profile.join("overlay.yaml")).unwrap();
    let current = profile.join("runs/tcp-127.0.0.1-37.json");
    fs::write(&current, report(&overlay).to_json().unwrap()).unwrap();
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(0), "{stderr}");

    let stale = profile.join("runs/tcp-127.0.0.1-38.json");
    fs::write(&stale, report(b"another overlay").to_json().unwrap()).unwrap();
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(&format!("{}: stale run", stale.display())),
        "{stderr}"
    );
    fs::remove_file(&stale).unwrap();

    let broken = profile.join("runs/broken.json");
    fs::write(&broken, "{\"format\": \"rfcspec-run/1\"}").unwrap();
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(&format!("{}: does not parse", broken.display())),
        "{stderr}"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn check_all_visits_each_profile_and_fails_when_one_fails() {
    let root = scratch("all");
    copy_time(&root.join("a"));
    copy_time(&root.join("b"));
    let output = rfcspec(&["check", "--all", root.to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(output.status.code(), Some(0), "{stdout}");
    for name in ["a", "b"] {
        assert!(
            stdout.contains(&format!("{}: overlay holds", root.join(name).display())),
            "{stdout}"
        );
    }

    fs::create_dir_all(root.join("b/runs")).unwrap();
    fs::write(
        root.join("b/runs/tcp-127.0.0.1-37.json"),
        report(b"stale").to_json().unwrap(),
    )
    .unwrap();
    let output = rfcspec(&["check", "--all", root.to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(1), "{stderr}");
    assert!(
        stdout.contains(&format!("{}: overlay holds", root.join("a").display())),
        "{stdout}"
    );
    assert!(
        stderr.contains(&format!(
            "1 of 2 profiles fail: {}",
            root.join("b").display()
        )),
        "{stderr}"
    );
    fs::remove_dir_all(root).unwrap();
}

/// A correct time server for one connection.
fn server() -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let target = format!("tcp://{}", listener.local_addr().unwrap());
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let now = rfcspec_conformance_local_1900();
        stream.write_all(&now.to_be_bytes()).unwrap();
        stream.shutdown(Shutdown::Write).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(10)))
            .unwrap();
        let _ = stream.read_to_end(&mut Vec::new());
    });
    (target, handle)
}

/// The local clock as RFC 868 time.
fn rfcspec_conformance_local_1900() -> u32 {
    let unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    ((unix + 2_208_988_800) & u64::from(u32::MAX)) as u32
}

#[test]
fn a_recorded_run_round_trips_through_check_and_the_catalogue() {
    let root = scratch("round-trip");
    let profile = root.join("time");
    copy_time(&profile);

    let (target, handle) = server();
    let quiet = rfcspec(&[
        "verify",
        "--profile",
        profile.to_str().unwrap(),
        "--target",
        &target,
        "--timeout",
        "2",
    ]);
    handle.join().unwrap();
    assert_eq!(quiet.status.code(), Some(0));
    assert!(
        !profile.join("runs").exists(),
        "verify without --report writes nothing"
    );

    let (target, handle) = server();
    let output = rfcspec(&[
        "verify",
        "--profile",
        profile.to_str().unwrap(),
        "--target",
        &target,
        "--timeout",
        "2",
        "--report",
        profile.join("runs").to_str().unwrap(),
    ]);
    handle.join().unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(output.status.code(), Some(0), "{stderr}");
    let rows = |output: &Output| -> Vec<String> {
        String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(|line| {
                line.split_whitespace()
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    };
    assert_eq!(
        rows(&output),
        rows(&quiet),
        "the table is printed as without --report"
    );
    assert!(
        stderr.contains("rfcspec verify: recorded pass as"),
        "{stderr}"
    );
    let slug = rfcspec_model::target_slug(&target);
    let written = profile.join(format!("runs/{slug}.json"));
    let report = RunReport::parse(&fs::read_to_string(&written).unwrap()).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
    assert_eq!(report.target, target);

    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(0), "{stderr}");

    let entry = catalog::load(&root, "time").unwrap();
    assert_eq!(entry.stage, Some(catalog::Stage::S7));
    let page = catalog::page(&entry);
    assert!(
        page.contains(&format!(
            "| pass | {target} | {} | {} | [{slug}.json]",
            report.date(),
            report.rfcspec
        )),
        "{page}"
    );
    let index = catalog::index(&[entry]);
    assert!(
        index.contains(&format!("{slug}.json), {target}, {} |", report.date())),
        "{index}"
    );

    let overlay = profile.join("overlay.yaml");
    let text = fs::read_to_string(&overlay).unwrap();
    fs::write(&overlay, text.replacen("kind: K1", "kind: K2", 1)).unwrap();
    let (code, stderr) = check(&profile);
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(&format!("{}: stale run", written.display())),
        "{stderr}"
    );
    fs::remove_dir_all(root).unwrap();
}
