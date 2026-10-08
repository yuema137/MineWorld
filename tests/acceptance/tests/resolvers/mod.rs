//! The synthetic packs that prove presence's arrival-resolver seam, and the small world they share
//! (step-11 §16, SD-R11; `docs/DECISIONS.md` `ARC-39`).
//!
//! Defined only here, in test files: no library is compiled against them, and no installed set lists
//! them. Each resolver owns one component on a `Place`, written by reducing its own genesis fact, so
//! it is inert wherever that state is absent:
//!
//! ```text
//! test-fences   Fence { x }   an arrival from west of the line to x ≥ X stops at X − 10, and every
//!                             other person within 300 mm of the stop on both axes, in EntityId order,
//!                             is displaced 100 mm along +x
//! test-grid     Grid { step } an arrival eastward within the place snaps back to the largest multiple
//!                             of step that is not west of where it started
//! test-rogue    Rogue(…)      one deliberate breach of presence's rules per mode, or none
//! test-echo     (no resolver) hears every arrived; when its trigger arrives, states arrivals() for
//!                             its follower 600 mm east of the trigger
//! test-placer   (no resolver) provides test-place { person, to } and states arrivals()
//! ```
//!
//! The world: two places, `yard` and `lane`, opening onto each other through a doorway at yard
//! (9 000, 2 000) and lane (0, 2 000); people placed by genesis through presence's `arrival`.
//! Genesis builds every fact before it applies any, the fence's own included, so a person can be
//! placed east of a fence (step-11 F-R4).
//!
//! Every position is a literal from a test's layout, in millimetres.

#![allow(dead_code)]

use std::collections::BTreeMap;

use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionTypeId, EntityId, EntityKey, EntityType,
    Event, EventEnvelope, EventSchemaVersion, EventTypeId, LocalPosition, Location, Millidegrees,
    Millimetres, Orientation, PersonId, PlaceId, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Declarations, Dispatched, Emission, KernelError, System, SystemDeclaration, SystemIdentity,
    SystemVersion, World, WorldRead, WorldView, owned_component,
};
use mineworld_movement::{Move, MovementSystem, passage};
use mineworld_presence::{
    ArrivalResolver, Arrived, Arriving, Presence, PresenceSystem, Resolution, StoppedShort,
    arrival, arrivals, register_resolvers, require_registered,
};
use serde::{Deserialize, Serialize};

fn encode<T: Serialize>(value: &T) -> Vec<u8> {
    serde_json::to_vec(value).expect("a payload encodes")
}

fn decode<T: for<'de> Deserialize<'de>>(event: &EventEnvelope) -> T {
    serde_json::from_slice(event.payload().payload()).expect("a payload decodes")
}

/// A location in `place` at (`x`, `y`) on the ground.
pub fn at(place: PlaceId, x: i32, y: i32) -> Location {
    Location::in_place(place).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

/// `location` moved to `x`, keeping its place, `y`, `z` and facing.
fn with_x(location: Location, x: i32) -> Location {
    let local = location.local().expect("a local position");
    let mut moved = Location::in_place(location.place()).with_local(LocalPosition::new(
        Millimetres::new(x),
        local.y(),
        local.z(),
    ));
    if let Some(facing) = location.facing() {
        moved = moved.with_facing(facing);
    }
    moved
}

fn x_of(location: Location) -> Option<i32> {
    location.local().map(|local| local.x().value())
}

/// The ids of the three resolvers, which is the order they are asked in.
pub const FENCES: SystemId = SystemId::from_static("test-fences");
pub const GRID: SystemId = SystemId::from_static("test-grid");
pub const ROGUE: SystemId = SystemId::from_static("test-rogue");

/// Registers `[grid, rogue, fences]` — deliberately not in id order — for a process whose tests need
/// resolvers. A no-op after the first call.
pub fn register_all() {
    register_resolvers(vec![
        Box::new(Grid) as Box<dyn ArrivalResolver>,
        Box::new(Rogue),
        Box::new(Fences),
    ]);
}

// ---------------------------------------------------------------------------------------------
// test-fences
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Fences;

impl SystemIdentity for Fences {
    const ID: SystemId = FENCES;
}

/// A line across a place at `x`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fence {
    x: i32,
}

owned_component! {
    component = Fence,
    owner = Fences,
    component_type = "test-fence",
    schema_version = 1,
}

/// Genesis: this place has a fence at `x`.
#[derive(Serialize, Deserialize)]
struct FenceRaised {
    place: PlaceId,
    x: i32,
}

impl Event for FenceRaised {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("test-fence-raised");
    const OWNER: SystemId = FENCES;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Fences {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<Fence>()
            .emitting::<FenceRaised>()
            .subscribing_to::<FenceRaised>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        require_registered(&Self::ID);
        tables.component::<Fence>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == FenceRaised::EVENT_TYPE {
            let raised: FenceRaised = decode(event);
            world.insert(raised.place.entity_id(), Fence { x: raised.x })?;
        }
        Ok(Vec::new())
    }
}

impl ArrivalResolver for Fences {
    fn resolver_of(&self) -> SystemId {
        FENCES
    }

    fn resolve(
        &self,
        world: &WorldRead<'_>,
        arriving: &Arriving,
        so_far: Resolution,
    ) -> Resolution {
        let reached = so_far.reached();
        let place = reached.place();
        let Some(fence) = world.component::<Fence>(place.entity_id()) else {
            return so_far;
        };
        let Some(to_x) = x_of(reached) else {
            return so_far;
        };
        let from_west = match arriving.from() {
            Some(from) if from.place() == place => x_of(from).is_some_and(|x| x < fence.x),
            _ => true,
        };
        if !from_west || to_x < fence.x {
            return so_far;
        }
        let stop = with_x(reached, fence.x - 10);
        let local = stop.local().expect("a local position");
        let mut others: Vec<(EntityId, Location)> = world
            .components::<Presence>()
            .map(|(entity, presence)| (entity, presence.location()))
            .filter(|(entity, location)| {
                *entity != arriving.person().entity_id() && location.place() == place
            })
            .collect();
        others.sort_by_key(|(entity, _)| *entity);
        let mut resolution = so_far.stopped_at(stop, None);
        for (entity, location) in others {
            let Some(there) = location.local() else {
                continue;
            };
            let near = (there.x().value() - local.x().value()).abs() <= 300
                && (there.y().value() - local.y().value()).abs() <= 300;
            if near {
                let person = PersonId::new(entity, EntityType::Person).expect("a person");
                resolution =
                    resolution.displacing(person, with_x(location, there.x().value() + 100));
            }
        }
        resolution
    }
}

/// The genesis fact that raises a fence at `x` across `place`.
pub fn fence(place: PlaceId, x: i32) -> Emission {
    Emission::new::<FenceRaised>(encode(&FenceRaised { place, x }), Visibility::Public)
}

// ---------------------------------------------------------------------------------------------
// test-grid
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Grid;

impl SystemIdentity for Grid {
    const ID: SystemId = GRID;
}

/// Positions eastward snap back to multiples of `step`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GridStep {
    step: i32,
}

owned_component! {
    component = GridStep,
    owner = Grid,
    component_type = "test-grid",
    schema_version = 1,
}

#[derive(Serialize, Deserialize)]
struct GridLaid {
    place: PlaceId,
    step: i32,
}

impl Event for GridLaid {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("test-grid-laid");
    const OWNER: SystemId = GRID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Grid {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<GridStep>()
            .emitting::<GridLaid>()
            .subscribing_to::<GridLaid>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        require_registered(&Self::ID);
        tables.component::<GridStep>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == GridLaid::EVENT_TYPE {
            let laid: GridLaid = decode(event);
            world.insert(laid.place.entity_id(), GridStep { step: laid.step })?;
        }
        Ok(Vec::new())
    }
}

impl ArrivalResolver for Grid {
    fn resolver_of(&self) -> SystemId {
        GRID
    }

    fn resolve(
        &self,
        world: &WorldRead<'_>,
        arriving: &Arriving,
        so_far: Resolution,
    ) -> Resolution {
        let reached = so_far.reached();
        let Some(grid) = world.component::<GridStep>(reached.place().entity_id()) else {
            return so_far;
        };
        let Some(from_x) = arriving
            .from()
            .filter(|from| from.place() == reached.place())
            .and_then(x_of)
        else {
            return so_far;
        };
        let Some(to_x) = x_of(reached) else {
            return so_far;
        };
        if to_x <= from_x {
            return so_far;
        }
        let snapped = to_x.div_euclid(grid.step) * grid.step;
        if snapped < from_x || snapped == to_x {
            return so_far;
        }
        let by = so_far.stopped_by();
        so_far.stopped_at(with_x(reached, snapped), by)
    }
}

/// The genesis fact that lays a grid of `step` over `place`.
pub fn grid(place: PlaceId, step: i32) -> Emission {
    Emission::new::<GridLaid>(encode(&GridLaid { place, step }), Visibility::Public)
}

// ---------------------------------------------------------------------------------------------
// test-rogue
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Rogue;

impl SystemIdentity for Rogue {
    const ID: SystemId = ROGUE;
}

/// What the rogue resolver does to every arrival into its place: one breach of presence's rules, or
/// (`Obey`) a change that keeps all of them.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Violation {
    /// Ends 1 000 mm beyond `to`, along +x.
    Lengthen,
    /// Ends in another place.
    OtherPlace(PlaceId),
    /// Ends facing 90°, where `to` has no facing.
    Turn,
    /// Ends with no local position.
    DropLocal,
    /// Displaces the person arriving.
    DisplaceWalker,
    /// Displaces one person twice.
    DisplaceTwice(PersonId),
    /// Displaces a person who is in another place.
    DisplaceAbsent(PersonId),
    /// Displaces an entity that is a place, named as if it were a person.
    DisplaceNonPerson(PersonId),
    /// Displaces a destroyed person.
    DisplaceDestroyed(PersonId),
    /// Displaces a person into another place.
    DisplaceElsewhere(PersonId, PlaceId),
    /// Displaces a person and turns them.
    TurnDisplaced(PersonId),
    /// Displaces a person and drops their local position.
    DropDisplacedLocal(PersonId),
    /// Stops 1 000 mm short, naming an entity this world never allocated.
    UnknownStopper,
    /// Reaches `to` exactly, but names what stopped them.
    StopperWithoutStop(EntityId),
    /// Stops 1 000 mm short, naming `stopper`, and displaces `other` 100 mm along +x: every rule kept.
    Obey { stopper: EntityId, other: PersonId },
}

/// The rogue resolver's state on a place.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct RogueMode {
    violation: Violation,
}

owned_component! {
    component = RogueMode,
    owner = Rogue,
    component_type = "test-rogue",
    schema_version = 1,
}

#[derive(Serialize, Deserialize)]
struct RogueSet {
    place: PlaceId,
    violation: Violation,
}

impl Event for RogueSet {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("test-rogue-set");
    const OWNER: SystemId = ROGUE;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Rogue {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .owning::<RogueMode>()
            .emitting::<RogueSet>()
            .subscribing_to::<RogueSet>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        require_registered(&Self::ID);
        tables.component::<RogueMode>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == RogueSet::EVENT_TYPE {
            let set: RogueSet = decode(event);
            world.insert(
                set.place.entity_id(),
                RogueMode {
                    violation: set.violation,
                },
            )?;
        }
        Ok(Vec::new())
    }
}

/// Where `person` is now, by presence's record.
fn now_at(world: &WorldRead<'_>, person: PersonId) -> Location {
    world
        .component::<Presence>(person.entity_id())
        .map(Presence::location)
        .expect("the person has a presence")
}

impl ArrivalResolver for Rogue {
    fn resolver_of(&self) -> SystemId {
        ROGUE
    }

    fn resolve(
        &self,
        world: &WorldRead<'_>,
        arriving: &Arriving,
        so_far: Resolution,
    ) -> Resolution {
        let to = arriving.to();
        let Some(mode) = world.component::<RogueMode>(to.place().entity_id()) else {
            return so_far;
        };
        let to_x = x_of(to).expect("the rogue's arrivals have local positions");
        let turned = Orientation::facing(Millidegrees::new(90_000));
        match mode.violation {
            Violation::Lengthen => so_far.stopped_at(with_x(to, to_x + 1_000), None),
            Violation::OtherPlace(elsewhere) => so_far.stopped_at(
                Location::in_place(elsewhere).with_local(to.local().expect("local")),
                None,
            ),
            Violation::Turn => so_far.stopped_at(to.with_facing(turned), None),
            Violation::DropLocal => so_far.stopped_at(Location::in_place(to.place()), None),
            Violation::DisplaceWalker => so_far.displacing(arriving.person(), to),
            Violation::DisplaceTwice(person) => {
                let here = now_at(world, person);
                let x = x_of(here).expect("local");
                so_far
                    .displacing(person, with_x(here, x + 100))
                    .displacing(person, with_x(here, x + 200))
            }
            Violation::DisplaceAbsent(person)
            | Violation::DisplaceNonPerson(person)
            | Violation::DisplaceDestroyed(person) => {
                so_far.displacing(person, at(to.place(), 1_000, 1_000))
            }
            Violation::DisplaceElsewhere(person, elsewhere) => {
                so_far.displacing(person, at(elsewhere, 1_000, 1_000))
            }
            Violation::TurnDisplaced(person) => {
                let here = now_at(world, person);
                so_far.displacing(person, here.with_facing(turned))
            }
            Violation::DropDisplacedLocal(person) => {
                so_far.displacing(person, Location::in_place(to.place()))
            }
            Violation::UnknownStopper => {
                so_far.stopped_at(with_x(to, to_x - 1_000), Some(EntityId::from_raw(999)))
            }
            Violation::StopperWithoutStop(stopper) => so_far.stopped_at(to, Some(stopper)),
            Violation::Obey { stopper, other } => {
                let here = now_at(world, other);
                let x = x_of(here).expect("local");
                so_far
                    .stopped_at(with_x(to, to_x - 1_000), Some(stopper))
                    .displacing(other, with_x(here, x + 100))
            }
        }
    }
}

/// The genesis fact that sets the rogue's behaviour in `place`.
pub fn rogue(place: PlaceId, violation: Violation) -> Emission {
    Emission::new::<RogueSet>(encode(&RogueSet { place, violation }), Visibility::Public)
}

// ---------------------------------------------------------------------------------------------
// test-echo (SC-8: a system that states presence's arrived and subscribes to it)
// ---------------------------------------------------------------------------------------------

pub struct Echo {
    trigger: PersonId,
    follower: PersonId,
}

impl SystemIdentity for Echo {
    const ID: SystemId = SystemId::from_static("test-echo");
}

/// Echo's own fact: it heard somebody arrive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Heard {
    pub person: PersonId,
}

impl Event for Heard {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("test-heard");
    const OWNER: SystemId = Echo::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for Echo {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .emitting::<Arrived>()
            .emitting::<StoppedShort>()
            .emitting::<Heard>()
            .subscribing_to::<Arrived>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() != Arrived::EVENT_TYPE {
            return Ok(Vec::new());
        }
        let arrived: Arrived = decode(event);
        let mut said = vec![
            Emission::new::<Heard>(
                encode(&Heard {
                    person: arrived.person(),
                }),
                Visibility::Public,
            )
            .about(vec![arrived.person().entity_id()]),
        ];
        if arrived.person() == self.trigger {
            let there = arrived.location();
            let beside = with_x(there, x_of(there).expect("local") + 600);
            said.extend(
                arrivals(&world.read(), self.follower, beside).map_err(|reason| {
                    KernelError::FactRefusedByOwner {
                        system: PresenceSystem::ID,
                        event_type: Arrived::EVENT_TYPE,
                        reason,
                    }
                })?,
            );
        }
        Ok(said)
    }
}

// ---------------------------------------------------------------------------------------------
// test-placer (SC-5: a system that places people through arrivals)
// ---------------------------------------------------------------------------------------------

#[derive(Default)]
pub struct Placer;

impl SystemIdentity for Placer {
    const ID: SystemId = SystemId::from_static("test-placer");
}

/// Put `person` at `to`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Place {
    pub person: PersonId,
    pub to: Location,
}

impl Action for Place {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("test-place");
    const OWNER: SystemId = Placer::ID;
}

impl System for Placer {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([PresenceSystem::ID])
            .providing::<Place>()
            .emitting::<Arrived>()
            .emitting::<StoppedShort>()
    }

    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let place: Place =
            serde_json::from_slice(intent.payload().payload()).expect("a test-place payload");
        arrivals(&world.read(), place.person, place.to).map_err(|reason| {
            KernelError::FactRefusedByOwner {
                system: PresenceSystem::ID,
                event_type: Arrived::EVENT_TYPE,
                reason,
            }
        })
    }
}

// ---------------------------------------------------------------------------------------------
// the world they share
// ---------------------------------------------------------------------------------------------

/// A System Pack a town installs, after presence, in the order listed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pack {
    Movement,
    Fences,
    Grid,
    Rogue,
    Placer,
}

/// Which of the two places.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    Yard,
    Lane,
}

/// Where genesis places somebody: a place and (x, y) in it.
pub type Spot = (Where, i32, i32);

/// A person by key, and where genesis places them (none: not placed).
pub type Resident = (&'static str, Option<Spot>);

/// Everything about a town decided before it begins.
pub struct Plan {
    /// Installed after presence, in this order.
    pub packs: Vec<Pack>,
    /// A fence across the yard at this x.
    pub fence: Option<i32>,
    /// A grid over the yard with this step.
    pub grid: Option<i32>,
    /// The people, created in this order, and where genesis places each (none: not placed).
    pub people: Vec<Resident>,
    /// The rogue's behaviour in the yard, given the town's identities.
    pub rogue: Option<fn(&Cast) -> Violation>,
    /// Install test-echo, last: (trigger, follower).
    pub echo: Option<(&'static str, &'static str)>,
}

impl Plan {
    /// A town with these packs and people, and nothing else.
    pub fn new(packs: &[Pack], people: &[Resident]) -> Self {
        Self {
            packs: packs.to_vec(),
            fence: None,
            grid: None,
            people: people.to_vec(),
            rogue: None,
            echo: None,
        }
    }

    /// The world with its systems installed and nothing else: what a save is resumed into.
    pub fn composed(&self) -> World {
        let mut world = World::new();
        world.install(PresenceSystem).expect("presence installs");
        for pack in &self.packs {
            match pack {
                Pack::Movement => world.install(MovementSystem),
                Pack::Fences => world.install(Fences),
                Pack::Grid => world.install(Grid),
                Pack::Rogue => world.install(Rogue),
                Pack::Placer => world.install(Placer),
            }
            .expect("a test pack installs");
        }
        world
    }

    /// The world composed, its entities created, and its genesis facts built but not applied.
    pub fn assembled(&self) -> (World, Cast, Vec<Emission>) {
        let mut world = self.composed();
        let mut create = |key: &str, entity_type: EntityType| {
            world
                .create_entity(EntityKey::new(key).expect("a key"), entity_type)
                .expect("an entity")
        };
        let yard = PlaceId::new(create("yard", EntityType::Place), EntityType::Place).expect("p");
        let lane = PlaceId::new(create("lane", EntityType::Place), EntityType::Place).expect("p");
        let mut people = BTreeMap::new();
        for (key, _) in &self.people {
            let id = create(key, EntityType::Person);
            people.insert(
                *key,
                PersonId::new(id, EntityType::Person).expect("a person"),
            );
        }
        let cast = Cast { yard, lane, people };
        if let Some((trigger, follower)) = self.echo {
            world
                .install(Echo {
                    trigger: cast.person(trigger),
                    follower: cast.person(follower),
                })
                .expect("echo installs");
        }

        let mut facts = Vec::new();
        if self.packs.contains(&Pack::Movement) {
            facts.push(passage(
                yard,
                Some(LocalPosition::on_ground(
                    Millimetres::new(9_000),
                    Millimetres::new(2_000),
                )),
                lane,
                Some(LocalPosition::on_ground(
                    Millimetres::new(0),
                    Millimetres::new(2_000),
                )),
            ));
        }
        if let Some(x) = self.fence {
            facts.push(fence(yard, x));
        }
        if let Some(step) = self.grid {
            facts.push(grid(yard, step));
        }
        if let Some(violation) = self.rogue {
            facts.push(rogue(yard, violation(&cast)));
        }
        for (key, spot) in &self.people {
            if let Some((place, x, y)) = spot {
                let location = at(cast.place(*place), *x, *y);
                facts.push(
                    arrival(&world.read(), cast.person(key), location)
                        .expect("genesis placement is accepted"),
                );
            }
        }
        (world, cast, facts)
    }
}

/// The identities a town was given.
#[derive(Debug, Clone)]
pub struct Cast {
    pub yard: PlaceId,
    pub lane: PlaceId,
    pub people: BTreeMap<&'static str, PersonId>,
}

impl Cast {
    pub fn person(&self, key: &str) -> PersonId {
        self.people[key]
    }

    pub const fn place(&self, place: Where) -> PlaceId {
        match place {
            Where::Yard => self.yard,
            Where::Lane => self.lane,
        }
    }
}

/// A town that has begun: its world, its identities, and every fact it has recorded.
pub struct Town {
    pub world: World,
    pub cast: Cast,
    pub log: Vec<EventEnvelope>,
    next_action: u64,
    now: i64,
}

impl Town {
    /// Assembles the plan and runs genesis at the epoch.
    pub fn begin(plan: &Plan) -> Self {
        let (mut world, cast, facts) = plan.assembled();
        let log = world
            .genesis(WorldTime::EPOCH, facts)
            .expect("genesis is accepted");
        Self {
            world,
            cast,
            log,
            next_action: 1,
            now: 0,
        }
    }

    pub fn person(&self, key: &str) -> PersonId {
        self.cast.person(key)
    }

    pub fn yard(&self, x: i32, y: i32) -> Location {
        at(self.cast.yard, x, y)
    }

    pub fn lane(&self, x: i32, y: i32) -> Location {
        at(self.cast.lane, x, y)
    }

    /// Dispatches `action` by `actor`, ten seconds after the last request, with the next id; records
    /// what it recorded. Returns the id and the kernel's answer.
    pub fn request<A: Action + Serialize>(
        &mut self,
        actor: PersonId,
        action: &A,
    ) -> (ActionId, Result<Dispatched, KernelError>) {
        let id = ActionId::from_raw(self.next_action);
        self.next_action += 1;
        self.now += 10;
        let at = WorldTime::from_seconds(self.now);
        let intent = ActionIntent::new(
            id,
            actor.entity_id(),
            ActionRecord::new::<A>(encode(action)),
            at,
        );
        let answer = self.world.dispatch(&intent, at);
        if let Ok(dispatched) = &answer {
            self.log.extend(dispatched.events().iter().cloned());
        }
        (id, answer)
    }

    /// `person` asks to move to `to`.
    pub fn walk(
        &mut self,
        person: &str,
        to: Location,
    ) -> (ActionId, Result<Dispatched, KernelError>) {
        let actor = self.person(person);
        self.request(actor, &Move::new(to))
    }

    /// Where `person` is, by presence's record.
    pub fn where_is(&self, person: &str) -> Option<Location> {
        self.world
            .read()
            .component::<Presence>(self.person(person).entity_id())
            .map(Presence::location)
    }
}

/// The state a refused dispatch must leave untouched: every component row and every edge, as bytes
/// (presence's own `owned_state` pattern).
pub fn owned_state(world: &World) -> Vec<u8> {
    let snapshot = world.snapshot().expect("a snapshot");
    encode(&(snapshot.components, snapshot.relations))
}

/// One recorded fact, read back for comparison with a literal.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fact {
    Arrived(PersonId, Location),
    StoppedShort {
        person: PersonId,
        wanted: Location,
        reached: Location,
        by: Option<EntityId>,
    },
    Entered(PersonId),
    Heard(PersonId),
    Other(String),
}

/// A recorded fact as a [`Fact`].
pub fn fact(event: &EventEnvelope) -> Fact {
    let kind = event.event_type().as_str();
    match kind {
        "arrived" => {
            let arrived: Arrived = decode(event);
            Fact::Arrived(arrived.person(), arrived.location())
        }
        "stopped-short" => {
            let stopped: StoppedShort = decode(event);
            Fact::StoppedShort {
                person: stopped.person(),
                wanted: stopped.wanted(),
                reached: stopped.reached(),
                by: stopped.by(),
            }
        }
        "person-entered-place" => {
            let entered: mineworld_presence::PersonEnteredPlace = decode(event);
            Fact::Entered(entered.person())
        }
        "test-heard" => {
            let heard: Heard = decode(event);
            Fact::Heard(heard.person)
        }
        other => Fact::Other(other.to_owned()),
    }
}

/// Every fact a dispatch recorded, read back.
pub fn facts(dispatched: &Dispatched) -> Vec<Fact> {
    dispatched.events().iter().map(fact).collect()
}

// ---------------------------------------------------------------------------------------------
// RS-7's script: twelve moves for alice, two of them through the doorway, two across x = 5 000
// ---------------------------------------------------------------------------------------------

/// Where alice starts.
pub const SCRIPT_START: Spot = (Where::Yard, 4_000, 2_000);

/// The twelve moves, each within movement's 2 m stride of the one before; a change of place goes
/// through the doorway (yard 9 000, 2 000 ↔ lane 0, 2 000).
pub const SCRIPT: [Spot; 12] = [
    (Where::Yard, 5_000, 2_000),
    (Where::Yard, 6_500, 2_000),
    (Where::Yard, 8_000, 2_000),
    (Where::Yard, 9_000, 2_500),
    (Where::Lane, 500, 2_000),
    (Where::Lane, 2_000, 2_000),
    (Where::Lane, 3_000, 3_000),
    (Where::Lane, 1_500, 2_000),
    (Where::Lane, 200, 1_800),
    (Where::Yard, 8_800, 2_000),
    (Where::Yard, 7_000, 1_500),
    (Where::Yard, 5_600, 2_000),
];

/// Runs the script in `town` and checks every move recorded exactly presence's `arrived` at the
/// requested location — its payload byte-equal to `Arrived::new`'s encoding — plus
/// `person-entered-place` on a change of place, and never `stopped-short`.
pub fn run_inert_script(town: &mut Town) {
    let alice = town.person("alice");
    let mut place = SCRIPT_START.0;
    for (step, (to_place, x, y)) in SCRIPT.into_iter().enumerate() {
        let to = at(town.cast.place(to_place), x, y);
        let (_, answer) = town.walk("alice", to);
        let dispatched = answer.expect("the move is answered");
        assert!(
            matches!(
                dispatched.result(),
                mineworld_contracts::ActionResult::Accepted { .. }
            ),
            "move {step} is accepted: {:?}",
            dispatched.result()
        );
        let events = dispatched.events();
        let crossing = to_place != place;
        assert_eq!(
            events.len(),
            if crossing { 2 } else { 1 },
            "move {step}: {:?}",
            facts(&dispatched)
        );
        assert_eq!(*events[0].event_type(), Arrived::EVENT_TYPE, "move {step}");
        assert_eq!(
            events[0].payload().payload(),
            &encode(&Arrived::new(alice, to)),
            "move {step}: exactly the arrival asked for, byte for byte"
        );
        if crossing {
            assert_eq!(fact(&events[1]), Fact::Entered(alice), "move {step}");
        }
        assert!(
            events
                .iter()
                .all(|event| event.event_type().as_str() != "stopped-short"),
            "move {step}: never stopped-short"
        );
        place = to_place;
    }
}
