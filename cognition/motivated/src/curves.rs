//! The shipped curve set `atus-a3-2025`: the committed ATUS Table A-3 rows, parsed and checked, and derived
//! into curves by a pure function (`pr-dl-b-motivated.md` SD-DLb-10 … SD-DLb-12, I-DLb-5).
//!
//! The table text is compiled in with `include_str!`; the run never opens a file (I-DLb-5). Derivation
//! happens once, when parameters are decoded.

use core::fmt;

use crate::curve::{Curve, SCALE};

/// The edition this set is named after (SD-DLb-10).
pub const EDITION: &str = "atus-a3-2025";

/// The committed table: six rows, per-mille integers, with its checksum and zero markers (SD-DLb-11).
pub const TABLE: &str = include_str!("../data/atus-a3-2025.tsv");

/// The six activity rows, by the names the TSV gives them.
const SLEEPING: &str = "sleeping";
const EATING: &str = "eating-and-drinking";
const WORKING: &str = "working";
const SOCIALIZING: &str = "socializing-and-communicating";
const LEISURE: &str = "leisure-and-sports";
const PURCHASING: &str = "purchasing-goods-and-services";
const ROWS: [&str; 6] = [SLEEPING, EATING, WORKING, SOCIALIZING, LEISURE, PURCHASING];

/// Clock hours in a day: each row has one value per hour.
const HOURS: usize = 24;

/// The parsed table: the six rows as per-mille values by clock hour (`0` is 00:00 … `23` is 23:00).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Table {
    sleeping: [u32; HOURS],
    eating: [u32; HOURS],
    working: [u32; HOURS],
    socializing: [u32; HOURS],
    leisure: [u32; HOURS],
    purchasing: [u32; HOURS],
}

/// Why the table text is not a valid table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableError {
    /// A line whose first field names no known row, `source`, `zero_markers` or `checksum`.
    UnknownLine {
        /// The one-based line.
        line: usize,
        /// The first field of the line.
        key: String,
    },
    /// A row with a value count other than 24, or a value that is not an integer or is above 1 000.
    BadValue {
        /// The one-based line.
        line: usize,
        /// What is wrong with it.
        reason: String,
    },
    /// A `zero_markers` entry that is malformed, or that names a cell whose value is not zero.
    BadZeroMarker {
        /// The one-based line.
        line: usize,
        /// What is wrong with it.
        reason: String,
    },
    /// A row that appears twice, or a row the table does not contain.
    Row {
        /// The row's name.
        row: String,
        /// What is wrong with it.
        reason: &'static str,
    },
    /// A checksum that does not equal the row's sum over the 24 hours.
    Checksum {
        /// The row's name.
        row: String,
        /// The sum the table states.
        stated: u32,
        /// The sum of the row's values.
        computed: u32,
    },
}

impl fmt::Display for TableError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnknownLine { line, key } => {
                write!(formatter, "line {line}: '{key}' is not a row of the table")
            }
            Self::BadValue { line, reason } => write!(formatter, "line {line}: {reason}"),
            Self::BadZeroMarker { line, reason } => write!(formatter, "line {line}: {reason}"),
            Self::Row { row, reason } => write!(formatter, "row '{row}': {reason}"),
            Self::Checksum {
                row,
                stated,
                computed,
            } => write!(
                formatter,
                "row '{row}': checksum says {stated}, the values sum to {computed}"
            ),
        }
    }
}

impl Table {
    /// Parses the committed table text (or any text in its shape).
    ///
    /// # Errors
    ///
    /// A line naming no known row, a row without 24 integer values of at most 1 000, a zero marker that
    /// names a cell that is not zero, a row given twice or missing, or a checksum that does not equal the
    /// row's sum.
    pub fn parse(text: &str) -> Result<Self, TableError> {
        let mut values: [Option<[u32; HOURS]>; 6] = [None; 6];
        let mut zeros: Vec<(usize, usize, usize)> = Vec::new();
        let mut checksums: Vec<(String, u32)> = Vec::new();
        for (index, text_line) in text.lines().enumerate() {
            let line = index + 1;
            if text_line.trim().is_empty() {
                continue;
            }
            let mut fields = text_line.split('\t');
            let key = fields.next().unwrap_or_default();
            match key {
                "source" => {}
                "zero_markers" => {
                    for entry in fields {
                        let (row, hours) =
                            entry.split_once(':').ok_or(TableError::BadZeroMarker {
                                line,
                                reason: format!("'{entry}' is not row:hours"),
                            })?;
                        let row_index =
                            row_index(row).ok_or_else(|| TableError::BadZeroMarker {
                                line,
                                reason: format!("'{row}' is not a row"),
                            })?;
                        for hour in hours.split(',') {
                            let hour: usize =
                                hour.parse().map_err(|_| TableError::BadZeroMarker {
                                    line,
                                    reason: format!("'{hour}' is not an hour"),
                                })?;
                            if hour >= HOURS {
                                return Err(TableError::BadZeroMarker {
                                    line,
                                    reason: format!("hour {hour} is outside 0 to 23"),
                                });
                            }
                            zeros.push((line, row_index, hour));
                        }
                    }
                }
                "checksum" => {
                    for entry in fields {
                        let (row, stated) = entry.split_once('=').ok_or(TableError::Row {
                            row: entry.to_owned(),
                            reason: "a checksum entry is row=sum",
                        })?;
                        let stated: u32 = stated.parse().map_err(|_| TableError::Row {
                            row: row.to_owned(),
                            reason: "the checksum is not an integer",
                        })?;
                        checksums.push((row.to_owned(), stated));
                    }
                }
                row if row_index(row).is_some() => {
                    let row_index = row_index(row).unwrap_or_default();
                    if values[row_index].is_some() {
                        return Err(TableError::Row {
                            row: row.to_owned(),
                            reason: "appears twice",
                        });
                    }
                    let mut hours = [0_u32; HOURS];
                    let mut count = 0;
                    for (position, field) in fields.enumerate() {
                        let value: u32 = field.parse().map_err(|_| TableError::BadValue {
                            line,
                            reason: format!("'{field}' is not an integer"),
                        })?;
                        if value > SCALE {
                            return Err(TableError::BadValue {
                                line,
                                reason: format!("{value} is above {SCALE}"),
                            });
                        }
                        if position < HOURS {
                            hours[position] = value;
                        }
                        count += 1;
                    }
                    if count != HOURS {
                        return Err(TableError::BadValue {
                            line,
                            reason: format!("{count} values, not {HOURS}"),
                        });
                    }
                    values[row_index] = Some(hours);
                }
                other => {
                    return Err(TableError::UnknownLine {
                        line,
                        key: other.to_owned(),
                    });
                }
            }
        }
        let mut rows = [[0_u32; HOURS]; 6];
        for (index, name) in ROWS.iter().enumerate() {
            rows[index] = values[index].ok_or(TableError::Row {
                row: (*name).to_owned(),
                reason: "is missing",
            })?;
        }
        for (line, row, hour) in zeros {
            if rows[row][hour] != 0 {
                return Err(TableError::BadZeroMarker {
                    line,
                    reason: format!(
                        "{} at hour {hour} is marked zero but is {}",
                        ROWS[row], rows[row][hour]
                    ),
                });
            }
        }
        for (index, name) in ROWS.iter().enumerate() {
            let stated = checksums
                .iter()
                .find(|(row, _)| row == name)
                .map(|(_, stated)| *stated)
                .ok_or(TableError::Row {
                    row: (*name).to_owned(),
                    reason: "has no checksum",
                })?;
            let computed: u32 = rows[index].iter().sum();
            if stated != computed {
                return Err(TableError::Checksum {
                    row: (*name).to_owned(),
                    stated,
                    computed,
                });
            }
        }
        let [sleeping, eating, working, socializing, leisure, purchasing] = rows;
        Ok(Self {
            sleeping,
            eating,
            working,
            socializing,
            leisure,
            purchasing,
        })
    }
}

/// The index of a named row in [`ROWS`], if it is one.
fn row_index(name: &str) -> Option<usize> {
    ROWS.iter().position(|row| *row == name)
}

/// The shipped curves derived from one table (SD-DLb-12). Each is a [`Curve`] over the 24 clock hours.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CurveSet {
    curves: Vec<(&'static str, Curve)>,
}

/// The seven curve names of `atus-a3-2025`, in the order the set holds them.
pub const CURVE_NAMES: [&str; 7] = [
    "sociability",
    "rest",
    "restless",
    "eating",
    "working",
    "shopping",
    "leisure",
];

impl CurveSet {
    /// The curve called `name`, if the set has one.
    pub fn get(&self, name: &str) -> Option<&Curve> {
        self.curves
            .iter()
            .find(|(curve, _)| *curve == name)
            .map(|(_, curve)| curve)
    }

    /// Replaces the curve called `name` with `curve`, if the set has one of that name; `true` when it did.
    pub fn replace(&mut self, name: &str, curve: Curve) -> bool {
        match self
            .curves
            .iter_mut()
            .find(|(existing, _)| *existing == name)
        {
            Some((_, slot)) => {
                *slot = curve;
                true
            }
            None => false,
        }
    }
}

/// Derives the curve set from a table: for each row, `y(h) = v(h) × 1 000 / max` with floor division, one point
/// per hour at `x = h`. `restless` is `1 000 − rest`.
pub fn from_a3(table: &Table) -> CurveSet {
    let rest = normalised(&table.sleeping);
    let restless = Curve::new(rest.points().iter().map(|&(x, y)| (x, SCALE - y)).collect())
        .expect("the complement of a curve in 0 to 1000 is a curve");
    CurveSet {
        curves: vec![
            ("sociability", normalised(&table.socializing)),
            ("rest", rest),
            ("restless", restless),
            ("eating", normalised(&table.eating)),
            ("working", normalised(&table.working)),
            ("shopping", normalised(&table.purchasing)),
            ("leisure", normalised(&table.leisure)),
        ],
    }
}

/// One row as a curve: each hour's value over the row's largest, scaled to 1 000, with floor division.
fn normalised(row: &[u32; HOURS]) -> Curve {
    let largest = u64::from(row.iter().copied().max().unwrap_or(0).max(1));
    let points = row
        .iter()
        .enumerate()
        .map(|(hour, &value)| {
            let scaled = u64::from(value) * u64::from(SCALE) / largest;
            (
                i64::try_from(hour).expect("an hour fits i64"),
                u32::try_from(scaled).expect("a value at most the largest scales to at most 1000"),
            )
        })
        .collect();
    Curve::new(points).expect("24 points at increasing hours with values in 0 to 1000 form a curve")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A table in the TSV's shape, with the values written here, every other cell zero.
    fn synthetic(sleeping: [u32; HOURS], socializing: [u32; HOURS]) -> Table {
        Table {
            sleeping,
            eating: [0; HOURS],
            working: [0; HOURS],
            socializing,
            leisure: [0; HOURS],
            purchasing: [0; HOURS],
        }
    }

    #[test]
    fn a_row_scales_by_its_largest_value_with_floor_division() {
        // Hand-computed: 500 is the largest, so 250 is 500, 100 is 200, and 1 is 2 (2.0 floored).
        let mut sleeping = [0; HOURS];
        sleeping[0] = 250;
        sleeping[1] = 500;
        sleeping[2] = 100;
        sleeping[3] = 1;
        let set = from_a3(&synthetic(sleeping, [0; HOURS]));
        let rest = set.get("rest").expect("rest is in the set");
        assert_eq!(rest.at(0), 500);
        assert_eq!(rest.at(1), 1000);
        assert_eq!(rest.at(2), 200);
        assert_eq!(rest.at(3), 2);
    }

    #[test]
    fn restless_is_the_complement_of_rest() {
        let mut sleeping = [0; HOURS];
        sleeping[0] = 250;
        sleeping[1] = 500;
        let set = from_a3(&synthetic(sleeping, [0; HOURS]));
        let rest = set.get("rest").expect("rest is in the set");
        let restless = set.get("restless").expect("restless is in the set");
        for hour in 0..HOURS as i64 {
            assert_eq!(rest.at(hour) + restless.at(hour), SCALE);
        }
    }

    #[test]
    fn the_set_names_its_seven_curves() {
        let set = from_a3(&synthetic([1; HOURS], [1; HOURS]));
        for name in CURVE_NAMES {
            assert!(set.get(name).is_some(), "{name} is in the set");
        }
    }

    #[test]
    fn the_committed_table_parses_and_its_checksums_hold() {
        let table = Table::parse(TABLE).expect("the committed table parses");
        assert_eq!(table.sleeping.iter().sum::<u32>(), 8826);
        assert_eq!(table.socializing.iter().sum::<u32>(), 566);
        assert_eq!(table.purchasing[1], 0, "a zero marker is zero");
    }

    #[test]
    fn a_checksum_that_disagrees_is_refused() {
        let broken = TABLE.replace("sleeping=8826", "sleeping=8827");
        let error = Table::parse(&broken).expect_err("a wrong checksum is refused");
        assert!(
            matches!(error, TableError::Checksum { ref row, .. } if row == "sleeping"),
            "{error}"
        );
    }

    #[test]
    fn a_value_above_the_scale_is_refused_with_its_line() {
        let broken = TABLE.replacen("\t868\t", "\t1001\t", 1);
        let error = Table::parse(&broken).expect_err("1001 is refused");
        assert!(
            matches!(error, TableError::BadValue { line: 3, .. }),
            "{error}"
        );
    }

    #[test]
    fn a_row_with_too_few_values_is_refused() {
        let broken = TABLE.replacen("\t728\n", "\n", 1);
        let error = Table::parse(&broken).expect_err("23 values are refused");
        assert!(matches!(error, TableError::BadValue { .. }), "{error}");
    }

    #[test]
    fn a_zero_marker_on_a_nonzero_cell_is_refused() {
        let broken = TABLE.replacen(
            "purchasing-goods-and-services:1,2,3",
            "purchasing-goods-and-services:0",
            1,
        );
        let error = Table::parse(&broken).expect_err("hour 0 of purchasing is 2, not zero");
        assert!(matches!(error, TableError::BadZeroMarker { .. }), "{error}");
    }

    #[test]
    fn the_real_sociability_curve_is_pinned() {
        // The golden for the shipped set (captured from the committed table, reviewed against the source):
        // socializing / max 58 scaled to 1000, floored, at each hour.
        let set = from_a3(&Table::parse(TABLE).expect("the committed table parses"));
        let sociability = set.get("sociability").expect("sociability is in the set");
        let expected: [i64; HOURS] = [
            120, 51, 17, 34, 0, 0, 34, 51, 155, 206, 258, 413, 482, 603, 655, 741, 810, 827, 1000,
            1000, 931, 672, 431, 258,
        ];
        for (hour, value) in expected.iter().enumerate() {
            assert_eq!(
                i64::from(sociability.at(hour as i64)),
                *value,
                "hour {hour}"
            );
        }
    }

    #[test]
    fn the_real_rest_curve_is_pinned_at_night_and_midday() {
        let set = from_a3(&Table::parse(TABLE).expect("the committed table parses"));
        let rest = set.get("rest").expect("rest is in the set");
        assert_eq!(rest.at(3), 1000, "3 am is the largest sleeping value");
        assert_eq!(rest.at(12), 36, "35 scaled to 949");
        assert_eq!(rest.at(0), 914, "868 scaled to 949");
    }
}
