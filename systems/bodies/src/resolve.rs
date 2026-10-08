//! This pack's answer to presence's question: what an arrival into a shaped place actually achieves
//! (`DECISIONS.md` `ARC-39`).

use mineworld_contracts::SystemId;
use mineworld_kernel::{SystemIdentity, WorldRead};
use mineworld_presence::{ArrivalResolver, Arriving, Resolution};

use crate::system::BodiesSystem;

impl ArrivalResolver for BodiesSystem {
    fn resolver_of(&self) -> SystemId {
        Self::ID
    }

    fn resolve(
        &self,
        _world: &WorldRead<'_>,
        _arriving: &Arriving,
        so_far: Resolution,
    ) -> Resolution {
        so_far
    }
}
