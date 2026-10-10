//! `fit`: the weather pack's rules (WGEN-lite) estimated from a record, in integers only, so the same
//! record and base give the same bytes on every machine (step-19 SD-TW-d-8c, SD-TW-d-9).
//!
//! Per month, from the record's complete days (maximum, minimum and precipitation present); a day is
//! wet when at least 0.3 mm fell (NOAA's 0.01 in, rounded up to the record's 0.1 mm), and a day belongs
//! to the month of its date, as the generator draws it:
//!
//! ```text
//! p_wet_after_dry, p_wet_after_wet   ‰ of counted transitions into the month's days from a complete
//!                                    yesterday: dry→wet among dry yesterdays, wet→wet among wet ones
//! rain_tenth_mm                      the 10th/30th/50th/70th/90th percentile of wet-day amounts
//!                                    (nearest rank: the ⌈p·n/100⌉-th smallest)
//! tmax_dc, tmin_dc                   means
//! t_ar_permille  ρ                   lag-1 autocorrelation of the anomalies (each day's departure
//!                                    from its own month's means), TMAX and TMIN pooled, over the
//!                                    month's days with a complete yesterday; clamped to 0 … 999
//! t_noise_dc                         ⌊√(3 · var · (1 − ρ²))⌋: uniform noise on ±n has variance n²/3,
//!                                    and an AR(1) anomaly with persistence ρ has variance
//!                                    noise / (1 − ρ²); var pools TMAX's and TMIN's anomalies
//! wet_tmax_shift_dc                  mean TMAX of wet days − mean TMAX of all days
//! fog_permille, thunder_permille     ‰ of days with fog; ‰ of wet days with thunder
//! wind_dms, wind_from_deg            mean wind speed; the modal direction in 10° bins (ties: the
//!                                    lowest bin)
//! overcast_morning_permille          not in GHCN-Daily: copied from --base
//! ```
//!
//! Rounding is to the nearest integer, halves away from zero. A month with no sample for an estimate
//! (no wet day, no transition from a wet day, no wind) keeps --base's value, and the output's comment
//! for that month says which. Every value is clamped to the pack's bounds, and the text is decoded by
//! the pack before it is returned.

use std::fmt::Write as _;

use mineworld_weather::WeatherConfiguration;
use mineworld_weather::record::{Fill, RecordFile, RecordRow};
use mineworld_weather::rules::{MONTHS, Month};

/// A wet day: at least 0.3 mm.
const WET: u16 = 3;

const NAMES: [&str; MONTHS] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// The record block the output names, when the world replays a record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecordBlock {
    /// The station.
    pub station: String,
    /// The attachment's path under the World Pack.
    pub data: String,
    /// The record year the world's first year replays.
    pub first_year: i32,
    /// `rules` or `none`.
    pub fill: String,
}

impl RecordBlock {
    /// The block a configuration already has.
    pub fn of(configuration: &WeatherConfiguration) -> Option<Self> {
        configuration.record().map(|record| Self {
            station: record.station().as_str().to_owned(),
            data: record.data().to_string(),
            first_year: record.first_year(),
            fill: match configuration.fill() {
                Fill::Rules => "rules".to_owned(),
                Fill::None => "none".to_owned(),
            },
        })
    }
}

/// `n / d` rounded to the nearest, halves away from zero; `d > 0`.
fn rounded(n: i128, d: i128) -> i128 {
    if n >= 0 {
        (n + d / 2) / d
    } else {
        (n - d / 2) / d
    }
}

/// `count / of` in ‰, rounded.
fn per_mille(count: usize, of: usize) -> i128 {
    rounded(wide(count) * 1_000, wide(of))
}

fn wide(count: usize) -> i128 {
    i128::try_from(count).expect("a count fits i128")
}

/// ⌊√n⌋ by Newton's method on integers.
fn isqrt(n: i128) -> i128 {
    if n < 2 {
        return n.max(0);
    }
    let mut x = n;
    let mut y = x / 2 + x % 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

fn wet(row: &RecordRow) -> bool {
    row.prcp_tenth_mm.is_some_and(|p| p >= WET)
}

fn month_of(row: &RecordRow) -> usize {
    usize::from(row.date.month()) - 1
}

fn tmax(row: &RecordRow) -> i128 {
    i128::from(row.tmax_dc.expect("a complete day"))
}

fn tmin(row: &RecordRow) -> i128 {
    i128::from(row.tmin_dc.expect("a complete day"))
}

/// One month's fitted values, in the file's key order, and the keys kept from the base.
struct MonthFit {
    values: Vec<(&'static str, i64)>,
    rain: [i64; 5],
    from_base: Vec<&'static str>,
}

impl MonthFit {
    /// `fitted`, clamped to `low ..= high`, or the base's `fallback` (noted) when there is no sample.
    fn take(
        &mut self,
        key: &'static str,
        fitted: Option<i128>,
        fallback: i64,
        (low, high): (i64, i64),
    ) {
        let value = fitted.map_or_else(
            || {
                self.from_base.push(key);
                fallback
            },
            |value| i64::try_from(value.clamp(i128::from(low), i128::from(high))).expect("clamped"),
        );
        self.values.push((key, value));
    }
}

/// Each month's mean TMAX and TMIN over its complete days; none for a month without one.
fn means(rows: &[RecordRow]) -> [(Option<i128>, Option<i128>); MONTHS] {
    core::array::from_fn(|month| {
        let days: Vec<&RecordRow> = rows
            .iter()
            .filter(|row| row.complete() && month_of(row) == month)
            .collect();
        if days.is_empty() {
            return (None, None);
        }
        let n = wide(days.len());
        (
            Some(rounded(days.iter().map(|row| tmax(row)).sum(), n)),
            Some(rounded(days.iter().map(|row| tmin(row)).sum(), n)),
        )
    })
}

fn fit_month(
    rows: &[RecordRow],
    month: usize,
    base: &Month,
    means: &[(Option<i128>, Option<i128>); MONTHS],
) -> MonthFit {
    let mut fit = MonthFit {
        values: Vec::new(),
        rain: base.rain_tenth_mm(),
        from_base: Vec::new(),
    };
    let days: Vec<(usize, &RecordRow)> = rows
        .iter()
        .enumerate()
        .filter(|(_, row)| row.complete() && month_of(row) == month)
        .collect();
    let yesterday = |index: usize| {
        index
            .checked_sub(1)
            .map(|at| &rows[at])
            .filter(|row| row.complete())
    };

    // The chain.
    let (mut dry, mut dry_wet, mut wet_before, mut wet_wet) = (0, 0, 0, 0);
    for (index, today) in &days {
        let Some(before) = yesterday(*index) else {
            continue;
        };
        if wet(before) {
            wet_before += 1;
            wet_wet += usize::from(wet(today));
        } else {
            dry += 1;
            dry_wet += usize::from(wet(today));
        }
    }
    fit.take(
        "p_wet_after_dry",
        (dry > 0).then(|| per_mille(dry_wet, dry)),
        base.p_wet_after_dry(),
        (0, 1_000),
    );
    fit.take(
        "p_wet_after_wet",
        (wet_before > 0).then(|| per_mille(wet_wet, wet_before)),
        base.p_wet_after_wet(),
        (0, 1_000),
    );

    // The amounts.
    let mut amounts: Vec<i64> = days
        .iter()
        .filter(|(_, row)| wet(row))
        .map(|(_, row)| i64::from(row.prcp_tenth_mm.expect("a complete day")))
        .collect();
    amounts.sort_unstable();
    if amounts.is_empty() {
        fit.from_base.push("rain_tenth_mm");
    } else {
        let count = amounts.len();
        fit.rain = [10, 30, 50, 70, 90]
            .map(|p| amounts[(p * count).div_ceil(100).max(1) - 1].clamp(3, 10_000));
    }

    // Temperature: the means, the persistence and the noise.
    let (tmax_mean, tmin_mean) = means[month];
    fit.take("tmax_dc", tmax_mean, base.tmax_dc(), (-899, 600));
    let tmax_dc = fit.values.last().expect("pushed").1;
    fit.take("tmin_dc", tmin_mean, base.tmin_dc(), (-900, tmax_dc - 1));
    let anomaly = |row: &RecordRow| -> Option<(i128, i128)> {
        let (tmax_mean, tmin_mean) = means[month_of(row)];
        Some((tmax(row) - tmax_mean?, tmin(row) - tmin_mean?))
    };
    let (mut cross, mut square, mut variance_sum) = (0i128, 0i128, 0i128);
    for (index, today) in &days {
        let Some((a1, b1)) = anomaly(today) else {
            continue;
        };
        variance_sum += a1 * a1 + b1 * b1;
        if let Some((a0, b0)) = yesterday(*index).and_then(anomaly) {
            cross += a1 * a0 + b1 * b0;
            square += a0 * a0 + b0 * b0;
        }
    }
    let rho = (square > 0).then(|| rounded(cross * 1_000, square).clamp(0, 999));
    let rho_value = rho.unwrap_or_else(|| i128::from(base.t_ar_permille()));
    let noise = (!days.is_empty()).then(|| {
        let variance = rounded(variance_sum, 2 * wide(days.len()));
        isqrt(3 * variance * (1_000_000 - rho_value * rho_value) / 1_000_000)
    });
    fit.take("t_noise_dc", noise, base.t_noise_dc(), (0, 200));
    fit.take("t_ar_permille", rho, base.t_ar_permille(), (0, 999));

    // Wet days' maximum, fog, thunder.
    let wet_days: Vec<&RecordRow> = days
        .iter()
        .map(|(_, row)| *row)
        .filter(|row| wet(row))
        .collect();
    let shift = match (wet_days.is_empty(), tmax_mean) {
        (false, Some(mean)) => Some(
            rounded(
                wet_days.iter().map(|row| tmax(row)).sum(),
                wide(wet_days.len()),
            ) - mean,
        ),
        _ => None,
    };
    fit.take(
        "wet_tmax_shift_dc",
        shift,
        base.wet_tmax_shift_dc(),
        (-200, 200),
    );
    let fog = days.iter().filter(|(_, row)| row.fog).count();
    fit.take(
        "fog_permille",
        (!days.is_empty()).then(|| per_mille(fog, days.len())),
        base.fog_permille(),
        (0, 1_000),
    );
    let thunder = wet_days.iter().filter(|row| row.thunder).count();
    fit.take(
        "thunder_permille",
        (!wet_days.is_empty()).then(|| per_mille(thunder, wet_days.len())),
        base.thunder_permille(),
        (0, 1_000),
    );
    fit.values.push((
        "overcast_morning_permille",
        base.overcast_morning_permille(),
    ));

    // Wind.
    let speeds: Vec<i128> = days
        .iter()
        .filter_map(|(_, row)| row.awnd_dms.map(i128::from))
        .collect();
    let speed = (!speeds.is_empty()).then(|| rounded(speeds.iter().sum(), wide(speeds.len())));
    fit.take("wind_dms", speed, base.wind_dms(), (0, 1_000));
    let mut bins = [0usize; 36];
    for (_, row) in &days {
        if let Some(degrees) = row.wdf2_deg {
            bins[usize::from(degrees % 360) / 10] += 1;
        }
    }
    let (bin, count) =
        bins.iter().enumerate().fold(
            (0, 0),
            |best, (bin, count)| if *count > best.1 { (bin, *count) } else { best },
        );
    fit.take(
        "wind_from_deg",
        (count > 0).then(|| wide(bin * 10)),
        base.wind_from_deg(),
        (0, 359),
    );
    fit
}

/// The rules fitted to `file`, as the full text of a `configure/weather.yaml`: a header saying how it
/// was made, the base's seed, the record block (from `record`, else the base's, else none), and the
/// fitted months.
///
/// # Errors
///
/// A base that is not a weather configuration, or (a defect) output the pack would not decode.
pub fn fit(
    file: &RecordFile,
    base_text: &str,
    record: Option<RecordBlock>,
) -> Result<String, String> {
    let base: WeatherConfiguration = serde_saphyr::from_str(base_text)
        .map_err(|error| format!("--base is not a weather configuration: {error}"))?;
    let record = record.or_else(|| RecordBlock::of(&base));
    let rows = file.rows();
    let means = means(rows);
    let months: Vec<MonthFit> = (0..MONTHS)
        .map(|month| {
            let number = u8::try_from(month + 1).expect("a month");
            fit_month(rows, month, base.rules().month(number), &means)
        })
        .collect();

    let from = record
        .as_ref()
        .map_or_else(|| "its input".to_owned(), |block| block.data.clone());
    let mut text = format!(
        "# Weather (`weather` System Pack; docs/DECISIONS.md ARC-61, ARC-68, DEP-31).\n\
         #\n\
         # Written by `weather-fetch fit` (mineworld-weather-fetch {version}) from {from}, with the seed and\n\
         # overcast_morning_permille of the base configuration; re-run the tool rather than edit by hand\n\
         # (tools/weather-fetch/README.md). WGEN-lite rules fitted in integers from the record's complete days\n\
         # (step-19 SD-TW-d-8c): wet = at least 0.3 mm; p_wet_after_dry and p_wet_after_wet are counted\n\
         # transitions; rain_tenth_mm the 10th, 30th, 50th, 70th and 90th percentiles of wet-day amounts;\n\
         # tmax_dc and tmin_dc means; t_ar_permille the lag-1 autocorrelation of the temperature anomalies;\n\
         # t_noise_dc = isqrt(3 * variance * (1 - rho^2)) of the anomalies; wet_tmax_shift_dc the wet days'\n\
         # mean maximum minus all days'; fog and thunder frequencies (thunder among wet days); wind the mean\n\
         # speed and the modal direction in 10-degree bins. overcast_morning_permille is not in GHCN-Daily.\n\
         # With `source: record` the rules draw only the days the record lacks for more than three days in a\n\
         # row, and give the month's wind where the record has none.\n",
        version = env!("CARGO_PKG_VERSION"),
    );
    match &record {
        Some(block) => {
            let _ = write!(
                text,
                "source: record\nseed: {}\nrecord:\n  station: {}\n  data: {}\n  first_year: {}\nfill: {}\n",
                base.seed(),
                block.station,
                block.data,
                block.first_year,
                block.fill
            );
        }
        None => {
            let _ = writeln!(text, "source: rules\nseed: {}", base.seed());
        }
    }
    text.push_str("rules:\n  months:\n");
    for (month, fit) in months.iter().enumerate() {
        let note = if fit.from_base.is_empty() {
            String::new()
        } else {
            format!(" (from the base: {})", fit.from_base.join(", "))
        };
        let _ = writeln!(text, "    - # {}{note}", NAMES[month]);
        for (index, (key, value)) in fit.values.iter().enumerate() {
            if index == 2 {
                let rain: Vec<String> = fit.rain.iter().map(ToString::to_string).collect();
                let _ = writeln!(text, "      rain_tenth_mm: [{}]", rain.join(", "));
            }
            let _ = writeln!(text, "      {key}: {value}");
        }
    }
    serde_saphyr::from_str::<WeatherConfiguration>(&text)
        .map_err(|error| format!("the fitted configuration does not decode (a defect): {error}"))?;
    Ok(text)
}
