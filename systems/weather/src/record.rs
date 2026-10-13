//! A station's daily record: the CSV a World Pack attaches (`source: record`), its decoding, its gaps,
//! the packed series `weather-configured` carries, and the mapping from a world date to a record date
//! (step-19 §18.3 SD-TW-d-1 … 5; `docs/DECISIONS.md` `DEP-31`).
//!
//! **The file** (SD-TW-d-1). UTF-8, one row per day, ascending, whole calendar years, no duplicate and no
//! skipped day; the header is exactly [`HEADER`]. Integer cells are in GHCN-Daily's own units and may be
//! empty (missing); `fog` and `thunder` are `0` or `1`. This module is the format's one authority:
//! `tools/weather-fetch` writes rows with [`encode`] and the pack reads them with [`decode`].
//!
//! **Decoding is the same on every platform** (SD-TW-d-3). The pack never sees a path, only the bytes the
//! loader read (`ARC-61` note). One leading UTF-8 byte-order mark is dropped; lines are split on `\n` and
//! one trailing `\r` is dropped from each, so LF and CRLF files decode identically; a final newline is
//! optional. Any other `\r`, a tab, a quote or a non-ASCII byte is refused with its 1-based line.
//!
//! **Gaps** (SD-TW-d-2). A day is missing when its maximum, minimum or precipitation is empty. A run of at
//! most [`FILL_RUN_MAX`] missing days copies the previous complete day (`Filled`); a longer run, or one at
//! the very start, is drawn by the rules (`Rule`) under `fill: rules` or refused under `fill: none`,
//! naming its first and last date. A missing wind alone is not a missing day: the month's wind is used.
//!
//! **The packed series** (SD-TW-d-5). Nine bytes a day — `tmax_dc i16`, `tmin_dc i16`, `prcp u16`,
//! `awnd u8`, `wdf2 / 2 u8`, `flags u8`, big-endian — written as one lowercase hex string, so ten years
//! are about 66 KB of JSON and need no codec dependency. Flags: bit 0 fog, bit 1 thunder, bits 2–3 the
//! origin (0 record, 1 filled, 2 rule), bit 4 wind speed missing, bit 5 wind direction missing.

use core::fmt;

use mineworld_calendar::CalendarDate;
use mineworld_calendar::civil::is_leap;
use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// The header of a record file, exactly.
pub const HEADER: &str = "date,tmax_dc,tmin_dc,prcp_tenth_mm,awnd_dms,wdf2_deg,fog,thunder";

/// The longest run of missing days that copies the previous complete day; a longer one is drawn by the
/// rules or refused.
pub const FILL_RUN_MAX: usize = 3;

/// The longest station identifier a record names (provenance only).
pub const STATION_MAX: usize = 32;

/// The temperature bounds of a record cell, 0.1 °C (the rules' own bounds, step-19 §17.4).
const TEMPERATURE_DC: (i64, i64) = (-900, 600);

/// One day of a record file, as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RecordRow {
    /// The day.
    pub date: CalendarDate,
    /// The maximum temperature, 0.1 °C.
    pub tmax_dc: Option<i16>,
    /// The minimum temperature, 0.1 °C.
    pub tmin_dc: Option<i16>,
    /// The precipitation, 0.1 mm.
    pub prcp_tenth_mm: Option<u16>,
    /// The mean wind speed, 0.1 m/s.
    pub awnd_dms: Option<u8>,
    /// The direction of the fastest 2-minute wind, degrees (360 is north).
    pub wdf2_deg: Option<u16>,
    /// Fog that day.
    pub fog: bool,
    /// Thunder that day.
    pub thunder: bool,
}

impl RecordRow {
    /// Whether the day has its maximum, minimum and precipitation.
    pub const fn complete(&self) -> bool {
        self.tmax_dc.is_some() && self.tmin_dc.is_some() && self.prcp_tenth_mm.is_some()
    }
}

/// A decoded record file: whole years, every day once, in order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordFile {
    rows: Vec<RecordRow>,
}

impl RecordFile {
    /// The rows, 1 January of the first year first.
    pub fn rows(&self) -> &[RecordRow] {
        &self.rows
    }

    /// The first year.
    pub fn first_year(&self) -> i32 {
        self.rows[0].date.year()
    }

    /// The last year.
    pub fn last_year(&self) -> i32 {
        self.rows[self.rows.len() - 1].date.year()
    }

    /// How many whole years.
    pub fn years(&self) -> u16 {
        u16::try_from(self.last_year() - self.first_year() + 1)
            .expect("a decoded file holds at most u16::MAX years")
    }
}

/// Why a record file is refused: its 1-based line and what is wrong there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DecodeError {
    /// The line, from 1 (the header).
    pub line: usize,
    /// What is wrong.
    pub message: String,
}

impl fmt::Display for DecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for DecodeError {}

const BOM: &[u8] = b"\xEF\xBB\xBF";

/// The lines of `bytes` with their 1-based numbers: one BOM dropped, split on `\n`, one trailing `\r`
/// dropped from each, the empty line after a final newline ignored; each line refused if it holds a
/// stray `\r`, a tab, a quote or a non-ASCII byte.
fn lines(bytes: &[u8]) -> Result<Vec<(usize, &str)>, DecodeError> {
    let bytes = bytes.strip_prefix(BOM).unwrap_or(bytes);
    let mut parts: Vec<&[u8]> = bytes.split(|byte| *byte == b'\n').collect();
    if parts.len() > 1 && parts.last().is_some_and(|last| last.is_empty()) {
        parts.pop();
    }
    parts
        .into_iter()
        .enumerate()
        .map(|(index, part)| {
            let line = index + 1;
            let part = part.strip_suffix(b"\r").unwrap_or(part);
            let refused = |message: &str| DecodeError {
                line,
                message: message.to_owned(),
            };
            for byte in part {
                match byte {
                    b'\r' => return Err(refused("a carriage return inside the line")),
                    b'\t' => return Err(refused("a tab; cells are separated by commas")),
                    b'"' => return Err(refused("a quoted cell; cells are bare")),
                    0x80..=0xFF => return Err(refused("a non-ASCII byte")),
                    _ => {}
                }
            }
            let text = core::str::from_utf8(part).map_err(|_| refused("not text"))?;
            Ok((line, text))
        })
        .collect()
}

/// A bare integer: an optional `-` and digits, nothing else.
fn integer(text: &str) -> Option<i64> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    if digits.is_empty() || digits.len() > 9 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

/// An optional integer cell in `low ..= high`.
fn cell<T: TryFrom<i64>>(
    column: &str,
    text: &str,
    (low, high): (i64, i64),
    unit: &str,
) -> Result<Option<T>, String> {
    if text.is_empty() {
        return Ok(None);
    }
    let value = integer(text)
        .filter(|value| (low..=high).contains(value))
        .ok_or_else(|| format!("{column} is {unit} from {low} to {high} or empty, not '{text}'"))?;
    Ok(T::try_from(value).ok())
}

/// `0` or `1`.
fn flag(column: &str, text: &str) -> Result<bool, String> {
    match text {
        "0" => Ok(false),
        "1" => Ok(true),
        _ => Err(format!("{column} is 0 or 1, not '{text}'")),
    }
}

/// `YYYY-MM-DD`, a date that exists.
fn date(text: &str) -> Result<CalendarDate, String> {
    let refused = || format!("date is YYYY-MM-DD, a day that exists, not '{text}'");
    let bytes = text.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return Err(refused());
    }
    let number = |range: core::ops::Range<usize>| -> Option<i64> {
        let part = &text[range];
        part.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| part.parse().ok())
            .flatten()
    };
    let (Some(year), Some(month), Some(day)) = (number(0..4), number(5..7), number(8..10)) else {
        return Err(refused());
    };
    CalendarDate::new(
        i32::try_from(year).map_err(|_| refused())?,
        u8::try_from(month).map_err(|_| refused())?,
        u8::try_from(day).map_err(|_| refused())?,
    )
    .ok_or_else(refused)
}

fn row(text: &str) -> Result<RecordRow, String> {
    let cells: Vec<&str> = text.split(',').collect();
    let [d, tmax, tmin, prcp, awnd, wdf2, fog, thunder] = cells.as_slice() else {
        return Err(format!("8 cells, as the header, not {}", cells.len()));
    };
    Ok(RecordRow {
        date: date(d)?,
        tmax_dc: cell("tmax_dc", tmax, TEMPERATURE_DC, "0.1 °C")?,
        tmin_dc: cell("tmin_dc", tmin, TEMPERATURE_DC, "0.1 °C")?,
        prcp_tenth_mm: cell("prcp_tenth_mm", prcp, (0, 65_535), "0.1 mm")?,
        awnd_dms: cell("awnd_dms", awnd, (0, 255), "0.1 m/s")?,
        wdf2_deg: cell("wdf2_deg", wdf2, (0, 360), "degrees")?,
        fog: flag("fog", fog)?,
        thunder: flag("thunder", thunder)?,
    })
}

/// Decodes a record file's bytes (SD-TW-d-1, -3).
///
/// # Errors
///
/// The first line that is not the format, with what is wrong there.
pub fn decode(bytes: &[u8]) -> Result<RecordFile, DecodeError> {
    let lines = lines(bytes)?;
    let refused = |line: usize, message: String| DecodeError { line, message };
    let Some(&(_, header)) = lines.first() else {
        return Err(refused(1, format!("the header is '{HEADER}', not nothing")));
    };
    if header != HEADER {
        return Err(refused(
            1,
            format!("the header is '{HEADER}', not '{header}'"),
        ));
    }
    let mut rows: Vec<RecordRow> = Vec::with_capacity(lines.len());
    for &(line, text) in &lines[1..] {
        let row = row(text).map_err(|message| refused(line, message))?;
        match rows.last() {
            None if (row.date.month(), row.date.day()) != (1, 1) => {
                return Err(refused(
                    line,
                    format!(
                        "the record begins on {}, not on 1 January (whole years only)",
                        show(row.date)
                    ),
                ));
            }
            Some(previous) if row.date <= previous.date => {
                let why = if row.date == previous.date {
                    "repeats the line before"
                } else {
                    "is before the line before (dates ascend)"
                };
                return Err(refused(line, format!("date {} {why}", show(row.date))));
            }
            Some(previous) if Some(row.date) != previous.date.next() => {
                return Err(refused(
                    line,
                    format!(
                        "date {} skips from {} (one row per day)",
                        show(row.date),
                        show(previous.date)
                    ),
                ));
            }
            _ => {}
        }
        rows.push(row);
    }
    let last_line = lines.len();
    let Some(last) = rows.last() else {
        return Err(refused(last_line, "no rows after the header".to_owned()));
    };
    if (last.date.month(), last.date.day()) != (12, 31) {
        return Err(refused(
            last_line,
            format!(
                "the record ends on {}, not on 31 December (whole years only)",
                show(last.date)
            ),
        ));
    }
    let years = i64::from(last.date.year()) - i64::from(rows[0].date.year()) + 1;
    if u16::try_from(years).is_err() {
        return Err(refused(last_line, format!("{years} years is too many")));
    }
    Ok(RecordFile { rows })
}

/// A date as `YYYY-MM-DD`.
pub fn show(date: CalendarDate) -> String {
    format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day())
}

/// Writes rows as a record file: the header, then one LF-terminated line per row. The inverse of
/// [`decode`] for rows that are whole years in order (the tool's writer, SD-TW-d-8b).
pub fn encode(rows: &[RecordRow]) -> String {
    let optional = |value: Option<i64>| value.map(|value| value.to_string()).unwrap_or_default();
    let mut text = format!("{HEADER}\n");
    for row in rows {
        text.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            show(row.date),
            optional(row.tmax_dc.map(i64::from)),
            optional(row.tmin_dc.map(i64::from)),
            optional(row.prcp_tenth_mm.map(i64::from)),
            optional(row.awnd_dms.map(i64::from)),
            optional(row.wdf2_deg.map(i64::from)),
            u8::from(row.fog),
            u8::from(row.thunder),
        ));
    }
    text
}

// ---- the packed series ------------------------------------------------------------------------------

/// Where a record day's values come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DayOrigin {
    /// The record itself.
    Record,
    /// Copied from the last complete day before a short gap.
    Filled,
    /// A long gap: the rules draw it.
    Rule,
}

const FOG: u8 = 1;
const THUNDER: u8 = 1 << 1;
const ORIGIN_SHIFT: u8 = 2;
const ORIGIN_MASK: u8 = 0b11 << ORIGIN_SHIFT;
const AWND_MISSING: u8 = 1 << 4;
const WDF2_MISSING: u8 = 1 << 5;
const FLAGS_KNOWN: u8 = FOG | THUNDER | ORIGIN_MASK | AWND_MISSING | WDF2_MISSING;

/// One day of the packed series: nine bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PackedDay {
    tmax_dc: i16,
    tmin_dc: i16,
    prcp_tenth_mm: u16,
    awnd_dms: u8,
    wdf2_half: u8,
    flags: u8,
}

/// Bytes per packed day.
const PACKED: usize = 9;

impl PackedDay {
    /// A complete row of the record, with the origin `origin` (`Record`, or `Filled` for a copy).
    fn of(row: &RecordRow, origin: DayOrigin) -> Self {
        let mut flags = (u8::from(row.fog) * FOG) | (u8::from(row.thunder) * THUNDER);
        flags |= match origin {
            DayOrigin::Record => 0,
            DayOrigin::Filled => 1,
            DayOrigin::Rule => 2,
        } << ORIGIN_SHIFT;
        if row.awnd_dms.is_none() {
            flags |= AWND_MISSING;
        }
        if row.wdf2_deg.is_none() {
            flags |= WDF2_MISSING;
        }
        // 0 … 360 halves to 0 … 180, which fits a byte; an odd degree rounds down (GHCN-Daily's are
        // multiples of ten).
        let wdf2_half = u8::try_from(row.wdf2_deg.unwrap_or(0) / 2).expect("at most 180");
        Self {
            tmax_dc: row.tmax_dc.unwrap_or(0),
            tmin_dc: row.tmin_dc.unwrap_or(0),
            prcp_tenth_mm: row.prcp_tenth_mm.unwrap_or(0),
            awnd_dms: row.awnd_dms.unwrap_or(0),
            wdf2_half,
            flags,
        }
    }

    /// A day of a long gap: the rules draw it.
    const fn rule() -> Self {
        Self {
            tmax_dc: 0,
            tmin_dc: 0,
            prcp_tenth_mm: 0,
            awnd_dms: 0,
            wdf2_half: 0,
            flags: 2 << ORIGIN_SHIFT,
        }
    }

    /// The same values, as a copy filling a short gap.
    const fn filled(self) -> Self {
        Self {
            flags: (self.flags & !ORIGIN_MASK) | (1 << ORIGIN_SHIFT),
            ..self
        }
    }

    /// Where the values come from.
    pub const fn origin(&self) -> DayOrigin {
        match (self.flags & ORIGIN_MASK) >> ORIGIN_SHIFT {
            0 => DayOrigin::Record,
            1 => DayOrigin::Filled,
            _ => DayOrigin::Rule,
        }
    }

    /// The maximum temperature, 0.1 °C.
    pub const fn tmax_dc(&self) -> i16 {
        self.tmax_dc
    }

    /// The minimum temperature, 0.1 °C.
    pub const fn tmin_dc(&self) -> i16 {
        self.tmin_dc
    }

    /// The precipitation, 0.1 mm.
    pub const fn prcp_tenth_mm(&self) -> u16 {
        self.prcp_tenth_mm
    }

    /// The mean wind speed, 0.1 m/s, if the record has it.
    pub const fn awnd_dms(&self) -> Option<u8> {
        if self.flags & AWND_MISSING == 0 {
            Some(self.awnd_dms)
        } else {
            None
        }
    }

    /// The direction the wind comes from, degrees 0 … 358 (north is 0), if the record has it.
    pub const fn wind_from_deg(&self) -> Option<u16> {
        if self.flags & WDF2_MISSING == 0 {
            Some((self.wdf2_half as u16 * 2) % 360)
        } else {
            None
        }
    }

    /// Fog that day.
    pub const fn fog(&self) -> bool {
        self.flags & FOG != 0
    }

    /// Thunder that day.
    pub const fn thunder(&self) -> bool {
        self.flags & THUNDER != 0
    }

    fn bytes(self) -> [u8; PACKED] {
        let [a, b] = self.tmax_dc.to_be_bytes();
        let [c, d] = self.tmin_dc.to_be_bytes();
        let [e, f] = self.prcp_tenth_mm.to_be_bytes();
        [a, b, c, d, e, f, self.awnd_dms, self.wdf2_half, self.flags]
    }

    fn from_bytes(bytes: [u8; PACKED]) -> Result<Self, String> {
        let day = Self {
            tmax_dc: i16::from_be_bytes([bytes[0], bytes[1]]),
            tmin_dc: i16::from_be_bytes([bytes[2], bytes[3]]),
            prcp_tenth_mm: u16::from_be_bytes([bytes[4], bytes[5]]),
            awnd_dms: bytes[6],
            wdf2_half: bytes[7],
            flags: bytes[8],
        };
        let bounded =
            |value: i16| (TEMPERATURE_DC.0..=TEMPERATURE_DC.1).contains(&i64::from(value));
        if day.flags & !FLAGS_KNOWN != 0
            || (day.flags & ORIGIN_MASK) >> ORIGIN_SHIFT == 3
            || day.wdf2_half > 180
            || !bounded(day.tmax_dc)
            || !bounded(day.tmin_dc)
        {
            return Err(format!("a packed day out of range: {bytes:02x?}"));
        }
        Ok(day)
    }
}

/// Every day of a record, gaps resolved, 1 January of the first year first. In a fact it is one
/// lowercase hex string of nine bytes a day; decoding it checks every day, so a fact can never hold a
/// day the file could not.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct PackedDays(Vec<PackedDay>);

impl PackedDays {
    /// The days.
    pub fn days(&self) -> &[PackedDay] {
        &self.0
    }
}

impl From<PackedDays> for String {
    fn from(days: PackedDays) -> Self {
        const HEX: &[u8; 16] = b"0123456789abcdef";
        let mut text = String::with_capacity(days.0.len() * PACKED * 2);
        for day in days.0 {
            for byte in day.bytes() {
                text.push(char::from(HEX[usize::from(byte >> 4)]));
                text.push(char::from(HEX[usize::from(byte & 0xF)]));
            }
        }
        text
    }
}

impl TryFrom<String> for PackedDays {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let nibble = |c: u8| match c {
            b'0'..=b'9' => Ok(c - b'0'),
            b'a'..=b'f' => Ok(c - b'a' + 10),
            _ => Err("the packed days are lowercase hex".to_owned()),
        };
        let bytes = text.as_bytes();
        if !bytes.len().is_multiple_of(PACKED * 2) {
            return Err(format!(
                "the packed days are {} hex digits a day, not {} in all",
                PACKED * 2,
                bytes.len()
            ));
        }
        bytes
            .chunks(PACKED * 2)
            .map(|chunk| {
                let mut day = [0u8; PACKED];
                for (index, pair) in chunk.chunks(2).enumerate() {
                    day[index] = (nibble(pair[0])? << 4) | nibble(pair[1])?;
                }
                PackedDay::from_bytes(day)
            })
            .collect::<Result<_, _>>()
            .map(Self)
    }
}

/// What to do with a run of more than [`FILL_RUN_MAX`] missing days.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Fill {
    /// The rules draw them.
    #[default]
    Rules,
    /// The configuration is refused.
    None,
}

/// A long gap refused under `fill: none`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GapRefused {
    /// The line of its first day.
    pub line: usize,
    /// Its first day.
    pub first: CalendarDate,
    /// Its last day.
    pub last: CalendarDate,
}

impl fmt::Display for GapRefused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "line {}: {} … {} are missing, more than {FILL_RUN_MAX} days, and `fill` is `none`",
            self.line,
            show(self.first),
            show(self.last)
        )
    }
}

/// The record's days with every gap resolved (SD-TW-d-2).
///
/// # Errors
///
/// A run of missing days longer than [`FILL_RUN_MAX`] (or at the start) under [`Fill::None`].
pub fn pack(file: &RecordFile, fill: Fill) -> Result<PackedDays, GapRefused> {
    let rows = file.rows();
    let mut days = Vec::with_capacity(rows.len());
    let mut last_complete: Option<PackedDay> = None;
    let mut index = 0;
    while index < rows.len() {
        if rows[index].complete() {
            let day = PackedDay::of(&rows[index], DayOrigin::Record);
            last_complete = Some(day);
            days.push(day);
            index += 1;
            continue;
        }
        let end = (index..rows.len())
            .find(|at| rows[*at].complete())
            .unwrap_or(rows.len());
        let run = end - index;
        match last_complete {
            Some(previous) if run <= FILL_RUN_MAX => {
                days.extend(core::iter::repeat_n(previous.filled(), run));
            }
            _ if fill == Fill::None => {
                return Err(GapRefused {
                    line: index + 2,
                    first: rows[index].date,
                    last: rows[end - 1].date,
                });
            }
            _ => days.extend(core::iter::repeat_n(PackedDay::rule(), run)),
        }
        index = end;
    }
    Ok(PackedDays(days))
}

// ---- the series and the date mapping ----------------------------------------------------------------

/// A station identifier: provenance only, 1 … [`STATION_MAX`] printable ASCII characters.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Station(String);

impl TryFrom<String> for Station {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        if text.is_empty()
            || text.len() > STATION_MAX
            || !text.bytes().all(|b| b.is_ascii_graphic())
        {
            return Err(format!(
                "station is 1 to {STATION_MAX} printable ASCII characters, not '{text}'"
            ));
        }
        Ok(Self(text))
    }
}

impl From<Station> for String {
    fn from(station: Station) -> Self {
        station.0
    }
}

impl Station {
    /// The identifier.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The record a world replays, as `weather-configured` states it: the station (provenance), the file's
/// first year, the record year the world's first year replays, the number of years, and every day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawSeries", into = "RawSeries")]
pub struct RecordSeries {
    raw: RawSeries,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawSeries {
    station: Station,
    file_first_year: i32,
    first_year: i32,
    years: u16,
    days: PackedDays,
}

impl TryFrom<RawSeries> for RecordSeries {
    type Error = String;

    fn try_from(raw: RawSeries) -> Result<Self, Self::Error> {
        let first = i64::from(raw.file_first_year);
        let end = first + i64::from(raw.years);
        let expected: i64 = (first..end)
            .map(|year| if is_leap(year) { 366 } else { 365 })
            .sum();
        if raw.years == 0 || i64::try_from(raw.days.0.len()) != Ok(expected) {
            return Err(format!(
                "a record of {} years from {} has {expected} days, not {}",
                raw.years,
                raw.file_first_year,
                raw.days.0.len()
            ));
        }
        if !(first..end).contains(&i64::from(raw.first_year)) {
            return Err(format!(
                "first_year {} is outside the record's years {} … {}",
                raw.first_year,
                first,
                end - 1
            ));
        }
        Ok(Self { raw })
    }
}

impl From<RecordSeries> for RawSeries {
    fn from(series: RecordSeries) -> Self {
        series.raw
    }
}

impl RecordSeries {
    /// The series of `file`, gaps resolved by `fill`, replayed from `first_year`.
    ///
    /// # Errors
    ///
    /// `first_year` outside the file's years, or a long gap under `fill: none`, as text naming the line.
    pub fn new(
        station: Station,
        file: &RecordFile,
        first_year: i32,
        fill: Fill,
    ) -> Result<Self, String> {
        let days = pack(file, fill).map_err(|gap| gap.to_string())?;
        Self::try_from(RawSeries {
            station,
            file_first_year: file.first_year(),
            first_year,
            years: file.years(),
            days,
        })
    }

    /// The station.
    pub fn station(&self) -> &Station {
        &self.raw.station
    }

    /// The file's first year.
    pub const fn file_first_year(&self) -> i32 {
        self.raw.file_first_year
    }

    /// The record year the world's first year replays.
    pub const fn first_year(&self) -> i32 {
        self.raw.first_year
    }

    /// The number of whole years.
    pub const fn years(&self) -> u16 {
        self.raw.years
    }

    /// Every day.
    pub fn days(&self) -> &[PackedDay] {
        self.raw.days.days()
    }

    /// The record date world date `world` replays, in a world whose first day fell in `epoch_year`
    /// (SD-TW-d-4): `file_first + ((first_year − file_first) + (world year − epoch_year)) mod years`,
    /// month and day kept; a world 29 February in a non-leap record year takes 28 February.
    pub fn record_date(&self, world: CalendarDate, epoch_year: i32) -> CalendarDate {
        let file_first = i64::from(self.raw.file_first_year);
        let offset = (i64::from(self.raw.first_year) - file_first)
            + (i64::from(world.year()) - i64::from(epoch_year));
        let year = file_first + offset.rem_euclid(i64::from(self.raw.years));
        let day = if world.month() == 2 && world.day() == 29 && !is_leap(year) {
            28
        } else {
            world.day()
        };
        CalendarDate::new(
            i32::try_from(year).expect("within the record's years"),
            world.month(),
            day,
        )
        .expect("month and day exist in the record year")
    }

    /// The record date `world` replays and that day's values.
    pub fn day(&self, world: CalendarDate, epoch_year: i32) -> (CalendarDate, PackedDay) {
        let date = self.record_date(world, epoch_year);
        let start = CalendarDate::new(self.raw.file_first_year, 1, 1).expect("1 January exists");
        let index = usize::try_from(date.days() - start.days()).expect("within the record");
        (date, self.days()[index])
    }
}
