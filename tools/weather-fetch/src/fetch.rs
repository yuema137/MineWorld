//! `fetch` (feature `fetch` only): downloads a station's `.dly` file once, verbatim (step-19
//! SD-TW-d-8a). It never retries: a failure is reported, and the file can be downloaded by hand from the
//! URL it names and given to `reshape`.

use std::time::{SystemTime, UNIX_EPOCH};

use mineworld_weather::CalendarDate;
use mineworld_weather::record::show;

use crate::notice::SOURCE;

/// The largest file accepted: NOAA's longest station records are a few megabytes.
const LIMIT: u64 = 64 * 1024 * 1024;

/// What a fetch did, for the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fetched {
    /// The URL read.
    pub url: String,
    /// The UTC date of the retrieval, `YYYY-MM-DD`.
    pub retrieved: String,
    /// The bytes written.
    pub bytes: usize,
}

/// The station file's URL.
pub fn url(station: &str) -> String {
    format!("{SOURCE}{station}.dly")
}

/// Downloads `station`'s file into `out`.
///
/// # Errors
///
/// The request or the write failing, with the URL; nothing is retried.
pub fn fetch(station: &str, out: &std::path::Path) -> Result<Fetched, String> {
    let url = url(station);
    let refused =
        |why: String| format!("{url}: {why} (no retry; download it by hand and run `reshape`)");
    let mut response = ureq::get(&url)
        .call()
        .map_err(|error| refused(error.to_string()))?;
    let bytes = response
        .body_mut()
        .with_config()
        .limit(LIMIT)
        .read_to_vec()
        .map_err(|error| refused(error.to_string()))?;
    std::fs::write(out, &bytes).map_err(|error| format!("{}: {error}", out.display()))?;
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| error.to_string())?
        .as_secs();
    let days = i64::try_from(seconds / 86_400).map_err(|error| error.to_string())?;
    let retrieved = CalendarDate::from_days(days)
        .map(show)
        .ok_or("a date outside the calendar")?;
    Ok(Fetched {
        url,
        retrieved,
        bytes: bytes.len(),
    })
}
