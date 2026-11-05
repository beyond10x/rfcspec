#![forbid(unsafe_code)]
//! S3 (design sections 9 and 13): ask a model, through the `llm` crates, for one overlay
//! requirement entry about one catalogue item; check the answer mechanically; record it as an
//! `rfcspec-proposal/1` document under the profile's `proposals/`; and replay every recording
//! offline, byte for byte. A proposal changes nothing: neither the overlay nor a generated file is
//! written. S4 over a recorded proposal ([`review`]) writes one decision into the overlay.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::{Path, PathBuf},
    time::Duration,
};

use anyhow::{Context, Result, anyhow, bail};
use llm_core::{Item, Model, ToolName, ToolSpec};
use llm_http::{HttpClient, Limits};
use llm_models::{
    Port,
    routes::{Routes, SecretSources},
};
use llm_tool_call::{ModelError, call_tool};
use rfcspec_model::{
    Catalogue, Document, Edit, OVERLAY_FILE, Overlay, PROFILE_FILE, Profile, Requirement, Span,
    Status, collapse_whitespace, read_yaml, text_digest,
};
use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

#[cfg(any(test, feature = "double"))]
pub mod double;
mod elements;
pub mod review;

use elements::tool_for;

/// The format of a proposal document.
pub const FORMAT: &str = "rfcspec-proposal/1";
/// The directory of a profile that holds its proposals, one `<proposal_id>.json` each.
pub const PROPOSALS_DIR: &str = "proposals";
/// The name of the one tool the model is forced to call.
pub const TOOL: &str = "propose_requirement_entry";

/// The requirement levels an entry may give: the BCP 14 keywords, negation included.
const LEVELS: [&str; 11] = [
    "MUST",
    "MUST NOT",
    "REQUIRED",
    "SHALL",
    "SHALL NOT",
    "SHOULD",
    "SHOULD NOT",
    "RECOMMENDED",
    "NOT RECOMMENDED",
    "MAY",
    "OPTIONAL",
];

/// Source lines shown around an item's span in the prompt.
const CONTEXT_LINES: usize = 3;

/// What a proposal asks for (ESS `rfcspec.review.ProposalTask`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Task {
    /// A status `unsupported` or `not-applicable` with a reason, and a level, for one BCP 14
    /// requirement of the catalogue.
    Status,
    /// A level and a status for one `prose_candidates` sentence of the catalogue.
    RequirementFromProse,
    /// One overlay `machines` entry read from named sections; the item is the whole profile.
    Machine,
    /// One overlay `layouts` entry read from a packet diagram of named sections; the item is
    /// `rfc<N>/layout/<name>`.
    BitWidths,
    /// A status `satisfied` by an element the overlay holds, for one BCP 14 requirement.
    RequirementLink,
}

impl Task {
    /// The wire name (ESS `rfcspec.review.ProposalTask`).
    pub fn wire(self) -> &'static str {
        match self {
            Self::Status => "status",
            Self::RequirementFromProse => "requirement-from-prose",
            Self::Machine => "machine",
            Self::BitWidths => "bit-widths",
            Self::RequirementLink => "requirement-link",
        }
    }

    /// The task with this wire name.
    pub fn from_wire(wire: &str) -> Option<Self> {
        [
            Self::Status,
            Self::RequirementFromProse,
            Self::Machine,
            Self::BitWidths,
            Self::RequirementLink,
        ]
        .into_iter()
        .find(|task| task.wire() == wire)
    }

    /// Whether the task reads named sections rather than one catalogue item.
    pub fn reads_sections(self) -> bool {
        matches!(self, Self::Machine | Self::BitWidths)
    }

    /// What an answer of the task is: `requirement`, `layout` or `machine`.
    pub fn element(self) -> &'static str {
        match self {
            Self::Machine => "machine",
            Self::BitWidths => "layout",
            Self::Status | Self::RequirementFromProse | Self::RequirementLink => "requirement",
        }
    }

    /// The standing instruction of the task's prompt.
    fn instructions(self) -> &'static str {
        match self {
            Self::Status => {
                "You review one BCP 14 requirement of an RFC for a protocol profile whose model \
                 does not implement it. Call the tool once with the requirement as an overlay \
                 requirement entry: its id and source exactly as given, the level as you read it, \
                 a quote copied verbatim from the numbered source lines, the span of source lines \
                 the quote lies in, and a status `unsupported` or `not-applicable` with a short, \
                 non-empty reason."
            }
            Self::RequirementFromProse => {
                "You review one sentence of an RFC that uses a lowercase `must`, `should` or \
                 `may`. Call the tool once with it as an overlay requirement entry: its id \
                 exactly as given, source `prose`, the BCP 14 level the sentence states as you \
                 read it, a quote copied verbatim from the numbered source lines, the span of \
                 source lines the quote lies in, and a status `unsupported` or `not-applicable` \
                 with a short, non-empty reason."
            }
            Self::Machine => {
                "You model the protocol behaviour of the numbered RFC sections given as one \
                 state machine of a protocol profile. Call the tool once with an overlay machine: \
                 a name, the transport (one of the profile's layers given), the messages \
                 exchanged, exactly two participants with their states, inputs and transitions, \
                 and properties. Every transition lists in `implements` the ids, from the \
                 requirements given, of the requirements it implements, at least one."
            }
            Self::BitWidths => {
                "You read the packet diagram and the field descriptions of the numbered RFC \
                 sections given. Call the tool once with an overlay layout under the name given: \
                 a quote copied verbatim from the numbered source lines and the span of source \
                 lines it lies in, and the fields in wire order with their widths in bits (1 to \
                 64), signedness, byte order, presence on an earlier field and constraints."
            }
            Self::RequirementLink => {
                "You review one BCP 14 requirement of an RFC against the model elements of a \
                 protocol profile. Call the tool once with the requirement as an overlay \
                 requirement entry: its id and source exactly as given, the level as you read it, \
                 a quote copied verbatim from the numbered source lines, the span of source lines \
                 the quote lies in, and a status `satisfied` naming the one element given that \
                 satisfies it."
            }
        }
    }
}

/// What a proposal is about: one catalogue item, or named sections of the RFC.
struct Subject {
    /// The stable item id (ESS `rfcspec.catalogue.ItemId`).
    item: String,
    /// The catalogue id, the layout name or the profile name.
    id: String,
    keyword: String,
    /// The BCP 14 level a requirement entry about the subject must give: the catalogue keyword
    /// of a BCP 14 requirement; none for a prose candidate, which has no level to keep.
    level: Option<String>,
    /// The text judged: the item's catalogue text, or the sections' text.
    text: String,
    span: Span,
    section: Option<String>,
    source: String,
    /// The sections read, for a task that reads sections.
    sections: Vec<String>,
}

/// The prompt and inputs of one proposal, and their digest.
struct Built {
    inputs: Value,
    prompt: String,
    input_digest: String,
}

/// A profile directory read for proposing: its profile, overlay, locked source and catalogue.
pub struct Workspace {
    dir: PathBuf,
    profile: Profile,
    overlay: Overlay,
    document: Document,
    catalogue: Catalogue,
    source: String,
    furniture: Vec<usize>,
}

impl Workspace {
    /// Read the profile at `dir`: its profile, overlay and the locked source of the overlay's RFC.
    pub fn load(dir: &Path) -> Result<Self> {
        let profile: Profile = read_yaml(&dir.join(PROFILE_FILE))?;
        let overlay: Overlay = read_yaml(&dir.join(OVERLAY_FILE))?;
        let locked = rfcspec_source::read_locked(dir, overlay.rfc)?;
        let document = rfcspec_doc::read(overlay.rfc, &locked.entry.sha256, &locked.text);
        Ok(Self {
            dir: dir.to_path_buf(),
            catalogue: rfcspec_extract::extract(&document),
            furniture: rfcspec_doc::furniture_lines(&locked.text),
            source: locked.text,
            document,
            profile,
            overlay,
        })
    }

    /// Whether the overlay holds a decision on `subject` judging its current text.
    fn decided(&self, subject: &Subject) -> bool {
        let digest = text_digest(&subject.text);
        self.overlay
            .decisions
            .iter()
            .any(|d| d.item == subject.item && d.text_digest == digest)
    }

    /// The overlay requirement, other than one with id `except`, that already reviews the text
    /// `text` at `span`: its span shares a line with `span` and its whitespace-collapsed quote
    /// lies within `text` or holds it. Spans alone do not decide it, because two sentences can
    /// share a source line (RFC 1035 line 1762 ends P-56 and starts P-57).
    pub(crate) fn covering(&self, text: &str, span: Span, except: &str) -> Option<&Requirement> {
        let text = collapse_whitespace(text);
        self.overlay.requirements.iter().find(|r| {
            let quote = collapse_whitespace(&r.quote);
            r.id != except
                && !quote.is_empty()
                && !text.is_empty()
                && r.span.start <= span.end
                && span.start <= r.span.end
                && (text.contains(&quote) || quote.contains(&text))
        })
    }

    /// The items of the given sections (and their subsections) that have no overlay status and
    /// no decision: BCP 14 requirements for [`Task::Status`] and [`Task::RequirementLink`],
    /// prose candidates for [`Task::RequirementFromProse`] (a candidate an overlay requirement
    /// of another id quotes has a status, see [`Workspace::covering`]); for [`Task::Machine`], the profile
    /// unless a decision judges the sections' current text. [`Task::BitWidths`] names its
    /// layout, so nothing is pending.
    pub fn pending(&self, task: Task, sections: &[String]) -> Vec<String> {
        let reviewed: BTreeSet<&str> = self
            .overlay
            .requirements
            .iter()
            .map(|r| r.id.as_str())
            .collect();
        let candidates: Vec<(&str, Option<String>)> = match task {
            Task::Status | Task::RequirementLink => self
                .catalogue
                .requirements
                .iter()
                .map(|r| (r.id.as_str(), r.section.clone()))
                .collect(),
            Task::RequirementFromProse => self
                .catalogue
                .prose_candidates
                .iter()
                .filter(|c| self.covering(&c.text, c.span, &c.id).is_none())
                .map(|c| (c.id.as_str(), self.section_of(c.span.start)))
                .collect(),
            Task::Machine => {
                return self
                    .subject(task, &self.profile.name, sections)
                    .ok()
                    .filter(|subject| !self.decided(subject))
                    .map(|subject| subject.id)
                    .into_iter()
                    .collect();
            }
            Task::BitWidths => return Vec::new(),
        };
        candidates
            .into_iter()
            .filter(|(id, section)| {
                !reviewed.contains(id)
                    && section
                        .as_deref()
                        .is_some_and(|section| in_sections(section, sections))
                    && self
                        .subject(task, id, &[])
                        .is_ok_and(|subject| !self.decided(&subject))
            })
            .map(|(id, _)| id.to_owned())
            .collect()
    }

    /// The numbered section a source line lies in.
    fn section_of(&self, line: usize) -> Option<String> {
        self.document
            .blocks
            .iter()
            .find(|block| block.span.contains(line))
            .and_then(|block| block.section.clone())
    }

    /// What a proposal of `task` about `id` asks about: a catalogue id (`R5.1-1`, `P-34`), the
    /// profile's name for [`Task::Machine`], or a layout name for [`Task::BitWidths`], whose
    /// `sections` are read.
    fn subject(&self, task: Task, id: &str, sections: &[String]) -> Result<Subject> {
        let rfc = self.overlay.rfc;
        match task {
            Task::Status | Task::RequirementLink => {
                let found = self
                    .catalogue
                    .requirements
                    .iter()
                    .find(|r| r.id == id)
                    .with_context(|| format!("{id} is no BCP 14 requirement of RFC {rfc}"))?;
                Ok(Subject {
                    item: self.item(&found.id),
                    id: found.id.clone(),
                    keyword: found.keyword.clone(),
                    level: Some(found.keyword.clone()),
                    text: found.text.clone(),
                    span: found.span,
                    section: found.section.clone(),
                    source: found.source.clone(),
                    sections: Vec::new(),
                })
            }
            Task::RequirementFromProse => {
                let found = self
                    .catalogue
                    .prose_candidates
                    .iter()
                    .find(|c| c.id == id)
                    .with_context(|| format!("{id} is no prose candidate of RFC {rfc}"))?;
                Ok(Subject {
                    item: self.item(&found.id),
                    id: found.id.clone(),
                    keyword: found.keyword.clone(),
                    level: None,
                    text: found.text.clone(),
                    span: found.span,
                    section: self.section_of(found.span.start),
                    source: "prose".into(),
                    sections: Vec::new(),
                })
            }
            Task::Machine | Task::BitWidths => {
                let item = if task == Task::Machine {
                    if id != self.profile.name {
                        bail!(
                            "a machine proposal is about the whole profile {}, not {id}",
                            self.profile.name
                        );
                    }
                    id.to_owned()
                } else {
                    let name = id.strip_prefix(&format!("rfc{rfc}/layout/")).unwrap_or(id);
                    if name.is_empty()
                        || !name
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b"-_.".contains(&b))
                    {
                        bail!("{id:?} is no layout name");
                    }
                    return self.sections_subject(
                        format!("rfc{rfc}/layout/{name}"),
                        name,
                        sections,
                    );
                };
                self.sections_subject(item.clone(), &item, sections)
            }
        }
    }

    /// The subject of a task that reads `sections`: their blocks' text, one block per
    /// paragraph, and the span from the first block to the last.
    fn sections_subject(&self, item: String, id: &str, sections: &[String]) -> Result<Subject> {
        if sections.is_empty() {
            bail!("give the sections to read for {item}");
        }
        let blocks: Vec<&rfcspec_model::Block> = self
            .document
            .blocks
            .iter()
            .filter(|block| {
                block
                    .section
                    .as_deref()
                    .is_some_and(|section| in_sections(section, sections))
            })
            .collect();
        let (Some(first), Some(last)) = (blocks.first(), blocks.last()) else {
            bail!(
                "RFC {} has no text in sections {}",
                self.overlay.rfc,
                sections.join(", ")
            );
        };
        Ok(Subject {
            item,
            id: id.to_owned(),
            keyword: String::new(),
            level: None,
            text: blocks
                .iter()
                .map(|block| block.text())
                .collect::<Vec<_>>()
                .join("\n\n"),
            span: Span {
                start: first.span.start,
                end: last.span.end,
            },
            section: None,
            source: String::new(),
            sections: sections.to_vec(),
        })
    }

    /// The item id of a catalogue id: `rfc<N>/<id>`.
    fn item(&self, id: &str) -> String {
        format!("rfc{}/{id}", self.overlay.rfc)
    }

    /// The overlay elements a requirement may be satisfied by, in the spelling of
    /// `Status::Satisfied`: `layout.<name>` and `<machine>.<participant>.<transition>`.
    fn elements(&self) -> BTreeMap<String, Option<&rfcspec_model::Transition>> {
        let mut elements = BTreeMap::new();
        for layout in &self.overlay.layouts {
            elements.insert(format!("layout.{}", layout.name), None);
        }
        for machine in &self.overlay.machines {
            for participant in &machine.participants {
                for transition in &participant.transitions {
                    elements.insert(
                        format!(
                            "{}.{}.{}",
                            machine.name,
                            participant.name,
                            transition.name()
                        ),
                        Some(transition),
                    );
                }
            }
        }
        elements
    }

    /// What a proposal of `task` is given besides the locked source: the profile's layers for a
    /// machine, the overlay's elements for a link. Replay takes it from the recording.
    fn context(&self, task: Task) -> Option<Value> {
        match task {
            Task::Machine => Some(json!({
                "layers": self
                    .profile
                    .semantics
                    .iter()
                    .map(|layer| layer.layer.as_str())
                    .collect::<Vec<_>>(),
            })),
            Task::RequirementLink => Some(json!({
                "elements": self.elements().into_keys().collect::<Vec<_>>(),
            })),
            Task::Status | Task::RequirementFromProse | Task::BitWidths => None,
        }
    }

    /// The prompt and inputs for `subject`, rebuilt from the locked source and `context`.
    fn build(&self, task: Task, subject: &Subject, context: Option<&Value>) -> Built {
        let lines: Vec<&str> = self.source.lines().collect();
        let numbered = |first: usize, last: usize| -> String {
            (first..=last)
                .map(|line| format!("{line:>5} | {}\n", lines[line - 1]))
                .collect()
        };
        let mut inputs = Map::new();
        inputs.insert("item".into(), json!(subject.item));
        let prompt = if task.reads_sections() {
            let requirements: Vec<Value> = self
                .catalogue
                .requirements
                .iter()
                .filter(|r| {
                    r.section
                        .as_deref()
                        .is_some_and(|section| in_sections(section, &subject.sections))
                })
                .map(|r| {
                    json!({
                        "id": r.id,
                        "keyword": r.keyword,
                        "span": {"start": r.span.start, "end": r.span.end},
                        "text": r.text,
                    })
                })
                .collect();
            let mut prompt = format!(
                "RFC {rfc}, sections {sections}.\nItem id: {item}\n",
                rfc = self.overlay.rfc,
                sections = subject.sections.join(", "),
                item = subject.item,
            );
            if task == Task::BitWidths {
                prompt.push_str(&format!("Layout name: {}\n", subject.id));
            }
            if task == Task::Machine {
                prompt.push_str("Requirements of these sections:\n");
                for r in &requirements {
                    prompt.push_str(&format!(
                        "- {} ({}, lines {}-{}): {}\n",
                        r["id"].as_str().unwrap_or_default(),
                        r["keyword"].as_str().unwrap_or_default(),
                        r["span"]["start"],
                        r["span"]["end"],
                        r["text"].as_str().unwrap_or_default(),
                    ));
                }
                inputs.insert("requirements".into(), Value::Array(requirements));
            }
            inputs.insert("sections".into(), json!(subject.sections));
            prompt
        } else {
            inputs.insert("keyword".into(), json!(subject.keyword));
            if let Some(section) = &subject.section {
                inputs.insert("section".into(), json!(section));
            }
            inputs.insert("source".into(), json!(subject.source));
            format!(
                "RFC {rfc}, section {section}.\nItem id: {id}\nSource: {source}\nKeyword: {keyword}\n\
                 Sentence: {text}\n",
                rfc = self.overlay.rfc,
                section = subject.section.as_deref().unwrap_or("(none)"),
                id = subject.id,
                source = subject.source,
                keyword = subject.keyword,
                text = subject.text,
            )
        };
        if let Some(context) = context {
            inputs.insert("context".into(), context.clone());
        }
        inputs.insert("rfc".into(), json!(self.overlay.rfc));
        inputs.insert("source_sha256".into(), json!(self.document.sha256));
        inputs.insert(
            "span".into(),
            json!({"start": subject.span.start, "end": subject.span.end}),
        );
        inputs.insert("text".into(), json!(subject.text));
        let inputs = canonical(Value::Object(inputs));

        let mut prompt = prompt;
        if let Some(context) = context {
            for (key, heading) in [
                ("layers", "Layers of the profile"),
                (
                    "elements",
                    "Overlay elements a requirement may be satisfied by",
                ),
            ] {
                if let Some(values) = context.get(key).and_then(Value::as_array) {
                    prompt.push_str(&format!("{heading}:\n"));
                    for value in values {
                        prompt.push_str(&format!("- {}\n", value.as_str().unwrap_or_default()));
                    }
                }
            }
        }
        let excerpt = if task.reads_sections() {
            prompt.push_str(&format!(
                "The sections lie in source lines {}-{}:\n\n",
                subject.span.start, subject.span.end
            ));
            numbered(subject.span.start, subject.span.end.min(lines.len()))
        } else {
            prompt.push_str(&format!(
                "The sentence lies in source lines {}-{}:\n\n",
                subject.span.start, subject.span.end
            ));
            let first = subject.span.start.saturating_sub(CONTEXT_LINES).max(1);
            let last = (subject.span.end + CONTEXT_LINES).min(lines.len());
            numbered(first, last)
        };
        prompt.push_str(&excerpt);
        let tool = tool_for(task);
        let digested = canonical(json!({
            "inputs": inputs,
            "instructions": task.instructions(),
            "prompt": prompt,
            "tool": {
                "description": tool.description,
                "input_schema": tool.input_schema,
                "name": tool.name.as_str(),
            },
        }));
        Built {
            input_digest: sha256(&serialize(&digested)),
            inputs,
            prompt,
        }
    }

    /// The mechanical checks of design section 9 on one answer, by task; the checked element
    /// and the spans the proposal cites.
    fn check(&self, task: Task, subject: &Subject, response: &Value) -> Result<Checked, String> {
        match task {
            Task::Status | Task::RequirementFromProse => {
                let entry = self.entry(subject, response)?;
                match &entry.status {
                    Status::Satisfied(element) => Err(format!(
                        "status satisfied by {element}: a proposal states unsupported or not-applicable"
                    )),
                    Status::Unsupported(reason) | Status::NotApplicable(reason)
                        if reason.trim().is_empty() =>
                    {
                        Err("empty reason".into())
                    }
                    Status::Unsupported(_) | Status::NotApplicable(_) => Ok(Checked {
                        cited: vec![entry.span],
                        element: Edit::Requirement(entry),
                    }),
                }
            }
            Task::RequirementLink => self.check_link(subject, response),
            Task::BitWidths => self.check_layout(subject, response),
            Task::Machine => self.check_machine(response),
        }
    }

    /// The checks every requirement entry passes: it is an overlay requirement entry for the
    /// item asked about, its quote occurs at its span in the locked source (the test `rfcspec
    /// check` applies), its span lies within the item's, and its level is a BCP 14 level, the
    /// catalogue keyword of a BCP 14 requirement (a prose candidate's entry chooses its level).
    fn entry(&self, subject: &Subject, response: &Value) -> Result<Requirement, String> {
        let entry: Requirement = serde_json::from_value(response.clone())
            .map_err(|error| format!("the answer is not an overlay requirement entry: {error}"))?;
        if entry.id != subject.id {
            return Err(format!(
                "the entry answers {}, asked about {}",
                entry.id, subject.id
            ));
        }
        if entry.source != subject.source {
            return Err(format!(
                "the entry has source {}, expected {}",
                entry.source, subject.source
            ));
        }
        self.quoted(&entry.quote, entry.span, subject)?;
        if !LEVELS.contains(&entry.level.as_str()) {
            return Err(format!("level {:?} is no BCP 14 level", entry.level));
        }
        if let Some(level) = &subject.level
            && entry.level != *level
        {
            return Err(format!(
                "level {} differs from the catalogue level {level} of {}",
                entry.level, subject.id
            ));
        }
        Ok(entry)
    }

    /// The quote occurs at its span in the locked source, and the span lies within the
    /// subject's.
    fn quoted(&self, quote: &str, span: Span, subject: &Subject) -> Result<(), String> {
        let lines: Vec<&str> = self.source.lines().collect();
        if let Some(problem) = rfcspec_extract::quote_problem(&lines, &self.furniture, quote, span)
        {
            return Err(problem);
        }
        if span.start < subject.span.start || subject.span.end < span.end {
            return Err(format!(
                "{span} does not lie within {} of {}",
                subject.span, subject.id
            ));
        }
        Ok(())
    }

    /// The proposal document, canonical, under the id of task, model id and input digest;
    /// `checked` is the checked response.
    fn document(
        &self,
        task: Task,
        subject: &Subject,
        built: &Built,
        model_id: &str,
        response: &Value,
        checked: &Checked,
    ) -> String {
        let cited: Vec<Value> = checked
            .cited
            .iter()
            .map(|span| json!({"start": span.start, "end": span.end}))
            .collect();
        serialize(&canonical(json!({
            "cited_spans": cited,
            "format": FORMAT,
            "input_digest": built.input_digest,
            "inputs": built.inputs,
            "item": subject.item,
            "model_id": model_id,
            "profile": self.profile.name,
            "proposal_id": proposal_id(task, model_id, &built.input_digest),
            "response": response,
            "task": task.wire(),
        })))
    }

    fn proposals(&self) -> PathBuf {
        self.dir.join(PROPOSALS_DIR)
    }
}

/// An answer that passed its checks: the overlay element it proposes and the spans it cites.
pub(crate) struct Checked {
    element: Edit,
    cited: Vec<Span>,
}

/// Whether `section` is one of `sections` or lies under one of them.
fn in_sections(section: &str, sections: &[String]) -> bool {
    sections.iter().any(|wanted| {
        section == wanted
            || section
                .strip_prefix(wanted.as_str())
                .is_some_and(|rest| rest.starts_with('.'))
    })
}

/// The tool the model is forced to call: its input is one overlay requirement entry in the
/// `rfcspec-overlay/1` shape.
pub fn tool() -> ToolSpec {
    let reason = |name: &str| {
        json!({
            "type": "object",
            "additionalProperties": false,
            "required": [name],
            "properties": {name: {"type": "string", "minLength": 1}},
        })
    };
    ToolSpec {
        name: ToolName::new(TOOL).expect("a valid tool name"),
        description: "Give one overlay requirement entry for the item asked about.".into(),
        input_schema: json!({
            "type": "object",
            "additionalProperties": false,
            "required": ["id", "source", "level", "quote", "span", "status"],
            "properties": {
                "id": {"type": "string", "description": "The item id exactly as given."},
                "source": {
                    "type": "string",
                    "enum": ["bcp14-xml", "capitals-text", "abnf-comment", "prose"],
                },
                "level": {"type": "string", "enum": LEVELS},
                "quote": {
                    "type": "string",
                    "description": "Copied verbatim from the numbered source lines, without the line numbers.",
                },
                "span": {
                    "type": "object",
                    "additionalProperties": false,
                    "required": ["start", "end"],
                    "properties": {
                        "start": {"type": "integer", "minimum": 1},
                        "end": {"type": "integer", "minimum": 1},
                    },
                    "description": "The first and last source line of the quote, inclusive.",
                },
                "status": {"oneOf": [reason("unsupported"), reason("not-applicable")]},
            },
        }),
    }
}

/// What recording one item did.
#[derive(Debug, PartialEq, Eq)]
pub enum Outcome {
    /// The answer passed its checks and was written to this file.
    Recorded(PathBuf),
    /// A proposal with this id was already recorded at this file; the model was not asked.
    AlreadyExists(PathBuf),
    /// The overlay holds a decision on the item under its current text; the model was not
    /// asked and nothing was written.
    Decided,
}

/// Why recording one item wrote nothing.
#[derive(Debug)]
pub enum ProposeError {
    /// A mechanical check failed (ESS `rfcspec.review.RecordProposal` outcome `check-failed`).
    CheckFailed { proposal_id: String, reason: String },
    /// The model gave no usable answer.
    Model(ModelError),
    /// Anything else: an unknown item, an unreadable or unwritable file.
    Other(anyhow::Error),
}

impl std::fmt::Display for ProposeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CheckFailed {
                proposal_id,
                reason,
            } => write!(f, "proposal {proposal_id}: check failed: {reason}"),
            Self::Model(error) => write!(f, "{error}"),
            Self::Other(error) => write!(f, "{error:#}"),
        }
    }
}

impl std::error::Error for ProposeError {}

/// Record one proposal for `item`, a catalogue id (`R5.1-1`, `P-34`): build the prompt, read the
/// proposal when one is already recorded under its id, else ask `model` once, check the answer
/// and write it. Nothing is written when a check fails. A task that reads sections
/// ([`Task::Machine`], [`Task::BitWidths`]) goes through [`record_sections`].
pub async fn record(
    workspace: &Workspace,
    model: &dyn Model,
    model_id: &str,
    task: Task,
    item: &str,
) -> Result<Outcome, ProposeError> {
    record_sections(workspace, model, model_id, task, item, &[]).await
}

/// [`record`] for any task: `item` is a catalogue id, the profile's name for
/// [`Task::Machine`] or a layout name for [`Task::BitWidths`], and `sections` the sections
/// those two read.
pub async fn record_sections(
    workspace: &Workspace,
    model: &dyn Model,
    model_id: &str,
    task: Task,
    item: &str,
    sections: &[String],
) -> Result<Outcome, ProposeError> {
    let subject = workspace
        .subject(task, item, sections)
        .map_err(ProposeError::Other)?;
    if workspace.decided(&subject) {
        return Ok(Outcome::Decided);
    }
    let context = workspace.context(task);
    let built = workspace.build(task, &subject, context.as_ref());
    let proposal_id = proposal_id(task, model_id, &built.input_digest);
    let path = workspace.proposals().join(format!("{proposal_id}.json"));
    if path.exists() {
        let text = fs::read_to_string(&path)
            .with_context(|| format!("reading {}", path.display()))
            .map_err(ProposeError::Other)?;
        serde_json::from_str::<Value>(&text)
            .with_context(|| format!("parsing {}", path.display()))
            .map_err(ProposeError::Other)?;
        return Ok(Outcome::AlreadyExists(path));
    }
    let response = call_tool(
        model,
        task.instructions(),
        vec![Item::user(built.prompt.clone())],
        tool_for(task),
    )
    .await
    .map_err(ProposeError::Model)?;
    let response = canonical(response);
    let checked = workspace
        .check(task, &subject, &response)
        .map_err(|reason| ProposeError::CheckFailed {
            proposal_id: proposal_id.clone(),
            reason,
        })?;
    let text = workspace.document(task, &subject, &built, model_id, &response, &checked);
    fs::create_dir_all(workspace.proposals())
        .and_then(|()| fs::write(&path, text))
        .with_context(|| format!("writing {}", path.display()))
        .map_err(ProposeError::Other)?;
    Ok(Outcome::Recorded(path))
}

/// [`record`] on a current-thread runtime of its own.
pub fn record_blocking(
    workspace: &Workspace,
    model: &dyn Model,
    model_id: &str,
    task: Task,
    item: &str,
) -> Result<Outcome, ProposeError> {
    record_sections_blocking(workspace, model, model_id, task, item, &[])
}

/// [`record_sections`] on a current-thread runtime of its own.
pub fn record_sections_blocking(
    workspace: &Workspace,
    model: &dyn Model,
    model_id: &str,
    task: Task,
    item: &str,
    sections: &[String],
) -> Result<Outcome, ProposeError> {
    runtime()
        .map_err(ProposeError::Other)?
        .block_on(record_sections(
            workspace, model, model_id, task, item, sections,
        ))
}

/// [`record_sections`] for each item in order, on one runtime. A check failure is reported and
/// the next item asked; a model that gives no usable answer stops the run.
pub fn record_all(
    workspace: &Workspace,
    model: &dyn Model,
    model_id: &str,
    task: Task,
    items: &[String],
    sections: &[String],
) -> Result<Vec<(String, Result<Outcome, ProposeError>)>> {
    let runtime = runtime()?;
    let mut results = Vec::new();
    for item in items {
        let result = runtime.block_on(record_sections(
            workspace, model, model_id, task, item, sections,
        ));
        let stop = matches!(result, Err(ProposeError::Model(_)));
        results.push((item.clone(), result));
        if stop {
            break;
        }
    }
    Ok(results)
}

fn runtime() -> Result<tokio::runtime::Runtime> {
    Ok(tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?)
}

/// A model reached through an `llm.routes/1` document.
pub struct Routed {
    /// The port the route builds.
    pub model: Port,
    /// The id a proposal records: `<endpoint>/<upstream model>` for a declared route, the name
    /// itself for a direct `<endpoint>/<model>` name.
    pub model_id: String,
}

/// The model `route` reaches in the routes document `text`. Its credentials are references the
/// operator injects (`env:`, `file:`); building reads none of them, and an unset or missing one
/// is refused at the first turn, by the endpoint's name, before any request is sent.
/// The transport bounds for a proposal call. A model served on one local GPU answers a forced tool
/// call without streaming, so its response headers arrive only after the whole answer: the
/// default 60 s header bound refuses a thinking model before it has answered once.
#[must_use]
pub fn limits() -> Limits {
    Limits {
        response_headers: Duration::from_secs(900),
        idle: Duration::from_secs(900),
        total: Duration::from_secs(1200),
    }
}

pub fn routed(text: &str, route: &str) -> Result<Routed> {
    let routes = Routes::parse(text).map_err(|error| anyhow!("routes: {error}"))?;
    let mut sources = SecretSources::new();
    if let Some(home) = std::env::var_os("HOME").filter(|home| !home.is_empty()) {
        sources = sources.with_home(home);
    }
    let resolvers = routes
        .resolvers(&sources)
        .map_err(|error| anyhow!("routes: {error}"))?;
    let http = HttpClient::new(limits()).map_err(|error| anyhow!("{error}"))?;
    let model = routes
        .port(route, http, &resolvers)
        .map_err(|error| anyhow!("route {route}: {error}"))?;
    let model_id = match routes.route(route) {
        Some(declared) => format!("{}/{}", declared.endpoint(), declared.upstream_model()),
        None => route.to_owned(),
    };
    Ok(Routed { model, model_id })
}

/// Replay every proposal of the profile at `dir`: rebuild its prompt and inputs from the locked
/// source and the recorded item and task, recompute `input_digest` and `proposal_id`, take the
/// response from the recording, rerun the checks and compare the serialized bytes with the file.
/// Returns one problem per finding, each naming its file; empty when every proposal is
/// identical or the profile has none.
pub fn replay(dir: &Path) -> Result<Vec<String>> {
    let proposals = dir.join(PROPOSALS_DIR);
    if !proposals.is_dir() {
        return Ok(Vec::new());
    }
    let workspace = Workspace::load(dir)?;
    let mut files: Vec<PathBuf> = fs::read_dir(&proposals)
        .with_context(|| format!("reading {}", proposals.display()))?
        .map(|entry| entry.map(|entry| entry.path()))
        .collect::<std::io::Result<_>>()?;
    files.retain(|file| file.extension().is_some_and(|ext| ext == "json"));
    files.sort();
    let mut problems = Vec::new();
    for file in files {
        if let Err(problem) = replay_one(&workspace, &file) {
            problems.push(format!("{}: {problem}", file.display()));
        }
    }
    Ok(problems)
}

/// A proposal file reproduced by replay: what it was asked, about which subject under the
/// current source, and the element it proposes, checked against the current profile.
pub(crate) struct Replayed {
    pub(crate) task: Task,
    pub(crate) subject: Subject,
    pub(crate) checked: Checked,
}

/// Replay one proposal file; the error says why it is not reproduced. The sections of a task
/// that reads sections, and the context of a task given one (the profile's layers, the
/// overlay's elements), are taken from the recording, as the item and task are; everything else
/// is rebuilt from the locked source.
fn replay_one(workspace: &Workspace, file: &Path) -> Result<Replayed> {
    let text = fs::read_to_string(file)?;
    let recorded: Value =
        serde_json::from_str(&text).context("not an rfcspec-proposal/1 document")?;
    let field = |name: &str| {
        recorded
            .get(name)
            .and_then(Value::as_str)
            .with_context(|| format!("no {name}"))
    };
    if field("format")? != FORMAT {
        bail!("format {:?}, expected {FORMAT}", recorded["format"]);
    }
    let task = Task::from_wire(field("task")?)
        .with_context(|| format!("unknown task {:?}", recorded["task"]))?;
    let item = field("item")?;
    let rfc = workspace.overlay.rfc;
    let id = match task {
        Task::Machine => item,
        Task::BitWidths => item
            .strip_prefix(&format!("rfc{rfc}/layout/"))
            .with_context(|| format!("item {item} is no layout of RFC {rfc}"))?,
        Task::Status | Task::RequirementFromProse | Task::RequirementLink => item
            .strip_prefix(&format!("rfc{rfc}/"))
            .with_context(|| format!("item {item} is not of RFC {rfc}"))?,
    };
    let inputs = recorded.get("inputs").context("no inputs")?;
    let sections: Vec<String> = if task.reads_sections() {
        serde_json::from_value(
            inputs
                .get("sections")
                .cloned()
                .context("no inputs.sections")?,
        )
        .context("inputs.sections is no list of sections")?
    } else {
        Vec::new()
    };
    let context = match workspace.context(task) {
        Some(_) => Some(inputs.get("context").context("no inputs.context")?),
        None => None,
    };
    let model_id = field("model_id")?;
    let response = canonical(recorded.get("response").cloned().context("no response")?);
    let subject = workspace.subject(task, id, &sections)?;
    let built = workspace.build(task, &subject, context);
    let proposal_id = proposal_id(task, model_id, &built.input_digest);
    let name = format!("{proposal_id}.json");
    if file.file_name().and_then(|name| name.to_str()) != Some(name.as_str()) {
        bail!("the proposal's id is {proposal_id}, so its file is {name}");
    }
    let checked = workspace
        .check(task, &subject, &response)
        .map_err(|reason| anyhow!("proposal {proposal_id}: check failed: {reason}"))?;
    let replayed = workspace.document(task, &subject, &built, model_id, &response, &checked);
    if replayed != text {
        bail!("differs from its replay");
    }
    Ok(Replayed {
        task,
        subject,
        checked,
    })
}

/// The SHA-256 of task, model id and input digest (ESS `rfcspec.review.ProposalId`).
fn proposal_id(task: Task, model_id: &str, input_digest: &str) -> String {
    sha256(&format!("{}\n{model_id}\n{input_digest}", task.wire()))
}

fn sha256(text: &str) -> String {
    Sha256::digest(text.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// A JSON value with every object's keys in sorted order, whatever map serde_json is built with.
fn canonical(value: Value) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map.into_iter().collect();
            entries.sort_by(|a, b| a.0.cmp(&b.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, canonical(value)))
                    .collect(),
            )
        }
        Value::Array(items) => Value::Array(items.into_iter().map(canonical).collect()),
        other => other,
    }
}

/// Canonical bytes: two-space indent and a trailing newline.
fn serialize(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).expect("a JSON value serializes");
    text.push('\n');
    text
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod elements_tests;
