//! Hand-built worlds for this pack's tests: presence, movement and bodies composed from the real
//! packs, begun by genesis, and driven the way a server drives a world.
//!
//! Every process that uses this registers `[bodies]` as the build's resolvers before it installs the
//! pack (`ARC-39` item 7; Cargo runs each test file as its own process, so the registration is that
//! file's). Genesis states what a World Pack's loader would, in its order: passages, then locations
//! (through presence's `arrival`), then each place's `place-shaped`.
//!
//! The rooms are the prototype's, so its numbers compare (step-11 §17.4):
//!
//! ```text
//! cafe    floor (0, 0)–(8 320, 10 320); the counter (3 860, 6 570)–(8 320, 7 170), 1 100 mm high
//! ```
//!
//! Every expected position in a test is a literal from its own layout (test rules §25).

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_bodies::{BodiesSystem, PlaceShape, place_shaped};
use mineworld_contracts::{
    ActionId, ActionIntent, ActionRecord, ActionResult, EntityId, EntityKey, EntityType,
    EventEnvelope, LocalPosition, Location, Millimetres, Observation, PersonId, PlaceId, WorldTime,
};
use mineworld_kernel::{Emission, KernelError, World};
use mineworld_movement::{Move, MovementSystem, passage};
use mineworld_presence::{
    ArrivalResolver, PerceptionProvider, Presence, PresenceSystem, arrival, register_resolvers,
};
use serde::Serialize;

/// The instant every request in these tests is made at (`AC-12`: supplied, never read from a clock).
pub const NOW: WorldTime = WorldTime::from_seconds(3_600);

/// Registers this build's one resolver; a no-op after the first call in a process.
pub fn register() {
    register_resolvers(vec![Box::new(BodiesSystem) as Box<dyn ArrivalResolver>]);
}

/// The prototype's café room: floor and counter.
pub fn cafe() -> PlaceShape {
    shape(
        (0, 0, 8_320, 10_320),
        &[((3_860, 6_570, 8_320, 7_170), 1_100)],
    )
}

/// A rectangle as literals: `(min.x, min.y, max.x, max.y)`.
pub type Rect = (i32, i32, i32, i32);
/// A point as literals.
pub type Xy = (i32, i32);
/// A person in a plan: key, place key, and a position (or none, for a bodiless presence).
pub type Resident = (&'static str, &'static str, Option<Xy>);
/// One side of a doorway: the place's key and the point.
pub type Side = (&'static str, Xy);

/// A shape from literals: the floor's corners, and each solid's corners and height.
pub fn shape(floor: Rect, solids: &[(Rect, i32)]) -> PlaceShape {
    let corner = |x: i32, y: i32| serde_json::json!({ "x": x, "y": y });
    let (x0, y0, x1, y1) = floor;
    serde_json::from_value(serde_json::json!({
        "floor": { "min": corner(x0, y0), "max": corner(x1, y1) },
        "solids": solids
            .iter()
            .map(|((a, b, c, d), height)| serde_json::json!({
                "min": corner(*a, *b), "max": corner(*c, *d), "height": height
            }))
            .collect::<Vec<_>>(),
    }))
    .expect("a valid shape")
}

/// What a test world is made of.
#[derive(Default)]
pub struct Plan {
    /// Places in identity order, each with a shape or none.
    pub places: Vec<(&'static str, Option<PlaceShape>)>,
    /// People in identity order.
    pub people: Vec<Resident>,
    /// Doorways: (place, here), (place, there).
    pub passages: Vec<(Side, Side)>,
    /// Whether bodies is installed at all.
    pub without_bodies: bool,
}

impl Plan {
    /// One shaped place, `room`, with people at the given positions.
    pub fn room(shape: PlaceShape, people: &[(&'static str, (i32, i32))]) -> Self {
        Self {
            places: vec![("room", Some(shape))],
            people: people
                .iter()
                .map(|(key, at)| (*key, "room", Some(*at)))
                .collect(),
            ..Self::default()
        }
    }
}

pub struct Yard {
    pub world: World,
    pub providers: Vec<Box<dyn PerceptionProvider>>,
    pub places: BTreeMap<&'static str, PlaceId>,
    pub people: BTreeMap<&'static str, EntityId>,
    pub genesis: Vec<EventEnvelope>,
    next_action: u64,
}

impl Yard {
    /// The world `plan` describes, begun — or the kernel's refusal of its genesis.
    pub fn try_new(plan: &Plan) -> Result<Self, KernelError> {
        register();
        let mut world = World::new();
        let mut providers: Vec<Box<dyn PerceptionProvider>> = vec![Box::new(PresenceSystem)];
        world.install(PresenceSystem).expect("presence installs");
        world.install(MovementSystem).expect("movement installs");
        providers.push(Box::new(MovementSystem));
        if !plan.without_bodies {
            world.install(BodiesSystem).expect("bodies installs");
            providers.push(Box::new(BodiesSystem));
        }

        let mut places = BTreeMap::new();
        for (key, _) in &plan.places {
            let id = world
                .create_entity(EntityKey::new(*key).expect("a key"), EntityType::Place)
                .expect("created");
            places.insert(*key, PlaceId::new(id, EntityType::Place).expect("a place"));
        }
        let mut people = BTreeMap::new();
        for (key, _, _) in &plan.people {
            let id = world
                .create_entity(EntityKey::new(*key).expect("a key"), EntityType::Person)
                .expect("created");
            people.insert(*key, id);
        }

        let mut facts: Vec<Emission> = Vec::new();
        for ((a, a_at), (b, b_at)) in &plan.passages {
            facts.push(passage(
                places[a],
                Some(local(*a_at)),
                places[b],
                Some(local(*b_at)),
            ));
        }
        for (key, place, at) in &plan.people {
            let location = match at {
                Some(at) => at_in(places[place], *at),
                None => Location::in_place(places[place]),
            };
            let person = PersonId::new(people[key], EntityType::Person).expect("a person");
            facts.push(arrival(&world.read(), person, location).expect("presence admits it"));
        }
        if !plan.without_bodies {
            for (key, shape) in &plan.places {
                if let Some(shape) = shape {
                    facts.push(place_shaped(places[key], shape.clone()));
                }
            }
        }
        let genesis = world.genesis(NOW, facts)?;
        Ok(Self {
            world,
            providers,
            places,
            people,
            genesis,
            next_action: 1,
        })
    }

    pub fn new(plan: &Plan) -> Self {
        Self::try_new(plan).expect("the yard begins")
    }

    /// The location `(x, y)` on the ground of `place`.
    pub fn at(&self, place: &str, (x, y): (i32, i32)) -> Location {
        at_in(self.places[place], (x, y))
    }

    pub fn next_id(&mut self) -> ActionId {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        id
    }

    /// Asks for `person` to move to `to`: the dispatch's answer and its facts.
    pub fn r#move(&mut self, person: &str, to: Location) -> (ActionResult, Vec<EventEnvelope>) {
        let id = self.next_id();
        let actor = self.people[person];
        let record = ActionRecord::new::<Move>(encode(&Move::new(to)));
        let intent = ActionIntent::new(id, actor, record, NOW);
        let dispatched = self
            .world
            .dispatch(&intent, NOW)
            .expect("dispatch answers rather than failing");
        (dispatched.result().clone(), dispatched.events().to_vec())
    }

    /// Where presence says `person` is, as a point; [`None`] without a position.
    pub fn point(&self, person: &str) -> Option<(i32, i32)> {
        self.world
            .components()
            .get::<Presence>(self.people[person])
            .and_then(|presence| presence.location().local())
            .map(|local| (local.x().value(), local.y().value()))
    }

    /// Where presence says `person` is.
    pub fn location(&self, person: &str) -> Location {
        self.world
            .components()
            .get::<Presence>(self.people[person])
            .map(Presence::location)
            .expect("placed")
    }

    /// The key of an entity of this yard.
    pub fn key_of(&self, entity: EntityId) -> &'static str {
        self.people
            .iter()
            .find(|(_, id)| **id == entity)
            .map(|(key, _)| *key)
            .expect("one of the yard's people")
    }

    /// What this world tells `observer`.
    pub fn observation(&self, observer: &str) -> Observation<serde_json::Value> {
        let providers: Vec<&dyn PerceptionProvider> =
            self.providers.iter().map(AsRef::as_ref).collect();
        mineworld_presence::observe(&self.world, self.people[observer], NOW, &providers)
    }
}

pub fn local((x, y): (i32, i32)) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

pub fn at_in(place: PlaceId, at: (i32, i32)) -> Location {
    Location::in_place(place).with_local(local(at))
}

pub fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

/// The squared distance between two points, exactly.
pub fn distance2(a: (i32, i32), b: (i32, i32)) -> i64 {
    let dx = i64::from(a.0) - i64::from(b.0);
    let dy = i64::from(a.1) - i64::from(b.1);
    dx * dx + dy * dy
}
