//! The fact this pack records: somebody is now somewhere.

use mineworld_contracts::{
    EntityId, EntityType, Event, EventSchemaVersion, EventTypeId, LifecycleState, Location,
    PersonId, PlaceId, Rejection, SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity, WorldRead};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::resolve;
use crate::system::PresenceSystem;

/// Somebody is now at a location.
///
/// The whole of what this pack writes to history, and the only thing that changes a
/// [`Presence`](crate::Presence): [`PresenceSystem`] subscribes to its own fact and reduces it into
/// the component (`kernel/src/system.rs` on `subscribing_to`). That is not ceremony — it is what
/// makes the state a projection of the log, so a world rebuilt from its events holds the same
/// positions rather than positions that happen to agree.
///
/// The payload names the person, although the envelope already lists subjects and participants.
/// Deliberately: a reducer that recovered "who arrived" from the position of an entity in a list of
/// participants would be reading a convention, and the first system to state that list differently
/// would break it silently. A typed payload cannot be misread that way.
///
/// The person is a [`PersonId`] rather than an
/// [`EntityId`](mineworld_contracts::EntityId), so a fact about a place or an item cannot be
/// recorded as somebody arriving — and cannot be *read back* as one either, because the reference
/// carries the entity type it was checked against through serialization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Arrived {
    person: PersonId,
    location: Location,
}

impl Event for Arrived {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("arrived");
    const OWNER: SystemId = PresenceSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Arrived {
    /// States that `person` is now at `location`.
    pub const fn new(person: PersonId, location: Location) -> Self {
        Self { person, location }
    }

    /// Who arrived.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where they now are.
    pub const fn location(&self) -> Location {
        self.location
    }
}

/// This pack's [`Arrived`] as a fact ready to be recorded: the payload, the audience, the subject,
/// the participants and the place.
///
/// Public because a world is *assembled* as well as run. A World Pack that wants Alice to start
/// behind the counter needs a `Presence` component, a component is written only by its owner while
/// reducing, and the only fact that writes this one is `Arrived` — so world assembly has to be able
/// to state that fact. [`World::genesis`](mineworld_kernel::World::genesis) records it with
/// [`Causation::WorldGenesis`](mineworld_contracts::Causation), and this pack reduces it through
/// exactly the path a decided arrival takes, which is what makes an authored position and a walked
/// one the same kind of thing in the log.
///
/// It is a function here rather than a payload a loader encodes for itself, because the encoding is
/// this pack's own business: "a payload's format is a contract between the system that declares the
/// event type and whoever reads it back" (`kernel/src/system.rs`). A loader that built these bytes
/// would be a second implementation of this pack's codec, and the first change to the codec would
/// silently make a loaded world unreadable.
///
/// **For placement**: genesis, and any system that puts a person somewhere exactly or not at all. A
/// system that moves people uses [`arrivals`], which records what the arrival actually achieves.
///
/// **Checked, because this pack still decides what its state may hold.** Another system may decide
/// that a person *goes* somewhere; only this pack decides whether [`Presence`](crate::Presence) may
/// *say* so. The constructor therefore asks [`admit`] against the world as it is and refuses before
/// anything is built — there is no unchecked public way to make this fact, so a stating system that
/// follows the contract cannot record one presence would refuse. [`PresenceSystem`] re-asks at
/// reduction, for the system that did not follow it.
///
/// **Resolved, and refused rather than recorded unresolved** (`DECISIONS.md` `ARC-39`). It asks every
/// registered [`ArrivalResolver`](crate::ArrivalResolver) as [`arrivals`] does. When their answer is
/// anything but the placement exactly as asked, it refuses — a placement that would end short or move
/// somebody else cannot be one fact. With no resolver registered, or none that changes it, the fact is
/// exactly the one this function built before resolvers existed.
///
/// # Errors
///
/// [`Rejection::PreconditionFailed`] when [`admit`] refuses; [`Rejection::System`] with code
/// `resolution-refused`, naming the resolver, when a resolver's answer is refused or would change the
/// placement.
pub fn arrival(
    world: &WorldRead<'_>,
    person: PersonId,
    location: Location,
) -> Result<Emission, Rejection> {
    let resolved = resolve::resolve(world, person, location)?;
    if let Some(resolver) = resolved.changed_by {
        return Err(resolve::refused(
            &resolver,
            "would change a placement; a system that moves people states it through arrivals",
        ));
    }
    Ok(arrived(person, location))
}

/// The facts one arrival becomes, **for systems that move people**: what the arrival actually
/// achieves, after every registered [`ArrivalResolver`](crate::ArrivalResolver) has answered
/// (`DECISIONS.md` `ARC-39`).
///
/// ```text
/// 1  admit(person, to)                                   as arrival
/// 2  Arriving { person, from: their Presence, to }
/// 3  fold the registered resolvers in ascending SystemId from Resolution::unchanged, checking
///    the answer after each one; a refused answer names its resolver
/// 4  the facts, in this order:
///      arrived { person, reached }
///      arrived { other, location }               for each person moved, in the resolution's order
///      stopped-short { person, wanted: to, reached, by }        when reached ≠ to
/// ```
///
/// Every fact is stated by the calling system in one list, so each is caused by the same request
/// (`AC-9`). With no resolver registered, or none that changes the arrival, the list is exactly the
/// one fact [`arrival`] builds.
///
/// # Errors
///
/// As [`arrival`], except that an answer which changes the arrival is recorded rather than refused.
pub fn arrivals(
    world: &WorldRead<'_>,
    person: PersonId,
    to: Location,
) -> Result<Vec<Emission>, Rejection> {
    let resolve::Resolved {
        arriving,
        resolution,
        ..
    } = resolve::resolve(world, person, to)?;
    let reached = resolution.reached();
    let mut facts = vec![arrived(arriving.person(), reached)];
    facts.extend(
        resolution
            .displaced()
            .iter()
            .map(|(other, location)| arrived(*other, *location)),
    );
    if reached != to {
        facts.push(stopped_short(StoppedShort {
            person,
            wanted: to,
            reached,
            by: resolution.stopped_by(),
        }));
    }
    Ok(facts)
}

/// The [`Arrived`] fact: heard in the place arrived in, about the person, at that place. The one
/// builder of every `arrived` this pack's constructors return.
fn arrived(person: PersonId, location: Location) -> Emission {
    Emission::new::<Arrived>(
        codec::encode(&Arrived::new(person, location)),
        Visibility::Place(location.place()),
    )
    .about(vec![person.entity_id()])
    .with_participants(vec![person.entity_id()])
    .at_place(location.place())
}

/// Whether [`Presence`](crate::Presence) may take this value: the person is in this world and not
/// destroyed, and the location's place is a place in this world and not destroyed.
///
/// The whole of what this pack refuses about its own state, in one function, asked at three
/// moments: by a deciding system's `validate` before a request is accepted, by [`arrival`] before a
/// fact is built, and by [`PresenceSystem::react`](crate::PresenceSystem) before a fact is reduced.
/// One function, so the three cannot come to disagree. It reads only; a refusal writes nothing.
///
/// Nothing here is about distance or reachability. Whether somebody may *go* somewhere is the
/// deciding system's question (`ARC-26`); this answers only whether the place they end up in is one
/// a presence can name.
///
/// # Errors
///
/// [`Rejection::PreconditionFailed`] when the person or the place is missing, destroyed, or not of
/// its entity type.
pub fn admit(world: &WorldRead<'_>, person: PersonId, location: Location) -> Result<(), Rejection> {
    let living = |entity: EntityId, entity_type: EntityType| {
        world.entity(entity).is_some_and(|record| {
            record.entity_type() == entity_type && record.lifecycle() != LifecycleState::Destroyed
        })
    };
    if !living(person.entity_id(), EntityType::Person) {
        return Err(Rejection::PreconditionFailed);
    }
    if !living(location.place().entity_id(), EntityType::Place) {
        return Err(Rejection::PreconditionFailed);
    }
    Ok(())
}

/// Somebody who was in one place is now in another: an occupancy change.
///
/// This pack's fact, because occupancy — the `present-in` edge — is this pack's state. It is stated
/// by [`PresenceSystem`] while reducing the [`Arrived`] that changed the place, so it is caused by
/// that arrival whoever decided it, and a system that wants to react to people entering a place
/// subscribes here rather than to every arrival and comparing positions itself.
///
/// Not stated for a first placement — a person placed at genesis did not *enter*, they were there —
/// nor for a change of position within one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PersonEnteredPlace {
    person: PersonId,
    place: PlaceId,
    from: PlaceId,
}

impl Event for PersonEnteredPlace {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("person-entered-place");
    const OWNER: SystemId = PresenceSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PersonEnteredPlace {
    /// Who entered.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// The place they entered.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The place they left.
    pub const fn from(&self) -> PlaceId {
        self.from
    }
}

/// Somebody's arrival ended short of where it was asked to go (`DECISIONS.md` `ARC-39`).
///
/// This pack's fact, because where a person ends up is this pack's domain, and it says what is true
/// of the person without naming who moved them or why. It is stated by the system that states the
/// arrival, in the same list, through [`arrivals`] — never by this pack, which therefore does not
/// declare it — and this pack reduces it into nothing: the position is the [`Arrived`] beside it.
/// It exists for biographies, controllers and clients.
///
/// `by` is what the person stopped at, when the resolver that stopped them could name it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoppedShort {
    person: PersonId,
    wanted: Location,
    reached: Location,
    by: Option<EntityId>,
}

impl Event for StoppedShort {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("stopped-short");
    const OWNER: SystemId = PresenceSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl StoppedShort {
    /// Who stopped short.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// Where they were asked to go.
    pub const fn wanted(&self) -> Location {
        self.wanted
    }

    /// Where they ended.
    pub const fn reached(&self) -> Location {
        self.reached
    }

    /// What they stopped at, when it could be named.
    pub const fn by(&self) -> Option<EntityId> {
        self.by
    }
}

/// The [`StoppedShort`] fact: heard in the place reached, about the person, at that place.
fn stopped_short(fact: StoppedShort) -> Emission {
    let place = fact.reached.place();
    Emission::new::<StoppedShort>(codec::encode(&fact), Visibility::Place(place))
        .about(vec![fact.person.entity_id()])
        .with_participants(vec![fact.person.entity_id()])
        .at_place(place)
}

/// The [`PersonEnteredPlace`] fact: heard in the place entered, about the person, at that place.
pub(crate) fn entered_place(person: PersonId, place: PlaceId, from: PlaceId) -> Emission {
    Emission::new::<PersonEnteredPlace>(
        codec::encode(&PersonEnteredPlace {
            person,
            place,
            from,
        }),
        Visibility::Place(place),
    )
    .about(vec![person.entity_id()])
    .with_participants(vec![person.entity_id()])
    .at_place(place)
}
