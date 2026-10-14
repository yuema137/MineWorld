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

use crate::event::HappeningsConfigured;
use crate::system::HappeningsSystem;

/// The kind of process the world's almanac is, owned — as a type — by [`HappeningsSystem`], so no
/// other system can change or end it (`INV-7`).
pub struct AlmanacProcess;

impl ProcessKind for AlmanacProcess {
    const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("almanac");
    type Owner = HappeningsSystem;
}

/// The almanac's state: what was configured.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AlmanacState {
    configured: HappeningsConfigured,
}

impl AlmanacState {
    /// Configured, nothing announced yet.
    pub(crate) const fn new(configured: HappeningsConfigured) -> Self {
        Self { configured }
    }

    /// The configuration.
    pub const fn configured(&self) -> &HappeningsConfigured {
        &self.configured
    }
}
