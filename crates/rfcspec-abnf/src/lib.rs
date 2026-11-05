#![forbid(unsafe_code)]
//! RFC ABNF (design section 13): the dialects a grammar is written in, the RFC 5234 and RFC 7405
//! reader, reference resolution against a rule set, the RFC 5234 Appendix B core rules and RFC
//! imports, and a matcher of a complete input against one rule.

pub mod bits;
mod core;
mod matcher;
mod parse;
pub mod rfc2616;

use std::fmt;

pub use matcher::{MATCH_DEPTH_LIMIT, MatchError, matches};

/// The ABNF dialect a grammar is written in (ESS `rfcspec.catalogue.AbnfDialect`, design
/// section 8.3): RFC 5234, RFC 2616 with its `#rule` lists and implied whitespace, or the
/// bit-level ABNF of RFC 6455 section 5.2.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Dialect {
    Rfc5234,
    Rfc2616,
    Bits,
}

impl Dialect {
    /// Every dialect, in declaration order.
    pub const ALL: [Dialect; 3] = [Dialect::Rfc5234, Dialect::Rfc2616, Dialect::Bits];

    /// The dialect's wire name: `rfc5234`, `rfc2616` or `bits`.
    pub fn wire(self) -> &'static str {
        match self {
            Dialect::Rfc5234 => "rfc5234",
            Dialect::Rfc2616 => "rfc2616",
            Dialect::Bits => "bits",
        }
    }

    /// The dialect a wire name names; `None` for any other name.
    pub fn from_wire(name: &str) -> Option<Dialect> {
        Dialect::ALL
            .into_iter()
            .find(|dialect| dialect.wire() == name)
    }
}

/// Parse figure lines, each with its 1-based source line number, as a rule set in `dialect`.
/// Blank lines are skipped; no figure is filtered, so a figure that does not parse is an error
/// naming its source line and column.
pub fn parse(dialect: Dialect, lines: &[(usize, &str)]) -> Result<RuleSet, Error> {
    match dialect {
        Dialect::Rfc5234 => parse::parse(lines),
        Dialect::Rfc2616 => rfc2616::parse(lines),
        Dialect::Bits => bits::parse(lines),
    }
}

/// Why a figure was not read as a rule set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// The text is not ABNF of the dialect: the 1-based source line and column, and what was
    /// expected there.
    Syntax {
        line: usize,
        column: usize,
        message: String,
    },
    /// The dialect has no reader yet.
    Unsupported { dialect: Dialect },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Syntax {
                line,
                column,
                message,
            } => write!(f, "line {line}, column {column}: {message}"),
            Error::Unsupported { dialect } => {
                write!(f, "the ABNF dialect `{}` is not supported", dialect.wire())
            }
        }
    }
}

impl std::error::Error for Error {}

/// The rules read from one or more figures, in the order they are first defined.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RuleSet {
    pub rules: Vec<Rule>,
    /// Comments on lines before the first rule.
    pub loose_comments: Vec<Comment>,
}

/// One rule: its name as first written, the source line of its first definition, its
/// definition (incremental alternatives `=/` folded into one alternation) and the comments on
/// the lines it spans, each on its own line.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Rule {
    pub name: String,
    pub line: usize,
    pub definition: Node,
    pub comments: Vec<Comment>,
}

/// A `;` comment: its source line and its text without the `;` and surrounding whitespace.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    pub line: usize,
    pub text: String,
}

/// One element of a rule's definition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Node {
    Alternation(Vec<Node>),
    Concatenation(Vec<Node>),
    /// `n*m`, `n` and `*`; an option `[x]` is `0*1`.
    Repetition {
        min: u64,
        max: Option<u64>,
        node: Box<Node>,
    },
    /// A rulename, as written.
    Reference(String),
    /// A quoted string; `case_sensitive` only for RFC 7405 `%s`.
    CharVal {
        text: String,
        case_sensitive: bool,
    },
    /// `%x30-39`: one value in the inclusive range.
    NumRange {
        low: u64,
        high: u64,
    },
    /// `%d13.10` or `%x41`: the values in order.
    NumSequence(Vec<u64>),
    /// `<...>`, the text between the brackets.
    Prose(String),
}

/// A rule another RFC defines, named by a prose-val `<rule, defined in [RFCn], ...>`
/// (`rfcspec.catalogue.RuleImport`).
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct RuleImport {
    pub rfc: u32,
    pub rule: String,
}

impl RuleImport {
    /// The import a prose-val's text names, if it has the form `rule, defined in [RFCn]`
    /// followed by nothing or by `, ...`.
    pub fn from_prose(text: &str) -> Option<RuleImport> {
        let (rule, rest) = text.split_once(',')?;
        let rule = rule.trim();
        if !parse::is_rulename(rule) {
            return None;
        }
        let rest = rest.trim_start().strip_prefix("defined in [RFC")?;
        let digits = rest
            .find(|c: char| !c.is_ascii_digit())
            .unwrap_or(rest.len());
        let rfc = rest[..digits].parse().ok()?;
        let after = rest[digits..].strip_prefix(']')?.trim_start();
        (after.is_empty() || after.starts_with(',')).then(|| RuleImport {
            rfc,
            rule: rule.to_string(),
        })
    }
}

impl RuleSet {
    /// The rule a name names in this set, compared without ASCII case as RFC 5234 does.
    pub fn rule(&self, name: &str) -> Option<&Rule> {
        self.rules
            .iter()
            .find(|rule| rule.name.eq_ignore_ascii_case(name))
    }

    /// The imports the set's prose-vals name, in rule order, each once.
    pub fn imports(&self) -> Vec<RuleImport> {
        let mut imports = Vec::new();
        for rule in &self.rules {
            visit(&rule.definition, &mut |node| {
                if let Node::Prose(text) = node
                    && let Some(import) = RuleImport::from_prose(text)
                    && !imports.contains(&import)
                {
                    imports.push(import);
                }
            });
        }
        imports
    }

    /// What the set leaves unresolved, in order of first occurrence, each once: every name
    /// referenced that neither the set nor the core rules define, and every prose-val that is
    /// not an import, written with its brackets (`<text>`).
    pub fn unresolved(&self) -> Vec<String> {
        let mut unresolved: Vec<String> = Vec::new();
        for rule in &self.rules {
            visit(&rule.definition, &mut |node| {
                let item = match node {
                    Node::Reference(name)
                        if self.rule(name).is_none() && core::rules().rule(name).is_none() =>
                    {
                        name.clone()
                    }
                    Node::Prose(text) if RuleImport::from_prose(text).is_none() => {
                        format!("<{text}>")
                    }
                    _ => return,
                };
                if !unresolved
                    .iter()
                    .any(|seen| seen.eq_ignore_ascii_case(&item))
                {
                    unresolved.push(item);
                }
            });
        }
        unresolved
    }
}

pub(crate) fn visit(node: &Node, f: &mut impl FnMut(&Node)) {
    f(node);
    match node {
        Node::Alternation(nodes) | Node::Concatenation(nodes) => {
            for node in nodes {
                visit(node, f);
            }
        }
        Node::Repetition { node, .. } => visit(node, f),
        _ => {}
    }
}
