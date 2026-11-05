#![forbid(unsafe_code)]
//! The `rfcspec/1` types shared by every stage: a profile and its lock (S0), the normalized document
//! (S1), the mechanical catalogue (S2) and the reviewed overlay (S4). Every type serializes in a
//! fixed field order, so the same value always writes the same bytes.

use std::{collections::BTreeMap, fs, path::Path};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

mod run;
mod semantics;

pub use run::{
    RUN_FORMAT, RUNS_DIR, RunReport, RunRow, Verdict, is_must_level, newest, overall, read_runs,
    sha256_hex, target_slug, utc,
};
pub use semantics::{
    Correlation, Delivery, Direction, Endpoint, Exchange, LayerSemantics, Layering, Lifetime,
    Multiplexing, Unit, wire,
};

/// The file name of a profile.
pub const PROFILE_FILE: &str = "profile.yaml";
/// The file name of a profile's lock.
pub const LOCK_FILE: &str = "rfcspec.lock";
/// The file name of a profile's overlay.
pub const OVERLAY_FILE: &str = "overlay.yaml";

/// A profile: the RFCs and the sections of each that are in scope, and the communication
/// semantics of each layer (design section 6).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Profile {
    pub name: String,
    pub rfcs: Vec<ProfileRfc>,
    /// One entry per layer; each transport is its own layer.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub semantics: Vec<LayerSemantics>,
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
    /// lines keep their indentation. Blank lines and page furniture inside a block (a paragraph
    /// or figure continued across a page break) are empty strings.
    pub lines: Vec<String>,
    /// The number of the numbered section the block lies in (`5.2`), absent before the first
    /// numbered heading and in an RFC without numbered sections.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

impl Block {
    /// The block's text: paragraph lines joined by single spaces, figure lines by newlines.
    /// Empty lines inside a paragraph (a page break) are skipped.
    pub fn text(&self) -> String {
        match self.kind {
            BlockKind::Figure => self.lines.join("\n"),
            BlockKind::Heading | BlockKind::Paragraph => self
                .lines
                .iter()
                .filter(|line| !line.is_empty())
                .map(String::as_str)
                .collect::<Vec<_>>()
                .join(" "),
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
    /// Bulleted lines that list octets (`0x81 0x05 ...`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub octet_examples: Vec<OctetExample>,
}

/// An example given as octets, such as RFC 6455's `*  0x82 0x7E 0x0100 [256 bytes of binary
/// data]`: the listed octets in order (a multi-octet token read big-endian), and the octets the
/// example elides.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct OctetExample {
    pub id: String,
    pub octets: Vec<u8>,
    pub elided: u64,
    pub text: String,
    pub span: Span,
}

/// The catalogue format.
pub const CATALOGUE_FORMAT: &str = "rfcspec/1";

/// A sentence with a BCP 14 keyword in capitals.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Bcp14Requirement {
    /// `R<section>-<ordinal>`, the ordinal counting the requirements of the section in document
    /// order (`R5.2-1`); `R-<ordinal>` outside every numbered section.
    pub id: String,
    /// The level: the keyword that occurs first in the sentence, the longest at that position.
    pub keyword: String,
    pub text: String,
    pub span: Span,
    /// `capitals-text` for a sentence of a paragraph, `abnf-comment` for a sentence of an ABNF
    /// comment inside a figure.
    pub source: String,
    /// The numbered section the sentence lies in.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

/// The requirement source of a sentence in a paragraph.
pub const CAPITALS_TEXT: &str = "capitals-text";
/// The requirement source of a sentence in an ABNF comment.
pub const ABNF_COMMENT: &str = "abnf-comment";

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
    /// The RFC's primary kind (design section 2.3), absent until classified.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<RfcKind>,
    pub requirements: Vec<Requirement>,
    pub layouts: Vec<Layout>,
    pub machines: Vec<Machine>,
    pub findings: Vec<Finding>,
}

/// The overlay format.
pub const OVERLAY_FORMAT: &str = "rfcspec-overlay/1";

/// The primary kind of an RFC (design section 2.3), `rfcspec.catalogue.RfcKind`, by its wire
/// spelling `K1` to `K8`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RfcKind {
    K1,
    K2,
    K3,
    K4,
    K5,
    K6,
    K7,
    K8,
}

impl RfcKind {
    /// What the kind names, as design section 2.3 lists it.
    pub fn label(self) -> &'static str {
        match self {
            RfcKind::K1 => "wire protocol",
            RfcKind::K2 => "data format or encoding",
            RfcKind::K3 => "algorithm or transform",
            RfcKind::K4 => "extension or update",
            RfcKind::K5 => "registry procedure",
            RfcKind::K6 => "management data model",
            RfcKind::K7 => "architecture, requirements, BCP, informational",
            RfcKind::K8 => "not a specification",
        }
    }
}

impl std::fmt::Display for RfcKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

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

/// A binary layout: fields with bit widths, packed most significant bit first.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Layout {
    pub name: String,
    pub quote: String,
    pub span: Span,
    /// The instant a one-field time value counts from, `YYYY-MM-DD` at 00:00 GMT.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub epoch: Option<String>,
    pub fields: Vec<Field>,
    /// The fields that carry the length of what follows the layout, most specific first: the
    /// length is the value of the first of them that is present.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub length: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Field {
    pub name: String,
    pub bits: u32,
    pub signed: bool,
    /// `big-endian` or `little-endian`.
    pub order: String,
    /// Present only when an earlier field of the layout has a value (ESS
    /// `rfcspec.catalogue.Presence`); absent means always present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presence: Option<Presence>,
    /// Constraints on the value (ESS `rfcspec.catalogue.FieldConstraint`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub constraints: Vec<Constraint>,
}

/// A field is present when another field of the layout equals the value.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Presence {
    pub field: String,
    pub equals: i64,
}

/// A constraint on a field's value (ESS `rfcspec.catalogue.ConstraintKind`): `{equals: N}`,
/// `{range: [LOW, HIGH]}` (inclusive) or `{one_of: [N, ...]}`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum Constraint {
    Equals(i64),
    Range([i64; 2]),
    OneOf(Vec<i64>),
}

impl Constraint {
    /// Every value the constraint names.
    pub fn values(&self) -> Vec<i64> {
        match self {
            Constraint::Equals(value) => vec![*value],
            Constraint::Range(bounds) => bounds.to_vec(),
            Constraint::OneOf(values) => values.clone(),
        }
    }

    /// Whether a value meets the constraint.
    pub fn admits(&self, value: i128) -> bool {
        match self {
            Constraint::Equals(expected) => value == i128::from(*expected),
            Constraint::Range([low, high]) => {
                i128::from(*low) <= value && value <= i128::from(*high)
            }
            Constraint::OneOf(values) => values.iter().any(|v| i128::from(*v) == value),
        }
    }
}

impl Field {
    /// The smallest and largest value the field's width holds.
    pub fn bounds(&self) -> (i128, i128) {
        if self.signed {
            (-(1i128 << (self.bits - 1)), (1i128 << (self.bits - 1)) - 1)
        } else {
            (0, (1i128 << self.bits) - 1)
        }
    }
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
    /// The exploration bounds of the lowered model; absent means the lowering's defaults.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bounds: Option<Bounds>,
    pub messages: Vec<Message>,
    pub participants: Vec<Participant>,
    pub properties: Vec<Property>,
}

/// The bounds of a bounded exploration.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bounds {
    pub max_steps: u32,
    pub max_states: u32,
    pub max_time_ms: u32,
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
    /// The dialogue party, `S` or `U`, for a machine read from a dialogue.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub party: Option<String>,
    pub initial: String,
    pub states: Vec<String>,
    /// The inputs the environment offers.
    #[serde(default)]
    pub inputs: Vec<Input>,
    /// The timers the participant arms and cancels.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub timers: Vec<String>,
    pub transitions: Vec<Transition>,
}

impl Participant {
    /// The input of that name.
    pub fn input(&self, name: &str) -> Option<&Input> {
        self.inputs.iter().find(|input| input.name() == name)
    }
}

/// An input the environment offers: a bare name has no fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Input {
    Bare(String),
    Typed(TypedInput),
}

/// An input with typed fields.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypedInput {
    pub name: String,
    /// Field name to type (`Integer`, `String`, `Boolean`).
    pub fields: BTreeMap<String, String>,
}

impl Input {
    pub fn name(&self) -> &str {
        match self {
            Input::Bare(name) => name,
            Input::Typed(typed) => &typed.name,
        }
    }

    /// Field name to type; none for a bare input.
    pub fn fields(&self) -> BTreeMap<String, String> {
        match self {
            Input::Bare(_) => BTreeMap::new(),
            Input::Typed(typed) => typed.fields.clone(),
        }
    }
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
    /// A timer of the participant fires.
    Timer(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case", deny_unknown_fields)]
pub enum Effect {
    Send(Send),
    Emit(String),
    Close(bool),
    /// Arm a timer for a fixed delay.
    Arm(Arm),
    /// Cancel an armed timer.
    Cancel(String),
    /// Wait until everything sent to the peer is delivered.
    Flush(bool),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Send {
    pub message: String,
    #[serde(default)]
    pub payload: BTreeMap<String, Value>,
}

/// A payload value: an integer or Boolean literal, or a field of the triggering input or
/// received message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Value {
    Integer(i64),
    Boolean(bool),
    Field(FieldRef),
}

/// A field of the triggering input or received message.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FieldRef {
    pub input: String,
}

impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::Integer(value)
    }
}

/// Arm a timer for a fixed delay in milliseconds.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Arm {
    pub timer: String,
    pub millis: u64,
}

/// A property the lowered model must hold.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Property {
    EventCount(EventCount),
    FlushBeforeClose(FlushBeforeClose),
}

impl Property {
    pub fn name(&self) -> &str {
        match self {
            Property::EventCount(count) => &count.name,
            Property::FlushBeforeClose(flush) => &flush.name,
        }
    }
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

/// A `flush_before_close` property: the participant named by `flush_before_close` closes only
/// after everything it sent to its peer is delivered.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlushBeforeClose {
    pub name: String,
    pub flush_before_close: String,
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

    #[test]
    fn inputs_values_triggers_and_properties_read_in_both_forms() {
        let inputs: Vec<Input> =
            from_yaml("- connect\n- {name: close, fields: {code: Integer}}\n").unwrap();
        assert_eq!(inputs[0].name(), "connect");
        assert!(inputs[0].fields().is_empty());
        assert_eq!(inputs[1].fields()["code"], "Integer");
        let send: Send =
            from_yaml("{message: close-frame, payload: {code: {input: code}, masked: true, n: 3}}")
                .unwrap();
        assert_eq!(
            send.payload["code"],
            Value::Field(FieldRef {
                input: "code".into()
            })
        );
        assert_eq!(send.payload["masked"], Value::Boolean(true));
        assert_eq!(send.payload["n"], Value::Integer(3));
        let effects: Vec<Effect> =
            from_yaml("- {arm: {timer: t, millis: 5000}}\n- {cancel: t}\n- {flush: true}\n")
                .unwrap();
        assert_eq!(
            effects,
            [
                Effect::Arm(Arm {
                    timer: "t".into(),
                    millis: 5000
                }),
                Effect::Cancel("t".into()),
                Effect::Flush(true)
            ]
        );
        assert_eq!(
            from_yaml::<Trigger>("{timer: t}").unwrap(),
            Trigger::Timer("t".into())
        );
        let properties: Vec<Property> = from_yaml(
            "- {name: a, participant: client, event: e, max: 1}\n- {name: b, flush_before_close: server}\n",
        )
        .unwrap();
        assert!(matches!(properties[0], Property::EventCount(_)));
        assert!(matches!(properties[1], Property::FlushBeforeClose(_)));
        assert!(
            from_yaml::<Vec<Property>>("- {name: c, flush_before_close: server, max: 1}\n")
                .is_err()
        );
        assert!(from_yaml::<Arm>("{timer: t, millis: 1, jitter: 2}").is_err());
    }

    #[test]
    fn a_layout_epoch_is_optional_and_fields_take_presence_and_constraints() {
        let layout: Layout = from_yaml(
            "name: h\nquote: q\nspan: {start: 1, end: 1}\nfields:\n  - {name: a, bits: 7, signed: false, order: big-endian}\n  - {name: b, bits: 16, signed: false, order: big-endian, presence: {field: a, equals: 126}, constraints: [{range: [126, 65535]}, {equals: 200}, {one_of: [1, 2]}]}\n",
        )
        .unwrap();
        assert_eq!(layout.epoch, None);
        let field = &layout.fields[1];
        assert_eq!(
            field.presence,
            Some(Presence {
                field: "a".into(),
                equals: 126
            })
        );
        assert_eq!(
            field.constraints,
            [
                Constraint::Range([126, 65535]),
                Constraint::Equals(200),
                Constraint::OneOf(vec![1, 2])
            ]
        );
        assert_eq!(field.bounds(), (0, 65535));
        assert!(Constraint::Range([126, 65535]).admits(126));
        assert!(!Constraint::Range([126, 65535]).admits(125));
        assert!(
            from_yaml::<Field>("{name: a, bits: 1, signed: false, order: big-endian, width: 2}")
                .is_err()
        );
        assert!(
            from_yaml::<Field>(
                "{name: a, bits: 1, signed: false, order: big-endian, constraints: [{between: [1, 2]}]}"
            )
            .is_err()
        );
        assert!(
            from_yaml::<Field>(
                "{name: a, bits: 1, signed: false, order: big-endian, presence: {field: b, equals: 1, not: 2}}"
            )
            .is_err()
        );
        let time: Overlay = read_yaml(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/time/overlay.yaml"),
        )
        .unwrap();
        assert_eq!(time.layouts[0].epoch.as_deref(), Some("1900-01-01"));
        assert!(time.layouts[0].fields[0].presence.is_none());
    }

    const OVERLAY_HEAD: &str = "format: rfcspec-overlay/1\nrfc: 1\nrequirements: []\nlayouts: []\nmachines: []\nfindings: []\n";

    #[test]
    fn an_overlay_kind_is_optional_and_an_unknown_kind_is_refused() {
        let none: Overlay = from_yaml(OVERLAY_HEAD).unwrap();
        assert_eq!(none.kind, None);
        let k1: Overlay = from_yaml(&format!("kind: K1\n{OVERLAY_HEAD}")).unwrap();
        assert_eq!(k1.kind, Some(RfcKind::K1));
        assert_eq!(k1.kind.unwrap().to_string(), "K1");
        assert!(from_yaml::<Overlay>(&format!("kind: K9\n{OVERLAY_HEAD}")).is_err());
        assert!(from_yaml::<Overlay>(&format!("kind: k1\n{OVERLAY_HEAD}")).is_err());
    }

    #[test]
    fn the_time_overlay_is_k1() {
        let overlay: Overlay = read_yaml(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../profiles/time/overlay.yaml"),
        )
        .unwrap();
        assert_eq!(overlay.kind, Some(RfcKind::K1));
    }
}
