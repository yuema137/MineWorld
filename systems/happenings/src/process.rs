//! The pack's Processes (design §5.4, F-WE-a-6): the happenings live here, not in a component on the
//! world's places, so a place nothing happens at is never written and every happening has one owner
//! record.
//!
//! The `almanac` — one per configured world, open-ended, never woken — holds the configuration and the
//! happenings announced and not yet ended. Its state is the fold of this pack's own facts, so a
//! snapshot and a replay of the log agree, and a resumed world continues the same happenings.

use mineworld_contracts::ProcessTypeId;
use mineworld_kernel::ProcessKind;
use serde::{Deserialize, Serialize};

use mineworld_contracts::WorldTime;

use crate::event::HappeningsConfigured;
use crate::host::{HappeningRef, OpenHappening};
use crate::system::HappeningsSystem;

/// The kind of process the world's almanac is, owned — as a type — by [`HappeningsSystem`], so no
/// other system can change or end it (`INV-7`).
///
/// Its start instant also says which pass was the world's first: the day it was started on is the day
/// whose `day-began` runs the catch-up (design §5.3), and the kernel keeps that instant in every
/// snapshot, so the catch-up runs once per world, across resume, without a flag to fold.
pub struct AlmanacProcess;

impl ProcessKind for AlmanacProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("almanac");
    type Owner = HappeningsSystem;
}

/// The almanac's state: what was configured, and every happening announced and not yet ended, in
/// identity order. Bounded: at most `per_day` begin on each day, a happening is announced at most 30
/// days ahead and runs at most 14, so at most `per_day × (31 + 14)` are open.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlmanacState {
    configured: HappeningsConfigured,
    open: Vec<OpenHappening>,
}

impl AlmanacState {
    /// Configured, nothing announced yet.
    pub(crate) const fn new(configured: HappeningsConfigured) -> Self {
        Self {
            configured,
            open: Vec::new(),
        }
    }

    /// The configuration.
    pub const fn configured(&self) -> &HappeningsConfigured {
        &self.configured
    }

    /// The happenings announced and not yet ended, in identity order.
    pub fn open(&self) -> &[OpenHappening] {
        &self.open
    }

    /// `happening-announced` folded: one more open happening.
    pub(crate) fn with_announced(mut self, open: OpenHappening) -> Self {
        if let Err(at) = self.open.binary_search(&open) {
            self.open.insert(at, open);
        }
        self
    }

    /// `happening-ended` folded: that happening is no longer open.
    pub(crate) fn with_ended(mut self, happening: &HappeningRef) -> Self {
        self.open.retain(|open| open.happening() != happening);
        self
    }
}

/// The kind of process one happening is: started when it is announced, at its place, woken when it
/// begins and when it ends.
pub struct HappeningProcess;

impl ProcessKind for HappeningProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("happening");
    type Owner = HappeningsSystem;
}

/// Where a happening is in its life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum HappeningPhase {
    /// Announced, not yet begun.
    Announced,
    /// Begun, not yet ended.
    Running,
}

/// One happening's state: which it is, when it ends, and its phase — the fold of its own facts.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningState {
    happening: HappeningRef,
    until: WorldTime,
    phase: HappeningPhase,
}

impl HappeningState {
    /// Announced.
    pub(crate) const fn announced(happening: HappeningRef, until: WorldTime) -> Self {
        Self {
            happening,
            until,
            phase: HappeningPhase::Announced,
        }
    }

    /// The same happening, begun.
    pub(crate) fn running(self) -> Self {
        Self {
            phase: HappeningPhase::Running,
            ..self
        }
    }

    /// The happening.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }

    /// The instant it ends.
    pub const fn until(&self) -> WorldTime {
        self.until
    }

    /// Its phase.
    pub const fn phase(&self) -> HappeningPhase {
        self.phase
    }
}
