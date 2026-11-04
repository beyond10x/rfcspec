#![forbid(unsafe_code)]
//! The `rfcspec/1` types shared by every stage: a profile and its lock (S0), the normalized document
//! (S1), the mechanical catalogue (S2) and the reviewed overlay (S4). Every type serializes in a
//! fixed field order, so the same value always writes the same bytes.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

/// The file name of a profile.
pub const PROFILE_FILE: &str = "profile.yaml";
/// The file name of a profile's lock.
pub const LOCK_FILE: &str = "rfcspec.lock";
/// The file name of a profile's overlay.
pub const OVERLAY_FILE: &str = "overlay.yaml";

/// A profile: the RFCs and the sections of each that are in scope.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub rfcs: Vec<ProfileRfc>,
}

/// One RFC of a profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProfileRfc {
    pub number: u32,
    pub title: String,
    /// The sections in scope, by number or, for an RFC without numbered sections, by title.
    pub sections: Vec<String>,
}

/// The lock: one entry per fetched source file, with its SHA-256.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Lock {
    pub format: String,
    pub sources: Vec<LockEntry>,
}

/// The lock format.
pub const LOCK_FORMAT: &str = "rfcspec-lock/1";

/// One locked source file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockEntry {
    pub rfc: u32,
    /// The file, relative to the profile directory.
    pub file: String,
    pub sha256: String,
}

impl Lock {
    /// The entry for one RFC.
    pub fn entry(&self, rfc: u32) -> Option<&LockEntry> {
        self.sources.iter().find(|entry| entry.rfc == rfc)
    }

    /// Insert or replace the entry for an RFC, keeping the entries ordered by RFC number.
    pub fn upsert(&mut self, entry: LockEntry) {
        self.sources.retain(|existing| existing.rfc != entry.rfc);
        self.sources.push(entry);
        self.sources.sort_by_key(|entry| entry.rfc);
    }
}

/// An inclusive, 1-based range of source lines.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn contains(&self, line: usize) -> bool {
        self.start <= line && line <= self.end
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "lines {}-{}", self.start, self.end)
    }
}

/// The normalized document of one RFC.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Document {
    pub rfc: u32,
    pub sha256: String,
    /// Source lines removed as page furniture (headers, footers, form feeds).
    pub furniture: Vec<usize>,
    pub blocks: Vec<Block>,
}

/// What a block of the document is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockKind {
    Heading,
    Paragraph,
    Figure,
}

/// One heading, paragraph or indented figure with the source lines it came from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub kind: BlockKind,
    pub span: Span,
    /// One entry per source line of the span. Paragraph and heading lines are trimmed; figure
    /// lines keep their indentation, and blank lines inside a figure are empty strings.
    pub lines: Vec<String>,
}

impl Block {
    /// The block's text: paragraph lines joined by single spaces, figure lines by newlines.
    pub fn text(&self) -> String {
        match self.kind {
            BlockKind::Figure => self.lines.join("\n"),
            BlockKind::Heading | BlockKind::Paragraph => self.lines.join(" "),
        }
    }
}

/// The mechanical catalogue of one RFC (S2).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Catalogue {
    pub format: String,
    pub rfc: u32,
    pub sha256: String,
    /// Sentences with a BCP 14 keyword in capitals.
    pub requirements: Vec<Bcp14Requirement>,
    /// Sentences with a lowercase `must`, `should` or `may`, for review.
    pub prose_candidates: Vec<ProseCandidate>,
    pub dialogue_steps: Vec<DialogueStep>,
    pub examples: Vec<NumericExample>,
}

/// The catalogue format.
pub const CATALOGUE_FORMAT: &str = "rfcspec/1";

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bcp14Requirement {
    pub id: String,
    pub keyword: String,
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProseCandidate {
    pub id: String,
    pub keyword: String,
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogueStep {
    pub id: String,
    /// `S` (server) or `U` (user).
    pub party: String,
    pub text: String,
    pub span: Span,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NumericExample {
    pub id: String,
    pub seconds: i64,
    pub text: String,
    pub span: Span,
}

/// The reviewed overlay of one RFC in a profile (S4): the only authored file besides the profile.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Overlay {
    pub format: String,
    pub rfc: u32,
    pub requirements: Vec<Requirement>,
    pub layouts: Vec<Layout>,
    pub machines: Vec<Machine>,
    pub findings: Vec<Finding>,
}

/// The overlay format.
pub const OVERLAY_FORMAT: &str = "rfcspec-overlay/1";

/// An accepted requirement.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Requirement {
    pub id: String,
    /// `bcp14-xml`, `capitals-text`, `abnf-comment` or `prose`.
    pub source: String,
    /// The level as the reviewer reads it: `MUST`, `SHOULD`, `MAY`, ...
    pub level: String,
    /// The transports it applies to; empty means every transport.
    #[serde(default)]
    pub transports: Vec<String>,
    pub quote: String,
    pub span: Span,
    pub status: Status,
    /// The observation that checks it against a live target, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe: Option<String>,
}

/// A requirement's status (design section 5).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Status {
    /// Satisfied by a model element: `<machine>.<participant>.<transition>` or `layout.<name>`.
    Satisfied(String),
    Unsupported(String),
    NotApplicable(String),
}

/// A binary layout: fields with bit widths.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub name: String,
    pub quote: String,
    pub span: Span,
    /// The instant the value counts from, `YYYY-MM-DD` at 00:00 GMT.
    pub epoch: String,
    pub fields: Vec<Field>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub bits: u32,
    pub signed: bool,
    /// `big-endian` or `little-endian`.
    pub order: String,
}

/// A review finding about a catalogue item.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    pub id: String,
    /// The catalogue item it is about, such as an example id.
    pub item: String,
    pub quote: String,
    pub span: Span,
    pub finding: String,
}

/// A reviewed machine for one transport, lowered to one `ess-protospec/1` model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Machine {
    pub name: String,
    pub transport: String,
    pub messages: Vec<Message>,
    pub participants: Vec<Participant>,
    pub properties: Vec<EventCount>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Message {
    pub name: String,
    /// Field name to type (`Integer`, `String`, `Boolean`).
    #[serde(default)]
    pub fields: BTreeMap<String, String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Participant {
    pub name: String,
    /// The dialogue party, `S` or `U`.
    pub party: String,
    pub initial: String,
    pub states: Vec<String>,
    /// Zero-field inputs the environment offers.
    #[serde(default)]
    pub inputs: Vec<String>,
    pub transitions: Vec<Transition>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Transition {
    /// The requirement and dialogue-step ids it implements; they lead its lowered name.
    pub implements: Vec<String>,
    pub label: String,
    pub from: String,
    pub to: String,
    pub on: Trigger,
    #[serde(default)]
    pub effects: Vec<Effect>,
}

impl Transition {
    /// The lowered transition name: the ids it implements, then its label.
    pub fn name(&self) -> String {
        let mut parts = self.implements.clone();
        parts.push(self.label.clone());
        parts.join("-")
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Trigger {
    Input(String),
    Receive(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Effect {
    Send(Send),
    Emit(String),
    Close(bool),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Send {
    pub message: String,
    #[serde(default)]
    pub payload: BTreeMap<String, i64>,
}

/// An `event_count` property: the participant emits the event at most `max` times.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventCount {
    pub name: String,
    pub participant: String,
    pub event: String,
    pub max: u32,
}

/// One generated file: its path relative to an output directory, and its bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Generated {
    pub file: String,
    pub text: String,
}

/// Write generated files into `out`.
pub fn write_generated(out: &Path, files: &[Generated]) -> Result<()> {
    for file in files {
        let path = out.join(&file.file);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, &file.text).with_context(|| format!("writing {}", path.display()))?;
    }
    Ok(())
}

/// The files under `out` that are missing or differ from `files`.
pub fn drift(out: &Path, files: &[Generated]) -> Vec<std::path::PathBuf> {
    files
        .iter()
        .map(|file| (out.join(&file.file), file))
        .filter(|(path, file)| fs::read_to_string(path).ok().as_deref() != Some(file.text.as_str()))
        .map(|(path, _)| path)
        .collect()
}

/// Read a YAML file.
pub fn read_yaml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
    from_yaml(&text).with_context(|| format!("parsing {}", path.display()))
}

/// Parse YAML; an enum is a single-key map (`{satisfied: layout.time}`), not a YAML tag.
pub fn from_yaml<T: DeserializeOwned>(text: &str) -> Result<T> {
    Ok(serde_yaml::with::singleton_map_recursive::deserialize(
        serde_yaml::Deserializer::from_str(text),
    )?)
}

/// Serialize a value as YAML, enums as single-key maps.
pub fn to_yaml<T: Serialize>(value: &T) -> Result<String> {
    let mut out = Vec::new();
    serde_yaml::with::singleton_map_recursive::serialize(
        value,
        &mut serde_yaml::Serializer::new(&mut out),
    )?;
    Ok(String::from_utf8(out)?)
}

/// Serialize a value as pretty JSON with a final newline.
pub fn to_json<T: Serialize>(value: &T) -> Result<String> {
    let mut text = serde_json::to_string_pretty(value)?;
    text.push('\n');
    Ok(text)
}

/// Collapse every run of whitespace to one space and trim the ends; quotes are compared this way,
/// because a line break in the source is layout, not text.
pub fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lock_upsert_keeps_one_entry_per_rfc_in_order() {
        let mut lock = Lock {
            format: LOCK_FORMAT.into(),
            sources: vec![],
        };
        for (rfc, sha) in [(868, "a"), (6455, "b"), (868, "c")] {
            lock.upsert(LockEntry {
                rfc,
                file: format!("sources/rfc{rfc}.txt"),
                sha256: sha.into(),
            });
        }
        assert_eq!(lock.sources.len(), 2);
        assert_eq!(lock.entry(868).unwrap().sha256, "c");
        assert_eq!(lock.sources[1].rfc, 6455);
    }

    #[test]
    fn effects_and_statuses_read_as_single_key_maps() {
        let effects: Vec<Effect> = from_yaml(
            "- {send: {message: time, payload: {seconds: 1}}}\n- {emit: sent}\n- {close: true}\n",
        )
        .unwrap();
        assert_eq!(effects.len(), 3);
        assert_eq!(
            from_yaml::<Vec<Effect>>(&to_yaml(&effects).unwrap()).unwrap(),
            effects
        );
        let status: Status = from_yaml("{satisfied: layout.time}").unwrap();
        assert_eq!(status, Status::Satisfied("layout.time".into()));
    }
}
