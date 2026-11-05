//! The bulk corpus: `rfc-index.xml` and the plain text of every RFC it lists with a TXT format,
//! downloaded one at a time into a cache directory outside every Git working tree, so that no RFC
//! text can land in a repository.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result, bail};

use crate::{MAX_SOURCE, agent, get, index::parse_index, sha256_hex};

/// Where `rfc-index.xml` is fetched from by default.
pub const RFC_INDEX: &str = "https://www.rfc-editor.org/rfc-index.xml";

/// The index file inside a cache.
pub const INDEX_FILE: &str = "rfc-index.xml";

/// The directory of RFC texts inside a cache.
pub const TEXT_DIR: &str = "txt";

/// The largest index accepted, in bytes.
const MAX_INDEX: u64 = 64 * 1024 * 1024;

/// The cache used when none is given: `$XDG_CACHE_HOME/rfcspec/bulk`, else
/// `$HOME/.cache/rfcspec/bulk`.
pub fn default_cache() -> Result<PathBuf> {
    let base = match std::env::var_os("XDG_CACHE_HOME").filter(|value| !value.is_empty()) {
        Some(cache) => PathBuf::from(cache),
        None => std::env::var_os("HOME")
            .filter(|value| !value.is_empty())
            .map(|home| PathBuf::from(home).join(".cache"))
            .context("neither XDG_CACHE_HOME nor HOME is set; give --cache")?,
    };
    Ok(base.join("rfcspec/bulk"))
}

/// The text of RFC `rfc` inside a cache.
pub fn text_path(cache: &Path, rfc: u32) -> PathBuf {
    cache.join(TEXT_DIR).join(format!("rfc{rfc}.txt"))
}

/// The Git working tree a path lies in, if any: the nearest directory, from the path's nearest
/// existing ancestor upwards, that holds a `.git` directory or file.
pub fn git_working_tree(path: &Path) -> Result<Option<PathBuf>> {
    let absolute =
        std::path::absolute(path).with_context(|| format!("resolving {}", path.display()))?;
    let existing = absolute
        .ancestors()
        .find(|ancestor| ancestor.exists())
        .context("no ancestor of the cache exists")?;
    let existing = existing
        .canonicalize()
        .with_context(|| format!("resolving {}", existing.display()))?;
    Ok(existing
        .ancestors()
        .find(|ancestor| ancestor.join(".git").exists())
        .map(Path::to_path_buf))
}

/// What [`fetch_corpus`] did.
#[derive(Debug, PartialEq, Eq)]
pub struct Corpus {
    /// SHA-256 of the index fetched.
    pub index_sha256: String,
    /// Index entries with a TXT format.
    pub texts: usize,
    /// Texts downloaded in this run; the others were already in the cache.
    pub downloaded: usize,
}

/// Download `index_url` to `<cache>/rfc-index.xml`, then, one at a time, `<base>rfc<N>.txt` to
/// `<cache>/txt/rfc<N>.txt` for every entry with a TXT format, skipping a text already cached
/// unless `refresh`. Refuses a cache inside a Git working tree before any request. Stops at the
/// first failed download, naming the RFC and its URL; every text written before it stays, so a
/// rerun resumes. `downloaded` is called with each RFC number after its text is written.
pub fn fetch_corpus(
    cache: &Path,
    index_url: &str,
    base: &str,
    refresh: bool,
    mut downloaded: impl FnMut(u32),
) -> Result<Corpus> {
    if let Some(tree) = git_working_tree(cache)? {
        bail!(
            "{} is inside the Git working tree {}; RFC texts are cached outside every repository",
            cache.display(),
            tree.display()
        );
    }
    let agent = agent();
    let index = get(&agent, index_url, MAX_INDEX)?;
    let xml = std::str::from_utf8(&index).with_context(|| format!("{index_url} is not UTF-8"))?;
    let entries = parse_index(xml).with_context(|| format!("reading {index_url}"))?;
    fs::create_dir_all(cache.join(TEXT_DIR))
        .with_context(|| format!("creating {}", cache.join(TEXT_DIR).display()))?;
    write(&cache.join(INDEX_FILE), &index)?;
    let mut corpus = Corpus {
        index_sha256: sha256_hex(&index),
        texts: 0,
        downloaded: 0,
    };
    for entry in entries.iter().filter(|entry| entry.has_text()) {
        corpus.texts += 1;
        let path = text_path(cache, entry.rfc);
        if !refresh && path.is_file() {
            continue;
        }
        let url = format!("{base}rfc{}.txt", entry.rfc);
        let bytes = get(&agent, &url, MAX_SOURCE).with_context(|| format!("RFC {}", entry.rfc))?;
        write(&path, &bytes)?;
        corpus.downloaded += 1;
        downloaded(entry.rfc);
    }
    Ok(corpus)
}

/// Write a file through a sibling `.part` file and a rename, so an interrupted run never leaves a
/// partial file under the final name.
fn write(path: &Path, bytes: &[u8]) -> Result<()> {
    let mut part = path.as_os_str().to_owned();
    part.push(".part");
    let part = PathBuf::from(part);
    fs::write(&part, bytes).with_context(|| format!("writing {}", part.display()))?;
    fs::rename(&part, path).with_context(|| format!("renaming {}", part.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_path_below_a_dot_git_file_is_inside_that_working_tree() {
        let dir = tempfile::tempdir().unwrap();
        let tree = dir.path().join("linked");
        fs::create_dir_all(&tree).unwrap();
        fs::write(tree.join(".git"), "gitdir: elsewhere\n").unwrap();
        let found = git_working_tree(&tree.join("not/yet/created")).unwrap();
        assert_eq!(found, Some(tree.canonicalize().unwrap()));
        assert_eq!(git_working_tree(dir.path()).unwrap(), None);
    }

    #[test]
    fn the_default_cache_is_under_the_cache_home() {
        let cache = default_cache().unwrap();
        assert!(cache.ends_with("rfcspec/bulk"), "{}", cache.display());
    }
}
