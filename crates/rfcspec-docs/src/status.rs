//! The status data, `website/data/status.json` (`b10x-status/1`), and the status page,
//! `website/docs/status.md`, both from [`CAPABILITIES`].
//!
//! RFCspec is at the design stage: every capability is `decided` (a design decision, design §0) or
//! `planned` (a milestone, design §14). Nothing is `shipped`, and a test refuses a shipped item
//! until one names the test that holds it.

use serde_json::json;

use crate::page::{Meta, cell, front_matter};

/// Where the status data lands, relative to the repository root.
pub const DATA: &str = "website/data/status.json";
/// Where the status page lands, relative to the repository root.
pub const PAGE: &str = "website/docs/status.md";

/// The date the list below was last checked against the design.
const AS_OF: &str = "2026-11-04";

/// One capability on the status page.
pub struct Capability {
    pub area: &'static str,
    pub label: &'static str,
    pub detail: &'static str,
    /// `decided` or `planned`.
    pub status: &'static str,
    /// The documentation route that explains it, `/docs/...`.
    pub href: &'static str,
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
    }
}

/// Every capability the site claims, in the order the status table lists them.
pub const CAPABILITIES: &[Capability] = &[
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
        "Fetch, read and extract",
        "RFC 6455 from plain text and RFC 9220 from RFCXML: sections, requirements, ABNF and defined terms; a survey classifying every RFC in the index by kind.",
        "/docs/design",
    ),
    planned(
        "M1 Wire",
        "ABNF and the wire layer",
        "The RFC 6455 handshake grammar and frame bit-level ABNF and diagram, an agreement check between them, and a generated codec with round-trip and mutation vectors.",
        "/docs/examples/websocket",
    ),
    planned(
        "M2 Lower",
        "Proposals, review and lowering",
        "Recorded proposals for parties, semantics and the WebSocket machines, overlay review, and lowering that ess specify validate and ess protocol explore accept.",
        "/docs/concepts/lowering-to-ess",
    ),
    planned(
        "M3 Generate",
        "A generated WebSocket core",
        "A generated sans-IO Rust crate with codec, machine, limits and roles, an Autobahn run and the requirement matrix.",
        "/docs/examples/websocket",
    ),
    planned(
        "M4 DNS",
        "A second, binary protocol",
        "DNS over UDP: binary layouts, correlation by id, concurrent sessions; the ESS gap proposals with WebSocket and DNS as evidence.",
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

/// `website/data/status.json`.
pub fn data() -> String {
    let items: Vec<_> = CAPABILITIES
        .iter()
        .map(|c| {
            json!({
                "area": c.area,
                "label": c.label,
                "detail": c.detail,
                "status": c.status,
                "href": c.href,
            })
        })
        .collect();
    let document = json!({"format": "b10x-status/1", "asOf": AS_OF, "items": items});
    serde_json::to_string_pretty(&document).expect("status data serializes") + "\n"
}

/// `website/docs/status.md`.
pub fn page() -> String {
    let mut page = front_matter(&Meta {
        title: "Status",
        label: "Status",
        description: "What RFCspec has decided and what it plans. Nothing is shipped yet.",
        position: 30,
    });
    let count = |status: &str| CAPABILITIES.iter().filter(|c| c.status == status).count();
    page.push_str(&format!(
        "Checked against the design on {AS_OF}. RFCspec is at the design stage: {} decisions, {} \
         planned milestones, nothing shipped. **Decided** is a decision in the design; **planned** \
         is a milestone of the design's section 14 and nothing more.\n\n\
         | Area | Capability | Status | What it means |\n|---|---|---|---|\n",
        count("decided"),
        count("planned"),
    ));
    for c in CAPABILITIES {
        page.push_str(&format!(
            "| {} | [{}]({}) | {} | {} |\n",
            cell(c.area),
            cell(c.label),
            c.href,
            c.status,
            cell(c.detail)
        ));
    }
    page
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nothing_is_claimed_as_shipped_and_every_route_is_a_doc() {
        for c in CAPABILITIES {
            assert!(
                c.status == "decided" || c.status == "planned",
                "{}: {}",
                c.label,
                c.status
            );
            assert!(c.href.starts_with("/docs/"), "{}", c.href);
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
