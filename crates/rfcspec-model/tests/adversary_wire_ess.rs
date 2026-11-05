use rfcspec_model::wire::CatalogueField;

#[test]
fn layout_field_reads_the_ess_dependent_length() {
    let ess_layout_field = r#"{
        "name": "payload-data",
        "length": {"field": "payload-len", "unit": "octets"}
    }"#;
    let read = serde_json::from_str::<CatalogueField>(ess_layout_field);
    assert!(
        read.is_ok(),
        "ESS rfcspec.catalogue.LayoutField declares length: Optional<DependentLength>: {read:?}"
    );
}

#[test]
#[ignore = "story:constraint-ess-shape: Constraint is externally tagged, not ESS FieldConstraint {kind, values}"]
fn layout_field_reads_the_ess_field_constraint_shape() {
    let ess_layout_field = r#"{
        "name": "opcode",
        "bits": 4,
        "constraints": [{"kind": "range", "values": [0, 15]}]
    }"#;
    let read = serde_json::from_str::<CatalogueField>(ess_layout_field);
    assert!(
        read.is_ok(),
        "ESS rfcspec.catalogue.FieldConstraint is {{kind, values}}: {read:?}"
    );
}
