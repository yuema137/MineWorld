//! `configure/weather.yaml`, decoded by this pack's own types (`ARC-61`; SD-TW-b-3, SD-TW-d-7):
//! deserializing is validating, so a bad value is refused at assembly with the file, its line and
//! column, and a message naming the key.
//!
//! ```yaml
//! source: record       # `rules`: every day drawn by the rules; `record`: a station's days replayed
//! seed: 19             # content, not --seed: the world's weather is part of the world; u64
//! record:              # required with `source: record`, refused otherwise
//!   station: USW00023188                                    # provenance only
//!   data: data/weather/san-diego-usw00023188-2015-2024.csv  # an attachment under the pack's data/
//!   first_year: 2015                                        # the record year the world's first replays
//! fill: rules          # `source: record` only; `rules` (the default) or `none`
//! rules:               # always: they draw rule days and fill gaps (see `rules.rs`)
//!   months:
//!     - { p_wet_after_dry: 147, p_wet_after_wet: 447, rain_tenth_mm: [8, 28, 54, 93, 178], … }
//! ```
//!
//! What the configuration becomes is [`WeatherConfigured`](crate::event::WeatherConfigured); the
//! record's bytes are decoded when the pack seeds it, because only then does it have them.

use mineworld_authoring::Attachment;
use serde::Deserialize;

use crate::record::{Fill, Station};
use crate::rules::Rules;

/// The weather a world keeps: drawn every day from its climate's rules, or replayed from a weather
/// station's record with the rules filling its gaps.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "RawConfiguration")]
#[schemars(example = example())]
pub struct WeatherConfiguration {
    seed: u64,
    rules: Rules,
    record: Option<RecordConfiguration>,
    fill: Fill,
}

/// The example the generated reference prints: rules only, one mild month repeated for the year.
fn example() -> serde_json::Value {
    let month = serde_json::json!({
        "p_wet_after_dry": 150, "p_wet_after_wet": 450, "rain_tenth_mm": [8, 28, 54, 93, 178],
        "tmax_dc": 210, "tmin_dc": 120, "t_noise_dc": 30, "t_ar_permille": 600,
        "wet_tmax_shift_dc": -20, "fog_permille": 50, "thunder_permille": 20,
        "overcast_morning_permille": 100, "wind_dms": 35, "wind_from_deg": 270
    });
    serde_json::json!({
        "source": "rules",
        "seed": 19,
        "rules": { "months": vec![month; crate::rules::MONTHS] },
    })
}

/// The file as written.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RawConfiguration {
    /// Where the days come from: rules (every day drawn from the climate) or record (a station's days
    /// replayed).
    source: Source,
    /// The seed every drawn day comes from: part of the world, so the same world has the same
    /// weather. Not the run's --seed.
    seed: u64,
    /// The climate: it draws every day with source rules, and fills the record's gaps with source
    /// record.
    rules: Rules,
    /// The record to replay: required with source record, refused otherwise.
    #[serde(default)]
    record: Option<RecordConfiguration>,
    /// With source record only: what a long gap in the record becomes; rules when unsaid.
    #[serde(default)]
    fill: Option<Fill>,
}

impl TryFrom<RawConfiguration> for WeatherConfiguration {
    type Error = String;

    fn try_from(raw: RawConfiguration) -> Result<Self, Self::Error> {
        let record = match (raw.source, raw.record) {
            (Source::Record, Some(record)) => Some(record),
            (Source::Record, None) => {
                return Err(
                    "source `record` needs a `record:` block (station, data, first_year)"
                        .to_owned(),
                );
            }
            (Source::Rules, Some(_)) => {
                return Err("`record:` is read only with source `record`".to_owned());
            }
            (Source::Rules, None) => None,
        };
        if record.is_none() && raw.fill.is_some() {
            return Err("`fill:` applies only to source `record`".to_owned());
        }
        Ok(Self {
            seed: raw.seed,
            rules: raw.rules,
            record,
            fill: raw.fill.unwrap_or_default(),
        })
    }
}

/// The record a world replays: a station's daily weather, as a file in the World Pack's data/.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RecordConfiguration {
    /// The station the record comes from, for provenance.
    station: Station,
    /// The record file, made by tools/weather-fetch: a CSV under data/.
    data: Attachment,
    /// The record year the world's first year replays.
    first_year: i32,
}

impl RecordConfiguration {
    /// The station, for provenance.
    pub const fn station(&self) -> &Station {
        &self.station
    }

    /// The file under the pack's `data/`.
    pub const fn data(&self) -> &Attachment {
        &self.data
    }

    /// The record year the world's first year replays.
    pub const fn first_year(&self) -> i32 {
        self.first_year
    }
}

impl WeatherConfiguration {
    /// The seed of every rule day and hour placement.
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// The climate.
    pub const fn rules(&self) -> &Rules {
        &self.rules
    }

    /// The record, with `source: record`.
    pub const fn record(&self) -> Option<&RecordConfiguration> {
        self.record.as_ref()
    }

    /// What a long gap in the record becomes.
    pub const fn fill(&self) -> Fill {
        self.fill
    }
}

/// Where the days come from: rules or record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "String")]
#[schemars(extend("enum" = ["rules", "record"]))]
enum Source {
    Rules,
    Record,
}

impl TryFrom<String> for Source {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        match text.as_str() {
            "rules" => Ok(Self::Rules),
            "record" => Ok(Self::Record),
            _ => Err(format!("source is `rules` or `record`, not '{text}'")),
        }
    }
}
