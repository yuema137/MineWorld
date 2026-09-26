//! The System interface: what a system declares about itself, and the four things it is asked to
//! do.
//!
//! `docs/CORE_CONCEPTS.md` §13 defines a System as *an enabled physics process* that declares its
//! identity, its version, its dependencies, the components it owns, the actions it provides and
//! the events it emits and subscribes to. This module is that declaration as Rust, plus the trait
//! whose four methods are the whole of a system's behaviour:
//!
//! ```text
//! install    declare the tables this system owns, once, when it joins a world
//! validate   read state and decide whether a request is admissible — no writes, by type
//! resolve    decide the outcome and return the facts it caused
//! react      apply a fact this system subscribed to, into the state it owns
//! ```
//!
//! # Why a declaration is a value and identity is a type
//!
//! [`SystemIdentity::ID`] is an associated constant, because component ownership is expressed
//! against the system *type* (`INV-7`, see [`crate::access`]). Everything else a system declares
//! is a **value**, produced once at installation (`BD-3`), because a registry cannot hold types:
//! it holds [`SystemDeclaration`]s and compares them — one system per component type, one system
//! per action type, and every declared dependency present.
//!
//! The two halves cannot disagree. [`SystemDeclaration::of`] is the only constructor and it reads
//! the name and the version off the type, so a declaration that claims another system's identity
//! is not something a system can write; the kernel checks the claim once more at installation,
//! because `of::<Another>()` is still a sentence a system could utter.
//!
//! # What a system may not do
//!
//! A system never sees the issuer of write capability, never holds another system's token, and is
//! never handed a `&mut ComponentStore` — only a [`WorldView`] whose writes are gated on
//! [`OwnedBy`](crate::access::OwnedBy) (`BD-1`, `BD-2`). A `&mut` to the store would permit
//! `*store = ComponentStore::new()`, which no ownership check of any kind can prevent, so what
//! dispatch passes is an `INV-7` decision rather than an ergonomic one.
//!
//! Nothing here knows a single action, event or component name. `give_item`, `WageDue` and
//! `Inventory` are System Pack vocabulary (`INV-12`); this module is the machinery a pack declares
//! them with.

#[cfg(test)]
mod tests;

use mineworld_contracts::{
    Action, ActionIntent, ActionTypeId, Component, ComponentDeclaration, ComponentTypeId, EntityId,
    Event, EventEnvelope, EventRecord, EventTypeId, PlaceId, Rejection, SystemId, Visibility,
    WorldTime,
};
use serde::{Deserialize, Serialize};

use crate::access::{SystemIdentity, WriteToken};
use crate::components::ComponentStore;
use crate::error::KernelError;
use crate::relations::RelationStore;
use crate::view::{Declarations, WorldParts, WorldRead, WorldView};

/// Which version of its own contract a system is.
///
/// A plain counter rather than a semantic-version triple, deliberately: the only question the
/// kernel and a migration ever ask is *which of these two is older*, and a richer scheme would be
/// an abstraction with no second implementation behind it yet
/// (`docs/ENGINEERING_STANDARDS.md` §28). A system that changes the schema of state it owns
/// raises this number, and S5's migration schema compares them.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct SystemVersion(u32);

impl SystemVersion {
    /// Declares a version. A system's first version is 1 by convention; the numbers carry no
    /// meaning beyond their order.
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    /// The version as a number, for a migration that has to compare or step through them.
    pub const fn get(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for SystemVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Everything a world needs to know about a system without running it.
///
/// This is the `docs/CORE_CONCEPTS.md` §13 declaration, and it is what composability is checked
/// from: two systems claiming one component type, a dependency nobody installed, two systems
/// providing one action — all three are decidable from declarations alone, before any world runs.
///
/// Two fields of §13's list are deliberately absent. The configuration schema arrives with world
/// packs (S7) and the migration schema with persistence (S5); adding either now would be a shape
/// guessed at before the layer that reads it exists.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SystemDeclaration {
    system: SystemId,
    version: SystemVersion,
    depends_on: Vec<SystemId>,
    owns: Vec<ComponentDeclaration>,
    provides: Vec<ActionTypeId>,
    emits: Vec<EventTypeId>,
    subscribes: Vec<EventTypeId>,
}

impl SystemDeclaration {
    /// Begins the declaration of `S`: its name and version, and nothing else claimed yet.
    ///
    /// The only constructor, and it takes no name, so a declaration cannot state an identity its
    /// type does not have. Everything a system claims is then added by the typed builders below,
    /// each of which reads the claim off a Rust type rather than taking a string — so a component
    /// this system does not own, or an action type spelled differently from the one a client
    /// submits, is a compile error rather than a world that quietly routes nothing.
    pub fn of<S: System>() -> Self {
        Self {
            system: S::ID,
            version: S::VERSION,
            depends_on: Vec::new(),
            owns: Vec::new(),
            provides: Vec::new(),
            emits: Vec::new(),
            subscribes: Vec::new(),
        }
    }

    /// Declares the systems this one needs in the world. The registry refuses to install or
    /// enable a system whose dependency is absent or disabled, naming it.
    #[must_use]
    pub fn depending_on(mut self, systems: impl IntoIterator<Item = SystemId>) -> Self {
        self.depends_on.extend(systems);
        self
    }

    /// Declares a component type this system owns, and therefore the only state it may write.
    #[must_use]
    pub fn owning<C: Component>(mut self) -> Self {
        self.owns.push(ComponentDeclaration::of::<C>());
        self
    }

    /// Declares an action this system provides. An action no enabled system provides is answered
    /// [`ActionResult::Unavailable`](mineworld_contracts::ActionResult::Unavailable) (`INV-10`).
    #[must_use]
    pub fn providing<A: Action>(mut self) -> Self {
        self.provides.push(A::ACTION_TYPE);
        self
    }

    /// Declares a kind of fact this system emits. Emitting an undeclared event type is refused:
    /// the declaration is what a reader of a world's composition goes by, so it must be true.
    #[must_use]
    pub fn emitting<E: Event>(mut self) -> Self {
        self.emits.push(E::EVENT_TYPE);
        self
    }

    /// Declares a kind of fact this system reacts to, including its own — which is how a system
    /// reduces the facts it emitted into the state it owns.
    #[must_use]
    pub fn subscribing_to<E: Event>(mut self) -> Self {
        self.subscribes.push(E::EVENT_TYPE);
        self
    }

    /// Whose declaration this is.
    pub const fn system(&self) -> &SystemId {
        &self.system
    }

    /// Which version of its contract the system declared.
    pub const fn version(&self) -> SystemVersion {
        self.version
    }

    /// The systems this one needs, in declaration order.
    pub fn depends_on(&self) -> &[SystemId] {
        &self.depends_on
    }

    /// The component types this system owns.
    pub fn owns(&self) -> &[ComponentDeclaration] {
        &self.owns
    }

    /// The actions this system provides.
    pub fn provides(&self) -> &[ActionTypeId] {
        &self.provides
    }

    /// The kinds of fact this system emits.
    pub fn emits(&self) -> &[EventTypeId] {
        &self.emits
    }

    /// The kinds of fact this system reacts to.
    pub fn subscribes(&self) -> &[EventTypeId] {
        &self.subscribes
    }

    /// Whether this system provides that action.
    pub fn provides_action(&self, action_type: &ActionTypeId) -> bool {
        self.provides.iter().any(|provided| provided == action_type)
    }

    /// Whether this system declared that it emits that kind of fact.
    pub fn emits_event(&self, event_type: &EventTypeId) -> bool {
        self.emits.iter().any(|emitted| emitted == event_type)
    }

    /// Whether this system reacts to that kind of fact.
    pub fn subscribes_to(&self, event_type: &EventTypeId) -> bool {
        self.subscribes
            .iter()
            .any(|subscribed| subscribed == event_type)
    }

    /// Whether this system owns that component type.
    pub fn owns_component(&self, component_type: &ComponentTypeId) -> bool {
        self.owns
            .iter()
            .any(|owned| owned.component_type() == component_type)
    }
}

/// A fact a system has decided on, before the kernel places it in history.
///
/// Not a synonym for an [`EventEnvelope`] — it is the part of one a *system* decides: the payload,
/// who could have learned of it, and which entities it is about. The four fields a system must not
/// choose are supplied by the kernel when the emission is recorded:
///
/// ```text
/// id            allocated by the world, monotonic, never reused
/// at            the instant dispatch was told to work in — the clock is S4's, not a system's
/// caused_by     the action or the parent event; a system cannot misstate why a fact happened
/// provenance    the emitting system, and the controller decision the chain started from
/// ```
///
/// That split is what makes `INV-15` and `AC-9` mechanical rather than remembered: causation is
/// the kernel's statement about its own pipeline, so no system can emit a fact that claims to
/// have been caused by something else.
///
/// The payload is bytes the emitting system encoded. The kernel never interprets them and never
/// chooses their encoding: a payload's format is a contract between the system that declares the
/// event type and whoever reads it back, exactly as `ComponentRecord`'s is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Emission {
    payload: EventRecord,
    visibility: Visibility,
    subjects: Vec<EntityId>,
    participants: Vec<EntityId>,
    place: Option<PlaceId>,
}

impl Emission {
    /// The two things only the emitting system knows: what happened, and who could have learned
    /// of it.
    ///
    /// The event type is read off `E` rather than taken as an argument, so an emission cannot be
    /// mislabelled, and the audience is required rather than defaulted — an event that did not say
    /// who could perceive it would default to omniscience, which is what `INV-13` forbids.
    pub fn new<E: Event>(payload: Vec<u8>, visibility: Visibility) -> Self {
        Self {
            payload: EventRecord::new::<E>(payload),
            visibility,
            subjects: Vec::new(),
            participants: Vec::new(),
            place: None,
        }
    }

    /// Names the entities the fact is about.
    #[must_use]
    pub fn about(mut self, subjects: Vec<EntityId>) -> Self {
        self.subjects = subjects;
        self
    }

    /// Names the entities that took part, which is also the audience
    /// [`Visibility::Participants`] refers to.
    #[must_use]
    pub fn with_participants(mut self, participants: Vec<EntityId>) -> Self {
        self.participants = participants;
        self
    }

    /// Records where in the world's semantic space the fact happened.
    #[must_use]
    pub fn at_place(mut self, place: PlaceId) -> Self {
        self.place = Some(place);
        self
    }

    /// What kind of fact this is — what the kernel checks against the emitting system's
    /// declaration.
    pub const fn event_type(&self) -> &EventTypeId {
        self.payload.event_type()
    }

    /// Who could have learned of it.
    pub const fn visibility(&self) -> &Visibility {
        &self.visibility
    }
}

/// A fact a system wants to happen *later*, handed back for the scheduler to queue.
///
/// The seam `D-6` requires and `BD-7` keeps narrow. Reaction is synchronous within the logical
/// instant — a wage paid in the same instant it falls due — and anything that must happen later is
/// queued at a strictly later `(WorldTime, sequence)`. This kernel does not own that queue: S4
/// does, and dispatch therefore returns deferrals unrecorded and unreduced rather than inventing
/// a scheduler here. What the kernel does enforce is that the instant is genuinely later, so a
/// system cannot use deferral to mean *now*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Deferral {
    at: WorldTime,
    emission: Emission,
}

impl Deferral {
    pub(crate) const fn new(at: WorldTime, emission: Emission) -> Self {
        Self { at, emission }
    }

    /// When the emitting system asked for it.
    pub const fn at(&self) -> WorldTime {
        self.at
    }

    /// The fact that is to happen then.
    pub const fn emission(&self) -> &Emission {
        &self.emission
    }
}

/// An installable system: an enabled process that owns some state and answers for some actions.
///
/// Implementing this trait is what makes a type a system. It requires [`SystemIdentity`], so a
/// system's name is part of its type and component ownership can be expressed against it
/// (`INV-7`); it is `Sized`, because a system is handed a [`WorldView`] parameterized by its own
/// type; and it is `'static`, because a world holds its systems for as long as it runs. Being
/// `Sized` also states plainly that there is no `dyn System` — the registry holds the object-safe
/// [`DynSystem`] half instead (`BD-3`).
///
/// ```
/// use mineworld_contracts::SystemId;
/// use mineworld_kernel::{System, SystemDeclaration, SystemIdentity, SystemVersion};
///
/// /// A system that owns nothing yet and answers for nothing: legal, and the smallest system
/// /// there is.
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
/// ```
///
/// The methods take `&self`, not `&mut self`, and that is `INV-7` again rather than an oversight:
/// a system's mutable state is the components it owns, held in the world and written through a
/// gated view. A system that kept simulation state in its own fields would be state no event log
/// could reconstruct and no snapshot could carry.
pub trait System: SystemIdentity + Sized + 'static {
    /// Which version of its own contract this system is.
    const VERSION: SystemVersion;

    /// What this system declares about itself, built with [`SystemDeclaration::of`].
    ///
    /// Called once, at installation, and the value is kept: a world's composition must not be
    /// able to change under it while it runs.
    fn declaration(&self) -> SystemDeclaration;

    /// Declares the tables this system owns, once, as it joins a world.
    ///
    /// This hook exists because the two halves of a declaration live in different places:
    /// [`SystemDeclaration::owns`] is a list of *values* the registry compares, while creating a
    /// table needs the component's Rust **type**, which only the owning system's crate can name.
    /// The kernel checks the two against each other — a table this system's declaration does not
    /// list is refused, and a declared component type with no table is refused as well.
    ///
    /// Only declarations belong here. There is no access to rows: initial state is a World Pack's
    /// business (S7), and a system that wrote state while being installed would write facts no
    /// event explains.
    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        let _ = tables;
        Ok(())
    }

    /// Decides whether a request is admissible against current authoritative state.
    ///
    /// The view is read-only, which is the point of `BD-6`: validation must be free of side
    /// effects or a rejected action leaves debris, and making that the type's business rather than
    /// a rule in a document means a system cannot break it by mistake.
    ///
    /// The default admits everything, because rejection is a system's own judgement and a system
    /// that declares no action is never asked. Returning [`Rejection`] is an ordinary answer, not
    /// a failure: `docs/CORE_CONCEPTS.md` §12.1 makes it a first-class result a client renders.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        let _ = (world, intent);
        Ok(())
    }

    /// Decides the outcome of an admissible request and returns the facts it caused.
    ///
    /// The view is writable, gated on the components this system owns. The default **refuses**,
    /// naming the action: a system that provides an action and does not resolve it is a bug in
    /// that system, and a silent `Accepted` with no events would hide it behind a world that looks
    /// like it worked. A system that provides no actions never reaches this method at all.
    fn resolve(
        &self,
        world: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let _ = world;
        Err(KernelError::ActionNotResolvedBySystem {
            system: Self::ID,
            action_type: intent.action_type().clone(),
        })
    }

    /// Applies a fact this system subscribed to, into the state it owns, and returns any further
    /// facts that follow.
    ///
    /// This is the reducer of `docs/ARCHITECTURE.md` §3, and the only place a cross-domain effect
    /// legitimately lands: `EmploymentSystem` emits `WageDue`, and `EconomySystem` — the owner of
    /// the balance — reacts by writing its own component. Reactions are synchronous within the
    /// instant and run in registration order (`BD-7`, `D-6`).
    ///
    /// The default does nothing, for the many systems that subscribe to nothing.
    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let _ = (world, event);
        Ok(Vec::new())
    }
}

/// What the registry can ask of an installed system without naming its type.
///
/// `System` carries an associated constant, so it is not object safe and a registry cannot hold
/// `dyn System` (`BD-3`). This is the object-safe half, and it is crate-private on purpose: a pack
/// implements [`System`], and nothing outside this crate has any reason to name the erased form or
/// the [`WorldParts`] its methods carry.
pub(crate) trait DynSystem {
    fn install(
        &self,
        components: &mut ComponentStore,
        relations: &mut RelationStore,
        owned: &[ComponentDeclaration],
    ) -> Result<(), KernelError>;

    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection>;

    fn resolve(
        &self,
        parts: WorldParts<'_>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError>;

    fn react(
        &self,
        parts: WorldParts<'_>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError>;
}

/// One system as a world holds it: the system itself, and the write token this world granted it.
///
/// The token lives here rather than being minted where it is used, because that is what makes
/// `BD-1` true in the code and not merely in the documentation: a token enters the world exactly
/// once, from the world's single [`WriteAccess`](crate::access::WriteAccess) at installation, and
/// the only thing that ever holds it afterwards is this wrapper — which hands it to the store, via
/// a [`WorldView`], while the system runs. The system itself never sees a token and never sees the
/// issuer.
pub(crate) struct InstalledSystem<T: System> {
    system: T,
    token: WriteToken<T>,
}

impl<T: System> InstalledSystem<T> {
    pub(crate) const fn new(system: T, token: WriteToken<T>) -> Self {
        Self { system, token }
    }
}

impl<T: System> DynSystem for InstalledSystem<T> {
    fn install(
        &self,
        components: &mut ComponentStore,
        relations: &mut RelationStore,
        owned: &[ComponentDeclaration],
    ) -> Result<(), KernelError> {
        let mut tables = Declarations::new(components, relations, &self.token, owned);
        self.system.install(&mut tables)
    }

    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        self.system.validate(world, intent)
    }

    fn resolve(
        &self,
        parts: WorldParts<'_>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let mut view = WorldView::new(parts, &self.token);
        self.system.resolve(&mut view, intent)
    }

    fn react(
        &self,
        parts: WorldParts<'_>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let mut view = WorldView::new(parts, &self.token);
        self.system.react(&mut view, event)
    }
}
