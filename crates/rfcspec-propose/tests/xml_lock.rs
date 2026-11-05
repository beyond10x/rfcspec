//! A profile whose lock names an RFCXML source, loaded by S3.

use std::{fs, path::PathBuf};

use rfcspec_model::{BlockKind, SourceFormat};

/// `rfcspec propose` over a profile in which `rfcspec fetch --format xml` locked the source reads
/// it through the RFCXML reader, not the plain-text reader over XML markup.
#[test]
fn loading_a_profile_whose_source_is_locked_as_xml_reads_it_with_the_xml_reader() {
    let xml = fs::read(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../rfcspec-doc/tests/fixtures/rfc9220.xml"),
    )
    .unwrap();
    let profile = tempfile::tempdir().unwrap();
    fs::write(
        profile.path().join("profile.yaml"),
        "name: h3\nrfcs:\n- number: 9220\n  title: Bootstrapping WebSockets with HTTP/3\n  sections:\n  - '3'\n",
    )
    .unwrap();
    fs::write(
        profile.path().join("overlay.yaml"),
        "format: rfcspec-overlay/1\nrfc: 9220\nrequirements: []\nlayouts: []\nmachines: []\nfindings: []\n",
    )
    .unwrap();
    rfcspec_source::store_format(profile.path(), 9220, SourceFormat::Xml, &xml).unwrap();

    let loaded = rfcspec_propose::Workspace::load(profile.path()).unwrap();
    let document = loaded.source_document();
    assert_eq!(document.format, SourceFormat::Xml);
    assert!(document.furniture.is_empty());
    assert!(
        document
            .blocks
            .iter()
            .all(|block| !block.text().contains('<')),
        "markup left in a block"
    );
    let heading = document
        .blocks
        .iter()
        .find(|block| block.kind == BlockKind::Heading && block.section.as_deref() == Some("3"))
        .unwrap();
    assert_eq!(
        heading.anchor.as_deref(),
        Some("websockets-upgrade-over-http3")
    );
}
