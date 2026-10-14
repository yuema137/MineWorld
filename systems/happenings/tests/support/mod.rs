//! A town with San Diego's calendar and happenings of its own, composed from the real packs — presence,
//! calendar and happenings — plus a test-only `placer` that lets an unplaced person arrive (weather's
//! `support/` precedent). The configurations are decoded and seeded exactly as the World Pack loader
//! does it: `decode_configuration` on the YAML stream, then `seed` (`ARC-61`).

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_authoring::{Attached, ConfigurationContext, EntityClasses, Seeding};
use mineworld_calendar::CalendarSystem;
use mineworld_contracts::{
    Action, ActionTypeId, EntityId, EntityKey, EntityType, Event, EventEnvelope, LocalPosition,
    Location, Millimetres, PersonId, PlaceId, Rejection, SystemId, WorldTime,
};
use mineworld_happenings::HappeningsSystem;
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldRead, WorldView,
};
use mineworld_presence::{Arrived, PerceptionProvider, PresenceSystem, arrival};
use mineworld_sdk::SystemPack;
use serde::{Deserialize, Serialize};

pub const DAY: i64 = 86_400;
pub const HOUR: i64 = 3_600;
pub const MINUTE: i64 = 60;

/// `worlds/market-town/configure/calendar.yaml`'s text: epoch 2026-10-08, a Thursday.
pub const SAN_DIEGO: &str =
    "epoch: 2026-10-08\nutc_offset: -08:00\nlatitude: 32.7157\nlongitude: -117.1611\n";

/// A calendar whose epoch is `epoch` (`YYYY-MM-DD`), otherwise San Diego's.
pub fn calendar_from(epoch: &str) -> String {
    SAN_DIEGO.replace("2026-10-08", epoch)
}

/// The town's places, in key order as the loader allocates entities.
pub const PLACES: [&str; 3] = ["park", "square", "street"];

pub fn t(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

pub fn key(name: &str) -> EntityKey {
    EntityKey::new(name).expect("a key")
}

/// A test-only pack: `place-me` puts its actor at a place, through presence's checked constructor, as
/// any pack that moves people states it.
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

/// presence, calendar, happenings (unless left out) and the placer, in that order.
pub fn compose(happenings: bool) -> World {
    let mut world = World::new();
    world.install(PresenceSystem).expect("presence installs");
    world.install(CalendarSystem).expect("calendar installs");
    if happenings {
        world
            .install(HappeningsSystem)
            .expect("happenings installs");
    }
    world.install(Placer).expect("the placer installs");
    world
}

/// Every provider an observation consults, in composition order.
pub fn providers() -> [&'static dyn PerceptionProvider; 4] {
    [&PresenceSystem, &CalendarSystem, &HappeningsSystem, &Placer]
}

pub fn decoded<P: SystemPack>(
    text: &str,
) -> std::sync::Arc<dyn mineworld_authoring::AuthoredConfiguration> {
    serde_saphyr::with_deserializer_from_str(text, |file| P::decode_configuration(file))
        .expect("the configuration decodes")
}

/// The town — `ada` on the street, `bo` nowhere, the park, the square and the street — entities in key
/// order, as the loader allocates them, and the genesis facts: ada's arrival, then the calendar's (with
/// epoch `calendar`, if given) and the happenings' configurations (if given), in that order, as the
/// loader orders them (locations, then `configure:`).
pub fn assemble(
    mut world: World,
    calendar: Option<&str>,
    happenings: Option<&str>,
) -> (World, BTreeMap<EntityKey, EntityId>, Vec<Emission>) {
    let mut keys = BTreeMap::new();
    let mut entities = vec![("ada", EntityType::Person), ("bo", EntityType::Person)];
    entities.extend(PLACES.iter().map(|place| (*place, EntityType::Place)));
    for (name, entity_type) in entities {
        let id = world
            .create_entity(key(name), entity_type)
            .expect("created");
        keys.insert(key(name), id);
    }
    let street = place(&keys, "street");
    let ada = PersonId::new(keys[&key("ada")], EntityType::Person).expect("a person");
    let mut facts = vec![arrival(&world.read(), ada, spot(street)).expect("admitted")];
    let mut configurations = Vec::new();
    if let Some(text) = calendar {
        configurations.push(decoded::<CalendarSystem>(text));
    }
    if let Some(text) = happenings {
        configurations.push(decoded::<HappeningsSystem>(text));
    }
    let (classes, attached) = (EntityClasses::default(), Attached::none());
    for configuration in configurations {
        let read = world.read();
        facts.extend(
            configuration
                .seed(
                    &Seeding::new(&read, &keys),
                    &ConfigurationContext::new(&classes, &attached),
                )
                .expect("seeds"),
        );
    }
    (world, keys, facts)
}

/// The place with this key.
pub fn place(keys: &BTreeMap<EntityKey, EntityId>, name: &str) -> PlaceId {
    PlaceId::new(keys[&key(name)], EntityType::Place).expect("a place")
}

/// A world with San Diego's calendar and the happenings of `happenings` (or none configured), begun at
/// instant 0.
pub fn begun(
    happenings: Option<&str>,
) -> (World, BTreeMap<EntityKey, EntityId>, Vec<EventEnvelope>) {
    begun_on(SAN_DIEGO, happenings)
}

/// As [`begun`], with the calendar text given.
pub fn begun_on(
    calendar: &str,
    happenings: Option<&str>,
) -> (World, BTreeMap<EntityKey, EntityId>, Vec<EventEnvelope>) {
    let (mut world, keys, facts) = assemble(compose(true), Some(calendar), happenings);
    let genesis = world.genesis(t(0), facts).expect("genesis");
    (world, keys, genesis)
}

/// Genesis and every fact through instant `end`.
pub fn run(
    happenings: Option<&str>,
    end: i64,
) -> (World, BTreeMap<EntityKey, EntityId>, Vec<EventEnvelope>) {
    run_on(SAN_DIEGO, happenings, end)
}

/// As [`run`], with the calendar text given.
pub fn run_on(
    calendar: &str,
    happenings: Option<&str>,
    end: i64,
) -> (World, BTreeMap<EntityKey, EntityId>, Vec<EventEnvelope>) {
    let (mut world, keys, mut facts) = begun_on(calendar, happenings);
    facts.extend(world.advance_to(t(end)).expect("advances").into_events());
    (world, keys, facts)
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

/// Whether a fact is one of this pack's.
pub fn ours(fact: &EventEnvelope) -> bool {
    let kind = fact.event_type().as_str();
    kind.starts_with("happening")
}
