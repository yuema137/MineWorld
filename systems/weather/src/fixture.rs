//! Tables the unit tests share: Market Town's San Diego climate (fitted to its record since TW-d) — read from the world's own
//! `configure/weather.yaml`, so the statistics are measured on the shipped table and there is one copy
//! of it (step-19 §17.4) — and a constant table for the spell-length check.

use crate::configuration::WeatherConfiguration;
use crate::rules::Rules;

/// `worlds/market-town/configure/weather.yaml`.
const MARKET_TOWN: &str = include_str!("../../../worlds/market-town/configure/weather.yaml");

/// Market Town's table as a `rules:` block's text: the file's lines after `rules:`, de-indented.
pub fn san_diego_text() -> String {
    MARKET_TOWN
        .lines()
        .skip_while(|line| *line != "rules:")
        .skip(1)
        .map(|line| format!("{}\n", line.strip_prefix("  ").unwrap_or(line)))
        .collect()
}

/// Market Town's table, decoded through the whole configuration as the loader decodes it.
pub fn san_diego() -> Rules {
    let configuration: WeatherConfiguration =
        serde_saphyr::from_str(MARKET_TOWN).expect("Market Town's weather decodes");
    assert_eq!(configuration.seed(), 19);
    configuration.rules().clone()
}

/// Decodes a `rules:` block as the loader would.
pub fn rules(text: &str) -> Rules {
    serde_saphyr::from_str(text).expect("the table decodes")
}

/// Every month the same: P(W|D) = `p_wd`, P(W|W) = `p_ww`, otherwise San Diego's January.
pub fn constant(p_wd: i64, p_ww: i64) -> Rules {
    let month = format!(
        "  - {{ p_wet_after_dry: {p_wd}, p_wet_after_wet: {p_ww}, rain_tenth_mm: [8, 28, 54, 93, 178], \
         tmax_dc: 191, tmin_dc: 102, t_noise_dc: 30, t_ar_permille: 600, wet_tmax_shift_dc: -20, \
         fog_permille: 80, thunder_permille: 40, overcast_morning_permille: 150, wind_dms: 25, \
         wind_from_deg: 300 }}\n"
    );
    rules(&format!("months:\n{}", month.repeat(12)))
}
