//! `configure/weather.yaml`, decoded by this pack's own types (`ARC-61`, SD-TW-b-3): deserializing is
//! validating, so a bad value is refused at assembly with the file, its line and column, and a message
//! naming the key.
//!
//! ```yaml
//! source: rules        # TW-b admits only `rules`; `record` arrives with TW-d's station data
//! seed: 19             # content, not --seed: the world's weather is part of the world; u64
//! rules:
//!   months:            # exactly 12, January first (see `rules.rs` for every key and bound)
//!     - { p_wet_after_dry: 147, p_wet_after_wet: 447, rain_tenth_mm: [8, 28, 54, 93, 178], … }
//! ```
//!
//! What the configuration becomes is [`WeatherConfigured`](crate::event::WeatherConfigured); this type
//! is only ever read from the file.

use serde::Deserialize;

use crate::rules::Rules;

/// The weather a world keeps: where its days come from, its seed and its climate.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeatherConfiguration {
    source: Source,
    seed: u64,
    rules: Rules,
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
}

/// Where the days come from. TW-b has one source.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
enum Source {
    Rules,
}

impl TryFrom<String> for Source {
    type Error = String;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        match text.as_str() {
            "rules" => Ok(Self::Rules),
            "record" => Err(
                "source `record` needs the record data of TW-d; this build accepts `rules`"
                    .to_owned(),
            ),
            _ => Err(format!("source is `rules`, not '{text}'")),
        }
    }
}
