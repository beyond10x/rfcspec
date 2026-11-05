//! The `websocket` profile (RFC 6455) as a user runs `rfcspec` on it: fetch, extract and check.

use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::{Path, PathBuf},
    process::{Command, Output},
    thread,
};

use rfcspec_docs::catalog;

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const PROFILE: &str = "profiles/websocket";

fn rfcspec(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_rfcspec"))
        .args(args)
        .current_dir(repository())
        .output()
        .unwrap()
}

/// A fresh directory under the workspace's build directory, not the system temporary one.
fn scratch(name: &str) -> PathBuf {
    let dir = repository()
        .join("target/rfcspec-cli")
        .join(format!("websocket-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

/// Copy every file of `profiles/websocket` to `to`.
fn copy_websocket(to: &Path) {
    let from = repository().join(PROFILE);
    for file in catalog::files(&from).unwrap() {
        fs::create_dir_all(to.join(&file).parent().unwrap()).unwrap();
        fs::copy(from.join(&file), to.join(&file)).unwrap();
    }
}

fn catalogue() -> serde_json::Value {
    let output = rfcspec(&["extract", "--profile", PROFILE, "--rfc", "6455"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

/// The BCP 14 keywords of a text as disjoint whole words, longest first at each position.
fn keywords(text: &str) -> Vec<&'static str> {
    const BCP14: &[&str] = &[
        "NOT RECOMMENDED",
        "SHALL NOT",
        "SHOULD NOT",
        "MUST NOT",
        "RECOMMENDED",
        "REQUIRED",
        "OPTIONAL",
        "SHOULD",
        "SHALL",
        "MUST",
        "MAY",
    ];
    let words: Vec<&str> = text
        .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
        .collect();
    let mut found = Vec::new();
    let mut index = 0;
    while index < words.len() {
        let two = words
            .get(index + 1)
            .map(|next| format!("{} {next}", words[index]));
        if let Some(keyword) = two
            .as_deref()
            .and_then(|two| BCP14.iter().find(|k| **k == two))
        {
            found.push(*keyword);
            index += 2;
        } else if let Some(keyword) = BCP14.iter().find(|k| **k == words[index]) {
            found.push(*keyword);
            index += 1;
        } else {
            index += 1;
        }
    }
    found
}

#[test]
fn extract_lists_requirements_carrying_the_214_keywords_each_with_its_section() {
    let catalogue = catalogue();
    let requirements = catalogue["requirements"].as_array().unwrap();
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for requirement in requirements {
        let section = requirement["section"].as_str().unwrap();
        let id = requirement["id"].as_str().unwrap();
        assert!(id.starts_with(&format!("R{section}-")), "{id}");
        for keyword in keywords(requirement["text"].as_str().unwrap()) {
            *counts.entry(keyword).or_default() += 1;
        }
    }
    assert_eq!(
        counts,
        BTreeMap::from([
            ("MAY", 47),
            ("MUST", 113),
            ("MUST NOT", 25),
            ("OPTIONAL", 1),
            ("SHOULD", 24),
            ("SHOULD NOT", 4),
        ])
    );
    assert_eq!(counts.values().sum::<usize>(), 214);
    let abnf: Vec<u64> = requirements
        .iter()
        .filter(|r| r["source"] == "abnf-comment")
        .map(|r| r["span"]["start"].as_u64().unwrap())
        .collect();
    assert_eq!(abnf, [1408, 1706, 1710, 1714, 2677]);
}

#[test]
fn extract_output_is_byte_identical_across_two_runs() {
    let args = ["extract", "--profile", PROFILE, "--rfc", "6455"];
    let first = rfcspec(&args);
    assert!(first.status.success());
    assert_eq!(first.stdout, rfcspec(&args).stdout);
}

#[test]
fn task_check_explores_every_websocket_model_with_its_witness() {
    let text = fs::read_to_string(repository().join("Taskfile.yml")).unwrap();
    let taskfile: serde_json::Value = rfcspec_model::from_yaml(&text).unwrap();
    assert!(
        taskfile["tasks"]["check"]["cmds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cmd| cmd["task"] == "websocket"),
        "{text}"
    );
    let websocket: Vec<&str> = taskfile["tasks"]["websocket"]["cmds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|cmd| cmd.as_str().or_else(|| cmd["cmd"].as_str()))
        .collect();
    let websocket = websocket.join("\n");
    for model in ["close", "ping", "fragments"] {
        let expected = format!(
            "ess protocol explore --path {PROFILE}/generated/{model}.protospec.yaml --inputs {PROFILE}/witnesses/{model}.inputs.json | grep -q '\"verdict\": \"passed\"'"
        );
        assert!(
            websocket.contains(&expected),
            "{expected} not in\n{websocket}"
        );
    }
}

#[test]
fn check_all_profiles_checks_dns_time_and_websocket() {
    let output = rfcspec(&["check", "--all", "profiles"]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for name in ["dns", "time", "websocket"] {
        assert!(
            stdout.contains(&format!("profiles/{name}: overlay holds")),
            "{stdout}"
        );
    }
    assert_eq!(stdout.lines().count(), 3, "{stdout}");
}

#[test]
fn the_profile_and_the_overlay_name_rfc_6455() {
    let profile = fs::read_to_string(repository().join(PROFILE).join("profile.yaml")).unwrap();
    assert!(profile.starts_with("name: websocket\n"), "{profile}");
    assert!(
        profile.contains("title: The WebSocket Protocol"),
        "{profile}"
    );
    assert!(
        profile.contains("sections: [\"5.2\", \"5.4\", \"5.5\", \"5.6\", \"7\"]"),
        "{profile}"
    );
    let overlay = fs::read_to_string(repository().join(PROFILE).join("overlay.yaml")).unwrap();
    assert!(
        overlay.starts_with("format: rfcspec-overlay/1\nrfc: 6455\nkind: K1\n"),
        "{overlay}"
    );
}

#[test]
fn fetch_of_rfc_6455_twice_from_a_local_server_writes_the_committed_lock_and_changes_no_byte() {
    let body = fs::read(repository().join(PROFILE).join("sources/rfc6455.txt")).unwrap();
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
    let profile = scratch("fetch");
    let dir = profile.to_str().unwrap();
    let args = [
        "fetch",
        "--rfc",
        "6455",
        "--profile",
        dir,
        "--base-url",
        &base,
    ];
    assert!(rfcspec(&args).status.success());
    let lock = fs::read(profile.join("rfcspec.lock")).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&lock),
        fs::read_to_string(repository().join(PROFILE).join("rfcspec.lock")).unwrap()
    );
    assert!(String::from_utf8_lossy(&lock).contains(
        "- rfc: 6455\n  file: sources/rfc6455.txt\n  sha256: 765775326aee0ecca9b04bde3fd1f52932d498e33e34e428bd61b8a24da0fa3b\n"
    ));
    let second = rfcspec(&args);
    assert!(second.status.success());
    assert!(String::from_utf8_lossy(&second.stdout).contains("(unchanged)"));
    assert_eq!(fs::read(profile.join("rfcspec.lock")).unwrap(), lock);
    assert_eq!(fs::read(profile.join("sources/rfc6455.txt")).unwrap(), body);
    fs::remove_dir_all(profile).unwrap();
}

/// A copy of `profiles/websocket` whose overlay has `from` replaced by `to`, and what `check`
/// says about it.
fn check_edited(name: &str, from: &str, to: &str) -> (Option<i32>, String) {
    let root = scratch(name);
    let profile = root.join("websocket");
    copy_websocket(&profile);
    let text = fs::read_to_string(profile.join("overlay.yaml")).unwrap();
    assert!(text.contains(from), "{from}");
    fs::write(profile.join("overlay.yaml"), text.replacen(from, to, 1)).unwrap();
    let output = rfcspec(&["check", "--profile", profile.to_str().unwrap()]);
    fs::remove_dir_all(root).unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn check_refuses_a_capitals_text_requirement_whose_id_is_no_catalogue_requirement() {
    let (code, stderr) = check_edited("unknown-id", "  - id: R7.1.2-1\n", "  - id: R7.1.2-9\n");
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(
            "requirement R7.1.2-9: source capitals-text, but R7.1.2-9 is no catalogue requirement"
        ),
        "{stderr}"
    );
}

#[test]
fn check_refuses_a_requirement_whose_span_leaves_its_catalogue_span() {
    let (code, stderr) = check_edited(
        "wide-span",
        "    span: {start: 2005, end: 2006}\n",
        "    span: {start: 2005, end: 2008}\n",
    );
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(
            "requirement R5.5.1-6: lines 2005-2008 does not lie within the catalogue requirement's lines 2005-2006"
        ),
        "{stderr}"
    );
}

#[test]
fn the_level_stays_the_reviewers() {
    let (code, stderr) = check_edited(
        "level",
        "    level: SHOULD\n    quote: \"Once an endpoint",
        "    level: MUST\n    quote: \"Once an endpoint",
    );
    assert_eq!(code, Some(0), "{stderr}");
}

#[test]
fn lower_writes_the_close_model_and_its_check_fails_without_it() {
    let output = rfcspec(&["lower", "--profile", PROFILE, "--check"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let root = scratch("lower");
    let profile = root.join("websocket");
    copy_websocket(&profile);
    fs::remove_file(profile.join("generated/close.protospec.yaml")).unwrap();
    let output = rfcspec(&["lower", "--profile", profile.to_str().unwrap(), "--check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("close.protospec.yaml is missing"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let output = rfcspec(&["lower", "--profile", profile.to_str().unwrap()]);
    assert!(output.status.success());
    assert_eq!(
        fs::read(profile.join("generated/close.protospec.yaml")).unwrap(),
        fs::read(
            repository()
                .join(PROFILE)
                .join("generated/close.protospec.yaml")
        )
        .unwrap()
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_profile_declares_one_tcp_layer_without_over() {
    let profile: rfcspec_model::Profile =
        rfcspec_model::read_yaml(&repository().join(PROFILE).join("profile.yaml")).unwrap();
    assert_eq!(profile.semantics.len(), 1);
    let layer = &profile.semantics[0];
    assert_eq!(layer.layer, "tcp");
    assert!(layer.over.is_none());
    assert_eq!(
        rfcspec_model::to_yaml(layer).unwrap(),
        "layer: tcp\nunit: bytes\ndelivery:\n  ordered: true\n  reliable: true\n  duplicates: false\ndirection: full-duplex\nmultiplexing: single\nexchange: session\ncorrelation: none\nlifetime: connection\nroles:\n- client\n- server\n"
    );
}

#[test]
fn the_overlay_accepts_the_closing_handshake_requirements_and_records_g9() {
    let overlay: rfcspec_model::Overlay =
        rfcspec_model::read_yaml(&repository().join(PROFILE).join("overlay.yaml")).unwrap();
    let status = |id: &str| {
        let requirement = overlay.requirements.iter().find(|r| r.id == id).unwrap();
        assert_eq!(requirement.source, "capitals-text", "{id}");
        requirement.status.clone()
    };
    use rfcspec_model::Status;
    assert_eq!(
        status("R7.1.2-1"),
        Status::Satisfied("close.client.R7.1.2-1-start-closing-handshake".into())
    );
    let echo = "close.server.R5.5.1-6-R5.5.1-7-R5.5.1-9-R5.5.1-10-echo-close";
    assert_eq!(status("R5.5.1-6"), Status::Satisfied(echo.into()));
    assert_eq!(status("R5.5.1-10"), Status::Satisfied(echo.into()));
    let Status::Unsupported(reason) = status("R7.1.7-1") else {
        panic!("R7.1.7-1 is not unsupported");
    };
    assert!(reason.starts_with("ess-gap G9"), "{reason}");
}

#[test]
fn check_refuses_a_presence_naming_no_earlier_field() {
    let (code, stderr) = check_edited(
        "presence",
        "presence: {field: mask, equals: 1}",
        "presence: {field: opcode-extension, equals: 1}",
    );
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(
            "layout frame-header field masking-key: present when opcode-extension, which is no earlier field of the layout"
        ),
        "{stderr}"
    );
    let (code, stderr) = check_edited(
        "presence-later",
        "presence: {field: payload-len, equals: 126}",
        "presence: {field: masking-key, equals: 126}",
    );
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(
            "field extended-payload-length-16: present when masking-key, which is no earlier field"
        ),
        "{stderr}"
    );
}

#[test]
fn check_refuses_a_range_that_does_not_fit_the_field_width() {
    let (code, stderr) = check_edited("range", "{range: [126, 65535]}", "{range: [126, 65536]}");
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains(
            "layout frame-header field extended-payload-length-16: range 126 to 65536 does not fit 16 bits"
        ),
        "{stderr}"
    );
}

#[test]
fn generate_check_passes_for_both_profiles_and_fails_on_drift() {
    for args in [
        vec!["generate", "--check"],
        vec!["generate", "--profile", "profiles/time", "--check"],
        vec!["generate", "--profile", PROFILE, "--check"],
    ] {
        let output = rfcspec(&args);
        assert_eq!(
            output.status.code(),
            Some(0),
            "{args:?}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    let root = scratch("generate");
    let profile = root.join("websocket");
    copy_websocket(&profile);
    let lib = profile.join("generated/rust/src/lib.rs");
    fs::write(&lib, "// edited\n").unwrap();
    let output = rfcspec(&[
        "generate",
        "--profile",
        profile.to_str().unwrap(),
        "--check",
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("src/lib.rs is missing or differs"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let time = root.join("time");
    let from = repository().join("profiles/time");
    for file in catalog::files(&from).unwrap() {
        fs::create_dir_all(time.join(&file).parent().unwrap()).unwrap();
        fs::copy(from.join(&file), time.join(&file)).unwrap();
    }
    fs::write(time.join("generated/rust/Cargo.toml"), "[package]\n").unwrap();
    let output = rfcspec(&["generate", "--profile", time.to_str().unwrap(), "--check"]);
    assert_eq!(output.status.code(), Some(1));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn the_generated_crate_is_a_workspace_member() {
    let manifest = fs::read_to_string(repository().join("Cargo.toml")).unwrap();
    assert!(
        manifest.contains("\"profiles/websocket/generated/rust\""),
        "{manifest}"
    );
    let generated =
        fs::read_to_string(repository().join(PROFILE).join("generated/rust/Cargo.toml")).unwrap();
    assert!(
        generated.contains("name = \"rfc6455-websocket\""),
        "{generated}"
    );
}

/// The overlay text without the requirement entry `id`.
fn without(text: &str, id: &str) -> String {
    let start = text.find(&format!("  - id: {id}\n")).unwrap();
    let rest = &text[start + 1..];
    let end = start
        + 1
        + rest
            .find("\n  - id: ")
            .unwrap_or_else(|| rest.find("\nlayouts:").unwrap());
    format!("{}{}", &text[..start], &text[end + 1..])
}

fn check_without(name: &str, id: &str) -> (Option<i32>, String) {
    let root = scratch(name);
    let profile = root.join("websocket");
    copy_websocket(&profile);
    let text = fs::read_to_string(profile.join("overlay.yaml")).unwrap();
    let removed = without(&text, id);
    assert_eq!(text.lines().count() - removed.lines().count(), 6, "{id}");
    fs::write(profile.join("overlay.yaml"), removed).unwrap();
    let output = rfcspec(&["check", "--profile", profile.to_str().unwrap()]);
    fs::remove_dir_all(root).unwrap();
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn check_refuses_an_in_scope_requirement_without_a_status_at_any_level() {
    let (code, stderr) = check_without("scope-must", "R5.2-3");
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("profile websocket: requirement R5.2-3 (MUST, section 5.2) is in scope but has no status in overlay.yaml"),
        "{stderr}"
    );
    let (code, stderr) = check_without("scope-may", "R5.5.2-1");
    assert_eq!(code, Some(1), "{stderr}");
    assert!(
        stderr.contains("profile websocket: requirement R5.5.2-1 (MAY, section 5.5.2) is in scope but has no status"),
        "{stderr}"
    );
}

#[test]
fn every_in_scope_requirement_has_a_status_and_those_outside_need_none() {
    let catalogue = catalogue();
    let overlay: rfcspec_model::Overlay =
        rfcspec_model::read_yaml(&repository().join(PROFILE).join("overlay.yaml")).unwrap();
    let reviewed: Vec<&str> = overlay.requirements.iter().map(|r| r.id.as_str()).collect();
    let scoped = |section: &str| {
        ["5.2", "5.4", "5.5", "5.6", "7"]
            .iter()
            .any(|s| section == *s || section.starts_with(&format!("{s}.")))
    };
    let mut in_scope = 0;
    let mut outside = 0;
    let mut keywords_in = 0;
    let mut musts_in = 0;
    for requirement in catalogue["requirements"].as_array().unwrap() {
        let id = requirement["id"].as_str().unwrap();
        if scoped(requirement["section"].as_str().unwrap()) {
            in_scope += 1;
            assert!(reviewed.contains(&id), "{id} has no status");
            let found = keywords(requirement["text"].as_str().unwrap());
            keywords_in += found.len();
            musts_in += found.iter().filter(|k| k.starts_with("MUST")).count();
        } else {
            outside += 1;
            assert!(!reviewed.contains(&id), "{id} is outside the scope");
        }
    }
    assert_eq!(in_scope, overlay.requirements.len());
    assert_eq!((keywords_in, musts_in), (78, 40));
    assert!(outside > 0);
    let unsupported_g1: Vec<&str> = overlay
        .requirements
        .iter()
        .filter(|r| matches!(&r.status, rfcspec_model::Status::Unsupported(reason) if reason.starts_with("ess-gap G1:")))
        .map(|r| r.id.as_str())
        .collect();
    for id in ["R7.4.1-3", "R7.4.1-4", "R7.4.1-6"] {
        assert!(unsupported_g1.contains(&id), "{id}");
    }
    let spans: Vec<(usize, &str)> = overlay
        .requirements
        .iter()
        .filter(|r| r.id.starts_with("R7.4.1-") && r.level == "MUST NOT")
        .map(|r| (r.span.start, r.id.as_str()))
        .collect();
    assert_eq!(
        spans,
        [(2533, "R7.4.1-3"), (2540, "R7.4.1-4"), (2595, "R7.4.1-6")]
    );
    let abnf: Vec<&str> = overlay
        .requirements
        .iter()
        .filter(|r| r.source == "abnf-comment")
        .map(|r| r.id.as_str())
        .collect();
    assert_eq!(abnf, ["R5.2-8", "R5.2-9", "R5.2-10"]);
}

#[test]
fn a_copy_of_the_profile_checks_as_the_original() {
    let root = scratch("copy");
    let profile = root.join("websocket");
    copy_websocket(&profile);
    let output = rfcspec(&["check", "--profile", profile.to_str().unwrap()]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::remove_dir_all(root).unwrap();
}
