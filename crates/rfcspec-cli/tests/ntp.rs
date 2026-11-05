//! The `ntp` profile (RFC 5905) as a user runs `rfcspec` on it: fetch, read, extract and check,
//! generate, its entry in the protocol catalogue, and the generated `rfc5905-ntp` codec itself,
//! compiled here from its committed source.

use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    process::{Command, Output},
    thread,
};

use rfcspec_docs::catalog;
use tempfile::TempDir;

#[path = "../../../profiles/ntp/generated/rust/src/lib.rs"]
mod rfc5905_ntp;

const PROFILE: &str = "profiles/ntp";

const SHA256: &str = "8b7abd903c60202e2953ee012fefa6916666c3b0c71825c748cdd262024fb268";

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

/// A fresh directory under the workspace's build directory, not the system temporary one,
/// removed when the returned guard drops.
fn scratch(name: &str) -> (TempDir, PathBuf) {
    let scratch = repository().join("target/rfcspec-cli");
    fs::create_dir_all(&scratch).unwrap();
    let temp = tempfile::Builder::new()
        .prefix(&format!("ntp-{name}-"))
        .tempdir_in(scratch)
        .unwrap();
    let dir = temp.path().join("ntp");
    fs::create_dir_all(&dir).unwrap();
    (temp, dir)
}

fn succeeded(output: &Output) -> String {
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    assert_eq!(
        output.status.code(),
        Some(0),
        "{stdout}{}",
        String::from_utf8_lossy(&output.stderr)
    );
    stdout
}

fn json(args: &[&str]) -> serde_json::Value {
    serde_json::from_slice(succeeded(&rfcspec(args)).as_bytes()).unwrap()
}

#[test]
fn extract_lists_the_13_bcp14_requirements_with_their_sections() {
    let catalogue = json(&["extract", "--profile", PROFILE, "--rfc", "5905"]);
    assert_eq!(catalogue["rfc"], 5905);
    let requirements = catalogue["requirements"].as_array().unwrap();
    let found: Vec<(&str, &str)> = requirements
        .iter()
        .map(|r| (r["id"].as_str().unwrap(), r["section"].as_str().unwrap()))
        .collect();
    assert_eq!(
        found,
        [
            ("R2-1", "2"),
            ("R2-2", "2"),
            ("R2-3", "2"),
            ("R7.4-1", "7.4"),
            ("R7.4-2", "7.4"),
            ("R7.4-3", "7.4"),
            ("R7.4-4", "7.4"),
            ("R7.4-5", "7.4"),
            ("R9.2-1", "9.2"),
            ("R9.2-2", "9.2"),
            ("R9.2-3", "9.2"),
            ("R11.2.3-1", "11.2.3"),
            ("R11.2.3-2", "11.2.3")
        ]
    );
    assert!(!catalogue["prose_candidates"].as_array().unwrap().is_empty());
}

#[test]
fn read_reports_329_furniture_lines_and_blocks_carry_the_37_headings_sections() {
    let document = json(&["read", "--profile", PROFILE, "--rfc", "5905"]);
    assert_eq!(document["furniture"].as_array().unwrap().len(), 329);
    let blocks = document["blocks"].as_array().unwrap();
    let mut sections: Vec<&str> = blocks
        .iter()
        .filter_map(|block| block["section"].as_str())
        .collect();
    sections.dedup();
    assert_eq!(sections.len(), 37, "{sections:?}");
    assert_eq!((sections[0], sections[36]), ("1", "18.2"));
    for section in &sections {
        let opens = blocks.iter().find(|block| block["section"] == *section);
        let first = opens.unwrap()["lines"][0].as_str().unwrap();
        assert!(first.starts_with(&format!("{section}. ")), "{first}");
    }
    let at = |line: u64| {
        blocks
            .iter()
            .find(|block| {
                block["span"]["start"].as_u64().unwrap() <= line
                    && line <= block["span"]["end"].as_u64().unwrap()
            })
            .unwrap()
    };
    for (line, section) in [
        (889, "7.2"),
        (938, "7.3"),
        (1061, "7.3"),
        (1431, "8"),
        (1804, "9.2"),
        (2829, "13"),
    ] {
        assert_eq!(at(line)["section"], section, "{line}");
    }
    assert_eq!(at(1015)["kind"], at(1059)["kind"]);
    assert_eq!(at(1015)["span"]["end"], 1059);
    assert!(at(7).get("section").is_none_or(serde_json::Value::is_null));
}

#[test]
fn the_rfc5905_catalogue_is_byte_identical_across_two_runs() {
    let args = ["extract", "--profile", PROFILE, "--rfc", "5905"];
    let first = rfcspec(&args);
    succeeded(&first);
    assert_eq!(first.stdout, rfcspec(&args).stdout);
}

#[test]
fn check_all_profiles_checks_dns_ntp_time_and_websocket() {
    let stdout = succeeded(&rfcspec(&["check", "--all", "profiles"]));
    let names: Vec<&str> = stdout
        .lines()
        .filter_map(|line| line.split_once(": overlay holds"))
        .map(|(path, _)| path)
        .collect();
    assert_eq!(
        names,
        [
            "profiles/dns",
            "profiles/ntp",
            "profiles/time",
            "profiles/websocket"
        ],
        "{stdout}"
    );
    succeeded(&rfcspec(&["check", "--profile", PROFILE]));
}

#[test]
fn dns_time_and_websocket_still_check_lower_and_generate_without_change() {
    for profile in ["profiles/dns", "profiles/time", "profiles/websocket"] {
        succeeded(&rfcspec(&["check", "--profile", profile]));
        succeeded(&rfcspec(&["lower", "--profile", profile, "--check"]));
        succeeded(&rfcspec(&["generate", "--profile", profile, "--check"]));
    }
}

#[test]
fn the_profile_names_rfc_5905_with_five_sections_and_no_semantics() {
    let profile: serde_json::Value = rfcspec_model::from_yaml(
        &fs::read_to_string(repository().join(PROFILE).join("profile.yaml")).unwrap(),
    )
    .unwrap();
    assert_eq!(profile["name"], "ntp");
    let rfcs = profile["rfcs"].as_array().unwrap();
    assert_eq!(rfcs.len(), 1);
    assert_eq!(rfcs[0]["number"], 5905);
    assert_eq!(
        rfcs[0]["title"],
        "Network Time Protocol Version 4: Protocol and Algorithms Specification"
    );
    let text = fs::read_to_string(repository().join(PROFILE).join("sources/rfc5905.txt")).unwrap();
    assert_eq!(
        text.lines().nth(16).unwrap().trim(),
        "Network Time Protocol Version 4: Protocol and Algorithms Specification"
    );
    let sections: Vec<&str> = rfcs[0]["sections"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap())
        .collect();
    assert_eq!(sections, ["7.2", "7.3", "8", "9.2", "13"]);
    assert!(
        profile
            .get("semantics")
            .is_none_or(|s| s.as_array().is_some_and(Vec::is_empty)),
        "{profile:?}"
    );
}

/// `rfcspec check` requires a status for every catalogue requirement in a scoped section, so
/// the three BCP 14 requirements of section 9.2 are settled; the prose requirement `R-VN-1`
/// (the version range of section 9.2) is satisfied by the `header` layout.
#[test]
fn the_overlay_is_k1_for_rfc_5905_with_the_header_layout_and_four_reviewed_requirements() {
    let overlay: serde_json::Value = rfcspec_model::from_yaml(
        &fs::read_to_string(repository().join(PROFILE).join("overlay.yaml")).unwrap(),
    )
    .unwrap();
    assert_eq!(overlay["format"], "rfcspec-overlay/1");
    assert_eq!(overlay["rfc"], 5905);
    assert_eq!(overlay["kind"], "K1");
    assert_eq!(overlay["layouts"][0]["name"], "header");
    for absent in ["machines", "findings"] {
        assert!(
            overlay
                .get(absent)
                .is_none_or(|s| s.as_array().is_some_and(Vec::is_empty)),
            "{absent}: {overlay:?}"
        );
    }
    let settled: Vec<(&str, &str, &str, u64, u64)> = overlay["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| {
            (
                r["id"].as_str().unwrap(),
                r["source"].as_str().unwrap(),
                r["level"].as_str().unwrap(),
                r["span"]["start"].as_u64().unwrap(),
                r["span"]["end"].as_u64().unwrap(),
            )
        })
        .collect();
    assert_eq!(
        settled,
        [
            ("R9.2-1", "capitals-text", "MAY", 1889, 1890),
            ("R9.2-2", "capitals-text", "MAY", 1924, 1925),
            ("R9.2-3", "capitals-text", "SHOULD", 1926, 1927),
            ("R-VN-1", "prose", "MUST", 1812, 1814)
        ]
    );
    let satisfied: Vec<(&str, &str)> = overlay["requirements"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| Some((r["id"].as_str()?, r["status"]["satisfied"].as_str()?)))
        .collect();
    assert_eq!(satisfied, [("R-VN-1", "layout.header")]);
}

#[test]
fn fetch_of_rfc_5905_twice_from_a_local_server_writes_the_committed_lock_and_changes_no_byte() {
    let body = fs::read(repository().join(PROFILE).join("sources/rfc5905.txt")).unwrap();
    assert_eq!(body.len(), 241_096);
    assert_eq!(String::from_utf8_lossy(&body).lines().count(), 6163);
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
    let (_temp, profile) = scratch("fetch");
    let dir = profile.to_str().unwrap();
    let args = [
        "fetch",
        "--rfc",
        "5905",
        "--profile",
        dir,
        "--base-url",
        &base,
    ];
    succeeded(&rfcspec(&args));
    let lock = fs::read(profile.join("rfcspec.lock")).unwrap();
    assert_eq!(
        String::from_utf8_lossy(&lock),
        fs::read_to_string(repository().join(PROFILE).join("rfcspec.lock")).unwrap()
    );
    assert!(String::from_utf8_lossy(&lock).contains(&format!(
        "- rfc: 5905\n  file: sources/rfc5905.txt\n  sha256: {SHA256}\n"
    )));
    let second = succeeded(&rfcspec(&args));
    assert!(second.contains("(unchanged)"), "{second}");
    assert_eq!(fs::read(profile.join("rfcspec.lock")).unwrap(), lock);
    assert_eq!(fs::read(profile.join("sources/rfc5905.txt")).unwrap(), body);
    fs::remove_dir_all(profile).unwrap();
}

#[test]
fn the_catalogue_lists_ntp_at_stage_s6_with_its_four_reviewed_requirements_and_writes_its_page() {
    let outputs = catalog::outputs(&repository()).unwrap();
    let index = &outputs["website/docs/catalog/index.md"];
    let row = index
        .lines()
        .find(|line| line.starts_with("| [ntp](/docs/catalog/ntp) |"))
        .unwrap_or_else(|| panic!("{index}"));
    assert_eq!(
        row,
        "| [ntp](/docs/catalog/ntp) | RFC 5905 Network Time Protocol Version 4: Protocol and Algorithms Specification | K1 | S6 | 1 MUST, 1 SHOULD, 2 MAY | 1 | 3 | 0 | 10 | no recorded run |"
    );
    assert!(outputs.contains_key("website/docs/catalog/ntp.md"));
    for (path, text) in [
        ("website/docs/catalog/index.md", index),
        (
            "website/docs/catalog/ntp.md",
            &outputs["website/docs/catalog/ntp.md"],
        ),
    ] {
        assert_eq!(
            &fs::read_to_string(repository().join(path)).unwrap_or_default(),
            text,
            "{path} is stale: run task docs-generate"
        );
    }
}

#[test]
fn check_reports_the_ntp_overlay_with_four_requirements_and_the_header_layout() {
    let stdout = succeeded(&rfcspec(&["check", "--profile", PROFILE]));
    assert!(
        stdout.contains(
            "profiles/ntp: overlay holds (4 requirements, 1 layouts, 0 machines, 0 findings, 0 layers, 0 recorded runs)"
        ),
        "{stdout}"
    );
}

#[test]
fn generate_check_passes_for_ntp_and_fails_on_drift() {
    succeeded(&rfcspec(&["generate", "--profile", PROFILE, "--check"]));
    let (_temp, profile) = scratch("generate");
    let from = repository().join(PROFILE);
    for file in catalog::files(&from).unwrap() {
        fs::create_dir_all(profile.join(&file).parent().unwrap()).unwrap();
        fs::copy(from.join(&file), profile.join(&file)).unwrap();
    }
    let lib = profile.join("generated/rust/src/lib.rs");
    assert!(lib.is_file());
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
    fs::remove_dir_all(profile).unwrap();
}

#[test]
fn the_ntp_codec_is_a_workspace_member_and_task_check_runs_the_ntp_task() {
    let manifest = fs::read_to_string(repository().join("Cargo.toml")).unwrap();
    assert!(
        manifest.contains("\"profiles/ntp/generated/rust\""),
        "{manifest}"
    );
    let generated =
        fs::read_to_string(repository().join(PROFILE).join("generated/rust/Cargo.toml")).unwrap();
    assert!(generated.contains("name = \"rfc5905-ntp\""), "{generated}");
    let text = fs::read_to_string(repository().join("Taskfile.yml")).unwrap();
    let taskfile: serde_json::Value = rfcspec_model::from_yaml(&text).unwrap();
    assert!(
        taskfile["tasks"]["check"]["cmds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cmd| cmd["task"] == "ntp"),
        "{text}"
    );
    let ntp: Vec<&str> = taskfile["tasks"]["ntp"]["cmds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|cmd| cmd.as_str().or_else(|| cmd["cmd"].as_str()))
        .collect();
    assert_eq!(
        ntp,
        [
            "cargo run -q --locked -p rfcspec-cli -- generate --profile profiles/ntp --check",
            "cargo test -q --locked -p rfc5905-ntp",
        ]
    );
}

#[test]
fn the_ntp_page_lists_the_crate_downloads_and_the_reviewed_requirement() {
    let outputs = catalog::outputs(&repository()).unwrap();
    let page = &outputs["website/docs/catalog/ntp.md"];
    assert!(
        page.contains(
            "| R-VN-1 | MUST | all | Format checks require correct field length and alignment, acceptable version number (1-4) | lines 1812-1814 | satisfied | layout.header |"
        ),
        "{page}"
    );
    for file in ["rust/Cargo.toml", "rust/src/lib.rs"] {
        assert!(
            page.contains(&format!(
                "| [`{file}`](pathname:///rfcspec/downloads/ntp/{file}) | generated Rust crate |"
            )),
            "{page}"
        );
        assert!(outputs.contains_key(&format!("website/static/downloads/ntp/{file}")));
    }
}

/// A header of version 4, client mode, with every other field set, as octets.
fn client_header() -> Vec<u8> {
    let mut bytes = vec![0x23, 0x02, 0xfa, 0xec];
    bytes.extend(4u8..48);
    bytes
}

#[test]
fn the_codec_parses_48_octets_prints_them_back_and_47_are_incomplete() {
    let bytes = client_header();
    let (header, used) = rfc5905_ntp::parse(&bytes).unwrap();
    assert_eq!(used, 48);
    assert_eq!(
        (
            header.leap,
            header.version,
            header.mode,
            header.stratum,
            header.poll,
            header.precision
        ),
        (0, 4, 3, 2, -6, -20)
    );
    assert_eq!(header.rootdelay, 0x0405_0607);
    assert_eq!(header.xmt, 0x2829_2a2b_2c2d_2e2f);
    assert_eq!(rfc5905_ntp::print(&header), bytes);
    assert_eq!(
        rfc5905_ntp::parse(&bytes[..47]),
        Err(rfc5905_ntp::Error::Incomplete)
    );
    let mut longer = bytes.clone();
    longer.push(0);
    assert_eq!(rfc5905_ntp::parse(&longer), Ok((header, 48)));
}

#[test]
fn the_codec_round_trips_poll_and_precision_at_minus_128_minus_1_and_127() {
    for (value, octet) in [(-128i8, 0x80u8), (-1, 0xff), (127, 0x7f)] {
        let header = rfc5905_ntp::Header {
            version: 4,
            poll: value,
            precision: value,
            ..rfc5905_ntp::Header::default()
        };
        let bytes = rfc5905_ntp::print(&header);
        assert_eq!(bytes.len(), 48);
        assert_eq!((bytes[2], bytes[3]), (octet, octet));
        assert_eq!(rfc5905_ntp::parse(&bytes), Ok((header, 48)));
    }
}

#[test]
fn the_codec_accepts_versions_1_to_4_and_refuses_0_and_5_to_7() {
    for version in 0u8..8 {
        let mut bytes = client_header();
        bytes[0] = (bytes[0] & 0b1100_0111) | (version << 3);
        let parsed = rfc5905_ntp::parse(&bytes);
        if (1..=4).contains(&version) {
            assert_eq!(parsed.map(|(h, _)| h.version), Ok(version));
        } else {
            assert_eq!(
                parsed,
                Err(rfc5905_ntp::Error::Constraint {
                    field: "version",
                    value: u64::from(version)
                })
            );
        }
    }
}
