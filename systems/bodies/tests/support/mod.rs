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

use mineworld_bodies::{
    BodiesSystem, BodyShape, LooseObjects, ObjectMoved, PlaceShape, body_formed, place_shaped,
};
use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionRequest, ActionResult, ActionTypeId,
    Causation, EntityId, EntityKey, EntityType, EventEnvelope, ItemId, LocalPosition, Location,
    Millimetres, Observation, PersonId, PlaceId, SystemId, Visibility, WorldTime,
};
use mineworld_item::{Category, ItemKindDeclared, ItemSystem};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldView,
};
use mineworld_movement::{Move, MovementSystem, passage};
use mineworld_presence::{
    ArrivalResolver, Arrived, PerceptionProvider, PersonEnteredPlace, Presence, PresenceSystem,
    StoppedShort, arrival, register_resolvers,
};
use serde::{Deserialize, Serialize};

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
    /// Whether the test-only bypassing pack is installed (PB-17).
    pub with_bypass: bool,
    /// Loose objects (12c), in identity order, created after the people: each an Item whose
    /// `body-formed` genesis states, as an item file's `body:` section would.
    pub objects: Vec<Thing>,
    /// Items that are declared kinds (the `item` pack is then installed, before bodies): each one of
    /// `objects`' keys, or a plain kind of its own, created after the objects.
    pub kinds: Vec<&'static str>,
}

/// A loose object in a plan: its key, its shape, the place's key and the floor point it lies on.
pub type Thing = (&'static str, BodyShape, &'static str, Xy);

/// A ball of radius `r` mm.
pub fn ball(r: i32) -> BodyShape {
    serde_json::from_value(serde_json::json!({ "ball": r })).expect("a ball")
}

/// A box of half-extents `half` mm on every axis.
pub fn cube(half: i32) -> BodyShape {
    serde_json::from_value(serde_json::json!({ "box": { "x": half, "y": half, "z": half } }))
        .expect("a box")
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
    /// The plan's items — its objects and its plain kinds — by key.
    pub items: BTreeMap<&'static str, ItemId>,
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
        if !plan.kinds.is_empty() {
            world.install(ItemSystem).expect("item installs");
        }
        if !plan.without_bodies {
            world.install(BodiesSystem).expect("bodies installs");
            providers.push(Box::new(BodiesSystem));
        }
        if plan.with_bypass {
            world.install(Bypass).expect("the bypassing pack installs");
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
        let mut items = BTreeMap::new();
        let plain = plan
            .kinds
            .iter()
            .filter(|kind| !plan.objects.iter().any(|(key, ..)| key == *kind));
        for key in plan.objects.iter().map(|(key, ..)| key).chain(plain) {
            let id = world
                .create_entity(EntityKey::new(*key).expect("a key"), EntityType::Item)
                .expect("created");
            items.insert(*key, ItemId::new(id, EntityType::Item).expect("an item"));
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
        // Items' sections before places', each item's in composition order (item, then bodies), as
        // the loader seeds them (ARC-36 item 7).
        for (key, item) in &items {
            if plan.kinds.contains(key) {
                let category = Category::new("toy").expect("a category");
                facts.push(ItemKindDeclared::new(*item, category).emission());
            }
            if let Some((_, shape, place, at)) = plan.objects.iter().find(|(k, ..)| k == key)
                && !plan.without_bodies
            {
                facts.push(body_formed(*item, *shape, at_in(places[place], *at)));
            }
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
            items,
            genesis,
            next_action: 1,
        })
    }

    /// Where each object lying in `place` is, by key, in identity order: `(x, y, z)`.
    pub fn objects(&self, place: &str) -> Vec<(&'static str, (i32, i32, i32))> {
        let Some(row) = self
            .world
            .components()
            .get::<LooseObjects>(self.places[place].entity_id())
        else {
            return Vec::new();
        };
        row.objects()
            .iter()
            .map(|lying| {
                let key = self
                    .items
                    .iter()
                    .find(|(_, id)| **id == lying.object())
                    .map(|(key, _)| *key)
                    .expect("one of the yard's items");
                let at = lying.at();
                (key, (at.x().value(), at.y().value(), at.z().value()))
            })
            .collect()
    }

    /// Where the object `key` lies, `(x, y, z)`.
    pub fn object(&self, key: &str) -> (i32, i32, i32) {
        self.objects_everywhere()
            .into_iter()
            .find(|(k, _)| *k == key)
            .map(|(_, at)| at)
            .expect("the object lies somewhere")
    }

    fn objects_everywhere(&self) -> Vec<(&'static str, (i32, i32, i32))> {
        self.places
            .keys()
            .flat_map(|place| self.objects(place))
            .collect()
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
        let moved = self.walk(person, to);
        (moved.result, moved.events)
    }

    /// Asks for `person` to move to `to`, keeping the request's identity with its answer.
    pub fn walk(&mut self, person: &str, to: Location) -> Moved {
        let record = ActionRecord::new::<Move>(encode(&Move::new(to)));
        self.submit(person, record)
    }

    /// Submits a request built elsewhere — from a complete affordance (`ARC-34`).
    pub fn submit_request(&mut self, request: ActionRequest<Vec<u8>>) -> Moved {
        let id = self.next_id();
        let intent = ActionIntent::allocate(request, id, NOW);
        let dispatched = self
            .world
            .dispatch(&intent, NOW)
            .expect("dispatch answers rather than failing");
        Moved {
            id,
            result: dispatched.result().clone(),
            events: dispatched.events().to_vec(),
        }
    }

    /// Submits `record` as `person`'s request, aimed at `target`.
    pub fn submit_at(&mut self, person: &str, record: ActionRecord, target: EntityId) -> Moved {
        let id = self.next_id();
        let actor = self.people[person];
        let intent = ActionIntent::new(id, actor, record, NOW).with_target(target);
        let dispatched = self
            .world
            .dispatch(&intent, NOW)
            .expect("dispatch answers rather than failing");
        Moved {
            id,
            result: dispatched.result().clone(),
            events: dispatched.events().to_vec(),
        }
    }

    /// Submits `record` as `person`'s request.
    pub fn submit(&mut self, person: &str, record: ActionRecord) -> Moved {
        let id = self.next_id();
        let actor = self.people[person];
        let intent = ActionIntent::new(id, actor, record, NOW);
        let dispatched = self
            .world
            .dispatch(&intent, NOW)
            .expect("dispatch answers rather than failing");
        Moved {
            id,
            result: dispatched.result().clone(),
            events: dispatched.events().to_vec(),
        }
    }

    /// Every person standing in `place` with a position, by key, in identity order.
    pub fn standing(&self, place: &str) -> Vec<(&'static str, (i32, i32))> {
        self.people
            .iter()
            .filter(|(_, id)| {
                self.world
                    .components()
                    .get::<Presence>(**id)
                    .is_some_and(|presence| presence.location().place() == self.places[place])
            })
            .filter_map(|(key, _)| self.point(key).map(|at| (*key, at)))
            .collect()
    }

    /// The closest pair standing in `place`, by key, and their distance squared.
    pub fn closest(&self, place: &str) -> Option<(&'static str, &'static str, i64)> {
        let standing = self.standing(place);
        let mut best: Option<(&'static str, &'static str, i64)> = None;
        for (i, (a, at_a)) in standing.iter().enumerate() {
            for (b, at_b) in &standing[i + 1..] {
                let d = distance2(*at_a, *at_b);
                if best.is_none_or(|(_, _, closest)| d < closest) {
                    best = Some((a, b, d));
                }
            }
        }
        best
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

/// One request and what it became.
pub struct Moved {
    pub id: ActionId,
    pub result: ActionResult,
    pub events: Vec<EventEnvelope>,
}

/// A recorded fact, decoded with its owner's published types.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fact {
    Arrived(EntityId, Location),
    StoppedShort {
        person: EntityId,
        wanted: Location,
        reached: Location,
        by: Option<EntityId>,
    },
    Entered(EntityId),
    Other(String),
}

pub fn fact(event: &EventEnvelope) -> Fact {
    let record = event.payload();
    if let Ok(payload) = record.payload_for::<Arrived>() {
        let arrived: Arrived = serde_json::from_slice(payload).expect("presence's encoding");
        return Fact::Arrived(arrived.person().entity_id(), arrived.location());
    }
    if let Ok(payload) = record.payload_for::<StoppedShort>() {
        let stopped: StoppedShort = serde_json::from_slice(payload).expect("presence's encoding");
        return Fact::StoppedShort {
            person: stopped.person().entity_id(),
            wanted: stopped.wanted(),
            reached: stopped.reached(),
            by: stopped.by(),
        };
    }
    if let Ok(payload) = record.payload_for::<PersonEnteredPlace>() {
        let entered: PersonEnteredPlace =
            serde_json::from_slice(payload).expect("presence's encoding");
        return Fact::Entered(entered.person().entity_id());
    }
    Fact::Other(event.event_type().as_str().to_owned())
}

impl Moved {
    pub fn accepted(&self) -> bool {
        matches!(self.result, ActionResult::Accepted { .. })
    }

    pub fn facts(&self) -> Vec<Fact> {
        self.events.iter().map(fact).collect()
    }

    /// Every fact but presence's own occupancy change is the request's, stated by movement (`AC-9`).
    pub fn assert_caused_by_the_request(&self) {
        for event in &self.events {
            if event.event_type().as_str() == "person-entered-place" {
                continue;
            }
            assert_eq!(
                *event.caused_by(),
                Causation::Action(self.id),
                "caused by the request: {:?}",
                fact(event)
            );
            assert_eq!(
                event.provenance().emitted_by().as_str(),
                "movement",
                "stated by movement"
            );
            assert_eq!(
                event.provenance().controller_decision(),
                Some(self.id),
                "the controller's decision"
            );
        }
    }

    /// The people this request moved besides the walker: every `arrived` after the first.
    pub fn displaced(&self) -> Vec<(EntityId, Location)> {
        self.facts()
            .into_iter()
            .skip(1)
            .filter_map(|fact| match fact {
                Fact::Arrived(person, at) => Some((person, at)),
                _ => None,
            })
            .collect()
    }
}

/// The ground point of a location.
pub fn xy(location: Location) -> (i32, i32) {
    let local = location.local().expect("a position");
    (local.x().value(), local.y().value())
}

// ---------------------------------------------------------------------------------------------
// test-bypass (PB-17): a stating system that encodes presence's `arrived` itself, bypassing the
// constructor that resolves it (ARC-39's accepted limitation, step-11 F-R11)
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Bypass;

impl SystemIdentity for Bypass {
    const ID: SystemId = SystemId::from_static("test-bypass");
}

/// Put `person` at `to`, unresolved.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Put {
    pub person: PersonId,
    pub to: Location,
}

impl Action for Put {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("test-put");
    const OWNER: SystemId = Bypass::ID;
}

impl System for Bypass {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .providing::<Put>()
            .emitting::<Arrived>()
    }

    fn resolve(
        &self,
        _world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let put: Put = serde_json::from_slice(intent.payload().payload()).expect("a test-put");
        Ok(vec![
            Emission::new::<Arrived>(
                encode(&Arrived::new(put.person, put.to)),
                Visibility::Place(put.to.place()),
            )
            .about(vec![put.person.entity_id()])
            .at_place(put.to.place()),
        ])
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

/// A recorded `object-moved`, decoded with its owner's published type.
pub fn object_moved(event: &EventEnvelope) -> Option<ObjectMoved> {
    let payload = event.payload().payload_for::<ObjectMoved>().ok()?;
    Some(serde_json::from_slice(payload).expect("bodies' encoding"))
}

/// The squared distance from `p` to the rectangle `rect`: zero inside it.
fn to_rect2(p: Xy, (x0, y0, x1, y1): Rect) -> i64 {
    let dx = i64::from((x0 - p.0).max(p.0 - x1).max(0));
    let dy = i64::from((y0 - p.1).max(p.1 - y1).max(0));
    dx * dx + dy * dy
}

/// Every invariant the pack keeps in `place`, checked from the test's own literals for the room —
/// `floor` and `solids` — and from the shapes and positions the world holds (step-11 V1–V4, SD-O2):
/// people 595 mm apart, inside the floor shrunk by 295 mm, 295 mm from every solid; every object
/// within the floor, resting on it or on a solid's top (± 5 mm), clear of the other solids, of the
/// other objects (by more than 5 mm) and of every person's disc (295 mm from its footprint).
pub fn assert_holds(yard: &Yard, place: &str, floor: Rect, solids: &[(Rect, i32)]) {
    let people = yard.standing(place);
    for (i, (a, at_a)) in people.iter().enumerate() {
        for (b, at_b) in &people[i + 1..] {
            let d = distance2(*at_a, *at_b);
            assert!(d >= 595 * 595, "{a} and {b} {} mm apart", d.isqrt());
        }
        let (x0, y0, x1, y1) = floor;
        assert!(
            (x0 + 295..=x1 - 295).contains(&at_a.0) && (y0 + 295..=y1 - 295).contains(&at_a.1),
            "{a} at {at_a:?} is inside the floor"
        );
        for (rect, _) in solids {
            assert!(
                to_rect2(*at_a, *rect) >= 295 * 295,
                "{a} at {at_a:?} clear of {rect:?}"
            );
        }
    }
    let objects = yard.objects(place);
    let outline = |key: &str| -> (bool, i32, i32, i32) {
        let shape = *yard
            .world
            .components()
            .get::<BodyShape>(yard.items[key].entity_id())
            .expect("a shape");
        match shape {
            BodyShape::Box(half) => (true, half.x().value(), half.y().value(), half.z().value()),
            BodyShape::Ball(r) => (false, r.value(), r.value(), r.value()),
        }
    };
    for (i, (key, (x, y, z))) in objects.iter().enumerate() {
        let (boxy, hx, hy, hz) = outline(key);
        let (x0, y0, x1, y1) = floor;
        assert!(
            x - hx >= x0 && x + hx <= x1 && y - hy >= y0 && y + hy <= y1,
            "{key} at ({x}, {y}) within the floor"
        );
        let rests_on_floor = (z - hz).abs() <= 5;
        let rests_on = solids.iter().position(|((a, b, c, d), height)| {
            (z - (height + hz)).abs() <= 5
                && x - hx >= *a
                && x + hx <= *c
                && y - hy >= *b
                && y + hy <= *d
        });
        assert!(rests_on_floor || rests_on.is_some(), "{key} at z {z} rests");
        for (index, ((a, b, c, d), _)) in solids.iter().enumerate() {
            if Some(index) == rests_on {
                continue;
            }
            let meets = if boxy {
                x - hx < *c && x + hx > *a && y - hy < *d && y + hy > *b
            } else {
                to_rect2((*x, *y), (*a, *b, *c, *d)) < i64::from(hx).pow(2)
            };
            assert!(!meets, "{key} at ({x}, {y}) clear of solid {index}");
        }
        for (person, at) in &people {
            let reach = if boxy {
                to_rect2(*at, (x - hx, y - hy, x + hx, y + hy)) >= 295 * 295
            } else {
                distance2(*at, (*x, *y)) >= i64::from(295 + hx).pow(2)
            };
            assert!(
                reach,
                "{person} at {at:?} keeps 295 mm from {key} at ({x}, {y})"
            );
        }
        for (other, (ox, oy, _)) in &objects[i + 1..] {
            let (other_boxy, ohx, ohy, _) = outline(other);
            let apart = match (boxy, other_boxy) {
                (true, true) => hx + ohx - (x - ox).abs() <= 5 || hy + ohy - (y - oy).abs() <= 5,
                (false, false) => distance2((*x, *y), (*ox, *oy)) >= i64::from(hx + ohx - 5).pow(2),
                (false, true) => {
                    to_rect2((*x, *y), (ox - ohx, oy - ohy, ox + ohx, oy + ohy))
                        >= i64::from(hx - 5).pow(2)
                }
                (true, false) => {
                    to_rect2((*ox, *oy), (x - hx, y - hy, x + hx, y + hy))
                        >= i64::from(ohx - 5).pow(2)
                }
            };
            assert!(apart, "{key} and {other} do not overlap");
        }
    }
}

/// The squared distance between two points, exactly.
pub fn distance2(a: (i32, i32), b: (i32, i32)) -> i64 {
    let dx = i64::from(a.0) - i64::from(b.0);
    let dy = i64::from(a.1) - i64::from(b.1);
    dx * dx + dy * dy
}
