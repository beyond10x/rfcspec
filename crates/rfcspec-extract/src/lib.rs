#![forbid(unsafe_code)]
//! S2 and the S4 check. [`extract`] finds what can be found mechanically in a normalized document:
//! sentences with a BCP 14 keyword in capitals, sentences with a lowercase `must`, `should` or
//! `may` (prose candidates for review), dialogue steps (`S: ...`, `U: ...`) in figures, and
//! numeric examples (`N corresponds to DATE`). [`check`] verifies a reviewer's overlay against the
//! catalogue and the source text.

mod check;

pub use check::check;
use rfcspec_model::{
    Bcp14Requirement, BlockKind, CATALOGUE_FORMAT, Catalogue, DialogueStep, Document,
    NumericExample, ProseCandidate, Span,
};

/// The BCP 14 keywords, longest first so `MUST NOT` wins over `MUST`.
const BCP14: &[&str] = &[
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

/// The lowercase words that make a sentence a prose candidate.
const PROSE: &[&str] = &["must", "should", "may"];

/// Extract the mechanical catalogue of a document.
pub fn extract(document: &Document) -> Catalogue {
    let mut requirements = Vec::new();
    let mut prose_candidates = Vec::new();
    for block in &document.blocks {
        if block.kind != BlockKind::Paragraph {
            continue;
        }
        for (text, span) in sentences(&block.lines, block.span.start) {
            if let Some(keyword) = BCP14.iter().find(|keyword| has_word(&text, keyword)) {
                requirements.push(Bcp14Requirement {
                    id: format!("R-{}", requirements.len() + 1),
                    keyword: (*keyword).to_string(),
                    text: text.clone(),
                    span,
                });
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
    }
}

/// Split a paragraph's lines into sentences, each with the source lines it spans.
fn sentences(lines: &[String], first: usize) -> Vec<(String, Span)> {
    let mut text = String::new();
    let mut line_of = Vec::new();
    for (offset, line) in lines.iter().enumerate() {
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
}
