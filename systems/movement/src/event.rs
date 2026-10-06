//! The fact this pack owns: two places open onto each other.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, LocalPosition, PlaceId, SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::system::MovementSystem;

/// Two places open onto each other through one doorway, at `a_at` in `a`'s frame and `b_at` in
/// `b`'s.
///
/// One fact for both directions, because a doorway is one thing: [`MovementSystem`] reduces it into
/// the [`Passages`](crate::Passages) of both places, so the way out and the way back cannot be
/// stated differently.
///
/// Stated at genesis only in this pack's first version. No action opens a passage at runtime; a
/// system that builds or locks doors is a later pack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PassageOpened {
    a: PlaceId,
    a_at: Option<LocalPosition>,
    b: PlaceId,
    b_at: Option<LocalPosition>,
}

impl Event for PassageOpened {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("passage-opened");
    const OWNER: SystemId = MovementSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PassageOpened {
    /// One side of the doorway.
    pub const fn a(&self) -> (PlaceId, Option<LocalPosition>) {
        (self.a, self.a_at)
    }

    /// The other side.
    pub const fn b(&self) -> (PlaceId, Option<LocalPosition>) {
        (self.b, self.b_at)
    }
}

/// This pack's [`PassageOpened`] as a fact ready to be recorded at genesis.
///
/// Public for the reason presence's `arrival` is: a world is assembled as well as run, and the only
/// way to give a place its [`Passages`](crate::Passages) is a fact this pack reduces. The encoding
/// stays this pack's own. Whether the two ends are distinct places of this world is checked when the
/// fact is reduced, by the owner ([`MovementSystem`]), which refuses rather than writes.
pub fn passage(
    a: PlaceId,
    a_at: Option<LocalPosition>,
    b: PlaceId,
    b_at: Option<LocalPosition>,
) -> Emission {
    Emission::new::<PassageOpened>(
        codec::encode(&PassageOpened { a, a_at, b, b_at }),
        Visibility::Public,
    )
    .about(vec![a.entity_id(), b.entity_id()])
    .at_place(a)
}
