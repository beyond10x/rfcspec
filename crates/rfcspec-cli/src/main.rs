#![forbid(unsafe_code)]
//! `rfcspec`: RFCs as executable specifications. Each subcommand is one pipeline stage
//! (`docs/design/rfcspec.md` section 4) over one profile directory.

mod bulk;
mod checks;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

use anyhow::{Context, Result, bail};
use clap::{Parser, Subcommand};
use rfcspec_model::{
    Catalogue, Document, Generated, OVERLAY_FILE, Overlay, PROFILE_FILE, Profile, SourceFormat,
    drift, read_yaml, to_json, write_generated,
};

#[derive(Parser)]
#[command(name = "rfcspec", about = "RFCs as executable specifications")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// S0: download an RFC's plain text or RFCXML v3 source into the profile's sources/ and lock
    /// its SHA-256, replacing the RFC's entry in the other format.
    Fetch {
        #[arg(long)]
        rfc: u32,
        #[arg(long)]
        profile: PathBuf,
        /// The source format: `rfc<N>.txt` or `rfc<N>.xml`.
        #[arg(long, value_enum, default_value_t = Format::Text)]
        format: Format,
        /// Where `rfc<N>.txt` or `rfc<N>.xml` is downloaded from.
        #[arg(long, default_value = rfcspec_source::RFC_EDITOR)]
        base_url: String,
    },
    /// S1: print the normalized document of a locked source as JSON.
    Read {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        rfc: u32,
    },
    /// S2: print the mechanical catalogue of a locked source as JSON.
    Extract {
        #[arg(long)]
        profile: PathBuf,
        #[arg(long)]
        rfc: u32,
    },
    /// S3: ask a model, through an `llm.routes/1` route, for an overlay element: a requirement
    /// entry (`status`, `requirement-from-prose`, `requirement-link`) for each catalogue item of
    /// the given sections without an overlay status or decision, a machine read from the sections
    /// (`machine`), or a layout named by `--item` read from them (`bit-widths`); check each answer
    /// and record it under the profile's `proposals/`; or, with `--replay`, rebuild every
    /// recorded proposal offline and exit 1 naming each file that is not reproduced byte for
    /// byte. Nothing else is written.
    Propose {
        #[arg(long)]
        profile: PathBuf,
        /// Replay the recorded proposals instead of asking a model.
        #[arg(long, conflicts_with_all = ["task", "sections", "items", "routes", "route"])]
        replay: bool,
        #[arg(long, value_enum, required_unless_present = "replay")]
        task: Option<ProposeTask>,
        /// The sections whose items (and their subsections') are asked about.
        #[arg(long = "section", value_name = "N", num_args = 1.., required_unless_present_any = ["replay", "items"])]
        sections: Vec<String>,
        /// Catalogue ids to ask about (`R5.1-1`, `P-34`), instead of every item of the sections.
        #[arg(long = "item", value_name = "ID", num_args = 1..)]
        items: Vec<String>,
        /// The `llm.routes/1` document naming the model and its credential references.
        #[arg(long, value_name = "FILE", required_unless_present = "replay")]
        routes: Option<PathBuf>,
        /// The route, or `<endpoint>/<model>` name, of the model to ask.
        #[arg(long, required_unless_present = "replay")]
        route: Option<String>,
    },
    /// S4: settle one recorded proposal with accept, edit or reject. Writes a decision into the
    /// profile's overlay, with the proposed (accept) or given (edit) requirement entry, layout or
    /// machine, only when the overlay and profile checks find no problem the overlay did not
    /// already have; otherwise writes nothing and exits 1.
    Review {
        #[arg(long)]
        profile: PathBuf,
        /// The proposal's id, its file name under `proposals/` without `.json`.
        #[arg(long, value_name = "ID")]
        proposal: String,
        #[command(subcommand)]
        verdict: ReviewVerdict,
    },
    /// S4: check the profile's overlay against the catalogue and the locked source, its
    /// communication semantics against the overlay's machines, that every catalogue requirement
    /// in the profile's scope has a status, every recorded run under `runs/` against the current
    /// overlay, that every recorded proposal under `proposals/` replays byte for byte, and that
    /// every overlay decision names a recorded proposal and judges its item's current text, a
    /// reject with a non-blank reason.
    Check {
        #[arg(long, required_unless_present = "all", conflicts_with = "all")]
        profile: Option<PathBuf>,
        /// Check every directory under this one as a profile; fail when one of them fails.
        #[arg(long, value_name = "DIR")]
        all: Option<PathBuf>,
    },
    /// S5: lower the overlay's machines to ess-protospec/1 models, one per transport.
    Lower {
        #[arg(long)]
        profile: PathBuf,
        /// Output directory; defaults to the profile's generated/.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Write nothing; fail when a generated model is missing or differs from a fresh lowering.
        #[arg(long)]
        check: bool,
    },
    /// S6: generate a codec crate from the overlay's layout, with the RFC's examples as tests.
    Generate {
        #[arg(value_enum, default_value = "rust")]
        target: Target,
        #[arg(long, default_value = "profiles/time")]
        profile: PathBuf,
        /// Output directory; defaults to the profile's generated/rust/.
        #[arg(long)]
        out: Option<PathBuf>,
        /// Write nothing; fail when a generated file is missing or differs from a fresh run.
        #[arg(long)]
        check: bool,
    },
    /// S7: probe a live TCP time server and print one requirement-matrix row per requirement.
    /// Exits 1 when a row fails.
    Verify {
        #[arg(long)]
        profile: PathBuf,
        /// `tcp://HOST:PORT`.
        #[arg(long)]
        target: String,
        /// Largest accepted difference from the local clock, in seconds.
        #[arg(long, default_value_t = 5)]
        tolerance: u64,
        /// How long to wait for the server to send and close, in seconds.
        #[arg(long, default_value_t = 10)]
        timeout: u64,
        /// Also record the run as an `rfcspec-run/1` report: at this `.json` path, or as
        /// `<target-slug>.json` in this directory, such as `profiles/time/runs`.
        #[arg(long, value_name = "PATH")]
        report: Option<PathBuf>,
    },
    /// S2 in bulk: the `rfcspec-bulk/1` summary of every RFC the index publishes as RFCXML or
    /// plain text.
    Bulk {
        #[command(subcommand)]
        command: BulkCommand,
    },
}

#[derive(Subcommand)]
enum BulkCommand {
    /// Download `rfc-index.xml` to `<cache>/rfc-index.xml` and, one at a time, the RFCXML of
    /// every RFC it lists with an XML format to `<cache>/xml/rfc<N>.xml` and the plain text of
    /// every other RFC it lists with a TXT format to `<cache>/txt/rfc<N>.txt`, skipping a source
    /// already cached unless `--refresh`. Refuses a cache inside a Git working tree. Exits 1 at
    /// the first failed download, naming the RFC and its URL; a rerun resumes.
    Fetch {
        /// Default `$XDG_CACHE_HOME/rfcspec/bulk`, else `$HOME/.cache/rfcspec/bulk`.
        #[arg(long, value_name = "DIR")]
        cache: Option<PathBuf>,
        #[arg(long, default_value = rfcspec_source::corpus::RFC_INDEX)]
        index_url: String,
        /// Where `rfc<N>.xml` and `rfc<N>.txt` are downloaded from.
        #[arg(long, default_value = rfcspec_source::RFC_EDITOR)]
        base_url: String,
        /// Download every source again, even one already cached.
        #[arg(long)]
        refresh: bool,
    },
    /// Write the summary of a copy of `rfc-index.xml`, a directory of RFC plain texts and a
    /// directory of RFCXML sources: per index entry with an XML or a TXT format, its index
    /// metadata, the source read (`<xml>/rfc<N>.xml` where the index lists XML, recorded as
    /// `"format":"xml"`, else `<texts>/rfc<N>.txt`), its SHA-256 and its catalogue requirements
    /// counted by BCP 14 level. Reads `<cache>/rfc-index.xml`, `<cache>/txt/` and `<cache>/xml/`
    /// (the cache `bulk fetch` writes, by default its default) unless `--index`, `--texts` and
    /// `--xml` are given. Exits 1, writing nothing, naming each RFC whose source is missing or
    /// whose XML does not read. Reads no network.
    Summarise {
        #[arg(long, value_name = "DIR", conflicts_with_all = ["index", "texts", "xml"])]
        cache: Option<PathBuf>,
        #[arg(long, value_name = "FILE", requires_all = ["texts", "xml"])]
        index: Option<PathBuf>,
        #[arg(long, value_name = "DIR", requires = "index")]
        texts: Option<PathBuf>,
        #[arg(long, value_name = "DIR", requires = "index")]
        xml: Option<PathBuf>,
        #[arg(long, value_name = "FILE")]
        out: PathBuf,
    },
    /// Check a summary offline: its header, row order, row formats, digests and counts, and every
    /// RFC locked in `<profiles>/*/rfcspec.lock` against its locked source, read in the locked
    /// format, which must be the row's. Exits 1 naming each line that fails.
    Check {
        #[arg(long, value_name = "FILE")]
        summary: PathBuf,
        #[arg(long, value_name = "DIR")]
        profiles: PathBuf,
    },
}

#[derive(Subcommand)]
enum ReviewVerdict {
    /// Add the proposed element and a decision of kind accept.
    Accept,
    /// Add the element of `--entry` instead and a decision of kind edit carrying it.
    Edit {
        /// A YAML file holding one overlay requirement entry, layout or machine, of the kind the
        /// proposal proposes.
        #[arg(long, value_name = "FILE")]
        entry: Option<PathBuf>,
    },
    /// Add a decision of kind reject and no element; the item stays unreviewed and is not
    /// proposed again.
    Reject {
        /// Why; recorded as the decision's reason and printed. An empty or blank reason is
        /// refused as `DecisionIncomplete`.
        #[arg(long)]
        reason: String,
    },
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Format {
    /// Plain text, `rfc<N>.txt`.
    Text,
    /// RFCXML v3, `rfc<N>.xml`.
    Xml,
}

impl From<Format> for SourceFormat {
    fn from(format: Format) -> Self {
        match format {
            Format::Text => SourceFormat::Text,
            Format::Xml => SourceFormat::Xml,
        }
    }
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Target {
    Rust,
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum ProposeTask {
    /// A status unsupported or not-applicable with a reason, and a level, for a BCP 14
    /// requirement.
    Status,
    /// A level and a status for a sentence with a lowercase `must`, `should` or `may`.
    RequirementFromProse,
    /// One overlay machine read from the sections; the item is the whole profile.
    Machine,
    /// One overlay layout read from a packet diagram of the sections, under the name `--item`
    /// gives.
    BitWidths,
    /// A status satisfied by an element the overlay holds, for a BCP 14 requirement.
    RequirementLink,
}

impl From<ProposeTask> for rfcspec_propose::Task {
    fn from(task: ProposeTask) -> Self {
        match task {
            ProposeTask::Status => Self::Status,
            ProposeTask::RequirementFromProse => Self::RequirementFromProse,
            ProposeTask::Machine => Self::Machine,
            ProposeTask::BitWidths => Self::BitWidths,
            ProposeTask::RequirementLink => Self::RequirementLink,
        }
    }
}

fn main() -> ExitCode {
    match run(Cli::parse()) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("rfcspec: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn run(cli: Cli) -> Result<ExitCode> {
    match cli.command {
        Command::Fetch {
            rfc,
            profile,
            format,
            base_url,
        } => {
            let fetched = rfcspec_source::fetch_format(&profile, rfc, &base_url, format.into())?;
            println!(
                "{} {} sha256 {}{}",
                fetched.entry.rfc,
                fetched.entry.file,
                fetched.entry.sha256,
                if fetched.source_written || fetched.lock_written {
                    ""
                } else {
                    " (unchanged)"
                }
            );
        }
        Command::Read { profile, rfc } => {
            let (document, _) = read_document(&profile, rfc)?;
            print!("{}", to_json(&document)?);
        }
        Command::Extract { profile, rfc } => {
            let (document, _) = read_document(&profile, rfc)?;
            let (catalogue, problems) = rfcspec_extract::extract_reporting(&document);
            for problem in problems {
                eprintln!("rfcspec extract: {problem}");
            }
            print!("{}", to_json(&catalogue)?);
        }
        Command::Lower {
            profile,
            out,
            check,
        } => {
            let loaded = Loaded::from(&profile)?;
            let lowered = rfcspec_ess::lower(&loaded.overlay, &loaded.profile.name)?;
            let out = out.unwrap_or_else(|| profile.join("generated"));
            return emit(check, "lower", &out, &lowered);
        }
        Command::Generate {
            target: Target::Rust,
            profile,
            out,
            check,
        } => {
            let loaded = Loaded::from(&profile)?;
            let files =
                rfcspec_gen::rust(&loaded.overlay, &loaded.catalogue, &loaded.profile.name)?;
            let out = out.unwrap_or_else(|| profile.join("generated/rust"));
            return emit(check, "generate", &out, &files);
        }
        Command::Verify {
            profile,
            target,
            tolerance,
            timeout,
            report,
        } => {
            let overlay_bytes = std::fs::read(profile.join(OVERLAY_FILE))?;
            let overlay: Overlay = read_yaml(&profile.join(OVERLAY_FILE))?;
            let options = rfcspec_conformance::Options {
                tolerance: Duration::from_secs(tolerance),
                timeout: Duration::from_secs(timeout),
            };
            let rows = rfcspec_conformance::verify(&overlay, &target, options)?;
            print!("{}", rfcspec_conformance::render(&rows));
            let code = rfcspec_conformance::exit_code(&rows);
            if let Some(path) = report {
                let path = rfcspec_conformance::report_path(&path, &target);
                let report = rfcspec_conformance::report(
                    &target,
                    rfcspec_conformance::now_unix(),
                    &overlay_bytes,
                    rows,
                );
                rfcspec_conformance::write_report(&path, &report)?;
                eprintln!(
                    "rfcspec verify: recorded {} as {}",
                    report.verdict,
                    path.display()
                );
            }
            return Ok(ExitCode::from(code));
        }
        Command::Propose {
            profile,
            replay: true,
            ..
        } => {
            let problems = rfcspec_propose::replay(&profile)?;
            for problem in &problems {
                eprintln!("rfcspec propose: {problem}");
            }
            if !problems.is_empty() {
                return Ok(ExitCode::FAILURE);
            }
            println!(
                "rfcspec propose: every proposal of {} replays",
                profile.display()
            );
        }
        Command::Propose {
            profile,
            task,
            sections,
            items,
            routes,
            route,
            ..
        } => {
            let (Some(task), Some(routes), Some(route)) = (task, routes, route) else {
                bail!("give --task, --routes and --route, or --replay");
            };
            return propose(&profile, task.into(), &sections, items, &routes, &route);
        }
        Command::Review {
            profile,
            proposal,
            verdict,
        } => return review(&profile, &proposal, verdict),
        Command::Bulk {
            command:
                BulkCommand::Fetch {
                    cache,
                    index_url,
                    base_url,
                    refresh,
                },
        } => {
            use rfcspec_source::corpus::{default_cache, fetch_corpus};
            let cache = cache.map_or_else(default_cache, Ok)?;
            let corpus = fetch_corpus(&cache, &index_url, &base_url, refresh, |rfc| {
                eprintln!("rfcspec bulk: fetched RFC {rfc}");
            })?;
            println!(
                "rfcspec bulk: {}: index sha256 {}, {} XML, {} texts, {} downloaded, {} already cached",
                cache.display(),
                corpus.index_sha256,
                corpus.xml,
                corpus.texts,
                corpus.downloaded,
                corpus.xml + corpus.texts - corpus.downloaded
            );
        }
        Command::Bulk {
            command:
                BulkCommand::Summarise {
                    cache,
                    index,
                    texts,
                    xml,
                    out,
                },
        } => {
            use rfcspec_source::corpus::{INDEX_FILE, TEXT_DIR, XML_DIR, default_cache};
            let (index, texts, xml) = match (index, texts, xml) {
                (Some(index), Some(texts), Some(xml)) => (index, texts, xml),
                _ => {
                    let cache = cache.map_or_else(default_cache, Ok)?;
                    (
                        cache.join(INDEX_FILE),
                        cache.join(TEXT_DIR),
                        cache.join(XML_DIR),
                    )
                }
            };
            let summary = bulk::summarise(&index, &texts, &xml)?;
            if let Some(parent) = out.parent().filter(|parent| !parent.as_os_str().is_empty()) {
                std::fs::create_dir_all(parent)
                    .with_context(|| format!("creating {}", parent.display()))?;
            }
            std::fs::write(&out, &summary).with_context(|| format!("writing {}", out.display()))?;
            println!(
                "rfcspec bulk: wrote {} ({} rows)",
                out.display(),
                summary.lines().count() - 1
            );
        }
        Command::Bulk {
            command: BulkCommand::Check { summary, profiles },
        } => {
            let problems = bulk::check(&summary, &profiles)?;
            for problem in &problems {
                eprintln!("rfcspec bulk: {}: {problem}", summary.display());
            }
            if !problems.is_empty() {
                return Ok(ExitCode::FAILURE);
            }
            println!("rfcspec bulk: {} holds", summary.display());
        }
        Command::Check { profile, all } => {
            let profiles = match (profile, all) {
                (Some(profile), _) => vec![profile],
                (None, Some(all)) => checks::profiles(&all)?,
                (None, None) => bail!("give --profile or --all"),
            };
            let mut failed = Vec::new();
            for profile in &profiles {
                if !check(profile) {
                    failed.push(profile.display().to_string());
                }
            }
            if !failed.is_empty() {
                if profiles.len() > 1 {
                    eprintln!(
                        "rfcspec check: {} of {} profiles fail: {}",
                        failed.len(),
                        profiles.len(),
                        failed.join(", ")
                    );
                }
                return Ok(ExitCode::FAILURE);
            }
        }
    }
    Ok(ExitCode::SUCCESS)
}

/// Check one profile, print what holds or every problem, and say whether it passed.
fn check(profile: &Path) -> bool {
    let problems = match Loaded::from(profile) {
        Err(error) => vec![format!("{}: {error:#}", profile.display())],
        Ok(loaded) => {
            let mut problems =
                rfcspec_extract::check(&loaded.overlay, &loaded.catalogue, &loaded.source);
            problems.extend(checks::semantics(profile, &loaded.profile, &loaded.overlay));
            problems.extend(checks::scope(
                &loaded.profile,
                &loaded.overlay,
                &loaded.catalogue,
            ));
            problems.extend(checks::runs(profile));
            problems.extend(checks::decisions(profile, &loaded.overlay));
            match rfcspec_propose::replay(profile) {
                Ok(replayed) => problems.extend(replayed),
                Err(error) => problems.push(format!("{}: {error:#}", profile.display())),
            }
            if problems.is_empty() {
                let constructs = rfcspec_ess::model_constructs(&loaded.overlay);
                let named = if constructs.is_empty() {
                    String::new()
                } else {
                    format!(
                        "; {} model constructs implement no requirement: {}",
                        constructs.len(),
                        constructs.join(", ")
                    )
                };
                println!(
                    "{}: overlay holds ({} requirements, {} layouts, {} machines, {} findings, {} layers, {} recorded runs){named}",
                    profile.display(),
                    loaded.overlay.requirements.len(),
                    loaded.overlay.layouts.len(),
                    loaded.overlay.machines.len(),
                    loaded.overlay.findings.len(),
                    loaded.profile.semantics.len(),
                    checks::run_files(profile).len(),
                );
            }
            problems
        }
    };
    for problem in &problems {
        eprintln!("rfcspec check: {problem}");
    }
    problems.is_empty()
}

/// Settle one proposal; write the overlay only when the overlay check (inside `decide`) and the
/// profile checks find no problem on the result that the overlay did not already have.
fn review(profile: &Path, proposal: &str, verdict: ReviewVerdict) -> Result<ExitCode> {
    use rfcspec_propose::review::{ReviewError, Verdict, decide, write};
    let verdict = match verdict {
        ReviewVerdict::Accept => Verdict::Accept,
        ReviewVerdict::Edit { entry } => Verdict::Edit(
            entry
                .as_deref()
                .map(read_yaml::<rfcspec_model::Edit>)
                .transpose()?,
        ),
        ReviewVerdict::Reject { reason } => Verdict::Reject(reason),
    };
    let reviewed = match decide(profile, proposal, verdict) {
        Ok(reviewed) => reviewed,
        Err(ReviewError::Refused(problems)) => {
            for problem in &problems {
                eprintln!("rfcspec review: {problem}");
            }
            return Ok(ExitCode::FAILURE);
        }
        Err(error) => {
            eprintln!("rfcspec review: {error}");
            return Ok(ExitCode::FAILURE);
        }
    };
    let loaded = Loaded::from(profile)?;
    let profile_problems = |overlay: &Overlay| {
        let mut problems = checks::semantics(profile, &loaded.profile, overlay);
        problems.extend(checks::scope(&loaded.profile, overlay, &loaded.catalogue));
        problems.extend(checks::decisions(profile, overlay));
        problems
    };
    let before = profile_problems(&loaded.overlay);
    let problems: Vec<String> = profile_problems(&reviewed.overlay)
        .into_iter()
        .filter(|problem| !before.contains(problem))
        .collect();
    if !problems.is_empty() {
        for problem in &problems {
            eprintln!("rfcspec review: {problem}");
        }
        return Ok(ExitCode::FAILURE);
    }
    write(profile, &reviewed)?;
    let decision = &reviewed.decision;
    println!(
        "rfcspec review: {}: {} of proposal {proposal} recorded in {}{}",
        decision.item,
        decision.kind,
        profile.join(OVERLAY_FILE).display(),
        decision
            .reason
            .as_deref()
            .map(|reason| format!(" ({reason})"))
            .unwrap_or_default()
    );
    Ok(ExitCode::SUCCESS)
}

/// Record a proposal for each item: the given ones, else every pending one of the sections.
/// Exits 1 when an answer fails its checks; stops at an error from the model.
fn propose(
    profile: &Path,
    task: rfcspec_propose::Task,
    sections: &[String],
    items: Vec<String>,
    routes: &Path,
    route: &str,
) -> Result<ExitCode> {
    let workspace = rfcspec_propose::Workspace::load(profile)?;
    if task.reads_sections() && sections.is_empty() {
        bail!("--task {} reads --section", task.wire());
    }
    if task == rfcspec_propose::Task::BitWidths && items.is_empty() {
        bail!("--task bit-widths names its layout with --item");
    }
    let items = if items.is_empty() {
        workspace.pending(task, sections)
    } else {
        items
    };
    if items.is_empty() {
        println!("rfcspec propose: nothing to propose");
        return Ok(ExitCode::SUCCESS);
    }
    let text =
        std::fs::read_to_string(routes).with_context(|| format!("reading {}", routes.display()))?;
    let routed = rfcspec_propose::routed(&text, route)?;
    let mut failed = false;
    for (item, result) in rfcspec_propose::record_all(
        &workspace,
        &routed.model,
        &routed.model_id,
        task,
        &items,
        sections,
    )? {
        match result {
            Ok(rfcspec_propose::Outcome::Recorded(path)) => {
                println!("rfcspec propose: {item}: recorded {}", path.display());
            }
            Ok(rfcspec_propose::Outcome::AlreadyExists(path)) => {
                println!(
                    "rfcspec propose: {item}: already recorded {}",
                    path.display()
                );
            }
            Ok(rfcspec_propose::Outcome::Decided) => {
                println!("rfcspec propose: {item}: decided in the overlay, not asked");
            }
            Err(rfcspec_propose::ProposeError::Model(error)) => {
                bail!("{item}: {error}");
            }
            Err(error) => {
                eprintln!("rfcspec propose: {item}: {error}");
                failed = true;
            }
        }
    }
    Ok(if failed {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    })
}

/// A locked source read into its normalized document by the reader of its format, refused unless
/// the profile names the RFC.
fn read_document(profile: &Path, rfc: u32) -> Result<(Document, String)> {
    let named: Profile = read_yaml(&profile.join(PROFILE_FILE))?;
    if !named.rfcs.iter().any(|entry| entry.number == rfc) {
        bail!("profile {} does not name RFC {rfc}", named.name);
    }
    let locked = rfcspec_source::read_locked(profile, rfc)?;
    let document = rfcspec_doc::read_format(rfc, locked.format, &locked.entry.sha256, &locked.text)
        .with_context(|| format!("reading {}", locked.entry.file))?;
    Ok((document, locked.text))
}

/// Write generated files, or with `check`, report the drifted ones and fail.
fn emit(check: bool, stage: &str, out: &Path, files: &[Generated]) -> Result<ExitCode> {
    if check {
        let drifted = drift(out, files);
        if drifted.is_empty() {
            println!("rfcspec {stage}: {} is current", out.display());
            return Ok(ExitCode::SUCCESS);
        }
        for path in drifted {
            eprintln!(
                "rfcspec {stage}: {} is missing or differs from a fresh run",
                path.display()
            );
        }
        return Ok(ExitCode::FAILURE);
    }
    write_generated(out, files)?;
    println!("rfcspec {stage}: wrote {}", out.display());
    Ok(ExitCode::SUCCESS)
}

/// A profile's overlay with the catalogue and source of its RFC.
struct Loaded {
    profile: Profile,
    overlay: Overlay,
    catalogue: Catalogue,
    source: String,
}

impl Loaded {
    fn from(profile: &Path) -> Result<Self> {
        let overlay: Overlay = read_yaml(&profile.join(OVERLAY_FILE))?;
        let (document, source) = read_document(profile, overlay.rfc)?;
        Ok(Self {
            profile: read_yaml(&profile.join(PROFILE_FILE))?,
            catalogue: rfcspec_extract::extract(&document),
            overlay,
            source,
        })
    }
}
