use std::{fs, path::PathBuf};

use rfcspec_model::{Lock, sha256_hex};
use rfcspec_wire::{diagram, grammars};

fn websocket() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/websocket")
}

#[test]
fn stubs_collect_nothing_from_rfc6455() {
    let text = fs::read_to_string(websocket().join("sources/rfc6455.txt")).unwrap();
    let sha256 = sha256_hex(text.as_bytes());
    let lock: Lock = rfcspec_model::read_yaml(&websocket().join("rfcspec.lock")).unwrap();
    assert_eq!(
        lock.sources[0].sha256, sha256,
        "the source is the locked one"
    );
    let document = rfcspec_doc::read(6455, &sha256, &text);
    assert!(!document.blocks.is_empty(), "RFC 6455 was read");

    assert_eq!(grammars::collect(&document), Vec::new());
    let (layouts, notes) = diagram::collect(&document);
    assert_eq!(layouts, Vec::new());
    assert_eq!(notes, Vec::<String>::new());
}
