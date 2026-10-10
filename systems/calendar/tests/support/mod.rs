//! A square with San Diego's calendar, composed from the real packs — presence and calendar — plus a
//! test-only `placer` that lets an unplaced person arrive, so disclosure can be seen before and after
//! a person has a place. The configuration is decoded and seeded exactly as the World Pack loader
//! does it: `CalendarSystem::decode_configuration` on the YAML stream, then `seed` (`ARC-61`).

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{Attached, ConfigurationContext, EntityClasses, Seeding};
use mineworld_calendar::{CalendarSystem, DayBegan, DaylightChanged};
use mineworld_contracts::{
    Action, ActionTypeId, EntityId, EntityKey, EntityType, Event, EventEnvelope, LocalPosition,
    Location, Millimetres, PersonId, PlaceId, Rejection, SystemId, WorldTime,
};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldRead, WorldView,
};
use mineworld_presence::{Arrived, PerceptionProvider, PresenceSystem, arrival};
use mineworld_sdk::SystemPack;
use serde::{Deserialize, Serialize};

pub const DAY: i64 = 86_400;

/// `worlds/market-town/configure/calendar.yaml`'s text.
pub const SAN_DIEGO: &str =
    "epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 32.7157\nlongitude: -117.1611\n";

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

/// A test-only pack: `place-me` puts its actor in the square, through presence's checked
/// constructor, as any pack that moves people states it.
#[derive(Default)]
pub struct Placer;

impl SystemIdentity for Placer {
    const ID: SystemId = SystemId::from_static("test-placer");
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceMe {
    pub place: PlaceId,
}

impl Action for PlaceMe {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("place-me");
    const OWNER: SystemId = Placer::ID;
}

impl System for Placer {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .providing::<PlaceMe>()
            .emitting::<Arrived>()
    }

    fn validate(
        &self,
        _: &WorldRead<'_>,
        intent: &mineworld_contracts::ActionIntent,
    ) -> Result<(), Rejection> {
        intent
            .payload()
            .payload_for::<PlaceMe>()
            .map(|_| ())
            .map_err(|_| Rejection::PreconditionFailed)
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &mineworld_contracts::ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let wanted: PlaceMe =
            serde_json::from_slice(intent.payload().payload_for::<PlaceMe>()?).expect("decodes");
        let person = PersonId::new(intent.actor(), EntityType::Person)?;
        let fact = arrival(&world.read(), person, spot(wanted.place)).map_err(|reason| {
            KernelError::FactRefusedByOwner {
                system: Placer::ID,
                event_type: Arrived::EVENT_TYPE,
                reason,
            }
        })?;
        Ok(vec![fact])
    }
}

impl PerceptionProvider for Placer {}

fn spot(place: PlaceId) -> Location {
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(1_000),
        Millimetres::new(1_000),
    ))
}

/// presence, calendar (unless left out) and the placer, in that order.
pub fn compose(calendar: bool) -> World {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    if calendar {
        world.install(CalendarSystem).expect("calendar installs");
    }
    world.install(Placer).expect("the placer installs");
    world
}

/// Every provider an observation consults, in composition order.
pub fn providers() -> [&'static dyn PerceptionProvider; 3] {
    [&PresenceSystem, &CalendarSystem, &Placer]
}

/// The square, `ada` in it and `bo` nowhere — entities in key order, as the loader allocates them —
/// and the genesis facts: ada's arrival, then the calendar's configuration from `configuration` (or
/// none), as the loader orders them (locations, then configuration).
pub fn assemble(
    mut world: World,
    configuration: Option<&str>,
) -> (World, BTreeMap<EntityKey, EntityId>, Vec<Emission>) {
    let mut keys = BTreeMap::new();
    for (name, entity_type) in [
        ("ada", EntityType::Person),
        ("bo", EntityType::Person),
        ("square", EntityType::Place),
    ] {
        let id = world
            .create_entity(key(name), entity_type)
            .expect("created");
        keys.insert(key(name), id);
    }
    let square = PlaceId::new(keys[&key("square")], EntityType::Place).expect("a place");
    let ada = PersonId::new(keys[&key("ada")], EntityType::Person).expect("a person");
    let mut facts = vec![arrival(&world.read(), ada, spot(square)).expect("admitted")];
    if let Some(text) = configuration {
        let decoded = serde_saphyr::with_deserializer_from_str(text, |file| {
            CalendarSystem::decode_configuration(file)
        })
        .expect("the configuration decodes");
        let read = world.read();
        let (classes, attached) = (EntityClasses::default(), Attached::none());
        facts.extend(
            decoded
                .seed(
                    &Seeding::new(&read, &keys),
                    &ConfigurationContext::new(&classes, &attached),
                )
                .expect("seeds"),
        );
    }
    (world, keys, facts)
}

/// A world begun at instant 0 from `assemble`.
pub fn begun(
    configuration: Option<&str>,
) -> (World, BTreeMap<EntityKey, EntityId>, Vec<EventEnvelope>) {
    let (mut world, keys, facts) = assemble(compose(true), configuration);
    let genesis = world.genesis(t(0), facts).expect("genesis");
    (world, keys, genesis)
}

/// The facts of type `E` among `facts`, decoded, with their instants.
pub fn of<E: Event + for<'de> Deserialize<'de>>(facts: &[EventEnvelope]) -> Vec<(WorldTime, E)> {
    facts
        .iter()
        .filter(|fact| *fact.event_type() == E::EVENT_TYPE)
        .map(|fact| {
            let bytes = fact.payload().payload_for::<E>().expect("its type");
            (fact.at(), serde_json::from_slice(bytes).expect("decodes"))
        })
        .collect()
}

pub fn days(facts: &[EventEnvelope]) -> Vec<(WorldTime, DayBegan)> {
    of::<DayBegan>(facts)
}

pub fn light(facts: &[EventEnvelope]) -> Vec<(WorldTime, DaylightChanged)> {
    of::<DaylightChanged>(facts)
}
