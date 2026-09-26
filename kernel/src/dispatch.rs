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
        .dispatch(intent)
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
            intent.action_id(),
        )?;

        self.reduce(generation, intent.action_id())?;

        Ok(Dispatched {
            result: ActionResult::Accepted {
                events: self.recorded.iter().map(EventEnvelope::id).collect(),
            },
            events: self.recorded,
            deferred: self.deferred,
        })
    }

    /// Reduction: every enabled subscriber applies each fact, in registration order, and whatever
    /// they emit is reduced in the next generation of the same instant (`BD-7`).
    fn reduce(
        &mut self,
        mut generation: Vec<EventEnvelope>,
        decision: ActionId,
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
        decision: ActionId,
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
            let provenance = Provenance::new(emitter.clone()).from_controller_decision(decision);
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
