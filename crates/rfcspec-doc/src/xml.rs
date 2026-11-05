//! S1: the RFCXML v3 reader (design D7, RFC 7991). It turns a published RFC's XML into the same
//! normalized document the plain-text reader produces, with spans in lines of the XML source, plus
//! what only the markup knows: the anchor of each block's section, the `type` of each figure and
//! every `<bcp14>` element.
//!
//! Rules:
//!
//! - the abstract, the boilerplate sections, `<note>`s and every `<section>` and `<references>`
//!   of `<middle>` and `<back>` are read; the table of contents, the title and author front
//!   matter, `<reference>` entries (the cited documents' titles and abstracts), `<author>`
//!   addresses, `<iref>` and `<cref>` give no block;
//! - a section's `<name>` is a heading; a numbered one renders as the text format renders it,
//!   `3.  Title` (`Appendix A.  Title` in an appendix), its number read from `pn` (`section-3` is
//!   `3`, `section-appendix.a` is `A`); a section numbered `false` has no number; the abstract's
//!   heading is `Abstract`;
//! - every block carries the number and the anchor (its `anchor`, else its `pn`) of the innermost
//!   section, `<references>`, abstract or note it lies in;
//! - a `<t>`, and any other element holding only inline content (a list item, a table cell), is a
//!   paragraph; a `<dl>` is one paragraph with one line per term and definition, each entry after
//!   the first recorded as a sentence boundary (`entries`);
//! - a `<sourcecode>` or `<artwork>` is a figure, its lines verbatim and its `type` kept; of an
//!   `<artset>` only the artworks that are not SVG (`type="svg"` or holding elements) are;
//! - inline markup is removed and references render as xml2rfc renders them in text: `[RFC8441]`
//!   for a citation, `Section 15.6.2 of [HTTP]` or `Appendix A.3 of [HTTP/3]` with `section`, the
//!   derived content for `format="title"` and `format="counter"`; an `<eref>` without content is
//!   its target; a `<contact>` is its full name; a `<br/>` is a space, as a line break in a
//!   paragraph's text is;
//! - a block has one line per source line of its span, each trimmed (a figure's only at the end);
//!   a source line without text of the block is an empty line.

use std::collections::BTreeSet;

use rfcspec_model::{Bcp14Mark, Block, BlockKind, Document, SourceFormat, Span};
use roxmltree::{Node, ParsingOptions};

/// Why a source is not a readable RFCXML v3 document.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct XmlError(String);

impl std::fmt::Display for XmlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for XmlError {}

/// Read an RFC's RFCXML v3 source into a normalized document.
pub fn read(rfc: u32, sha256: &str, text: &str) -> Result<Document, XmlError> {
    let options = ParsingOptions {
        allow_dtd: true,
        ..ParsingOptions::default()
    };
    let xml = roxmltree::Document::parse_with_options(text, options)
        .map_err(|error| XmlError(format!("not well-formed XML: {error}")))?;
    let root = xml.root_element();
    if name(root) != "rfc" {
        return Err(XmlError(format!(
            "the root element is <{}>, not <rfc>",
            name(root)
        )));
    }
    if root.attribute("version") != Some("3") {
        return Err(XmlError(format!(
            "not RFCXML v3: <rfc version={:?}>",
            root.attribute("version").unwrap_or("")
        )));
    }
    let reader = Reader {
        line_starts: std::iter::once(0)
            .chain(text.match_indices('\n').map(|(index, _)| index + 1))
            .collect(),
        references: root
            .descendants()
            .filter(|node| matches!(name(*node), "reference" | "referencegroup"))
            .filter_map(|node| node.attribute("anchor"))
            .collect(),
    };
    let mut blocks = Vec::new();
    let outside = Context::default();
    for part in elements(root) {
        match name(part) {
            "front" => {
                for front in elements(part) {
                    match name(front) {
                        "abstract" | "note" => reader.section(front, &mut blocks),
                        "boilerplate" => reader.container(front, &outside, &mut blocks),
                        _ => {}
                    }
                }
            }
            "middle" | "back" => reader.container(part, &outside, &mut blocks),
            _ => {}
        }
    }
    Ok(Document {
        rfc,
        sha256: sha256.to_string(),
        format: SourceFormat::Xml,
        furniture: Vec::new(),
        blocks,
    })
}

/// Elements that make their parent a container of blocks rather than a paragraph.
const BLOCK_ELEMENTS: &[&str] = &[
    "t",
    "dl",
    "dt",
    "dd",
    "ul",
    "ol",
    "li",
    "artwork",
    "artset",
    "sourcecode",
    "figure",
    "table",
    "thead",
    "tbody",
    "tfoot",
    "tr",
    "td",
    "th",
    "aside",
    "blockquote",
    "section",
    "name",
];

/// Elements that give no block and no text.
const SKIPPED: &[&str] = &[
    "name",
    "toc",
    "reference",
    "referencegroup",
    "iref",
    "cref",
    "author",
    "displayreference",
    "seriesInfo",
    "title",
];

fn name<'a>(node: Node<'a, '_>) -> &'a str {
    node.tag_name().name()
}

fn elements<'a, 'input>(node: Node<'a, 'input>) -> impl Iterator<Item = Node<'a, 'input>> {
    node.children().filter(Node::is_element)
}

/// The section a block lies in.
#[derive(Clone, Default)]
struct Context {
    number: Option<String>,
    anchor: Option<String>,
}

/// Text placed on the source lines of one element, with its `<bcp14>` marks as
/// (line, byte offset in the line, keyword), and the starts of its `<dl>` entries after the first
/// as (line, byte offset in the line).
struct Rendered {
    lines: Vec<String>,
    marks: Vec<(usize, usize, String)>,
    entries: Vec<(usize, usize)>,
}

impl Rendered {
    fn index(&self, line: usize) -> usize {
        line.min(self.lines.len() - 1)
    }

    fn push(&mut self, line: usize, text: &str) {
        let index = self.index(line);
        self.lines[index].push_str(text);
    }

    fn mark(&mut self, line: usize, keyword: String) {
        let index = self.index(line);
        self.marks.push((index, self.lines[index].len(), keyword));
    }

    /// Trim every line at the end, and at the start too unless `verbatim`.
    fn trim(&mut self, verbatim: bool) {
        for (index, line) in self.lines.iter_mut().enumerate() {
            line.truncate(line.trim_end().len());
            if verbatim {
                continue;
            }
            let lead = line.len() - line.trim_start().len();
            line.drain(..lead);
            for mark in self.marks.iter_mut().filter(|mark| mark.0 == index) {
                mark.1 = mark.1.saturating_sub(lead);
            }
        }
    }

    fn is_empty(&self) -> bool {
        self.lines.iter().all(String::is_empty)
    }

    /// The marks as offsets in the text the lines join to ([`Block::text`] of a paragraph).
    fn bcp14(&self) -> Vec<Bcp14Mark> {
        let starts = self.starts();
        self.marks
            .iter()
            .map(|(line, offset, keyword)| Bcp14Mark {
                keyword: keyword.clone(),
                offset: starts[*line] + offset,
            })
            .collect()
    }

    /// The entry starts as offsets in the text the lines join to.
    fn entry_offsets(&self) -> Vec<usize> {
        let starts = self.starts();
        self.entries
            .iter()
            .map(|(line, offset)| starts[*line] + offset)
            .collect()
    }

    /// The offset at which each line starts in the text the lines join to.
    fn starts(&self) -> Vec<usize> {
        let mut starts = Vec::with_capacity(self.lines.len());
        let mut position = 0;
        let mut any = false;
        for line in &self.lines {
            if !line.is_empty() && any {
                position += 1;
            }
            starts.push(position);
            if !line.is_empty() {
                position += line.len();
                any = true;
            }
        }
        starts
    }

    fn joined(&self) -> String {
        self.lines
            .iter()
            .filter(|line| !line.is_empty())
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(" ")
    }
}

struct Reader<'input> {
    /// The byte offset at which each source line starts.
    line_starts: Vec<usize>,
    /// The anchors of the cited documents.
    references: BTreeSet<&'input str>,
}

impl Reader<'_> {
    /// The 1-based source line holding a byte.
    fn line(&self, byte: usize) -> usize {
        self.line_starts.partition_point(|&start| start <= byte)
    }

    fn span(&self, node: Node) -> Span {
        let range = node.range();
        Span {
            start: self.line(range.start),
            end: self.line(range.end.saturating_sub(1).max(range.start)),
        }
    }

    /// The blocks of a section-like element: its heading, then its content.
    fn section(&self, node: Node, out: &mut Vec<Block>) {
        let pn = node.attribute("pn");
        let number = (node.attribute("numbered") != Some("false"))
            .then(|| pn.and_then(number_from_pn))
            .flatten();
        let context = Context {
            anchor: node.attribute("anchor").or(pn).map(String::from),
            number: number.clone(),
        };
        let prefix = match &number {
            Some(number) if pn.is_some_and(|pn| pn.starts_with("section-appendix.")) => {
                format!("Appendix {number}.  ")
            }
            Some(number) => format!("{number}.  "),
            None => String::new(),
        };
        if let Some(title) = elements(node).find(|child| name(*child) == "name") {
            let mut rendered = self.render(title, false);
            let first = rendered
                .lines
                .iter()
                .position(|line| !line.is_empty())
                .unwrap_or(0);
            rendered.lines[first].insert_str(0, &prefix);
            let trimmed = rendered.lines[first].trim_end().len();
            rendered.lines[first].truncate(trimmed);
            for mark in rendered.marks.iter_mut().filter(|mark| mark.0 == first) {
                mark.1 += prefix.len();
            }
            out.push(self.block(BlockKind::Heading, title, &context, &rendered, None));
        } else if name(node) == "abstract" {
            let line = self.span(node).start;
            out.push(Block {
                kind: BlockKind::Heading,
                span: Span {
                    start: line,
                    end: line,
                },
                lines: vec!["Abstract".into()],
                section: context.number.clone(),
                anchor: context.anchor.clone(),
                figure_type: None,
                bcp14: Vec::new(),
                entries: Vec::new(),
            });
        }
        self.container(node, &context, out);
    }

    /// The blocks of the children of an element.
    fn container(&self, node: Node, context: &Context, out: &mut Vec<Block>) {
        for child in elements(node) {
            match name(child) {
                "section" | "references" | "abstract" | "note" => self.section(child, out),
                skipped if SKIPPED.contains(&skipped) => {}
                "t" => self.paragraph(child, context, out),
                "dl" => self.definitions(child, context, out),
                "artwork" | "sourcecode" => self.figure(child, context, out),
                "artset" => {
                    for artwork in elements(child).filter(|artwork| {
                        name(*artwork) == "artwork"
                            && artwork.attribute("type") != Some("svg")
                            && elements(*artwork).next().is_none()
                    }) {
                        self.figure(artwork, context, out);
                    }
                }
                _ if elements(child).any(|grand| BLOCK_ELEMENTS.contains(&name(grand))) => {
                    self.container(child, context, out)
                }
                _ => self.paragraph(child, context, out),
            }
        }
    }

    /// A `<sourcecode>` or `<artwork>` as a figure: its lines verbatim and its `type`; an artwork
    /// holding elements (an SVG drawing) has no text lines.
    fn figure(&self, node: Node, context: &Context, out: &mut Vec<Block>) {
        let mut rendered = self.render(node, true);
        if elements(node).next().is_some() {
            rendered.lines.iter_mut().for_each(String::clear);
            rendered.marks.clear();
        }
        let figure_type = node.attribute("type").map(String::from);
        out.push(self.block(BlockKind::Figure, node, context, &rendered, figure_type));
    }

    fn paragraph(&self, node: Node, context: &Context, out: &mut Vec<Block>) {
        let rendered = self.render(node, false);
        if !rendered.is_empty() {
            out.push(self.block(BlockKind::Paragraph, node, context, &rendered, None));
        }
    }

    /// A `<dl>` as one paragraph: each term and its definition on the line the term's text
    /// starts on, every entry after the first recorded as a sentence boundary.
    fn definitions(&self, node: Node, context: &Context, out: &mut Vec<Block>) {
        let span = self.span(node);
        let mut rendered = Rendered {
            lines: vec![String::new(); span.end - span.start + 1],
            marks: Vec::new(),
            entries: Vec::new(),
        };
        let mut children = elements(node).peekable();
        while let Some(term) = children.next() {
            if name(term) != "dt" {
                continue;
            }
            let (mut text, mut marks, line) = self.flat(term);
            if let Some(definition) = children.next_if(|child| name(*child) == "dd") {
                let (more, more_marks, _) = self.flat(definition);
                if !more.is_empty() {
                    let shift = if text.is_empty() {
                        0
                    } else {
                        text.push(' ');
                        text.len()
                    };
                    text.push_str(&more);
                    marks.extend(
                        more_marks
                            .into_iter()
                            .map(|(offset, keyword)| (offset + shift, keyword)),
                    );
                }
            }
            if text.is_empty() {
                continue;
            }
            let index = rendered.index(line.saturating_sub(span.start));
            if !rendered.lines[index].is_empty() {
                rendered.lines[index].push(' ');
            }
            let shift = rendered.lines[index].len();
            if !rendered.is_empty() {
                rendered.entries.push((index, shift));
            }
            rendered.lines[index].push_str(&text);
            rendered.marks.extend(
                marks
                    .into_iter()
                    .map(|(offset, keyword)| (index, offset + shift, keyword)),
            );
        }
        if !rendered.is_empty() {
            out.push(self.block(BlockKind::Paragraph, node, context, &rendered, None));
        }
    }

    /// An element's text on one line, its marks as offsets in it, and the source line its text
    /// starts on.
    fn flat(&self, node: Node) -> (String, Vec<(usize, String)>, usize) {
        let rendered = self.render(node, false);
        let first = rendered
            .lines
            .iter()
            .position(|line| !line.is_empty())
            .unwrap_or(0);
        let marks = rendered
            .bcp14()
            .into_iter()
            .map(|mark| (mark.offset, mark.keyword))
            .collect();
        (rendered.joined(), marks, self.span(node).start + first)
    }

    fn block(
        &self,
        kind: BlockKind,
        node: Node,
        context: &Context,
        rendered: &Rendered,
        figure_type: Option<String>,
    ) -> Block {
        Block {
            kind,
            span: self.span(node),
            lines: rendered.lines.clone(),
            section: context.number.clone(),
            anchor: context.anchor.clone(),
            figure_type,
            bcp14: if kind == BlockKind::Figure {
                Vec::new()
            } else {
                rendered.bcp14()
            },
            entries: rendered.entry_offsets(),
        }
    }

    /// The inline content of an element placed on its source lines.
    fn render(&self, node: Node, verbatim: bool) -> Rendered {
        let span = self.span(node);
        let mut rendered = Rendered {
            lines: vec![String::new(); span.end - span.start + 1],
            marks: Vec::new(),
            entries: Vec::new(),
        };
        self.inline(node, span.start, &mut rendered);
        rendered.trim(verbatim);
        rendered
    }

    fn inline(&self, node: Node, first: usize, rendered: &mut Rendered) {
        for child in node.children() {
            let line = self.line(child.range().start).saturating_sub(first);
            if child.is_text() {
                for (offset, piece) in child.text().unwrap_or_default().split('\n').enumerate() {
                    rendered.push(line + offset, piece);
                }
                continue;
            }
            if !child.is_element() {
                continue;
            }
            match name(child) {
                "bcp14" => {
                    rendered.mark(line, collapsed_text(child));
                    self.inline(child, first, rendered);
                }
                "xref" | "relref" => rendered.push(line, &self.xref(child)),
                "eref" => rendered.push(line, &eref(child)),
                "contact" => rendered.push(
                    line,
                    child
                        .attribute("fullname")
                        .or_else(|| child.attribute("asciiFullname"))
                        .unwrap_or_default(),
                ),
                "br" => rendered.push(line, " "),
                "iref" | "cref" => {}
                _ => self.inline(child, first, rendered),
            }
        }
    }

    /// An `<xref>` as xml2rfc renders it in text.
    fn xref(&self, node: Node) -> String {
        let content = collapsed_text(node);
        if !content.is_empty() {
            return content;
        }
        let target = node.attribute("target").unwrap_or_default();
        let derived = node.attribute("derivedContent").unwrap_or(target);
        let cited = if self.references.contains(target) {
            format!("[{derived}]")
        } else {
            derived.to_string()
        };
        if let Some(section) = node.attribute("section") {
            let label = if section.starts_with(|c: char| c.is_ascii_alphabetic()) {
                format!("Appendix {section}")
            } else {
                format!("Section {section}")
            };
            return match node.attribute("sectionFormat").unwrap_or("of") {
                "comma" => format!("{cited}, {label}"),
                "parens" => format!("{cited} ({label})"),
                "bare" => section.to_string(),
                _ => format!("{label} of {cited}"),
            };
        }
        match node.attribute("format").unwrap_or("default") {
            "none" => String::new(),
            "default" => cited,
            _ => derived.to_string(),
        }
    }
}

/// An `<eref>` as xml2rfc renders it in text: its content, else its target.
fn eref(node: Node) -> String {
    let content = collapsed_text(node);
    if !content.is_empty() {
        return content;
    }
    let target = node.attribute("target").unwrap_or_default();
    match node.attribute("brackets") {
        Some("angle") => format!("<{target}>"),
        _ => target.to_string(),
    }
}

/// The text of an element's descendants, whitespace collapsed.
fn collapsed_text(node: Node) -> String {
    node.descendants()
        .filter(Node::is_text)
        .filter_map(|text| text.text())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

/// The section number in a `pn`: `section-3.1` is `3.1`, `section-appendix.a.2` is `A.2`;
/// `section-abstract`, `section-boilerplate.1` and `section-toc.1` have none.
fn number_from_pn(pn: &str) -> Option<String> {
    let rest = pn.strip_prefix("section-")?;
    if let Some(appendix) = rest.strip_prefix("appendix.") {
        return Some(appendix.to_uppercase());
    }
    rest.split('.')
        .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
        .then(|| rest.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_definition_list_is_one_paragraph_whose_later_entries_start_a_sentence() {
        let text = "<rfc version=\"3\">\n<middle>\n<section pn=\"section-1\">\n<name>Intro</name>\n<dl>\n<dt>first:</dt>\n<dd>One <bcp14>MUST</bcp14> go.</dd>\n<dt>second:</dt> <dd>two</dd> <dt>third:</dt>\n<dd>three</dd>\n</dl>\n</section>\n</middle>\n</rfc>\n";
        let document = read(1, "digest", text).unwrap();
        let dl = &document.blocks[1];
        assert_eq!(dl.kind, BlockKind::Paragraph);
        assert_eq!(
            dl.lines,
            [
                "",
                "first: One MUST go.",
                "",
                "second: two third: three",
                "",
                ""
            ]
        );
        let joined = dl.text();
        let starts: Vec<&str> = dl.entries.iter().map(|&at| &joined[at..]).collect();
        assert_eq!(starts, ["second: two third: three", "third: three"]);
        assert!(
            read(
                1,
                "digest",
                &text.replace("<dl>", "<t>").replace("</dl>", "</t>")
            )
            .unwrap()
            .blocks
            .iter()
            .all(|block| block.entries.is_empty())
        );
    }

    #[test]
    fn numbers_come_from_pn() {
        assert_eq!(number_from_pn("section-3").as_deref(), Some("3"));
        assert_eq!(number_from_pn("section-5.2.1").as_deref(), Some("5.2.1"));
        assert_eq!(number_from_pn("section-appendix.a").as_deref(), Some("A"));
        assert_eq!(number_from_pn("section-abstract"), None);
        assert_eq!(number_from_pn("section-boilerplate.1"), None);
        assert_eq!(number_from_pn("section-3-1"), None);
    }
}
