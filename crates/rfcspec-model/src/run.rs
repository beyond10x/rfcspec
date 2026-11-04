//! A recorded conformance run (`rfcspec-run/1`): what `rfcspec verify --report` writes under a
//! profile's `runs/`, and what the protocol catalogue reads as the profile's conformance verdict.
//! The same target, time, version, overlay and rows always serialize to the same bytes.

use std::{fmt, fs, path::Path};

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// The report format.
pub const RUN_FORMAT: &str = "rfcspec-run/1";
/// The directory of a profile that holds its recorded runs.
pub const RUNS_DIR: &str = "runs";

/// A row's verdict (design section 11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Pass,
    Fail,
    Inconclusive,
    OutOfScope,
}

impl fmt::Display for Verdict {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::Inconclusive => "inconclusive",
            Verdict::OutOfScope => "out of scope",
        })
    }
}

/// One row of the requirement matrix.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRow {
    pub requirement: String,
    pub level: String,
    pub verdict: Verdict,
    pub detail: String,
}

/// One recorded run of `rfcspec verify` against a target.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunReport {
    pub format: String,
    /// The target as given on the command line.
    pub target: String,
    /// When the run happened, UTC, `YYYY-MM-DDTHH:MM:SSZ`.
    pub time: String,
    /// The rfcspec version that ran it.
    pub rfcspec: String,
    /// The SHA-256 of the profile's `overlay.yaml` the run was judged against.
    pub overlay_sha256: String,
    /// The overall verdict by the design section 11 rule.
    pub verdict: Verdict,
    pub rows: Vec<RunRow>,
}

impl RunReport {
    /// The report of a run at `unix` seconds against `target`, judged on the overlay `overlay`.
    pub fn new(target: &str, unix: u64, version: &str, overlay: &[u8], rows: Vec<RunRow>) -> Self {
        Self {
            format: RUN_FORMAT.into(),
            target: target.into(),
            time: utc(unix),
            rfcspec: version.into(),
            overlay_sha256: sha256_hex(overlay),
            verdict: overall(&rows),
            rows,
        }
    }

    /// The day of the run, `YYYY-MM-DD`.
    pub fn date(&self) -> &str {
        self.time.get(..10).unwrap_or(&self.time)
    }

    /// The report as pretty JSON in field order, with a final newline.
    pub fn to_json(&self) -> Result<String> {
        crate::to_json(self)
    }

    /// Parse a report and refuse one of another format or with a malformed time.
    pub fn parse(text: &str) -> Result<Self> {
        let report: Self = serde_json::from_str(text)?;
        if report.format != RUN_FORMAT {
            bail!("format {:?}, expected {RUN_FORMAT}", report.format);
        }
        if !is_utc(&report.time) {
            bail!("time {:?} is not YYYY-MM-DDTHH:MM:SSZ", report.time);
        }
        Ok(report)
    }
}

/// Whether a level is MUST-level: the claim "X conforms to P" turns on these rows.
pub fn is_must_level(level: &str) -> bool {
    matches!(
        level,
        "MUST" | "MUST NOT" | "REQUIRED" | "SHALL" | "SHALL NOT"
    )
}

/// The overall verdict (design section 11): fail when a MUST-level row fails, inconclusive when
/// none fails but one is inconclusive, else pass.
pub fn overall(rows: &[RunRow]) -> Verdict {
    let must = || rows.iter().filter(|row| is_must_level(&row.level));
    if must().any(|row| row.verdict == Verdict::Fail) {
        Verdict::Fail
    } else if must().any(|row| row.verdict == Verdict::Inconclusive) {
        Verdict::Inconclusive
    } else {
        Verdict::Pass
    }
}

/// The SHA-256 of `bytes`, lowercase hex.
pub fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

/// The file stem a report of `target` is written under: every run of characters other than ASCII
/// letters, digits and dots becomes one `-`, so `tcp://127.0.0.1:37` is `tcp-127.0.0.1-37`.
pub fn target_slug(target: &str) -> String {
    let mut slug = String::new();
    for c in target.chars() {
        if c.is_ascii_alphanumeric() || c == '.' {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    slug.trim_end_matches('-').to_owned()
}

/// `unix` seconds as UTC `YYYY-MM-DDTHH:MM:SSZ`.
pub fn utc(unix: u64) -> String {
    let days = i64::try_from(unix / 86_400).unwrap_or(i64::MAX);
    let seconds = unix % 86_400;
    // Days to a civil date (Howard Hinnant's algorithm), proleptic Gregorian.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

fn is_utc(time: &str) -> bool {
    let b = time.as_bytes();
    b.len() == 20
        && b.iter().enumerate().all(|(i, c)| match i {
            4 | 7 => *c == b'-',
            10 => *c == b'T',
            13 | 16 => *c == b':',
            19 => *c == b'Z',
            _ => c.is_ascii_digit(),
        })
}

/// Every report under `profile/runs/`, by file name: `(file name, report)`. A file that does not
/// parse is an error naming it.
pub fn read_runs(profile: &Path) -> Result<Vec<(String, RunReport)>> {
    let dir = profile.join(RUNS_DIR);
    if !dir.is_dir() {
        return Ok(Vec::new());
    }
    let mut names: Vec<String> = fs::read_dir(&dir)
        .with_context(|| format!("reading {}", dir.display()))?
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_file())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.ends_with(".json"))
        .collect();
    names.sort();
    names
        .into_iter()
        .map(|name| {
            let path = dir.join(&name);
            let text =
                fs::read_to_string(&path).with_context(|| format!("reading {}", path.display()))?;
            let report = RunReport::parse(&text)
                .with_context(|| format!("{} does not parse", path.display()))?;
            Ok((name, report))
        })
        .collect()
}

/// The newest report by run time; on equal times, the last by file name.
pub fn newest(runs: &[(String, RunReport)]) -> Option<&(String, RunReport)> {
    runs.iter()
        .max_by(|a, b| a.1.time.cmp(&b.1.time).then(a.0.cmp(&b.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(level: &str, verdict: Verdict) -> RunRow {
        RunRow {
            requirement: "R".into(),
            level: level.into(),
            verdict,
            detail: String::new(),
        }
    }

    #[test]
    fn the_slug_keeps_letters_digits_and_dots() {
        assert_eq!(target_slug("tcp://127.0.0.1:37"), "tcp-127.0.0.1-37");
        assert_eq!(target_slug("tcp://time.example:37/"), "tcp-time.example-37");
        assert_eq!(target_slug("tcp://[::1]:37"), "tcp-1-37");
    }

    #[test]
    fn utc_formats_a_civil_date() {
        assert_eq!(utc(0), "1970-01-01T00:00:00Z");
        assert_eq!(utc(951_782_400), "2000-02-29T00:00:00Z");
        assert_eq!(utc(1_793_793_600), "2026-11-04T12:00:00Z");
        assert!(is_utc(&utc(1_793_793_600)));
        assert!(!is_utc("2026-11-04"));
    }

    #[test]
    fn the_overall_verdict_turns_on_must_level_rows() {
        assert_eq!(overall(&[row("MUST", Verdict::Pass)]), Verdict::Pass);
        assert_eq!(
            overall(&[row("MUST", Verdict::Pass), row("SHOULD", Verdict::Fail)]),
            Verdict::Pass
        );
        assert_eq!(
            overall(&[
                row("MUST", Verdict::Inconclusive),
                row("MUST", Verdict::Pass)
            ]),
            Verdict::Inconclusive
        );
        assert_eq!(
            overall(&[
                row("MUST", Verdict::Inconclusive),
                row("SHALL", Verdict::Fail)
            ]),
            Verdict::Fail
        );
        assert_eq!(overall(&[row("MUST", Verdict::OutOfScope)]), Verdict::Pass);
    }

    #[test]
    fn the_same_run_writes_the_same_bytes_and_parses_back() {
        let rows = vec![
            row("MUST", Verdict::Pass),
            row("SHOULD", Verdict::OutOfScope),
        ];
        let a = RunReport::new(
            "tcp://127.0.0.1:37",
            1_793_793_600,
            "0.0.0",
            b"x",
            rows.clone(),
        );
        let b = RunReport::new("tcp://127.0.0.1:37", 1_793_793_600, "0.0.0", b"x", rows);
        let text = a.to_json().unwrap();
        assert_eq!(text, b.to_json().unwrap());
        assert!(text.ends_with("}\n"));
        let keys: Vec<usize> = [
            "\"format\"",
            "\"target\"",
            "\"time\"",
            "\"rfcspec\"",
            "\"overlay_sha256\"",
            "\"verdict\"",
            "\"rows\"",
        ]
        .iter()
        .map(|key| text.find(key).unwrap())
        .collect();
        assert!(keys.windows(2).all(|w| w[0] < w[1]), "{text}");
        assert!(text.contains("\"verdict\": \"out-of-scope\""), "{text}");
        assert_eq!(RunReport::parse(&text).unwrap(), a);
        assert_eq!(a.date(), "2026-11-04");
        assert!(RunReport::parse(&text.replace("rfcspec-run/1", "rfcspec-run/2")).is_err());
        assert!(RunReport::parse("{}").is_err());
    }
}
