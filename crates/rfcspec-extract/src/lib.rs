#![forbid(unsafe_code)]
//! S2 and the S4 check. [`extract`] finds what can be found mechanically in a normalized document:
//! sentences with a BCP 14 keyword in capitals, sentences with a lowercase `must`, `should` or
//! `may` (prose candidates for review), dialogue steps (`S: ...`, `U: ...`) in figures, and
//! numeric examples (`N corresponds to DATE`). [`check`] verifies a reviewer's overlay against the
//! catalogue and the source text.

mod check;

pub use check::{check, layout_problems, quote_problem};
use std::collections::BTreeMap;

use rfcspec_model::{
    ABNF_COMMENT, Bcp14Requirement, BlockKind, CAPITALS_TEXT, CATALOGUE_FORMAT, Catalogue,
    DialogueStep, Document, NumericExample, OctetExample, ProseCandidate, Span,
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
    let mut found: Vec<(&'static str, String, Span, &'static str, Option<String>)> = Vec::new();
    let mut prose_candidates = Vec::new();
    for block in &document.blocks {
        if block.kind == BlockKind::Figure && block.lines.iter().any(|l| rfcspec_doc::abnf_rule(l))
        {
            for (lines, first) in abnf_comments(&block.lines, block.span.start) {
                for (text, span) in sentences(&lines, first) {
                    if let Some(level) = keywords(&text).first() {
                        found.push((level, text, span, ABNF_COMMENT, block.section.clone()));
                    }
                }
            }
        }
        if block.kind != BlockKind::Paragraph {
            continue;
        }
        let skip = boilerplate(&block.text());
        for (text, span) in sentences(&block.lines, block.span.start) {
            if let Some(level) = keywords(&text).first().filter(|_| !skip) {
                found.push((
                    level,
                    text.clone(),
                    span,
                    CAPITALS_TEXT,
                    block.section.clone(),
                ));
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

    Catalogue {
        format: CATALOGUE_FORMAT.into(),
        rfc: document.rfc,
        sha256: document.sha256.clone(),
        requirements,
        prose_candidates,
        dialogue_steps: dialogue_steps.into_iter().flatten().collect(),
        examples,
        octet_examples: octet_examples(document),
    }
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
        let boundary = end_mark
            && (next == bytes.len()
                || (next > index + 1 && (bytes[next].is_ascii_uppercase() || bytes[next] == b'(')));
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
        };
        let document = Document {
            rfc: 1,
            sha256: String::new(),
            furniture: vec![],
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
}
