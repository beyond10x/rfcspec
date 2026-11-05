//! The protocol catalogue under `website/docs/catalog/`: an index with one row per profile
//! directory under `profiles/`, one page per profile, and each profile's `generated/` files copied
//! to `website/static/downloads/<name>/` as downloads. Everything here is a projection of the
//! profile directories; nothing on the pages is written by hand.

use std::{
    collections::BTreeMap,
    fmt, fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};
use rfcspec_model::{
    Bcp14Requirement, Catalogue, Decision, DecisionKind, LOCK_FILE, Lock, OVERLAY_FILE, Overlay,
    PROFILE_FILE, Profile, RUNS_DIR, RunReport, Status, newest, read_runs, read_yaml, sha256_hex,
    wire,
};

use crate::page::{Meta, SOURCE, cell, front_matter};

/// The profile directories, relative to the repository root.
pub const PROFILES: &str = "profiles";
/// Where the catalogue pages land, relative to the repository root.
pub const PAGES: &str = "website/docs/catalog";
/// The catalogue index.
pub const INDEX: &str = "website/docs/catalog/index.md";
/// Where the downloads land, relative to the repository root.
pub const DOWNLOADS: &str = "website/static/downloads";
/// The route of the catalogue, under the site's base URL.
pub const ROUTE: &str = "/docs/catalog";
/// The gaps table of the design page.
pub const GAPS: &str = "/docs/design#7-lowering-to-ess-and-the-gaps";

/// The highest pipeline stage (design section 4) whose output is checked into a profile. S1 to S3
/// store nothing in a profile and are never reported.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Stage {
    /// `rfcspec.lock`, with every locked source present at its SHA-256.
    S0,
    /// `overlay.yaml`.
    S4,
    /// `generated/*.protospec.yaml`.
    S5,
    /// `generated/rust/Cargo.toml`.
    S6,
    /// A recorded run under `runs/`.
    S7,
}

impl fmt::Display for Stage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Whether every source of the profile's lock is present at its SHA-256.
fn locked(dir: &Path) -> bool {
    let Ok(lock) = read_yaml::<Lock>(&dir.join(LOCK_FILE)) else {
        return false;
    };
    !lock.sources.is_empty()
        && lock.sources.iter().all(|entry| {
            fs::read(dir.join(&entry.file)).is_ok_and(|bytes| sha256_hex(&bytes) == entry.sha256)
        })
}

fn has_file(dir: &Path, matches: impl Fn(&str) -> bool) -> bool {
    fs::read_dir(dir).is_ok_and(|entries| {
        entries
            .filter_map(std::result::Result::ok)
            .any(|entry| entry.path().is_file() && entry.file_name().to_str().is_some_and(&matches))
    })
}

/// The stage a profile directory has reached, or `None` when it holds no stage's output.
pub fn stage(dir: &Path) -> Option<Stage> {
    [
        (
            Stage::S7,
            has_file(&dir.join(RUNS_DIR), |n| n.ends_with(".json")),
        ),
        (Stage::S6, dir.join("generated/rust/Cargo.toml").is_file()),
        (
            Stage::S5,
            has_file(&dir.join("generated"), |n| n.ends_with(".protospec.yaml")),
        ),
        (Stage::S4, dir.join(OVERLAY_FILE).is_file()),
        (Stage::S0, locked(dir)),
    ]
    .into_iter()
    .find_map(|(stage, reached)| reached.then_some(stage))
}

/// One file under a profile's `generated/`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Artifact {
    /// Relative to `generated/`, `/`-separated.
    pub path: String,
    pub text: String,
}

/// One profile directory, read.
#[derive(Clone, Debug)]
pub struct Entry {
    /// The directory's name.
    pub name: String,
    /// The directory, relative to the repository root, `/`-separated.
    pub path: String,
    pub profile: Profile,
    pub overlay: Option<Overlay>,
    pub stage: Option<Stage>,
    pub generated: Vec<Artifact>,
    /// The recorded runs, `(file name, report)`.
    pub runs: Vec<(String, RunReport)>,
    /// The mechanical catalogue of the overlay's RFC, read offline from its locked source; none
    /// without an overlay or a locked source at its SHA-256.
    pub catalogue: Option<Catalogue>,
}

impl Entry {
    /// The catalogue requirements without a status in the overlay; none when the catalogue
    /// cannot be read.
    pub fn unreviewed(&self) -> Option<Vec<&Bcp14Requirement>> {
        let catalogue = self.catalogue.as_ref()?;
        let reviewed: Vec<&str> = self
            .overlay
            .iter()
            .flat_map(|o| &o.requirements)
            .map(|r| r.id.as_str())
            .collect();
        Some(
            catalogue
                .requirements
                .iter()
                .filter(|r| !reviewed.contains(&r.id.as_str()))
                .collect(),
        )
    }
}

/// The catalogue of the overlay's RFC from the profile's locked source, when it is locked and
/// present at its SHA-256.
fn read_catalogue(dir: &Path, overlay: Option<&Overlay>) -> Option<Catalogue> {
    let rfc = overlay?.rfc;
    let locked = rfcspec_source::read_locked(dir, rfc).ok()?;
    Some(rfcspec_extract::extract(&rfcspec_doc::read(
        rfc,
        &locked.entry.sha256,
        &locked.text,
    )))
}

fn walk(base: &Path, dir: &Path, out: &mut Vec<PathBuf>) -> Result<()> {
    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .map(|entry| entry.map(|e| e.path()))
        .collect::<std::io::Result<_>>()?;
    entries.sort();
    for path in entries {
        if path.is_dir() {
            walk(base, &path, out)?;
        } else {
            out.push(path.strip_prefix(base).unwrap_or(&path).to_path_buf());
        }
    }
    Ok(())
}

/// Every file under `dir`, relative to it, `/`-separated and sorted; none when `dir` is absent.
pub fn files(dir: &Path) -> Result<Vec<String>> {
    let mut out = Vec::new();
    if dir.is_dir() {
        walk(dir, dir, &mut out)?;
    }
    Ok(out
        .iter()
        .map(|path| {
            path.components()
                .map(|c| c.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/")
        })
        .collect())
}

/// Read the profile directory `root/relative`.
pub fn load(root: &Path, relative: &str) -> Result<Entry> {
    let dir = root.join(relative);
    let name = dir
        .file_name()
        .and_then(|n| n.to_str())
        .with_context(|| format!("{relative} has no directory name"))?
        .to_owned();
    let profile: Profile = read_yaml(&dir.join(PROFILE_FILE))?;
    let overlay = if dir.join(OVERLAY_FILE).is_file() {
        Some(read_yaml(&dir.join(OVERLAY_FILE))?)
    } else {
        None
    };
    let generated = files(&dir.join("generated"))?
        .into_iter()
        .map(|path| {
            let file = dir.join("generated").join(&path);
            let bytes = fs::read(&file).with_context(|| format!("reading {}", file.display()))?;
            let text = String::from_utf8(bytes).with_context(|| {
                format!("{relative}/generated/{path} is not UTF-8 and is not published")
            })?;
            Ok(Artifact { path, text })
        })
        .collect::<Result<_>>()?;
    Ok(Entry {
        stage: stage(&dir),
        runs: read_runs(&dir)?,
        catalogue: read_catalogue(&dir, overlay.as_ref()),
        path: relative.trim_end_matches('/').to_owned(),
        name,
        profile,
        overlay,
        generated,
    })
}

/// Every profile directory under `root/profiles/`, by name.
pub fn entries(root: &Path) -> Result<Vec<Entry>> {
    let dir = root.join(PROFILES);
    let mut names: Vec<String> = fs::read_dir(&dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .collect();
    names.sort();
    names
        .iter()
        .map(|name| {
            if name == "index" {
                bail!("{PROFILES}/index would overwrite the catalogue index; rename it");
            }
            load(root, &format!("{PROFILES}/{name}"))
        })
        .collect()
}

/// Every catalogue file, by path relative to the repository root.
pub fn outputs(root: &Path) -> Result<BTreeMap<String, String>> {
    let entries = entries(root)?;
    let mut out = BTreeMap::from([(INDEX.to_owned(), index(&entries))]);
    for entry in &entries {
        out.insert(format!("{PAGES}/{}.md", entry.name), page(entry));
        for artifact in &entry.generated {
            out.insert(
                format!("{DOWNLOADS}/{}/{}", entry.name, artifact.path),
                artifact.text.clone(),
            );
        }
    }
    Ok(out)
}

/// The rank of a BCP 14 level, so counts read MUST before SHOULD before MAY.
fn rank(level: &str) -> usize {
    [
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
    ]
    .iter()
    .position(|known| *known == level)
    .unwrap_or(usize::MAX)
}

/// Requirement counts by level, `4 MUST, 2 SHOULD`; `none` without requirements.
pub fn level_counts(overlay: Option<&Overlay>) -> String {
    by_level(
        overlay
            .map_or(&[][..], |o| &o.requirements)
            .iter()
            .map(|r| r.level.as_str()),
    )
}

/// The unreviewed count of a profile: a number, or `no locked source` when its catalogue
/// cannot be read.
pub fn unreviewed_cell(entry: &Entry) -> String {
    entry.unreviewed().map_or_else(
        || "no locked source".into(),
        |found| found.len().to_string(),
    )
}

/// Counts by level, `4 MUST, 2 SHOULD`; `none` when there is nothing to count.
fn by_level<'a>(levels: impl Iterator<Item = &'a str>) -> String {
    let mut counts: BTreeMap<(usize, &str), usize> = BTreeMap::new();
    for level in levels {
        *counts.entry((rank(level), level)).or_default() += 1;
    }
    if counts.is_empty() {
        return "none".into();
    }
    counts
        .iter()
        .map(|((_, level), n)| format!("{n} {level}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Counts by level from a count per level, `4 MUST, 2 SHOULD` in the order of [`by_level`];
/// `none` when every count is zero.
pub fn counted(counts: &BTreeMap<String, u64>) -> String {
    let mut ranked: Vec<(usize, &str, u64)> = counts
        .iter()
        .filter(|(_, n)| **n > 0)
        .map(|(level, n)| (rank(level), level.as_str(), *n))
        .collect();
    if ranked.is_empty() {
        return "none".into();
    }
    ranked.sort_unstable();
    ranked
        .iter()
        .map(|(_, level, n)| format!("{n} {level}"))
        .collect::<Vec<_>>()
        .join(", ")
}

/// Requirement counts by status: satisfied, unsupported, not applicable.
pub fn status_counts(overlay: Option<&Overlay>) -> [usize; 3] {
    let mut counts = [0; 3];
    for requirement in overlay.map_or(&[][..], |o| &o.requirements) {
        counts[match requirement.status {
            Status::Satisfied(_) => 0,
            Status::Unsupported(_) => 1,
            Status::NotApplicable(_) => 2,
        }] += 1;
    }
    counts
}

fn kind(entry: &Entry) -> String {
    entry
        .overlay
        .as_ref()
        .and_then(|o| o.kind)
        .map_or_else(|| "unclassified".into(), |k| k.to_string())
}

fn stage_cell(entry: &Entry) -> String {
    entry.stage.map_or_else(|| "none".into(), |s| s.to_string())
}

/// The newest recorded run as `verdict` linked to its file, target and date; else `no recorded run`.
fn conformance_cell(entry: &Entry) -> String {
    match newest(&entry.runs) {
        None => "no recorded run".into(),
        Some((file, report)) => format!(
            "[{}]({SOURCE}/{}/{RUNS_DIR}/{file}), {}, {}",
            report.verdict,
            entry.path,
            cell(&report.target),
            report.date()
        ),
    }
}

/// The catalogue index: one row per profile, in the order given.
pub fn index(entries: &[Entry]) -> String {
    let mut page = front_matter(&Meta {
        title: "Protocol catalogue",
        label: "All profiles",
        description: "Every protocol RFCspec has turned into a specification, one row per profile: its RFCs, kind, stage reached, requirement counts and conformance verdict.",
        position: 1,
    });
    page.push_str(&format!(
        ":::note[Generated]\nThis page is generated by `rfcspec-docs` from the profile directories under [`{PROFILES}/`]({SOURCE}/{PROFILES}) on `main`.\n:::\n\n"
    ));
    page.push_str(
        "One row per profile. Kind is the RFC's primary kind ([RFC kinds](/docs/concepts/rfc-kinds)). Stage is the highest pipeline stage whose output is checked into the profile: S0 locked sources, S4 the reviewed overlay, S5 the lowered ESS models, S6 generated code, S7 a recorded conformance run. Requirements counts the reviewed requirements by level, and Satisfied, Unsupported and Not applicable count them by status. Unreviewed counts the requirements the mechanical catalogue of the locked source holds and the overlay gives no status. Conformance is the overall verdict of the newest recorded run.\n\n",
    );
    page.push_str(&format!(
        "The [RFC index]({}) lists every RFC published as plain text with its requirement counts by level, extracted mechanically and unreviewed, and marks the RFCs these profiles hold.\n\n",
        crate::bulk::ROUTE
    ));
    if entries.is_empty() {
        page.push_str("No profile yet.\n");
        return page;
    }
    page.push_str("| Profile | RFCs | Kind | Stage | Requirements | Satisfied | Unsupported | Not applicable | Unreviewed | Conformance |\n|---|---|---|---|---|---|---|---|---|---|\n");
    for entry in entries {
        let rfcs = entry
            .profile
            .rfcs
            .iter()
            .map(|rfc| format!("RFC {} {}", rfc.number, cell(&rfc.title)))
            .collect::<Vec<_>>()
            .join("; ");
        let [satisfied, unsupported, not_applicable] = status_counts(entry.overlay.as_ref());
        page.push_str(&format!(
            "| [{}]({ROUTE}/{}) | {rfcs} | {} | {} | {} | {satisfied} | {unsupported} | {not_applicable} | {} | {} |\n",
            cell(&entry.name),
            entry.name,
            kind(entry),
            stage_cell(entry),
            level_counts(entry.overlay.as_ref()),
            unreviewed_cell(entry),
            conformance_cell(entry),
        ));
    }
    page
}

/// The gap a reason names, `ess-gap G<n>`, by its number.
pub fn gap(reason: &str) -> Option<u32> {
    let rest = reason.split_once("ess-gap")?.1.trim_start();
    let digits: String = rest
        .strip_prefix('G')?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    digits.parse().ok()
}

/// What a generated file is, for the downloads table.
fn artifact_kind(entry: &Entry, path: &str) -> String {
    if path.starts_with("rust/") {
        return "generated Rust crate".into();
    }
    match path.strip_suffix(".protospec.yaml") {
        Some(machine)
            if !machine.contains('/')
                && entry
                    .overlay
                    .as_ref()
                    .is_some_and(|o| o.machines.iter().any(|m| m.name == machine)) =>
        {
            format!("`ess-protospec/1` model of machine `{machine}`")
        }
        Some(_) => "`ess-protospec/1` model".into(),
        None => "generated file".into(),
    }
}

fn status_cells(status: &Status) -> (&'static str, &str) {
    match status {
        Status::Satisfied(element) => ("satisfied", element),
        Status::Unsupported(reason) => ("unsupported", reason),
        Status::NotApplicable(reason) => ("not applicable", reason),
    }
}

/// The page of one profile.
pub fn page(entry: &Entry) -> String {
    let titles = entry
        .profile
        .rfcs
        .iter()
        .map(|rfc| format!("RFC {} {}", rfc.number, rfc.title))
        .collect::<Vec<_>>()
        .join("; ");
    let mut page = front_matter(&Meta {
        title: &entry.name,
        label: &entry.name,
        description: &format!(
            "The {} profile of the RFCspec protocol catalogue ({titles}): its RFCs, communication semantics, requirement matrix, ESS gaps, conformance and downloads.",
            entry.name
        ),
        position: 10,
    });
    page.push_str(&format!(
        ":::note[Generated]\nThis page is generated by `rfcspec-docs` from [`{path}/`]({SOURCE}/{path}) on `main`.\n:::\n\n",
        path = entry.path
    ));
    page.push_str(&format!(
        "Part of the [protocol catalogue]({ROUTE}). Kind {}, stage reached {}, conformance {}.\n\n",
        kind(entry),
        stage_cell(entry),
        conformance_cell(entry)
    ));
    rfcs(&mut page, entry);
    semantics(&mut page, entry);
    matrix(&mut page, entry);
    gaps(&mut page, entry);
    rejected(&mut page, entry);
    conformance(&mut page, entry);
    downloads(&mut page, entry);
    page
}

fn rfcs(page: &mut String, entry: &Entry) {
    page.push_str("## RFCs\n\n| RFC | Title | Sections in scope | Kind |\n|---|---|---|---|\n");
    for rfc in &entry.profile.rfcs {
        let sections = if rfc.sections.is_empty() {
            "the whole RFC".to_owned()
        } else {
            cell(&rfc.sections.join("; "))
        };
        let kind = entry
            .overlay
            .as_ref()
            .filter(|o| o.rfc == rfc.number)
            .and_then(|o| o.kind)
            .map_or_else(
                || "unclassified".to_owned(),
                |k| format!("{k} {}", k.label()),
            );
        page.push_str(&format!(
            "| [RFC {n}](https://www.rfc-editor.org/rfc/rfc{n}) | {} | {sections} | {} |\n",
            cell(&rfc.title),
            cell(&kind),
            n = rfc.number,
        ));
    }
    page.push('\n');
}

fn semantics(page: &mut String, entry: &Entry) {
    page.push_str("## Communication semantics\n\n");
    if entry.profile.semantics.is_empty() {
        page.push_str("No communication semantics declared.\n\n");
        return;
    }
    page.push_str("One row per layer, as `profile.yaml` declares them ([communication semantics](/docs/concepts/communication-semantics)).\n\n| Layer | Unit | Delivery | Direction | Multiplexing | Exchange | Correlation | Lifetime | Over | Roles |\n|---|---|---|---|---|---|---|---|---|---|\n");
    for layer in &entry.profile.semantics {
        page.push_str(&format!(
            "| {} | {} | {} | {} | {} | {} | {} | {} | {} | {} |\n",
            match layer.port {
                Some(port) => cell(&format!("{} (port {port})", layer.layer)),
                None => cell(&layer.layer),
            },
            wire(&layer.unit),
            layer.delivery.describe(),
            wire(&layer.direction),
            wire(&layer.multiplexing),
            wire(&layer.exchange),
            wire(&layer.correlation),
            wire(&layer.lifetime),
            layer
                .over
                .as_ref()
                .map_or_else(|| "not declared".to_owned(), |o| cell(&o.describe())),
            cell(&layer.roles.join(", ")),
        ));
    }
    page.push('\n');
}

fn matrix(page: &mut String, entry: &Entry) {
    page.push_str("## Requirement matrix\n\n");
    let Some(overlay) = &entry.overlay else {
        page.push_str("No reviewed overlay yet.\n\n");
        return;
    };
    let [satisfied, unsupported, not_applicable] = status_counts(Some(overlay));
    page.push_str(&format!(
        "Reviewed: {} requirements, by level {}; by status {satisfied} satisfied, {unsupported} unsupported, {not_applicable} not applicable.\n\n",
        overlay.requirements.len(),
        level_counts(Some(overlay)),
    ));
    match entry.unreviewed() {
        None => page.push_str(&format!(
            "Unreviewed: not counted, because the profile holds no locked source of RFC {}.\n\n",
            overlay.rfc
        )),
        Some(found) if found.is_empty() => page.push_str(&format!(
            "Unreviewed: none. Every requirement the mechanical catalogue of RFC {} holds has a status.\n\n",
            overlay.rfc
        )),
        Some(found) => page.push_str(&format!(
            "Unreviewed: {} requirements of the mechanical catalogue of RFC {} have no status, by level {}. `rfcspec extract` lists them; the matrix below holds the reviewed ones.\n\n",
            found.len(),
            overlay.rfc,
            by_level(found.iter().map(|r| r.keyword.as_str())),
        )),
    }
    page.push_str(&format!(
        "One row per requirement of [`overlay.yaml`]({SOURCE}/{}/{OVERLAY_FILE}), in its order. The source is the line span in RFC {}.\n\n| Id | Level | Transports | Quote | Source | Status | Element or reason |\n|---|---|---|---|---|---|---|\n",
        entry.path, overlay.rfc
    ));
    for requirement in &overlay.requirements {
        let transports = if requirement.transports.is_empty() {
            "all".to_owned()
        } else {
            requirement.transports.join(", ")
        };
        let (status, detail) = status_cells(&requirement.status);
        page.push_str(&format!(
            "| {} | {} | {} | {} | {} | {status} | {} |\n",
            cell(&requirement.id),
            cell(&requirement.level),
            cell(&transports),
            cell(&requirement.quote),
            requirement.span,
            cell(detail),
        ));
    }
    page.push('\n');
}

fn gaps(page: &mut String, entry: &Entry) {
    page.push_str("## ESS gaps\n\n");
    let mut by_gap: BTreeMap<u32, Vec<&str>> = BTreeMap::new();
    let mut other: Vec<(&str, &str)> = Vec::new();
    for requirement in entry.overlay.iter().flat_map(|o| &o.requirements) {
        if let Status::Unsupported(reason) = &requirement.status {
            match gap(reason) {
                Some(n) => by_gap.entry(n).or_default().push(&requirement.id),
                None => other.push((&requirement.id, reason)),
            }
        }
    }
    if by_gap.is_empty() && other.is_empty() {
        page.push_str("No ESS gap: every requirement in scope lowers.\n\n");
        return;
    }
    if by_gap.is_empty() {
        page.push_str("No ESS gap.\n\n");
    } else {
        page.push_str(&format!(
            "Requirements marked unsupported because ESS cannot carry their rule yet, by gap of the [gaps table]({GAPS}).\n\n| Gap | Requirements |\n|---|---|\n"
        ));
        for (n, ids) in &by_gap {
            page.push_str(&format!(
                "| [G{n}]({GAPS}) | {} |\n",
                ids.iter().map(|id| cell(id)).collect::<Vec<_>>().join(", ")
            ));
        }
        page.push('\n');
    }
    if !other.is_empty() {
        page.push_str("### Unsupported, no ESS gap\n\n| Requirement | Reason |\n|---|---|\n");
        for (id, reason) in other {
            page.push_str(&format!("| {} | {} |\n", cell(id), cell(reason)));
        }
        page.push('\n');
    }
}

/// The overlay's reject decisions with their reasons, in the order they were recorded; no section
/// when there is none.
fn rejected(page: &mut String, entry: &Entry) {
    let rejects: Vec<&Decision> = entry
        .overlay
        .iter()
        .flat_map(|o| &o.decisions)
        .filter(|d| d.kind == DecisionKind::Reject)
        .collect();
    if rejects.is_empty() {
        return;
    }
    page.push_str(&format!(
        "## Rejected proposals\n\nProposals a reviewer rejected, from the decisions in [`{path}/{OVERLAY_FILE}`]({SOURCE}/{path}/{OVERLAY_FILE}); each item stays unreviewed and is not proposed again.\n\n| Item | Proposal | Reason |\n|---|---|---|\n",
        path = entry.path
    ));
    for decision in rejects {
        page.push_str(&format!(
            "| {} | {} | {} |\n",
            cell(&decision.item),
            decision
                .proposal_id
                .as_deref()
                .map_or_else(|| "none".to_owned(), |id| format!("`{id}`")),
            cell(decision.reason.as_deref().unwrap_or_default()),
        ));
    }
    page.push('\n');
}

fn conformance(page: &mut String, entry: &Entry) {
    page.push_str("## Conformance\n\n");
    match newest(&entry.runs) {
        None => page.push_str(&format!(
            "There is no recorded run: no report under `{}/{RUNS_DIR}/`.\n\n",
            entry.path
        )),
        Some((file, report)) => page.push_str(&format!(
            "The newest recorded run of `rfcspec verify`, of {} under `{}/{RUNS_DIR}/`.\n\n| Verdict | Target | Date | rfcspec | Report |\n|---|---|---|---|---|\n| {} | {} | {} | {} | [{}]({SOURCE}/{}/{RUNS_DIR}/{file}) |\n\n",
            entry.runs.len(),
            entry.path,
            report.verdict,
            cell(&report.target),
            report.date(),
            cell(&report.rfcspec),
            cell(file),
            entry.path,
        )),
    }
}

fn downloads(page: &mut String, entry: &Entry) {
    page.push_str("## Downloads\n\n");
    if entry.generated.is_empty() {
        page.push_str("No generated artifacts yet.\n");
        return;
    }
    page.push_str(&format!(
        "The files under [`{path}/generated/`]({SOURCE}/{path}/generated), copied unchanged.\n\n| File | What | Bytes | SHA-256 |\n|---|---|---|---|\n",
        path = entry.path
    ));
    for artifact in &entry.generated {
        page.push_str(&format!(
            "| [`{path}`](pathname://{base}downloads/{name}/{path}) | {} | {} | `{}` |\n",
            artifact_kind(entry, &artifact.path),
            artifact.text.len(),
            sha256_hex(artifact.text.as_bytes()),
            path = artifact.path,
            base = crate::page::BASE_URL,
            name = entry.name,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn repository() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    const FIXTURES: &str = "crates/rfcspec-docs/tests/fixtures/profiles";

    fn time() -> Entry {
        load(&repository(), "profiles/time").unwrap()
    }

    fn fixture() -> Entry {
        load(&repository(), &format!("{FIXTURES}/fixture")).unwrap()
    }

    /// The table rows of the section headed `heading`, header and separator dropped.
    fn rows<'a>(page: &'a str, heading: &str) -> Vec<&'a str> {
        let section = page.split(heading).nth(1).unwrap_or_default();
        let section = section.split("\n## ").next().unwrap_or_default();
        section
            .lines()
            .filter(|l| l.starts_with('|'))
            .skip(2)
            .collect()
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = repository()
            .join("target/rfcspec-docs")
            .join(format!("catalog-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn the_stage_is_the_highest_output_checked_in() {
        let dir = scratch("stage");
        assert_eq!(stage(&dir), None);
        let time = repository().join("profiles/time");
        let copy = |file: &str| {
            fs::create_dir_all(dir.join(file).parent().unwrap()).unwrap();
            fs::copy(time.join(file), dir.join(file)).unwrap();
        };
        copy("profile.yaml");
        copy(LOCK_FILE);
        assert_eq!(stage(&dir), None, "a locked source must be present");
        fs::create_dir_all(dir.join("sources")).unwrap();
        fs::write(dir.join("sources/rfc868.txt"), "not the RFC\n").unwrap();
        assert_eq!(stage(&dir), None, "a locked source must match its SHA-256");
        copy("sources/rfc868.txt");
        assert_eq!(stage(&dir), Some(Stage::S0));
        copy(OVERLAY_FILE);
        assert_eq!(stage(&dir), Some(Stage::S4));
        copy("generated/tcp.protospec.yaml");
        assert_eq!(stage(&dir), Some(Stage::S5));
        copy("generated/rust/Cargo.toml");
        assert_eq!(stage(&dir), Some(Stage::S6));
        fs::create_dir_all(dir.join(RUNS_DIR)).unwrap();
        fs::write(dir.join("runs/notes.txt"), "").unwrap();
        assert_eq!(stage(&dir), Some(Stage::S6), "only a .json report is a run");
        fs::write(dir.join("runs/tcp-127.0.0.1-37.json"), "{}").unwrap();
        assert_eq!(stage(&dir), Some(Stage::S7));
        fs::remove_dir_all(dir).unwrap();
        assert_eq!(time_stage(), Some(Stage::S6));
    }

    fn time_stage() -> Option<Stage> {
        stage(&repository().join("profiles/time"))
    }

    #[test]
    fn the_index_lists_each_profile_in_name_order_with_counts_and_stage() {
        let mut entries = vec![time(), fixture()];
        entries.sort_by(|a, b| a.name.cmp(&b.name));
        let page = index(&entries);
        let rows = rows(&page, "newest recorded run.");
        assert_eq!(rows.len(), 2, "{page}");
        assert_eq!(
            rows[0],
            "| [fixture](/docs/catalog/fixture) | RFC 9999 A \\{braced\\} &lt;Fixture> \\| Protocol | unclassified | S4 | 1 MUST, 1 SHOULD, 1 MAY | 1 | 1 | 1 | 0 | no recorded run |"
        );
        assert_eq!(
            rows[1],
            "| [time](/docs/catalog/time) | RFC 868 Time Protocol | K1 | S6 | 4 MUST, 2 SHOULD | 6 | 0 | 0 | 0 | no recorded run |"
        );
        assert!(crate::page::passive_problems(&page).is_empty(), "{page}");
        assert_eq!(
            page.lines()
                .filter(|l| l.contains("](/docs/catalog/rfcs)"))
                .count(),
            1,
            "the index links the RFC index once: {page}"
        );
    }

    #[test]
    fn the_matrix_has_one_row_per_overlay_requirement() {
        for entry in [time(), fixture()] {
            let page = page(&entry);
            assert_eq!(
                rows(&page, "## Requirement matrix").len(),
                entry.overlay.as_ref().unwrap().requirements.len(),
                "{page}"
            );
        }
        let page = page(&time());
        assert!(page.contains("| R-TCP-1 | MUST | tcp | When the connection is established, the server returns a 32-bit time value | lines 44-45 | satisfied | tcp.server.D1-3-R-TCP-1-send-time |"), "{page}");
        assert!(page.contains("| R-TIME-1 | MUST | tcp, udp |"), "{page}");
    }

    #[test]
    fn the_rfcs_link_to_the_rfc_editor() {
        let page = page(&time());
        assert_eq!(
            rows(&page, "## RFCs"),
            [
                "| [RFC 868](https://www.rfc-editor.org/rfc/rfc868) | Time Protocol | Time Protocol; The Time | K1 wire protocol |"
            ]
        );
        assert!(rows(&self::page(&fixture()), "## RFCs")[0].ends_with("| unclassified |"));
    }

    #[test]
    fn gaps_are_grouped_and_an_unsupported_reason_without_a_gap_is_kept() {
        let time = page(&time());
        assert!(time.contains("## ESS gaps\n\nNo ESS gap: every requirement in scope lowers.\n"));
        let fixture = page(&fixture());
        assert_eq!(
            rows(&fixture, "## ESS gaps")[0],
            format!("| [G1]({GAPS}) | R-FIX-1 |")
        );
        let mut entry = self::fixture();
        let overlay = entry.overlay.as_mut().unwrap();
        overlay.requirements[1].status = Status::Unsupported("no model of the clock".into());
        overlay.requirements.push(overlay.requirements[0].clone());
        overlay.requirements[3].id = "R-FIX-4".into();
        overlay.requirements[3].status = Status::Unsupported("needs ess-gap G12 and more".into());
        let page = page(&entry);
        assert!(
            page.contains(&format!(
                "| [G1]({GAPS}) | R-FIX-1 |\n| [G12]({GAPS}) | R-FIX-4 |\n"
            )),
            "{page}"
        );
        assert!(page.contains("### Unsupported, no ESS gap\n\n| Requirement | Reason |\n|---|---|\n| R-FIX-2 | no model of the clock |\n"), "{page}");
        assert_eq!(gap("ess-gap G1"), Some(1));
        assert_eq!(gap("ess-gap"), None);
        assert_eq!(gap("an ess-gap Gx"), None);
    }

    #[test]
    fn quotes_with_braces_tags_and_pipes_stay_passive() {
        let page = page(&fixture());
        assert!(
            page.contains(
                "| R-FIX-3 | MAY | all | A client MAY send \\{x\\}, &lt;Tag> or a \\| pipe. |"
            ),
            "{page}"
        );
        assert!(crate::page::passive_problems(&page).is_empty(), "{page}");
        assert!(crate::page::passive_problems(&self::page(&time())).is_empty());
    }

    #[test]
    fn a_profile_without_generated_files_has_no_downloads() {
        let page = page(&fixture());
        assert!(
            page.ends_with("## Downloads\n\nNo generated artifacts yet.\n"),
            "{page}"
        );
        assert!(fixture().generated.is_empty());
    }

    #[test]
    fn the_time_downloads_list_every_generated_file_with_size_and_digest() {
        let entry = time();
        let page = page(&entry);
        let rows = rows(&page, "## Downloads");
        let paths: Vec<&str> = entry.generated.iter().map(|a| a.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "rust/Cargo.toml",
                "rust/src/lib.rs",
                "tcp.protospec.yaml",
                "udp.protospec.yaml"
            ]
        );
        assert_eq!(rows.len(), 4, "{page}");
        let tcp =
            fs::read(repository().join("profiles/time/generated/tcp.protospec.yaml")).unwrap();
        assert!(rows.contains(&format!(
            "| [`tcp.protospec.yaml`](pathname:///rfcspec/downloads/time/tcp.protospec.yaml) | `ess-protospec/1` model of machine `tcp` | {} | `{}` |",
            tcp.len(),
            sha256_hex(&tcp)
        ).as_str()), "{page}");
        assert!(rows[0].contains("| generated Rust crate |"), "{page}");
    }

    #[test]
    fn a_non_utf8_generated_file_is_refused_by_name() {
        let root = scratch("binary");
        fs::create_dir_all(root.join("p/generated")).unwrap();
        fs::write(root.join("p/profile.yaml"), "name: p\nrfcs: []\n").unwrap();
        fs::write(root.join("p/generated/blob.bin"), [0xff, 0xfe]).unwrap();
        let error = format!("{:#}", load(&root, "p").unwrap_err());
        assert!(
            error.contains("p/generated/blob.bin is not UTF-8"),
            "{error}"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn semantics_show_one_row_per_layer_or_a_sentence() {
        let page = page(&time());
        assert_eq!(
            rows(&page, "## Communication semantics"),
            [
                "| tcp | bytes | ordered, reliable, no duplicates | simplex | single | one-shot | none | connection | not declared | client, server |",
                "| udp | datagrams | unordered, lossy, duplicates | half-duplex | single | request-response | none | connectionless | not declared | client, server |",
            ]
        );
        assert!(
            self::page(&fixture())
                .contains("## Communication semantics\n\nNo communication semantics declared.\n")
        );
    }

    #[test]
    fn a_declared_port_shows_in_its_layer_cell_and_an_absent_one_changes_nothing() {
        let mut entry = time();
        let before = page(&entry);
        entry.profile.semantics[1].port = std::num::NonZeroU16::new(37);
        assert_eq!(
            rows(&page(&entry), "## Communication semantics"),
            [
                "| tcp | bytes | ordered, reliable, no duplicates | simplex | single | one-shot | none | connection | not declared | client, server |",
                "| udp (port 37) | datagrams | unordered, lossy, duplicates | half-duplex | single | request-response | none | connectionless | not declared | client, server |",
            ]
        );
        entry.profile.semantics[1].port = None;
        assert_eq!(page(&entry), before);
    }

    #[test]
    fn the_newest_recorded_run_is_the_conformance_verdict() {
        let entry = load(&repository(), &format!("{FIXTURES}/recorded")).unwrap();
        assert_eq!(entry.stage, Some(Stage::S7));
        assert_eq!(entry.runs.len(), 2);
        let link = format!("{SOURCE}/{FIXTURES}/recorded/runs/tcp-127.0.0.1-37.json");
        let page = page(&entry);
        assert!(page.contains(&format!("| fail | tcp://127.0.0.1:37 | 2026-11-03 | 0.0.0 | [tcp-127.0.0.1-37.json]({link}) |")), "{page}");
        let index = index(&[entry]);
        assert!(
            index.contains(&format!(
                "| S7 | none | 0 | 0 | 0 | no locked source | [fail]({link}), tcp://127.0.0.1:37, 2026-11-03 |"
            )),
            "{index}"
        );
        assert!(self::page(&time()).contains("There is no recorded run"));
        assert!(index_of_time().contains("| no recorded run |"));
    }

    fn index_of_time() -> String {
        index(&[time()])
    }

    fn websocket() -> Entry {
        load(&repository(), "profiles/websocket").unwrap()
    }

    const WEBSOCKET_SCOPE: [&str; 5] = ["5.2", "5.4", "5.5", "5.6", "7"];

    fn in_websocket_scope(section: Option<&str>) -> bool {
        section.is_some_and(|section| {
            WEBSOCKET_SCOPE.iter().any(|listed| {
                section == *listed
                    || section
                        .strip_prefix(listed)
                        .is_some_and(|rest| rest.starts_with('.'))
            })
        })
    }

    /// The catalogue `rfcspec extract` lists for the committed RFC 6455 source, read here
    /// independently of `load`.
    fn extract_6455() -> Catalogue {
        let text = fs::read_to_string(repository().join("profiles/websocket/sources/rfc6455.txt"))
            .unwrap();
        let sha = sha256_hex(text.as_bytes());
        rfcspec_extract::extract(&rfcspec_doc::read(6455, &sha, &text))
    }

    #[test]
    fn the_index_counts_unreviewed_requirements_for_both_profiles() {
        let catalogue = extract_6455();
        let (outside, inside): (Vec<_>, Vec<_>) = catalogue
            .requirements
            .iter()
            .partition(|r| !in_websocket_scope(r.section.as_deref()));
        let keywords: usize = outside
            .iter()
            .map(|r| rfcspec_extract::keywords(&r.text).len())
            .sum();
        assert_eq!(keywords, 136, "keyword occurrences outside the scope");
        let entry = websocket();
        let reviewed: Vec<&str> = entry
            .overlay
            .as_ref()
            .unwrap()
            .requirements
            .iter()
            .map(|r| r.id.as_str())
            .collect();
        assert!(
            inside.iter().all(|r| reviewed.contains(&r.id.as_str())),
            "every in-scope requirement has a status"
        );
        let unreviewed = entry.unreviewed().unwrap();
        assert!(
            unreviewed
                .iter()
                .all(|r| !in_websocket_scope(r.section.as_deref())),
            "0 unreviewed requirements inside the scope"
        );
        let outside_with_status = outside
            .iter()
            .filter(|r| reviewed.contains(&r.id.as_str()))
            .count();
        assert_eq!(
            outside_with_status, 7,
            "the accepted and edited proposals of sections 5.1 and 5.3"
        );
        let expected = outside.len() - outside_with_status;
        assert_eq!(expected, 119);
        assert_eq!(unreviewed_cell(&entry), expected.to_string());
        let page = index(&[time(), entry]);
        let rows = rows(&page, "newest recorded run.");
        assert_eq!(
            rows,
            [
                "| [time](/docs/catalog/time) | RFC 868 Time Protocol | K1 | S6 | 4 MUST, 2 SHOULD | 6 | 0 | 0 | 0 | no recorded run |".to_string(),
                format!(
                    "| [websocket](/docs/catalog/websocket) | RFC 6455 The WebSocket Protocol | K1 | S6 | 31 MUST, 12 MUST NOT, 11 SHOULD, 3 SHOULD NOT, 20 MAY | 29 | 40 | 8 | {expected} | no recorded run |"
                ),
            ]
        );
        assert!(crate::page::passive_problems(&page).is_empty(), "{page}");
    }

    #[test]
    fn a_profile_without_a_locked_source_shows_no_unreviewed_count() {
        let recorded = load(&repository(), &format!("{FIXTURES}/recorded")).unwrap();
        assert!(recorded.unreviewed().is_none());
        assert_eq!(unreviewed_cell(&recorded), "no locked source");
        let mut fixture = fixture();
        assert_eq!(unreviewed_cell(&fixture), "0");
        fixture.catalogue = None;
        assert_eq!(unreviewed_cell(&fixture), "no locked source");
        assert!(
            page(&fixture).contains(
                "Unreviewed: not counted, because the profile holds no locked source of RFC 9999.\n"
            ),
            "{}",
            page(&fixture)
        );
        assert!(page(&time()).contains("Unreviewed: none. Every requirement the mechanical catalogue of RFC 868 holds has a status.\n"));
    }

    #[test]
    fn the_websocket_page_shows_rfc_semantics_counts_gaps_and_downloads() {
        let entry = websocket();
        let page = page(&entry);
        assert_eq!(
            rows(&page, "## RFCs"),
            [
                "| [RFC 6455](https://www.rfc-editor.org/rfc/rfc6455) | The WebSocket Protocol | 5.2; 5.4; 5.5; 5.6; 7 | K1 wire protocol |"
            ]
        );
        assert_eq!(
            rows(&page, "## Communication semantics"),
            [
                "| tcp | bytes | ordered, reliable, no duplicates | full-duplex | single | session | none | connection | not declared | client, server |"
            ]
        );
        assert!(page.contains("Reviewed: 77 requirements, by level 31 MUST, 12 MUST NOT, 11 SHOULD, 3 SHOULD NOT, 20 MAY; by status 29 satisfied, 40 unsupported, 8 not applicable.\n"), "{page}");
        let unreviewed = entry.unreviewed().unwrap();
        assert!(page.contains(&format!(
            "Unreviewed: {} requirements of the mechanical catalogue of RFC 6455 have no status, by level {}.",
            unreviewed.len(),
            by_level(unreviewed.iter().map(|r| r.keyword.as_str()))
        )), "{page}");
        assert!(page.contains("Unreviewed: 119 requirements of the mechanical catalogue of RFC 6455 have no status, by level 77 MUST, 11 MUST NOT, 7 SHOULD, 1 SHOULD NOT, 22 MAY, 1 OPTIONAL."), "{page}");
        let rejected: Vec<&str> = rows(&page, "## Rejected proposals")
            .iter()
            .map(|row| row.split(" | ").next().unwrap())
            .collect();
        assert_eq!(
            rejected,
            ["| rfc6455/R5.1-4", "| rfc6455/R5.1-3", "| rfc6455/R5.3-1"]
        );
        assert_eq!(
            rows(&page, "## Requirement matrix").len(),
            entry.overlay.as_ref().unwrap().requirements.len()
        );
        let gaps = rows(&page, "## ESS gaps");
        assert!(
            gaps.contains(
                &format!("| [G1]({GAPS}) | R5.5-1, R7.4.1-3, R7.4.1-4, R7.4.1-6 |").as_str()
            ),
            "{page}"
        );
        assert!(
            gaps.contains(&format!("| [G9]({GAPS}) | R7.1.1-4, R7.1.7-1 |").as_str()),
            "{page}"
        );
        assert!(
            gaps.contains(&format!("| [G10]({GAPS}) | R5.5.3-1 |").as_str()),
            "{page}"
        );
        assert!(
            gaps.contains(
                &format!("| [G3]({GAPS}) | R5.4-6, R5.5.1-1, R5.5.1-3, R5.6-1 |").as_str()
            ),
            "{page}"
        );
        for stale in [
            "Ping and Pong frames are not modelled",
            "data frames are not modelled",
            "carries no data frames",
            "fragmentation are not modelled",
            "fragmentation is not modelled",
        ] {
            assert!(!page.contains(stale), "{stale}: {page}");
        }
        let generated = repository().join("profiles/websocket/generated");
        let paths: Vec<&str> = entry.generated.iter().map(|a| a.path.as_str()).collect();
        assert_eq!(
            paths,
            [
                "close.protospec.yaml",
                "fragments.protospec.yaml",
                "ping.protospec.yaml",
                "rust/Cargo.toml",
                "rust/src/lib.rs"
            ]
        );
        let downloads = rows(&page, "## Downloads");
        assert_eq!(downloads.len(), 5, "{page}");
        for artifact in &entry.generated {
            let bytes = fs::read(generated.join(&artifact.path)).unwrap();
            assert_eq!(
                artifact.text.as_bytes(),
                bytes.as_slice(),
                "{}",
                artifact.path
            );
            assert!(
                downloads.iter().any(|row| row.contains(&format!(
                    "(pathname:///rfcspec/downloads/websocket/{}) | ",
                    artifact.path
                )) && row.ends_with(&format!(
                    "| {} | `{}` |",
                    bytes.len(),
                    sha256_hex(&bytes)
                ))),
                "{}",
                artifact.path
            );
        }
    }

    /// A copy of `profiles/websocket` under a scratch directory as it stood before its section
    /// 5.1 and 5.3 proposals were reviewed: without `proposals/`, the decisions, or the
    /// requirement entries of those two sections, so a test can record and decide them itself.
    fn websocket_copy(name: &str) -> PathBuf {
        let root = scratch(name);
        let from = repository().join("profiles/websocket");
        let dir = root.join("profiles/websocket");
        for file in files(&from).unwrap() {
            if file.starts_with("proposals/") {
                continue;
            }
            fs::create_dir_all(dir.join(&file).parent().unwrap()).unwrap();
            fs::copy(from.join(&file), dir.join(&file)).unwrap();
        }
        let path = dir.join(OVERLAY_FILE);
        let text = fs::read_to_string(&path).unwrap();
        fs::write(&path, before_the_slice(&text)).unwrap();
        root
    }

    #[test]
    fn accepting_two_proposals_and_rejecting_one_counts_two_fewer_unreviewed() {
        use rfcspec_propose::{
            Outcome, Task, Workspace,
            double::Double,
            record_blocking,
            review::{Verdict, decide, write},
        };
        use serde_json::json;

        let root = websocket_copy("reviewed");
        let dir = root.join("profiles/websocket");
        let status = |id: &str, quote: &str, start: usize, end: usize| {
            json!({
                "id": id,
                "source": "capitals-text",
                "level": "MUST",
                "quote": quote,
                "span": {"start": start, "end": end},
                "status": {"not-applicable": "section 5.1 is outside the profile's scope"}
            })
        };
        let answers = [
            (
                "R5.1-1",
                status(
                    "R5.1-1",
                    "a client MUST mask all frames that it sends to the server",
                    1490,
                    1493,
                ),
            ),
            (
                "R5.1-2",
                status(
                    "R5.1-2",
                    "The server MUST close the connection upon receiving a frame that is not masked.",
                    1493,
                    1496,
                ),
            ),
            (
                "R5.1-5",
                status(
                    "R5.1-5",
                    "A client MUST close a connection if it detects a masked frame.",
                    1499,
                    1500,
                ),
            ),
        ];
        let model = Double::new("double-model", answers.iter().map(|(_, a)| a.clone()));
        let workspace = Workspace::load(&dir).unwrap();
        let ids: Vec<String> = answers
            .iter()
            .map(|(item, _)| {
                match record_blocking(&workspace, &model, "double-model", Task::Status, item) {
                    Ok(Outcome::Recorded(path)) => {
                        path.file_stem().unwrap().to_str().unwrap().to_owned()
                    }
                    other => panic!("{item}: {other:?}"),
                }
            })
            .collect();

        let before = load(&root, "profiles/websocket").unwrap();
        let unreviewed_before: Vec<String> = before
            .unreviewed()
            .unwrap()
            .iter()
            .map(|r| r.id.clone())
            .collect();
        for (id, verdict) in ids.iter().zip([
            Verdict::Accept,
            Verdict::Accept,
            Verdict::Reject("the level | is wrong".into()),
        ]) {
            let reviewed = decide(&dir, id, verdict).unwrap_or_else(|error| panic!("{error}"));
            write(&dir, &reviewed).unwrap();
        }
        let after = load(&root, "profiles/websocket").unwrap();
        let unreviewed_after: Vec<&str> = after
            .unreviewed()
            .unwrap()
            .iter()
            .map(|r| r.id.as_str())
            .collect();
        assert_eq!(unreviewed_after.len(), unreviewed_before.len() - 2);
        assert!(!unreviewed_after.contains(&"R5.1-1"));
        assert!(!unreviewed_after.contains(&"R5.1-2"));
        assert!(
            unreviewed_after.contains(&"R5.1-5"),
            "a rejected requirement stays unreviewed"
        );
        assert_eq!(
            unreviewed_cell(&after),
            (unreviewed_before.len() - 2).to_string()
        );
        let page = page(&after);
        assert_eq!(
            rows(&page, "## Rejected proposals"),
            [format!(
                "| rfc6455/R5.1-5 | `{}` | the level \\| is wrong |",
                ids[2]
            )]
        );
        assert!(
            !self::page(&before).contains("## Rejected proposals"),
            "a profile without a reject decision has no such section"
        );
    }

    /// The text of the websocket overlay without its decisions and without the requirement entries
    /// of sections 5.1 and 5.3, every other line as written.
    fn before_the_slice(overlay: &str) -> String {
        let overlay = overlay.split("\ndecisions:\n").next().unwrap();
        let mut out = String::new();
        let mut skipping = false;
        for line in overlay.lines() {
            if line.starts_with("  - ") || !line.starts_with("    ") {
                skipping = line.starts_with("  - id: R5.1-") || line.starts_with("  - id: R5.3-");
            }
            if !skipping {
                out.push_str(line);
                out.push('\n');
            }
        }
        out
    }
}
