//! The two records this pack discloses. They are declared component types so that presence keeps
//! them (`owned_by_an_enabled_system`), but no entity carries either: the state lives in the
//! `calendar` Process, and these are how it is shown, on the place an observer is in.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::day::{CalendarDay, Phase};
use crate::system::CalendarSystem;

/// `calendar-day`: today's record (step-19 §5.4).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CalendarDayRecord(pub CalendarDay);

owned_component! {
    component = CalendarDayRecord,
    owner = CalendarSystem,
    component_type = "calendar-day",
    schema_version = 1,
}

/// `calendar-light`: the light now.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarLight {
    /// The phase.
    pub phase: Phase,
}

owned_component! {
    component = CalendarLight,
    owner = CalendarSystem,
    component_type = "calendar-light",
    schema_version = 1,
}
