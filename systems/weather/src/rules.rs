//! The climate: twelve months of WGEN-lite parameters, all integers (step-19 §17.4). Decoding is
//! validating — a value out of range is refused while the file is read, naming the key, so the loader
//! reports the file's line and column (`ARC-61`). The same types decode the copy carried in
//! `weather-configured`, so a fact can never hold a table the file could not.

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Months in a year.
pub const MONTHS: usize = 12;

/// Quintile amounts per month.
pub const QUINTILES: usize = 5;

/// The smallest wet-day amount, 0.1 mm: 0.3 mm is the first value of the GHCN-Daily record (0.1 mm
/// units) at or above NOAA's 0.01 in wet-day threshold (0.254 mm).
pub const WET_DAY_MIN_TENTH_MM: u16 = 3;

/// The twelve months, January first.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawRules", into = "RawRules")]
pub struct Rules {
    months: [Month; MONTHS],
}

impl Rules {
    /// The parameters of `month` (1 = January … 12 = December).
    ///
    /// # Panics
    ///
    /// When `month` is not 1 … 12; a calendar date's month always is.
    pub fn month(&self, month: u8) -> &Month {
        &self.months[usize::from(month) - 1]
    }

    /// All twelve, January first.
    pub const fn months(&self) -> &[Month; MONTHS] {
        &self.months
    }
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawRules {
    months: Vec<Month>,
}

impl TryFrom<RawRules> for Rules {
    type Error = String;

    fn try_from(raw: RawRules) -> Result<Self, Self::Error> {
        let count = raw.months.len();
        let months: [Month; MONTHS] = raw.months.try_into().map_err(|_| {
            format!("months has exactly {MONTHS} entries, January first, not {count}")
        })?;
        Ok(Self { months })
    }
}

impl From<Rules> for RawRules {
    fn from(rules: Rules) -> Self {
        Self {
            months: rules.months.to_vec(),
        }
    }
}

/// One month's climate. Every field is bounded; see [`Month::check`] for the bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RawMonth", into = "RawMonth")]
pub struct Month {
    raw: RawMonth,
}

/// The month as written: the keys of `configure/weather.yaml`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawMonth {
    p_wet_after_dry: i64,
    p_wet_after_wet: i64,
    rain_tenth_mm: [i64; QUINTILES],
    tmax_dc: i64,
    tmin_dc: i64,
    t_noise_dc: i64,
    t_ar_permille: i64,
    wet_tmax_shift_dc: i64,
    fog_permille: i64,
    thunder_permille: i64,
    overcast_morning_permille: i64,
    wind_dms: i64,
    wind_from_deg: i64,
}

impl TryFrom<RawMonth> for Month {
    type Error = String;

    fn try_from(raw: RawMonth) -> Result<Self, Self::Error> {
        Month::check(&raw)?;
        Ok(Self { raw })
    }
}

impl From<Month> for RawMonth {
    fn from(month: Month) -> Self {
        month.raw
    }
}

/// `key` must lie in `low ..= high`, in `unit`.
fn bounded(key: &str, value: i64, low: i64, high: i64, unit: &str) -> Result<(), String> {
    if (low..=high).contains(&value) {
        Ok(())
    } else {
        Err(format!("{key} is {unit} from {low} to {high}, not {value}"))
    }
}

impl Month {
    /// The bounds of step-19 §17.4, each refusal naming its key.
    fn check(raw: &RawMonth) -> Result<(), String> {
        let per_mille = "per mille";
        bounded("p_wet_after_dry", raw.p_wet_after_dry, 0, 1_000, per_mille)?;
        bounded("p_wet_after_wet", raw.p_wet_after_wet, 0, 1_000, per_mille)?;
        let rain = raw.rain_tenth_mm;
        let min = i64::from(WET_DAY_MIN_TENTH_MM);
        if rain.iter().any(|amount| !(min..=10_000).contains(amount))
            || rain.windows(2).any(|pair| pair[1] < pair[0])
        {
            return Err(format!(
                "rain_tenth_mm is {QUINTILES} non-decreasing amounts in 0.1 mm, each from {min} to \
                 10000, not {rain:?}"
            ));
        }
        bounded("tmax_dc", raw.tmax_dc, -900, 600, "0.1 °C")?;
        bounded("tmin_dc", raw.tmin_dc, -900, 600, "0.1 °C")?;
        if raw.tmin_dc >= raw.tmax_dc {
            return Err(format!(
                "tmin_dc must be below tmax_dc, not {} against {}",
                raw.tmin_dc, raw.tmax_dc
            ));
        }
        bounded("t_noise_dc", raw.t_noise_dc, 0, 200, "0.1 °C")?;
        bounded("t_ar_permille", raw.t_ar_permille, 0, 999, per_mille)?;
        bounded(
            "wet_tmax_shift_dc",
            raw.wet_tmax_shift_dc,
            -200,
            200,
            "0.1 °C",
        )?;
        bounded("fog_permille", raw.fog_permille, 0, 1_000, per_mille)?;
        bounded(
            "thunder_permille",
            raw.thunder_permille,
            0,
            1_000,
            per_mille,
        )?;
        bounded(
            "overcast_morning_permille",
            raw.overcast_morning_permille,
            0,
            1_000,
            per_mille,
        )?;
        bounded("wind_dms", raw.wind_dms, 0, 1_000, "0.1 m/s")?;
        bounded("wind_from_deg", raw.wind_from_deg, 0, 359, "degrees")
    }

    /// P(wet | yesterday dry), per mille.
    pub const fn p_wet_after_dry(&self) -> i64 {
        self.raw.p_wet_after_dry
    }

    /// P(wet | yesterday wet), per mille.
    pub const fn p_wet_after_wet(&self) -> i64 {
        self.raw.p_wet_after_wet
    }

    /// A wet day's amount in each quintile, 0.1 mm, non-decreasing, each at least 0.3 mm.
    pub const fn rain_tenth_mm(&self) -> [i64; QUINTILES] {
        self.raw.rain_tenth_mm
    }

    /// The mean daily maximum, 0.1 °C.
    pub const fn tmax_dc(&self) -> i64 {
        self.raw.tmax_dc
    }

    /// The mean daily minimum, 0.1 °C.
    pub const fn tmin_dc(&self) -> i64 {
        self.raw.tmin_dc
    }

    /// The bound of the temperature noise, 0.1 °C.
    pub const fn t_noise_dc(&self) -> i64 {
        self.raw.t_noise_dc
    }

    /// The temperature anomaly's day-to-day persistence, per mille.
    pub const fn t_ar_permille(&self) -> i64 {
        self.raw.t_ar_permille
    }

    /// How much cooler (negative) a wet day's maximum is, 0.1 °C.
    pub const fn wet_tmax_shift_dc(&self) -> i64 {
        self.raw.wet_tmax_shift_dc
    }

    /// P(fog), per mille.
    pub const fn fog_permille(&self) -> i64 {
        self.raw.fog_permille
    }

    /// P(thunder | wet), per mille.
    pub const fn thunder_permille(&self) -> i64 {
        self.raw.thunder_permille
    }

    /// P(grey morning | dry), per mille.
    pub const fn overcast_morning_permille(&self) -> i64 {
        self.raw.overcast_morning_permille
    }

    /// The mean wind speed, 0.1 m/s.
    pub const fn wind_dms(&self) -> i64 {
        self.raw.wind_dms
    }

    /// The prevailing direction the wind comes from, degrees.
    pub const fn wind_from_deg(&self) -> i64 {
        self.raw.wind_from_deg
    }
}
