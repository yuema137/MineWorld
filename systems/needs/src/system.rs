//! The installable system.

use mineworld_contracts::SystemId;
use mineworld_kernel::SystemIdentity;

/// Hunger and tiredness.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct NeedsSystem;

impl SystemIdentity for NeedsSystem {
    const ID: SystemId = SystemId::from_static("needs");
}
