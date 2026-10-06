//! The single error type every fallible kernel operation returns.
//!
//! One enum rather than one per module, for the same reason the contract layer has one: a
//! caller loading a world, replaying an event log or running a system handles these at the same
//! boundary, and each variant carries the offending values as fields so that the caller can
//! react to them rather than parse a message.
//!
//! Every variant here describes a *refusal*, and a refusal never leaves state half-changed.
//! That is a promise of this crate, not an accident of the current implementation: an operation
//! that can fail validates before it mutates, so a caller that handles the error is looking at
//! the state it had before the call.

use mineworld_contracts::{
    ActionTypeId, ComponentTypeId, ContractError, EntityId, EntityKey, EventTypeId, LifecycleState,
    ProcessId, ProcessTypeId, Rejection, RelationTypeId, SystemId, WorldTime,
};

use thiserror::Error;

use crate::schedule::Sequence;

/// Every way a kernel operation can refuse.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum KernelError {
    /// A contract value refused to be built or decoded. The kernel never repairs one; it
    /// reports what the contract layer said.
    #[error(transparent)]
    Contract(#[from] ContractError),

    /// Two entities were created with the same authoring key. Keys are how authored content
    /// refers to entities, so allowing a second one would make the reference ambiguous.
    #[error("entity key '{key}' is already used by entity {existing}")]
    EntityKeyAlreadyUsed {
        /// The key that was offered a second time.
        key: EntityKey,
        /// The entity that already holds it.
        existing: EntityId,
    },

    /// An authoring key was resolved that no entity in this world holds.
    #[error("no entity in this world has the key '{key}'")]
    UnknownEntityKey {
        /// The key that could not be resolved.
        key: EntityKey,
    },

    /// An operation named an entity this world has never allocated.
    #[error("entity {entity} does not exist in this world")]
    UnknownEntity {
        /// The identity that does not exist.
        entity: EntityId,
    },

    /// Identity allocation reached the top of the identifier space. Reported rather than
    /// wrapped: reusing an identity would break every event that already refers to it.
    #[error("this world has allocated every available entity identity")]
    EntityIdSpaceExhausted,

    /// A second write token was requested for a system that already holds one. Refused rather
    /// than served: two holders would both believe they were the single writer of that system's
    /// state.
    #[error("system '{system}' already holds this world's write token")]
    WriteAccessAlreadyGranted {
        /// The system whose token was requested twice.
        system: SystemId,
    },

    /// A component type was declared with a token belonging to a system other than the one the
    /// component's own declaration names. The declaration and the ownership relationship
    /// disagree, so one of them is a lie and the kernel refuses to pick.
    #[error(
        "component type '{component_type}' is declared as owned by '{declared_owner}', \
         but was registered by '{writing_system}'"
    )]
    ComponentOwnerDisagreesWithDeclaration {
        /// The component type being declared.
        component_type: ComponentTypeId,
        /// The owner the component type's own declaration names.
        declared_owner: SystemId,
        /// The system whose token was used to declare it.
        writing_system: SystemId,
    },

    /// Two systems claimed the same component type. This is the conflict `INV-7` forbids: one
    /// piece of state cannot have two writers.
    #[error(
        "component type '{component_type}' is already owned by '{declared_by}', \
         so '{claimed_by}' cannot claim it"
    )]
    ComponentTypeClaimedByAnotherSystem {
        /// The contested component type.
        component_type: ComponentTypeId,
        /// The system that declared it first.
        declared_by: SystemId,
        /// The system that tried to claim it as well.
        claimed_by: SystemId,
    },

    /// The same component type name was declared twice by one system with a different Rust type
    /// or a different schema version. Each declaration would see an empty table and lose the
    /// other's rows.
    #[error("component type '{component_type}' is already declared in this world")]
    ComponentTypeAlreadyDeclared {
        /// The component type declared a second time.
        component_type: ComponentTypeId,
    },

    /// A write named a component type this world has not declared, so no system in it owns that
    /// state.
    #[error("component type '{component_type}' is not declared in this world")]
    ComponentTypeNotDeclared {
        /// The component type that was written.
        component_type: ComponentTypeId,
    },

    /// An edge type was declared or written by a system other than the one that owns it. The
    /// declaring system is the single writer of its edges, exactly as it is of its components.
    #[error(
        "relation type '{relation_type}' is owned by '{owner}', \
         so '{writing_system}' cannot write it"
    )]
    RelationTypeNotOwned {
        /// The edge type.
        relation_type: RelationTypeId,
        /// The system that owns it.
        owner: SystemId,
        /// The system that tried to write it.
        writing_system: SystemId,
    },

    /// Two systems claimed the same edge type.
    #[error(
        "relation type '{relation_type}' is already owned by '{declared_by}', \
         so '{claimed_by}' cannot claim it"
    )]
    RelationTypeClaimedByAnotherSystem {
        /// The contested edge type.
        relation_type: RelationTypeId,
        /// The system that declared it first.
        declared_by: SystemId,
        /// The system that tried to claim it as well.
        claimed_by: SystemId,
    },

    /// One system declared the same edge type name twice with different rules. Edges already
    /// stored were checked against the first declaration, so the second cannot quietly replace it.
    #[error("relation type '{relation_type}' is already declared in this world with other rules")]
    RelationTypeAlreadyDeclared {
        /// The edge type declared a second time.
        relation_type: RelationTypeId,
    },

    /// An edge named a type no system in this world declared.
    #[error("relation type '{relation_type}' is not declared in this world")]
    RelationTypeNotDeclared {
        /// The undeclared edge type.
        relation_type: RelationTypeId,
    },

    /// The destruction cascade was asked to clear the edges of an entity that is still part of the
    /// world. Removing a live entity's edges is its declaring systems' business, not the kernel's.
    #[error(
        "entity {entity} is {lifecycle}, not destroyed, so its edges are not the kernel's to remove"
    )]
    EntityNotDestroyed {
        /// The entity whose edges were to be removed.
        entity: EntityId,
        /// The lifecycle state it is actually in.
        lifecycle: LifecycleState,
    },

    /// A system declared a table for a component type its own [`SystemDeclaration`] does not
    /// list. The declaration is what a world's composition is checked against — one system per
    /// component type — so a table outside it would be state no conflict check ever saw.
    ///
    /// [`SystemDeclaration`]: crate::system::SystemDeclaration
    #[error(
        "system '{system}' declared a table for component type '{component_type}', \
         which its own declaration does not list"
    )]
    ComponentTypeNotInSystemDeclaration {
        /// The system that declared the table.
        system: SystemId,
        /// The component type it declared.
        component_type: ComponentTypeId,
    },

    /// A system was installed into a world that already has one by that name. Refused rather than
    /// replacing it: the installed system owns state, and a second system under one name would
    /// either inherit state it never wrote or silently orphan it.
    #[error("system '{system}' is already installed in this world")]
    SystemAlreadyInstalled {
        /// The system name claimed a second time.
        system: SystemId,
    },

    /// An operation named a system this world has never installed. Enabling or disabling one is a
    /// statement about a world's composition, and a name that is not part of that composition is a
    /// mistake in the configuration rather than a system that happens to be off.
    #[error("system '{system}' is not installed in this world")]
    SystemNotInstalled {
        /// The name that is not part of this world.
        system: SystemId,
    },

    /// A system declared a dependency on a system this world does not have. Refused by name,
    /// because the name is the whole of the diagnosis: a world pack listed one system and not the
    /// one it needs.
    #[error("system '{system}' depends on '{dependency}', which is not installed in this world")]
    SystemDependencyMissing {
        /// The system whose dependency is absent.
        system: SystemId,
        /// The dependency it declared.
        dependency: SystemId,
    },

    /// A system's declared dependency is installed and disabled. A dependency that does not act is
    /// not a dependency that is present: the dependent would run against state nobody is
    /// maintaining.
    #[error("system '{system}' depends on '{dependency}', which is installed but disabled")]
    SystemDependencyDisabled {
        /// The system whose dependency is off.
        system: SystemId,
        /// The dependency that is off.
        dependency: SystemId,
    },

    /// A system was to be disabled while an enabled system depends on it. Refused naming the
    /// dependent: a world in which an enabled system's declared dependency is disabled is one the
    /// registry would have refused to assemble, and reaching it by disabling is the same mistake
    /// arrived at backwards.
    #[error("system '{system}' cannot be disabled while '{required_by}' depends on it")]
    SystemRequiredByAnotherSystem {
        /// The system that was to be disabled.
        system: SystemId,
        /// The enabled system that depends on it.
        required_by: SystemId,
    },

    /// Two systems provide the same action type. One request must have one answer: dispatch routes
    /// an action to exactly one system, and `INV-10`'s answer — *no enabled system provides this* —
    /// has to be decidable without asking which of two systems meant it.
    #[error(
        "action type '{action_type}' is already provided by '{provided_by}', \
         so '{claimed_by}' cannot provide it"
    )]
    ActionTypeProvidedByAnotherSystem {
        /// The contested action type.
        action_type: ActionTypeId,
        /// The system that provides it.
        provided_by: SystemId,
        /// The system that tried to provide it as well.
        claimed_by: SystemId,
    },

    /// A system's `declaration()` handed back a declaration belonging to another system. Nothing in
    /// the type system stops that — `SystemDeclaration::of::<Another>()` is an ordinary expression —
    /// so installation compares the two, because a declaration is what ownership conflicts are
    /// decided from and a system must not be able to claim another's.
    #[error("system '{system}' returned a declaration belonging to '{declared}'")]
    SystemDeclarationNamesAnotherSystem {
        /// The system being installed.
        system: SystemId,
        /// The system its declaration named.
        declared: SystemId,
    },

    /// A system's declaration claims a component type, and the system did not declare a table for
    /// it while being installed. The two halves of ownership would then disagree: the registry
    /// would refuse another system's claim on state that does not exist, and a write to it would be
    /// refused as undeclared.
    #[error(
        "system '{system}' declares that it owns component type '{component_type}' \
         but did not declare its table"
    )]
    SystemDidNotDeclareOwnedComponent {
        /// The system being installed.
        system: SystemId,
        /// The component type it claimed and did not declare.
        component_type: ComponentTypeId,
    },

    /// An action was routed to the system that provides it, and that system has no resolution for
    /// it. Refused rather than accepted with no events: a system that provides an action and does
    /// not resolve it is a bug in that system, and a silent acceptance would hide it behind a
    /// world that looks like it worked.
    #[error("system '{system}' provides action '{action_type}' but does not resolve it")]
    ActionNotResolvedBySystem {
        /// The system the action was routed to.
        system: SystemId,
        /// The action it did not resolve.
        action_type: ActionTypeId,
    },

    /// Identity allocation for recorded facts reached the top of the identifier space. Reported
    /// rather than wrapped, for the reason entity identity is: reusing an identity would make two
    /// different facts the same fact in every log that refers to them.
    #[error("this world has recorded every available event identity")]
    EventIdSpaceExhausted,

    /// A system emitted a fact of a kind its own declaration does not list. The declaration is what
    /// a reader of a world's composition goes by — which facts this world can produce, and from
    /// whom — so a fact outside it would make that reading wrong.
    #[error(
        "system '{system}' emitted event type '{event_type}', \
         which its own declaration does not list"
    )]
    EventTypeNotInSystemDeclaration {
        /// The emitting system.
        system: SystemId,
        /// The kind of fact it emitted.
        event_type: EventTypeId,
    },

    /// A system declares that it emits a kind of fact whose vocabulary another system owns, and does
    /// not declare a dependency on that owner (`DECISIONS.md` `ARC-26`).
    ///
    /// Stating a fact in another system's vocabulary is how a system asks the owner to change state
    /// the owner alone writes. Without the dependency the owner may be absent, and the fact would
    /// be recorded with nothing to reduce it — history claiming a change that no state reflects.
    #[error(
        "system '{system}' declares that it emits '{event_type}', owned by '{owner}', \
         without depending on '{owner}'"
    )]
    EmittedEventOwnerNotADependency {
        /// The system being installed.
        system: SystemId,
        /// The kind of fact it declared it emits.
        event_type: EventTypeId,
        /// The system that owns that kind of fact.
        owner: SystemId,
    },

    /// The system that owns a kind of fact refused to reduce one into its state, because the state
    /// may not take that value (`DECISIONS.md` `ARC-26`: the owner still decides).
    ///
    /// A system that states a fact in an owner's vocabulary is required to build it through the
    /// owner's checked constructor, which refuses the same value as an ordinary rejection before
    /// anything is recorded. Reaching this error therefore means the stating system broke that
    /// contract, and — like every error out of reduction — it is a bug in a system rather than a
    /// refused request. The owner wrote nothing for the refused fact.
    #[error("system '{system}' refused to reduce '{event_type}': {reason:?}")]
    FactRefusedByOwner {
        /// The owner that refused.
        system: SystemId,
        /// The kind of fact it refused.
        event_type: EventTypeId,
        /// Why the state may not take the value, in the contract's own vocabulary.
        reason: Rejection,
    },

    /// Reaction within one logical instant went deeper than the cascade limit: systems kept
    /// reacting to each other's facts without the clock ever moving.
    ///
    /// Named rather than silent, because an infinite cascade is a system bug and a world that
    /// freezes gives its author nothing to go on. The systems listed are those that emitted while
    /// reducing in the second half of the budget, in registration order — every member of a cycle
    /// of period up to half the limit, and not a system that answered once, early in the chain, and
    /// stopped.
    ///
    /// Unlike every other refusal in this crate, this one is reported after state has changed:
    /// reduction is not transactional. See [`crate::dispatch`].
    #[error(
        "reduction within one instant exceeded {limit} generations; \
         these systems kept emitting while reducing: {systems:?}"
    )]
    ReductionCascadeTooDeep {
        /// The limit that was exceeded.
        limit: usize,
        /// The systems still emitting when the limit was crossed, in registration order.
        systems: Vec<SystemId>,
    },

    /// A system asked for a fact to happen at an instant that is not later than the one it is
    /// running in. Deferral means *later*: a fact for the current instant is emitted, not
    /// deferred, and blurring the two would lose the ordering replay depends on (`D-6`).
    #[error("a deferral to {at} is not later than the current instant {now}")]
    DeferralNotInTheFuture {
        /// The instant the system asked for.
        at: WorldTime,
        /// The instant it is running in.
        now: WorldTime,
    },

    /// A world was asked to state a genesis fact after it had already dispatched something.
    ///
    /// Genesis is world *assembly*: [`Causation::WorldGenesis`](mineworld_contracts::Causation)
    /// means "the world coming into existence", and a world that could state an uncaused fact
    /// while running would make that claim untrue of its own log. Initial state is stated once,
    /// before the world runs; everything afterwards is caused by a request, a process or another
    /// fact.
    #[error(
        "this world has already dispatched a request, so it cannot state {facts} more \
         genesis fact(s): initial state is stated before a world runs"
    )]
    GenesisAfterTheWorldHasRun {
        /// How many facts were offered too late.
        facts: usize,
    },

    /// A genesis fact named an event type whose owning system this world has not installed.
    ///
    /// The owner is read off the event type's own contract rather than supplied by the caller, so
    /// this is not a mislabelling: it is a world being asked to begin with a fact that belongs to
    /// a vocabulary it was not composed with. Nothing would reduce it, so the state it describes
    /// would never exist.
    #[error(
        "genesis fact of type '{event_type}' belongs to system '{system}', \
         which this world has not installed"
    )]
    GenesisFactHasNoInstalledOwner {
        /// The system that owns the event type.
        system: SystemId,
        /// The kind of fact that was offered.
        event_type: EventTypeId,
    },

    /// A world was asked to move its clock to an instant before the one it is at. A fact recorded
    /// after a later one would be a history that runs backwards.
    #[error("this world is at {now}; it cannot move back to {at}")]
    ClockWouldMoveBackwards {
        /// The instant the world is at.
        now: WorldTime,
        /// The earlier instant it was asked to move to.
        at: WorldTime,
    },

    /// A request was dispatched at an instant at or after which scheduled work is still due.
    ///
    /// Refused rather than run, because the request would overtake work that was due first and the
    /// `(WorldTime, Sequence)` order replay depends on would be lost. The caller advances the world
    /// to the request's instant first ([`World::advance_to`](crate::World::advance_to)).
    #[error("work scheduled for {due} is still due; advance the world before dispatching at {at}")]
    ScheduledWorkDue {
        /// The earliest instant something is still scheduled for.
        due: WorldTime,
        /// The instant the request was to be dispatched at.
        at: WorldTime,
    },

    /// The schedule assigned every available sequence number. Reported rather than wrapped: a
    /// reused sequence would leave two entries with no defined order between them.
    #[error("this world's schedule has assigned every available sequence number")]
    ScheduleSequenceExhausted,

    /// A saved schedule was restored into a world that has already run. Restoring is assembly:
    /// it states where a world's time stands, and a world that has acted has a time of its own.
    #[error("this world has already run, so a saved schedule cannot replace its time")]
    RestoreAfterTheWorldHasRun,

    /// A saved schedule held an entry due before the instant it was saved at — work a world would
    /// already have fired.
    #[error("persisted schedule holds an entry due at {at}, before its own instant {now}")]
    PersistedEntryBeforeNow {
        /// When the entry was due.
        at: WorldTime,
        /// The instant the schedule was saved at.
        now: WorldTime,
    },

    /// A saved schedule held a sequence number its own counter had not yet assigned, or the
    /// never-assigned zero.
    #[error(
        "persisted schedule holds sequence {sequence}, which its counter (next {next}) never assigned"
    )]
    PersistedSequenceOutsideCounter {
        /// The sequence number found.
        sequence: Sequence,
        /// The next sequence number the saved schedule would assign.
        next: u64,
    },

    /// A saved schedule held one sequence number on two entries.
    #[error("persisted schedule holds sequence {sequence} twice")]
    PersistedSequenceRepeated {
        /// The repeated sequence number.
        sequence: Sequence,
    },

    /// A saved schedule's counter is below the first sequence number a schedule assigns.
    #[error("persisted schedule would next assign {next}, below the first sequence {first}")]
    PersistedSequenceCounterTooLow {
        /// The next sequence number the saved schedule would assign.
        next: u64,
        /// The first sequence number a schedule assigns.
        first: u64,
    },

    /// A saved world's event counter is below the first event identity, so restoring it would hand
    /// out identity zero.
    #[error("persisted event counter would next allocate {next}, below the first identity {first}")]
    PersistedEventCounterTooLow {
        /// The next event identity the saved world would allocate.
        next: u64,
        /// The first event identity a world allocates.
        first: u64,
    },

    /// A saved schedule holds work for a system this world has not installed. Nothing could fire it
    /// as that system, so the fact it describes would never happen.
    #[error(
        "persisted schedule holds work for system '{system}', which this world has not installed"
    )]
    PersistedEntryNamesUninstalledSystem {
        /// The system the entry belongs to.
        system: SystemId,
    },

    /// A process was given an expected end that is not later than the instant it is in. A process
    /// takes time; something that starts and ends in one instant is an event (`INV-3`).
    #[error(
        "a process cannot be expected to end at {end}: it is {now} now, and a process takes time"
    )]
    ProcessEndNotInTheFuture {
        /// The expected end that was asked for.
        end: WorldTime,
        /// The instant the world is at.
        now: WorldTime,
    },

    /// Process identity allocation reached the top of the identifier space.
    #[error("this world has allocated every available process identity")]
    ProcessIdSpaceExhausted,

    /// An operation named a process that is not running or suspended in this world: it has ended,
    /// or it never existed.
    #[error("process {process} is not running in this world")]
    ProcessNotRunning {
        /// The process named.
        process: ProcessId,
    },

    /// A system tried to change a process another system owns. The owner is the single writer of
    /// its processes (`INV-7`); anyone else asks it to interrupt instead.
    #[error("process {process} is owned by '{owner}', so '{writing_system}' cannot change it")]
    ProcessNotOwned {
        /// The process.
        process: ProcessId,
        /// The system that owns it.
        owner: SystemId,
        /// The system that tried to change it.
        writing_system: SystemId,
    },

    /// A process was addressed as a kind it is not — its state read, or its owner acting on it,
    /// through the wrong kind. Refused rather than decoded on the chance that the bytes fit.
    #[error("process {process} is a '{actual}', not a '{expected}'")]
    ProcessKindMismatch {
        /// The process.
        process: ProcessId,
        /// The kind it was addressed as.
        expected: ProcessTypeId,
        /// The kind it is.
        actual: ProcessTypeId,
    },

    /// A system started a process with an expected end and has no answer when that end comes. A
    /// world that let the process run on silently would hide the bug.
    #[error(
        "system '{system}' started process {process} with an expected end but does not wake it"
    )]
    ProcessNotWokenBySystem {
        /// The owning system.
        system: SystemId,
        /// The process whose end arrived.
        process: ProcessId,
    },

    /// Interruption requests nested deeper than the cascade limit: owners deciding interruptions by
    /// requesting interruptions of their own, without end.
    #[error(
        "interruption requests nested deeper than {limit}: '{requester}' asked '{owner}' while \
         already answering a request"
    )]
    InterruptionsTooDeep {
        /// The limit that was exceeded.
        limit: usize,
        /// The system whose request crossed the limit.
        requester: SystemId,
        /// The owner it would have been delivered to.
        owner: SystemId,
    },

    /// A saved world's process counter is below the first process identity.
    #[error(
        "persisted process counter would next allocate {next}, below the first identity {first}"
    )]
    PersistedProcessCounterTooLow {
        /// The next process identity the saved world would allocate.
        next: u64,
        /// The first process identity a world allocates.
        first: u64,
    },

    /// A saved world held a process whose identity its own counter had not yet allocated.
    #[error("persisted process {process} was never allocated by its counter (next {next})")]
    PersistedProcessOutsideCounter {
        /// The process found.
        process: ProcessId,
        /// The next process identity the saved world would allocate.
        next: u64,
    },

    /// A saved world held two processes under one identity.
    #[error("persisted processes hold identity {process} twice")]
    PersistedProcessRepeated {
        /// The repeated identity.
        process: ProcessId,
    },

    /// A persisted graph filed an edge type's declaration under a different name.
    #[error("persisted relation type declaration at '{at}' calls itself '{found}'")]
    PersistedRelationTypeMismatch {
        /// The name it was filed under.
        at: RelationTypeId,
        /// The name the declaration itself carries.
        found: RelationTypeId,
    },

    /// A persisted graph held an undirected edge with its ends the wrong way round. Stored that
    /// way, one fact would read as two edges.
    #[error(
        "persisted undirected '{relation_type}' edge holds {from} and {to} in non-canonical order"
    )]
    PersistedRelationNotCanonical {
        /// The undirected edge type.
        relation_type: RelationTypeId,
        /// The end stored as `from`.
        from: EntityId,
        /// The end stored as `to`.
        to: EntityId,
    },

    /// A persisted registry held an entity under an identity other than its own — the record
    /// and its key disagree, so one of them is wrong and the kernel cannot tell which.
    #[error("persisted entity record at {at} carries identity {found}")]
    PersistedEntityIdMismatch {
        /// The identity the record was stored under.
        at: EntityId,
        /// The identity the record itself claims.
        found: EntityId,
    },

    /// A persisted registry would allocate an identity it has already used. Loading it would
    /// break the promise that an identity is never reused.
    #[error(
        "persisted registry would next allocate {next}, which entity {allocated} already holds"
    )]
    PersistedIdWouldBeReused {
        /// The next identity the persisted registry would hand out.
        next: u64,
        /// The entity that already holds it.
        allocated: EntityId,
    },

    /// A persisted registry's counter is below the first identity a world allocates, so loading
    /// it would hand out identity 0 — the value this crate documents as never allocated.
    #[error("persisted registry would next allocate {next}, below the first identity {first}")]
    PersistedIdCounterTooLow {
        /// The next identity the persisted registry would hand out.
        next: u64,
        /// The first identity a world allocates.
        first: u64,
    },

    /// A persisted registry held one authoring key on two entities.
    #[error("persisted registry holds key '{key}' on both entity {first} and entity {second}")]
    PersistedEntityKeyRepeated {
        /// The repeated key.
        key: EntityKey,
        /// The first entity holding it.
        first: EntityId,
        /// The second entity holding it.
        second: EntityId,
    },

    /// A component could not be written down for a snapshot. A component type whose `Serialize`
    /// fails is a bug in the system that declared it; the snapshot is refused rather than taken
    /// without that row.
    #[error("component '{component_type}' of entity {entity} cannot be encoded: {detail}")]
    ComponentNotEncodable {
        /// The component type.
        component_type: ComponentTypeId,
        /// The entity it belongs to.
        entity: EntityId,
        /// What the encoder said.
        detail: String,
    },

    /// A world can only be restored while it is being assembled: systems installed, and nothing
    /// else — no entity, no component row, no edge, no clock. Restoring over state would leave two
    /// worlds' state mixed in one.
    #[error("a snapshot can only be restored into a world with systems installed and nothing else")]
    RestoreIntoPopulatedWorld,

    /// The snapshot was taken of a world composed differently from this one: another system, a
    /// different declaration or version, another enabled state, or the same systems in another
    /// order — which is another reduction order, and therefore another world (`BD-4`).
    #[error(
        "the snapshot's composition differs at position {position}: saved {saved:?}, installed {installed:?}"
    )]
    RestoredCompositionDiffers {
        /// The registration position of the first difference.
        position: usize,
        /// The system the snapshot has there, if any.
        saved: Option<SystemId>,
        /// The system this world has there, if any.
        installed: Option<SystemId>,
    },

    /// A snapshot held a row of a component type this world has no table for: state of a system
    /// this world is not composed of.
    #[error("persisted component '{component_type}' of entity {entity} has no table in this world")]
    PersistedComponentTypeNotInstalled {
        /// The component type.
        component_type: ComponentTypeId,
        /// The entity the row belongs to.
        entity: EntityId,
    },

    /// A snapshot held a component row for an entity its own registry does not have.
    #[error(
        "persisted component '{component_type}' belongs to entity {entity}, which does not exist"
    )]
    PersistedComponentForUnknownEntity {
        /// The component type.
        component_type: ComponentTypeId,
        /// The missing entity.
        entity: EntityId,
    },

    /// A snapshot held two rows of one component type for one entity.
    #[error("persisted component '{component_type}' appears twice for entity {entity}")]
    PersistedComponentRepeated {
        /// The component type.
        component_type: ComponentTypeId,
        /// The entity.
        entity: EntityId,
    },

    /// A snapshot row did not decode as the component type its table was declared with.
    #[error("persisted component '{component_type}' of entity {entity} does not decode: {detail}")]
    PersistedComponentUndecodable {
        /// The component type.
        component_type: ComponentTypeId,
        /// The entity.
        entity: EntityId,
        /// What the decoder said.
        detail: String,
    },

    /// A snapshot declared relation types differently from the systems installed in this world.
    #[error(
        "the snapshot's relation type declarations differ from this world's at '{relation_type}'"
    )]
    RestoredRelationDeclarationsDiffer {
        /// The first relation type whose declaration differs or is missing on one side.
        relation_type: RelationTypeId,
    },

    /// A snapshot held an edge whose endpoint its own registry does not have.
    #[error("persisted '{relation_type}' edge touches entity {entity}, which does not exist")]
    PersistedRelationForUnknownEntity {
        /// The edge's relation type.
        relation_type: RelationTypeId,
        /// The missing endpoint.
        entity: EntityId,
    },
}
