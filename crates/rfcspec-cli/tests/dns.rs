//! The `dns` profile (RFC 1035) as a user runs `rfcspec` on it: fetch, read, extract and check,
//! and its entry in the protocol catalogue.

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

const PROFILE: &str = "profiles/dns";

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
        .prefix(&format!("dns-{name}-"))
        .tempdir_in(scratch)
        .unwrap();
    let dir = temp.path().join("dns");
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
fn extract_lists_no_bcp14_requirement_and_blocks_carry_the_71_headings_sections() {
    let catalogue = json(&["extract", "--profile", PROFILE, "--rfc", "1035"]);
    assert_eq!(catalogue["rfc"], 1035);
    assert_eq!(catalogue["requirements"].as_array().unwrap().len(), 0);
    assert!(!catalogue["prose_candidates"].as_array().unwrap().is_empty());

    let document = json(&["read", "--profile", PROFILE, "--rfc", "1035"]);
    assert_eq!(document["furniture"].as_array().unwrap().len(), 164);
    let blocks = document["blocks"].as_array().unwrap();
    let mut sections: Vec<&str> = blocks
        .iter()
        .filter_map(|block| block["section"].as_str())
        .collect();
    sections.dedup();
    assert_eq!(sections.len(), 71, "{sections:?}");
    assert_eq!(sections.iter().filter(|n| !n.contains('.')).count(), 9);
    assert_eq!((sections[0], sections[70]), ("1", "9"));
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
    assert_eq!(at(1401)["section"], "4.1.1");
    assert_eq!(at(1752)["section"], "4.2.1");
    assert_eq!(at(2526)["section"], "7.3");
    assert!(at(1).get("section").is_none_or(serde_json::Value::is_null));
}

#[test]
fn the_rfc1035_catalogue_is_byte_identical_across_two_runs() {
    let args = ["extract", "--profile", PROFILE, "--rfc", "1035"];
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
    assert!(
        stdout.contains(
            "profiles/dns: overlay holds (6 requirements, 1 layouts, 1 machines, 0 findings, 1 layers, 0 recorded runs)"
        ),
        "{stdout}"
    );
}

#[test]
fn time_and_websocket_still_check_lower_and_generate_without_change() {
    for profile in ["profiles/time", "profiles/websocket"] {
        succeeded(&rfcspec(&["check", "--profile", profile]));
        succeeded(&rfcspec(&["lower", "--profile", profile, "--check"]));
        succeeded(&rfcspec(&["generate", "--profile", profile, "--check"]));
    }
}

#[test]
fn the_profile_names_rfc_1035_and_the_overlay_reviews_the_header_and_its_z_requirement() {
    let profile = fs::read_to_string(repository().join(PROFILE).join("profile.yaml")).unwrap();
    assert!(
        profile.starts_with(
            "name: dns\nrfcs:\n  - number: 1035\n    title: Domain Names - Implementation and Specification\n    sections: [\"4.1.1\", \"4.2.1\", \"7.3\"]\n"
        ),
        "{profile}"
    );
    let text = fs::read_to_string(repository().join(PROFILE).join("overlay.yaml")).unwrap();
    assert!(
        text.starts_with("format: rfcspec-overlay/1\nrfc: 1035\nkind: K1\n"),
        "{text}"
    );
    assert!(text.ends_with("findings: []\n"), "{text}");
    let overlay: serde_json::Value = rfcspec_model::from_yaml(&text).unwrap();
    assert_eq!(
        overlay["requirements"][0],
        serde_json::json!({
            "id": "R-HDR-1",
            "source": "prose",
            "level": "MUST",
            "quote": "Must be zero in all queries and responses.",
            "span": {"start": 1473, "end": 1474},
            "status": {"satisfied": "layout.header"}
        })
    );
    let layouts = overlay["layouts"].as_array().unwrap();
    assert_eq!(layouts.len(), 1);
    let header = &layouts[0];
    assert_eq!(header["name"], "header");
    assert_eq!(header["quote"], "The header contains the following fields:");
    assert_eq!(
        header["span"],
        serde_json::json!({"start": 1403, "end": 1403})
    );
    assert!(header.get("epoch").is_none() && header.get("length").is_none());
    let fields: Vec<(&str, u64)> = header["fields"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| {
            assert_eq!(f["signed"], false, "{f}");
            assert_eq!(f["order"], "big-endian", "{f}");
            let constraints = if f["name"] == "z" {
                serde_json::json!([{"equals": 0}])
            } else {
                serde_json::Value::Null
            };
            assert_eq!(
                f.get("constraints").cloned().unwrap_or_default(),
                constraints
            );
            (f["name"].as_str().unwrap(), f["bits"].as_u64().unwrap())
        })
        .collect();
    assert_eq!(
        fields,
        [
            ("id", 16),
            ("qr", 1),
            ("opcode", 4),
            ("aa", 1),
            ("tc", 1),
            ("rd", 1),
            ("ra", 1),
            ("z", 3),
            ("rcode", 4),
            ("qdcount", 16),
            ("ancount", 16),
            ("nscount", 16),
            ("arcount", 16),
        ]
    );
    assert_eq!(fields.iter().map(|(_, bits)| bits).sum::<u64>(), 96);
}

#[test]
fn generate_check_passes_for_dns_and_fails_on_drift() {
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
fn the_dns_codec_is_a_workspace_member_and_task_check_runs_the_dns_task() {
    let manifest = fs::read_to_string(repository().join("Cargo.toml")).unwrap();
    assert!(
        manifest.contains("\"profiles/dns/generated/rust\""),
        "{manifest}"
    );
    let generated =
        fs::read_to_string(repository().join(PROFILE).join("generated/rust/Cargo.toml")).unwrap();
    assert!(generated.contains("name = \"rfc1035-dns\""), "{generated}");
    let text = fs::read_to_string(repository().join("Taskfile.yml")).unwrap();
    let taskfile: serde_json::Value = rfcspec_model::from_yaml(&text).unwrap();
    assert!(
        taskfile["tasks"]["check"]["cmds"]
            .as_array()
            .unwrap()
            .iter()
            .any(|cmd| cmd["task"] == "dns"),
        "{text}"
    );
    let dns: Vec<&str> = taskfile["tasks"]["dns"]["cmds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|cmd| cmd.as_str().or_else(|| cmd["cmd"].as_str()))
        .collect();
    assert_eq!(
        dns,
        [
            "cargo run -q --locked -p rfcspec-cli -- lower --profile profiles/dns --check",
            "ess protocol explore --path profiles/dns/generated/query.protospec.yaml --inputs profiles/dns/witnesses/query.inputs.json | grep -q '\"verdict\": \"passed\"'",
            "cargo run -q --locked -p rfcspec-cli -- generate --profile profiles/dns --check",
            "cargo test -q --locked -p rfc1035-dns",
        ]
    );
}

#[test]
fn the_profile_declares_one_udp_layer_correlated_by_id() {
    let profile: rfcspec_model::Profile =
        rfcspec_model::read_yaml(&repository().join(PROFILE).join("profile.yaml")).unwrap();
    assert_eq!(profile.semantics.len(), 1);
    assert_eq!(
        rfcspec_model::to_yaml(&profile.semantics[0]).unwrap(),
        "layer: udp\nunit: datagrams\ndelivery:\n  ordered: false\n  reliable: false\n  duplicates: true\ndirection: half-duplex\nmultiplexing: sessions\nexchange: request-response\ncorrelation: by-id\nlifetime: connectionless\nroles:\n- client\n- server\n"
    );
    let outputs = catalog::outputs(&repository()).unwrap();
    let page = &outputs["website/docs/catalog/dns.md"];
    let section = page
        .split_once("## Communication semantics")
        .unwrap()
        .1
        .split("\n## ")
        .next()
        .unwrap();
    let rows: Vec<&str> = section
        .lines()
        .filter(|line| line.starts_with("| ") && !line.starts_with("| Layer"))
        .collect();
    assert_eq!(rows.len(), 1, "{section}");
    assert!(rows[0].starts_with("| udp | datagrams |"), "{}", rows[0]);
    assert!(rows[0].contains("| by-id |"), "{}", rows[0]);
}

const QUERY_REQUIREMENTS: [(&str, &str, &str, u64, u64, &str); 5] = [
    (
        "R-ID-1",
        "MUST",
        "This identifier is copied the corresponding reply",
        1424,
        1425,
        "satisfied",
    ),
    (
        "R-MATCH-1",
        "SHOULD",
        "The recommended strategy is to do a preliminary matching using the ID field in the domain header",
        2544,
        2545,
        "satisfied",
    ),
    (
        "R-MATCH-2",
        "SHOULD",
        "and then to verify that the question section corresponds to the information currently desired.",
        2545,
        2546,
        "unsupported",
    ),
    (
        "R-UDP-1",
        "SHOULD NOT",
        "Queries or their responses may be reordered by the network, or by processing in name servers, so resolvers should not depend on them being returned in order.",
        1762,
        1764,
        "satisfied",
    ),
    (
        "R-UDP-2",
        "MUST",
        "Queries sent using UDP may be lost, and hence a retransmission strategy is required.",
        1761,
        1762,
        "unsupported",
    ),
];

#[test]
fn the_overlay_accepts_the_five_query_requirements_with_their_statuses() {
    let overlay: rfcspec_model::Overlay =
        rfcspec_model::read_yaml(&repository().join(PROFILE).join("overlay.yaml")).unwrap();
    let value: serde_json::Value =
        serde_json::from_str(&rfcspec_model::to_json(&overlay).unwrap()).unwrap();
    let requirements = value["requirements"].as_array().unwrap();
    assert_eq!(requirements.len(), 6);
    for ((id, level, quote, start, end, status), requirement) in
        QUERY_REQUIREMENTS.iter().zip(&requirements[1..])
    {
        assert_eq!(requirement["id"], *id);
        assert_eq!(requirement["source"], "prose", "{id}");
        assert_eq!(requirement["level"], *level, "{id}");
        assert_eq!(requirement["quote"], *quote, "{id}");
        assert_eq!(
            requirement["span"],
            serde_json::json!({"start": start, "end": end}),
            "{id}"
        );
        let statuses: Vec<&String> = requirement["status"].as_object().unwrap().keys().collect();
        assert_eq!(statuses, [status], "{id}");
    }
    let satisfied = |id: &str| {
        requirements.iter().find(|r| r["id"] == id).unwrap()["status"]["satisfied"].clone()
    };
    assert_eq!(satisfied("R-ID-1"), "query.server.R-ID-1-answer-query");
    assert_eq!(
        satisfied("R-MATCH-1"),
        "query.client.R-MATCH-1-accept-response"
    );
    assert_eq!(satisfied("R-UDP-1"), "query.server.R-UDP-1-late-response");
    let machine = &overlay.machines[0];
    assert_eq!(
        (
            overlay.machines.len(),
            machine.name.as_str(),
            machine.transport.as_str()
        ),
        (1, "query", "udp")
    );
    let ids: std::collections::BTreeSet<&str> =
        overlay.requirements.iter().map(|r| r.id.as_str()).collect();
    for participant in &machine.participants {
        for transition in &participant.transitions {
            assert!(!transition.implements.is_empty(), "{}", transition.label);
            for id in &transition.implements {
                assert!(ids.contains(id.as_str()), "{id} of {}", transition.label);
            }
        }
    }
}

#[test]
fn lower_writes_the_query_model_and_its_check_fails_without_it() {
    succeeded(&rfcspec(&["lower", "--profile", PROFILE, "--check"]));
    let (_temp, profile) = scratch("lower");
    let from = repository().join(PROFILE);
    for file in catalog::files(&from).unwrap() {
        fs::create_dir_all(profile.join(&file).parent().unwrap()).unwrap();
        fs::copy(from.join(&file), profile.join(&file)).unwrap();
    }
    let model = profile.join("generated/query.protospec.yaml");
    fs::remove_file(&model).unwrap();
    let output = rfcspec(&["lower", "--profile", profile.to_str().unwrap(), "--check"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("query.protospec.yaml is missing"),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    succeeded(&rfcspec(&["lower", "--profile", profile.to_str().unwrap()]));
    assert_eq!(
        fs::read(&model).unwrap(),
        fs::read(from.join("generated/query.protospec.yaml")).unwrap()
    );
    fs::remove_dir_all(profile).unwrap();
}

#[test]
fn the_dns_page_lists_the_query_model_and_the_five_query_requirements() {
    let outputs = catalog::outputs(&repository()).unwrap();
    let page = &outputs["website/docs/catalog/dns.md"];
    assert!(
        page.contains(
            "| [`query.protospec.yaml`](pathname:///rfcspec/downloads/dns/query.protospec.yaml) | `ess-protospec/1` model of machine `query` |"
        ),
        "{page}"
    );
    assert!(outputs.contains_key("website/static/downloads/dns/query.protospec.yaml"));
    for (id, level, quote, start, end, status) in QUERY_REQUIREMENTS {
        let row = format!("| {id} | {level} | all | {quote} | lines {start}-{end} | {status} |");
        assert!(page.contains(&row), "{row} not in\n{page}");
    }
}

#[test]
fn the_dns_page_lists_the_crate_downloads_and_the_reviewed_requirement() {
    let outputs = catalog::outputs(&repository()).unwrap();
    let page = &outputs["website/docs/catalog/dns.md"];
    assert!(
        page.contains(
            "| R-HDR-1 | MUST | all | Must be zero in all queries and responses. | lines 1473-1474 | satisfied | layout.header |"
        ),
        "{page}"
    );
    for file in ["rust/Cargo.toml", "rust/src/lib.rs"] {
        assert!(
            page.contains(&format!(
                "| [`{file}`](pathname:///rfcspec/downloads/dns/{file}) | generated Rust crate |"
            )),
            "{page}"
        );
        assert!(outputs.contains_key(&format!("website/static/downloads/dns/{file}")));
    }
}

#[test]
fn fetch_of_rfc_1035_twice_from_a_local_server_writes_the_committed_lock_and_changes_no_byte() {
    let body = fs::read(repository().join(PROFILE).join("sources/rfc1035.txt")).unwrap();
    assert_eq!(body.len(), 122549);
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
        "1035",
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
    assert!(String::from_utf8_lossy(&lock).contains(
        "- rfc: 1035\n  file: sources/rfc1035.txt\n  sha256: d14ae809fc9b41bbcae26bb38c937c9515808b944f3252b00de9fe0e95f4fdfb\n"
    ));
    let second = succeeded(&rfcspec(&args));
    assert!(second.contains("(unchanged)"), "{second}");
    assert_eq!(fs::read(profile.join("rfcspec.lock")).unwrap(), lock);
    assert_eq!(fs::read(profile.join("sources/rfc1035.txt")).unwrap(), body);
    fs::remove_dir_all(profile).unwrap();
}

#[test]
fn the_catalogue_lists_dns_beside_ntp_time_and_websocket_and_writes_its_page() {
    let outputs = catalog::outputs(&repository()).unwrap();
    let index = &outputs["website/docs/catalog/index.md"];
    let rows: Vec<&str> = index
        .lines()
        .filter(|line| line.starts_with("| ["))
        .collect();
    assert_eq!(rows.len(), 4, "{index}");
    assert!(
        rows[0].starts_with("| [dns](/docs/catalog/dns) |"),
        "{index}"
    );
    assert!(
        rows[1].starts_with("| [ntp](/docs/catalog/ntp) |"),
        "{index}"
    );
    assert!(
        rows[2].starts_with("| [time](/docs/catalog/time) |"),
        "{index}"
    );
    assert!(
        rows[3].starts_with("| [websocket](/docs/catalog/websocket) |"),
        "{index}"
    );
    assert_eq!(
        rows[0],
        "| [dns](/docs/catalog/dns) | RFC 1035 Domain Names - Implementation and Specification | K1 | S6 | 3 MUST, 2 SHOULD, 1 SHOULD NOT | 4 | 2 | 0 | 0 | no recorded run |"
    );
    assert!(outputs.contains_key("website/docs/catalog/dns.md"));
    for (path, text) in [
        ("website/docs/catalog/index.md", index),
        (
            "website/docs/catalog/dns.md",
            &outputs["website/docs/catalog/dns.md"],
        ),
    ] {
        assert_eq!(
            &fs::read_to_string(repository().join(path)).unwrap_or_default(),
            text,
            "{path} is stale: run task docs-generate"
        );
    }
}
