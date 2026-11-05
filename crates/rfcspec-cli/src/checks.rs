//! The profile rules `rfcspec check` adds to the overlay check: communication semantics against
//! the overlay's machines (design section 6), every requirement in scope has a status (design
//! section 5), and recorded runs against the current overlay.

use std::{
    collections::BTreeSet,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use rfcspec_model::{Catalogue, OVERLAY_FILE, Overlay, Profile, RUNS_DIR, RunReport, sha256_hex};

/// Every directory under `dir`, by name.
pub fn profiles(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_dir())
        .collect();
    found.sort();
    Ok(found)
}

/// The semantics problems of a profile, each naming the profile and the layer: a layer given
/// twice, machines without semantics, a machine transport with no layer of that name, and an
/// `over.profile` naming no directory beside the profile.
pub fn semantics(dir: &Path, profile: &Profile, overlay: &Overlay) -> Vec<String> {
    let name = &profile.name;
    let mut problems = Vec::new();
    let mut seen = BTreeSet::new();
    for layer in &profile.semantics {
        if !seen.insert(layer.layer.as_str()) {
            problems.push(format!(
                "profile {name}: layer {} is declared twice",
                layer.layer
            ));
        }
        if let Some(over) = &layer.over {
            let parent = dir.parent().unwrap_or_else(|| Path::new("."));
            if over.profile.is_empty()
                || over.profile.contains(['/', '\\'])
                || !parent.join(&over.profile).is_dir()
            {
                problems.push(format!(
                    "profile {name}: layer {} is over profile {}, but {} holds no such directory",
                    layer.layer,
                    over.profile,
                    parent.display()
                ));
            }
        }
    }
    if !overlay.machines.is_empty() && profile.semantics.is_empty() {
        problems.push(format!(
            "profile {name}: the overlay has machines ({}) but profile.yaml declares no semantics",
            overlay
                .machines
                .iter()
                .map(|m| m.name.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        ));
    } else {
        for machine in &overlay.machines {
            if !seen.contains(machine.transport.as_str()) {
                problems.push(format!(
                    "profile {name}: machine {} runs over transport {}, but no layer {} is declared",
                    machine.name, machine.transport, machine.transport
                ));
            }
        }
    }
    problems
}

/// Whether a catalogue section is in a profile scope: it or a section it lies under is listed.
pub fn in_scope(section: &str, scope: &[String]) -> bool {
    scope.iter().any(|listed| {
        section == listed
            || section
                .strip_prefix(listed.as_str())
                .is_some_and(|rest| rest.starts_with('.'))
    })
}

/// The scope problems of a profile (design section 5): every catalogue requirement, at every
/// level, of a section in scope (a section `profile.yaml` lists, or a subsection of one) needs a
/// status in the overlay. Each problem names the profile and the requirement.
pub fn scope(profile: &Profile, overlay: &Overlay, catalogue: &Catalogue) -> Vec<String> {
    let Some(rfc) = profile.rfcs.iter().find(|rfc| rfc.number == overlay.rfc) else {
        return Vec::new();
    };
    let reviewed: BTreeSet<&str> = overlay.requirements.iter().map(|r| r.id.as_str()).collect();
    catalogue
        .requirements
        .iter()
        .filter(|requirement| {
            rfc.sections.is_empty()
                || requirement
                    .section
                    .as_deref()
                    .is_some_and(|section| in_scope(section, &rfc.sections))
        })
        .filter(|requirement| !reviewed.contains(requirement.id.as_str()))
        .map(|requirement| {
            format!(
                "profile {}: requirement {} ({}, section {}) is in scope but has no status in {OVERLAY_FILE}",
                profile.name,
                requirement.id,
                requirement.keyword,
                requirement.section.as_deref().unwrap_or("none"),
            )
        })
        .collect()
}

/// Every `.json` file under the profile's `runs/`, by name.
pub fn run_files(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = fs::read_dir(dir.join(RUNS_DIR))
        .map(|entries| {
            entries
                .filter_map(std::result::Result::ok)
                .map(|entry| entry.path())
                .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "json"))
                .collect()
        })
        .unwrap_or_default();
    found.sort();
    found
}

/// The problems of the profile's recorded runs: a report that does not parse, or whose overlay
/// digest differs from the current `overlay.yaml` (a stale run), each naming the file.
pub fn runs(dir: &Path) -> Vec<String> {
    let files = run_files(dir);
    if files.is_empty() {
        return Vec::new();
    }
    let overlay = match fs::read(dir.join(OVERLAY_FILE)) {
        Ok(bytes) => sha256_hex(&bytes),
        Err(error) => return vec![format!("{}: {error}", dir.join(OVERLAY_FILE).display())],
    };
    files
        .iter()
        .filter_map(|path| {
            let report = fs::read_to_string(path)
                .map_err(anyhow::Error::from)
                .and_then(|text| RunReport::parse(&text));
            match report {
                Err(error) => Some(format!("{}: does not parse: {error:#}", path.display())),
                Ok(report) if report.overlay_sha256 != overlay => Some(format!(
                    "{}: stale run: recorded against overlay {}, overlay.yaml is now {overlay}",
                    path.display(),
                    report.overlay_sha256
                )),
                Ok(_) => None,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::in_scope;

    #[test]
    fn a_section_is_in_scope_with_its_subsections_only() {
        let scope = ["5.2".to_string(), "7".to_string()];
        for (section, expected) in [
            ("5.2", true),
            ("5.2.1", true),
            ("7", true),
            ("7.4.1", true),
            ("5.20", false),
            ("70", false),
            ("5", false),
            ("5.3", false),
        ] {
            assert_eq!(in_scope(section, &scope), expected, "{section}");
        }
    }
}
