//! The RFC 5234 reader with the RFC 7405 `%s` and `%i` prefixes (dialect `rfc5234`).
//!
//! A rule starts wherever a rulename is followed by `=` or `=/`, so figure indentation does not
//! matter; every `;` comment is kept with the line it is on.

use crate::{Comment, Error, Node, Rule, RuleSet};

/// Whether a text is a rulename: `ALPHA *(ALPHA / DIGIT / "-")`.
pub(crate) fn is_rulename(text: &str) -> bool {
    text.starts_with(|c: char| c.is_ascii_alphabetic())
        && text.chars().all(|c| c.is_ascii_alphanumeric() || c == '-')
}

pub(crate) fn parse(lines: &[(usize, &str)]) -> Result<RuleSet, Error> {
    let (tokens, comments) = lex(lines)?;
    let end = lines
        .iter()
        .rev()
        .find(|(_, text)| !text.trim().is_empty())
        .map(|(line, text)| (*line, text.chars().count() + 1))
        .unwrap_or((lines.first().map_or(0, |(line, _)| *line), 1));
    let mut parser = Parser {
        tokens,
        next: 0,
        end,
    };
    let mut set = RuleSet::default();
    let mut starts: Vec<(usize, usize)> = Vec::new();
    while parser.next < parser.tokens.len() {
        let name_token = parser.tokens[parser.next].clone();
        let Kind::Name(name) = &name_token.kind else {
            return Err(parser.error_at(&name_token, "expected a rulename"));
        };
        parser.next += 1;
        let incremental = match parser.peek().map(|token| &token.kind) {
            Some(Kind::Defined) => false,
            Some(Kind::Incremental) => true,
            _ => return Err(parser.error_here("expected `=` or `=/` after the rulename")),
        };
        parser.next += 1;
        let definition = parser.alternation()?;
        let existing = set
            .rules
            .iter()
            .position(|rule| rule.name.eq_ignore_ascii_case(name));
        let index = match (existing, incremental) {
            (None, false) => {
                set.rules.push(Rule {
                    name: name.clone(),
                    line: name_token.line,
                    definition,
                    comments: Vec::new(),
                });
                set.rules.len() - 1
            }
            (Some(index), true) => {
                let rule = &mut set.rules[index];
                let mut alternatives =
                    match std::mem::replace(&mut rule.definition, Node::Alternation(Vec::new())) {
                        Node::Alternation(nodes) => nodes,
                        other => vec![other],
                    };
                match definition {
                    Node::Alternation(nodes) => alternatives.extend(nodes),
                    other => alternatives.push(other),
                }
                rule.definition = Node::Alternation(alternatives);
                index
            }
            (Some(_), false) => {
                return Err(parser.error_at(
                    &name_token,
                    &format!("the rule `{name}` is already defined; use `=/` to add to it"),
                ));
            }
            (None, true) => {
                return Err(parser.error_at(
                    &name_token,
                    &format!("`=/` adds to the rule `{name}`, which is not defined before it"),
                ));
            }
        };
        starts.push((name_token.line, index));
    }
    for comment in comments {
        match starts.iter().rev().find(|(line, _)| *line <= comment.line) {
            Some((_, index)) => set.rules[*index].comments.push(comment),
            None => set.loose_comments.push(comment),
        }
    }
    Ok(set)
}

#[derive(Clone, Debug)]
struct Token {
    line: usize,
    column: usize,
    kind: Kind,
}

#[derive(Clone, Debug)]
enum Kind {
    Name(String),
    Defined,
    Incremental,
    Slash,
    Open,
    Close,
    OptionOpen,
    OptionClose,
    Repeat { min: u64, max: Option<u64> },
    Element(Node),
}

fn lex(lines: &[(usize, &str)]) -> Result<(Vec<Token>, Vec<Comment>), Error> {
    let mut tokens = Vec::new();
    let mut comments = Vec::new();
    for (line, text) in lines {
        let chars: Vec<char> = text.chars().collect();
        let mut at = 0;
        while at < chars.len() {
            let c = chars[at];
            let column = at + 1;
            let error = |column: usize, message: &str| Error::Syntax {
                line: *line,
                column,
                message: message.to_string(),
            };
            let mut push = |kind: Kind| {
                tokens.push(Token {
                    line: *line,
                    column,
                    kind,
                })
            };
            match c {
                ' ' | '\t' => at += 1,
                ';' => {
                    let rest: String = chars[at + 1..].iter().collect();
                    comments.push(Comment {
                        line: *line,
                        text: rest.trim().to_string(),
                    });
                    at = chars.len();
                }
                '=' => {
                    if chars.get(at + 1) == Some(&'/') {
                        push(Kind::Incremental);
                        at += 2;
                    } else {
                        push(Kind::Defined);
                        at += 1;
                    }
                }
                '/' => {
                    push(Kind::Slash);
                    at += 1;
                }
                '(' => {
                    push(Kind::Open);
                    at += 1;
                }
                ')' => {
                    push(Kind::Close);
                    at += 1;
                }
                '[' => {
                    push(Kind::OptionOpen);
                    at += 1;
                }
                ']' => {
                    push(Kind::OptionClose);
                    at += 1;
                }
                '"' => {
                    let (text, next) = quoted(&chars, at).ok_or_else(|| {
                        error(column, "a quoted string is not closed on its line")
                    })?;
                    push(Kind::Element(Node::CharVal {
                        text,
                        case_sensitive: false,
                    }));
                    at = next;
                }
                '<' => {
                    let close = chars[at + 1..]
                        .iter()
                        .position(|&c| c == '>')
                        .ok_or_else(|| error(column, "a prose value is not closed on its line"))?;
                    let text: String = chars[at + 1..at + 1 + close].iter().collect();
                    if let Some(bad) = text.chars().position(|c| !(' '..='~').contains(&c)) {
                        return Err(error(
                            column + 1 + bad,
                            "a prose value holds a character outside %x20-7E",
                        ));
                    }
                    push(Kind::Element(Node::Prose(text)));
                    at += close + 2;
                }
                '%' => {
                    let (node, next) =
                        percent(&chars, at).map_err(|(at, message)| Error::Syntax {
                            line: *line,
                            column: at + 1,
                            message,
                        })?;
                    push(Kind::Element(node));
                    at = next;
                }
                '0'..='9' | '*' => {
                    let too_large = || error(column, "a repetition count does not fit in 64 bits");
                    let (min, mut next) = digits(&chars, at, 10);
                    let (min, max) = if chars.get(next) == Some(&'*') {
                        let (max, after) = digits(&chars, next + 1, 10);
                        next = after;
                        let min = match min {
                            Digits::Absent => 0,
                            Digits::Value(min) => min,
                            Digits::TooLarge => return Err(too_large()),
                        };
                        let max = match max {
                            Digits::Absent => None,
                            Digits::Value(max) => Some(max),
                            Digits::TooLarge => return Err(too_large()),
                        };
                        (min, max)
                    } else {
                        match min {
                            Digits::Value(min) => (min, Some(min)),
                            _ => return Err(too_large()),
                        }
                    };
                    if max.is_some_and(|max| max < min) {
                        return Err(error(column, "a repetition's maximum is below its minimum"));
                    }
                    push(Kind::Repeat { min, max });
                    at = next;
                }
                c if c.is_ascii_alphabetic() => {
                    let length = chars[at..]
                        .iter()
                        .position(|c| !(c.is_ascii_alphanumeric() || *c == '-'))
                        .unwrap_or(chars.len() - at);
                    push(Kind::Name(chars[at..at + length].iter().collect()));
                    at += length;
                }
                c => return Err(error(column, &format!("unexpected character `{c}`"))),
            }
        }
    }
    Ok((tokens, comments))
}

/// The quoted string starting at `at` and the index after its closing quote.
fn quoted(chars: &[char], at: usize) -> Option<(String, usize)> {
    let close = chars[at + 1..].iter().position(|&c| c == '"')?;
    let text: String = chars[at + 1..at + 1 + close].iter().collect();
    text.chars()
        .all(|c| (' '..='~').contains(&c))
        .then_some((text, at + close + 2))
}

/// A run of digits read as a number.
enum Digits {
    Absent,
    TooLarge,
    Value(u64),
}

/// The digits of `radix` starting at `at` and the index after them.
fn digits(chars: &[char], at: usize, radix: u32) -> (Digits, usize) {
    let length = chars[at.min(chars.len())..]
        .iter()
        .position(|c| !c.is_digit(radix))
        .unwrap_or(chars.len().saturating_sub(at));
    if length == 0 {
        return (Digits::Absent, at);
    }
    let text: String = chars[at..at + length].iter().collect();
    let value = match u64::from_str_radix(&text, radix) {
        Ok(value) => Digits::Value(value),
        Err(_) => Digits::TooLarge,
    };
    (value, at + length)
}

/// A num-val or an RFC 7405 `%s` / `%i` char-val starting at the `%` at `at`.
fn percent(chars: &[char], at: usize) -> Result<(Node, usize), (usize, String)> {
    let base = chars.get(at + 1).map(char::to_ascii_lowercase);
    let radix = match base {
        Some('b') => 2,
        Some('d') => 10,
        Some('x') => 16,
        Some(sensitivity @ ('s' | 'i')) => {
            if chars.get(at + 2) != Some(&'"') {
                return Err((
                    at + 2,
                    format!("expected a quoted string after `%{sensitivity}`"),
                ));
            }
            let (text, next) = quoted(chars, at + 2).ok_or((
                at + 2,
                "a quoted string is not closed on its line".to_string(),
            ))?;
            return Ok((
                Node::CharVal {
                    text,
                    case_sensitive: sensitivity == 's',
                },
                next,
            ));
        }
        _ => {
            return Err((
                at + 1,
                "expected `b`, `d`, `x`, `s` or `i` after `%`".to_string(),
            ));
        }
    };
    let value = |at: usize| -> Result<(u64, usize), (usize, String)> {
        match digits(chars, at, radix) {
            (Digits::Value(value), next) => Ok((value, next)),
            (Digits::TooLarge, _) => {
                Err((at, "a numeric value does not fit in 64 bits".to_string()))
            }
            (Digits::Absent, _) => Err((at, format!("expected a base-{radix} digit"))),
        }
    };
    let (first, mut next) = value(at + 2)?;
    match chars.get(next) {
        Some('-') => {
            let (high, after) = value(next + 1)?;
            if high < first {
                return Err((
                    next + 1,
                    "a range's upper bound is below its lower".to_string(),
                ));
            }
            Ok((Node::NumRange { low: first, high }, after))
        }
        Some('.') => {
            let mut values = vec![first];
            while chars.get(next) == Some(&'.') {
                let (value, after) = value(next + 1)?;
                values.push(value);
                next = after;
            }
            Ok((Node::NumSequence(values), next))
        }
        _ => Ok((Node::NumSequence(vec![first]), next)),
    }
}

struct Parser {
    tokens: Vec<Token>,
    next: usize,
    end: (usize, usize),
}

impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.next)
    }

    /// Whether the next token starts a rule: a rulename followed by `=` or `=/`.
    fn at_rule_start(&self) -> bool {
        matches!(self.peek().map(|token| &token.kind), Some(Kind::Name(_)))
            && matches!(
                self.tokens.get(self.next + 1).map(|token| &token.kind),
                Some(Kind::Defined | Kind::Incremental)
            )
    }

    fn error_at(&self, token: &Token, message: &str) -> Error {
        Error::Syntax {
            line: token.line,
            column: token.column,
            message: message.to_string(),
        }
    }

    fn error_here(&self, message: &str) -> Error {
        match self.peek() {
            Some(token) => self.error_at(token, message),
            None => Error::Syntax {
                line: self.end.0,
                column: self.end.1,
                message: message.to_string(),
            },
        }
    }

    fn alternation(&mut self) -> Result<Node, Error> {
        let mut alternatives = vec![self.concatenation()?];
        while matches!(self.peek().map(|token| &token.kind), Some(Kind::Slash)) {
            self.next += 1;
            alternatives.push(self.concatenation()?);
        }
        Ok(if alternatives.len() == 1 {
            alternatives.remove(0)
        } else {
            Node::Alternation(alternatives)
        })
    }

    fn concatenation(&mut self) -> Result<Node, Error> {
        let mut items = Vec::new();
        while !self.at_rule_start()
            && matches!(
                self.peek().map(|token| &token.kind),
                Some(
                    Kind::Name(_)
                        | Kind::Open
                        | Kind::OptionOpen
                        | Kind::Repeat { .. }
                        | Kind::Element(_)
                )
            )
        {
            items.push(self.repetition()?);
        }
        match items.len() {
            0 => Err(self.error_here("expected an element")),
            1 => Ok(items.remove(0)),
            _ => Ok(Node::Concatenation(items)),
        }
    }

    fn repetition(&mut self) -> Result<Node, Error> {
        let Some(Token {
            kind: Kind::Repeat { min, max },
            ..
        }) = self.peek().cloned()
        else {
            return self.element();
        };
        let repeat = self.tokens[self.next].clone();
        self.next += 1;
        if self.at_rule_start() || !self.at_element() {
            return Err(self.error_at(&repeat, "expected an element after the repetition"));
        }
        Ok(Node::Repetition {
            min,
            max,
            node: Box::new(self.element()?),
        })
    }

    fn at_element(&self) -> bool {
        matches!(
            self.peek().map(|token| &token.kind),
            Some(Kind::Name(_) | Kind::Open | Kind::OptionOpen | Kind::Element(_))
        )
    }

    fn element(&mut self) -> Result<Node, Error> {
        let Some(token) = self.peek().cloned() else {
            return Err(self.error_here("expected an element"));
        };
        self.next += 1;
        match token.kind {
            Kind::Name(name) => Ok(Node::Reference(name)),
            Kind::Element(node) => Ok(node),
            Kind::Open => {
                let inner = self.alternation()?;
                self.close(&token, |kind| matches!(kind, Kind::Close), "`)`")?;
                Ok(inner)
            }
            Kind::OptionOpen => {
                let inner = self.alternation()?;
                self.close(&token, |kind| matches!(kind, Kind::OptionClose), "`]`")?;
                Ok(Node::Repetition {
                    min: 0,
                    max: Some(1),
                    node: Box::new(inner),
                })
            }
            _ => Err(self.error_at(&token, "expected an element")),
        }
    }

    fn close(
        &mut self,
        open: &Token,
        is_close: fn(&Kind) -> bool,
        what: &str,
    ) -> Result<(), Error> {
        match self.peek() {
            Some(token) if is_close(&token.kind) => {
                self.next += 1;
                Ok(())
            }
            _ => Err(self.error_at(
                open,
                &format!("this group is not closed with {what} before the rule ends"),
            )),
        }
    }
}
