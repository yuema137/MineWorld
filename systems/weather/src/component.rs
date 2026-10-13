//! The two records this pack discloses (SD-TW-b-11). They are declared component types so that presence
//! keeps them (`owned_by_an_enabled_system`), but no entity carries either: the state lives in the
//! `climate` Process, and these are how it is shown, on the place an observer is in.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::day::{Condition, WeatherDay};
use crate::system::WeatherSystem;

/// `weather-today`: today's weather — its summary and its 24 hours.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct WeatherToday(pub WeatherDay);

owned_component! {
    component = WeatherToday,
    owner = WeatherSystem,
    component_type = "weather-today",
    schema_version = 1,
}

/// `weather-now`: the condition in force and the hour of today it holds from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeatherNow {
    /// The hour of today the condition holds from, 0 … 23: the hour it began, or 0 when it carried
    /// over from yesterday.
    pub hour: u8,
    /// The condition.
    pub condition: Condition,
}

owned_component! {
    component = WeatherNow,
    owner = WeatherSystem,
    component_type = "weather-now",
    schema_version = 1,
}
