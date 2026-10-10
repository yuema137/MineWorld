//! NOAA GHCN-Daily's `.dly` station file, by the fixed columns of its `readme.txt` §III
//! (<https://www.ncei.noaa.gov/pub/data/ghcn/daily/readme.txt>, read 2026-10-09):
//!
//! ```text
//! ID 1–11 · YEAR 12–15 · MONTH 16–17 · ELEMENT 18–21 · then 31 times: VALUE (5) MFLAG (1) QFLAG (1)
//! SFLAG (1), from column 22; VALUE31 is 262–269. Missing is −9999. A blank QFLAG passed every check.
//! ```

use std::collections::BTreeMap;

/// A `.dly` line's length.
pub const LINE: usize = 269;

/// The missing value.
pub const MISSING: i32 = -9_999;

/// One day's value of an element, with its quality flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Value {
    /// The value in the element's unit, or [`MISSING`].
    pub value: i32,
    /// The quality flag; `b' '` passed every check.
    pub qflag: u8,
}

impl Value {
    /// The value, if present and unflagged.
    pub const fn usable(self) -> Option<i32> {
        if self.value != MISSING && self.qflag == b' ' {
            Some(self.value)
        } else {
            None
        }
    }

    /// Present but quality-flagged: dropped.
    pub const fn flagged(self) -> bool {
        self.value != MISSING && self.qflag != b' '
    }
}

/// A station's file: its id, and each element's month of values by `(year, month, element)`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Dly {
    /// The station.
    pub station: String,
    /// Every line's values.
    pub months: BTreeMap<(i32, u8, String), [Value; 31]>,
}

impl Dly {
    /// The years the file has any line for.
    pub fn years(&self) -> Option<(i32, i32)> {
        let first = self.months.keys().next()?.0;
        let last = self.months.keys().next_back()?.0;
        Some((first, last))
    }

    /// The value of `element` on `day` (1 … 31) of `year`-`month`, if the file has that month.
    pub fn value(&self, year: i32, month: u8, element: &str, day: u8) -> Option<Value> {
        self.months
            .get(&(year, month, element.to_owned()))
            .map(|values| values[usize::from(day) - 1])
    }
}

/// Parses a `.dly` file. Lines may end LF or CRLF; a final newline is optional.
///
/// # Errors
///
/// The first malformed line, by its 1-based number: wrong length, a field that is not a number, a
/// station other than the first line's, or a line repeated.
pub fn parse(text: &str) -> Result<Dly, String> {
    let mut dly = Dly::default();
    let text = text.strip_suffix('\n').unwrap_or(text);
    for (index, raw) in text.split('\n').enumerate() {
        let line = raw.strip_suffix('\r').unwrap_or(raw);
        let number = index + 1;
        let refused = |why: String| format!("line {number}: {why}");
        if line.len() != LINE || !line.is_ascii() {
            return Err(refused(format!(
                "a .dly line is {LINE} ASCII characters, not {}",
                line.len()
            )));
        }
        let field = |from: usize, to: usize| &line[from - 1..to];
        let integer = |from: usize, to: usize, what: &str| -> Result<i32, String> {
            field(from, to)
                .trim()
                .parse::<i32>()
                .map_err(|_| refused(format!("{what} '{}' is not a number", field(from, to))))
        };
        let station = field(1, 11).to_owned();
        if dly.station.is_empty() {
            dly.station.clone_from(&station);
        } else if station != dly.station {
            return Err(refused(format!(
                "station {station}, but the file began with {}",
                dly.station
            )));
        }
        let year = integer(12, 15, "YEAR")?;
        let month = u8::try_from(integer(16, 17, "MONTH")?)
            .ok()
            .filter(|month| (1..=12).contains(month))
            .ok_or_else(|| refused(format!("MONTH '{}' is not 1 … 12", field(16, 17))))?;
        let element = field(18, 21).to_owned();
        let mut values = [Value {
            value: MISSING,
            qflag: b' ',
        }; 31];
        for (day, slot) in values.iter_mut().enumerate() {
            let start = 22 + day * 8;
            *slot = Value {
                value: integer(start, start + 4, "VALUE")?,
                qflag: line.as_bytes()[start + 5],
            };
        }
        if dly
            .months
            .insert((year, month, element.clone()), values)
            .is_some()
        {
            return Err(refused(format!(
                "{year}-{month:02} {element} appears twice"
            )));
        }
    }
    Ok(dly)
}
