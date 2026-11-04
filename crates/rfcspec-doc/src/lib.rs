#![forbid(unsafe_code)]
//! S1: the plain-text reader. It removes page furniture (form feeds, `... [Page N]` footers and the
//! `RFC N ... date` header block after each page break) and splits the rest into headings,
//! paragraphs and indented figures, each with the 1-based source lines it came from.
//!
//! Rules, in order, over the groups of consecutive non-blank lines that remain:
//!
//! - a one-line group without closing punctuation is a heading;
//! - a run of one-line indented groups, each one blank line from the next, is one figure (the
//!   dialogue lines and example lists of older RFCs);
//! - every other group is a paragraph.
//!
//! A paragraph split by a page break stays two paragraphs; RFC 868 has none.

use rfcspec_model::{Block, BlockKind, Document, Span};

/// Read an RFC's plain text into a normalized document.
pub fn read(rfc: u32, sha256: &str, text: &str) -> Document {
    let lines: Vec<&str> = text
        .strip_suffix('\n')
        .unwrap_or(text)
        .split('\n')
        .collect();
    let furniture = furniture(&lines);
    let blank = |index: usize| furniture.contains(&(index + 1)) || lines[index].trim().is_empty();

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
        groups.push((start, index - 1));
    }

    let indented = |(start, end): (usize, usize)| {
        lines[start..=end]
            .iter()
            .all(|line| line.starts_with([' ', '\t']))
    };
    let heading = |(start, end): (usize, usize)| {
        start == end
            && !lines[start]
                .trim_end()
                .ends_with(['.', ',', ':', ';', '?', '!'])
    };
    let figure_line =
        |group: (usize, usize)| group.0 == group.1 && indented(group) && !heading(group);

    let mut blocks = Vec::new();
    let mut next = 0;
    while next < groups.len() {
        let group = groups[next];
        next += 1;
        let (kind, start, end) = if heading(group) {
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
        let block_lines = lines[start..=end]
            .iter()
            .map(|line| match kind {
                BlockKind::Figure => line.trim_end().to_string(),
                BlockKind::Heading | BlockKind::Paragraph => line.trim().to_string(),
            })
            .collect();
        blocks.push(Block {
            kind,
            span: Span {
                start: start + 1,
                end: end + 1,
            },
            lines: block_lines,
        });
    }

    Document {
        rfc,
        sha256: sha256.to_string(),
        furniture,
        blocks,
    }
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
    }
}
