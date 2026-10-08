//! The fact this pack records: a place has this shape.

use mineworld_contracts::{Event, EventSchemaVersion, EventTypeId, PlaceId, SystemId, Visibility};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::component::PlaceShape;
use crate::system::BodiesSystem;

/// `place` has the floor and the solids of `shape`.
///
/// Stated at genesis from the place file's `body:` section and reduced by this pack alone into the
/// place's [`PlaceShape`] — after this pack has checked that the people authored into the place fit
/// it (step-11 SD-B4). Static for MVP-0: nothing reshapes a place once it exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceShaped {
    place: PlaceId,
    shape: PlaceShape,
}

impl Event for PlaceShaped {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("place-shaped");
    const OWNER: SystemId = BodiesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PlaceShaped {
    /// The place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Its shape.
    pub const fn shape(&self) -> &PlaceShape {
        &self.shape
    }
}

/// This pack's [`PlaceShaped`] as a fact ready to be recorded at genesis: public, as a place's walls
/// are, about the place.
///
/// Public for the reason movement's `passage` is: a world is assembled as well as run, and the only
/// way to give a place its [`PlaceShape`] is a fact this pack reduces. Whether the people authored
/// into the place fit it is checked when the fact is reduced, by the owner, which refuses rather than
/// writes.
pub fn place_shaped(place: PlaceId, shape: PlaceShape) -> Emission {
    Emission::new::<PlaceShaped>(
        codec::encode(&PlaceShaped { place, shape }),
        Visibility::Public,
    )
    .about(vec![place.entity_id()])
    .at_place(place)
}
