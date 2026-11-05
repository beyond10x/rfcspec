#![forbid(unsafe_code)]
//! `rfcspec`: RFCs as executable specifications. Each subcommand is one pipeline stage
//! (`docs/design/rfcspec.md` section 4) over one profile directory.

mod checks;

use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    time::Duration,
};

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use rfcspec_model::{
    Catalogue, Document, Generated, OVERLAY_FILE, Overlay, PROFILE_FILE, Profile, drift, read_yaml,
    to_json, write_generated,
};

#[derive(Parser)]
#[command(name = "rfcspec", about = "RFCs as executable specifications")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// S0: download an RFC's plain text into the profile's sources/ and lock its SHA-256.
    Fetch {
        #[arg(long)]
        rfc: u32,
        #[arg(long)]
        profile: PathBuf,
        /// Where `rfc<N>.txt` is downloaded from.
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
    /// S4: check the profile's overlay against the catalogue and the locked source, its
    /// communication semantics against the overlay's machines, that every catalogue requirement
    /// in the profile's scope has a status, and every recorded run under `runs/` against the
    /// current overlay.
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
}

#[derive(Clone, Copy, clap::ValueEnum)]
enum Target {
    Rust,
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
            base_url,
        } => {
            let fetched = rfcspec_source::fetch(&profile, rfc, &base_url)?;
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
            print!("{}", to_json(&rfcspec_extract::extract(&document))?);
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
            if problems.is_empty() {
                println!(
                    "{}: overlay holds ({} requirements, {} layouts, {} machines, {} findings, {} layers, {} recorded runs)",
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

/// A locked source read into its normalized document, refused unless the profile names the RFC.
fn read_document(profile: &Path, rfc: u32) -> Result<(Document, String)> {
    let named: Profile = read_yaml(&profile.join(PROFILE_FILE))?;
    if !named.rfcs.iter().any(|entry| entry.number == rfc) {
        bail!("profile {} does not name RFC {rfc}", named.name);
    }
    let locked = rfcspec_source::read_locked(profile, rfc)?;
    Ok((
        rfcspec_doc::read(rfc, &locked.entry.sha256, &locked.text),
        locked.text,
    ))
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
