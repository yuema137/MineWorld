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

/// The weather a world keeps: its seed, its climate, and, when it replays one, its record.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "RawConfiguration")]
pub struct WeatherConfiguration {
    seed: u64,
    rules: Rules,
    record: Option<RecordConfiguration>,
    fill: Fill,
}

/// The file as written.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawConfiguration {
    source: Source,
    seed: u64,
    rules: Rules,
    #[serde(default)]
    record: Option<RecordConfiguration>,
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

/// The record a world replays, as its configuration names it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RecordConfiguration {
    station: Station,
    data: Attachment,
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

/// Where the days come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
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
