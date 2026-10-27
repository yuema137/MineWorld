//! `price-reporter`'s parameter block (`pr-s31d-reporter-policies.md` §5.2), decoded by hand: an unknown key
//! is refused listing the known ones, each bound at its value, a key written twice refused, and
//! `last_minute < first_minute` refused naming `last_minute`. Every key may be omitted.

use core::fmt;

use mineworld_decision::ModelParameters;
use mineworld_notices::TopicId;
use schemars::JsonSchema;
use serde::de::{Error as DeError, MapAccess, Visitor};
use serde::{Deserializer, Serialize};

/// The keys of the block.
const KEYS: &[&str] = &[
    "topic",
    "first_minute",
    "last_minute",
    "every_minutes",
    "valid_minutes",
    "min_basis",
    "scarce_below",
    "scarce_permille",
    "plenty_above",
    "plenty_permille",
];

/// The clerk's parameters, as `decisions.yaml` writes them under `models: { price-reporter: … }`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, JsonSchema)]
#[schemars(extend("required" = Vec::<&str>::new()))]
pub struct ReporterParameters {
    /// The topic the bulletin is posted on: a dotted slug, 1 … 64 bytes (`prices.ingredients`).
    pub topic: TopicId,
    /// The first publish slot, minute of the day, 0 … 1 439 (480: 08:00).
    #[schemars(range(min = 0, max = 1_439))]
    pub first_minute: u32,
    /// No slot after this minute of the day, `first_minute` … 1 439 (1 200: 20:00).
    #[schemars(range(min = 0, max = 1_439))]
    pub last_minute: u32,
    /// Minutes between slots, 15 … 1 440 (240).
    #[schemars(range(min = 15, max = 1_440))]
    pub every_minutes: u32,
    /// The notice's validity in minutes, 1 … 43 200 (360); the topic's own bounds are the server's check.
    #[schemars(range(min = 1, max = 43_200))]
    pub valid_minutes: u32,
    /// Notes a line must rest on, 1 … 64 (3).
    #[schemars(range(min = 1, max = 64))]
    pub min_basis: u32,
    /// Median shop stock below this raises the line, 0 … 1 000 (3; 0: off).
    #[schemars(range(min = 0, max = 1_000))]
    pub scarce_below: u32,
    /// Per mille added when scarce, 0 … 1 000 (100).
    #[schemars(range(min = 0, max = 1_000))]
    pub scarce_permille: u32,
    /// Median shop stock above this lowers the line, 1 … 100 000 (20).
    #[schemars(range(min = 1, max = 100_000))]
    pub plenty_above: u32,
    /// Per mille subtracted when plentiful, 0 … 900 (50).
    #[schemars(range(min = 0, max = 900))]
    pub plenty_permille: u32,
}

impl Default for ReporterParameters {
    fn default() -> Self {
        Self {
            topic: TopicId::new("prices.ingredients").expect("a valid default topic"),
            first_minute: 480,
            last_minute: 1_200,
            every_minutes: 240,
            valid_minutes: 360,
            min_basis: 3,
            scarce_below: 3,
            scarce_permille: 100,
            plenty_above: 20,
            plenty_permille: 50,
        }
    }
}

impl ModelParameters for ReporterParameters {
    fn decode<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_map(Fields)
    }
}

/// A value inside `low ..= high`, or a refusal naming the bound.
fn within<E: DeError>(key: &str, value: u32, low: u32, high: u32) -> Result<u32, E> {
    if value < low || value > high {
        Err(E::custom(format!("'{key}' is {low} … {high}, not {value}")))
    } else {
        Ok(value)
    }
}

struct Fields;

impl<'de> Visitor<'de> for Fields {
    type Value = ReporterParameters;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a mapping of price-reporter parameters ({})",
            KEYS.join(", ")
        )
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<ReporterParameters, A::Error> {
        let mut parameters = ReporterParameters::default();
        let mut seen: Vec<String> = Vec::new();
        while let Some(key) = map.next_key::<String>()? {
            if seen.contains(&key) {
                return Err(A::Error::custom(format!("'{key}' is written twice")));
            }
            let bounded = |low, high, value| within::<A::Error>(&key, value, low, high);
            match key.as_str() {
                "topic" => {
                    let text: String = map.next_value()?;
                    parameters.topic = TopicId::new(text).map_err(A::Error::custom)?;
                }
                "first_minute" => parameters.first_minute = bounded(0, 1_439, map.next_value()?)?,
                "last_minute" => parameters.last_minute = bounded(0, 1_439, map.next_value()?)?,
                "every_minutes" => {
                    parameters.every_minutes = bounded(15, 1_440, map.next_value()?)?;
                }
                "valid_minutes" => {
                    parameters.valid_minutes = bounded(1, 43_200, map.next_value()?)?;
                }
                "min_basis" => parameters.min_basis = bounded(1, 64, map.next_value()?)?,
                "scarce_below" => parameters.scarce_below = bounded(0, 1_000, map.next_value()?)?,
                "scarce_permille" => {
                    parameters.scarce_permille = bounded(0, 1_000, map.next_value()?)?;
                }
                "plenty_above" => {
                    parameters.plenty_above = bounded(1, 100_000, map.next_value()?)?;
                }
                "plenty_permille" => {
                    parameters.plenty_permille = bounded(0, 900, map.next_value()?)?;
                }
                _ => return Err(A::Error::unknown_field(&key, KEYS)),
            }
            seen.push(key);
        }
        if parameters.last_minute < parameters.first_minute {
            return Err(A::Error::custom(format!(
                "'last_minute' is {} … 1439 (not before first_minute), not {}",
                parameters.first_minute, parameters.last_minute
            )));
        }
        Ok(parameters)
    }
}
