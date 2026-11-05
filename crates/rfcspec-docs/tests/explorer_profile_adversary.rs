//! Adversarial cases over the explorer's profile files (`website/static/data/profiles/<name>.json`)
//! and the copied view rows they are written through.

use std::{fs, path::PathBuf};

use serde_json::{Map, Value};

fn repository() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn committed(name: &str) -> Value {
    let path = repository().join(format!("website/static/data/profiles/{name}.json"));
    serde_json::from_str(&fs::read_to_string(&path).unwrap()).unwrap()
}

/// A satisfied status shows its element only: no overlay records test evidence, so the explorer
/// writes none, and `rfcspec.profile.RecordSatisfiedStatus`'s evidence rule is not yet met by
/// any committed status. Publishing the evidence is story:status-evidence; when it lands, this
/// case turns into "every satisfied status row names its test evidence".
#[test]
fn every_satisfied_status_row_names_its_element_and_no_evidence_until_story_status_evidence() {
    let mut wrong = Vec::new();
    let mut satisfied = 0;
    for name in ["dns", "ntp", "time", "websocket"] {
        for requirement in committed(name)["requirements"].as_array().unwrap() {
            let status = &requirement["status"];
            if status["kind"] == "satisfied" {
                satisfied += 1;
                let evidence = status["evidence"].as_array().map_or(0, Vec::len);
                let element = status["element"].as_str().unwrap_or_default();
                if evidence != 0 || element.trim().is_empty() {
                    wrong.push(status["requirement_id"].as_str().unwrap().to_owned());
                }
            }
        }
    }
    assert!(satisfied > 0, "the profiles hold satisfied statuses");
    assert!(
        wrong.is_empty(),
        "{} satisfied status rows with evidence or without an element, first {:?}",
        wrong.len(),
        &wrong[..wrong.len().min(3)]
    );
}

/// The schema definitions of a copy and its view row, without what only names them.
fn shape(defs: &Map<String, Value>, name: &str) -> Value {
    let mut def = defs[name].clone();
    let members = def.as_object_mut().unwrap();
    for naming in ["title", "description", "x-ess-kind", "x-ess-name"] {
        members.remove(naming);
    }
    def
}

/// The copies and the view rows they claim to repeat field for field, compared by their schema:
/// every property, its type and whether it is required.
fn drifted(schema: &Value) -> Vec<String> {
    let defs = schema["$defs"].as_object().unwrap();
    [
        (
            "rfcspec.profile.ExplorerStatus",
            "rfcspec.profile.Statuses.row",
        ),
        (
            "rfcspec.profile.ExplorerDecisionRow",
            "rfcspec.review.Decisions.row",
        ),
        (
            "rfcspec.profile.ExplorerLayoutRow",
            "rfcspec.catalogue.Layouts.row",
        ),
    ]
    .into_iter()
    .filter(|(copy, row)| shape(defs, copy) != shape(defs, row))
    .map(|(copy, row)| format!("{copy} is not {row}"))
    .collect()
}

fn generated_schema() -> Value {
    serde_json::from_str(
        &fs::read_to_string(
            repository().join("generated/rust/rfcspec-explorer-profile-types/source.schema.json"),
        )
        .unwrap(),
    )
    .unwrap()
}

/// The implementor's drift test rereads each written value through the row type. The row types
/// carry `deny_unknown_fields`, so a field the row loses fails it; an optional field the row
/// gains does not, because the value lacks it, it reads back absent and is skipped again on
/// writing. Nor do the fields the writer always leaves absent (`evidence`, `edit`, `errata_id`,
/// `errata_status`, `corrected_text`, `confirms`) ever reach the reread, so their types are
/// unchecked. Here the row gains an optional field in a copy of the committed schema: the
/// field-for-field comparison catches it, and a value written through the copy still rereads
/// through the row it no longer matches.
#[test]
fn a_row_gaining_an_optional_field_is_drift_the_reread_does_not_see() {
    let schema = generated_schema();
    assert!(drifted(&schema).is_empty(), "{:?}", drifted(&schema));

    let mut mutated = schema.clone();
    mutated["$defs"]["rfcspec.profile.Statuses.row"]["properties"]["superseded_by"] =
        serde_json::json!({ "type": "string" });
    assert_eq!(
        drifted(&mutated),
        ["rfcspec.profile.ExplorerStatus is not rfcspec.profile.Statuses.row"]
    );
    let row = mutated["$defs"]["rfcspec.profile.Statuses.row"].clone();
    let required: Vec<&str> = row["required"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect();
    let allowed = row["properties"].as_object().unwrap();
    for requirement in committed("websocket")["requirements"].as_array().unwrap() {
        let status = requirement["status"].as_object().unwrap();
        assert!(status.keys().all(|k| allowed.contains_key(k)));
        assert!(required.iter().all(|k| status.contains_key(*k)));
    }
}
