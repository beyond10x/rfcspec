//! S4 over recorded proposals (ESS `rfcspec.review.DecideProposal` and
//! `rfcspec.review.RecordDecision`): settle one proposal with accept, edit or reject by a decision
//! in the profile's overlay. Accept and edit also add the proposed or edited element (a
//! requirement entry, a layout or a machine); reject adds only the decision. The proposal file is
//! never rewritten, so its state is read from the decision that names it.

use std::{fs, path::Path};

use anyhow::{Context, Result};
use rfcspec_model::{
    Decision, DecisionKind, Edit, OVERLAY_FILE, Overlay, Requirement, from_yaml, text_digest,
    to_yaml,
};
use serde_json::Value;

use crate::{PROPOSALS_DIR, Task, Workspace, replay_one};

/// How a reviewer settles a proposal (ESS `rfcspec.review.Verdict`).
#[derive(Clone, Debug)]
pub enum Verdict {
    /// Take the proposed element as it is.
    Accept,
    /// Take this element instead, of the kind the proposal proposes; `None` is refused as
    /// [`ReviewError::DecisionIncomplete`].
    Edit(Option<Edit>),
    /// Record only the decision; the item stays unreviewed and is not proposed again.
    Reject,
}

impl Verdict {
    fn kind(&self) -> DecisionKind {
        match self {
            Self::Accept => DecisionKind::Accept,
            Self::Edit(_) => DecisionKind::Edit,
            Self::Reject => DecisionKind::Reject,
        }
    }
}

/// Why a review wrote nothing, each refusal by its ESS error name.
#[derive(Debug)]
pub enum ReviewError {
    /// No proposal of the profile has this id (`rfcspec.review.ProposalNotFound`).
    ProposalNotFound { proposal_id: String },
    /// The proposal already has a decision (`rfcspec.review.ProposalStateConflict`).
    ProposalStateConflict {
        proposal_id: String,
        state: &'static str,
    },
    /// The item already has a decision under its current text or an overlay status, or an
    /// accept or edit would add a layout or machine of a name the overlay holds or a
    /// requirement another overlay requirement already quotes (`rfcspec.review.AlreadyExists`).
    AlreadyExists { item: String, reason: String },
    /// An edit without its replacement entry (`rfcspec.review.DecisionIncomplete`).
    DecisionIncomplete { item: String },
    /// The proposal does not replay, the edit fails its checks, or the overlay with the
    /// decision fails its check: one problem per entry.
    Refused(Vec<String>),
    /// Anything else: an unreadable profile or file.
    Other(anyhow::Error),
}

impl std::fmt::Display for ReviewError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ProposalNotFound { proposal_id } => write!(
                f,
                "ProposalNotFound: no proposal {proposal_id:?} is recorded under {PROPOSALS_DIR}/"
            ),
            Self::ProposalStateConflict { proposal_id, state } => write!(
                f,
                "ProposalStateConflict: proposal {proposal_id} is already {state}"
            ),
            Self::AlreadyExists { item, reason } => write!(f, "AlreadyExists: {item}: {reason}"),
            Self::DecisionIncomplete { item } => write!(
                f,
                "DecisionIncomplete: {item}: an edit names no replacement entry"
            ),
            Self::Refused(problems) => write!(f, "{}", problems.join("\n")),
            Self::Other(error) => write!(f, "{error:#}"),
        }
    }
}

impl std::error::Error for ReviewError {}

/// A decision and the overlay that holds it, not yet written.
#[derive(Debug)]
pub struct Reviewed {
    /// The decision recorded.
    pub decision: Decision,
    /// The overlay with the decision and, for accept and edit, the element.
    pub overlay: Overlay,
    /// The overlay file's new bytes: the old file with the element and the decision inserted,
    /// so hand-written entries keep their form.
    text: String,
}

/// A recorded proposal as review reads it.
#[derive(Clone, Debug)]
pub struct Recorded {
    pub proposal_id: String,
    /// The item's stable id: `rfc<N>/<catalogue id>`, `rfc<N>/layout/<name>` or the profile.
    pub item: String,
    pub task: Task,
    pub response: Value,
}

/// Whether `id` is a proposal id: 64 lowercase hex digits (ESS `rfcspec.review.ProposalId`).
fn is_proposal_id(id: &str) -> bool {
    id.len() == 64
        && id
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The proposal `proposal_id` recorded under the profile at `dir`; `None` when no such file
/// exists (or the id is no proposal id).
pub fn recorded(dir: &Path, proposal_id: &str) -> Result<Option<Recorded>> {
    if !is_proposal_id(proposal_id) {
        return Ok(None);
    }
    let path = dir.join(PROPOSALS_DIR).join(format!("{proposal_id}.json"));
    if !path.is_file() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
    let document: Value =
        serde_json::from_str(&text).with_context(|| format!("parsing {}", path.display()))?;
    let field = |name: &str| {
        document
            .get(name)
            .and_then(Value::as_str)
            .with_context(|| format!("{}: no {name}", path.display()))
    };
    let task = field("task")?;
    Ok(Some(Recorded {
        proposal_id: proposal_id.to_owned(),
        item: field("item")?.to_owned(),
        task: Task::from_wire(task)
            .with_context(|| format!("{}: unknown task {task:?}", path.display()))?,
        response: document
            .get("response")
            .cloned()
            .with_context(|| format!("{}: no response", path.display()))?,
    }))
}

/// The digest of the text proposal `proposal_id` of the profile at `dir` judged, under the
/// current source, and the name of the layout or machine it proposes; an error when it is not
/// recorded or does not replay.
pub fn judged(dir: &Path, proposal_id: &str) -> Result<(String, Option<String>)> {
    let workspace = Workspace::load(dir)?;
    let path = dir.join(PROPOSALS_DIR).join(format!("{proposal_id}.json"));
    let replayed = replay_one(&workspace, &path)?;
    let name = match replayed.checked.element {
        Edit::Requirement(_) => None,
        Edit::Layout(layout) => Some(layout.name),
        Edit::Machine(machine) => Some(machine.name),
    };
    Ok((text_digest(&replayed.subject.text), name))
}

/// Settle proposal `proposal_id` of the profile at `dir` with `verdict`: the proposal must replay,
/// have no decision, and its item neither a decision under its current text nor an overlay
/// status (for a prose candidate, also under another id that quotes it, see
/// [`Workspace::covering`]); an accept or edit must not add a layout or machine of a name the
/// overlay holds, nor a requirement entry an overlay requirement of another id already quotes
/// (a reject adds no element, so neither applies); an edit must be of the kind the
/// proposal proposes, and an edited layout or machine pass the proposal's checks; the overlay
/// with the decision (and, for accept and edit, the element: a requirement entry inserted in
/// catalogue order, a layout or machine appended) must pass the overlay check, apart from
/// problems the overlay already had. Writes nothing; see [`write`].
pub fn decide(dir: &Path, proposal_id: &str, verdict: Verdict) -> Result<Reviewed, ReviewError> {
    let not_found = || ReviewError::ProposalNotFound {
        proposal_id: proposal_id.to_owned(),
    };
    recorded(dir, proposal_id)
        .map_err(ReviewError::Other)?
        .ok_or_else(not_found)?;
    let workspace = Workspace::load(dir).map_err(ReviewError::Other)?;
    let path = dir.join(PROPOSALS_DIR).join(format!("{proposal_id}.json"));
    let replayed = replay_one(&workspace, &path)
        .map_err(|error| ReviewError::Refused(vec![format!("{}: {error:#}", path.display())]))?;

    let overlay = &workspace.overlay;
    if let Some(decision) = overlay
        .decisions
        .iter()
        .find(|d| d.proposal_id.as_deref() == Some(proposal_id))
    {
        return Err(ReviewError::ProposalStateConflict {
            proposal_id: proposal_id.to_owned(),
            state: match decision.kind {
                DecisionKind::Accept => "accepted",
                DecisionKind::Edit => "edited",
                DecisionKind::Reject => "rejected",
            },
        });
    }
    let task = replayed.task;
    let subject = &replayed.subject;
    let item = subject.item.clone();
    let digest = text_digest(&subject.text);
    if let Some(decision) = overlay
        .decisions
        .iter()
        .find(|d| d.item == item && d.text_digest == digest)
    {
        return Err(ReviewError::AlreadyExists {
            item,
            reason: format!(
                "the overlay already holds a {} decision under text_digest {digest}",
                decision.kind
            ),
        });
    }
    if task.element() == "requirement" && overlay.requirements.iter().any(|r| r.id == subject.id) {
        return Err(ReviewError::AlreadyExists {
            reason: format!(
                "the overlay already gives requirement {} a status",
                subject.id
            ),
            item,
        });
    }
    if task == Task::RequirementFromProse
        && let Some(existing) = workspace.covering(&subject.text, subject.span, &subject.id)
    {
        return Err(ReviewError::AlreadyExists {
            reason: format!(
                "the overlay already gives {} a status as requirement {}",
                subject.id, existing.id
            ),
            item,
        });
    }

    let kind = verdict.kind();
    let (element, edit) = match verdict {
        Verdict::Accept => (replayed.checked.element.clone(), None),
        Verdict::Edit(None) => return Err(ReviewError::DecisionIncomplete { item }),
        Verdict::Edit(Some(edit)) => {
            if edit.kind() != task.element() {
                return Err(ReviewError::Refused(vec![format!(
                    "{item}: the edit is a {}, the proposal a {}",
                    edit.kind(),
                    task.element()
                )]));
            }
            if matches!(edit, Edit::Layout(_) | Edit::Machine(_)) {
                let value = serde_json::to_value(&edit)
                    .map_err(|error| ReviewError::Other(error.into()))?;
                workspace
                    .check(task, subject, &value)
                    .map_err(|reason| ReviewError::Refused(vec![format!("{item}: {reason}")]))?;
            }
            (edit.clone(), Some(edit))
        }
        Verdict::Reject => (replayed.checked.element.clone(), None),
    };
    let held =
        match &element {
            _ if kind == DecisionKind::Reject => None,
            Edit::Requirement(entry) => workspace
                .covering(&entry.quote, entry.span, &entry.id)
                .map(|existing| {
                    format!(
                        "requirement {}, which quotes the entry's text at {}",
                        existing.id, existing.span
                    )
                }),
            Edit::Layout(layout) => overlay
                .layouts
                .iter()
                .any(|l| l.name == layout.name)
                .then(|| format!("layout {}", layout.name)),
            Edit::Machine(machine) => overlay
                .machines
                .iter()
                .any(|m| m.name == machine.name)
                .then(|| format!("machine {}", machine.name)),
        };
    if let Some(held) = held {
        return Err(ReviewError::AlreadyExists {
            item,
            reason: format!("the overlay already holds {held}"),
        });
    }
    let decision = Decision {
        item,
        text_digest: digest,
        kind,
        proposal_id: Some(proposal_id.to_owned()),
        edit,
    };
    let mut changed = overlay.clone();
    let insert = if kind == DecisionKind::Reject {
        None
    } else {
        Some(match element {
            Edit::Requirement(entry) => {
                let key = workspace.order(&entry);
                let at = overlay
                    .requirements
                    .iter()
                    .position(|r| workspace.order(r) > key)
                    .unwrap_or(overlay.requirements.len());
                changed.requirements.insert(at, entry.clone());
                Insert {
                    key: "requirements",
                    at,
                    count: overlay.requirements.len(),
                    yaml: to_yaml(&[entry]).map_err(ReviewError::Other)?,
                }
            }
            Edit::Layout(layout) => {
                changed.layouts.push(layout.clone());
                Insert {
                    key: "layouts",
                    at: overlay.layouts.len(),
                    count: overlay.layouts.len(),
                    yaml: to_yaml(&[layout]).map_err(ReviewError::Other)?,
                }
            }
            Edit::Machine(machine) => {
                changed.machines.push(machine.clone());
                Insert {
                    key: "machines",
                    at: overlay.machines.len(),
                    count: overlay.machines.len(),
                    yaml: to_yaml(&[machine]).map_err(ReviewError::Other)?,
                }
            }
        })
    };
    changed.decisions.push(decision.clone());
    let before = rfcspec_extract::check(overlay, &workspace.catalogue, &workspace.source);
    let problems: Vec<String> =
        rfcspec_extract::check(&changed, &workspace.catalogue, &workspace.source)
            .into_iter()
            .filter(|problem| !before.contains(problem))
            .collect();
    if !problems.is_empty() {
        return Err(ReviewError::Refused(problems));
    }
    let original = fs::read_to_string(dir.join(OVERLAY_FILE))
        .with_context(|| format!("reading {}", dir.join(OVERLAY_FILE).display()))
        .map_err(ReviewError::Other)?;
    let text =
        render(&original, overlay, &changed, insert, &decision).map_err(ReviewError::Other)?;
    Ok(Reviewed {
        decision,
        overlay: changed,
        text,
    })
}

/// Write the reviewed overlay to the profile at `dir`.
pub fn write(dir: &Path, reviewed: &Reviewed) -> Result<()> {
    let path = dir.join(OVERLAY_FILE);
    fs::write(&path, &reviewed.text).with_context(|| format!("writing {}", path.display()))
}

impl Workspace {
    /// Where a requirement entry sorts in catalogue order: the first source line of the
    /// catalogue item with its id, else of its own span.
    fn order(&self, entry: &Requirement) -> usize {
        self.catalogue
            .requirements
            .iter()
            .find(|r| r.id == entry.id)
            .map(|r| r.span.start)
            .or_else(|| {
                self.catalogue
                    .prose_candidates
                    .iter()
                    .find(|c| c.id == entry.id)
                    .map(|c| c.span.start)
            })
            .unwrap_or(entry.span.start)
    }
}

/// One element to insert into the overlay file: the YAML of a one-element sequence, inserted
/// as item `at` of the sequence under top-level `key`, which holds `count` items.
struct Insert {
    key: &'static str,
    at: usize,
    count: usize,
    yaml: String,
}

/// The overlay file with the element inserted and `decision` appended, keeping every other
/// line as written; when the file's shape is not the expected one, or the text read back is not
/// `changed`, the whole overlay serialized instead.
fn render(
    original: &str,
    overlay: &Overlay,
    changed: &Overlay,
    insert: Option<Insert>,
    decision: &Decision,
) -> Result<String> {
    let inserted = insert_lines(original, overlay, insert, decision)?;
    match inserted {
        Some(text) if from_yaml::<Overlay>(&text).ok().as_ref() == Some(changed) => Ok(text),
        _ => to_yaml(changed),
    }
}

/// `yaml`, a block sequence, with each line prefixed by `indent`.
fn indented(yaml: &str, indent: &str) -> String {
    yaml.lines()
        .map(|line| format!("{indent}{line}\n"))
        .collect()
}

/// The line indexes at which the block sequence under the top-level key `key` has its items,
/// the item indent, and the index of the line after the block; `None` when the key is absent or
/// not followed by a block sequence.
fn sequence(lines: &[&str], key: &str) -> Option<(Vec<usize>, String, usize)> {
    let head = lines.iter().position(|line| *line == format!("{key}:"))?;
    let end = (head + 1..lines.len())
        .find(|&i| {
            let line = lines[i];
            !line.is_empty() && !line.starts_with([' ', '-'])
        })
        .unwrap_or(lines.len());
    let first = lines.get(head + 1)?;
    let indent = &first[..first.len() - first.trim_start().len()];
    if !first.trim_start().starts_with("- ") {
        return None;
    }
    let starts = (head + 1..end)
        .filter(|&i| {
            lines[i]
                .strip_prefix(indent)
                .is_some_and(|rest| rest.starts_with("- "))
        })
        .collect();
    Some((starts, indent.to_owned(), end))
}

fn insert_lines(
    original: &str,
    overlay: &Overlay,
    insert: Option<Insert>,
    decision: &Decision,
) -> Result<Option<String>> {
    if !original.ends_with('\n') {
        return Ok(None);
    }
    let mut lines: Vec<String> = original.lines().map(str::to_owned).collect();
    if let Some(insert) = insert {
        let view: Vec<&str> = lines.iter().map(String::as_str).collect();
        let empty = format!("{}: []", insert.key);
        if let Some(head) = view.iter().position(|line| *line == empty) {
            if insert.count != 0 {
                return Ok(None);
            }
            let text = indented(&insert.yaml, "  ");
            lines.splice(
                head..=head,
                std::iter::once(format!("{}:", insert.key)).chain(text.lines().map(str::to_owned)),
            );
        } else {
            let Some((starts, indent, end)) = sequence(&view, insert.key) else {
                return Ok(None);
            };
            if starts.len() != insert.count {
                return Ok(None);
            }
            let line = starts.get(insert.at).copied().unwrap_or(end);
            let text = indented(&insert.yaml, &indent);
            lines.splice(line..line, text.lines().map(str::to_owned));
        }
    }
    let view: Vec<&str> = lines.iter().map(String::as_str).collect();
    let appended = if overlay.decisions.is_empty() {
        if view.iter().any(|line| line.starts_with("decisions:")) {
            return Ok(None);
        }
        format!("decisions:\n{}", to_yaml(&[decision])?)
    } else {
        let Some((_, indent, end)) = sequence(&view, "decisions") else {
            return Ok(None);
        };
        if end != view.len() {
            return Ok(None);
        }
        indented(&to_yaml(&[decision])?, &indent)
    };
    let mut text: String = lines.iter().map(|line| format!("{line}\n")).collect();
    text.push_str(&appended);
    Ok(Some(text))
}
