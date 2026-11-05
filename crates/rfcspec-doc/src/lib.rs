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
//! - a one-line group without closing punctuation is a heading;
//! - a run of one-line groups indented deeper than the body text, each one blank line from the
//!   next, is one figure (the dialogue lines and example lists of older RFCs); the body text's
//!   indentation is the most common one among the lines of multi-line groups;
//! - every other group is a paragraph.
//!
//! A numbered heading is an unindented line `N.  Title`, `N.M.  Title`, ...; every block from it to
//! the next one carries its number.

use rfcspec_model::{Block, BlockKind, Document, Span};

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
    let mut blocks = Vec::new();
    let mut next = 0;
    while next < groups.len() {
        let group = groups[next];
        next += 1;
        let (kind, start, end) = if drawn(group) {
            (BlockKind::Figure, group.0, group.1)
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
        let block_lines = (start..=end)
            .map(|index| match kind {
                _ if is_furniture[index] => String::new(),
                BlockKind::Figure => lines[index].trim_end().to_string(),
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
        });
    }

    Document {
        rfc,
        sha256: sha256.to_string(),
        furniture,
        blocks,
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

/// The section of every source line: the number of the last numbered heading at or before it.
fn sections(lines: &[&str], is_furniture: &[bool]) -> Vec<Option<String>> {
    let mut current: Option<String> = None;
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            if !is_furniture[index]
                && let Some(number) = numbered_heading(line)
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
}
