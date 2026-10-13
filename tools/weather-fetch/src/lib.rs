//! `weather-fetch`: a developer tool that makes a World Pack's weather record from NOAA's data
//! (`docs/DECISIONS.md` `DEP-31`; design `.structured-coding/plans/mvp0/step-19-time-weather.md` §18).
//!
//! ```text
//! reshape  --input FILE.dly --station ID --from YYYY --to YYYY --retrieved YYYY-MM-DD --out-dir DIR
//!          [--name NAME]
//!          a GHCN-Daily station file → NAME.csv (the weather pack's record format) and NOTICE, and a
//!          report of missing values, gaps and weather-type population; byte-reproducible
//! fit      --input FILE.csv --base configure/weather.yaml --out FILE.yaml
//!          [--station ID --data data/… --first-year YYYY [--fill rules|none]]
//!          the rules (WGEN-lite, integers) estimated from a record; byte-reproducible
//! fetch    --station ID --out FILE.dly          only with `--features fetch` (off by default)
//!          downloads NOAA's file verbatim and prints the URL, the UTC date and the byte count
//! ```
//!
//! The record's CSV and the rules are the `weather` pack's own types: the tool writes what the pack
//! decodes. Nothing a world runs depends on this crate — `run`, the server and the clients never open a
//! network connection for weather (`INV-TW-7`) — and its HTTP client exists only with `fetch`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod dly;
#[cfg(feature = "fetch")]
pub mod fetch;
pub mod fit;
pub mod notice;
pub mod reshape;

use std::path::Path;

use mineworld_weather::record::{decode, encode};

/// What `reshape` wrote, and its report.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Written {
    /// The CSV's text.
    pub csv: String,
    /// The NOTICE's text.
    pub notice: String,
    /// The report.
    pub report: String,
}

/// `reshape` without the file system: `dly` (the station file's bytes) into the CSV, the NOTICE and the
/// report. `command` is recorded in the NOTICE as given.
///
/// # Errors
///
/// A malformed `.dly` line (with its number), a station mismatch, years outside the file, or a CSV the
/// pack would not decode (a defect).
pub fn reshape_bytes(
    dly: &[u8],
    station: &str,
    (from, to): (i32, i32),
    retrieved: &str,
    name: &str,
    command: &str,
) -> Result<Written, String> {
    let text = core::str::from_utf8(dly).map_err(|_| "the .dly file is not text".to_owned())?;
    let parsed = dly::parse(text)?;
    let reshaped = reshape::reshape(&parsed, station, from, to)?;
    let csv = encode(&reshaped.rows);
    decode(csv.as_bytes())
        .map_err(|error| format!("the CSV does not decode (a defect): {error}"))?;
    let csv_name = format!("{name}.csv");
    let notice = notice::notice(&notice::Provenance {
        station,
        csv_name: &csv_name,
        years: (from, to),
        retrieved,
        command,
        dly_bytes: dly.len(),
        csv_bytes: csv.len(),
        csv_lines: csv.lines().count(),
    });
    Ok(Written {
        csv,
        notice,
        report: reshaped.report,
    })
}

/// Writes `written` into `directory` as `name.csv` and `NOTICE`.
///
/// # Errors
///
/// A write failing, naming the file.
pub fn write(directory: &Path, name: &str, written: &Written) -> Result<(), String> {
    std::fs::create_dir_all(directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?;
    for (file, text) in [
        (format!("{name}.csv"), &written.csv),
        ("NOTICE".to_owned(), &written.notice),
    ] {
        let path = directory.join(file);
        std::fs::write(&path, text).map_err(|error| format!("{}: {error}", path.display()))?;
    }
    Ok(())
}

/// A `YYYY-MM-DD` date argument, checked.
///
/// # Errors
///
/// Anything else.
pub fn date_argument(text: &str) -> Result<String, String> {
    let ok = text.len() == 10
        && text.bytes().enumerate().all(|(index, b)| {
            if index == 4 || index == 7 {
                b == b'-'
            } else {
                b.is_ascii_digit()
            }
        });
    ok.then(|| text.to_owned())
        .ok_or_else(|| format!("'{text}' is not YYYY-MM-DD"))
}
