//! The world's calendar as a Process: started once, from the configuration, woken at every light
//! event and every local midnight, never ended.
//!
//! Its state is the fold of this pack's three facts — the configuration, the current day, the
//! current phase — so a snapshot and a replay of the log agree, and a resumed world recomputes
//! nothing (step-19 §5.3).

use mineworld_contracts::ProcessTypeId;
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use crate::day::{CalendarDay, Phase};
use crate::event::CalendarConfigured;
use crate::system::CalendarSystem;

/// The kind of process the world's calendar is, owned — as a type — by [`CalendarSystem`], so no
/// other system can reschedule or end it (`INV-7`).
pub struct CalendarProcess;

impl ProcessKind for CalendarProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("calendar");
    type Owner = CalendarSystem;
}

/// The calendar's state: what was configured, today, and how light it is now.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarState {
    configured: CalendarConfigured,
    day: CalendarDay,
    phase: Phase,
}

impl CalendarState {
    pub(crate) const fn new(
        configured: CalendarConfigured,
        day: CalendarDay,
        phase: Phase,
    ) -> Self {
        Self {
            configured,
            day,
            phase,
        }
    }

    /// The configuration.
    pub const fn configured(&self) -> &CalendarConfigured {
        &self.configured
    }

    /// Today.
    pub const fn day(&self) -> &CalendarDay {
        &self.day
    }

    /// The light now.
    pub const fn phase(&self) -> Phase {
        self.phase
    }

    pub(crate) fn with_day(self, day: CalendarDay, phase: Phase) -> Self {
        Self { day, phase, ..self }
    }

    pub(crate) fn with_phase(self, phase: Phase) -> Self {
        Self { phase, ..self }
    }
}
