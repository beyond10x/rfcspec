//! The status data, `website/data/status.json` (`b10x-status/1`), and the status page,
//! `website/docs/status.md`, both from [`CAPABILITIES`].
//!
//! A capability is `shipped` (it exists in the release named by [`RELEASE`] and names the gate
//! step or test that holds it), `decided` (a design decision, design §0) or `planned` (a
//! milestone of design §14, or what remains of one). A test refuses a shipped item without its
//! evidence, and a test path that does not exist.

use serde_json::json;

use crate::page::{Meta, cell, front_matter};

/// Where the status data lands, relative to the repository root.
pub const DATA: &str = "website/data/status.json";
/// Where the status page lands, relative to the repository root.
pub const PAGE: &str = "website/docs/status.md";

/// The date the list below was last checked against the repository.
const AS_OF: &str = "2026-11-06";
/// The release the shipped rows were checked against.
const RELEASE: &str = "v0.2.0";

/// One capability on the status page.
pub struct Capability {
    pub area: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    /// `shipped`, `decided` or `planned`.
    pub status: &'static str,
    /// The documentation route that explains it, `/docs/...`.
    pub href: &'static str,
    /// For a shipped capability, what holds it: `task <name>` (a step of `task check` or
    /// `task website`) or a test file relative to the repository root. Empty otherwise.
    pub evidence: &'static str,
}

const fn shipped(
    area: &'static str,
    label: &'static str,
    detail: &'static str,
    href: &'static str,
    evidence: &'static str,
) -> Capability {
    Capability {
        area,
        label,
        detail,
        status: "shipped",
        href,
        evidence,
    }
}

const fn decided(
    area: &'static str,
    label: &'static str,
    detail: &'static str,
    href: &'static str,
) -> Capability {
    Capability {
        area,
        label,
        detail,
        status: "decided",
        href,
        evidence: "",
    }
}

const fn planned(
    area: &'static str,
    label: &'static str,
    detail: &'static str,
    href: &'static str,
) -> Capability {
    Capability {
        area,
        label,
        detail,
        status: "planned",
        href,
        evidence: "",
    }
}

/// Every capability the site claims, in the order the status table lists them.
pub const CAPABILITIES: &[Capability] = &[
    shipped(
        "Read",
        "Plain text and RFCXML",
        "Fetch and lock an RFC's plain text or RFCXML v3 source, read it into one document model, and extract its BCP 14 requirements with cited spans; RFC 9220 is the RFCXML test case.",
        "/docs/concepts/catalogue",
        "crates/rfcspec-cli/tests/rfcxml.rs",
    ),
    shipped(
        "Read",
        "The whole RFC index",
        "Every RFC the index publishes as RFCXML or plain text, with its requirement counts by level and section and its RFCXML references, unreviewed.",
        "/docs/catalog/rfcs",
        "task bulk",
    ),
    shipped(
        "Profiles",
        "Four profiles end to end",
        "RFC 868, RFC 6455, RFC 1035 and RFC 5905: a reviewed overlay, lowered models that pass bounded exploration, and a generated codec crate with its tests.",
        "/docs/catalog",
        "task time, task websocket, task dns, task ntp",
    ),
    shipped(
        "Profiles",
        "Recorded proposals and review",
        "Status proposals from a model, recorded with their cited spans, settled by review into the overlay and replayed offline in the gate.",
        "/docs/concepts/ai-proposals",
        "task websocket",
    ),
    shipped(
        "Site",
        "The RFC explorer",
        "Search and filter the RFC index, open an RFC's requirements by section, update graph and references, and see a profiled RFC's reviewed layer.",
        "/docs/catalog/rfcs",
        "task explorer",
    ),
    shipped(
        "Wire",
        "ABNF parse, resolve and match",
        "RFC 5234 and RFC 7405 ABNF parsed, references resolved to rules, core rules and RFC imports, and input matched against a rule.",
        "/docs/concepts/lowering-to-ess",
        "crates/rfcspec-abnf/tests/rfc6455_core.rs",
    ),
    shipped(
        "Wire",
        "Catalogue grammars",
        "Each section's ABNF figures collected into one catalogue grammar, from plain text and RFCXML.",
        "/docs/concepts/catalogue",
        "crates/rfcspec-wire/tests/grammars.rs",
    ),
    decided(
        "Design",
        "The catalogue, rfcspec/1",
        "An RFC becomes requirements, grammars, layouts, registries, defined procedures and limits, each with a source span and a digest (design D1).",
        "/docs/concepts/catalogue",
    ),
    decided(
        "Design",
        "ESS as the target",
        "Machines lower to ess-protospec/1, message and value types and registries to an ess/N system; what ESS cannot carry is listed as a gap (design D2).",
        "/docs/concepts/lowering-to-ess",
    ),
    decided(
        "Design",
        "A wire-format sidecar",
        "Bytes and bits live in rfcspec-wire/1: ABNF verbatim, bit-level ABNF and packet layouts (design D3).",
        "/docs/concepts/lowering-to-ess",
    ),
    decided(
        "Design",
        "Communication semantics",
        "One value per axis for each protocol layer: unit, delivery, direction, multiplexing, exchange, correlation, lifetime, layering, roles (design D4).",
        "/docs/concepts/communication-semantics",
    ),
    decided(
        "Design",
        "AI proposes, review decides",
        "Model output is a recorded proposal with cited spans, replayed offline; no gate calls a model (design D5, D6).",
        "/docs/concepts/ai-proposals",
    ),
    decided(
        "Design",
        "Which kinds of RFC are modelled",
        "Wire protocols, data formats, extensions and registries are modelled; algorithms as signatures with test vectors; the rest get a requirement catalogue (design D11).",
        "/docs/concepts/rfc-kinds",
    ),
    planned(
        "M0 Read",
        "Defined terms and a survey by kind",
        "What remains of M0: defined terms in the catalogue, and a survey classifying every RFC in the index by kind.",
        "/docs/design",
    ),
    planned(
        "M1 Wire",
        "The rest of the wire layer",
        "What remains of M1: the RFC 2616 and bit-level ABNF dialects, packet diagrams read into layouts, the RFC 6455 frame agreement check, and the handshake codec and frame codec vectors.",
        "/docs/examples/websocket",
    ),
    planned(
        "M2 Lower",
        "Proposals for parties, semantics and machines",
        "What remains of M2: recorded proposals for the parties, the communication semantics and the WebSocket machines; status proposals, review and lowering are shipped.",
        "/docs/concepts/ai-proposals",
    ),
    planned(
        "M3 Generate",
        "A generated WebSocket core",
        "What remains of M3: machine, limits and roles in the generated crate, an Autobahn run and a recorded requirement matrix; the frame codec is shipped.",
        "/docs/examples/websocket",
    ),
    planned(
        "M4 DNS",
        "Concurrent DNS sessions and the gap proposals",
        "What remains of M4: concurrent sessions, and the ESS gap proposals G1, G3 and G4 with WebSocket and DNS as evidence; the header codec and query machine are shipped.",
        "/docs/concepts/lowering-to-ess",
    ),
    planned(
        "M5 Spindle",
        "RFCspec cores in Spindle",
        "The WebSocket codec as a pure WASM module and a websocket channel port in a Spindle environment.",
        "/docs/concepts/lowering-to-ess",
    ),
    planned(
        "M6 TCP",
        "A transport",
        "TCP as the hard case: sequence-number arithmetic, windows and timers from measurements.",
        "/docs/concepts/communication-semantics",
    ),
];

/// A capability's detail, with what holds it when it is shipped.
fn detail(c: &Capability) -> String {
    if c.evidence.is_empty() {
        c.detail.to_owned()
    } else {
        format!("{} Held by {}.", c.detail, c.evidence)
    }
}

/// `website/data/status.json`.
pub fn data() -> String {
    let items: Vec<_> = CAPABILITIES
        .iter()
        .map(|c| {
            json!({
                "area": c.area,
                "label": c.label,
                "detail": detail(c),
                "status": c.status,
                "href": c.href,
            })
        })
        .collect();
    let document = json!({
        "format": "b10x-status/1",
        "asOf": AS_OF,
        "source": format!("rfcspec-docs generate (crates/rfcspec-docs/src/status.rs), checked against {RELEASE}"),
        "items": items,
    });
    serde_json::to_string_pretty(&document).expect("status data serializes") + "\n"
}

/// `website/docs/status.md`.
pub fn page() -> String {
    let mut page = front_matter(&Meta {
        title: "Status",
        label: "Status",
        description: "What RFCspec has shipped, what it has decided and what it plans.",
        position: 30,
    });
    let count = |status: &str| CAPABILITIES.iter().filter(|c| c.status == status).count();
    page.push_str(&format!(
        "Checked against {RELEASE} on {AS_OF}: {} shipped, {} decided, {} planned. **Shipped** \
         exists in {RELEASE} and names the gate step (`task <name>`) or the test that holds it; \
         **decided** is a decision in the design; **planned** is a milestone of the design's \
         section 14, or what remains of one, and nothing more.\n\n\
         | Area | Capability | Status | What it means | Held by |\n|---|---|---|---|---|\n",
        count("shipped"),
        count("decided"),
        count("planned"),
    ));
    for c in CAPABILITIES {
        let held = if c.evidence.is_empty() {
            "-".to_owned()
        } else {
            c.evidence
                .split(", ")
                .map(|e| format!("`{e}`"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        page.push_str(&format!(
            "| {} | [{}]({}) | {} | {} | {} |\n",
            cell(c.area),
            cell(c.label),
            c.href,
            c.status,
            cell(c.detail),
            held,
        ));
    }
    page
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn every_status_is_known_and_every_route_is_a_doc() {
        for c in CAPABILITIES {
            assert!(
                matches!(c.status, "shipped" | "decided" | "planned"),
                "{}: {}",
                c.label,
                c.status
            );
            assert!(c.href.starts_with("/docs/"), "{}", c.href);
        }
    }

    #[test]
    fn a_shipped_item_names_evidence_that_exists_and_no_other_item_does() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let taskfile = std::fs::read_to_string(root.join("Taskfile.yml")).unwrap();
        for c in CAPABILITIES {
            if c.status != "shipped" {
                assert!(c.evidence.is_empty(), "{}", c.label);
                continue;
            }
            assert!(!c.evidence.is_empty(), "{} names no evidence", c.label);
            for e in c.evidence.split(", ") {
                match e.strip_prefix("task ") {
                    Some(task) => assert!(
                        taskfile.contains(&format!("\n  {task}:\n")),
                        "{}: no task {task}",
                        c.label
                    ),
                    None => assert!(root.join(e).is_file(), "{}: no file {e}", c.label),
                }
            }
        }
    }

    #[test]
    fn data_and_page_list_every_capability() {
        let data: serde_json::Value = serde_json::from_str(&data()).unwrap();
        assert_eq!(data["format"], "b10x-status/1");
        assert_eq!(data["items"].as_array().unwrap().len(), CAPABILITIES.len());
        let page = page();
        for c in CAPABILITIES {
            assert!(
                page.contains(&format!("[{}]({})", c.label, c.href)),
                "{}",
                c.label
            );
        }
    }
}
