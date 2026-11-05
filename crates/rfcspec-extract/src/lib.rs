#![forbid(unsafe_code)]
//! S2 and the S4 check. [`extract`] finds what can be found mechanically in a normalized document:
//! sentences with a BCP 14 keyword in capitals (in a document read from RFCXML, sentences
//! containing a `<bcp14>` element), sentences with a lowercase `must`, `should` or
//! `may` (prose candidates for review), dialogue steps (`S: ...`, `U: ...`) in figures, and
//! numeric examples (`N corresponds to DATE`). [`check`] verifies a reviewer's overlay against the
//! catalogue and the source text.

mod check;

pub use check::{check, layout_problems, quote_problem};
use std::{collections::BTreeMap, ops::Range};

use rfcspec_model::{
    ABNF_COMMENT, BCP14_XML, Bcp14Requirement, BlockKind, CAPITALS_TEXT, CATALOGUE_FORMAT,
    Catalogue, DialogueStep, Document, NumericExample, OctetExample, ProseCandidate, SourceFormat,
    Span,
};

/// The eleven BCP 14 keywords, the levels a requirement is extracted at, longest first so `MUST
/// NOT` wins over `MUST`.
pub const LEVELS: &[&str] = &[
    "NOT RECOMMENDED",
    "SHALL NOT",
    "SHOULD NOT",
    "MUST NOT",
    "RECOMMENDED",
    "REQUIRED",
    "OPTIONAL",
    "SHOULD",
    "SHALL",
    "MUST",
    "MAY",
];

/// The version of the extraction: raised whenever [`extract`] or `rfcspec_doc::read` changes the
/// catalogue of some text, so a bulk summary records which extraction its counts came from.
pub const EXTRACTOR_VERSION: u32 = 1;

/// The number of a catalogue's BCP 14 requirements at each level that has any, keyed by the level
/// as [`LEVELS`] spells it.
pub fn level_counts(catalogue: &Catalogue) -> BTreeMap<String, u64> {
    let mut counts = BTreeMap::new();
    for requirement in &catalogue.requirements {
        *counts.entry(requirement.keyword.clone()).or_default() += 1;
    }
    counts
}

/// The lowercase words that make a sentence a prose candidate.
const PROSE: &[&str] = &["must", "should", "may"];

/// The BCP 14 keywords of a text in order, as disjoint whole words: at each position the longest
/// keyword wins, so a `MUST NOT` is not also a `MUST`.
pub fn keywords(text: &str) -> Vec<&'static str> {
    let bytes = text.as_bytes();
    let is_word = |b: u8| b.is_ascii_alphanumeric() || b == b'_';
    let mut found = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if index > 0 && is_word(bytes[index - 1]) {
            index += 1;
            continue;
        }
        let hit = LEVELS.iter().find(|keyword| {
            bytes[index..].starts_with(keyword.as_bytes())
                && !bytes
                    .get(index + keyword.len())
                    .is_some_and(|&b| is_word(b))
        });
        match hit {
            Some(keyword) => {
                found.push(*keyword);
                index += keyword.len();
            }
            None => index += 1,
        }
    }
    found
}

/// Whether a paragraph is the BCP 14 boilerplate that cites RFC 2119 and lists the keywords.
fn boilerplate(text: &str) -> bool {
    (text.contains("RFC2119") || text.contains("RFC 2119"))
        && text.contains("\"MUST\"")
        && text.contains("\"MAY\"")
}

/// The comments of an ABNF figure as runs of consecutive lines: a comment after a rule element
/// starts a run, a line that is only a comment continues the run above it. Each run is its
/// comment texts and its first line.
fn abnf_comments(lines: &[String], first: usize) -> Vec<(Vec<String>, usize)> {
    let mut runs: Vec<(Vec<String>, usize)> = Vec::new();
    let mut open = false;
    for (offset, line) in lines.iter().enumerate() {
        let Some((code, comment)) = comment(line) else {
            open = false;
            continue;
        };
        if open && code.trim().is_empty() {
            runs.last_mut()
                .expect("an open run")
                .0
                .push(comment.trim().to_string());
        } else {
            runs.push((vec![comment.trim().to_string()], first + offset));
            open = true;
        }
    }
    runs
}

/// A line split at its ABNF comment `;`, outside quoted strings and prose values.
fn comment(line: &str) -> Option<(&str, &str)> {
    let mut quoted = false;
    let mut prose = false;
    for (at, c) in line.char_indices() {
        match c {
            '"' if !prose => quoted = !quoted,
            '<' if !quoted => prose = true,
            '>' if !quoted => prose = false,
            ';' if !quoted && !prose => return Some((&line[..at], &line[at + 1..])),
            _ => {}
        }
    }
    None
}

/// Extract the mechanical catalogue of a document.
pub fn extract(document: &Document) -> Catalogue {
    extract_reporting(document).0
}

/// Extract the mechanical catalogue of a document, with one problem for each `<bcp14>` element
/// of an RFCXML document whose content is none of the [`LEVELS`], naming its source line.
///
/// A plain-text document takes a requirement from every sentence with a BCP 14 keyword in
/// capitals (`capitals-text`). An RFCXML document takes one from every sentence containing a
/// `<bcp14>` element instead (`bcp14-xml`), at the level the first such element holds; a
/// capitals keyword outside the markup is none, except in a figure: a figure carries no markup,
/// so an RFCXML figure that is neither ABNF nor a drawing (a line starting `+-` or `+ -`) gives
/// its capitals sentences as `capitals-text`, as the plain-text reader's paragraphs give the same
/// lines (a blank line ending a sentence, as it ends a paragraph). A figure is ABNF by its `type`
/// when it has one (`abnf`), else when a line defines a rule; its comments give `abnf-comment`
/// on both paths. The entries of an RFCXML `<dl>` (`Block::entries`) are sentence boundaries.
pub fn extract_reporting(document: &Document) -> (Catalogue, Vec<String>) {
    let xml = document.format == SourceFormat::Xml;
    let mut problems = Vec::new();
    let mut found: Vec<(&'static str, String, Span, &'static str, Option<String>)> = Vec::new();
    let mut prose_candidates = Vec::new();
    for block in &document.blocks {
        let abnf = block.kind == BlockKind::Figure
            && match block.figure_type.as_deref() {
                Some(kind) => kind == "abnf",
                None => block.lines.iter().any(|l| rfcspec_doc::abnf_rule(l)),
            };
        let drawn = block.lines.iter().any(|line| {
            let line = line.trim();
            line.starts_with("+-") || line.starts_with("+ -")
        });
        if abnf {
            for (lines, first) in abnf_comments(&block.lines, block.span.start) {
                for (text, span) in sentences(&lines, first) {
                    if let Some(level) = keywords(&text).first() {
                        found.push((level, text, span, ABNF_COMMENT, block.section.clone()));
                    }
                }
            }
        } else if xml && block.kind == BlockKind::Figure && !drawn {
            let lines: Vec<String> = block.lines.iter().map(|l| l.trim().to_string()).collect();
            let mut start = 0;
            for group in lines.split(String::is_empty) {
                for (text, span) in sentences(group, block.span.start + start) {
                    if let Some(level) = keywords(&text).first() {
                        found.push((level, text, span, CAPITALS_TEXT, block.section.clone()));
                    }
                }
                start += group.len() + 1;
            }
        }
        if block.kind != BlockKind::Paragraph {
            continue;
        }
        let skip = boilerplate(&block.text());
        let (_, line_of) = joined(&block.lines, block.span.start);
        if xml {
            for mark in block.bcp14.iter().filter(|m| level(&m.keyword).is_none()) {
                problems.push(format!(
                    "line {}: <bcp14>{}</bcp14> holds no BCP 14 level and does not set its sentence's level",
                    line_of.get(mark.offset).copied().unwrap_or(block.span.start),
                    mark.keyword
                ));
            }
        }
        for (text, span, range) in sentences_at(&block.lines, block.span.start, &block.entries) {
            let (level, source) = if xml {
                let first = block.bcp14.iter().find(|m| range.contains(&m.offset));
                (first.and_then(|m| level(&m.keyword)), BCP14_XML)
            } else {
                (keywords(&text).first().copied(), CAPITALS_TEXT)
            };
            if let Some(level) = level.filter(|_| !skip) {
                found.push((level, text.clone(), span, source, block.section.clone()));
            }
            let keywords: Vec<&str> = PROSE
                .iter()
                .copied()
                .filter(|keyword| has_word(&text, keyword))
                .collect();
            if !keywords.is_empty() {
                prose_candidates.push(ProseCandidate {
                    id: format!("P-{}", prose_candidates.len() + 1),
                    keyword: keywords.join(" "),
                    text,
                    span,
                });
            }
        }
    }
    let mut ordinals: BTreeMap<Option<String>, usize> = BTreeMap::new();
    let requirements = found
        .into_iter()
        .map(|(level, text, span, source, section)| {
            let ordinal = ordinals.entry(section.clone()).or_default();
            *ordinal += 1;
            Bcp14Requirement {
                id: format!("R{}-{ordinal}", section.as_deref().unwrap_or_default()),
                keyword: level.to_string(),
                text,
                span,
                source: source.to_string(),
                section,
            }
        })
        .collect();

    let mut dialogue_steps = Vec::new();
    let mut dialogues = 0;
    let mut examples = Vec::new();
    for block in &document.blocks {
        let mut in_dialogue = false;
        for (offset, line) in block.lines.iter().enumerate() {
            let number = block.span.start + offset;
            let span = Span {
                start: number,
                end: number,
            };
            let trimmed = line.trim();
            if block.kind == BlockKind::Figure
                && let Some((party, text)) = dialogue(trimmed)
            {
                if !in_dialogue {
                    in_dialogue = true;
                    dialogues += 1;
                    dialogue_steps.push(Vec::new());
                }
                let steps = dialogue_steps.last_mut().expect("pushed above");
                let id = format!("D{dialogues}-{}", steps.len() + 1);
                steps.push(DialogueStep {
                    id,
                    party: party.to_string(),
                    text: text.to_string(),
                    span,
                });
            }
            if let Some(seconds) = example(trimmed) {
                examples.push(NumericExample {
                    id: format!("E{}", examples.len() + 1),
                    seconds,
                    text: trimmed.to_string(),
                    span,
                });
            }
        }
    }

    let catalogue = Catalogue {
        format: CATALOGUE_FORMAT.into(),
        rfc: document.rfc,
        sha256: document.sha256.clone(),
        requirements,
        prose_candidates,
        dialogue_steps: dialogue_steps.into_iter().flatten().collect(),
        examples,
        octet_examples: octet_examples(document),
    };
    (catalogue, problems)
}

/// The level a `<bcp14>` element's whitespace-collapsed content names, if it is one of the
/// [`LEVELS`].
fn level(content: &str) -> Option<&'static str> {
    LEVELS.iter().copied().find(|level| *level == content)
}

/// The octet examples of a document: a bulleted line (`*`, `o` or `-`) whose text starts with a
/// `0x` token, with the lines of its block that continue it (no bullet of their own) joined.
fn octet_examples(document: &Document) -> Vec<OctetExample> {
    fn bullet(line: &str) -> Option<&str> {
        ["* ", "o ", "- "]
            .iter()
            .find_map(|mark| line.strip_prefix(mark).map(str::trim_start))
    }
    let mut found = Vec::new();
    for block in &document.blocks {
        let mut offset = 0;
        while offset < block.lines.len() {
            let line = block.lines[offset].trim();
            offset += 1;
            let Some(text) = bullet(line).filter(|text| text.starts_with("0x")) else {
                continue;
            };
            let start = block.span.start + offset - 1;
            let mut joined = text.to_string();
            while offset < block.lines.len() {
                let next = block.lines[offset].trim();
                if next.is_empty() || bullet(next).is_some() {
                    break;
                }
                joined.push(' ');
                joined.push_str(next);
                offset += 1;
            }
            if let Some((octets, elided)) = octets(&joined) {
                found.push(OctetExample {
                    id: format!("O{}", found.len() + 1),
                    octets,
                    elided,
                    text: joined,
                    span: Span {
                        start,
                        end: block.span.start + offset - 1,
                    },
                });
            }
        }
    }
    found
}

/// The octets of `0x81 0x7E 0x0100 [256 bytes of binary data] (comment)`: each token's hex
/// digits read big-endian, two per octet, and the `[N bytes ...]` count of elided octets.
fn octets(text: &str) -> Option<(Vec<u8>, u64)> {
    let mut octets = Vec::new();
    let mut rest = text;
    while let Some(after) = rest.strip_prefix("0x") {
        let digits: String = after.chars().take_while(char::is_ascii_hexdigit).collect();
        if digits.is_empty() || !digits.len().is_multiple_of(2) {
            return None;
        }
        for pair in digits.as_bytes().chunks(2) {
            octets.push(u8::from_str_radix(std::str::from_utf8(pair).ok()?, 16).ok()?);
        }
        rest = after[digits.len()..].trim_start();
    }
    let elided = rest
        .strip_prefix('[')
        .and_then(|inner| inner.split_once(" bytes"))
        .and_then(|(count, _)| count.trim().parse().ok())
        .unwrap_or(0);
    (!octets.is_empty()).then_some((octets, elided))
}

/// Split a paragraph's lines into sentences, each with the source lines it spans.
fn sentences(lines: &[String], first: usize) -> Vec<(String, Span)> {
    sentences_at(lines, first, &[])
        .into_iter()
        .map(|(text, span, _)| (text, span))
        .collect()
}

/// A paragraph's lines joined as [`rfcspec_model::Block::text`] joins them, with the source line
/// of every byte of the result.
fn joined(lines: &[String], first: usize) -> (String, Vec<usize>) {
    let mut text = String::new();
    let mut line_of = Vec::new();
    for (offset, line) in lines.iter().enumerate() {
        if line.is_empty() {
            continue;
        }
        if !text.is_empty() {
            text.push(' ');
            line_of.push(first + offset);
        }
        text.push_str(line);
        line_of.extend(std::iter::repeat_n(first + offset, line.len()));
    }
    (text, line_of)
}

/// [`sentences`], each also with the byte range it covers in the joined text, the range the
/// offsets of a block's `<bcp14>` marks are measured in. A sentence also ends before each offset
/// of `breaks` (the entries of a `<dl>`).
fn sentences_at(
    lines: &[String],
    first: usize,
    breaks: &[usize],
) -> Vec<(String, Span, Range<usize>)> {
    let (text, line_of) = joined(lines, first);
    let bytes = text.as_bytes();
    let mut found = Vec::new();
    let mut start = 0;
    let mut index = 0;
    while index < bytes.len() {
        let end_mark = matches!(bytes[index], b'.' | b'?' | b'!');
        let mut next = index + 1;
        while next < bytes.len() && bytes[next] == b' ' {
            next += 1;
        }
        let boundary = (end_mark
            && (next == bytes.len()
                || (next > index + 1
                    && (bytes[next].is_ascii_uppercase() || bytes[next] == b'('))))
            || (bytes[index] != b' ' && next > index + 1 && breaks.contains(&next));
        if boundary {
            let sentence = text[start..=index].trim();
            if !sentence.is_empty() {
                let lead = start + (text[start..].len() - text[start..].trim_start().len());
                found.push((
                    sentence.to_string(),
                    Span {
                        start: line_of[lead],
                        end: line_of[index],
                    },
                    lead..index + 1,
                ));
            }
            start = next;
            index = next;
            continue;
        }
        index += 1;
    }
    let rest = text[start..].trim();
    if !rest.is_empty() {
        found.push((
            rest.to_string(),
            Span {
                start: line_of[start],
                end: line_of[text.len() - 1],
            },
            start..text.len(),
        ));
    }
    found
}

/// Whether `word` occurs in `text` as a whole word (or, for a phrase, as whole words).
fn has_word(text: &str, word: &str) -> bool {
    let is_word = |c: char| c.is_ascii_alphanumeric() || c == '_';
    text.match_indices(word).any(|(at, _)| {
        let before = text[..at].chars().next_back();
        let after = text[at + word.len()..].chars().next();
        !before.is_some_and(is_word) && !after.is_some_and(is_word)
    })
}

/// `S: Listen on port 37.` as `("S", "Listen on port 37.")`.
fn dialogue(line: &str) -> Option<(&str, &str)> {
    let (party, text) = line.split_once(": ")?;
    matches!(party, "S" | "U").then_some((party, text.trim()))
}

/// The number before ` corresponds to ` in an example line such as
/// `the time  2,208,988,800 corresponds to 00:00  1 Jan 1970 GMT,`.
fn example(line: &str) -> Option<i64> {
    let (before, _) = line.split_once(" corresponds to ")?;
    let token = before.split_whitespace().next_back()?;
    let digits = token.strip_prefix('-').unwrap_or(token);
    let groups: Vec<&str> = digits.split(',').collect();
    let valid = !groups[0].is_empty()
        && groups[0].len() <= 3
        && groups.iter().all(|g| g.bytes().all(|b| b.is_ascii_digit()))
        && groups[1..].iter().all(|g| g.len() == 3);
    if !valid {
        return None;
    }
    let value: i64 = groups.concat().parse().ok()?;
    Some(if token.starts_with('-') {
        -value
    } else {
        value
    })
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use rfcspec_model::{Block, to_json};

    use std::collections::BTreeMap;

    use super::*;

    pub(crate) fn source() -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/time/sources/rfc868.txt"),
        )
        .unwrap()
    }

    pub(crate) fn catalogue() -> Catalogue {
        extract(&rfcspec_doc::read(868, "digest", &source()))
    }

    #[test]
    fn rfc868_has_no_bcp14_requirement_and_both_should_sentences_are_prose_candidates() {
        let catalogue = catalogue();
        assert!(catalogue.requirements.is_empty());
        let texts: Vec<&str> = catalogue
            .prose_candidates
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert!(texts.contains(
            &"If the server is unable to determine the time at its site, it should either refuse the connection or close it without sending anything."
        ), "{texts:#?}");
        assert!(texts.contains(
            &"If the server is unable to determine the time at its site, it should discard the arriving datagram and make no reply."
        ), "{texts:#?}");
        let tcp = catalogue
            .prose_candidates
            .iter()
            .find(|c| c.text.contains("refuse the connection"))
            .unwrap();
        assert_eq!(tcp.span, Span { start: 46, end: 48 });
        assert_eq!(tcp.keyword, "should");
    }

    #[test]
    fn every_sentence_with_lowercase_must_should_or_may_is_a_prose_candidate() {
        let document = rfcspec_doc::read(868, "digest", &source());
        let catalogue = extract(&document);
        let mut expected = Vec::new();
        for block in document
            .blocks
            .iter()
            .filter(|b| b.kind == BlockKind::Paragraph)
        {
            for (text, _) in sentences(&block.lines, block.span.start) {
                if PROSE.iter().any(|word| has_word(&text, word)) {
                    expected.push(text);
                }
            }
        }
        let found: Vec<String> = catalogue
            .prose_candidates
            .iter()
            .map(|c| c.text.clone())
            .collect();
        assert_eq!(found, expected);
        assert_eq!(found.len(), 3, "{found:#?}");
        assert!(found[0].starts_with("This protocol may be used"));
    }

    #[test]
    fn capitals_are_bcp14_requirements_lowercase_are_not() {
        let block = Block {
            kind: BlockKind::Paragraph,
            span: Span { start: 10, end: 11 },
            lines: vec![
                "The client MUST NOT send data.  A server".into(),
                "may close.".into(),
            ],
            section: None,
            anchor: None,
            figure_type: None,
            bcp14: vec![],
            entries: vec![],
        };
        let document = Document {
            rfc: 1,
            sha256: String::new(),
            format: Default::default(),
            furniture: vec![],
            references: vec![],
            reference_counts: None,
            blocks: vec![block],
        };
        let catalogue = extract(&document);
        assert_eq!(catalogue.requirements.len(), 1);
        assert_eq!(catalogue.requirements[0].keyword, "MUST NOT");
        assert_eq!(catalogue.requirements[0].span, Span { start: 10, end: 10 });
        assert_eq!(catalogue.prose_candidates.len(), 1);
        assert_eq!(
            catalogue.prose_candidates[0].span,
            Span { start: 10, end: 11 }
        );
    }

    #[test]
    fn eleven_dialogue_steps_six_tcp_five_udp() {
        let steps = catalogue().dialogue_steps;
        assert_eq!(steps.len(), 11);
        let tcp: Vec<_> = steps.iter().filter(|s| s.id.starts_with("D1-")).collect();
        let udp: Vec<_> = steps.iter().filter(|s| s.id.starts_with("D2-")).collect();
        assert_eq!((tcp.len(), udp.len()), (6, 5));
        assert!(steps.iter().all(|s| s.party == "S" || s.party == "U"));
        assert_eq!(
            (tcp[0].party.as_str(), tcp[0].text.as_str(), tcp[0].span),
            (
                "S",
                "Listen on port 37 (45 octal).",
                Span { start: 32, end: 32 }
            )
        );
        assert_eq!(
            (
                tcp[5].party.as_str(),
                tcp[5].text.as_str(),
                tcp[5].span.start
            ),
            ("S", "Close the connection.", 42)
        );
        assert_eq!(
            (
                udp[1].party.as_str(),
                udp[1].text.as_str(),
                udp[1].span.start
            ),
            ("U", "Send an empty datagram to port 37.", 68)
        );
        assert_eq!(udp[4].span.start, 74);
    }

    #[test]
    fn five_numeric_examples_the_last_negative() {
        let examples = catalogue().examples;
        let seconds: Vec<i64> = examples.iter().map(|e| e.seconds).collect();
        assert_eq!(
            seconds,
            [2208988800, 2398291200, 2524521600, 2629584000, -1297728000]
        );
        assert_eq!(examples[4].span, Span { start: 97, end: 97 });
        assert_eq!(
            examples[4].text,
            "and -1,297,728,000 corresponds to 00:00 17 Nov 1858 GMT."
        );
    }

    #[test]
    fn catalogue_is_byte_identical_across_two_runs() {
        let first = to_json(&catalogue()).unwrap();
        let second = to_json(&catalogue()).unwrap();
        assert_eq!(first.as_bytes(), second.as_bytes());
    }

    pub(crate) fn source_6455() -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/websocket/sources/rfc6455.txt"),
        )
        .unwrap()
    }

    pub(crate) fn catalogue_6455() -> Catalogue {
        extract(&rfcspec_doc::read(6455, "digest", &source_6455()))
    }

    /// Keyword occurrences, disjoint, over the texts of the requirements `keep` selects.
    fn counts(
        catalogue: &Catalogue,
        keep: impl Fn(&Bcp14Requirement) -> bool,
    ) -> BTreeMap<&'static str, usize> {
        let mut counts = BTreeMap::new();
        for requirement in catalogue.requirements.iter().filter(|r| keep(r)) {
            for keyword in keywords(&requirement.text) {
                *counts.entry(keyword).or_default() += 1;
            }
        }
        counts
    }

    fn in_section(requirement: &Bcp14Requirement, section: &str) -> bool {
        requirement.section.as_deref().is_some_and(|s| {
            s == section
                || s.strip_prefix(section)
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    }

    #[test]
    fn rfc6455_requirement_texts_carry_214_keywords_outside_the_boilerplate() {
        let catalogue = catalogue_6455();
        let counts = counts(&catalogue, |_| true);
        assert_eq!(
            counts,
            BTreeMap::from([
                ("MUST", 113),
                ("MUST NOT", 25),
                ("SHOULD", 24),
                ("SHOULD NOT", 4),
                ("MAY", 47),
                ("OPTIONAL", 1),
            ])
        );
        assert_eq!(counts.values().sum::<usize>(), 214);
        for absent in [
            "REQUIRED",
            "SHALL",
            "SHALL NOT",
            "RECOMMENDED",
            "NOT RECOMMENDED",
        ] {
            assert!(!counts.contains_key(absent), "{absent}");
        }
        let whole = keywords(&source_6455());
        let count = |k: &str| whole.iter().filter(|w| **w == k).count();
        assert_eq!(
            [
                count("MUST") + count("MUST NOT"),
                count("MUST NOT"),
                count("SHOULD") + count("SHOULD NOT"),
                count("SHOULD NOT"),
                count("MAY"),
                count("OPTIONAL"),
                count("REQUIRED"),
                count("RECOMMENDED"),
                count("SHALL"),
                count("SHALL NOT"),
            ],
            [140, 26, 30, 5, 48, 2, 1, 1, 1, 1]
        );
        assert_eq!(whole.len() - 214, 10);
        assert!(
            catalogue
                .requirements
                .iter()
                .all(|r| !(658..=660).contains(&r.span.start))
        );
        assert!(
            catalogue
                .requirements
                .iter()
                .all(|r| r.section.is_some() && r.id.starts_with('R'))
        );
    }

    #[test]
    fn rfc6455_scoped_sections_carry_their_counts() {
        let catalogue = catalogue_6455();
        assert_eq!(
            counts(&catalogue, |r| in_section(r, "5.2")),
            BTreeMap::from([("MUST", 10), ("MAY", 1)])
        );
        assert_eq!(
            catalogue
                .requirements
                .iter()
                .filter(|r| in_section(r, "5.2") && r.source == ABNF_COMMENT)
                .count(),
            3
        );
        assert_eq!(
            counts(&catalogue, |r| in_section(r, "5.5")),
            BTreeMap::from([("MUST", 6), ("MUST NOT", 3), ("SHOULD", 3), ("MAY", 9)])
        );
        assert_eq!(
            counts(&catalogue, |r| in_section(r, "7")),
            BTreeMap::from([
                ("MUST", 6),
                ("MUST NOT", 4),
                ("SHOULD", 11),
                ("SHOULD NOT", 3),
                ("MAY", 9)
            ])
        );
    }

    #[test]
    fn rfc6455_has_exactly_five_abnf_comment_requirements() {
        let catalogue = catalogue_6455();
        let abnf: Vec<&Bcp14Requirement> = catalogue
            .requirements
            .iter()
            .filter(|r| r.source == ABNF_COMMENT)
            .collect();
        let source = source_6455();
        let lines: Vec<&str> = source.lines().collect();
        let keyword_lines: Vec<(usize, &str)> = abnf
            .iter()
            .map(|r| {
                let line = (r.span.start..=r.span.end)
                    .find(|&n| !keywords(lines[n - 1]).is_empty())
                    .unwrap();
                (line, r.section.as_deref().unwrap())
            })
            .collect();
        assert_eq!(
            keyword_lines,
            [
                (1409, "4.3"),
                (1706, "5.2"),
                (1710, "5.2"),
                (1714, "5.2"),
                (2678, "9.1")
            ]
        );
        for requirement in &abnf[1..4] {
            assert_eq!(
                requirement.text,
                "1 bit in length, MUST be 0 unless negotiated otherwise"
            );
            assert_eq!(requirement.keyword, "MUST");
        }
        assert!(
            catalogue
                .requirements
                .iter()
                .filter(|r| r.source == CAPITALS_TEXT)
                .count()
                == catalogue.requirements.len() - 5
        );
    }

    #[test]
    fn ids_count_requirements_per_section_in_document_order() {
        let catalogue = catalogue_6455();
        let first = catalogue
            .requirements
            .iter()
            .find(|r| r.id == "R5.2-1")
            .unwrap();
        assert_eq!(
            first.text,
            "The first fragment MAY also be the final fragment."
        );
        assert_eq!(
            first.span,
            Span {
                start: 1556,
                end: 1557
            }
        );
        assert_eq!(first.section.as_deref(), Some("5.2"));
        let mut seen: BTreeMap<&str, usize> = BTreeMap::new();
        for requirement in &catalogue.requirements {
            let section = requirement.section.as_deref().unwrap();
            let ordinal = seen.entry(section).or_default();
            *ordinal += 1;
            assert_eq!(requirement.id, format!("R{section}-{ordinal}"));
        }
        let spans: Vec<usize> = catalogue
            .requirements
            .iter()
            .map(|r| r.span.start)
            .collect();
        assert!(spans.windows(2).all(|w| w[0] <= w[1]));
    }

    #[test]
    fn the_level_is_the_first_keyword_of_the_sentence() {
        let catalogue = catalogue_6455();
        let both = catalogue
            .requirements
            .iter()
            .find(|r| r.text.starts_with("The server MUST close the underlying TCP connection immediately; the client SHOULD wait"))
            .unwrap();
        assert_eq!(
            both.span,
            Span {
                start: 2025,
                end: 2029
            }
        );
        assert_eq!(both.keyword, "MUST");
        assert_eq!(
            keywords("A client SHOULD NOT send and MUST NOT"),
            ["SHOULD NOT", "MUST NOT"]
        );
        assert_eq!(keywords("MAY or MUST"), ["MAY", "MUST"]);
        assert_eq!(keywords("MUSTARD and NOT RECOMMENDED"), ["NOT RECOMMENDED"]);
    }

    #[test]
    fn a_sentence_continued_across_a_page_break_is_one_requirement() {
        let catalogue = catalogue_6455();
        let sentence = catalogue
            .requirements
            .iter()
            .find(|r| r.text == "The list of extensions that are needed SHOULD appear in the /reason/ part of the Close frame.")
            .unwrap();
        assert_eq!(
            sentence.span,
            Span {
                start: 2572,
                end: 2583
            }
        );
        assert_eq!(sentence.section.as_deref(), Some("7.4.1"));
    }

    #[test]
    fn rfc6455_has_eight_octet_examples_and_rfc868_none() {
        let examples = catalogue_6455().octet_examples;
        let spans: Vec<(usize, usize)> = examples
            .iter()
            .map(|e| (e.span.start, e.span.end))
            .collect();
        assert_eq!(
            spans,
            [
                (2111, 2111),
                (2115, 2116),
                (2120, 2120),
                (2122, 2122),
                (2137, 2138),
                (2140, 2141),
                (2145, 2145),
                (2149, 2149)
            ]
        );
        assert_eq!(
            examples[0].octets,
            [0x81, 0x05, 0x48, 0x65, 0x6c, 0x6c, 0x6f]
        );
        assert_eq!(examples[0].elided, 0);
        assert_eq!(
            examples[1].text,
            "0x81 0x85 0x37 0xfa 0x21 0x3d 0x7f 0x9f 0x4d 0x51 0x58 (contains \"Hello\")"
        );
        assert_eq!(examples[6].octets, [0x82, 0x7e, 0x01, 0x00]);
        assert_eq!(examples[6].elided, 256);
        assert_eq!(
            examples[7].octets,
            [0x82, 0x7f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00]
        );
        assert_eq!(examples[7].elided, 65536);
        assert_eq!(
            examples.iter().map(|e| e.id.as_str()).collect::<Vec<_>>(),
            ["O1", "O2", "O3", "O4", "O5", "O6", "O7", "O8"]
        );
        let time = catalogue();
        assert!(time.octet_examples.is_empty());
        assert_eq!(time.examples.len(), 5);
        assert!(!to_json(&time).unwrap().contains("octet_examples"));
        assert_eq!(octets("0x0100 [2 bytes of x]"), Some((vec![1, 0], 2)));
        assert_eq!(octets("0x123"), None);
    }

    #[test]
    fn the_rfc6455_catalogue_is_byte_identical_across_two_runs() {
        let first = to_json(&catalogue_6455()).unwrap();
        let second = to_json(&catalogue_6455()).unwrap();
        assert_eq!(first.as_bytes(), second.as_bytes());
    }

    fn source_1035() -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/dns/sources/rfc1035.txt"),
        )
        .unwrap()
    }

    fn catalogue_1035() -> Catalogue {
        extract(&rfcspec_doc::read(1035, "digest", &source_1035()))
    }

    /// How often `word` occurs in `text` as a whole word, case-sensitively.
    fn occurrences(text: &str, word: &str) -> usize {
        text.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .filter(|w| *w == word)
            .count()
    }

    #[test]
    fn rfc1035_holds_no_bcp14_keyword_and_its_catalogue_no_requirement() {
        let text = source_1035();
        for keyword in [
            "MUST",
            "SHOULD",
            "MAY",
            "SHALL",
            "REQUIRED",
            "RECOMMENDED",
            "OPTIONAL",
        ] {
            assert_eq!(occurrences(&text, keyword), 0, "{keyword}");
        }
        assert_eq!(catalogue_1035().requirements.len(), 0);
    }

    #[test]
    fn rfc1035_scoped_keyword_lines_lie_in_prose_candidates_and_the_z_field_does_not() {
        let text = source_1035();
        assert_eq!(
            PROSE
                .iter()
                .map(|word| occurrences(&text, word))
                .collect::<Vec<_>>(),
            [23, 98, 46],
            "must, should, may"
        );
        let lines: Vec<&str> = text.lines().collect();
        assert!(lines[1472].contains("Must be zero"));
        assert!(lines[2892].contains(" May 1987"));
        let catalogue = catalogue_1035();
        for (line, word) in [
            (1447, "may"),
            (1464, "may"),
            (1501, "may"),
            (1503, "may"),
            (1761, "may"),
            (1763, "may"),
            (1764, "should"),
            (1769, "should"),
            (1772, "should"),
            (1776, "should"),
            (2529, "should"),
            (2557, "should"),
            (2559, "must"),
            (2565, "should"),
            (2566, "should"),
        ] {
            assert!(has_word(lines[line - 1], word), "{line}: {word}");
            assert!(
                catalogue
                    .prose_candidates
                    .iter()
                    .any(|c| c.span.contains(line) && has_word(&c.text, word)),
                "line {line} ({word}) is in no prose candidate"
            );
        }
        assert!(
            !catalogue
                .prose_candidates
                .iter()
                .any(|c| c.span.contains(1473) || c.text.contains("Must be zero")),
            "the Z field sentence is a prose candidate"
        );
    }

    #[test]
    fn the_rfc1035_catalogue_is_byte_identical_across_two_runs() {
        let first = to_json(&catalogue_1035()).unwrap();
        let second = to_json(&catalogue_1035()).unwrap();
        assert_eq!(first.as_bytes(), second.as_bytes());
    }

    fn source_5905() -> String {
        fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../../profiles/ntp/sources/rfc5905.txt"),
        )
        .unwrap()
    }

    fn document_5905() -> rfcspec_model::Document {
        rfcspec_doc::read(5905, "digest", &source_5905())
    }

    fn catalogue_5905() -> Catalogue {
        extract(&document_5905())
    }

    const NTP_SCOPE: [&str; 5] = ["7.2", "7.3", "8", "9.2", "13"];

    fn in_ntp_scope(section: &str) -> bool {
        NTP_SCOPE.iter().any(|listed| {
            section == *listed
                || section
                    .strip_prefix(listed)
                    .is_some_and(|rest| rest.starts_with('.'))
        })
    }

    #[test]
    fn rfc5905_has_13_bcp14_requirements_three_of_them_in_scope_in_section_9_2() {
        let catalogue = catalogue_5905();
        assert_eq!(
            catalogue
                .requirements
                .iter()
                .map(|r| r.id.as_str())
                .collect::<Vec<_>>(),
            [
                "R2-1",
                "R2-2",
                "R2-3",
                "R7.4-1",
                "R7.4-2",
                "R7.4-3",
                "R7.4-4",
                "R7.4-5",
                "R9.2-1",
                "R9.2-2",
                "R9.2-3",
                "R11.2.3-1",
                "R11.2.3-2"
            ]
        );
        for requirement in &catalogue.requirements {
            let section = requirement.section.as_deref().unwrap();
            assert!(
                requirement.id.starts_with(&format!("R{section}-")),
                "{}",
                requirement.id
            );
        }
        let in_scope: Vec<(&str, &str, usize, usize)> = catalogue
            .requirements
            .iter()
            .filter(|r| in_ntp_scope(r.section.as_deref().unwrap()))
            .map(|r| (r.id.as_str(), r.keyword.as_str(), r.span.start, r.span.end))
            .collect();
        assert_eq!(
            in_scope,
            [
                ("R9.2-1", "MAY", 1889, 1890),
                ("R9.2-2", "MAY", 1924, 1925),
                ("R9.2-3", "SHOULD", 1926, 1927)
            ]
        );
    }

    #[test]
    fn rfc5905_scoped_prose_candidates_are_the_ten_the_reviewer_reads() {
        let document = document_5905();
        let catalogue = extract(&document);
        let section_of = |line: usize| {
            document
                .blocks
                .iter()
                .find(|block| block.span.contains(line))
                .and_then(|block| block.section.clone())
        };
        let scoped: Vec<(&str, &str, usize)> = catalogue
            .prose_candidates
            .iter()
            .filter(|c| section_of(c.span.start).is_some_and(|s| in_ntp_scope(&s)))
            .map(|c| (c.id.as_str(), c.keyword.as_str(), c.span.start))
            .collect();
        assert_eq!(
            scoped,
            [
                ("P-14", "may", 932),
                ("P-15", "should", 1265),
                ("P-16", "should", 1283),
                ("P-17", "should", 1440),
                ("P-18", "may", 1451),
                ("P-19", "must", 1613),
                ("P-20", "should", 1639),
                ("P-24", "must", 1816),
                ("P-25", "must", 1866),
                ("P-29", "must", 2958)
            ]
        );
    }

    #[test]
    fn the_rfc5905_catalogue_is_byte_identical_across_two_runs() {
        let first = to_json(&catalogue_5905()).unwrap();
        let second = to_json(&catalogue_5905()).unwrap();
        assert_eq!(first.as_bytes(), second.as_bytes());
    }

    /// An RFCXML v3 document whose section 1 holds `body`, starting on source line 5.
    fn xml_document(body: &str) -> Document {
        let text = format!(
            "<rfc version=\"3\">\n<middle>\n<section pn=\"section-1\">\n<name>Intro</name>\n{body}\n</section>\n</middle>\n</rfc>\n"
        );
        rfcspec_doc::xml::read(1, "digest", &text).unwrap()
    }

    fn levels(catalogue: &Catalogue) -> Vec<(&str, &str, &str)> {
        catalogue
            .requirements
            .iter()
            .map(|r| (r.id.as_str(), r.keyword.as_str(), r.source.as_str()))
            .collect()
    }

    #[test]
    fn an_xml_sentence_with_bcp14_must_not_is_a_requirement_at_must_not() {
        let (catalogue, problems) = extract_reporting(&xml_document(
            "<t>The client <bcp14>MUST NOT</bcp14> send data. Nothing else.</t>",
        ));
        assert_eq!(levels(&catalogue), [("R1-1", "MUST NOT", BCP14_XML)]);
        assert_eq!(
            catalogue.requirements[0].text,
            "The client MUST NOT send data."
        );
        assert_eq!(catalogue.requirements[0].span, Span { start: 5, end: 5 });
        assert!(problems.is_empty(), "{problems:?}");
    }

    #[test]
    fn an_xml_sentence_with_two_bcp14_elements_is_one_requirement_at_the_first() {
        let (catalogue, _) = extract_reporting(&xml_document(
            "<t>A server <bcp14>MAY</bcp14> close and\nthe client <bcp14>MUST</bcp14> then stop.</t>",
        ));
        assert_eq!(levels(&catalogue), [("R1-1", "MAY", BCP14_XML)]);
        assert_eq!(catalogue.requirements[0].span, Span { start: 5, end: 6 });
    }

    #[test]
    fn an_xml_sentence_with_capitals_must_and_no_bcp14_element_is_no_requirement() {
        let (catalogue, problems) =
            extract_reporting(&xml_document("<t>The client MUST send data.</t>"));
        assert!(
            catalogue.requirements.is_empty(),
            "{:?}",
            levels(&catalogue)
        );
        assert!(problems.is_empty(), "{problems:?}");
    }

    #[test]
    fn an_xml_bcp14_element_holding_no_level_gives_no_requirement_and_names_its_line() {
        let (catalogue, problems) = extract_reporting(&xml_document(
            "<t>The first sentence.\nThe client <bcp14>must</bcp14> send data.</t>",
        ));
        assert!(
            catalogue.requirements.is_empty(),
            "{:?}",
            levels(&catalogue)
        );
        assert_eq!(problems.len(), 1, "{problems:?}");
        assert!(problems[0].contains("line 6"), "{problems:?}");
        assert!(problems[0].contains("must"), "{problems:?}");
        assert_eq!(catalogue.prose_candidates.len(), 1);
    }

    #[test]
    fn an_xml_abnf_comment_with_capitals_must_is_an_abnf_comment_requirement() {
        let (catalogue, _) = extract_reporting(&xml_document(
            "<sourcecode type=\"abnf\">\nframe = %x00-FF ; MUST be zero\n</sourcecode>",
        ));
        assert_eq!(levels(&catalogue), [("R1-1", "MUST", ABNF_COMMENT)]);
        assert_eq!(catalogue.requirements[0].text, "MUST be zero");
    }

    #[test]
    fn an_xml_type_abnf_figure_without_a_rule_line_gives_its_comment_as_abnf_comment() {
        let body = "<sourcecode type=\"abnf\">\n        / %x02-FF\n        ; MUST be reserved\n</sourcecode>";
        let document = xml_document(body);
        assert!(
            !document.blocks[1]
                .lines
                .iter()
                .any(|l| rfcspec_doc::abnf_rule(l))
        );
        let (catalogue, _) = extract_reporting(&document);
        assert_eq!(levels(&catalogue), [("R1-1", "MUST", ABNF_COMMENT)]);
        assert_eq!(catalogue.requirements[0].text, "MUST be reserved");
        assert_eq!(catalogue.requirements[0].span, Span { start: 7, end: 7 });
    }

    #[test]
    fn an_xml_figure_that_is_no_abnf_gives_capitals_requirements_as_capitals_text() {
        let (catalogue, problems) = extract_reporting(&xml_document(
            "<sourcecode type=\"yang\">\n  leaf port {\n    description\n      \"The port MUST be set.\";\n  }\n</sourcecode>",
        ));
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(levels(&catalogue), [("R1-1", "MUST", CAPITALS_TEXT)]);
        assert_eq!(
            catalogue.requirements[0].text,
            "leaf port { description \"The port MUST be set.\"; }"
        );
        assert_eq!(catalogue.requirements[0].span, Span { start: 6, end: 9 });
        assert!(catalogue.prose_candidates.is_empty());
    }

    #[test]
    fn a_blank_line_in_an_xml_figure_ends_a_sentence_as_it_ends_a_text_paragraph() {
        let (catalogue, _) = extract_reporting(&xml_document(
            "<sourcecode type=\"yang\">\n  \"It is NOT RECOMMENDED to enable this.\"; }\n\n  grouping g { description \"A client MUST send.\"; }\n</sourcecode>",
        ));
        let texts: Vec<(&str, Span)> = catalogue
            .requirements
            .iter()
            .map(|r| (r.text.as_str(), r.span))
            .collect();
        assert_eq!(
            texts,
            [
                (
                    "\"It is NOT RECOMMENDED to enable this.\"; }",
                    Span { start: 6, end: 6 }
                ),
                (
                    "grouping g { description \"A client MUST send.\"; }",
                    Span { start: 8, end: 8 }
                )
            ]
        );
    }

    #[test]
    fn an_xml_figure_typed_as_another_language_is_no_abnf_though_a_line_looks_like_a_rule() {
        let document = xml_document(
            "<sourcecode type=\"yang\">\n  description\n    \"The unit:\n     kbps = kilobits per second. A rate MUST be set.\";\n</sourcecode>",
        );
        assert!(
            document.blocks[1]
                .lines
                .iter()
                .any(|l| rfcspec_doc::abnf_rule(l))
        );
        let (catalogue, _) = extract_reporting(&document);
        assert_eq!(levels(&catalogue), [("R1-1", "MUST", CAPITALS_TEXT)]);
        assert_eq!(catalogue.requirements[0].text, "A rate MUST be set.\";");
    }

    #[test]
    fn an_xml_diagram_with_a_border_line_gives_no_requirement_as_a_text_figure_gives_none() {
        let (catalogue, _) = extract_reporting(&xml_document(
            "<artwork type=\"ascii-art\">\n  +-----------------+\n  | header OPTIONAL |\n  +-----------------+\n</artwork>",
        ));
        assert!(
            catalogue.requirements.is_empty(),
            "{:?}",
            levels(&catalogue)
        );
    }

    fn catalogue_9220_xml() -> (Catalogue, Vec<String>) {
        let text = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../rfcspec-doc/tests/fixtures/rfc9220.xml"),
        )
        .unwrap();
        extract_reporting(&rfcspec_doc::xml::read(9220, "digest", &text).unwrap())
    }

    #[test]
    fn rfc9220_xml_gives_r3_1_should_and_r3_2_may_and_the_boilerplate_none() {
        let (catalogue, problems) = catalogue_9220_xml();
        assert!(problems.is_empty(), "{problems:?}");
        let found: Vec<(&str, &str, &str, usize, usize)> = catalogue
            .requirements
            .iter()
            .map(|r| {
                (
                    r.id.as_str(),
                    r.keyword.as_str(),
                    r.source.as_str(),
                    r.span.start,
                    r.span.end,
                )
            })
            .collect();
        assert_eq!(
            found,
            [
                ("R3-1", "SHOULD", BCP14_XML, 130, 133),
                ("R3-2", "MAY", BCP14_XML, 133, 134)
            ]
        );
        assert_eq!(
            catalogue.requirements[1].text,
            "A server MAY provide more information via a \"problem details\" response [RFC7807]."
        );
    }

    #[test]
    fn rfc9220_xml_and_text_give_the_same_requirements_and_prose_candidates() {
        let text = fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../rfcspec-doc/tests/fixtures/rfc9220.txt"),
        )
        .unwrap();
        let from_text = extract(&rfcspec_doc::read(9220, "digest", &text));
        let (from_xml, _) = catalogue_9220_xml();
        let requirements =
            |catalogue: &Catalogue| -> Vec<(String, String, Option<String>, String)> {
                catalogue
                    .requirements
                    .iter()
                    .map(|r| {
                        (
                            r.id.clone(),
                            r.keyword.clone(),
                            r.section.clone(),
                            rfcspec_model::collapse_whitespace(&r.text),
                        )
                    })
                    .collect()
            };
        assert_eq!(requirements(&from_xml), requirements(&from_text));
        assert!(
            from_text
                .requirements
                .iter()
                .all(|r| r.source == CAPITALS_TEXT)
        );
        let prose = |catalogue: &Catalogue| -> Vec<String> {
            catalogue
                .prose_candidates
                .iter()
                .map(|c| rfcspec_model::collapse_whitespace(&c.text))
                .collect()
        };
        assert_eq!(prose(&from_xml), prose(&from_text));
        assert_eq!(prose(&from_xml).len(), 2, "{:?}", prose(&from_xml));
    }
}
