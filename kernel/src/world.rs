//! The world: the composed whole, and the only issuer of write capability in it.
//!
//! A [`World`] holds the three stores of PR 03a, the systems installed into it, and one
//! [`WriteAccess`] — the world's sole issuer of write tokens. That last field is what closes the
//! bypass 03a's review reproduced from an external crate (attempt `A8`): construct your own
//! `WriteAccess`, grant yourself another system's token, write its component. Holding one issuer
//! here would close nothing while anyone could construct a second, so
//! [`WriteAccess::new`](crate::WriteAccess::new) is crate-private as of this PR and the sequence
//! does not compile outside the kernel — pinned by
//! `tests/compile_fail/an_external_crate_cannot_construct_write_access.rs` (`BD-1`).
//!
//! What that buys, stated exactly: a write token enters a world once, at installation, from this
//! field; the only thing that holds it afterwards is the registry entry of the system it belongs
//! to; and a running system reaches it only as a [`WorldView`], never as the issuer and never as a
//! `&mut ComponentStore` (`BD-2`).
//!
//! # Installation is world assembly, and it is not transactional
//!
//! A world is assembled once, before it runs. [`World::install`] validates everything it can from
//! declarations first, so an ordinary refusal — a name already taken, a dependency absent, two
//! systems claiming one component type — leaves the world untouched. What it cannot undo is a
//! failure *after* that: a system's own `install` hook declaring three tables and refusing the
//! fourth leaves the first three declared and its token granted, because `ComponentStore` has no
//! way to undeclare a type and a granted token cannot be recalled.
//!
//! That is deliberate rather than overlooked, and the rule it follows from is the honest one: **a
//! failed installation is a failed world assembly, not a recoverable operation.** The caller
//! discards the world and reports; it does not retry. Recorded here because the rest of this crate
//! promises that a refusal changes nothing, and this is the one place that promise is narrower than
//! it sounds.
//!
//! # Assembly has three steps, and the third one is not a write
//!
//! ```text
//! install         the systems this world is composed of        World::install
//! create_entity   the things that exist in it                  World::create_authored_entity
//! genesis         what is true of them as the world begins     World::genesis
//! ```
//!
//! The third step is [`World::genesis`](crate::World::genesis), and it lives in
//! [`crate::dispatch`] rather than here because it *records facts and reduces them* — the same
//! pipeline a request goes through, with [`Causation::WorldGenesis`](mineworld_contracts::Causation)
//! in place of a request. There is deliberately no method on this type that writes a component:
//! initial state is state some system owns, and a world that could write it directly would hold
//! state its own event log could not explain (`AC-9`).

use mineworld_contracts::{
    Entity, EntityId, EntityKey, EntityType, LifecycleState, Metadata, Relation, SystemId, Tags,
    WorldTime,
};

use crate::access::WriteAccess;
use crate::clock::WorldClock;
use crate::components::ComponentStore;
use crate::dispatch::EventIds;
use crate::entities::EntityRegistry;
use crate::error::KernelError;
use crate::registry::SystemRegistry;
use crate::relations::RelationStore;
use crate::schedule::{Schedule, ScheduleSnapshot, Scheduled, Sequence};
use crate::system::{DynSystem, InstalledSystem, System};
use crate::view::WorldRead;

/// A world taken apart for one pass of the pipeline: the registry and the entity registry to read,
/// and the stores, the event counter and the schedule to write. Crate-private; see
/// [`World::dispatch_parts`].
pub(crate) struct WorldSplit<'a> {
    pub(crate) systems: &'a SystemRegistry,
    pub(crate) entities: &'a EntityRegistry,
    pub(crate) components: &'a mut ComponentStore,
    pub(crate) relations: &'a mut RelationStore,
    pub(crate) events: &'a mut EventIds,
    pub(crate) schedule: &'a mut Schedule,
}

/// A composed world: its state, its systems, and the capability to write its state.
///
/// ```
/// use mineworld_contracts::SystemId;
/// use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion, World};
///
/// struct Places;
///
/// impl SystemIdentity for Places {
///     const ID: SystemId = SystemId::from_static("places");
/// }
///
/// impl System for Places {
///     const VERSION: SystemVersion = SystemVersion::new(1);
///
///     fn declaration(&self) -> SystemDeclaration {
///         SystemDeclaration::of::<Self>()
///     }
/// }
///
/// # fn main() -> Result<(), mineworld_kernel::KernelError> {
/// let mut world = World::new();
/// world.install(Places)?;
///
/// assert!(world.systems().is_installed(&Places::ID));
/// assert!(world.install(Places).is_err(), "a system is installed once");
/// # Ok(())
/// # }
/// ```
///
/// There is no `&mut` accessor for any store. Everything that writes component or edge state goes
/// through a system, and a system writes only what it owns — which is the whole of `INV-7`, and the
/// reason this type exposes reads and assembly and nothing else.
pub struct World {
    entities: EntityRegistry,
    components: ComponentStore,
    relations: RelationStore,
    systems: SystemRegistry,
    /// The counter every recorded fact's identity comes from. Held by the world because identity is
    /// the world's to allocate, and monotonic because a replay has to reproduce it (`AC-12`).
    events: EventIds,
    /// This world's only issuer of write capability. Never lent out, never returned, and not
    /// constructible outside this crate (`BD-1`).
    access: WriteAccess,
    /// The instant this world is at. Moved only by this world's own operations (S4).
    clock: WorldClock,
    /// What is to happen later, in `(WorldTime, Sequence)` order (`DEP-6`).
    schedule: Schedule,
    /// Whether this world has run yet: dispatched a request or advanced its clock.
    ///
    /// One bit, and it exists for one reason: [`World::genesis`] states facts caused by the world
    /// coming into existence, and a world that had already run would be stating them about a past
    /// that has moved on. Assembly is over as soon as the world first acts.
    ran: bool,
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl World {
    /// An empty world: no entities, no systems, no state, and nothing holding a write token.
    pub fn new() -> Self {
        Self {
            entities: EntityRegistry::new(),
            components: ComponentStore::new(),
            relations: RelationStore::new(),
            systems: SystemRegistry::new(),
            events: EventIds::new(),
            access: WriteAccess::new(),
            clock: WorldClock::new(),
            schedule: Schedule::new(),
            ran: false,
        }
    }

    /// Installs a system: the one act that grants write capability in this world.
    ///
    /// In order, and the order matters:
    ///
    /// 1. the system's declaration is read once and its name compared with the system's own `ID`,
    ///    because `declaration()` is a method and can hand back another system's declaration;
    /// 2. the registry decides whether the declaration can join this world at all, from
    ///    declarations alone, while nothing has yet been changed;
    /// 3. the world grants the system its write token — once, from the single issuer;
    /// 4. the system declares the tables it owns, through a
    ///    [`Declarations`](crate::Declarations) that can do nothing else;
    /// 5. every component type the declaration claims is confirmed to have a table, so a
    ///    declaration cannot claim ownership of state that does not exist.
    ///
    /// Takes the system **by value**, which is also the possession check
    /// [`WriteAccess::grant`](crate::WriteAccess::grant) applies: a pack whose system type keeps its
    /// constructor cannot be installed by anybody else. The stronger protection is simply that
    /// installation is the world assembler's act — a system that is already installed cannot be
    /// installed again under the same name, so an impersonator is detected when the real
    /// installation fails.
    pub fn install<T: System>(&mut self, system: T) -> Result<(), KernelError> {
        let declaration = system.declaration();
        if *declaration.system() != T::ID {
            return Err(KernelError::SystemDeclarationNamesAnotherSystem {
                system: T::ID,
                declared: declaration.system().clone(),
            });
        }
        self.systems.check_installable(&declaration)?;

        let token = self.access.grant(&system)?;
        let installed = InstalledSystem::new(system, token);
        installed.install(
            &mut self.components,
            &mut self.relations,
            declaration.owns(),
        )?;

        for owned in declaration.owns() {
            if !self.components.is_declared(owned.component_type()) {
                return Err(KernelError::SystemDidNotDeclareOwnedComponent {
                    system: T::ID,
                    component_type: owned.component_type().clone(),
                });
            }
        }

        self.systems.register(declaration, Box::new(installed));
        Ok(())
    }

    /// The systems this world is composed of.
    pub const fn systems(&self) -> &SystemRegistry {
        &self.systems
    }

    /// Puts an installed system back in the pipeline: its actions are routed again and it reduces
    /// the facts it subscribes to again.
    ///
    /// Refused if the system is not installed, or if a dependency it declared is absent or
    /// disabled — the same check installation applies, because enabling is when a system starts
    /// acting and that is when its dependencies have to be there.
    pub fn enable(&mut self, system: &SystemId) -> Result<(), KernelError> {
        self.systems.enable(system)
    }

    /// Takes an installed system out of the pipeline: nothing routes to it and nothing reduces
    /// through it.
    ///
    /// This is the observable half of `AC-2`. An action the disabled system provided is answered
    /// [`ActionResult::Unavailable`](mineworld_contracts::ActionResult::Unavailable) — exactly as in
    /// a world where that system had never been installed — and no other module changes, which is
    /// what "materially different games from the same core" has to mean at this layer.
    ///
    /// The system stays installed and the state it owns stays in the world, because that state is
    /// still its state and no other system may write it. Refused while an enabled system depends on
    /// it, naming the dependent.
    pub fn disable(&mut self, system: &SystemId) -> Result<(), KernelError> {
        self.systems.disable(system)
    }

    /// Every system holding a write token in this world, in name order.
    ///
    /// The observable half of `BD-1`: a system appears here because this world granted it a token
    /// at installation, and there is no other way for a name to appear.
    pub fn writers(&self) -> impl Iterator<Item = &SystemId> {
        self.access.granted()
    }

    /// This world's entities.
    pub const fn entities(&self) -> &EntityRegistry {
        &self.entities
    }

    /// This world's component state.
    pub const fn components(&self) -> &ComponentStore {
        &self.components
    }

    /// This world's relation graph.
    pub const fn relations(&self) -> &RelationStore {
        &self.relations
    }

    /// Splits the world into the parts one dispatch works on.
    ///
    /// The only place a [`World`] is taken apart, and it is crate-private. Dispatch needs the
    /// registry, the three stores and the event counter at once, with two of them mutably; a method
    /// per field would hand out a `&mut ComponentStore` as a public shape, which is exactly what
    /// `BD-2` forbids, and borrowing the whole world would make the pipeline impossible to write.
    /// Splitting once, here, keeps both true.
    pub(crate) fn dispatch_parts(&mut self) -> WorldSplit<'_> {
        WorldSplit {
            systems: &self.systems,
            entities: &self.entities,
            components: &mut self.components,
            relations: &mut self.relations,
            events: &mut self.events,
            schedule: &mut self.schedule,
        }
    }

    /// Whether this world has left assembly: `true` once it has dispatched a request or advanced.
    ///
    /// Read by [`World::genesis`](crate::World::genesis), which is in [`crate::dispatch`] and
    /// therefore cannot see this struct's fields.
    pub(crate) const fn has_run(&self) -> bool {
        self.ran
    }

    /// Moves the clock to `at` and records that assembly is over — the first thing dispatching and
    /// advancing do, and the only way the clock moves while a world runs. There is no way back.
    pub(crate) fn run_at(&mut self, at: WorldTime) -> Result<(), KernelError> {
        self.clock.advance_to(at)?;
        self.ran = true;
        Ok(())
    }

    /// States the instant the world begins at. Assembly only: [`World::genesis`] calls it before
    /// the world has run, and the clock still refuses an instant before one it was already given.
    pub(crate) fn begin_at(&mut self, at: WorldTime) -> Result<(), KernelError> {
        self.clock.advance_to(at)
    }

    /// The instant this world is at: the instant of the last request it dispatched, the last
    /// instant it advanced to, or the instant it began at.
    ///
    /// The epoch until the world is first given an instant. Simulated seconds and nothing else
    /// (`INV-12`): what those seconds *mean* — a day, an hour, opening time — is a system's.
    pub const fn now(&self) -> WorldTime {
        self.clock.now()
    }

    /// Whether the world has been given an instant yet.
    pub(crate) const fn clock_has_started(&self) -> bool {
        self.clock.has_started()
    }

    /// Removes the next entry due exactly at `at`, if there is one.
    pub(crate) fn pop_due(&mut self, at: WorldTime) -> Option<(Sequence, Scheduled)> {
        self.schedule.pop_at(at)
    }

    /// The earliest instant anything is scheduled for, if anything is.
    pub fn next_instant(&self) -> Option<WorldTime> {
        self.schedule.next_instant()
    }

    /// How many entries are waiting in the schedule.
    pub fn scheduled(&self) -> usize {
        self.schedule.len()
    }

    /// Refuses an instant a request may not be dispatched at, changing nothing.
    ///
    /// Two refusals. An instant before the clock would record facts that run backwards. An instant
    /// at or after which something is still scheduled would let a request overtake work that was
    /// due first — the order `(WorldTime, Sequence)` exists to fix — so the caller advances the world
    /// to that instant before dispatching at it.
    pub(crate) fn check_dispatch_instant(&self, at: WorldTime) -> Result<(), KernelError> {
        self.clock.check(at)?;
        match self.schedule.next_instant() {
            Some(due) if due <= at => Err(KernelError::ScheduledWorkDue { due, at }),
            _ => Ok(()),
        }
    }

    /// Everything this PR adds to a world's state, for a caller that has to save it (S5) — the
    /// clock, the schedule with its pending work, and the event counter, so that a restored world
    /// continues identity where this one stopped.
    pub fn schedule_snapshot(&self) -> ScheduleSnapshot {
        ScheduleSnapshot {
            now: self.clock.now(),
            next_sequence: self.schedule.next_sequence(),
            entries: self.schedule.entries(),
            next_event: self.events.next(),
        }
    }

    /// Replaces this world's clock, schedule and event counter with a saved world's.
    ///
    /// World assembly, like [`World::genesis`]: refused once the world has run, and refused —
    /// changing nothing — if the snapshot could not have come from a world: an entry before its own
    /// `now`, a sequence used twice or ahead of its counter, an event counter below the first
    /// identity, or a deferral by a system this world has not installed. Component state is not in
    /// it: that is S5's.
    pub fn restore_schedule(&mut self, snapshot: ScheduleSnapshot) -> Result<(), KernelError> {
        if self.ran {
            return Err(KernelError::RestoreAfterTheWorldHasRun);
        }
        let ScheduleSnapshot {
            now,
            next_sequence,
            entries,
            next_event,
        } = snapshot;
        for entry in &entries {
            if let Some(system) = entry.item.deferring_system()
                && !self.systems.is_installed(system)
            {
                return Err(KernelError::PersistedEntryNamesUninstalledSystem {
                    system: system.clone(),
                });
            }
        }
        let schedule = Schedule::restore(now, next_sequence, entries)?;
        let events = EventIds::restore(next_event)?;
        self.schedule = schedule;
        self.events = events;
        self.clock.restore(now);
        Ok(())
    }

    /// The whole world as a system reads it: the same view `validate` is handed.
    pub const fn read(&self) -> WorldRead<'_> {
        WorldRead::new(&self.entities, &self.components, &self.relations)
    }

    /// Brings an entity into being, allocating its identity.
    ///
    /// Entity creation is not gated on ownership: an entity is not any system's state, it is the
    /// thing systems attach state *to* (`KD-3`, `INV-8`). What is gated is every component and every
    /// edge that then describes it.
    pub fn create_entity(
        &mut self,
        key: EntityKey,
        entity_type: EntityType,
    ) -> Result<EntityId, KernelError> {
        self.entities.create(key, entity_type)
    }

    /// Brings an entity into being as authored content describes it.
    pub fn create_authored_entity(
        &mut self,
        key: EntityKey,
        entity_type: EntityType,
        tags: Tags,
        metadata: Option<Metadata>,
    ) -> Result<EntityId, KernelError> {
        self.entities
            .create_authored(key, entity_type, tags, metadata)
    }

    /// Destroys an entity and clears every edge that touched it, returning those edges.
    ///
    /// The identity stays spent and the record stays in the registry, because the event log already
    /// refers to it (`INV-11`). The edges go because an edge to something that has permanently
    /// stopped participating is a dangling reference no declaring system could want kept — and they
    /// are *returned* rather than dropped, because their owners have to be able to learn what
    /// happened to state they own.
    ///
    /// Components are not removed. They are their owners' state, and the kernel deleting them would
    /// be the kernel writing another system's component.
    pub fn destroy_entity(&mut self, entity: EntityId) -> Result<Vec<Relation>, KernelError> {
        self.entities
            .transition(entity, LifecycleState::Destroyed)?;
        let record: Entity = self.entities.require(entity)?.clone();
        self.relations.remove_edges_of_destroyed_entity(&record)
    }
}

impl core::fmt::Debug for World {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("World")
            .field("entities", &self.entities.len())
            .field("components", &self.components)
            .field("relations", &self.relations.len())
            .field("systems", &self.systems)
            .field("now", &self.clock.now())
            .field("scheduled", &self.schedule.len())
            .finish()
    }
}
