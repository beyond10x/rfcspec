//! The RFC explorer's data under `website/static/data/rfcs/`: one `rfcspec-explorer/1` shard per
//! hundred RFC numbers that hold a summary row (`00000.json`, `00100.json` and so on), generated
//! from the bulk summary `bulk/rfc-summary.jsonl` and its section sidecar `bulk/rfc-sections.jsonl`.
//! A shard is written through the types `ess generate types --target rust` generates from
//! `rfcspec.catalogue.ExplorerShard` (`generated/rust/rfcspec-explorer-types`); the explorer app
//! (`explorer/`) reads it through the TypeScript declarations generated from the same type. It
//! holds no requirement text.
//!
//! Beside the shards, the index `website/static/data/rfc-index.json` (`rfcspec-explorer-index/1`,
//! `rfcspec.catalogue.ExplorerIndex`) lists every summary row with its metadata, counts by level
//! and the profiles locking it, no SHA-256; the explorer's front page filters it in the browser.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result, bail, ensure};
use rfcspec_explorer_types::{
    Bcp14Level, Digest, EssPresence, ExplorerIndex, ExplorerIndexRow, ExplorerReference,
    ExplorerRfc, ExplorerSection, ExplorerShard, LevelCount, ReferenceKind, RfcNumber,
    SourceFormat, YearMonth,
};
use rfcspec_model::{
    BulkHeader, BulkRow, Constraint, Decision, DecisionKind, Layout, Machine, Requirement,
    SECTIONS_FORMAT, SectionsRow, Status, sha256_hex,
};
use serde_json::Number;

/// The generated types of `rfcspec.profile.ExplorerProfile` and the view rows it holds, by the
/// last segment of their qualified names.
mod profile_types {
    pub use rfcspec_explorer_profile_types::{
        EssPresence, RfcspecCatalogueConstraintKind as ConstraintKind,
        RfcspecCatalogueDigest as Digest, RfcspecCatalogueFieldConstraint as FieldConstraint,
        RfcspecCatalogueItemId as ItemId, RfcspecCatalogueLayoutField as LayoutField,
        RfcspecCatalogueLayoutState as LayoutState, RfcspecCatalogueLevelCount as LevelCount,
        RfcspecCataloguePresence as Presence, RfcspecCatalogueRfcNumber as RfcNumber,
        RfcspecProfileExplorerDecision as ExplorerDecision,
        RfcspecProfileExplorerDecisionRow as ExplorerDecisionRow,
        RfcspecProfileExplorerLayout as ExplorerLayout,
        RfcspecProfileExplorerLayoutRow as ExplorerLayoutRow,
        RfcspecProfileExplorerMachine as ExplorerMachine,
        RfcspecProfileExplorerProfile as ExplorerProfile,
        RfcspecProfileExplorerRequirement as ExplorerRequirement,
        RfcspecProfileExplorerStatus as ExplorerStatus, RfcspecProfileMachineKey as MachineKey,
        RfcspecProfileMachineState as MachineState, RfcspecProfileProfileName as ProfileName,
        RfcspecProfileStatusKey as StatusKey, RfcspecProfileStatusKind as StatusKind,
        RfcspecProfileStatusState as StatusState, RfcspecReviewDecisionKey as DecisionKey,
        RfcspecReviewDecisionKind as DecisionKind, RfcspecReviewDecisionState as DecisionState,
        RfcspecReviewModelId as ModelId, RfcspecReviewProposalId as ProposalId,
    };
}

use crate::{
    bulk::{SUMMARY, Summary, locked},
    catalog::{Entry, rank},
    page::BASE_URL,
};

/// The section sidecar of the bulk summary, relative to the repository root.
pub const SECTIONS: &str = "bulk/rfc-sections.jsonl";
/// Where the shards land, relative to the repository root.
pub const DIR: &str = "website/static/data/rfcs";
/// The shard format.
pub const FORMAT: &str = "rfcspec-explorer/1";
/// The index the explorer's front page lists and filters, relative to the repository root.
pub const INDEX: &str = "website/static/data/rfc-index.json";
/// The index format.
pub const INDEX_FORMAT: &str = "rfcspec-explorer-index/1";

/// The explorer's view of an RFC, as a link a generated page can carry.
pub fn link(rfc: u32) -> String {
    format!("pathname://{BASE_URL}explorer/?rfc={rfc}")
}

/// The explorer's front page, the filtered list of every RFC, as a link a generated page can carry.
pub fn front() -> String {
    format!("pathname://{BASE_URL}explorer/")
}

/// A section sidecar, read: its header and rows.
pub struct Sidecar {
    pub header: BulkHeader,
    pub rows: Vec<SectionsRow>,
}

/// Read a sidecar's text. Refused, naming the line, when a line does not parse, the format is
/// another or the header counts other rows; the full check is `rfcspec bulk check --sections`.
pub fn parse(text: &str) -> Result<Sidecar> {
    let mut lines = text.lines().enumerate();
    let Some((_, first)) = lines.next() else {
        bail!("the sidecar is empty");
    };
    let header: BulkHeader =
        serde_json::from_str(first).context("line 1: not an rfcspec-sections/1 header")?;
    ensure!(
        header.format == SECTIONS_FORMAT,
        "line 1: format {} is not {SECTIONS_FORMAT}",
        header.format
    );
    let rows = lines
        .map(|(n, line)| {
            serde_json::from_str(line).with_context(|| format!("line {}: not a sidecar row", n + 1))
        })
        .collect::<Result<Vec<SectionsRow>>>()?;
    ensure!(
        header.rows == rows.len(),
        "line 1: the header counts {} rows, the sidecar holds {}",
        header.rows,
        rows.len()
    );
    Ok(Sidecar { header, rows })
}

/// The first number of the hundred an RFC's shard holds.
fn range(rfc: u32) -> u32 {
    rfc / 100 * 100
}

/// The path of a range's shard, by its first number.
pub fn path(start: u32) -> String {
    format!("{DIR}/{start:05}.json")
}

/// Counts by level as the generated type lists them, in BCP 14 order.
fn levels(counts: &BTreeMap<String, u64>) -> Result<Vec<LevelCount>> {
    let mut ranked: Vec<(usize, &String, u64)> = counts
        .iter()
        .map(|(level, n)| (rank(level), level, *n))
        .collect();
    ranked.sort_unstable();
    ranked
        .into_iter()
        .map(|(_, level, n)| {
            let level: Bcp14Level = serde_json::from_value(level.as_str().into())
                .with_context(|| format!("{level:?} is not a BCP 14 level"))?;
            Ok(LevelCount {
                level,
                count: Number::from(n),
            })
        })
        .collect()
}

fn numbers(numbers: &[u32]) -> Vec<Number> {
    numbers.iter().map(|n| Number::from(*n)).collect()
}

fn present(value: Option<&String>) -> EssPresence<String> {
    value.map_or(EssPresence::Absent, |value| {
        EssPresence::Present(value.clone())
    })
}

fn format(format: rfcspec_model::SourceFormat) -> SourceFormat {
    match format {
        rfcspec_model::SourceFormat::Xml => SourceFormat::Xml,
        rfcspec_model::SourceFormat::Text => SourceFormat::Text,
    }
}

/// One RFC as the explorer reads it: its summary row and its sidecar row.
fn rfc(row: &BulkRow, sections: &SectionsRow) -> Result<ExplorerRfc> {
    Ok(ExplorerRfc {
        rfc: RfcNumber(Number::from(row.rfc)),
        title: row.title.clone(),
        status: row.status.clone(),
        stream: row.stream.clone(),
        area: present(row.area.as_ref()),
        date: YearMonth(row.date.clone()),
        format: format(row.format),
        sha256: Digest(row.sha256.clone()),
        counts: levels(&row.counts)?,
        obsoletes: numbers(&row.obsoletes),
        obsoleted_by: numbers(&row.obsoleted_by),
        updates: numbers(&row.updates),
        updated_by: numbers(&row.updated_by),
        sections: sections
            .sections
            .iter()
            .map(|section| {
                Ok(ExplorerSection {
                    number: present(section.number.as_ref()),
                    anchor: present(section.anchor.as_ref()),
                    title: present(section.title.as_ref()),
                    counts: levels(&section.counts)?,
                })
            })
            .collect::<Result<_>>()?,
        references: sections
            .references
            .iter()
            .map(|reference| ExplorerReference {
                section: present(reference.section.as_ref()),
                target_rfc: Number::from(reference.target_rfc),
                target_section: present(reference.target_section.as_ref()),
                kind: match reference.kind {
                    rfcspec_model::ReferenceKind::Normative => ReferenceKind::Normative,
                    rfcspec_model::ReferenceKind::Informative => ReferenceKind::Informative,
                },
            })
            .collect(),
    })
}

/// A document's JSON given its other members serialised around an empty `rfcs` list: the members
/// in declared order with one RFC per line, so a changed RFC changes one line.
fn one_per_line(head: &str, rows: &[String]) -> Result<String> {
    let Some((before, after)) = head.split_once("\"rfcs\":[]") else {
        bail!("a document without its rfcs member: {head}");
    };
    Ok(format!(
        "{before}\"rfcs\":[\n{}\n]{after}\n",
        rows.join(",\n")
    ))
}

/// A shard's JSON, one RFC per line.
fn write(shard: &ExplorerShard) -> Result<String> {
    let head = ExplorerShard {
        rfcs: Vec::new(),
        ..shard.clone()
    };
    one_per_line(
        &serde_json::to_string(&head)?,
        &shard
            .rfcs
            .iter()
            .map(serde_json::to_string)
            .collect::<serde_json::Result<Vec<_>>>()?,
    )
}

/// The explorer index of a summary: one row per summary row, in its order, each with the name of
/// every profile whose lock holds it, in the order `profiles` lists them; one RFC per line.
pub fn index(summary: &Summary, profiles: &[(String, Vec<u32>)]) -> Result<String> {
    let mut locking: BTreeMap<u32, Vec<String>> = BTreeMap::new();
    for (name, rfcs) in profiles {
        for rfc in rfcs {
            locking.entry(*rfc).or_default().push(name.clone());
        }
    }
    let rfcs = summary
        .rows
        .iter()
        .map(|row| {
            Ok(ExplorerIndexRow {
                rfc: RfcNumber(Number::from(row.rfc)),
                title: row.title.clone(),
                status: row.status.clone(),
                stream: row.stream.clone(),
                area: present(row.area.as_ref()),
                date: YearMonth(row.date.clone()),
                format: format(row.format),
                counts: levels(&row.counts)?,
                profiles: locking.remove(&row.rfc).unwrap_or_default(),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let head = ExplorerIndex {
        format: INDEX_FORMAT.into(),
        rfcs: Vec::new(),
    };
    one_per_line(
        &serde_json::to_string(&head)?,
        &rfcs
            .iter()
            .map(serde_json::to_string)
            .collect::<serde_json::Result<Vec<_>>>()?,
    )
}

/// The shards of a summary and its sidecar, by path relative to the repository root. Refused
/// when the sidecar is of another index or extraction or does not hold the summary's RFCs in
/// its order.
pub fn render(summary: &Summary, sidecar: &Sidecar) -> Result<BTreeMap<String, String>> {
    ensure!(
        sidecar.header.index_sha256 == summary.header.index_sha256
            && sidecar.header.extractor == summary.header.extractor,
        "{SECTIONS} is of index {} extractor {}, {SUMMARY} of index {} extractor {}; run `task bulk-regenerate`",
        sidecar.header.index_sha256,
        sidecar.header.extractor,
        summary.header.index_sha256,
        summary.header.extractor
    );
    let summary_numbers: Vec<u32> = summary.rows.iter().map(|row| row.rfc).collect();
    let sidecar_numbers: Vec<u32> = sidecar.rows.iter().map(|row| row.rfc).collect();
    ensure!(
        summary_numbers == sidecar_numbers,
        "{SECTIONS} does not hold the RFCs of {SUMMARY} in its order; run `task bulk-regenerate`"
    );
    let indexed: BTreeSet<u32> = summary_numbers.iter().copied().collect();
    let mut ranges: BTreeMap<u32, Vec<ExplorerRfc>> = BTreeMap::new();
    let mut absent: BTreeMap<u32, BTreeSet<u32>> = BTreeMap::new();
    for (row, sections) in summary.rows.iter().zip(&sidecar.rows) {
        let start = range(row.rfc);
        let named = [
            &row.obsoletes,
            &row.obsoleted_by,
            &row.updates,
            &row.updated_by,
        ]
        .into_iter()
        .flatten()
        .copied()
        .chain(sections.references.iter().map(|r| r.target_rfc));
        absent
            .entry(start)
            .or_default()
            .extend(named.filter(|n| !indexed.contains(n)));
        ranges.entry(start).or_default().push(rfc(row, sections)?);
    }
    let mut out = BTreeMap::new();
    for (start, rfcs) in ranges {
        let shard = ExplorerShard {
            format: FORMAT.into(),
            index_sha256: Digest(summary.header.index_sha256.clone()),
            start: Number::from(start),
            rfcs,
            not_in_index: absent
                .remove(&start)
                .unwrap_or_default()
                .into_iter()
                .map(Number::from)
                .collect(),
        };
        out.insert(path(start), write(&shard)?);
    }
    Ok(out)
}

/// Every shard and the index, by path relative to the repository root; none without a summary.
/// Refused when the summary has no sidecar.
pub fn outputs(root: &Path) -> Result<BTreeMap<String, String>> {
    let summary_path = root.join(SUMMARY);
    if !summary_path.is_file() {
        return Ok(BTreeMap::new());
    }
    let summary = crate::bulk::parse(
        &fs::read_to_string(&summary_path).with_context(|| format!("reading {SUMMARY}"))?,
    )
    .with_context(|| format!("reading {SUMMARY}"))?;
    let sidecar = parse(
        &fs::read_to_string(root.join(SECTIONS)).with_context(|| format!("reading {SECTIONS}"))?,
    )
    .with_context(|| format!("reading {SECTIONS}"))?;
    let mut out = render(&summary, &sidecar)?;
    out.insert(INDEX.to_owned(), index(&summary, &locked(root)?)?);
    Ok(out)
}

/// Where the profile files land, relative to the repository root.
pub const PROFILES_DIR: &str = "website/static/data/profiles";
/// The profile file format.
pub const PROFILE_FORMAT: &str = "rfcspec-explorer-profile/1";

/// The path of a profile's file, by its name.
pub fn profile_path(name: &str) -> String {
    format!("{PROFILES_DIR}/{name}.json")
}

fn absent_or<T, U>(value: Option<T>, f: impl FnOnce(T) -> U) -> profile_types::EssPresence<U> {
    value.map_or(profile_types::EssPresence::Absent, |value| {
        profile_types::EssPresence::Present(f(value))
    })
}

/// A JSON number of a count or line.
fn count(n: usize) -> Number {
    Number::from(n as u64)
}

/// The status row of one overlay requirement, `rfcspec.profile.Statuses` field for field.
fn status_row(name: &str, rfc: u32, requirement: &Requirement) -> profile_types::ExplorerStatus {
    let id = profile_types::ItemId(format!("rfc{rfc}/{}", requirement.id));
    let (kind, element, reason) = match &requirement.status {
        Status::Satisfied(element) => (profile_types::StatusKind::Satisfied, Some(element), None),
        Status::Unsupported(reason) => (profile_types::StatusKind::Unsupported, None, Some(reason)),
        Status::NotApplicable(reason) => {
            (profile_types::StatusKind::NotApplicable, None, Some(reason))
        }
    };
    profile_types::ExplorerStatus {
        key: profile_types::StatusKey {
            profile: profile_types::ProfileName(name.to_owned()),
            requirement: id.clone(),
        },
        state: profile_types::StatusState::Recorded,
        profile: profile_types::ProfileName(name.to_owned()),
        requirement_id: id,
        kind,
        element: absent_or(element, Clone::clone),
        reason: absent_or(reason, Clone::clone),
        evidence: profile_types::EssPresence::Absent,
    }
}

fn requirement(
    name: &str,
    rfc: u32,
    requirement: &Requirement,
) -> profile_types::ExplorerRequirement {
    let gap = match &requirement.status {
        Status::Unsupported(reason) => crate::catalog::gap(reason),
        _ => None,
    };
    profile_types::ExplorerRequirement {
        status: status_row(name, rfc, requirement),
        level: requirement.level.clone(),
        transports: requirement.transports.clone(),
        quote: requirement.quote.clone(),
        start_line: count(requirement.span.start),
        end_line: count(requirement.span.end),
        gap: absent_or(gap, Number::from),
    }
}

/// One decision with the proposal it decided: its file and the model id the file records.
/// Refused when the file is missing or records another id; `rfcspec check` refuses the same.
fn decision(
    root: &Path,
    entry: &Entry,
    decision: &Decision,
) -> Result<profile_types::ExplorerDecision> {
    let (model_id, proposal_path) = match &decision.proposal_id {
        None => (None, None),
        Some(id) => {
            let path = format!("{}/proposals/{id}.json", entry.path);
            let file: serde_json::Value = serde_json::from_str(
                &fs::read_to_string(root.join(&path)).with_context(|| format!("reading {path}"))?,
            )
            .with_context(|| format!("reading {path}"))?;
            ensure!(
                file["proposal_id"] == id.as_str(),
                "{path} records another proposal id"
            );
            let model = file["model_id"]
                .as_str()
                .with_context(|| format!("{path} records no model id"))?
                .to_owned();
            (Some(model), Some(path))
        }
    };
    let name = || profile_types::ProfileName(entry.name.clone());
    Ok(profile_types::ExplorerDecision {
        row: profile_types::ExplorerDecisionRow {
            key: profile_types::DecisionKey {
                profile: name(),
                item: profile_types::ItemId(decision.item.clone()),
                text_digest: profile_types::Digest(decision.text_digest.clone()),
            },
            state: profile_types::DecisionState::Current,
            profile: name(),
            kind: match decision.kind {
                DecisionKind::Accept => profile_types::DecisionKind::Accept,
                DecisionKind::Edit => profile_types::DecisionKind::Edit,
                DecisionKind::Reject => profile_types::DecisionKind::Reject,
            },
            proposal_id: absent_or(decision.proposal_id.as_ref(), |id| {
                profile_types::ProposalId(id.clone())
            }),
            edit: profile_types::EssPresence::Absent,
            errata_id: profile_types::EssPresence::Absent,
            errata_status: profile_types::EssPresence::Absent,
            corrected_text: profile_types::EssPresence::Absent,
            reason: absent_or(decision.reason.as_ref(), Clone::clone),
            confirms: profile_types::EssPresence::Absent,
        },
        model_id: absent_or(model_id, profile_types::ModelId),
        proposal_path: absent_or(proposal_path, |path| path),
    })
}

fn machine(
    root: &Path,
    entry: &Entry,
    machine: &Machine,
) -> Result<profile_types::ExplorerMachine> {
    let file = format!("{}.protospec.yaml", machine.name);
    let protospec_path = format!("{}/generated/{file}", entry.path);
    let lowered = fs::read(root.join(&protospec_path))
        .with_context(|| format!("reading {protospec_path}"))?;
    let witness = format!("{}/witnesses/{}.inputs.json", entry.path, machine.name);
    ensure!(
        root.join(&witness).is_file(),
        "machine {} of {} has no witness {witness}",
        machine.name,
        entry.path
    );
    Ok(profile_types::ExplorerMachine {
        key: profile_types::MachineKey {
            profile: profile_types::ProfileName(entry.name.clone()),
            slug: machine.name.clone(),
        },
        state: profile_types::MachineState::Recorded,
        name: machine.name.clone(),
        protospec_path,
        protospec_digest: profile_types::Digest(sha256_hex(&lowered)),
        transport: machine.transport.clone(),
        download: format!("downloads/{}/{file}", entry.name),
        witness,
    })
}

fn layout(rfc: u32, layout: &Layout) -> profile_types::ExplorerLayout {
    profile_types::ExplorerLayout {
        row: profile_types::ExplorerLayoutRow {
            layout_id: profile_types::ItemId(format!("rfc{rfc}/layout/{}", layout.name)),
            state: profile_types::LayoutState::Read,
            rfc: profile_types::RfcNumber(Number::from(rfc)),
            name: layout.name.clone(),
        },
        fields: layout
            .fields
            .iter()
            .map(|field| profile_types::LayoutField {
                name: field.name.clone(),
                bits: profile_types::EssPresence::Present(Number::from(field.bits)),
                length: profile_types::EssPresence::Absent,
                presence: absent_or(field.presence.as_ref(), |p| profile_types::Presence {
                    field: p.field.clone(),
                    equals: Number::from(p.equals),
                }),
                constraints: field
                    .constraints
                    .iter()
                    .map(|c| profile_types::FieldConstraint {
                        kind: match c {
                            Constraint::Equals(_) => profile_types::ConstraintKind::Equals,
                            Constraint::Range(_) => profile_types::ConstraintKind::Range,
                            Constraint::OneOf(_) => profile_types::ConstraintKind::OneOf,
                        },
                        values: c.values().into_iter().map(Number::from).collect(),
                    })
                    .collect(),
            })
            .collect(),
    }
}

/// The unreviewed requirements of the entry's catalogue counted by level, as its page counts
/// them; none when the catalogue cannot be read.
fn unreviewed(entry: &Entry) -> Result<Option<Vec<profile_types::LevelCount>>> {
    let Some(found) = entry.unreviewed() else {
        return Ok(None);
    };
    let mut counts: BTreeMap<String, u64> = BTreeMap::new();
    for requirement in found {
        *counts.entry(requirement.keyword.clone()).or_default() += 1;
    }
    let mut ranked: Vec<(usize, String, u64)> = counts
        .into_iter()
        .map(|(level, n)| (rank(&level), level, n))
        .collect();
    ranked.sort_unstable();
    ranked
        .into_iter()
        .map(|(_, level, n)| {
            Ok(profile_types::LevelCount {
                level: serde_json::from_value(level.as_str().into())
                    .with_context(|| format!("{level:?} is not a BCP 14 level"))?,
                count: Number::from(n),
            })
        })
        .collect::<Result<_>>()
        .map(Some)
}

/// Write the elements of the list member `list`, serialised empty in `text`, one per line, so a
/// changed requirement or decision changes one line; an empty list stays `[]`.
fn per_line<T: serde::Serialize>(text: &mut String, list: &str, items: &[T]) -> Result<()> {
    let empty = format!("\"{list}\":[]");
    ensure!(
        text.matches(&empty).count() == 1,
        "{list} is not one member"
    );
    if !items.is_empty() {
        let rows = items
            .iter()
            .map(serde_json::to_string)
            .collect::<serde_json::Result<Vec<_>>>()?;
        *text = text.replacen(&empty, &format!("\"{list}\":[\n{}\n]", rows.join(",\n")), 1);
    }
    Ok(())
}

/// The reviewed layer of one profile as the explorer reads it; none without an overlay.
pub fn profile(root: &Path, entry: &Entry) -> Result<Option<String>> {
    let Some(overlay) = &entry.overlay else {
        return Ok(None);
    };
    let document = profile_types::ExplorerProfile {
        format: PROFILE_FORMAT.into(),
        profile: profile_types::ProfileName(entry.name.clone()),
        path: entry.path.clone(),
        rfc: profile_types::RfcNumber(Number::from(overlay.rfc)),
        unreviewed: absent_or(unreviewed(entry)?, |counts| counts),
        requirements: overlay
            .requirements
            .iter()
            .map(|r| requirement(&entry.name, overlay.rfc, r))
            .collect(),
        decisions: overlay
            .decisions
            .iter()
            .map(|d| decision(root, entry, d))
            .collect::<Result<_>>()?,
        machines: overlay
            .machines
            .iter()
            .map(|m| machine(root, entry, m))
            .collect::<Result<_>>()?,
        layouts: overlay
            .layouts
            .iter()
            .map(|l| layout(overlay.rfc, l))
            .collect(),
    };
    let head = profile_types::ExplorerProfile {
        requirements: Vec::new(),
        decisions: Vec::new(),
        machines: Vec::new(),
        layouts: Vec::new(),
        ..document.clone()
    };
    let mut text = serde_json::to_string(&head)?;
    per_line(&mut text, "requirements", &document.requirements)?;
    per_line(&mut text, "decisions", &document.decisions)?;
    per_line(&mut text, "machines", &document.machines)?;
    per_line(&mut text, "layouts", &document.layouts)?;
    text.push('\n');
    Ok(Some(text))
}

/// The file of every profile with an overlay, by path relative to the repository root.
pub fn profiles(root: &Path, entries: &[Entry]) -> Result<BTreeMap<String, String>> {
    let mut out = BTreeMap::new();
    for entry in entries {
        if let Some(text) = profile(root, entry)? {
            out.insert(profile_path(&entry.name), text);
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rfcspec_model::{
        Reference, ReferenceKind as Kind, SectionCounts, SourceFormat as Format, bulk_summary,
        sections_sidecar,
    };

    use super::*;

    fn repository() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn row(rfc: u32, updated_by: &[u32], counts: &[(&str, u64)]) -> BulkRow {
        BulkRow {
            rfc,
            title: format!("RFC {rfc} title"),
            status: "PROPOSED STANDARD".into(),
            stream: "IETF".into(),
            area: None,
            date: "2020-01".into(),
            obsoletes: vec![],
            obsoleted_by: vec![],
            updates: vec![],
            updated_by: updated_by.to_vec(),
            format: Format::Text,
            sha256: "0".repeat(64),
            counts: counts.iter().map(|(l, n)| ((*l).into(), *n)).collect(),
            lossy: false,
        }
    }

    /// A summary of RFCs 1, 99, 1000 and 2500 and their sidecar; RFC 99 is updated by 1000 and
    /// 1001 (no row), RFC 2500 cites 99 and 4000 (no row).
    fn fixture() -> (Summary, Sidecar) {
        let rows = vec![
            row(1, &[], &[]),
            row(99, &[1000, 1001], &[("MAY", 1), ("MUST", 2)]),
            row(1000, &[], &[]),
            BulkRow {
                format: Format::Xml,
                ..row(2500, &[], &[("SHOULD", 1)])
            },
        ];
        let sections = vec![
            SectionsRow {
                rfc: 1,
                sections: vec![],
                references: vec![],
            },
            SectionsRow {
                rfc: 99,
                sections: vec![
                    SectionCounts {
                        number: Some("2".into()),
                        anchor: None,
                        title: Some("Protocol".into()),
                        counts: BTreeMap::from([("MAY".into(), 1), ("MUST".into(), 1)]),
                    },
                    SectionCounts {
                        number: None,
                        anchor: None,
                        title: None,
                        counts: BTreeMap::from([("MUST".into(), 1)]),
                    },
                ],
                references: vec![],
            },
            SectionsRow {
                rfc: 1000,
                sections: vec![],
                references: vec![],
            },
            SectionsRow {
                rfc: 2500,
                sections: vec![SectionCounts {
                    number: Some("3".into()),
                    anchor: Some("sec-three".into()),
                    title: Some("Three".into()),
                    counts: BTreeMap::from([("SHOULD".into(), 1)]),
                }],
                references: vec![
                    Reference {
                        section: Some("sec-three".into()),
                        target_rfc: 99,
                        target_section: Some("2".into()),
                        kind: Kind::Normative,
                    },
                    Reference {
                        section: None,
                        target_rfc: 4000,
                        target_section: None,
                        kind: Kind::Informative,
                    },
                ],
            },
        ];
        let index = "1".repeat(64);
        let summary = crate::bulk::parse(&bulk_summary(&index, 1, &rows).unwrap()).unwrap();
        let sidecar = parse(&sections_sidecar(&index, 1, &sections).unwrap()).unwrap();
        (summary, sidecar)
    }

    fn shard(text: &str) -> ExplorerShard {
        serde_json::from_str(text).unwrap()
    }

    fn number(n: &Number) -> u32 {
        u32::try_from(n.as_u64().unwrap()).unwrap()
    }

    #[test]
    fn a_shard_per_hundred_holds_its_rows_in_order_one_per_line_with_absent_numbers() {
        let (summary, sidecar) = fixture();
        let shards = render(&summary, &sidecar).unwrap();
        assert_eq!(
            shards.keys().collect::<Vec<_>>(),
            [
                "website/static/data/rfcs/00000.json",
                "website/static/data/rfcs/01000.json",
                "website/static/data/rfcs/02500.json",
            ]
        );
        let first = &shards["website/static/data/rfcs/00000.json"];
        assert_eq!(first.lines().count(), 4, "{first}");
        assert!(
            first.starts_with(&format!(
                "{{\"format\":\"rfcspec-explorer/1\",\"index_sha256\":\"{}\",\"start\":0,\"rfcs\":[\n{{\"rfc\":1,",
                "1".repeat(64)
            )),
            "{first}"
        );
        assert!(first.ends_with("\n],\"not_in_index\":[1001]}\n"), "{first}");
        let parsed = shard(first);
        assert_eq!(
            parsed
                .rfcs
                .iter()
                .map(|r| number(&r.rfc.0))
                .collect::<Vec<_>>(),
            [1, 99]
        );
        let rfc99 = &parsed.rfcs[1];
        assert_eq!(
            serde_json::to_value(&rfc99.counts).unwrap(),
            serde_json::json!([{"level": "MUST", "count": 2}, {"level": "MAY", "count": 1}])
        );
        assert_eq!(
            serde_json::to_value(&rfc99.sections).unwrap(),
            serde_json::json!([
                {"number": "2", "title": "Protocol", "counts": [{"level": "MUST", "count": 1}, {"level": "MAY", "count": 1}]},
                {"counts": [{"level": "MUST", "count": 1}]}
            ])
        );
        assert_eq!(
            rfc99.updated_by.iter().map(number).collect::<Vec<_>>(),
            [1000, 1001]
        );
        assert!(rfc99.references.is_empty());

        let third = shard(&shards["website/static/data/rfcs/02500.json"]);
        assert_eq!(
            third.not_in_index.iter().map(number).collect::<Vec<_>>(),
            [4000]
        );
        assert_eq!(
            serde_json::to_value(&third.rfcs[0].references).unwrap(),
            serde_json::json!([
                {"section": "sec-three", "target_rfc": 99, "target_section": "2", "kind": "normative"},
                {"target_rfc": 4000, "kind": "informative"}
            ])
        );
        assert_eq!(serde_json::to_value(&third.rfcs[0].format).unwrap(), "xml");
    }

    #[test]
    fn the_index_holds_one_row_per_summary_row_in_order_one_per_line_with_profiles_and_no_sha256() {
        let (summary, _) = fixture();
        let profiles = [
            ("dns".to_owned(), vec![99, 2500]),
            ("time".to_owned(), vec![99, 4000]),
        ];
        let text = index(&summary, &profiles).unwrap();
        assert!(
            text.starts_with(&format!(
                "{{\"format\":\"{INDEX_FORMAT}\",\"rfcs\":[\n{{\"rfc\":1,"
            )),
            "{text}"
        );
        assert!(text.ends_with("\n]}\n"), "{text}");
        assert_eq!(text.lines().count(), 6, "{text}");
        assert!(!text.contains("sha256"), "{text}");
        let parsed: ExplorerIndex = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed.format, INDEX_FORMAT);
        assert_eq!(
            parsed
                .rfcs
                .iter()
                .map(|r| number(&r.rfc.0))
                .collect::<Vec<_>>(),
            [1, 99, 1000, 2500]
        );
        assert_eq!(
            serde_json::to_value(&parsed.rfcs[1]).unwrap(),
            serde_json::json!({
                "rfc": 99,
                "title": "RFC 99 title",
                "status": "PROPOSED STANDARD",
                "stream": "IETF",
                "date": "2020-01",
                "format": "text",
                "counts": [{"level": "MUST", "count": 2}, {"level": "MAY", "count": 1}],
                "profiles": ["dns", "time"]
            })
        );
        assert_eq!(parsed.rfcs[3].profiles, ["dns"]);
        assert_eq!(serde_json::to_value(&parsed.rfcs[3].format).unwrap(), "xml");
        assert!(parsed.rfcs[0].profiles.is_empty());
        assert!(parsed.rfcs[0].counts.is_empty());
    }

    #[test]
    fn the_repository_index_holds_every_summary_row_in_order_with_the_profile_marks_under_3_mib() {
        let root = repository();
        let (summary, outputs) = committed();
        let text = fs::read_to_string(root.join(INDEX)).unwrap();
        assert_eq!(text, outputs[INDEX], "{INDEX} is current");
        assert!(
            text.len() < 3 * 1024 * 1024,
            "{INDEX} is {} bytes",
            text.len()
        );
        assert!(!text.contains("sha256"));
        let parsed: ExplorerIndex = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed.format, INDEX_FORMAT);
        let numbers: Vec<u32> = parsed.rfcs.iter().map(|r| number(&r.rfc.0)).collect();
        let expected: Vec<u32> = summary.rows.iter().map(|r| r.rfc).collect();
        assert_eq!(numbers, expected, "every summary row once, in number order");
        let marked: Vec<(u32, Vec<String>)> = parsed
            .rfcs
            .iter()
            .filter(|r| !r.profiles.is_empty())
            .map(|r| (number(&r.rfc.0), r.profiles.clone()))
            .collect();
        assert_eq!(
            marked,
            [
                (868, vec!["time".to_owned()]),
                (1035, vec!["dns".to_owned()]),
                (5905, vec!["ntp".to_owned()]),
                (6455, vec!["websocket".to_owned()]),
            ]
        );
        for (row, entry) in summary.rows.iter().zip(&parsed.rfcs) {
            assert_eq!(entry.title, row.title);
            assert_eq!(entry.status, row.status);
            assert_eq!(
                serde_json::to_value(&entry.counts).unwrap(),
                serde_json::to_value(levels(&row.counts).unwrap()).unwrap(),
                "RFC {}",
                row.rfc
            );
        }
    }

    #[test]
    fn a_sidecar_of_another_index_or_other_rfcs_is_refused() {
        let (summary, mut sidecar) = fixture();
        sidecar.header.index_sha256 = "2".repeat(64);
        let error = render(&summary, &sidecar).unwrap_err().to_string();
        assert!(error.contains("bulk-regenerate"), "{error}");

        let (summary, mut sidecar) = fixture();
        sidecar.rows.remove(2);
        let error = render(&summary, &sidecar).unwrap_err().to_string();
        assert!(error.contains("in its order"), "{error}");
    }

    /// The committed summary's shards, parsed back through the generated type, by first number.
    fn committed() -> (Summary, BTreeMap<String, String>) {
        let root = repository();
        let summary = crate::bulk::parse(&fs::read_to_string(root.join(SUMMARY)).unwrap()).unwrap();
        (summary, outputs(&root).unwrap())
    }

    fn find(shards: &BTreeMap<String, String>, rfc: u32) -> ExplorerRfc {
        shard(&shards[&path(range(rfc))])
            .rfcs
            .into_iter()
            .find(|r| number(&r.rfc.0) == rfc)
            .unwrap()
    }

    #[test]
    fn the_repository_has_a_shard_per_hundred_with_rows_holding_every_row_once_under_the_caps() {
        let (summary, mut shards) = committed();
        assert!(
            shards.remove(INDEX).is_some(),
            "the index beside the shards"
        );
        let hundreds: BTreeSet<u32> = summary.rows.iter().map(|r| range(r.rfc)).collect();
        let expected_paths: Vec<String> = hundreds.iter().map(|start| path(*start)).collect();
        let paths: Vec<&String> = shards.keys().collect();
        assert_eq!(paths, expected_paths.iter().collect::<Vec<_>>());
        assert_eq!(paths[0], "website/static/data/rfcs/00000.json");
        assert!(paths.len() > 90, "{}", paths.len());
        let mut seen = Vec::new();
        let mut total = 0;
        for (path, text) in &shards {
            assert!(
                text.len() < 2 * 1024 * 1024,
                "{path} is {} bytes",
                text.len()
            );
            total += text.len();
            let shard = shard(text);
            assert_eq!(shard.format, FORMAT);
            assert_eq!(shard.index_sha256.0, summary.header.index_sha256);
            let start = number(&shard.start);
            assert_eq!(super::path(start), *path);
            let numbers: Vec<u32> = shard.rfcs.iter().map(|r| number(&r.rfc.0)).collect();
            assert!(numbers.iter().all(|n| range(*n) == start), "{path}");
            seen.extend(numbers);
        }
        assert!(total < 16 * 1024 * 1024, "{total} bytes");
        let expected: Vec<u32> = summary.rows.iter().map(|r| r.rfc).collect();
        assert_eq!(seen, expected, "every summary row once, in number order");
    }

    #[test]
    fn rfc_6455_is_updated_by_7936_8307_and_8441_each_in_the_index() {
        let (_, shards) = committed();
        let websocket = find(&shards, 6455);
        assert_eq!(
            websocket.updated_by.iter().map(number).collect::<Vec<_>>(),
            [7936, 8307, 8441]
        );
        let absent = shard(&shards[&path(range(6455))]).not_in_index;
        for n in [7936u32, 8307, 8441] {
            assert!(!absent.contains(&Number::from(n)), "{n}");
            find(&shards, n);
        }
    }

    #[test]
    fn rfc_9220_holds_the_references_rfcspec_read_gives_among_them_8441_normative() {
        let root = repository();
        let bytes = fs::read(root.join("crates/rfcspec-doc/tests/fixtures/rfc9220.xml")).unwrap();
        let sha256 = rfcspec_model::sha256_hex(&bytes);
        let document = rfcspec_doc::read_format(
            9220,
            Format::Xml,
            &sha256,
            &String::from_utf8(bytes).unwrap(),
        )
        .unwrap();
        let (_, shards) = committed();
        let h3 = find(&shards, 9220);
        assert_eq!(
            h3.sha256.0, sha256,
            "the committed row was read from the fixture"
        );
        let read: Vec<Reference> = h3
            .references
            .iter()
            .map(|r| serde_json::from_value(serde_json::to_value(r).unwrap()).unwrap())
            .collect();
        assert!(!read.is_empty());
        assert_eq!(read, document.references);
        assert!(
            read.iter()
                .any(|r| r.target_rfc == 8441 && r.kind == Kind::Normative)
        );
        assert!(!h3.sections.is_empty());
        let absent = shard(&shards[&path(range(9220))]).not_in_index;
        assert!(!absent.contains(&Number::from(8441u32)));
    }
}

#[cfg(test)]
#[path = "explorer_profile_tests.rs"]
mod profile_tests;
