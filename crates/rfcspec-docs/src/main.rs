#![forbid(unsafe_code)]
//! Generates the derivable part of the RFCspec documentation site, checks it for drift, and binds a
//! built site to its source commit. The site build itself is Docusaurus, under `website/`.
//!
//! It generates:
//!
//! - the design page (`website/docs/design.md`) from `docs/design/rfcspec.md` ([`design`]);
//! - the status page and data (`website/docs/status.md`, `website/data/status.json`) ([`status`]);
//! - the protocol catalogue (`website/docs/catalog/`) and its downloads
//!   (`website/static/downloads/`) from `profiles/` ([`catalog`]).
//!
//! The catalogue directories hold nothing else: a file there that `generate` would not write is
//! reported `unexpected` by `generate --check` and deleted by `generate`.
//!
//! `generate --check` also refuses a hand-written page that an MDX reader would execute: a brace
//! or a capitalised tag outside code ([`page::passive_problems`]).
mod design;
mod routes;
mod status;

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail, ensure};
use clap::{Parser, Subcommand};
use rfcspec_docs::{catalog, page};

#[derive(Parser)]
#[command(about = "Generate and check the derived pages of the RFCspec documentation site")]
struct Cli {
    #[command(subcommand)]
    command: Action,
}

#[derive(Subcommand)]
enum Action {
    /// Regenerate the derived pages of website/ from the repository, or check them for drift.
    Generate {
        /// Write nothing; fail when a generated file is missing, stale, edited by hand or
        /// unexpected, or a hand-written page is not passive Markdown.
        #[arg(long)]
        check: bool,
        /// Repository root.
        #[arg(long, default_value = ".")]
        root: PathBuf,
    },
    /// Bind a built site to its source commit: write `.well-known/b10x-site.json` and
    /// `.well-known/b10x-routes.json` into it.
    Provenance {
        /// The built site, `website/build`.
        #[arg(long)]
        site: PathBuf,
        /// The full Git revision the site was built from.
        #[arg(long)]
        commit: String,
    },
}

/// Whether a file under an owned directory, by path from the repository root, is generated output.
type Owned = fn(&str) -> bool;

/// The directories only `generate` writes, relative to the repository root, and whether a file
/// there is generated output: every Markdown page under the catalogue, every download.
const OWNED: &[(&str, Owned)] = &[
    (catalog::PAGES, |path| path.ends_with(".md")),
    (catalog::DOWNLOADS, |_| true),
];

/// Every generated file, by path relative to the repository root.
fn outputs(root: &Path) -> Result<BTreeMap<String, String>> {
    let design = fs::read_to_string(root.join(design::SOURCE))
        .with_context(|| format!("reading {}", design::SOURCE))?;
    let mut out = BTreeMap::from([
        (design::PAGE.to_owned(), design::render(&design)?),
        (status::PAGE.to_owned(), status::page()),
        (status::DATA.to_owned(), status::data()),
    ]);
    out.extend(catalog::outputs(root)?);
    Ok(out)
}

/// Every generated file that is missing or differs from what the repository generates.
fn drift(root: &Path, outputs: &BTreeMap<String, String>) -> Vec<String> {
    outputs
        .iter()
        .filter_map(|(path, content)| match fs::read(root.join(path)) {
            Err(_) => Some(format!("missing {path}")),
            Ok(found) if found != content.as_bytes() => Some(format!("stale or edited {path}")),
            Ok(_) => None,
        })
        .collect()
}

/// Every file in a directory `generate` owns that it would not write.
fn unexpected(root: &Path, outputs: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let mut found = Vec::new();
    for (dir, owned) in OWNED {
        for file in catalog::files(&root.join(dir))? {
            let path = format!("{dir}/{file}");
            if owned(&path) && !outputs.contains_key(&path) {
                found.push(path);
            }
        }
    }
    Ok(found)
}

/// Write every generated file and delete the unexpected ones, with the directories they leave
/// empty.
fn write(root: &Path, outputs: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let removed = unexpected(root, outputs)?;
    for path in &removed {
        let target = root.join(path);
        fs::remove_file(&target).with_context(|| format!("deleting {path}"))?;
        let mut dir = target.parent();
        while let Some(parent) = dir {
            if !OWNED
                .iter()
                .any(|(owned, _)| parent.starts_with(root.join(owned)))
                || fs::remove_dir(parent).is_err()
            {
                break;
            }
            dir = parent.parent();
        }
    }
    for (path, content) in outputs {
        let target = root.join(path);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&target, content).with_context(|| format!("writing {path}"))?;
    }
    Ok(removed)
}

/// What `generate --check` reports: drift, unexpected files and pages that are not passive.
fn problems(root: &Path, outputs: &BTreeMap<String, String>) -> Result<Vec<String>> {
    let mut problems = drift(root, outputs);
    problems.extend(
        unexpected(root, outputs)?
            .into_iter()
            .map(|path| format!("unexpected {path}")),
    );
    problems.extend(passive_check(root)?);
    Ok(problems)
}

/// Every Markdown page under `website/docs` with a line that is not passive.
fn passive_check(root: &Path) -> Result<Vec<String>> {
    let mut problems = Vec::new();
    let mut stack = vec![root.join("website/docs")];
    while let Some(dir) = stack.pop() {
        if !dir.is_dir() {
            continue;
        }
        for entry in fs::read_dir(&dir).with_context(|| format!("reading {}", dir.display()))? {
            let path = entry?.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "md") {
                let text = fs::read_to_string(&path)?;
                for line in page::passive_problems(&text) {
                    let relative = path.strip_prefix(root).unwrap_or(&path);
                    problems.push(format!(
                        "{}:{line}: a brace or capitalised tag outside code",
                        relative.display()
                    ));
                }
            }
        }
    }
    problems.sort();
    Ok(problems)
}

/// The project-site publication record for one build of the `/rfcspec/` site.
fn site_manifest(commit: &str) -> Result<String> {
    ensure!(
        commit.len() == 40
            && commit.bytes().all(|b| b.is_ascii_hexdigit())
            && commit != "0".repeat(40),
        "commit must be a nonzero full Git revision"
    );
    Ok(format!(
        "{{\n  \"baseUrl\": \"{}\",\n  \"commit\": \"{commit}\",\n  \"repository\": \"rfcspec\",\n  \"schema\": \"b10x-project-site/v1\"\n}}",
        page::BASE_URL
    ))
}

fn write_provenance(site: &Path, commit: &str) -> Result<()> {
    let manifest = site_manifest(commit)?;
    ensure!(
        site.join("index.html").is_file(),
        "{} holds no built site",
        site.display()
    );
    let routes = routes::inventory(site, commit)?;
    fs::create_dir_all(site.join(".well-known"))?;
    fs::write(site.join(".well-known/b10x-site.json"), manifest)
        .context("writing site provenance")?;
    fs::write(site.join(".well-known/b10x-routes.json"), routes)
        .context("writing the route inventory")?;
    fs::write(site.join(".nojekyll"), "")?;
    Ok(())
}

fn main() -> Result<()> {
    match Cli::parse().command {
        Action::Generate { check, root } => {
            let outputs = outputs(&root)?;
            if check {
                let problems = problems(&root, &outputs)?;
                if !problems.is_empty() {
                    bail!(
                        "the documentation has drifted; run `task docs-generate`:\n  {}",
                        problems.join("\n  ")
                    );
                }
                println!("website: {} generated files current", outputs.len());
            } else {
                for path in write(&root, &outputs)? {
                    println!("website: deleted unexpected {path}");
                }
                println!("website: wrote {} files", outputs.len());
            }
        }
        Action::Provenance { site, commit } => {
            write_provenance(&site, &commit)?;
            println!("{}: bound to {commit}", site.display());
        }
    }
    Ok(())
}

/// A fresh directory under the workspace's build directory, not the system temporary one.
#[cfg(test)]
fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/rfcspec-docs")
        .join(format!("test-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_names_the_commit_writes_routes_and_refuses_a_bad_commit() {
        let commit = "e86f43d90b637fc2c1790f77d59efff2eb852386";
        let manifest = site_manifest(commit).unwrap();
        assert!(manifest.contains(&format!("\"commit\": \"{commit}\"")));
        assert!(manifest.contains("\"baseUrl\": \"/rfcspec/\""));
        assert!(site_manifest("e86f43d").is_err());
        assert!(site_manifest(&"0".repeat(40)).is_err());

        let site = scratch("provenance");
        assert!(write_provenance(&site, commit).is_err());
        fs::write(site.join("index.html"), "<main id=\"main\">").unwrap();
        write_provenance(&site, commit).unwrap();
        assert_eq!(
            fs::read_to_string(site.join(".well-known/b10x-site.json")).unwrap(),
            manifest
        );
        let routes = fs::read_to_string(site.join(".well-known/b10x-routes.json")).unwrap();
        assert!(routes.contains("\"path\": \"/rfcspec/\""));
        fs::remove_dir_all(site).unwrap();
    }

    #[test]
    fn a_missing_or_edited_page_is_drift() {
        let root = scratch("drift");
        let outputs = BTreeMap::from([
            ("a.md".to_owned(), "A\n".to_owned()),
            ("b.md".to_owned(), "B\n".to_owned()),
        ]);
        fs::write(root.join("b.md"), "B, edited\n").unwrap();
        assert_eq!(
            drift(&root, &outputs),
            ["missing a.md", "stale or edited b.md"]
        );
        fs::write(root.join("a.md"), "A\n").unwrap();
        fs::write(root.join("b.md"), "B\n").unwrap();
        assert!(drift(&root, &outputs).is_empty());
        fs::remove_dir_all(root).unwrap();
    }

    /// A scratch repository root holding the design document and a copy of `profiles/time`.
    fn repository_with_time(name: &str) -> PathBuf {
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root = scratch(name);
        let files = [design::SOURCE.to_owned()].into_iter().chain(
            catalog::files(&repository.join("profiles/time"))
                .unwrap()
                .into_iter()
                .map(|file| format!("profiles/time/{file}")),
        );
        for file in files {
            fs::create_dir_all(root.join(&file).parent().unwrap()).unwrap();
            fs::copy(repository.join(&file), root.join(&file)).unwrap();
        }
        root
    }

    fn check(root: &Path) -> Vec<String> {
        problems(root, &outputs(root).unwrap()).unwrap()
    }

    #[test]
    fn the_catalogue_is_generated_from_profiles_and_its_drift_is_reported_by_name() {
        let root = repository_with_time("catalog-drift");
        let outputs = outputs(&root).unwrap();
        for path in [
            catalog::INDEX,
            "website/docs/catalog/time.md",
            "website/static/downloads/time/tcp.protospec.yaml",
            "website/static/downloads/time/udp.protospec.yaml",
            "website/static/downloads/time/rust/Cargo.toml",
            "website/static/downloads/time/rust/src/lib.rs",
        ] {
            assert!(outputs.contains_key(path), "{path}");
        }
        assert!(
            check(&root).contains(&format!("missing {}", catalog::INDEX)),
            "a missing index is drift"
        );
        assert!(write(&root, &outputs).unwrap().is_empty());
        assert_eq!(check(&root), Vec::<String>::new());

        for path in [
            catalog::INDEX,
            "website/docs/catalog/time.md",
            "website/static/downloads/time/rust/src/lib.rs",
        ] {
            let mut bytes = fs::read(root.join(path)).unwrap();
            bytes.push(b'\n');
            fs::write(root.join(path), &bytes).unwrap();
            assert_eq!(check(&root), [format!("stale or edited {path}")], "{path}");
            fs::write(root.join(path), &outputs[path]).unwrap();

            fs::remove_file(root.join(path)).unwrap();
            assert_eq!(check(&root), [format!("missing {path}")], "{path}");
            fs::write(root.join(path), &outputs[path]).unwrap();
        }

        let generated = root.join("profiles/time/generated/tcp.protospec.yaml");
        let mut source = fs::read_to_string(&generated).unwrap();
        source.push_str("# changed\n");
        fs::write(&generated, &source).unwrap();
        assert_eq!(
            check(&root),
            [
                "stale or edited website/docs/catalog/time.md",
                "stale or edited website/static/downloads/time/tcp.protospec.yaml"
            ],
            "a changed source makes its copy and the page that lists its digest stale"
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn an_orphaned_page_or_download_is_unexpected_and_deleted_by_generate() {
        let root = repository_with_time("catalog-orphan");
        write(&root, &outputs(&root).unwrap()).unwrap();
        let orphans = [
            "website/docs/catalog/gone.md",
            "website/docs/catalog/notes/extra.md",
            "website/static/downloads/gone/rust/Cargo.toml",
            "website/static/downloads/time/old.protospec.yaml",
        ];
        for orphan in orphans {
            fs::create_dir_all(root.join(orphan).parent().unwrap()).unwrap();
            fs::write(root.join(orphan), "x\n").unwrap();
        }
        fs::write(root.join("website/docs/catalog/_category_.json"), "{}").unwrap();
        let mut expected: Vec<String> = orphans.iter().map(|o| format!("unexpected {o}")).collect();
        expected.sort();
        let mut found = check(&root);
        found.sort();
        assert_eq!(found, expected);

        let mut removed = write(&root, &outputs(&root).unwrap()).unwrap();
        removed.sort();
        assert_eq!(removed, orphans);
        for orphan in orphans {
            assert!(!root.join(orphan).exists(), "{orphan}");
        }
        assert!(!root.join("website/static/downloads/gone").exists());
        assert!(!root.join("website/docs/catalog/notes").exists());
        assert!(root.join("website/docs/catalog/_category_.json").exists());
        assert_eq!(check(&root), Vec::<String>::new());

        fs::remove_dir_all(root.join("profiles/time")).unwrap();
        assert!(check(&root).contains(&"unexpected website/docs/catalog/time.md".to_owned()));
        write(&root, &outputs(&root).unwrap()).unwrap();
        assert!(!root.join("website/docs/catalog/time.md").exists());
        assert!(!root.join("website/static/downloads/time").exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn the_repository_catalogue_is_current() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let outputs = catalog::outputs(&root).unwrap();
        assert!(drift(&root, &outputs).is_empty());
        assert!(unexpected(&root, &outputs).unwrap().is_empty());
    }
}
