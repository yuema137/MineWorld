//! The central pipeline: a request in, the facts it caused out.
//!
//! `docs/CORE_CONCEPTS.md` §12 states it in one line, and this module is that line as code:
//!
//! ```text
//! ActionIntent → route → validate → resolve → Event(s) → reduce → new world state
//! ```
//!
//! Each arrow is a decision about *who* is allowed to make it, and that is what the module is
//! really for:
//!
//! ```text
//! route      the world decides, from the route map alone — no system is consulted (INV-10, BD-5)
//! validate   the one system that provides the action decides, reading only (BD-6)
//! resolve    the same system decides the outcome and returns the facts it caused
//! record     the kernel decides identity, instant, causation and provenance (INV-15, AC-9)
//! reduce     every enabled subscriber applies the fact to state it owns, in registration order
//! ```
//!
//! # And the one thing that happens before any of it: genesis
//!
//! A world has to start with something true of it, and component state is writable only by its
//! owning system while resolving an action or reacting to a fact. So a World Pack's initial state
//! arrives as [`World::genesis`]: the same recording and the same reduction as above, with
//! [`Causation::WorldGenesis`] in place of a request that never happened.
//!
//! ```text
//! dispatch   Causation::Action(id)   provenance names the emitting system and the request
//! genesis    Causation::WorldGenesis provenance names the owning system and NO request
//! ```
//!
//! That second line is the whole of the decision. The alternative — writing components directly at
//! assembly, or inventing an `ActionId` for state nobody requested — would leave a world whose
//! initial state either has no cause in its log or claims a cause that never existed. `AC-9`
//! requires every mutation to trace to something, and `contracts/src/event.rs` already says what
//! initial state traces to: "a loaded World Pack's initial facts are caused by this and by nothing
//! else."
//!
//! # Three things dispatch will not do
//!
//! **It does not interpret a payload.** An action's and an event's payloads are bytes the declaring
//! system encoded, and no kernel code decodes them (`BI-1`).
//!
//! **It does not hand a system the store.** Every call gets a [`WorldRead`] or a
//! [`WorldView`](crate::WorldView), never a `&mut ComponentStore`, because a `&mut` permits
//! `*store = ComponentStore::new()` and no ownership check can prevent that (`BD-2`).
//!
//! **It does not own a clock or a queue.** Dispatch is *told* which instant it is working in, and
//! deferred work comes back in [`Dispatched::deferred`] rather than being scheduled. The clock and
//! the queue are S4's, and this is the seam that keeps S4 from re-plumbing the pipeline (`BD-7`,
//! `D-6`). For the same reason nothing here appends to an event log: the recorded facts are
//! *returned*, and the log is S5's.
//!
//! # Where the "a refusal changes nothing" promise narrows
//!
//! Every refusal in this crate validates before it mutates, and two of this module's do not,
//! because they cannot. A [`KernelError`] out of `resolve` or `react` — an undeclared event type, a
//! cascade that will not terminate — is reported *after* that system's writes have landed, and
//! reduction is not transactional: there is no undo log, and inventing one would mean the kernel
//! holding a shadow copy of every system's state.
//!
//! That is deliberate, and the rule it follows from is the honest one: **an error out of dispatch is
//! a bug in a system, not a rejected request.** A rejected request is
//! [`ActionResult::Rejected`], which changes nothing by construction because `validate` cannot
//! write. An `Err` means a system broke its own contract, and the caller's correct response is to
//! stop the world and report it, not to retry.

use std::collections::BTreeSet;

use mineworld_contracts::{
    ActionId, ActionIntent, ActionResult, Causation, EventEnvelope, EventId, Provenance, SystemId,
    WorldTime,
};

use crate::components::ComponentStore;
use crate::entities::EntityRegistry;
use crate::error::KernelError;
use crate::registry::SystemRegistry;
use crate::relations::RelationStore;
use crate::system::{Deferral, Emission};
use crate::view::{WorldParts, WorldRead};
use crate::world::World;

/// How many generations of reaction one logical instant may contain.
///
/// Reaction is synchronous within the instant (`D-6`): a system emits, its subscribers reduce, and
/// what they emit is reduced in turn — all before the clock moves. That is what makes a wage paid in
/// the same instant it falls due rather than a tick later, and it is also how two systems that react
/// to each other can loop forever inside one instant.
///
/// So the depth is bounded, and the bound errors rather than stopping quietly. Sixteen is chosen to
/// be far above any honest chain — a cross-domain consequence is two or three steps, not ten — and
/// far below anything a person would experience as a hang. **An infinite cascade is a system bug, and
/// a world that freezes silently is worse than one that says which systems were cycling.**
pub const CASCADE_DEPTH_LIMIT: usize = 16;

/// The first event identity a world records.
///
/// One, not zero, for the reason entity identity starts at one: zero is what an absent, defaulted or
/// truncated number looks like in a serialized record.
const FIRST_EVENT_ID: u64 = 1;

/// A world's event-identity counter.
///
/// A plain monotonic counter, consulted by nothing else: identity may not come from a clock, a random
/// source or a hash, because a world replayed from its log has to produce the same identities in the
/// same order (`AC-12`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EventIds {
    next: u64,
}

impl EventIds {
    pub(crate) const fn new() -> Self {
        Self {
            next: FIRST_EVENT_ID,
        }
    }

    /// Allocates the next identity. Never reuses one, and never moves backwards.
    fn allocate(&mut self) -> Result<EventId, KernelError> {
        let id = EventId::from_raw(self.next);
        self.next = self
            .next
            .checked_add(1)
            .ok_or(KernelError::EventIdSpaceExhausted)?;
        Ok(id)
    }
}

/// What one dispatch produced: the answer, the facts, and the work to be scheduled.
///
/// Three things, because three different layers consume them. The answer goes back to whoever
/// submitted the request — a controller, a client, a protocol. The facts are history, and it is the
/// persistence layer (S5) that appends them to the log; dispatch has recorded them and holds no log
/// of its own. The deferrals go to the scheduler (S4), which queues them at their instants.
///
/// A world that hits [`CASCADE_DEPTH_LIMIT`] produces none of this: it produces the error naming the
/// systems that were cycling.
#[derive(Debug, Clone, PartialEq, Eq)]
#[must_use]
pub struct Dispatched {
    result: ActionResult,
    events: Vec<EventEnvelope>,
    deferred: Vec<Deferral>,
}

impl Dispatched {
    /// What the world answers the request.
    pub const fn result(&self) -> &ActionResult {
        &self.result
    }

    /// The facts the request caused, in the order they were recorded: what resolution emitted first,
    /// then each generation of reaction.
    pub fn events(&self) -> &[EventEnvelope] {
        &self.events
    }

    /// The facts a system asked to happen at a later instant, for the scheduler to queue.
    pub fn deferred(&self) -> &[Deferral] {
        &self.deferred
    }

    /// The three parts, for a caller that has to move them onward rather than read them.
    pub fn into_parts(self) -> (ActionResult, Vec<EventEnvelope>, Vec<Deferral>) {
        (self.result, self.events, self.deferred)
    }
}

impl World {
    /// Dispatches one request: routes it, validates it, resolves it, records the facts, and reduces
    /// them into the state their owners hold.
    ///
    /// The instant is supplied rather than read from a clock, because the clock is S4's and a system
    /// that asked a clock what time it was would produce facts a replay could not reproduce
    /// (`AC-12`).
    ///
    /// Three answers and one error, and the difference between them is the whole of `INV-10`:
    ///
    /// ```text
    /// Unavailable   no enabled system provides this action type — no system was consulted
    /// Rejected      the one system that provides it considered the request and said no
    /// Accepted      it resolved, and these are the identities of the facts that followed
    /// Err           a system broke its own contract; see this module's documentation
    /// ```
    ///
    /// ```
    /// # use mineworld_contracts::{
    /// #     Action, ActionId, ActionIntent, ActionRecord, ActionResult, ActionTypeId, EntityKey,
    /// #     EntityType, SystemId, WorldTime,
    /// # };
    /// # use mineworld_kernel::{
    /// #     System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    /// # };
    /// # use serde::{Deserialize, Serialize};
    /// # struct Places;
    /// # impl SystemIdentity for Places {
    /// #     const ID: SystemId = SystemId::from_static("places");
    /// # }
    /// # impl System for Places {
    /// #     const VERSION: SystemVersion = SystemVersion::new(1);
    /// #     fn declaration(&self) -> SystemDeclaration {
    /// #         SystemDeclaration::of::<Self>()
    /// #     }
    /// # }
    /// # #[derive(Serialize, Deserialize)]
    /// # struct Shoot;
    /// # impl Action for Shoot {
    /// #     const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("shoot");
    /// #     const OWNER: SystemId = SystemId::from_static("combat");
    /// # }
    /// # fn main() -> Result<(), mineworld_kernel::KernelError> {
    /// let mut world = World::new();
    /// world.install(Places)?;
    /// let actor = world.create_entity(EntityKey::new("alice")?, EntityType::Person)?;
    ///
    /// // There is no combat system in this world, so there is no shooting in it.
    /// let intent = ActionIntent::new(
    ///     ActionId::from_raw(1),
    ///     actor,
    ///     ActionRecord::new::<Shoot>(Vec::new()),
    ///     WorldTime::EPOCH,
    /// );
    /// let dispatched = world.dispatch(&intent, WorldTime::EPOCH)?;
    ///
    /// assert_eq!(*dispatched.result(), ActionResult::Unavailable);
    /// assert!(dispatched.events().is_empty());
    /// # Ok(())
    /// # }
    /// ```
    pub fn dispatch(
        &mut self,
        intent: &ActionIntent,
        at: WorldTime,
    ) -> Result<Dispatched, KernelError> {
        self.note_dispatch();
        self.dispatcher(at).dispatch(intent)
    }

    /// States what is true of this world as it comes into existence, and reduces those facts into
    /// the state their owners hold.
    ///
    /// This is how a World Pack seeds component state, and it is the only way: a component is
    /// written by its owning system, while resolving or reacting, so initial state has to be a fact
    /// the owning system reduces. What genesis supplies is the fact — and the causation that makes
    /// it honest.
    ///
    /// ```text
    /// identity      allocated from this world's own event counter, continuing into dispatch
    /// instant       `at`, the instant the world begins in
    /// caused_by     Causation::WorldGenesis, always. There is no request, so none is named.
    /// provenance    the system that owns the event type, with NO controller decision
    /// ```
    ///
    /// The emitting system is read off each fact's own event type
    /// ([`Emission::owner`](crate::Emission::owner)), never supplied by the caller: world assembly
    /// may state facts drawn from the vocabulary its systems declared, and nothing else. A fact
    /// whose owner is not installed, or whose owner does not declare emitting that type, is
    /// refused by name.
    ///
    /// Refused once the world has dispatched anything. Genesis is assembly: a person who arrives
    /// while a world is running arrives by *acting*, and a path that let a running world state an
    /// uncaused fact would make the log's own claim about genesis untrue.
    ///
    /// Returns the facts as recorded, for the caller that has to log them (S5) or show them (S11).
    ///
    /// ```
    /// # use mineworld_contracts::{
    /// #     Causation, EntityKey, EntityType, Event, EventSchemaVersion, EventTypeId, SystemId,
    /// #     Visibility, WorldTime,
    /// # };
    /// # use mineworld_kernel::{
    /// #     Declarations, Emission, System, SystemDeclaration, SystemIdentity, SystemVersion,
    /// #     World, owned_component,
    /// # };
    /// # use serde::{Deserialize, Serialize};
    /// # struct Weather;
    /// # impl SystemIdentity for Weather {
    /// #     const ID: SystemId = SystemId::from_static("weather");
    /// # }
    /// # #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    /// # struct Season { name: String }
    /// # owned_component! {
    /// #     component = Season,
    /// #     owner = Weather,
    /// #     component_type = "season",
    /// #     schema_version = 1,
    /// # }
    /// # #[derive(Serialize, Deserialize)]
    /// # struct SeasonTurned { name: String }
    /// # impl Event for SeasonTurned {
    /// #     const EVENT_TYPE: EventTypeId = EventTypeId::from_static("season-turned");
    /// #     const OWNER: SystemId = Weather::ID;
    /// #     const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
    /// # }
    /// # impl System for Weather {
    /// #     const VERSION: SystemVersion = SystemVersion::new(1);
    /// #     fn declaration(&self) -> SystemDeclaration {
    /// #         SystemDeclaration::of::<Self>()
    /// #             .owning::<Season>()
    /// #             .emitting::<SeasonTurned>()
    /// #             .subscribing_to::<SeasonTurned>()
    /// #     }
    /// #     fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), mineworld_kernel::KernelError> {
    /// #         tables.component::<Season>()
    /// #     }
    /// #     fn react(
    /// #         &self,
    /// #         world: &mut mineworld_kernel::WorldView<'_, Self>,
    /// #         event: &mineworld_contracts::EventEnvelope,
    /// #     ) -> Result<Vec<Emission>, mineworld_kernel::KernelError> {
    /// #         let turned: SeasonTurned = serde_json::from_slice(event.payload().payload()).unwrap();
    /// #         let subject = event.subjects()[0];
    /// #         world.insert(subject, Season { name: turned.name })?;
    /// #         Ok(Vec::new())
    /// #     }
    /// # }
    /// # fn main() -> Result<(), mineworld_kernel::KernelError> {
    /// let mut world = World::new();
    /// world.install(Weather)?;
    /// let valley = world.create_entity(EntityKey::new("valley")?, EntityType::Place)?;
    ///
    /// let recorded = world.genesis(
    ///     WorldTime::EPOCH,
    ///     vec![
    ///         Emission::new::<SeasonTurned>(
    ///             serde_json::to_vec(&SeasonTurned { name: "spring".to_owned() }).unwrap(),
    ///             Visibility::Public,
    ///         )
    ///         .about(vec![valley]),
    ///     ],
    /// )?;
    ///
    /// assert_eq!(*recorded[0].caused_by(), Causation::WorldGenesis);
    /// assert_eq!(recorded[0].provenance().controller_decision(), None);
    /// assert_eq!(
    ///     world.read().component::<Season>(valley).cloned(),
    ///     Some(Season { name: "spring".to_owned() }),
    ///     "the owning system reduced the fact into the state it owns",
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub fn genesis(
        &mut self,
        at: WorldTime,
        facts: Vec<Emission>,
    ) -> Result<Vec<EventEnvelope>, KernelError> {
        if self.has_dispatched() {
            return Err(KernelError::GenesisAfterTheWorldHasRun { facts: facts.len() });
        }
        self.dispatcher(at).genesis(facts)
    }

    /// One pipeline over this world at one instant. The single place a [`Dispatcher`] is built, so
    /// that dispatch and genesis cannot end up recording or reducing differently.
    fn dispatcher(&mut self, at: WorldTime) -> Dispatcher<'_> {
        let (systems, entities, components, relations, events) = self.dispatch_parts();
        Dispatcher {
            systems,
            entities,
            components,
            relations,
            ids: events,
            at,
            recorded: Vec::new(),
            deferred: Vec::new(),
            emitted_while_reducing: BTreeSet::new(),
        }
    }
}

/// One dispatch, holding the parts of a world it is allowed to touch and nothing else.
///
/// Crate-private and never handed to a system: what a system receives is built from these for the
/// duration of one call (`BD-2`). It is a struct rather than a long argument list because the
/// pipeline is a sequence of steps over the same borrows, and threading six of them through five
/// functions is how one of them eventually gets forgotten.
struct Dispatcher<'a> {
    systems: &'a SystemRegistry,
    entities: &'a EntityRegistry,
    components: &'a mut ComponentStore,
    relations: &'a mut RelationStore,
    ids: &'a mut EventIds,
    at: WorldTime,
    /// Every fact recorded so far, in the order it was recorded.
    recorded: Vec<EventEnvelope>,
    /// What systems asked to happen later, in the order they asked.
    deferred: Vec<Deferral>,
    /// Which systems have emitted a fact *while reducing*. This is the set the cascade limit names,
    /// because a system that only emitted during resolution is not the one that will not stop.
    emitted_while_reducing: BTreeSet<SystemId>,
}

impl Dispatcher<'_> {
    /// The pipeline, in the order `docs/CORE_CONCEPTS.md` §12 states it.
    fn dispatch(mut self, intent: &ActionIntent) -> Result<Dispatched, KernelError> {
        // Route. The map is the whole decision: an action type that is not in it is not provided by
        // any enabled system, so the world answers without consulting one (`INV-10`, `BD-5`).
        let Some((provider, system)) = self.systems.routed(intent.action_type()) else {
            return Ok(Dispatched {
                result: ActionResult::Unavailable,
                events: Vec::new(),
                deferred: Vec::new(),
            });
        };

        // Validate. The view is read-only, so "validation has no side effects" is the type's
        // statement rather than a rule a system could break (`BD-6`).
        let read = WorldRead::new(self.entities, self.components, self.relations);
        if let Err(rejection) = system.validate(&read, intent) {
            return Ok(Dispatched {
                result: ActionResult::Rejected(rejection),
                events: Vec::new(),
                deferred: Vec::new(),
            });
        }

        // Resolve, then record what it decided. The facts are caused by the request itself, which is
        // the kernel's statement and not the system's (`AC-9`, `INV-15`).
        let emissions = system.resolve(self.parts(), intent)?;
        let generation = self.record(
            provider,
            emissions,
            &Causation::Action(intent.action_id()),
            Some(intent.action_id()),
        )?;

        self.reduce(generation, Some(intent.action_id()))?;

        Ok(Dispatched {
            result: ActionResult::Accepted {
                events: self.recorded.iter().map(EventEnvelope::id).collect(),
            },
            events: self.recorded,
            deferred: self.deferred,
        })
    }

    /// The genesis pipeline: record what the world begins with, then reduce it exactly as dispatch
    /// reduces what a request caused.
    ///
    /// Two differences from [`Dispatcher::dispatch`], and no third. Causation is
    /// [`Causation::WorldGenesis`] rather than a request, and there is no controller decision to
    /// name. Everything else — identity from the world's counter, the declaration check, the
    /// subscriber order, the cascade limit — is shared code, because two recording paths that could
    /// drift would be two accounts of one world's history.
    ///
    /// The emitting system comes from the fact itself. A caller may state a fact of any type an
    /// installed system declares it emits, and nothing else: assembly composes a world out of the
    /// vocabularies its systems brought, and inventing a fact outside them would be a World Pack
    /// defining a rule (`MODULE_SPEC.md` §4).
    fn genesis(mut self, facts: Vec<Emission>) -> Result<Vec<EventEnvelope>, KernelError> {
        let mut generation = Vec::with_capacity(facts.len());
        for fact in facts {
            let owner = fact.owner().clone();
            if self.systems.system(&owner).is_none() {
                return Err(KernelError::GenesisFactHasNoInstalledOwner {
                    system: owner,
                    event_type: fact.event_type().clone(),
                });
            }
            generation.extend(self.record(&owner, vec![fact], &Causation::WorldGenesis, None)?);
        }

        self.reduce(generation, None)?;
        Ok(self.recorded)
    }

    /// Reduction: every enabled subscriber applies each fact, in registration order, and whatever
    /// they emit is reduced in the next generation of the same instant (`BD-7`).
    fn reduce(
        &mut self,
        mut generation: Vec<EventEnvelope>,
        decision: Option<ActionId>,
    ) -> Result<(), KernelError> {
        let mut depth = 0;
        while !generation.is_empty() {
            depth += 1;
            if depth > CASCADE_DEPTH_LIMIT {
                return Err(KernelError::ReductionCascadeTooDeep {
                    limit: CASCADE_DEPTH_LIMIT,
                    systems: self.cycling_systems(),
                });
            }

            let mut next = Vec::new();
            for event in &generation {
                // Collected rather than iterated lazily, because reducing writes to the world the
                // subscriber list is borrowed from — and because the order is a statement worth
                // making in one place: registration order, decided by the registry (`BD-4`).
                let subscribers: Vec<SystemId> = self
                    .systems
                    .subscribers(event.event_type())
                    .cloned()
                    .collect();
                for subscriber in subscribers {
                    let Some(system) = self.systems.system(&subscriber) else {
                        continue;
                    };
                    let emissions = system.react(self.parts(), event)?;
                    if !emissions.is_empty() {
                        self.emitted_while_reducing.insert(subscriber.clone());
                    }
                    next.extend(self.record(
                        &subscriber,
                        emissions,
                        &Causation::Event(event.id()),
                        decision,
                    )?);
                }
            }
            generation = next;
        }
        Ok(())
    }

    /// Turns what a system decided into facts the world has recorded.
    ///
    /// The four fields a system may not choose are supplied here, which is what makes `INV-15` and
    /// `AC-9` mechanical: identity comes from the world's counter, the instant is the one dispatch
    /// was told to work in, causation is the kernel's statement about its own pipeline, and
    /// provenance names the emitting system and the controller decision the chain started from.
    ///
    /// An emission of a type the system's own declaration does not list is refused. The declaration
    /// is what a reader of a world's composition goes by — which facts this world can produce, and
    /// from whom — so a fact outside it would make that reading wrong.
    fn record(
        &mut self,
        emitter: &SystemId,
        emissions: Vec<Emission>,
        caused_by: &Causation,
        decision: Option<ActionId>,
    ) -> Result<Vec<EventEnvelope>, KernelError> {
        let mut recorded = Vec::with_capacity(emissions.len());
        for emission in emissions {
            let declared = self
                .systems
                .declaration(emitter)
                .is_some_and(|declaration| declaration.emits_event(emission.event_type()));
            if !declared {
                return Err(KernelError::EventTypeNotInSystemDeclaration {
                    system: emitter.clone(),
                    event_type: emission.event_type().clone(),
                });
            }

            let id = self.ids.allocate()?;
            // `None` is not a missing value: a genesis fact came from no request, and
            // `controller_decision` is exactly the field that says so.
            let provenance = match decision {
                Some(decision) => {
                    Provenance::new(emitter.clone()).from_controller_decision(decision)
                }
                None => Provenance::new(emitter.clone()),
            };
            let envelope = emission.into_envelope(id, self.at, caused_by.clone(), provenance);
            self.recorded.push(envelope.clone());
            recorded.push(envelope);
        }
        Ok(recorded)
    }

    /// The systems that emitted a fact while reducing, in registration order.
    ///
    /// Registration order rather than name order, so that the error reads like the world's own
    /// composition, and reproducibly: the same cycle names the same systems in the same order on
    /// every run.
    fn cycling_systems(&self) -> Vec<SystemId> {
        self.systems
            .order()
            .iter()
            .filter(|system| self.emitted_while_reducing.contains(*system))
            .cloned()
            .collect()
    }

    /// The parts of the world one call of one system is given, for the duration of that call.
    fn parts(&mut self) -> WorldParts<'_> {
        WorldParts::new(
            self.entities,
            self.components,
            self.relations,
            self.at,
            &mut self.deferred,
        )
    }
}
