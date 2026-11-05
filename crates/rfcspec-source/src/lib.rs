#![forbid(unsafe_code)]
//! S0: fetch an RFC's plain text into a profile's `sources/` and lock it by SHA-256, and read a
//! locked source back, refusing one whose bytes no longer match the lock. This is the only RFCspec
//! crate that reaches the network, and only [`fetch`] and [`corpus::fetch_corpus`] do; [`index`]
//! parses a copy of `rfc-index.xml` and reads nothing else.

pub mod corpus;
pub mod index;

use std::{fs, path::Path, time::Duration};

use anyhow::{Context, Result, bail};
use rfcspec_model::{LOCK_FILE, LOCK_FORMAT, Lock, LockEntry, read_yaml, to_yaml};
use sha2::{Digest, Sha256};

/// Where RFCs are fetched from by default.
pub const RFC_EDITOR: &str = "https://www.rfc-editor.org/rfc/";

/// The largest source accepted, in bytes.
const MAX_SOURCE: u64 = 16 * 1024 * 1024;

/// The lowercase hex SHA-256 of some bytes.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The source file of an RFC, relative to its profile.
pub fn source_file(rfc: u32) -> String {
    format!("sources/rfc{rfc}.txt")
}

/// What [`fetch`] did.
#[derive(Debug, PartialEq, Eq)]
pub struct Fetched {
    pub entry: LockEntry,
    pub source_written: bool,
    pub lock_written: bool,
}

/// Download `rfc<N>.txt` from `base` into `<profile>/sources/` and record it in the lock. A file
/// or lock whose bytes would not change is not rewritten.
pub fn fetch(profile: &Path, rfc: u32, base: &str) -> Result<Fetched> {
    let url = format!("{base}rfc{rfc}.txt");
    let bytes = get(&agent(), &url, MAX_SOURCE)?;
    store(profile, rfc, &bytes)
}

/// An HTTP agent with a 60 s timeout per request.
fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(60)))
        .build()
        .into()
}

/// The body of `GET url`, refused above `limit` bytes; errors name the URL.
fn get(agent: &ureq::Agent, url: &str, limit: u64) -> Result<Vec<u8>> {
    agent
        .get(url)
        .call()
        .with_context(|| format!("fetching {url}"))?
        .body_mut()
        .with_config()
        .limit(limit)
        .read_to_vec()
        .with_context(|| format!("reading {url}"))
}

/// Store fetched bytes as an RFC's source and lock them.
pub fn store(profile: &Path, rfc: u32, bytes: &[u8]) -> Result<Fetched> {
    let file = source_file(rfc);
    let path = profile.join(&file);
    let source_written = fs::read(&path).ok().as_deref() != Some(bytes);
    if source_written {
        fs::create_dir_all(path.parent().expect("a source file has a directory"))?;
        fs::write(&path, bytes).with_context(|| format!("writing {}", path.display()))?;
    }
    let entry = LockEntry {
        rfc,
        file,
        sha256: sha256_hex(bytes),
    };
    let lock_path = profile.join(LOCK_FILE);
    let mut lock = read_lock(profile)?;
    lock.upsert(entry.clone());
    let text = to_yaml(&lock)?;
    let lock_written = fs::read_to_string(&lock_path).ok().as_deref() != Some(text.as_str());
    if lock_written {
        fs::write(&lock_path, text).with_context(|| format!("writing {}", lock_path.display()))?;
    }
    Ok(Fetched {
        entry,
        source_written,
        lock_written,
    })
}

/// A profile's lock; an absent lock is empty.
pub fn read_lock(profile: &Path) -> Result<Lock> {
    let path = profile.join(LOCK_FILE);
    if !path.exists() {
        return Ok(Lock {
            format: LOCK_FORMAT.into(),
            sources: vec![],
        });
    }
    let lock: Lock = read_yaml(&path)?;
    if lock.format != LOCK_FORMAT {
        bail!(
            "{}: format {:?}, expected {LOCK_FORMAT}",
            path.display(),
            lock.format
        );
    }
    Ok(lock)
}

/// A locked source: its text and digest.
#[derive(Debug)]
pub struct Locked {
    pub entry: LockEntry,
    pub text: String,
}

/// Read an RFC's locked source; refuse when its digest differs from the lock, naming both.
pub fn read_locked(profile: &Path, rfc: u32) -> Result<Locked> {
    let lock = read_lock(profile)?;
    let Some(entry) = lock.entry(rfc).cloned() else {
        bail!(
            "RFC {rfc} is not in {}; run `rfcspec fetch`",
            profile.join(LOCK_FILE).display()
        );
    };
    let path = profile.join(&entry.file);
    let bytes = fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let actual = sha256_hex(&bytes);
    if actual != entry.sha256 {
        bail!(
            "{}: SHA-256 {actual} differs from the locked {}",
            entry.file,
            entry.sha256
        );
    }
    let text = String::from_utf8(bytes).with_context(|| format!("{} is not UTF-8", entry.file))?;
    Ok(Locked { entry, text })
}

#[cfg(test)]
mod tests {
    use std::{
        io::{Read, Write},
        net::TcpListener,
        path::PathBuf,
        thread,
    };

    use super::*;

    fn committed() -> Vec<u8> {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../profiles/time/sources/rfc868.txt");
        fs::read(path).unwrap()
    }

    /// An HTTP server on 127.0.0.1 that answers every request with `body`, `requests` times.
    fn serve(body: Vec<u8>, requests: usize) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let address = listener.local_addr().unwrap();
        thread::spawn(move || {
            for stream in listener.incoming().take(requests) {
                let mut stream = stream.unwrap();
                let mut request = Vec::new();
                let mut buffer = [0u8; 1024];
                while !request.ends_with(b"\r\n\r\n") {
                    let read = stream.read(&mut buffer).unwrap();
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                }
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream.write_all(head.as_bytes()).unwrap();
                stream.write_all(&body).unwrap();
            }
        });
        format!("http://{address}/rfc/")
    }

    #[test]
    fn fetch_writes_source_and_lock_entry_and_a_second_run_changes_no_byte() {
        let body = committed();
        let base = serve(body.clone(), 2);
        let profile = tempfile::tempdir().unwrap();

        let first = fetch(profile.path(), 868, &base).unwrap();
        assert!(first.source_written && first.lock_written);
        assert_eq!(first.entry.file, "sources/rfc868.txt");
        assert_eq!(first.entry.sha256, sha256_hex(&body));
        assert_eq!(
            fs::read(profile.path().join("sources/rfc868.txt")).unwrap(),
            body
        );
        let lock_bytes = fs::read(profile.path().join(LOCK_FILE)).unwrap();
        let lock: Lock = read_yaml(&profile.path().join(LOCK_FILE)).unwrap();
        assert_eq!(lock.sources, vec![first.entry.clone()]);

        let second = fetch(profile.path(), 868, &base).unwrap();
        assert!(!second.source_written && !second.lock_written);
        assert_eq!(
            fs::read(profile.path().join("sources/rfc868.txt")).unwrap(),
            body
        );
        assert_eq!(
            fs::read(profile.path().join(LOCK_FILE)).unwrap(),
            lock_bytes
        );
    }

    #[test]
    fn committed_lock_matches_committed_source() {
        let profile = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../profiles/time");
        let locked = read_locked(&profile, 868).unwrap();
        assert_eq!(locked.text.as_bytes(), committed().as_slice());
    }

    #[test]
    fn read_refuses_a_source_whose_digest_differs_from_the_lock_naming_both() {
        let profile = tempfile::tempdir().unwrap();
        let body = committed();
        let fetched = store(profile.path(), 868, &body).unwrap();
        let mut changed = body.clone();
        changed[100] ^= 1;
        fs::write(profile.path().join("sources/rfc868.txt"), &changed).unwrap();

        let error = read_locked(profile.path(), 868).unwrap_err().to_string();
        assert!(error.contains(&fetched.entry.sha256), "{error}");
        assert!(error.contains(&sha256_hex(&changed)), "{error}");
    }
}
