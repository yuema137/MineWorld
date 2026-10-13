//! `fit`'s own oracle (step-19 §18.4 C3 (c)): a 100-year record drawn by the pack's own generator from a
//! known table is fitted back; the chain's probabilities come back within ±20 ‰ and the mean extremes
//! within ±3 (0.1 °C). The fit is also byte-reproducible and integer-only.

use mineworld_weather::generate::day;
use mineworld_weather::record::{RecordRow, decode, encode};
use mineworld_weather::rules::Rules;
use mineworld_weather::{CalendarDate, Chain, WeatherConfiguration};
use mineworld_weather_fetch::fit::fit;

/// The known table: every month different, no wet-day temperature shift (so the all-day means are the
/// table's means), as a whole configuration.
fn known() -> String {
    let months: String = (0..12)
        .map(|m| {
            format!(
                "    - {{ p_wet_after_dry: {}, p_wet_after_wet: {}, rain_tenth_mm: [5, 20, 60, 150, 400], \
                 tmax_dc: {}, tmin_dc: {}, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: 0, \
                 fog_permille: {}, thunder_permille: 50, overcast_morning_permille: 150, wind_dms: 25, \
                 wind_from_deg: 300 }}\n",
                40 + 20 * m,
                250 + 30 * m,
                150 + 10 * m,
                80 + 8 * m,
                20 * m
            )
        })
        .collect();
    format!("source: rules\nseed: 7\nrules:\n  months:\n{months}")
}

fn rules(text: &str) -> Rules {
    let configuration: WeatherConfiguration = serde_saphyr::from_str(text).expect("decodes");
    configuration.rules().clone()
}

/// 100 years, 2001–2100, drawn day by day by the pack's generator.
fn synthetic(rules: &Rules) -> Vec<RecordRow> {
    let mut rows = Vec::new();
    let mut on = CalendarDate::new(2001, 1, 1).expect("a date");
    let mut chain = Chain::default();
    let mut index = 0i64;
    while on.year() <= 2100 {
        let (summary, carry) = day(rules, 7, index, on.month(), chain);
        chain = carry;
        rows.push(RecordRow {
            date: on,
            tmax_dc: Some(summary.tmax_dc),
            tmin_dc: Some(summary.tmin_dc),
            prcp_tenth_mm: Some(summary.prcp_tenth_mm),
            awnd_dms: Some(u8::try_from(summary.awnd_dms).expect("small")),
            wdf2_deg: Some(summary.wind_from_deg),
            fog: summary.fog,
            thunder: summary.thunder,
        });
        on = on.next().expect("a next day");
        index += 1;
    }
    rows
}

#[test]
fn fit_recovers_a_known_table_from_its_own_weather() {
    let text = known();
    let truth = rules(&text);
    let rows = synthetic(&truth);
    // How many of each month's days follow a wet day: the sample p_wet_after_wet is estimated from.
    let mut after_wet = [0u32; 12];
    for pair in rows.windows(2) {
        if pair[0].prcp_tenth_mm.is_some_and(|p| p >= 3) {
            after_wet[usize::from(pair[1].date.month()) - 1] += 1;
        }
    }
    let csv = encode(&rows);
    let file = decode(csv.as_bytes()).expect("decodes");
    assert_eq!(file.rows().len(), 36_524);
    let fitted_text = fit(&file, &text, None).expect("fits");
    assert_eq!(
        fit(&file, &text, None).expect("fits"),
        fitted_text,
        "byte-reproducible"
    );
    let fitted = rules(&fitted_text);
    for (month, (truth, fitted)) in truth.months().iter().zip(fitted.months()).enumerate() {
        println!(
            "month {:>2}: p_wd {:>3}/{:>3} p_ww {:>3}/{:>3} tmax {}/{} tmin {}/{} noise {}/{} ar {}/{} fog {}/{} \
             wind {}/{} from {}/{}",
            month + 1,
            fitted.p_wet_after_dry(),
            truth.p_wet_after_dry(),
            fitted.p_wet_after_wet(),
            truth.p_wet_after_wet(),
            fitted.tmax_dc(),
            truth.tmax_dc(),
            fitted.tmin_dc(),
            truth.tmin_dc(),
            fitted.t_noise_dc(),
            truth.t_noise_dc(),
            fitted.t_ar_permille(),
            truth.t_ar_permille(),
            fitted.fog_permille(),
            truth.fog_permille(),
            fitted.wind_dms(),
            truth.wind_dms(),
            fitted.wind_from_deg(),
            truth.wind_from_deg(),
        );
        assert!(
            (fitted.p_wet_after_dry() - truth.p_wet_after_dry()).abs() <= 20,
            "month {} p_wd",
            month + 1
        );
        // ±20 ‰, or three standard errors of a binomial estimate from this month's days after a wet
        // day when that is wider: a dry month has only a few hundred, so ±20 ‰ alone would fail by
        // chance, not by defect (step-19 TWd-D8).
        let p = f64::from(u32::try_from(truth.p_wet_after_wet()).expect("‰"));
        let standard_error = (p * (1_000.0 - p) / f64::from(after_wet[month])).sqrt();
        let tolerance = 20_i64.max(i64::from(
            u16::try_from((3.0 * standard_error).ceil() as i64).expect("small"),
        ));
        println!(
            "           p_ww tolerance {tolerance} ‰ ({} days after a wet day)",
            after_wet[month]
        );
        assert!(
            (fitted.p_wet_after_wet() - truth.p_wet_after_wet()).abs() <= tolerance,
            "month {} p_ww",
            month + 1
        );
        assert!(
            (fitted.tmax_dc() - truth.tmax_dc()).abs() <= 3,
            "month {} tmax",
            month + 1
        );
        assert!(
            (fitted.tmin_dc() - truth.tmin_dc()).abs() <= 3,
            "month {} tmin",
            month + 1
        );
        assert_eq!(
            fitted.overcast_morning_permille(),
            truth.overcast_morning_permille(),
            "copied from the base"
        );
    }
    assert!(fitted_text.starts_with("# Weather (`weather` System Pack"));
    assert!(fitted_text.contains("source: rules\nseed: 7\n"));
}

/// No floating-point type in the tool's source: the fit is integer arithmetic (as TW-b criterion 6 scans
/// the pack).
#[test]
fn no_floating_point_type_appears_in_the_tools_source() {
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let tokens = [concat!("f", "32"), concat!("f", "64")];
    let mut scanned = 0;
    for entry in std::fs::read_dir(&directory).expect("readable") {
        let path = entry.expect("an entry").path();
        let text = std::fs::read_to_string(&path).expect("readable");
        scanned += 1;
        for token in tokens {
            let word = |c: char| c.is_ascii_alphanumeric() || c == '_';
            let found = text.match_indices(token).any(|(at, _)| {
                text[..at].chars().next_back().is_none_or(|c| !word(c))
                    && text[at + token.len()..]
                        .chars()
                        .next()
                        .is_none_or(|c| !word(c))
            });
            assert!(!found, "{} holds {token}", path.display());
        }
    }
    assert!(scanned >= 6, "scanned {scanned} files");
}
