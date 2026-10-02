//! What a running system is handed: open reads, and writes gated on what it owns.
//!
//! This module is the answer to the sharpest finding of PR 03a's review (`BD-2`). A system needs
//! access to the world's state while it runs, and the obvious way to give it that — a
//! `&mut ComponentStore` — destroys `INV-7` no matter how carefully every write is gated, because
//! a holder of a `&mut` can write:
//!
//! ```text
//! *store = ComponentStore::new();
//! ```
//!
//! and no ownership check of any kind can prevent it. The whole store, every system's state, gone
//! in one assignment that names no component type at all. What dispatch passes a system is
//! therefore an `INV-7` decision and not an ergonomic one, and what it passes is a **view**:
//!
//! ```text
//! WorldRead        every read the kernel offers, and nothing else — handed to validate
//! WorldView<S>     the same reads, plus writes gated on OwnedBy<S> — handed to resolve and react
//! ```
//!
//! Both hold their stores in private fields behind a narrow API. A view cannot be assigned
//! through, cannot be traded for the store it borrows, and cannot outlive the call it was made
//! for.
//!
//! # Reads are open, and that is deliberate
//!
//! Any system may read any component (`KD-4`). Gating reads would make cross-system reaction
//! impossible without inventing a query language, and a component type whose owning system is not
//! installed reads as *absent* rather than as an error — which is exactly what lets a system
//! tolerate the absence of one it does not own. What a *controller* may perceive is a different
//! question with a different answer: `Observation`, which is a value listing what was exposed
//! (`INV-13`), not a view.

use mineworld_contracts::{
    Component, ComponentDeclaration, Entity, EntityId, EntityKey, ProcessId, Relation,
    RelationTypeDeclaration, RelationTypeId, SystemId, WorldTime,
};

use crate::access::{OwnedBy, SystemIdentity, WriteToken};
use crate::components::ComponentStore;
use crate::dispatch::CASCADE_DEPTH_LIMIT;
use crate::entities::EntityRegistry;
use crate::error::KernelError;
use crate::process::{
    InterruptOutcome, InterruptRequest, Interruptibility, Process, ProcessKind, ProcessPhase,
    ProcessStart, ProcessStore,
};
use crate::registry::SystemRegistry;
use crate::relations::RelationStore;
use crate::schedule::Scheduled;
use crate::system::{Cause, Deferral, Emission};

/// Everything a system may read of a world, and nothing more.
///
/// Held by `validate`, where the type is the guarantee: a system cannot write during validation
/// because it has nothing to write through (`BD-6`). Also reachable from a writable view through
/// [`WorldView::read`], so that a system reasoning about state before changing it writes the same
/// reads either way.
pub struct WorldRead<'a> {
    entities: &'a EntityRegistry,
    components: &'a ComponentStore,
    relations: &'a RelationStore,
    processes: &'a ProcessStore,
}

impl<'a> WorldRead<'a> {
    pub(crate) const fn new(
        entities: &'a EntityRegistry,
        components: &'a ComponentStore,
        relations: &'a RelationStore,
        processes: &'a ProcessStore,
    ) -> Self {
        Self {
            entities,
            components,
            relations,
            processes,
        }
    }

    /// A running or suspended process, if there is one with this identity.
    ///
    /// Open to every system, as components are: a phone system may see that Bob is having dinner.
    /// What it may not do is change that — it asks the owner instead.
    pub fn process(&self, process: ProcessId) -> Option<&'a Process> {
        self.processes.get(process)
    }

    /// Every running or suspended process, in [`ProcessId`] order.
    pub fn processes(&self) -> impl Iterator<Item = &'a Process> {
        self.processes.iter()
    }

    /// The entity record, if this world allocated that identity.
    pub fn entity(&self, entity: EntityId) -> Option<&'a Entity> {
        self.entities.get(entity)
    }

    /// The entity record, or the refusal naming the identity that does not exist.
    pub fn require_entity(&self, entity: EntityId) -> Result<&'a Entity, KernelError> {
        self.entities.require(entity)
    }

    /// Every entity, in [`EntityId`] order.
    pub fn entities(&self) -> impl Iterator<Item = &'a Entity> {
        self.entities.iter()
    }

    /// Resolves an authoring key to the identity the runtime uses.
    pub fn resolve_key(&self, key: &EntityKey) -> Result<EntityId, KernelError> {
        self.entities.resolve(key)
    }

    /// An entity's `C`, if it has one and this world declares that component type.
    ///
    /// Absent, not an error, when the owning system is not installed: that is what makes a
    /// system's reaction to another system's state tolerate the other system's removal (`AC-2`).
    pub fn component<C: Component + 'static>(&self, entity: EntityId) -> Option<&'a C> {
        self.components.get::<C>(entity)
    }

    /// Whether an entity has a `C`.
    pub fn has_component<C: Component + 'static>(&self, entity: EntityId) -> bool {
        self.components.contains::<C>(entity)
    }

    /// Every `C` in the world, in [`EntityId`] order — never in the order they were written.
    pub fn components<C: Component + 'static>(&self) -> impl Iterator<Item = (EntityId, &'a C)> {
        self.components.iter::<C>()
    }

    /// How many entities carry a `C`.
    pub fn component_count<C: Component + 'static>(&self) -> usize {
        self.components.count::<C>()
    }

    /// Every component type this world declares, in name order: what its state is made of.
    pub fn component_declarations(&self) -> impl Iterator<Item = &'a ComponentDeclaration> {
        self.components.declarations()
    }

    /// Whether this exact edge exists.
    pub fn is_related(&self, relation: &Relation) -> bool {
        self.relations.contains(relation)
    }

    /// Every edge with `entity` at either end.
    pub fn relations_touching(&self, entity: EntityId) -> impl Iterator<Item = &'a Relation> {
        self.relations.touching(entity)
    }

    /// Every edge of one type, in endpoint order.
    pub fn relations_of_type(
        &self,
        relation_type: &'a RelationTypeId,
    ) -> impl Iterator<Item = &'a Relation> {
        self.relations.of_type(relation_type)
    }

    /// What this world says about an edge type, if any system declared one.
    pub fn relation_declaration(
        &self,
        relation_type: &RelationTypeId,
    ) -> Option<&'a RelationTypeDeclaration> {
        self.relations.declaration(relation_type)
    }
}

/// The parts of a world one call of one system is given, before its token is attached.
///
/// Crate-private, and never handed to a system: [`InstalledSystem`](crate::system::InstalledSystem)
/// wraps it in a [`WorldView`] together with the token this world granted that system. Splitting it
/// this way is what lets the object-safe half of the trait carry a world across a `dyn` boundary
/// without naming a system type (`BD-3`).
pub(crate) struct WorldParts<'a> {
    pub(crate) entities: &'a EntityRegistry,
    pub(crate) components: &'a mut ComponentStore,
    pub(crate) relations: &'a mut RelationStore,
    pub(crate) processes: &'a mut ProcessStore,
    /// The registry, read-only: how an interruption request finds the owner it is delivered to.
    pub(crate) systems: &'a SystemRegistry,
    pub(crate) at: WorldTime,
    /// Why the system being called is running. Supplied by the kernel, never by the system.
    pub(crate) cause: &'a Cause,
    /// What this call asks to happen later, in the order it asked. The world files it in its
    /// schedule once the call returns, so sequence numbers follow the order of asking.
    pub(crate) pending: &'a mut Vec<(WorldTime, Scheduled)>,
    /// Facts another system decided while this call ran — an owner answering an interruption this
    /// call requested — each with the system that decided it. Recorded as that system's once this
    /// call returns, ahead of this call's own facts, because they happened first.
    pub(crate) foreign: &'a mut Vec<(SystemId, Vec<Emission>)>,
    /// How many interruption deliveries deep this call is. Bounded like reduction is.
    pub(crate) depth: usize,
}

impl WorldParts<'_> {
    /// The same parts, reborrowed for one nested call — an owner deciding an interruption.
    fn nested(&mut self) -> WorldParts<'_> {
        WorldParts {
            entities: self.entities,
            components: self.components,
            relations: self.relations,
            processes: self.processes,
            systems: self.systems,
            at: self.at,
            cause: self.cause,
            pending: self.pending,
            foreign: self.foreign,
            depth: self.depth + 1,
        }
    }
}

/// What a running system writes through: its own state, and nothing else.
///
/// The type parameter is the system, and it is the whole mechanism. Every write requires the
/// component type to be [`OwnedBy`] `S`, checked by the compiler, so a system cannot *name* a
/// write to state it does not own (`INV-7`, `KD-1`). The view holds the world's stores privately
/// and offers no way to reach them, so the `&mut` that would defeat every check does not escape
/// the kernel (`BD-2`).
///
/// It also carries the instant dispatch is working in. The world clock is S4's, not a system's:
/// a system is *told* the instant rather than asking a clock, which is what keeps a replayed world
/// producing the same facts as the run it is replaying (`AC-12`).
pub struct WorldView<'a, S: SystemIdentity> {
    parts: WorldParts<'a>,
    token: &'a WriteToken<S>,
}

impl<'a, S: SystemIdentity> WorldView<'a, S> {
    pub(crate) const fn new(parts: WorldParts<'a>, token: &'a WriteToken<S>) -> Self {
        Self { parts, token }
    }

    /// The system this view writes as — the name on the token the world granted at installation.
    pub fn writer(&self) -> SystemId {
        self.token.system()
    }

    /// The instant this call is working in.
    pub const fn at(&self) -> WorldTime {
        self.parts.at
    }

    /// The same world, read-only: every read of [`WorldRead`], with the writes left behind.
    pub fn read(&self) -> WorldRead<'_> {
        WorldRead::new(
            self.parts.entities,
            self.parts.components,
            self.parts.relations,
            self.parts.processes,
        )
    }

    /// Writes an entity's `C`, returning what was there before.
    ///
    /// Only `C`'s owner can call this, and the compiler is what says so.
    pub fn insert<C>(&mut self, entity: EntityId, component: C) -> Result<Option<C>, KernelError>
    where
        C: OwnedBy<S> + 'static,
    {
        self.parts.components.insert(self.token, entity, component)
    }

    /// An entity's `C`, to be changed in place by its owner.
    pub fn component_mut<C>(&mut self, entity: EntityId) -> Result<Option<&mut C>, KernelError>
    where
        C: OwnedBy<S> + 'static,
    {
        self.parts.components.get_mut(self.token, entity)
    }

    /// Removes an entity's `C`, returning it if it was there.
    pub fn remove<C>(&mut self, entity: EntityId) -> Result<Option<C>, KernelError>
    where
        C: OwnedBy<S> + 'static,
    {
        self.parts.components.remove(self.token, entity)
    }

    /// Forms the edge `from` → `to` of an edge type this system declared, and stores it.
    ///
    /// The endpoints are identities rather than records, so the view resolves them against this
    /// world's registry and refuses an entity it never allocated. The edge itself is formed by the
    /// store against the authoritative declaration, so a caller cannot smuggle in one built
    /// against rules it invented.
    pub fn relate(
        &mut self,
        relation_type: &RelationTypeId,
        from: EntityId,
        to: EntityId,
    ) -> Result<Relation, KernelError> {
        let entities = self.parts.entities;
        let from = entities.require(from)?;
        let to = entities.require(to)?;
        self.parts
            .relations
            .insert(self.token, relation_type, from, to)
    }

    /// Removes an edge of a type this system declared, reporting whether it was there.
    pub fn unrelate(&mut self, relation: &Relation) -> Result<bool, KernelError> {
        self.parts.relations.remove(self.token, relation)
    }

    /// Asks for a fact to happen at a strictly later instant.
    ///
    /// The world files it in its schedule when this call returns, and records it at `at` as this
    /// system's fact, caused by whatever this call is handling — the request, the fact or the
    /// process boundary it was handed (step-04 §8 F-2). What is checked here is that the instant
    /// really is later, so that "defer" cannot quietly mean "now" and skip the ordering `D-6`
    /// depends on.
    pub fn defer(&mut self, at: WorldTime, emission: Emission) -> Result<(), KernelError> {
        if at <= self.parts.at {
            return Err(KernelError::DeferralNotInTheFuture {
                at,
                now: self.parts.at,
            });
        }
        let deferral = Deferral::new(at, emission, self.writer(), self.parts.cause);
        self.parts
            .pending
            .push((at, Scheduled::Fact(Box::new(deferral))));
        Ok(())
    }

    /// Starts a process of a kind this system owns, at this instant, and returns its identity.
    ///
    /// `P::Owner = S` is the compiler's check: a system cannot start a process of a kind it does not
    /// own. If the start names an expected end, it must be strictly later than now, and this
    /// system's [`System::wake`](crate::System::wake) is called then.
    pub fn start_process<P>(&mut self, start: ProcessStart<P>) -> Result<ProcessId, KernelError>
    where
        P: ProcessKind<Owner = S>,
    {
        let end = start.expected_end_instant();
        let process = self.parts.processes.start(S::ID, start, self.parts.at)?;
        if let Some(end) = end {
            self.parts.pending.push((end, Scheduled::Wake(process)));
        }
        Ok(process)
    }

    /// Ends a process this system owns, returning its final record. The facts that say it ended are
    /// this system's to emit; ending one records nothing by itself.
    pub fn end_process<P>(&mut self, process: ProcessId) -> Result<Process, KernelError>
    where
        P: ProcessKind<Owner = S>,
    {
        self.parts.processes.end::<P>(process, &S::ID)
    }

    /// Pauses a process this system owns. A suspended process is not woken at its expected end.
    pub fn suspend_process<P>(&mut self, process: ProcessId) -> Result<(), KernelError>
    where
        P: ProcessKind<Owner = S>,
    {
        self.parts
            .processes
            .owned_mut::<P>(process, &S::ID)?
            .set_phase(ProcessPhase::Suspended);
        Ok(())
    }

    /// Sets when a process this system owns is expected to end — or that it has no expected end —
    /// and resumes it if it was suspended. A new end must be strictly later than now; the system is
    /// woken then, and a wake for the old end is ignored when it falls due.
    pub fn reschedule_process<P>(
        &mut self,
        process: ProcessId,
        expected_end: Option<WorldTime>,
    ) -> Result<(), KernelError>
    where
        P: ProcessKind<Owner = S>,
    {
        let now = self.parts.at;
        if let Some(end) = expected_end
            && end <= now
        {
            return Err(KernelError::ProcessEndNotInTheFuture { end, now });
        }
        let record = self.parts.processes.owned_mut::<P>(process, &S::ID)?;
        record.set_expected_end(expected_end);
        record.set_phase(ProcessPhase::Running);
        if let Some(end) = expected_end {
            self.parts.pending.push((end, Scheduled::Wake(process)));
        }
        Ok(())
    }

    /// Replaces this system's own state for a process it owns: the course a dinner is on.
    pub fn set_process_state<P>(
        &mut self,
        process: ProcessId,
        state: Vec<u8>,
    ) -> Result<(), KernelError>
    where
        P: ProcessKind<Owner = S>,
    {
        self.parts
            .processes
            .owned_mut::<P>(process, &S::ID)?
            .set_state(state);
        Ok(())
    }

    /// Asks the owner of a process to interrupt it, and returns what the owner did.
    ///
    /// The request is delivered now, to the owning system's
    /// [`System::interrupt`](crate::System::interrupt), with the owner's own view of the world — so
    /// the owner writes only what the owner owns — and the outcome is read off the world afterwards.
    /// This system learns the answer and changes nothing itself (`INV-7`, SD-6).
    ///
    /// Answered without consulting the owner when the process is not running
    /// ([`InterruptOutcome::NotRunning`]), is uninterruptible, or belongs to a disabled system.
    /// Facts the owner emits while deciding are recorded as the owner's, caused by what this call is
    /// handling, ahead of this call's own facts.
    pub fn request_interrupt(
        &mut self,
        process: ProcessId,
    ) -> Result<InterruptOutcome, KernelError> {
        let Some(record) = self.parts.processes.get(process) else {
            return Ok(InterruptOutcome::NotRunning);
        };
        if record.interruptibility() == Interruptibility::Uninterruptible {
            return Ok(InterruptOutcome::Uninterruptible);
        }
        let owner = record.owner().clone();
        let was_suspended = record.phase() == ProcessPhase::Suspended;
        let systems = self.parts.systems;
        let Some(handler) = systems
            .is_enabled(&owner)
            .then(|| systems.system(&owner))
            .flatten()
        else {
            return Ok(InterruptOutcome::OwnerDisabled);
        };
        if self.parts.depth >= CASCADE_DEPTH_LIMIT {
            return Err(KernelError::InterruptionsTooDeep {
                limit: CASCADE_DEPTH_LIMIT,
                requester: S::ID,
                owner,
            });
        }

        let request = InterruptRequest::new(process, S::ID, self.parts.cause.caused_by.clone());
        let emissions = handler.interrupt(self.parts.nested(), &request)?;
        self.parts.foreign.push((owner, emissions));

        // What the owner *did*, read off the store: a process already suspended and left so has
        // been refused, not suspended again.
        Ok(match self.parts.processes.get(process) {
            None => InterruptOutcome::Ended,
            Some(after) if after.phase() == ProcessPhase::Suspended && !was_suspended => {
                InterruptOutcome::Suspended
            }
            Some(_) => InterruptOutcome::Refused,
        })
    }
}

/// The one thing a system does while it is being installed: declare the tables it owns.
///
/// Narrow on purpose. A system is installed to state what state it holds, not to run, so this
/// exposes declaration and nothing else — no rows, no reads, no edges beyond their type. Initial
/// state belongs to a World Pack (S7) and would otherwise be state no event explains (`INV-15`).
///
/// Every declaration is checked twice: the component type must be one the system's own
/// [`SystemDeclaration`](crate::system::SystemDeclaration) lists, and the store checks that the
/// component's declared owner is the system whose token is declaring it.
pub struct Declarations<'a, S: SystemIdentity> {
    components: &'a mut ComponentStore,
    relations: &'a mut RelationStore,
    token: &'a WriteToken<S>,
    owned: &'a [ComponentDeclaration],
}

impl<'a, S: SystemIdentity> Declarations<'a, S> {
    pub(crate) const fn new(
        components: &'a mut ComponentStore,
        relations: &'a mut RelationStore,
        token: &'a WriteToken<S>,
        owned: &'a [ComponentDeclaration],
    ) -> Self {
        Self {
            components,
            relations,
            token,
            owned,
        }
    }

    /// Declares the table for a component type this system owns.
    ///
    /// Refused if the system's declaration does not list `C`: the declaration is what the registry
    /// checks composition against — one system per component type — so a table outside it would be
    /// state no conflict check ever saw.
    pub fn component<C>(&mut self) -> Result<(), KernelError>
    where
        C: OwnedBy<S> + 'static,
    {
        if !self
            .owned
            .iter()
            .any(|owned| *owned.component_type() == C::COMPONENT_TYPE)
        {
            return Err(KernelError::ComponentTypeNotInSystemDeclaration {
                system: S::ID,
                component_type: C::COMPONENT_TYPE,
            });
        }
        self.components.declare::<C, S>(self.token)
    }

    /// Declares an edge type this system owns.
    ///
    /// Unlike a component type, an edge type is a value rather than a Rust type, so there is no
    /// type-level owner to check and the store compares the declaration's owner with this system at
    /// run time (§2.10.6 of the step document has the evidence for why that asymmetry is forced).
    pub fn relation(&mut self, declaration: RelationTypeDeclaration) -> Result<(), KernelError> {
        self.relations.declare(self.token, declaration)
    }
}
