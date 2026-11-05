#![forbid(unsafe_code)]
//! S1: the plain-text reader. It removes page furniture (form feeds, `... [Page N]` footers and the
//! `RFC N ... date` header block after each page break) and splits the rest into headings,
//! paragraphs and indented figures, each with the 1-based source lines it came from and the
//! numbered section it lies in.
//!
//! Rules, in order, over the groups of consecutive non-blank lines that remain:
//!
//! - a group continues across a page break when its last line is indented, has no closing
//!   punctuation (`.`, `?`, `!`, `:`), and the first line after the break has the same indentation
//!   and is no numbered heading; the furniture and blank lines between become empty lines of the
//!   block;
//! - an indented group whose first line defines an ABNF rule (`name = ...`) or that draws a
//!   diagram (a border line starting with `+-` or `+ -`) is a figure;
//! - before the first section heading, an indented multi-line group that starts with a
//!   table-of-contents entry (`5.2.  Title`, `A.1.  Title`, `Appendix A.  Title`), holds at least
//!   two, and has no line ending in closing punctuation is a table of contents, read as a figure;
//! - a one-line list item (a line starting with a mark `*`, `-`, `o` or `+` and two spaces) is a
//!   paragraph;
//! - a one-line group without closing punctuation is a heading;
//! - a run of one-line groups indented deeper than the body text, each one blank line from the
//!   next, is one figure (the dialogue lines and example lists of older RFCs); the body text's
//!   indentation is the most common one among the lines of multi-line groups;
//! - every other group is a paragraph, split into one paragraph per list item and one for the
//!   text before and after a list (not in a group opening or closing a C comment, whose `*`
//!   lines continue it); a group of `|` lines is an aside, read without its bars, a line of only
//!   `|` ending a paragraph.
//!
//! A section heading is an unindented line `N.  Title`, `N.M.  Title`, ... (a numbered heading),
//! or `Appendix A.  Title`, `A.1.  Title`, ... (an appendix heading); every block from it to the
//! next one carries its number.

pub mod xml;

use rfcspec_model::{Block, BlockKind, Document, SourceFormat, Span};

/// Read a locked source into a normalized document by the reader of its format: [`read`] for
/// plain text, [`xml::read`] for RFCXML v3.
pub fn read_format(
    rfc: u32,
    format: SourceFormat,
    sha256: &str,
    text: &str,
) -> Result<Document, xml::XmlError> {
    match format {
        SourceFormat::Text => Ok(read(rfc, sha256, text)),
        SourceFormat::Xml => xml::read(rfc, sha256, text),
    }
}

/// Read an RFC's plain text into a normalized document.
pub fn read(rfc: u32, sha256: &str, text: &str) -> Document {
    let lines = split(text);
    let furniture = furniture(&lines);
    let mut is_furniture = vec![false; lines.len()];
    for line in &furniture {
        is_furniture[line - 1] = true;
    }
    let blank = |index: usize| is_furniture[index] || lines[index].trim().is_empty();

    let mut groups: Vec<(usize, usize)> = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        if blank(index) {
            index += 1;
            continue;
        }
        let start = index;
        while index < lines.len() && !blank(index) {
            index += 1;
        }
        let end = index - 1;
        match groups.last_mut() {
            Some(last) if continues(&lines, &is_furniture, *last, start) => last.1 = end,
            _ => groups.push((start, end)),
        }
    }

    let indented = |(start, end): (usize, usize)| {
        lines[start..=end]
            .iter()
            .enumerate()
            .all(|(offset, line)| blank(start + offset) || line.starts_with([' ', '\t']))
    };
    let heading = |(start, end): (usize, usize)| {
        start == end
            && !lines[start]
                .trim_end()
                .ends_with(['.', ',', ':', ';', '?', '!'])
    };
    let indent = |line: &str| line.len() - line.trim_start().len();
    let mut counts = std::collections::BTreeMap::<usize, usize>::new();
    for &(start, end) in groups.iter().filter(|(start, end)| end > start) {
        for index in (start..=end).filter(|&index| !blank(index)) {
            *counts.entry(indent(lines[index])).or_default() += 1;
        }
    }
    let body = counts
        .iter()
        .max_by(|a, b| a.1.cmp(b.1).then(b.0.cmp(a.0)))
        .map_or(0, |(indent, _)| *indent);
    let figure_line = |group: (usize, usize)| {
        group.0 == group.1 && indent(lines[group.0]) > body && !heading(group)
    };
    let drawn = |(start, end): (usize, usize)| {
        indented((start, end))
            && (abnf_rule(lines[start])
                || lines[start..=end].iter().any(|line| {
                    let line = line.trim();
                    line.starts_with("+-") || line.starts_with("+ -")
                }))
    };

    let sections = sections(&lines, &is_furniture);
    let contents = |(start, end): (usize, usize)| {
        let first = lines[start];
        start < end
            && indented((start, end))
            && sections[start].is_none()
            && (start..=end)
                .filter(|&index| !blank(index) && toc_entry(lines[index]))
                .take(2)
                .count()
                == 2
            && (start..=end).filter(|&index| !blank(index)).all(|index| {
                !lines[index]
                    .trim_end()
                    .ends_with(['.', ',', ':', ';', '?', '!'])
            })
            && toc_entry(first)
    };
    let mut blocks = Vec::new();
    let mut push = |kind: BlockKind, start: usize, end: usize, aside: bool| {
        let block_lines = (start..=end)
            .map(|index| match kind {
                _ if is_furniture[index] => String::new(),
                BlockKind::Figure => lines[index].trim_end().to_string(),
                BlockKind::Paragraph if aside => aside_text(lines[index]).to_string(),
                BlockKind::Heading | BlockKind::Paragraph => lines[index].trim().to_string(),
            })
            .collect();
        blocks.push(Block {
            kind,
            span: Span {
                start: start + 1,
                end: end + 1,
            },
            lines: block_lines,
            section: sections[start].clone(),
            anchor: None,
            figure_type: None,
            bcp14: Vec::new(),
            entries: Vec::new(),
        });
    };
    let mut next = 0;
    while next < groups.len() {
        let group = groups[next];
        next += 1;
        let (kind, start, end) = if drawn(group) || contents(group) {
            (BlockKind::Figure, group.0, group.1)
        } else if group.0 == group.1 && bullet(lines[group.0]) {
            (BlockKind::Paragraph, group.0, group.1)
        } else if heading(group) {
            (BlockKind::Heading, group.0, group.1)
        } else if figure_line(group) {
            let mut end = group.1;
            while next < groups.len() && figure_line(groups[next]) && groups[next].0 == end + 2 {
                end = groups[next].1;
                next += 1;
            }
            (BlockKind::Figure, group.0, end)
        } else {
            (BlockKind::Paragraph, group.0, group.1)
        };
        if kind != BlockKind::Paragraph {
            push(kind, start, end, false);
            continue;
        }
        let aside = (start..=end)
            .filter(|&index| !blank(index))
            .all(|index| aside_line(lines[index]));
        for (start, end) in paragraphs(&lines, &is_furniture, start, end, aside) {
            push(BlockKind::Paragraph, start, end, aside);
        }
    }

    Document {
        rfc,
        sha256: sha256.to_string(),
        format: SourceFormat::Text,
        furniture,
        blocks,
        references: Vec::new(),
        reference_counts: None,
    }
}

/// The source lines of a text, without the final newline.
fn split(text: &str) -> Vec<&str> {
    text.strip_suffix('\n')
        .unwrap_or(text)
        .split('\n')
        .collect()
}

/// The 1-based numbers of the page-furniture lines of a text: what [`read`] removes, and what a
/// quote spanning a page break skips.
pub fn furniture_lines(text: &str) -> Vec<usize> {
    furniture(&split(text))
}

/// Whether the group `last` continues, across a page break, in the group starting at `start`.
fn continues(lines: &[&str], is_furniture: &[bool], last: (usize, usize), start: usize) -> bool {
    let tail = lines[last.1];
    let head = lines[start];
    let indent = |line: &str| line.len() - line.trim_start().len();
    (last.1 + 1..start).any(|index| is_furniture[index])
        && indent(tail) > 0
        && indent(tail) == indent(head)
        && !tail.trim_end().ends_with(['.', '?', '!', ':'])
        && numbered_heading(head).is_none()
}

/// The number of a numbered heading, `5.2` for `5.2.  Base Framing Protocol`.
pub fn numbered_heading(line: &str) -> Option<&str> {
    if line.starts_with([' ', '\t']) {
        return None;
    }
    let (number, title) = line.split_once(". ")?;
    let numbered = !number.is_empty()
        && number
            .split('.')
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()));
    (numbered && !title.trim().is_empty()).then_some(number)
}

/// The number of an appendix heading, an unindented `Appendix A.  Title` (`A`) or `A.1.  Title`
/// (`A.1`): one capital letter, then, without the `Appendix` prefix, at least one numeric part.
pub fn appendix_heading(line: &str) -> Option<&str> {
    if line.starts_with([' ', '\t']) {
        return None;
    }
    let (prefixed, rest) = match line.strip_prefix("Appendix ") {
        Some(rest) => (true, rest),
        None => (false, line),
    };
    let (number, title) = rest.split_once(". ")?;
    let mut parts = number.split('.');
    let letter = parts.next()?;
    let mut digits = 0;
    for part in parts {
        if part.is_empty() || !part.bytes().all(|b| b.is_ascii_digit()) {
            return None;
        }
        digits += 1;
    }
    let numbered = letter.len() == 1
        && letter.bytes().all(|b| b.is_ascii_uppercase())
        && (prefixed || digits > 0);
    (numbered && !title.trim().is_empty()).then_some(number)
}

/// The number of a section heading: a [`numbered_heading`] or an [`appendix_heading`].
pub fn section_heading(line: &str) -> Option<&str> {
    numbered_heading(line).or_else(|| appendix_heading(line))
}

/// Whether a line is an indented table-of-contents entry: `5.2.  Title`, `A.1.  Title` or
/// `Appendix A.  Title`, the title not ending a sentence.
fn toc_entry(line: &str) -> bool {
    if !line.starts_with([' ', '\t']) {
        return false;
    }
    let line = line.trim();
    let line = line.strip_prefix("Appendix ").unwrap_or(line);
    let Some((number, title)) = line.split_once(char::is_whitespace) else {
        return false;
    };
    let Some(number) = number.strip_suffix('.') else {
        return false;
    };
    let mut parts = number.split('.');
    let first = parts.next().unwrap_or_default();
    let first_ok = (!first.is_empty() && first.bytes().all(|b| b.is_ascii_digit()))
        || (first.len() == 1 && first.bytes().all(|b| b.is_ascii_uppercase()));
    first_ok
        && parts.all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
        && !title.trim().is_empty()
}

/// Whether a line belongs to an aside as plain text renders it: `|` alone, or `|` then text that
/// does not end in `|` as a table row does.
fn aside_line(line: &str) -> bool {
    let line = line.trim();
    line == "|" || (line.starts_with("| ") && !line.ends_with('|'))
}

/// An aside line's text without its `|` mark.
fn aside_text(line: &str) -> &str {
    let line = line.trim();
    line.strip_prefix('|').unwrap_or(line).trim()
}

/// Whether a line starts a list item as RFCXML renders one: a mark (`*`, then `-`, `o` and `+`
/// for nested lists), two spaces, then text.
fn bullet(line: &str) -> bool {
    ["*  ", "-  ", "o  ", "+  "].iter().any(|mark| {
        line.trim_start()
            .strip_prefix(mark)
            .is_some_and(|text| !text.is_empty() && !text.starts_with([' ', '\t']))
    })
}

/// Whether a line opens or closes a C comment, which makes its group code, whose `*` lines
/// continue the comment rather than start list items.
fn comment_mark(line: &str) -> bool {
    let line = line.trim();
    line.starts_with("/*") || line.starts_with("*/") || line.ends_with("*/")
}

/// Whether a line ends a sentence or clause: its last character, after closing quotes and
/// brackets, is `.`, `!`, `?`, `:` or `;`.
fn sentence_end(line: &str) -> bool {
    line.trim_end()
        .trim_end_matches(['"', '\'', ')', ']'])
        .ends_with(['.', '!', '?', ':', ';'])
}

/// The paragraphs of one paragraph group, lines `start..=end`: each list item (a [`bullet`]
/// line, with the lines after it that are indented deeper than its mark, or that sit at or left
/// of its mark while the item's last line does not end a sentence, as a wrapped line does in
/// RFC 3995 or RFC 2639) is its own paragraph,
/// and so is the text before and after a list, unless the group is code (a [`comment_mark`]
/// line); in an aside a line holding only `|` ends a paragraph. Leading and trailing furniture
/// and blank lines are left out of each.
fn paragraphs(
    lines: &[&str],
    is_furniture: &[bool],
    start: usize,
    end: usize,
    aside: bool,
) -> Vec<(usize, usize)> {
    let skip = |index: usize| is_furniture[index] || lines[index].trim().is_empty();
    let indent = |line: &str| line.len() - line.trim_start().len();
    let mut found = Vec::new();
    let mut piece: Option<(usize, usize)> = None;
    let mut item: Option<usize> = None;
    let mut close = |piece: &mut Option<(usize, usize)>| {
        if let Some(done) = piece.take() {
            found.push(done);
        }
    };
    let code = (start..=end).any(|index| !skip(index) && comment_mark(lines[index]));
    for (index, &line) in lines.iter().enumerate().take(end + 1).skip(start) {
        if skip(index) {
            continue;
        }
        if aside {
            if aside_text(line).is_empty() {
                close(&mut piece);
            } else {
                piece = Some((piece.map_or(index, |p| p.0), index));
            }
            continue;
        }
        let breaks = if code {
            false
        } else if bullet(line) {
            item = Some(indent(line));
            true
        } else if item.is_some_and(|mark| indent(line) <= mark)
            && piece.is_none_or(|(_, last)| sentence_end(lines[last]))
        {
            item = None;
            true
        } else {
            false
        };
        if breaks {
            close(&mut piece);
        }
        piece = Some((piece.map_or(index, |p| p.0), index));
    }
    close(&mut piece);
    found
}

/// The section of every source line: the number of the last section heading at or before it.
fn sections(lines: &[&str], is_furniture: &[bool]) -> Vec<Option<String>> {
    let mut current: Option<String> = None;
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if !is_furniture[index]
                && let Some(number) = section_heading(line)
            {
                current = Some(number.to_string());
            }
            current.clone()
        })
        .collect()
}

/// Whether a line starts an ABNF rule definition: `name = ...` or `name= ...`.
pub fn abnf_rule(line: &str) -> bool {
    let line = line.trim_start();
    let name_end = line
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '-'))
        .unwrap_or(line.len());
    let rest = line[name_end..].trim_start();
    name_end > 0
        && line.starts_with(|c: char| c.is_ascii_alphabetic())
        && rest.starts_with('=')
        && !rest.starts_with("==")
}

/// The 1-based numbers of the page-furniture lines.
fn furniture(lines: &[&str]) -> Vec<usize> {
    let mut found = Vec::new();
    let mut index = 0;
    while index < lines.len() {
        let line = lines[index];
        if is_footer(line) {
            found.push(index + 1);
        }
        if line.contains('\u{c}') {
            found.push(index + 1);
            let mut header = index + 1;
            while header < lines.len() && lines[header].trim().is_empty() {
                header += 1;
            }
            if header < lines.len() && is_header(lines[header]) {
                while header < lines.len() && !lines[header].trim().is_empty() {
                    found.push(header + 1);
                    header += 1;
                }
                index = header;
                continue;
            }
        }
        index += 1;
    }
    found
}

fn is_footer(line: &str) -> bool {
    let line = line.trim_end();
    let Some(open) = line.rfind("[Page ") else {
        return false;
    };
    let number = &line[open + "[Page ".len()..];
    !line.starts_with([' ', '\t'])
        && number
            .strip_suffix(']')
            .is_some_and(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

fn is_header(line: &str) -> bool {
    line.strip_prefix("RFC ")
        .is_some_and(|rest| rest.starts_with(|c: char| c.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    fn rfc868() -> Document {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../profiles/time/sources/rfc868.txt");
        read(868, "digest", &fs::read_to_string(path).unwrap())
    }

    #[test]
    fn removes_both_footers_and_the_page_two_header() {
        let document = rfc868();
        for block in &document.blocks {
            let text = block.text();
            assert!(!text.contains("[Page"), "{text}");
            assert!(!text.starts_with("RFC 868"), "{text}");
            for line in [56, 60, 61, 114] {
                assert!(
                    !block.span.contains(line),
                    "line {line} in {:?}",
                    block.span
                );
            }
        }
        for line in [56, 60, 61, 114] {
            assert!(
                document.furniture.contains(&line),
                "{:?}",
                document.furniture
            );
        }
    }

    #[test]
    fn every_block_keeps_its_source_line_range() {
        let document = rfc868();
        let source = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/time/sources/rfc868.txt"),
        )
        .unwrap();
        let source: Vec<&str> = source.lines().collect();
        for block in &document.blocks {
            assert_eq!(block.lines.len(), block.span.end - block.span.start + 1);
            for (offset, line) in block.lines.iter().enumerate() {
                assert_eq!(line.trim(), source[block.span.start - 1 + offset].trim());
            }
        }
        let paragraph = document
            .blocks
            .iter()
            .find(|block| {
                block
                    .text()
                    .starts_with("The server listens for a connection")
            })
            .unwrap();
        assert_eq!(paragraph.kind, BlockKind::Paragraph);
        assert_eq!(paragraph.span, Span { start: 44, end: 48 });
    }

    #[test]
    fn the_tcp_dialogue_block_is_one_figure() {
        let document = rfc868();
        let figure = document
            .blocks
            .iter()
            .find(|block| block.lines[0].trim() == "S: Listen on port 37 (45 octal).")
            .unwrap();
        assert_eq!(figure.kind, BlockKind::Figure);
        assert_eq!(figure.span, Span { start: 32, end: 42 });
        assert_eq!(
            figure.lines.last().unwrap().trim(),
            "S: Close the connection."
        );
        let udp = document
            .blocks
            .iter()
            .find(|block| block.span.start == 66)
            .unwrap();
        assert_eq!((udp.kind, udp.span.end), (BlockKind::Figure, 74));
        let examples = document
            .blocks
            .iter()
            .find(|block| block.span.start == 89)
            .unwrap();
        assert_eq!((examples.kind, examples.span.end), (BlockKind::Figure, 97));
    }

    #[test]
    fn headings_are_one_line_groups_without_closing_punctuation() {
        let document = rfc868();
        let headings: Vec<String> = document
            .blocks
            .iter()
            .filter(|block| block.kind == BlockKind::Heading)
            .map(Block::text)
            .collect();
        assert_eq!(headings, ["Time Protocol", "The Time"]);
        assert!(rfc868().blocks.iter().all(|block| block.section.is_none()));
    }

    fn rfc6455_text() -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/websocket/sources/rfc6455.txt"),
        )
        .unwrap()
    }

    fn rfc6455() -> Document {
        read(6455, "digest", &rfc6455_text())
    }

    fn block_at(document: &Document, line: usize) -> &Block {
        document
            .blocks
            .iter()
            .find(|block| block.span.contains(line))
            .unwrap_or_else(|| panic!("no block holds line {line}"))
    }

    #[test]
    fn rfc6455_furniture_is_71_footers_71_form_feeds_and_70_headers() {
        let text = rfc6455_text();
        let lines: Vec<&str> = text.lines().collect();
        let document = rfc6455();
        let of = |matches: &dyn Fn(&str) -> bool| {
            document
                .furniture
                .iter()
                .filter(|&&line| matches(lines[line - 1]))
                .count()
        };
        assert_eq!(
            of(&|l| l.trim_end().ends_with(']') && l.contains("[Page ")),
            71
        );
        assert_eq!(of(&|l| l.contains('\u{c}')), 71);
        assert_eq!(
            of(&|l| l.starts_with("RFC 6455 ") && l.trim_end().ends_with("December 2011")),
            70
        );
        assert_eq!(document.furniture.len(), 212);
        let pages: Vec<usize> = document
            .furniture
            .iter()
            .filter_map(|&line| {
                let line = lines[line - 1].trim_end();
                line.rsplit_once("[Page ")
                    .and_then(|(_, n)| n.strip_suffix(']')?.parse().ok())
            })
            .collect();
        assert_eq!(pages, (1..=71).collect::<Vec<_>>());
        for block in &document.blocks {
            assert!(!block.text().contains("[Page"), "{:?}", block.span);
            assert!(!block.text().starts_with("RFC 6455 "), "{:?}", block.span);
            assert!(
                !block.lines.iter().any(|line| line.contains('\u{c}')),
                "{:?}",
                block.span
            );
        }
    }

    #[test]
    fn a_paragraph_without_closing_punctuation_continues_after_the_page_break() {
        let document = rfc6455();
        let block = block_at(&document, 2572);
        assert_eq!(block.kind, BlockKind::Paragraph);
        assert_eq!(
            block.span,
            Span {
                start: 2569,
                end: 2585
            }
        );
        assert_eq!(block.lines.len(), 2585 - 2569 + 1);
        assert!(block.text().contains(
            "The list of extensions that are needed SHOULD appear in the /reason/ part of the Close frame."
        ));
        assert!(document.furniture.contains(&2578));
    }

    #[test]
    fn rfc6455_has_88_numbered_headings_14_top_level_and_every_block_carries_its_section() {
        let text = rfc6455_text();
        let document = rfc6455();
        let headings: Vec<(usize, &str)> = text
            .lines()
            .enumerate()
            .filter_map(|(index, line)| numbered_heading(line).map(|n| (index + 1, n)))
            .collect();
        assert_eq!(headings.len(), 88);
        assert_eq!(
            headings.iter().filter(|(_, n)| !n.contains('.')).count(),
            14
        );
        assert_eq!(headings[0], (175, "1"));
        assert_eq!(headings[87], (3856, "14.2"));
        assert!(
            text.lines()
                .nth(174)
                .unwrap()
                .starts_with("1.  Introduction")
        );
        assert!(
            text.lines()
                .nth(3855)
                .unwrap()
                .starts_with("14.2.  Informative References")
        );
        for block in &document.blocks {
            let expected = headings
                .iter()
                .rev()
                .find(|(line, _)| *line <= block.span.start)
                .map(|(_, n)| n.to_string());
            assert_eq!(block.section, expected, "{:?}", block.span);
            if block.span.start < 175 {
                assert_eq!(block.section, None);
            }
        }
        assert_eq!(block_at(&document, 1556).section.as_deref(), Some("5.2"));
        assert_eq!(block_at(&document, 2572).section.as_deref(), Some("7.4.1"));
    }

    #[test]
    fn the_packet_diagram_and_the_bit_abnf_are_figures_and_field_text_stays_a_paragraph() {
        let document = rfc6455();
        let diagram = block_at(&document, 1535);
        assert_eq!(
            (diagram.kind, diagram.span),
            (
                BlockKind::Figure,
                Span {
                    start: 1535,
                    end: 1552
                }
            )
        );
        for line in (1687..=1783).filter(|line| {
            !document.furniture.contains(line)
                && !rfc6455_text()
                    .lines()
                    .nth(line - 1)
                    .unwrap()
                    .trim()
                    .is_empty()
        }) {
            assert_eq!(
                block_at(&document, line).kind,
                BlockKind::Figure,
                "line {line}"
            );
        }
        assert_eq!(block_at(&document, 1687).span.start, 1687);
        let field = block_at(&document, 1561);
        assert_eq!(
            (field.kind, field.span),
            (
                BlockKind::Paragraph,
                Span {
                    start: 1561,
                    end: 1565
                }
            )
        );
    }

    fn rfc1035_text() -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/dns/sources/rfc1035.txt"),
        )
        .unwrap()
    }

    fn rfc1035() -> Document {
        read(1035, "digest", &rfc1035_text())
    }

    #[test]
    fn rfc1035_furniture_is_55_footers_55_form_feeds_and_54_headers() {
        let text = rfc1035_text();
        let lines: Vec<&str> = text.lines().collect();
        let document = rfc1035();
        let of = |matches: &dyn Fn(&str) -> bool| {
            document
                .furniture
                .iter()
                .filter(|&&line| matches(lines[line - 1]))
                .count()
        };
        assert_eq!(
            of(&|l| l.starts_with("Mockapetris ") && l.trim_end().ends_with(']')),
            55
        );
        assert_eq!(of(&|l| l.contains('\u{c}')), 55);
        assert_eq!(
            of(&|l| l.trim_end()
                == "RFC 1035        Domain Implementation and Specification    November 1987"),
            54
        );
        assert_eq!(document.furniture.len(), 164);
        let pages: Vec<usize> = document
            .furniture
            .iter()
            .filter_map(|&line| {
                let line = lines[line - 1].trim_end();
                line.rsplit_once("[Page ")
                    .and_then(|(_, n)| n.strip_suffix(']')?.parse().ok())
            })
            .collect();
        assert_eq!(pages, (1..=55).collect::<Vec<_>>());
        for block in &document.blocks {
            assert!(!block.text().contains("[Page"), "{:?}", block.span);
            assert!(!block.text().starts_with("RFC 1035 "), "{:?}", block.span);
        }
    }

    #[test]
    fn rfc1035_has_71_numbered_headings_9_top_level_and_every_block_carries_its_section() {
        let text = rfc1035_text();
        let document = rfc1035();
        let headings: Vec<(usize, &str)> = text
            .lines()
            .enumerate()
            .filter_map(|(index, line)| numbered_heading(line).map(|n| (index + 1, n)))
            .collect();
        assert_eq!(headings.len(), 71);
        assert_eq!(headings.iter().filter(|(_, n)| !n.contains('.')).count(), 9);
        assert_eq!(headings[0], (9, "1"));
        assert_eq!(headings[70], (2748, "9"));
        let line = |number: usize| text.lines().nth(number - 1).unwrap();
        assert!(line(9).starts_with("1. STATUS OF THIS MEMO"));
        assert!(line(2748).starts_with("9. REFERENCES and BIBLIOGRAPHY"));
        for (number, section, title) in [
            (1401, "4.1.1", "4.1.1. Header section format"),
            (1752, "4.2.1", "4.2.1. UDP usage"),
            (2526, "7.3", "7.3. Processing responses"),
        ] {
            assert!(headings.contains(&(number, section)), "{number}");
            assert_eq!(line(number).trim_end(), title);
        }
        for block in &document.blocks {
            let expected = headings
                .iter()
                .rev()
                .find(|(line, _)| *line <= block.span.start)
                .map(|(_, n)| n.to_string());
            assert_eq!(block.section, expected, "{:?}", block.span);
            if block.span.start < 9 {
                assert_eq!(block.section, None);
            }
        }
        assert!(document.blocks.iter().any(|block| block.span.start < 9));
    }

    #[test]
    fn the_rfc1035_header_diagram_is_one_figure_and_the_field_descriptions_are_paragraphs() {
        let document = rfc1035();
        let diagram = block_at(&document, 1405);
        assert_eq!(
            (diagram.kind, diagram.span, diagram.section.as_deref()),
            (
                BlockKind::Figure,
                Span {
                    start: 1405,
                    end: 1419
                },
                Some("4.1.1")
            )
        );
        for (start, end, first) in [
            (1423, 1426, "ID"),
            (1473, 1474, "Z               Reserved for future use."),
        ] {
            let field = block_at(&document, start);
            assert_eq!(
                (field.kind, field.span, field.section.as_deref()),
                (BlockKind::Paragraph, Span { start, end }, Some("4.1.1"))
            );
            assert!(field.lines[0].starts_with(first), "{:?}", field.lines);
        }
        assert!(
            block_at(&document, 1473)
                .text()
                .contains("Must be zero in all queries and responses.")
        );
    }

    fn rfc5905_text() -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/ntp/sources/rfc5905.txt"),
        )
        .unwrap()
    }

    fn rfc5905() -> Document {
        read(5905, "digest", &rfc5905_text())
    }

    #[test]
    fn rfc5905_furniture_is_110_footers_110_form_feeds_and_109_headers() {
        let text = rfc5905_text();
        let lines: Vec<&str> = text.lines().collect();
        let document = rfc5905();
        let of = |matches: &dyn Fn(&str) -> bool| {
            document
                .furniture
                .iter()
                .filter(|&&line| matches(lines[line - 1]))
                .count()
        };
        assert_eq!(
            of(&|l| l.starts_with("Mills, et al. ") && l.trim_end().ends_with(']')),
            110
        );
        assert_eq!(of(&|l| l.contains('\u{c}')), 110);
        assert_eq!(
            of(&|l| l.split_whitespace().collect::<Vec<_>>()
                == ["RFC", "5905", "NTPv4", "Specification", "June", "2010"]),
            109
        );
        assert_eq!(document.furniture.len(), 329);
        let pages: Vec<usize> = document
            .furniture
            .iter()
            .filter_map(|&line| {
                let line = lines[line - 1].trim_end();
                line.rsplit_once("[Page ")
                    .and_then(|(_, n)| n.strip_suffix(']')?.parse().ok())
            })
            .collect();
        assert_eq!(pages, (1..=110).collect::<Vec<_>>());
        for block in &document.blocks {
            assert!(!block.text().contains("[Page"), "{:?}", block.span);
            assert!(!block.text().starts_with("RFC 5905 "), "{:?}", block.span);
        }
    }

    #[test]
    fn rfc5905_has_37_numbered_and_31_appendix_headings_and_every_block_carries_its_section() {
        let text = rfc5905_text();
        let document = rfc5905();
        let headings: Vec<(usize, &str)> = text
            .lines()
            .enumerate()
            .filter_map(|(index, line)| numbered_heading(line).map(|n| (index + 1, n)))
            .collect();
        assert_eq!(
            headings.iter().map(|(_, n)| *n).collect::<Vec<_>>(),
            [
                "1", "1.1", "2", "3", "3.1", "4", "5", "6", "7", "7.1", "7.2", "7.3", "7.4", "7.5",
                "8", "9", "9.1", "9.2", "10", "11", "11.1", "11.2", "11.2.1", "11.2.2", "11.2.3",
                "11.3", "12", "13", "13.1", "13.2", "14", "15", "16", "17", "18", "18.1", "18.2"
            ]
        );
        assert_eq!(headings.len(), 37);
        assert_eq!(headings[0], (175, "1"));
        assert_eq!(headings[36], (3295, "18.2"));
        let line = |number: usize| text.lines().nth(number - 1).unwrap();
        assert!(line(175).starts_with("1.  Introduction"));
        assert!(line(3295).starts_with("18.2.  Informative References"));
        for (number, section, title) in [
            (889, "7.2", "7.2.  Global Parameters"),
            (938, "7.3", "7.3.  Packet Header Variables"),
            (1431, "8", "8.  On-Wire Protocol"),
            (1804, "9.2", "9.2.  Peer Process Operations"),
            (2829, "13", "13.  Poll Process"),
        ] {
            assert!(headings.contains(&(number, section)), "{number}");
            assert_eq!(line(number).trim_end(), title);
        }
        assert!(
            line(1061)
                .trim()
                .starts_with("Figure 8: Packet Header Format")
        );
        assert_eq!(numbered_heading(line(1061)), None);
        assert_eq!(block_at(&document, 1061).section.as_deref(), Some("7.3"));
        let all: Vec<(usize, &str)> = text
            .lines()
            .enumerate()
            .filter_map(|(index, line)| section_heading(line).map(|n| (index + 1, n)))
            .collect();
        let appendix: Vec<(usize, &str)> = all
            .iter()
            .copied()
            .filter(|heading| !headings.contains(heading))
            .collect();
        assert_eq!(all.len(), 37 + appendix.len());
        assert_eq!(&all[..37], &headings[..]);
        assert_eq!(appendix.first(), Some(&(3367, "A")));
        assert!(line(3367).starts_with("Appendix A.  Code Skeleton"));
        assert_eq!(appendix.last(), Some(&(6015, "A.5.7.3")));
        assert!(
            appendix.iter().all(|(_, n)| n.starts_with('A')),
            "{appendix:?}"
        );
        assert_eq!(appendix.len(), 31);
        assert_eq!(block_at(&document, 3394).section.as_deref(), Some("A.1"));
        for block in &document.blocks {
            let expected = all
                .iter()
                .rev()
                .find(|(line, _)| *line <= block.span.start)
                .map(|(_, n)| n.to_string());
            assert_eq!(block.section, expected, "{:?}", block.span);
            if block.span.start < 175 {
                assert_eq!(block.section, None);
            }
        }
    }

    #[test]
    fn the_rfc5905_header_diagram_and_the_global_parameters_table_are_one_figure_each() {
        let document = rfc5905();
        for (start, end, section) in [(1015, 1059, "7.3"), (903, 915, "7.2")] {
            let figure = block_at(&document, start);
            assert_eq!(
                (figure.kind, figure.span, figure.section.as_deref()),
                (BlockKind::Figure, Span { start, end }, Some(section)),
                "{start}"
            );
        }
        let table = block_at(&document, 903).text();
        for row in ["PORT", "123", "MINPOLL", "4", "MAXPOLL", "17"] {
            assert!(
                table.split_whitespace().any(|word| word == row),
                "{row}: {table}"
            );
        }
    }
}
