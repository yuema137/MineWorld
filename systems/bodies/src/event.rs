//! The facts this pack records (step-11 §4.7, SD-O6): a place has this shape; an object has this
//! shape and lies here; an object moved; somebody shoved somebody.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, ItemId, LocalPosition, Location, PersonId, PlaceId,
    SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::component::{BodyShape, PlaceShape};
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

/// `object` — an Item whose file carries a `body:` section — has this shape and lies on the floor at
/// `lies` (step-11 SD-O5, generation 1).
///
/// Reduced by this pack into the Item's [`BodyShape`]; the same reaction states [`ObjectPlaced`],
/// reduced in the next generation, when every place's shape and every person is in place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BodyFormed {
    object: ItemId,
    shape: BodyShape,
    lies: Location,
}

impl Event for BodyFormed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("body-formed");
    const OWNER: SystemId = BodiesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl BodyFormed {
    /// The object.
    pub const fn object(&self) -> ItemId {
        self.object
    }

    /// Its shape.
    pub const fn shape(&self) -> BodyShape {
        self.shape
    }

    /// The place and the floor point it lies on.
    pub const fn lies(&self) -> Location {
        self.lies
    }
}

/// [`BodyFormed`] as a genesis fact: heard in the place it lies in, about the object.
pub fn body_formed(object: ItemId, shape: BodyShape, lies: Location) -> Emission {
    Emission::new::<BodyFormed>(
        codec::encode(&BodyFormed {
            object,
            shape,
            lies,
        }),
        Visibility::Place(lies.place()),
    )
    .about(vec![object.entity_id()])
    .at_place(lies.place())
}

/// `object` lies in `place` with its centre at `at` (step-11 SD-O5, generation 2).
///
/// Reduced by this pack into the place's [`LooseObjects`](crate::LooseObjects), after the checks
/// SD-O5 lists, in order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectPlaced {
    object: ItemId,
    place: PlaceId,
    at: LocalPosition,
}

impl Event for ObjectPlaced {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("object-placed");
    const OWNER: SystemId = BodiesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ObjectPlaced {
    /// The object.
    pub const fn object(&self) -> ItemId {
        self.object
    }

    /// The place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Where its centre is.
    pub const fn at(&self) -> LocalPosition {
        self.at
    }
}

pub(crate) fn object_placed(object: ItemId, place: PlaceId, at: LocalPosition) -> Emission {
    Emission::new::<ObjectPlaced>(
        codec::encode(&ObjectPlaced { object, place, at }),
        Visibility::Place(place),
    )
    .about(vec![object.entity_id()])
    .at_place(place)
}

/// How an object came to move.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum How {
    /// Somebody's arrival pushed it out of the way (a reaction to their `arrived`).
    Pushed,
    /// Somebody kicked it.
    Kicked,
    /// Somebody picked it up and threw it, in one action.
    Thrown,
}

/// `object`, lying in `place` at `from`, moved to `to` (step-11 SD-O6).
///
/// `path` is presentation data the owner states because only the owner knows it: a keyframe every
/// 0.1 s of a flight, at most 40, the first being `from`; empty for a push, which is one straight
/// line. `to` is the authoritative end; a client may ignore `path` and draw `to`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObjectMoved {
    object: ItemId,
    place: PlaceId,
    from: LocalPosition,
    to: LocalPosition,
    how: How,
    by: PersonId,
    path: Vec<LocalPosition>,
}

impl Event for ObjectMoved {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("object-moved");
    const OWNER: SystemId = BodiesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl ObjectMoved {
    /// The object.
    pub const fn object(&self) -> ItemId {
        self.object
    }

    /// The place it lies in, before and after.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// Where its centre was.
    pub const fn from(&self) -> LocalPosition {
        self.from
    }

    /// Where its centre is now.
    pub const fn to(&self) -> LocalPosition {
        self.to
    }

    /// How it moved.
    pub const fn how(&self) -> How {
        self.how
    }

    /// Who moved it.
    pub const fn by(&self) -> PersonId {
        self.by
    }

    /// The flight's keyframes, for a client to animate; empty for a push.
    pub fn path(&self) -> &[LocalPosition] {
        &self.path
    }
}

/// The parts of an [`ObjectMoved`], as this pack builds one.
pub(crate) struct Movement {
    pub(crate) object: ItemId,
    pub(crate) place: PlaceId,
    pub(crate) from: LocalPosition,
    pub(crate) to: LocalPosition,
    pub(crate) how: How,
    pub(crate) by: PersonId,
    pub(crate) path: Vec<LocalPosition>,
}

/// [`ObjectMoved`] as a fact: heard in the place, about the object and whoever moved it.
pub(crate) fn object_moved(movement: Movement) -> Emission {
    let Movement {
        object,
        place,
        from,
        to,
        how,
        by,
        path,
    } = movement;
    Emission::new::<ObjectMoved>(
        codec::encode(&ObjectMoved {
            object,
            place,
            from,
            to,
            how,
            by,
            path,
        }),
        Visibility::Place(place),
    )
    .about(vec![object.entity_id(), by.entity_id()])
    .with_participants(vec![by.entity_id()])
    .at_place(place)
}

/// `by` shoved `person` (step-11 SD-O6). Reduced into nothing: where the shoved person ended is
/// presence's `arrived`, and where they were asked to go is `stopped-short`'s `wanted`. It exists for
/// biographies, controllers and clients, as `stopped-short` does.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonShoved {
    by: PersonId,
    person: PersonId,
}

impl Event for PersonShoved {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("person-shoved");
    const OWNER: SystemId = BodiesSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PersonShoved {
    /// Who shoved.
    pub const fn by(&self) -> PersonId {
        self.by
    }

    /// Who was shoved.
    pub const fn person(&self) -> PersonId {
        self.person
    }
}

/// [`PersonShoved`] as a fact: heard in the place, about both people.
pub(crate) fn person_shoved(by: PersonId, person: PersonId, place: PlaceId) -> Emission {
    Emission::new::<PersonShoved>(
        codec::encode(&PersonShoved { by, person }),
        Visibility::Place(place),
    )
    .about(vec![by.entity_id(), person.entity_id()])
    .with_participants(vec![by.entity_id(), person.entity_id()])
    .at_place(place)
}
