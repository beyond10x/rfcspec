#![forbid(unsafe_code)]
//! S7 for the Time Protocol: probe a live TCP time server and give every requirement of the
//! profile one row with a verdict of design section 11. The probe connects, writes nothing, reads
//! until the server closes or the read timeout passes, and decodes what it read with the generated
//! codec. The expected value comes from the local clock, never from the generated code.

use std::{
    fmt,
    io::{ErrorKind, Read},
    net::{TcpStream, ToSocketAddrs},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result, bail};
use rfcspec_model::{Overlay, Requirement};

/// Seconds from 00:00 1 January 1900 GMT to the Unix epoch: RFC 868's own first example,
/// "the time 2,208,988,800 corresponds to 00:00 1 Jan 1970 GMT".
const SECONDS_1900_TO_1970: u64 = 2_208_988_800;

/// A row's verdict.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub requirement: String,
    pub level: String,
    pub verdict: Verdict,
    pub detail: String,
}

/// How the probe judges what it observes.
#[derive(Clone, Copy, Debug)]
pub struct Options {
    /// The largest accepted difference between the server's time and the local clock.
    pub tolerance: Duration,
    /// How long the probe waits for the server to send and close.
    pub timeout: Duration,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            tolerance: Duration::from_secs(5),
            timeout: Duration::from_secs(10),
        }
    }
}

/// What the probe saw.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Observation {
    pub octets: Vec<u8>,
    /// The server closed the connection before the timeout.
    pub closed: bool,
    /// The local clock when the probe stopped reading, as seconds since 1900 (mod 2^32).
    pub local_1900: u32,
}

/// The `HOST:PORT` of a `tcp://HOST:PORT` target.
pub fn tcp_address(target: &str) -> Result<&str> {
    match target.split_once("://") {
        Some(("tcp", address)) if !address.is_empty() => Ok(address),
        Some((scheme, _)) => bail!("target {target}: only tcp:// is verified, not {scheme}://"),
        None => bail!("target {target}: expected tcp://HOST:PORT"),
    }
}

/// Connect, write nothing, and read until the server closes or `timeout` passes.
pub fn probe(address: &str, timeout: Duration) -> Result<Observation> {
    let socket = address
        .to_socket_addrs()
        .with_context(|| format!("resolving {address}"))?
        .next()
        .with_context(|| format!("{address} resolves to no address"))?;
    let mut stream = TcpStream::connect_timeout(&socket, timeout)
        .with_context(|| format!("connecting to {address}"))?;
    let deadline = Instant::now() + timeout;
    let mut octets = Vec::new();
    let mut buffer = [0u8; 64];
    let closed = loop {
        let left = deadline.saturating_duration_since(Instant::now());
        if left.is_zero() {
            break false;
        }
        stream.set_read_timeout(Some(left))?;
        match stream.read(&mut buffer) {
            Ok(0) => break true,
            Ok(read) => octets.extend_from_slice(&buffer[..read]),
            Err(error) if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {
                break false;
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => return Err(error).with_context(|| format!("reading from {address}")),
        }
    };
    Ok(Observation {
        octets,
        closed,
        local_1900: local_1900(),
    })
}

/// The local clock as RFC 868 time, seconds since 1900 modulo 2^32.
pub fn local_1900() -> u32 {
    let unix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| since.as_secs());
    ((unix + SECONDS_1900_TO_1970) & u64::from(u32::MAX)) as u32
}

/// One row per requirement of the overlay, in its order, judged on an observation of a TCP target.
pub fn matrix(overlay: &Overlay, observation: &Observation, options: Options) -> Result<Vec<Row>> {
    overlay
        .requirements
        .iter()
        .map(|requirement| {
            let (verdict, detail) = judge(requirement, observation, options)?;
            Ok(Row {
                requirement: requirement.id.clone(),
                level: requirement.level.clone(),
                verdict,
                detail,
            })
        })
        .collect()
}

fn judge(
    requirement: &Requirement,
    seen: &Observation,
    options: Options,
) -> Result<(Verdict, String)> {
    if !requirement.transports.is_empty() && !requirement.transports.iter().any(|t| t == "tcp") {
        return Ok((
            Verdict::OutOfScope,
            format!(
                "applies to {}; the target is TCP",
                requirement.transports.join(", ")
            ),
        ));
    }
    let count = seen.octets.len();
    Ok(match requirement.probe.as_deref() {
        None => (
            Verdict::Inconclusive,
            "not observable from a healthy server".into(),
        ),
        Some("tcp-octets") if count == rfc868_time::OCTETS => {
            (Verdict::Pass, format!("received {count} octets"))
        }
        Some("tcp-octets") => (
            Verdict::Fail,
            format!("received {count} octets, expected {}", rfc868_time::OCTETS),
        ),
        Some("tcp-value") => match rfc868_time::parse(&seen.octets) {
            Err(error) => (
                Verdict::Inconclusive,
                format!("no value to compare: {error}"),
            ),
            Ok(time) => {
                let offset = i64::from(time.0.wrapping_sub(seen.local_1900) as i32);
                let within = offset.unsigned_abs() <= options.tolerance.as_secs();
                (
                    if within { Verdict::Pass } else { Verdict::Fail },
                    format!(
                        "server time {} is {offset:+} s from the local clock (tolerance {} s)",
                        time.0,
                        options.tolerance.as_secs()
                    ),
                )
            }
        },
        Some("tcp-close") if !seen.closed => (
            Verdict::Fail,
            format!(
                "connection still open {} s after connecting",
                options.timeout.as_secs_f64()
            ),
        ),
        Some("tcp-close") if count == 0 => (
            Verdict::Inconclusive,
            "the server closed without sending".into(),
        ),
        Some("tcp-close") => (
            Verdict::Pass,
            format!("server closed after sending {count} octets"),
        ),
        Some(other) => bail!("requirement {}: unknown probe {other}", requirement.id),
    })
}

/// Probe `tcp://HOST:PORT` and judge every requirement of the overlay.
pub fn verify(overlay: &Overlay, target: &str, options: Options) -> Result<Vec<Row>> {
    let observation = probe(tcp_address(target)?, options.timeout)?;
    matrix(overlay, &observation, options)
}

/// 1 when a row fails, else 0.
pub fn exit_code(rows: &[Row]) -> u8 {
    u8::from(rows.iter().any(|row| row.verdict == Verdict::Fail))
}

/// The matrix as a fixed-width text table.
pub fn render(rows: &[Row]) -> String {
    let width = rows
        .iter()
        .map(|row| row.requirement.len())
        .max()
        .unwrap_or(0)
        .max("requirement".len());
    let mut out = format!(
        "{:width$}  {:6}  {:12}  detail\n",
        "requirement", "level", "verdict"
    );
    for row in rows {
        out.push_str(&format!(
            "{:width$}  {:6}  {:12}  {}\n",
            row.requirement,
            row.level,
            row.verdict.to_string(),
            row.detail
        ));
    }
    out
}
