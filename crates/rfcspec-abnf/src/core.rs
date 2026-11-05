//! The core rules of RFC 5234 Appendix B.1, built as code.

use std::sync::OnceLock;

use crate::{Node, Rule, RuleSet};

/// The sixteen core rules; their references resolve among themselves.
pub(crate) fn rules() -> &'static RuleSet {
    static RULES: OnceLock<RuleSet> = OnceLock::new();
    RULES.get_or_init(|| RuleSet {
        rules: vec![
            rule("ALPHA", alt(vec![range(0x41, 0x5A), range(0x61, 0x7A)])),
            rule("BIT", alt(vec![chars("0"), chars("1")])),
            rule("CHAR", range(0x01, 0x7F)),
            rule("CR", byte(0x0D)),
            rule("CRLF", cat(vec![refer("CR"), refer("LF")])),
            rule("CTL", alt(vec![range(0x00, 0x1F), byte(0x7F)])),
            rule("DIGIT", range(0x30, 0x39)),
            rule("DQUOTE", byte(0x22)),
            rule(
                "HEXDIG",
                alt(vec![
                    refer("DIGIT"),
                    chars("A"),
                    chars("B"),
                    chars("C"),
                    chars("D"),
                    chars("E"),
                    chars("F"),
                ]),
            ),
            rule("HTAB", byte(0x09)),
            rule("LF", byte(0x0A)),
            rule(
                "LWSP",
                Node::Repetition {
                    min: 0,
                    max: None,
                    node: Box::new(alt(vec![
                        refer("WSP"),
                        cat(vec![refer("CRLF"), refer("WSP")]),
                    ])),
                },
            ),
            rule("OCTET", range(0x00, 0xFF)),
            rule("SP", byte(0x20)),
            rule("VCHAR", range(0x21, 0x7E)),
            rule("WSP", alt(vec![refer("SP"), refer("HTAB")])),
        ],
        loose_comments: Vec::new(),
    })
}

fn rule(name: &str, definition: Node) -> Rule {
    Rule {
        name: name.to_string(),
        line: 0,
        definition,
        comments: Vec::new(),
    }
}

fn alt(nodes: Vec<Node>) -> Node {
    Node::Alternation(nodes)
}

fn cat(nodes: Vec<Node>) -> Node {
    Node::Concatenation(nodes)
}

fn refer(name: &str) -> Node {
    Node::Reference(name.to_string())
}

fn range(low: u64, high: u64) -> Node {
    Node::NumRange { low, high }
}

fn byte(value: u64) -> Node {
    Node::NumSequence(vec![value])
}

fn chars(text: &str) -> Node {
    Node::CharVal {
        text: text.to_string(),
        case_sensitive: false,
    }
}
