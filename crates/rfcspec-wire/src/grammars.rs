//! The ABNF grammars of an RFC.
//!
//! Collection is mechanical (design sections 4, 5 and 12.2): every ABNF figure of a section joins
//! that section's one grammar, whether or not it parses. A figure is ABNF by its `type` when the
//! RFCXML reader kept one (`abnf`, or a type starting with it such as RFC 9110's `abnf9110`),
//! else when one of its lines defines a rule ([`rfcspec_doc::abnf_rule`]).

use rfcspec_model::{Block, BlockKind, Document, SourceFormat, Span, text_digest, wire::Grammar};

/// Whether a block is an ABNF figure: by its `type` when it has one (any type starting with
/// `abnf`), else by a line that defines a rule.
pub fn is_abnf(block: &Block) -> bool {
    block.kind == BlockKind::Figure
        && match block.figure_type.as_deref() {
            Some(kind) => kind.starts_with("abnf"),
            None => block.lines.iter().any(|line| rfcspec_doc::abnf_rule(line)),
        }
}

/// The grammars of a document, in document order: one per section holding an ABNF figure, its
/// id `rfc<N>/grammar/<section number>` (the anchor for an unnumbered RFCXML section, `-` outside
/// every section), its rules the lines of the section's ABNF figures verbatim, joined by
/// newlines, its span from the first to the last of those lines, and no imports.
pub fn collect(document: &Document) -> Vec<Grammar> {
    let xml = document.format == SourceFormat::Xml;
    let mut grammars: Vec<(Option<String>, Grammar)> = Vec::new();
    for block in document.blocks.iter().filter(|block| is_abnf(block)) {
        let key = block.section.clone().or_else(|| block.anchor.clone());
        let rules = block.lines.join("\n");
        match grammars.iter_mut().find(|(seen, _)| *seen == key) {
            Some((_, grammar)) => {
                grammar.rules.push('\n');
                grammar.rules.push_str(&rules);
                grammar.span.start = grammar.span.start.min(block.span.start);
                grammar.span.end = grammar.span.end.max(block.span.end);
            }
            None => {
                let name = key.clone().unwrap_or_else(|| "-".to_string());
                let section = if xml {
                    block.anchor.clone()
                } else {
                    block.section.clone()
                };
                grammars.push((
                    key,
                    Grammar {
                        id: format!("rfc{}/grammar/{name}", document.rfc),
                        rfc: document.rfc,
                        name,
                        section,
                        rules,
                        uses: Vec::new(),
                        span: Span {
                            start: block.span.start,
                            end: block.span.end,
                        },
                        digest: String::new(),
                    },
                ));
            }
        }
    }
    grammars
        .into_iter()
        .map(|(_, mut grammar)| {
            grammar.digest = text_digest(&grammar.rules);
            grammar
        })
        .collect()
}
