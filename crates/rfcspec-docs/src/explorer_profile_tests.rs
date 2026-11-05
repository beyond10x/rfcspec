//! Tests of the explorer's profile files (`website/static/data/profiles/<name>.json`) over the
//! committed profiles, read back through the generated types.

use std::path::PathBuf;

use rfcspec_explorer_profile_types::{
    EssPresence as Presence, RfcspecCatalogueLayoutsRow, RfcspecCatalogueLevelCount,
    RfcspecProfileExplorerProfile, RfcspecProfileStatusesRow, RfcspecReviewDecisionsRow,
};
use rfcspec_model::{DecisionKind, Status, sha256_hex};
use serde_json::Value;

use super::*;
use crate::catalog::{self, gap, load};

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

const PROFILES: [&str; 4] = ["dns", "ntp", "time", "websocket"];

fn entry(name: &str) -> Entry {
    load(&repository(), &format!("profiles/{name}")).unwrap()
}

/// A profile's entry, its file and the file read back through the generated type.
fn generated(name: &str) -> (Entry, String, RfcspecProfileExplorerProfile) {
    let entry = entry(name);
    let text = profile(&repository(), &entry)
        .unwrap()
        .unwrap_or_else(|| panic!("{name} has an overlay, so it has a file"));
    let parsed = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name}: {e}"));
    (entry, text, parsed)
}

fn number(n: &Number) -> u64 {
    n.as_u64().unwrap()
}

fn present<T>(value: &Presence<T>) -> Option<&T> {
    match value {
        Presence::Present(value) => Some(value),
        Presence::Absent => None,
    }
}

/// A value read back as a generated row type and written again: equal to the value only when
/// the value has exactly that row's shape.
fn reread<T: serde::de::DeserializeOwned + serde::Serialize>(value: &Value) -> Value {
    let row: T =
        serde_json::from_value(value.clone()).unwrap_or_else(|e| panic!("not a row: {e}: {value}"));
    serde_json::to_value(row).unwrap()
}

/// The wire spelling of a generated enum value.
fn wire(value: &impl serde::Serialize) -> String {
    serde_json::to_value(value)
        .unwrap()
        .as_str()
        .unwrap()
        .to_owned()
}

/// Counts by level as the profile page writes them, `21 MUST, 2 MUST NOT`.
fn written(counts: &[RfcspecCatalogueLevelCount]) -> String {
    counts
        .iter()
        .map(|c| format!("{} {}", number(&c.count), wire(&c.level)))
        .collect::<Vec<_>>()
        .join(", ")
}

#[test]
fn the_websocket_file_holds_the_reviewed_and_unreviewed_counts_its_page_prints() {
    let (entry, _, parsed) = generated("websocket");
    assert_eq!(parsed.format, PROFILE_FORMAT);
    assert_eq!(parsed.profile.0, "websocket");
    assert_eq!(parsed.path, "profiles/websocket");
    assert_eq!(number(&parsed.rfc.0), 6455);
    let page = catalog::page(&entry);
    let mut levels: BTreeMap<(usize, String), usize> = BTreeMap::new();
    let mut kinds: BTreeMap<String, usize> = BTreeMap::new();
    for requirement in &parsed.requirements {
        *levels
            .entry((rank(&requirement.level), requirement.level.clone()))
            .or_default() += 1;
        *kinds.entry(wire(&requirement.status.kind)).or_default() += 1;
    }
    let levels = levels
        .iter()
        .map(|((_, level), n)| format!("{n} {level}"))
        .collect::<Vec<_>>()
        .join(", ");
    let count = |kind: &str| kinds.get(kind).copied().unwrap_or_default();
    let reviewed = format!(
        "Reviewed: {} requirements, by level {levels}; by status {} satisfied, {} unsupported, {} not applicable.\n",
        parsed.requirements.len(),
        count("satisfied"),
        count("unsupported"),
        count("not-applicable"),
    );
    assert!(page.contains(&reviewed), "{reviewed}");
    assert_eq!(parsed.requirements.len(), 155);

    let unreviewed = present(&parsed.unreviewed).expect("the profile holds its locked source");
    let total: u64 = unreviewed.iter().map(|c| number(&c.count)).sum();
    assert_eq!(total, 41);
    let line = format!(
        "Unreviewed: {total} requirements of the mechanical catalogue of RFC 6455 have no status, by level {}.",
        written(unreviewed)
    );
    assert!(page.contains(&line), "{line}");
}

#[test]
fn the_websocket_decisions_are_85_accepted_or_edited_and_29_rejected_each_naming_its_proposal_file()
{
    let (entry, _, parsed) = generated("websocket");
    let overlay = entry.overlay.as_ref().unwrap();
    assert_eq!(parsed.decisions.len(), overlay.decisions.len());
    let requirement_ids: Vec<&str> = parsed
        .requirements
        .iter()
        .map(|r| r.status.requirement_id.0.as_str())
        .collect();
    let (mut decided, mut rejected) = (0, 0);
    for (decision, recorded) in parsed.decisions.iter().zip(&overlay.decisions) {
        let row = &decision.row;
        assert_eq!(row.key.item.0, recorded.item);
        assert_eq!(row.key.text_digest.0, recorded.text_digest);
        assert_eq!(row.key.profile.0, "websocket");
        assert_eq!(row.profile.0, "websocket");
        assert_eq!(wire(&row.state), "current");
        assert_eq!(wire(&row.kind), recorded.kind.wire());
        assert!(row.edit.is_absent(), "{}", recorded.item);
        let value = serde_json::to_value(row).unwrap();
        assert_eq!(reread::<RfcspecReviewDecisionsRow>(&value), value);
        match recorded.kind {
            DecisionKind::Accept | DecisionKind::Edit => {
                decided += 1;
                assert!(row.reason.is_absent());
                assert!(
                    requirement_ids.contains(&recorded.item.as_str()),
                    "{} joins a requirement on its item id",
                    recorded.item
                );
            }
            DecisionKind::Reject => {
                rejected += 1;
                let reason = present(&row.reason).expect("a reject holds its reason");
                assert!(!reason.trim().is_empty());
                assert_eq!(Some(reason), recorded.reason.as_ref());
                assert!(!requirement_ids.contains(&recorded.item.as_str()));
            }
        }
        let id = present(&row.proposal_id).expect("every websocket decision decided a proposal");
        assert_eq!(Some(&id.0), recorded.proposal_id.as_ref());
        let path = present(&decision.proposal_path).unwrap();
        assert_eq!(*path, format!("profiles/websocket/proposals/{}.json", id.0));
        let file: Value = serde_json::from_str(
            &fs::read_to_string(repository().join(path)).unwrap_or_else(|e| panic!("{path}: {e}")),
        )
        .unwrap();
        assert_eq!(file["proposal_id"], id.0.as_str());
        assert_eq!(
            file["model_id"],
            present(&decision.model_id).unwrap().0.as_str()
        );
    }
    assert_eq!((decided, rejected), (85, 29));
}

#[test]
fn every_requirement_status_is_a_statuses_row_and_every_long_text_is_one_the_page_publishes() {
    for name in PROFILES {
        let (entry, text, parsed) = generated(name);
        let overlay = entry.overlay.as_ref().unwrap();
        assert_eq!(
            parsed.requirements.len(),
            overlay.requirements.len(),
            "{name}"
        );
        let mut published: BTreeSet<&str> = BTreeSet::new();
        for (requirement, recorded) in parsed.requirements.iter().zip(&overlay.requirements) {
            let status = &requirement.status;
            let id = format!("rfc{}/{}", overlay.rfc, recorded.id);
            assert_eq!(status.requirement_id.0, id);
            assert_eq!(status.key.requirement.0, id);
            assert_eq!(status.key.profile.0, name);
            assert_eq!(status.profile.0, name);
            assert_eq!(wire(&status.state), "Recorded");
            assert!(status.evidence.is_absent());
            let value = serde_json::to_value(status).unwrap();
            assert_eq!(reread::<RfcspecProfileStatusesRow>(&value), value);
            let (kind, detail) = match &recorded.status {
                Status::Satisfied(element) => ("satisfied", element),
                Status::Unsupported(reason) => ("unsupported", reason),
                Status::NotApplicable(reason) => ("not-applicable", reason),
            };
            assert_eq!(wire(&status.kind), kind, "{id}");
            let written = if kind == "satisfied" {
                assert!(status.reason.is_absent());
                present(&status.element)
            } else {
                assert!(status.element.is_absent());
                present(&status.reason)
            };
            assert_eq!(written, Some(detail), "{id}");
            assert_eq!(
                present(&requirement.gap).map(number),
                (kind == "unsupported")
                    .then(|| gap(detail))
                    .flatten()
                    .map(u64::from),
                "{id}"
            );
            assert_eq!(requirement.level, recorded.level);
            assert_eq!(requirement.transports, recorded.transports);
            assert_eq!(requirement.quote, recorded.quote);
            assert_eq!(
                (
                    number(&requirement.start_line),
                    number(&requirement.end_line)
                ),
                (recorded.span.start as u64, recorded.span.end as u64)
            );
            published.insert(&recorded.quote);
            published.insert(detail);
        }
        for decision in &overlay.decisions {
            if let Some(reason) = &decision.reason {
                published.insert(reason);
            }
        }
        let value: Value = serde_json::from_str(&text).unwrap();
        let mut strings = Vec::new();
        collect(&value, &mut strings);
        for string in strings.iter().filter(|s| s.len() > 40 && s.contains(' ')) {
            assert!(
                published.contains(string.as_str()),
                "{name}: a text its page does not publish: {string}"
            );
        }
    }
}

fn collect(value: &Value, out: &mut Vec<String>) {
    match value {
        Value::String(s) => out.push(s.clone()),
        Value::Array(items) => items.iter().for_each(|v| collect(v, out)),
        Value::Object(members) => members.values().for_each(|v| collect(v, out)),
        _ => {}
    }
}

#[test]
fn every_machine_links_its_lowered_document_download_and_an_existing_witness() {
    let expected: [(&str, &[&str]); 4] = [
        ("dns", &["query"]),
        ("ntp", &["exchange", "poll"]),
        ("time", &["tcp", "udp"]),
        ("websocket", &["close", "ping", "fragments"]),
    ];
    for (name, machines) in expected {
        let (entry, _, parsed) = generated(name);
        let downloads = catalog::render(std::slice::from_ref(&entry));
        let names: Vec<&str> = parsed.machines.iter().map(|m| m.name.as_str()).collect();
        assert_eq!(names, machines, "{name}");
        for (machine, recorded) in parsed
            .machines
            .iter()
            .zip(&entry.overlay.as_ref().unwrap().machines)
        {
            let slug = machine.name.as_str();
            assert_eq!(machine.key.profile.0, name);
            assert_eq!(machine.key.slug, slug);
            assert_eq!(wire(&machine.state), "Recorded");
            assert_eq!(machine.transport, recorded.transport);
            assert_eq!(
                machine.protospec_path,
                format!("profiles/{name}/generated/{slug}.protospec.yaml")
            );
            let lowered = fs::read(repository().join(&machine.protospec_path)).unwrap();
            assert_eq!(machine.protospec_digest.0, sha256_hex(&lowered));
            assert_eq!(
                machine.download,
                format!("downloads/{name}/{slug}.protospec.yaml")
            );
            assert_eq!(
                downloads[&format!("website/static/{}", machine.download)].as_bytes(),
                lowered.as_slice(),
                "{name}/{slug}"
            );
            assert_eq!(
                machine.witness,
                format!("profiles/{name}/witnesses/{slug}.inputs.json")
            );
            assert!(
                repository().join(&machine.witness).is_file(),
                "{}",
                machine.witness
            );
        }
    }
}

#[test]
fn the_time_profile_shows_its_tcp_and_udp_machines_and_no_decisions() {
    let (_, _, parsed) = generated("time");
    assert!(parsed.decisions.is_empty());
    assert_eq!(
        parsed
            .machines
            .iter()
            .map(|m| m.witness.as_str())
            .collect::<Vec<_>>(),
        [
            "profiles/time/witnesses/tcp.inputs.json",
            "profiles/time/witnesses/udp.inputs.json"
        ]
    );
    assert_eq!(present(&parsed.unreviewed).map(Vec::len), Some(0));
}

#[test]
fn the_websocket_frame_header_is_a_layouts_row_with_its_fields_and_bit_widths() {
    let (entry, _, parsed) = generated("websocket");
    let recorded = &entry.overlay.as_ref().unwrap().layouts;
    assert_eq!(parsed.layouts.len(), recorded.len());
    let layout = &parsed.layouts[0];
    assert_eq!(layout.row.layout_id.0, "rfc6455/layout/frame-header");
    assert_eq!(layout.row.name, "frame-header");
    assert_eq!(number(&layout.row.rfc.0), 6455);
    assert_eq!(wire(&layout.row.state), "read");
    let value = serde_json::to_value(&layout.row).unwrap();
    assert_eq!(reread::<RfcspecCatalogueLayoutsRow>(&value), value);
    let fields: Vec<(String, u64)> = layout
        .fields
        .iter()
        .map(|f| (f.name.clone(), number(present(&f.bits).unwrap())))
        .collect();
    let expected: Vec<(String, u64)> = recorded[0]
        .fields
        .iter()
        .map(|f| (f.name.clone(), u64::from(f.bits)))
        .collect();
    assert_eq!(fields, expected);
    assert_eq!(fields[..2], [("fin".into(), 1), ("rsv1".into(), 1)]);
    for (field, recorded) in layout.fields.iter().zip(&recorded[0].fields) {
        assert_eq!(
            present(&field.presence).map(|p| (p.field.clone(), p.equals.as_i64().unwrap())),
            recorded
                .presence
                .as_ref()
                .map(|p| (p.field.clone(), p.equals)),
            "{}",
            field.name
        );
        assert_eq!(field.constraints.len(), recorded.constraints.len());
    }
    assert!(
        layout.fields.iter().any(|f| !f.presence.is_absent()),
        "a field present only when another has a value"
    );
}

#[test]
fn no_overlay_no_file_and_no_locked_source_no_unreviewed_count() {
    let recorded = load(
        &repository(),
        "crates/rfcspec-docs/tests/fixtures/profiles/recorded",
    )
    .unwrap();
    assert!(profile(&repository(), &recorded).unwrap().is_none());
    let mut fixture = load(
        &repository(),
        "crates/rfcspec-docs/tests/fixtures/profiles/fixture",
    )
    .unwrap();
    let text = profile(&repository(), &fixture).unwrap().unwrap();
    let parsed: RfcspecProfileExplorerProfile = serde_json::from_str(&text).unwrap();
    assert_eq!(present(&parsed.unreviewed).map(Vec::len), Some(0));
    assert_eq!(parsed.requirements.len(), 3);
    assert_eq!(present(&parsed.requirements[0].gap).map(number), Some(1));
    fixture.catalogue = None;
    let text = profile(&repository(), &fixture).unwrap().unwrap();
    let parsed: RfcspecProfileExplorerProfile = serde_json::from_str(&text).unwrap();
    assert!(parsed.unreviewed.is_absent());
}

#[test]
fn the_committed_profile_files_are_current_one_per_profile() {
    let root = repository();
    let outputs = profiles(&root, &catalog::entries(&root).unwrap()).unwrap();
    assert_eq!(
        outputs.keys().cloned().collect::<Vec<_>>(),
        PROFILES.map(profile_path)
    );
    for (path, text) in &outputs {
        assert_eq!(
            &fs::read_to_string(root.join(path)).unwrap_or_default(),
            text,
            "{path} is current"
        );
    }
}

/// The generated schema the profile types and the view rows are generated from.
fn profile_schema() -> Value {
    serde_json::from_str(
        &fs::read_to_string(
            repository().join("generated/rust/rfcspec-explorer-profile-types/source.schema.json"),
        )
        .unwrap(),
    )
    .unwrap()
}

/// One property of a definition: its schema without the summary that only describes it, and
/// whether it is required.
fn property(def: &Value, name: &str) -> Option<(Value, bool)> {
    let mut schema = def["properties"].get(name)?.clone();
    if let Some(members) = schema.as_object_mut() {
        members.remove("description");
        members.remove("title");
    }
    let required = def["required"]
        .as_array()
        .is_some_and(|r| r.iter().any(|v| v == name));
    Some((schema, required))
}

fn property_names(def: &Value) -> BTreeSet<String> {
    def["properties"]
        .as_object()
        .map(|p| p.keys().cloned().collect())
        .unwrap_or_default()
}

/// The copies that repeat a view row, each with the row and the row fields it leaves out: every
/// other row field must have the same schema and requiredness in the copy, and the copy may hold
/// no field the row lacks unless `extras` names it.
const COPIES: [(&str, &str, &[&str], &[&str]); 4] = [
    (
        "rfcspec.profile.ExplorerStatus",
        "rfcspec.profile.Statuses.row",
        &[],
        &[],
    ),
    (
        "rfcspec.profile.ExplorerDecisionRow",
        "rfcspec.review.Decisions.row",
        &[],
        &[],
    ),
    (
        "rfcspec.profile.ExplorerLayoutRow",
        "rfcspec.catalogue.Layouts.row",
        &[],
        &[],
    ),
    (
        "rfcspec.profile.ExplorerMachine",
        "rfcspec.profile.Machines.row",
        &["unlinked_musts"],
        &["transport", "download", "witness"],
    ),
];

/// Every way a copy differs from its row in the schema: a property's schema, its requiredness,
/// a row property the copy lacks, a copy property the row lacks, or `additionalProperties`.
fn schema_drift(schema: &Value) -> Vec<String> {
    let defs = &schema["$defs"];
    let mut found = Vec::new();
    for (copy, row, left_out, extras) in COPIES {
        let (copy_def, row_def) = (&defs[copy], &defs[row]);
        assert!(copy_def.is_object(), "{copy} is in the schema");
        assert!(row_def.is_object(), "{row} is in the schema");
        if copy_def["additionalProperties"] != row_def["additionalProperties"] {
            found.push(format!("{copy}: additionalProperties"));
        }
        for name in property_names(row_def) {
            if left_out.contains(&name.as_str()) {
                if property(copy_def, &name).is_some() {
                    found.push(format!("{copy}.{name} is said to be left out"));
                }
                continue;
            }
            match (property(copy_def, &name), property(row_def, &name)) {
                (None, _) => found.push(format!("{copy} lacks {row}.{name}")),
                (Some(copied), Some(original)) if copied != original => {
                    found.push(format!("{copy}.{name} differs from {row}.{name}"));
                }
                _ => {}
            }
        }
        for name in property_names(copy_def) {
            if property(row_def, &name).is_none() && !extras.contains(&name.as_str()) {
                found.push(format!("{copy}.{name} is not in {row}"));
            }
        }
    }
    found
}

#[test]
fn the_copied_rows_and_the_machine_repeat_their_view_rows_in_the_generated_schema() {
    let schema = profile_schema();
    assert_eq!(schema_drift(&schema), Vec::<String>::new());

    for (copy, row, left_out, _) in COPIES {
        let mut gained = schema.clone();
        gained["$defs"][row]["properties"]["superseded_by"] = serde_json::json!({"type": "string"});
        assert_eq!(
            schema_drift(&gained),
            [format!("{copy} lacks {row}.superseded_by")],
            "an optional field the row gains"
        );

        let field = property_names(&schema["$defs"][row])
            .into_iter()
            .find(|name| !left_out.contains(&name.as_str()))
            .unwrap();
        let mut retyped = schema.clone();
        retyped["$defs"][row]["properties"][&field] = serde_json::json!({"type": "boolean"});
        assert_eq!(
            schema_drift(&retyped),
            [format!("{copy}.{field} differs from {row}.{field}")],
            "a field the row retypes"
        );

        let mut required = schema.clone();
        let list = required["$defs"][row]["required"].as_array_mut().unwrap();
        let was = list.len();
        list.retain(|name| *name != field.as_str());
        if list.len() == was {
            list.push(Value::String(field.clone()));
        }
        assert_eq!(
            schema_drift(&required),
            [format!("{copy}.{field} differs from {row}.{field}")],
            "a field whose requiredness the row changes"
        );
    }
}
