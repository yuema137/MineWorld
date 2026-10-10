//! WGEN-lite: one world day's weather from the month's climate, the seed, the day's index and
//! yesterday's carry (SD-TW-b-5).
//!
//! The structure is WGEN's (Richardson, C.W. 1981, "Stochastic simulation of daily precipitation,
//! temperature, and solar radiation", *Water Resources Research* 17:182–190; Richardson, C.W. & Wright,
//! D.A. 1984, *WGEN: A model for generating daily weather variables*, USDA-ARS ARS-8): a first-order
//! two-state Markov chain decides wet or dry, and temperature is an AR(1) process conditioned on wet or
//! dry. It is simplified to integers (`ARC-68`): a wet day's amount comes from five quintile amounts
//! instead of a gamma sampler, and the temperature noise is uniform and bounded.
//!
//! **Rounding.** Every division is `i64` division, which rounds toward zero; every operand that is
//! divided is bounded so the result fits (|anomaly| ≤ [`ANOMALY_LIMIT_DC`], |noise| ≤ 200, draws < 1000).

use mineworld_calendar::{CalendarDay, DAY};

use crate::day::{Chain, DailyWeather, Origin, WeatherDay};
use crate::draw::{draw, index};
use crate::hours::hours;
use crate::rules::{QUINTILES, Rules, WET_DAY_MIN_TENTH_MM};

#[cfg(test)]
mod tests;

/// The largest temperature anomaly the AR(1) process may carry, 0.1 °C (40 °C). It is reached only by
/// content with near-unit persistence and maximal noise; any realistic table stays far inside it. It
/// keeps every temperature inside `i16` (TW-b deviation TWb-D2).
pub const ANOMALY_LIMIT_DC: i64 = 400;

/// The day numbered `day_index` (`day_start / 86 400`) in `month` (1 … 12), after a day that left
/// `chain`: its summary, and the carry it leaves.
pub fn day(
    rules: &Rules,
    seed: u64,
    day_index: i64,
    month: u8,
    chain: Chain,
) -> (DailyWeather, Chain) {
    let m = rules.month(month);
    let u = |k: u64| i64::from(draw(seed, day_index, k));

    // (1) Wet or dry: yesterday decides which branch of the chain applies.
    let p_wet = if chain.wet {
        m.p_wet_after_wet()
    } else {
        m.p_wet_after_dry()
    };
    let wet = u(index::WET) < p_wet;

    // (2) A wet day's amount: its quintile, 0 … 4 because the draw is below 1000.
    let prcp = if wet {
        let quintile = usize::try_from(u(index::AMOUNT) * 5 / 1_000).unwrap_or(QUINTILES - 1);
        m.rain_tenth_mm()[quintile].max(i64::from(WET_DAY_MIN_TENTH_MM))
    } else {
        0
    };

    // (3) Temperature anomalies, integer AR(1): a = a' × ρ / 1000 + noise, noise uniform in ±bound.
    let ar = |previous: i16, noise_draw: i64| {
        let bound = m.t_noise_dc();
        let noise = noise_draw * (2 * bound + 1) / 1_000 - bound;
        (i64::from(previous) * m.t_ar_permille() / 1_000 + noise)
            .clamp(-ANOMALY_LIMIT_DC, ANOMALY_LIMIT_DC)
    };
    let a_max = ar(chain.tmax_anomaly_dc, u(index::TMAX_NOISE));
    let a_min = ar(chain.tmin_anomaly_dc, u(index::TMIN_NOISE));
    let tmax = m.tmax_dc() + a_max + if wet { m.wet_tmax_shift_dc() } else { 0 };
    // The minimum stays below the maximum: noise that would cross them lowers the minimum.
    let tmin = (m.tmin_dc() + a_min).min(tmax - 1);

    // (4) Fog any day; thunder on a wet day; a grey morning on a dry day.
    let fog = u(index::FOG) < m.fog_permille();
    let thunder = wet && u(index::THUNDER) < m.thunder_permille();
    let overcast_morning = !wet && u(index::OVERCAST) < m.overcast_morning_permille();

    let summary = DailyWeather {
        tmax_dc: narrow(tmax),
        tmin_dc: narrow(tmin),
        prcp_tenth_mm: narrow(prcp),
        // (5) The month's wind; no day-to-day noise in TW-b.
        awnd_dms: narrow(m.wind_dms()),
        wind_from_deg: narrow(m.wind_from_deg()),
        fog,
        thunder,
        overcast_morning,
    };
    let carry = Chain {
        wet,
        tmax_anomaly_dc: narrow(a_max),
        tmin_anomaly_dc: narrow(a_min),
    };
    (summary, carry)
}

/// The weather of calendar's day `calendar` (decoded from its `day-began`), after a day that left
/// `chain`: the summary drawn for its date and index, spread over its hours from its light events.
pub fn weather_day(rules: &Rules, seed: u64, calendar: &CalendarDay, chain: Chain) -> WeatherDay {
    let day_index = calendar.day_start().seconds().div_euclid(DAY);
    let (summary, carry) = day(rules, seed, day_index, calendar.date().month(), chain);
    let hours = hours(&summary, calendar.events(), seed, day_index);
    WeatherDay::new(
        calendar.day_start(),
        calendar.date(),
        Origin::Rule,
        summary,
        carry,
        hours,
    )
}

/// A value the bounds above keep inside the target type.
fn narrow<T: TryFrom<i64>>(value: i64) -> T {
    T::try_from(value)
        .ok()
        .expect("bounded by the rules' validation and ANOMALY_LIMIT_DC")
}
