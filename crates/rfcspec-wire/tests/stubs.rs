use std::{fs, path::PathBuf};

use rfcspec_model::{Lock, sha256_hex};
use rfcspec_wire::{diagram, grammars};

fn websocket() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/websocket")
}

#[test]
fn grammars_are_collected_and_diagrams_are_not_yet_from_rfc6455() {
    let text = fs::read_to_string(websocket().join("sources/rfc6455.txt")).unwrap();
    let sha256 = sha256_hex(text.as_bytes());
    let lock: Lock = rfcspec_model::read_yaml(&websocket().join("rfcspec.lock")).unwrap();
    assert_eq!(
        lock.sources[0].sha256, sha256,
        "the source is the locked one"
    );
    let document = rfcspec_doc::read(6455, &sha256, &text);
    assert!(!document.blocks.is_empty(), "RFC 6455 was read");

    let found = grammars::collect(&document);
    let ids: Vec<&str> = found.iter().map(|g| g.id.as_str()).collect();
    assert_eq!(
        ids,
        [
            "rfc6455/grammar/3",
            "rfc6455/grammar/4.2.2",
            "rfc6455/grammar/4.3",
            "rfc6455/grammar/5.2",
            "rfc6455/grammar/5.3",
            "rfc6455/grammar/9.1",
        ]
    );

    let lines: Vec<&str> = text.lines().collect();
    let line = |number: usize| lines[number - 1];
    let handshake = &found[2];
    assert_eq!(handshake.section.as_deref(), Some("4.3"));
    assert_eq!(
        handshake.span,
        rfcspec_model::Span {
            start: 1384,
            end: 1423
        }
    );
    for number in [1384, 1387, 1389, 1415, 1420, 1423] {
        assert!(
            handshake.rules.lines().any(|rule| rule == line(number)),
            "line {number} is in the 4.3 grammar"
        );
    }
    for number in 1416..=1419 {
        let prose = line(number);
        assert!(
            prose.trim().is_empty() || !handshake.rules.lines().any(|rule| rule == prose),
            "prose line {number} is not in the 4.3 grammar"
        );
    }

    let frame = &found[3];
    assert_eq!(
        frame.span,
        rfcspec_model::Span {
            start: 1687,
            end: 1783
        }
    );
    let figures: Vec<_> = document
        .blocks
        .iter()
        .filter(|block| block.section.as_deref() == Some("5.2") && grammars::is_abnf(block))
        .collect();
    assert_eq!(figures.len(), 19, "the 5.2 grammar holds 19 rule figures");
    for figure in figures {
        for number in figure.span.start..=figure.span.end {
            assert!(
                line(number).trim().is_empty()
                    || frame.rules.lines().any(|rule| rule == line(number)),
                "rule figure line {number} is in the 5.2 grammar"
            );
        }
    }
    for number in 1535..=1552 {
        let drawn = line(number);
        assert!(
            drawn.trim().is_empty() || !frame.rules.lines().any(|rule| rule == drawn),
            "diagram line {number} is not in the 5.2 grammar"
        );
    }
    for grammar in &found {
        assert_eq!(grammar.uses, Vec::new());
        assert_eq!(grammar.digest, rfcspec_model::text_digest(&grammar.rules));
    }
    let (layouts, notes) = diagram::collect(&document);
    assert_eq!(layouts, Vec::new());
    assert_eq!(notes, Vec::<String>::new());
}
