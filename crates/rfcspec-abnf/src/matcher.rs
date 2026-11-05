//! Matching a complete input against one rule, octet by octet, with backtracking over every end
//! position a node can reach.
//!
//! The end positions of a rule reference are memoized per rule and offset, so a grammar whose
//! repetitions each repeat a reference does polynomial work in the input's length. Repetitions
//! nested inline within one rule (`*(*(*"a"))`) are recomputed at every offset they are tried
//! from: their work grows with the input's length raised to the nesting depth, and there is no
//! bound but the caller's input. Left-recursive closures are refused before matching, and
//! nesting deeper than [`MATCH_DEPTH_LIMIT`] stops the match with [`MatchError::TooDeep`].

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fmt;

use crate::{Node, Rule, RuleImport, RuleSet, core};

/// How many nodes deep one match may nest (each group, repetition, alternation and reference
/// is one level) before it stops with [`MatchError::TooDeep`] instead of exhausting the stack.
pub const MATCH_DEPTH_LIMIT: usize = 1000;

/// Why [`matches`] did not accept an input.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MatchError {
    /// The input is not a complete `rule`: `offset` is the furthest octet any alternative
    /// failed at, or where the longest match stopped short of the input's end.
    Invalid { rule: String, offset: usize },
    /// The rule's closure holds a rule another RFC defines; it is not resolved here.
    Import(RuleImport),
    /// The rule's closure references a name neither the set nor the core rules define.
    Unresolved(String),
    /// The rule's closure holds a prose-val that is not an import.
    Prose { rule: String, text: String },
    /// The rule's closure holds a rule that reaches itself without consuming input; it names
    /// that rule.
    LeftRecursive(String),
    /// Matching `rule` nested deeper than `limit` nodes.
    TooDeep { rule: String, limit: usize },
}

impl fmt::Display for MatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatchError::Invalid { rule, offset } => {
                write!(f, "the input is not a `{rule}` at offset {offset}")
            }
            MatchError::Import(import) => write!(
                f,
                "the rule `{}` is imported from RFC {} and not resolved",
                import.rule, import.rfc
            ),
            MatchError::Unresolved(name) => write!(f, "the rule `{name}` is not defined"),
            MatchError::Prose { rule, text } => {
                write!(f, "the rule `{rule}` holds the prose value <{text}>")
            }
            MatchError::LeftRecursive(rule) => write!(
                f,
                "the rule `{rule}` reaches itself without consuming input"
            ),
            MatchError::TooDeep { rule, limit } => {
                write!(f, "matching `{rule}` nests deeper than {limit} levels")
            }
        }
    }
}

impl std::error::Error for MatchError {}

/// Match the whole of `input` against `rule`, resolved in `set` and then among the core rules.
/// Gives the octets consumed (the input's length), or refuses a rule whose closure holds an
/// import, a prose-val or an unresolved name, naming the first one met (a leaf under a
/// repetition of at most zero is never matched and does not count), and a closure in which a
/// rule reaches itself without consuming input, naming that rule.
pub fn matches(set: &RuleSet, rule: &str, input: &[u8]) -> Result<usize, MatchError> {
    let (top, in_core) =
        resolve(set, false, rule).ok_or_else(|| MatchError::Unresolved(rule.to_string()))?;
    let mut closure = Closure::new();
    check_closure(set, top, in_core, &mut closure)?;
    left_recursion(set, &closure, key(top, in_core))?;
    let mut matcher = Matcher {
        set,
        input,
        furthest: 0,
        memo: HashMap::new(),
        depth: 0,
        too_deep: false,
    };
    let ends = matcher.ends(&top.definition, 0, in_core);
    if matcher.too_deep {
        return Err(MatchError::TooDeep {
            rule: top.name.clone(),
            limit: MATCH_DEPTH_LIMIT,
        });
    }
    if ends.contains(&input.len()) {
        return Ok(input.len());
    }
    let offset = ends.last().copied().unwrap_or(0).max(matcher.furthest);
    Err(MatchError::Invalid {
        rule: top.name.clone(),
        offset,
    })
}

/// The rule a name resolves to, and whether it is a core rule: a rule of the set first (unless
/// the reference is made from a core rule), else a core rule.
fn resolve<'a>(set: &'a RuleSet, in_core: bool, name: &str) -> Option<(&'a Rule, bool)> {
    let own = if in_core { None } else { set.rule(name) };
    own.map(|rule| (rule, false))
        .or_else(|| core::rules().rule(name).map(|rule| (rule, true)))
}

/// A rule's identity in a match: whether it is a core rule, and its lower-cased name.
type Key = (bool, String);

/// The rules a match can reach, by key, in the order they were first reached.
type Closure<'a> = Vec<(Key, &'a Rule)>;

fn key(rule: &Rule, in_core: bool) -> Key {
    (in_core, rule.name.to_ascii_lowercase())
}

/// The references and prose-vals of a node in order, leaving out those under a repetition of
/// at most zero, which is never matched.
fn live_leaves<'a>(node: &'a Node, leaves: &mut Vec<&'a Node>) {
    match node {
        Node::Reference(_) | Node::Prose(_) => leaves.push(node),
        Node::Alternation(nodes) | Node::Concatenation(nodes) => {
            for node in nodes {
                live_leaves(node, leaves);
            }
        }
        Node::Repetition { max: Some(0), .. } => {}
        Node::Repetition { node, .. } => live_leaves(node, leaves),
        _ => {}
    }
}

fn check_closure<'a>(
    set: &'a RuleSet,
    rule: &'a Rule,
    in_core: bool,
    closure: &mut Closure<'a>,
) -> Result<(), MatchError> {
    let rule_key = key(rule, in_core);
    if closure.iter().any(|(seen, _)| *seen == rule_key) {
        return Ok(());
    }
    closure.push((rule_key, rule));
    let mut leaves = Vec::new();
    live_leaves(&rule.definition, &mut leaves);
    for leaf in leaves {
        match leaf {
            Node::Reference(name) => {
                let (target, target_core) = resolve(set, in_core, name)
                    .ok_or_else(|| MatchError::Unresolved(name.clone()))?;
                check_closure(set, target, target_core, closure)?;
            }
            Node::Prose(text) => {
                return Err(match RuleImport::from_prose(text) {
                    Some(import) => MatchError::Import(import),
                    None => MatchError::Prose {
                        rule: rule.name.clone(),
                        text: text.clone(),
                    },
                });
            }
            _ => {}
        }
    }
    Ok(())
}

/// Refuse a closure in which a rule reaches itself through references in leftmost position,
/// that is with only nullable elements before them, naming the first such rule a depth-first
/// walk from `top` meets again.
fn left_recursion(set: &RuleSet, closure: &Closure<'_>, top: Key) -> Result<(), MatchError> {
    let mut nullable: HashSet<Key> = HashSet::new();
    loop {
        let before = nullable.len();
        for (rule_key, rule) in closure {
            if !nullable.contains(rule_key)
                && is_nullable(set, &rule.definition, rule_key.0, &nullable)
            {
                nullable.insert(rule_key.clone());
            }
        }
        if nullable.len() == before {
            break;
        }
    }
    let mut edges: HashMap<Key, Vec<Key>> = HashMap::new();
    for (rule_key, rule) in closure {
        let mut firsts = Vec::new();
        leftmost(set, &rule.definition, rule_key.0, &nullable, &mut firsts);
        edges.insert(rule_key.clone(), firsts);
    }
    let names: HashMap<&Key, &str> = closure
        .iter()
        .map(|(rule_key, rule)| (rule_key, rule.name.as_str()))
        .collect();
    let mut on_path = Vec::new();
    let mut done = HashSet::new();
    match cycle(&top, &edges, &mut on_path, &mut done) {
        Some(found) => Err(MatchError::LeftRecursive(
            names.get(&found).copied().unwrap_or(&found.1).to_string(),
        )),
        None => Ok(()),
    }
}

fn cycle(
    at: &Key,
    edges: &HashMap<Key, Vec<Key>>,
    on_path: &mut Vec<Key>,
    done: &mut HashSet<Key>,
) -> Option<Key> {
    if on_path.contains(at) {
        return Some(at.clone());
    }
    if done.contains(at) {
        return None;
    }
    on_path.push(at.clone());
    for next in edges.get(at).into_iter().flatten() {
        if let Some(found) = cycle(next, edges, on_path, done) {
            return Some(found);
        }
    }
    on_path.pop();
    done.insert(at.clone());
    None
}

fn reference_key(set: &RuleSet, name: &str, in_core: bool) -> Option<Key> {
    resolve(set, in_core, name).map(|(rule, rule_core)| key(rule, rule_core))
}

/// Whether a node matches the empty input, given the rules already known to.
fn is_nullable(set: &RuleSet, node: &Node, in_core: bool, nullable: &HashSet<Key>) -> bool {
    match node {
        Node::CharVal { text, .. } => text.is_empty(),
        Node::NumSequence(values) => values.is_empty(),
        Node::NumRange { .. } | Node::Prose(_) => false,
        Node::Alternation(nodes) => nodes
            .iter()
            .any(|node| is_nullable(set, node, in_core, nullable)),
        Node::Concatenation(nodes) => nodes
            .iter()
            .all(|node| is_nullable(set, node, in_core, nullable)),
        Node::Repetition { min, node, .. } => {
            *min == 0 || is_nullable(set, node, in_core, nullable)
        }
        Node::Reference(name) => {
            reference_key(set, name, in_core).is_some_and(|target| nullable.contains(&target))
        }
    }
}

/// The rules a node can call before it consumes any input.
fn leftmost(
    set: &RuleSet,
    node: &Node,
    in_core: bool,
    nullable: &HashSet<Key>,
    firsts: &mut Vec<Key>,
) {
    match node {
        Node::Reference(name) => firsts.extend(reference_key(set, name, in_core)),
        Node::Alternation(nodes) => {
            for node in nodes {
                leftmost(set, node, in_core, nullable, firsts);
            }
        }
        Node::Concatenation(nodes) => {
            for node in nodes {
                leftmost(set, node, in_core, nullable, firsts);
                if !is_nullable(set, node, in_core, nullable) {
                    break;
                }
            }
        }
        Node::Repetition { max: Some(0), .. } => {}
        Node::Repetition { node, .. } => leftmost(set, node, in_core, nullable, firsts),
        _ => {}
    }
}

struct Matcher<'a> {
    set: &'a RuleSet,
    input: &'a [u8],
    furthest: usize,
    memo: HashMap<(bool, String, usize), Option<BTreeSet<usize>>>,
    depth: usize,
    too_deep: bool,
}

impl Matcher<'_> {
    fn fail(&mut self, at: usize) -> BTreeSet<usize> {
        self.furthest = self.furthest.max(at);
        BTreeSet::new()
    }

    /// The end positions of `node` from `at`; none once the match nests too deep.
    fn ends(&mut self, node: &Node, at: usize, in_core: bool) -> BTreeSet<usize> {
        if self.too_deep {
            return BTreeSet::new();
        }
        if self.depth >= MATCH_DEPTH_LIMIT {
            self.too_deep = true;
            return BTreeSet::new();
        }
        self.depth += 1;
        let ends = self.node_ends(node, at, in_core);
        self.depth -= 1;
        ends
    }

    fn node_ends(&mut self, node: &Node, at: usize, in_core: bool) -> BTreeSet<usize> {
        match node {
            Node::CharVal {
                text,
                case_sensitive,
            } => {
                for (index, expected) in text.bytes().enumerate() {
                    let same = |actual: &u8| {
                        if *case_sensitive {
                            *actual == expected
                        } else {
                            actual.eq_ignore_ascii_case(&expected)
                        }
                    };
                    if !self.input.get(at + index).is_some_and(same) {
                        return self.fail(at + index);
                    }
                }
                BTreeSet::from([at + text.len()])
            }
            Node::NumRange { low, high } => match self.input.get(at) {
                Some(&byte) if (*low..=*high).contains(&u64::from(byte)) => {
                    BTreeSet::from([at + 1])
                }
                _ => self.fail(at),
            },
            Node::NumSequence(values) => {
                for (index, value) in values.iter().enumerate() {
                    if self.input.get(at + index).map(|&byte| u64::from(byte)) != Some(*value) {
                        return self.fail(at + index);
                    }
                }
                BTreeSet::from([at + values.len()])
            }
            Node::Alternation(nodes) => {
                let mut ends = BTreeSet::new();
                for node in nodes {
                    ends.extend(self.ends(node, at, in_core));
                }
                ends
            }
            Node::Concatenation(nodes) => {
                let mut current = BTreeSet::from([at]);
                for node in nodes {
                    let mut next = BTreeSet::new();
                    for position in current {
                        next.extend(self.ends(node, position, in_core));
                    }
                    current = next;
                    if current.is_empty() {
                        break;
                    }
                }
                current
            }
            Node::Repetition { min, max, node } => self.repetition(*min, *max, node, at, in_core),
            Node::Reference(name) => self.reference(name, at, in_core),
            Node::Prose(_) => BTreeSet::new(),
        }
    }

    fn repetition(
        &mut self,
        min: u64,
        max: Option<u64>,
        node: &Node,
        at: usize,
        in_core: bool,
    ) -> BTreeSet<usize> {
        let mut result = BTreeSet::new();
        let mut current = BTreeSet::from([at]);
        if min == 0 {
            result.insert(at);
        }
        let mut count: u64 = 0;
        while !current.is_empty() && max.is_none_or(|max| count < max) {
            let mut next = BTreeSet::new();
            for position in &current {
                next.extend(self.ends(node, *position, in_core));
            }
            count += 1;
            if count < min {
                if next != current {
                    current = next;
                    continue;
                }
                count = min;
            }
            let fresh: BTreeSet<usize> = next.difference(&result).copied().collect();
            result.extend(next);
            current = fresh;
        }
        result
    }

    fn reference(&mut self, name: &str, at: usize, in_core: bool) -> BTreeSet<usize> {
        let Some((rule, rule_core)) = resolve(self.set, in_core, name) else {
            return BTreeSet::new();
        };
        let key = (rule_core, rule.name.to_ascii_lowercase(), at);
        match self.memo.get(&key) {
            Some(Some(ends)) => return ends.clone(),
            Some(None) => return BTreeSet::new(),
            None => {}
        }
        self.memo.insert(key.clone(), None);
        let ends = self.ends(&rule.definition, at, rule_core);
        self.memo.insert(key, Some(ends.clone()));
        ends
    }
}
