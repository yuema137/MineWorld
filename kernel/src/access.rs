//! Ownership: who may write what, and why the compiler is the one that checks it.
//!
//! `INV-7` is the invariant MineWorld's composability rests on — *only systems mutate state, and
//! only the components they own*. A world is assembled from systems written by people who have
//! never met, so if that rule were a review convention it would be broken, once, in a pack
//! nobody reviewed, and every guarantee built on it would quietly stop holding.
//!
//! So it is not a convention here. It is three things the compiler enforces:
//!
//! ```text
//! SystemIdentity     a system is a type, and its declared name is part of that type
//! OwnedBy<S>         a component type states, in the type system, which system writes it
//! WriteToken<S>      writing needs S's token, and only the kernel can make one
//! ```
//!
//! Put together: a write takes `&WriteToken<S>` and requires `C: OwnedBy<S>`, so a system cannot
//! *name* a write to a component it does not own. There is no runtime ownership check on the
//! write path to forget, get wrong, or have to test.
//!
//! # Where tokens come from, and why that is the other half of the guarantee
//!
//! A token names a system; it cannot prove that its holder *is* that system, because Rust has no
//! notion of which crate is calling. So the question "who may obtain which token" is not answered
//! by the type system at all — it is answered by how a world is assembled, and the answer is
//! [`World`](crate::World):
//!
//! ```text
//! WriteAccess       a world's only issuer of write capability, and pub(crate) — BD-1
//! World::install    the one act that grants a token, once per system, at assembly
//! WorldView<S>      what a running system is handed: gated writes, never the issuer
//! ```
//!
//! PR 03a left [`WriteAccess::new`] public, and its review reproduced the consequence from an
//! external crate (attempt `A8`): construct your own issuer, grant yourself another system's token,
//! write its component. It succeeded. The constructor is therefore crate-private as of PR 03b, and
//! `WriteAccess` derives no `Default` either — a derived `Default` would be a public second
//! constructor and would reopen the hole exactly. The refusal is pinned by
//! `tests/compile_fail/an_external_crate_cannot_construct_write_access.rs`, so it cannot silently
//! close again.
//!
//! Two smaller mitigations remain, and they are worth stating because they are weaker than they
//! look:
//!
//! 1. [`WriteAccess::grant`] requires a **value** of the system type, and
//!    [`World::install`](crate::World::install) takes one for the same reason. A pack whose system
//!    type keeps its constructor cannot be installed by anybody else — but most packs export
//!    `pub struct InventorySystem;`, so this protects only the packs that ask for it.
//! 2. `grant` refuses a second token for one [`SystemId`], and the registry refuses a second
//!    installation under one name. An impersonator that got in first is therefore *detected*: the
//!    real system's installation is what fails.
//!
//! # What this still does not, and cannot, prevent
//!
//! [`OwnedBy`] is a public trait, so a pack can implement it for a component type **it defines**,
//! including dishonestly. Coherence stops it from implementing the trait for another pack's
//! component type, so a lie can only ever widen who may write the liar's own state, never narrow
//! anyone else's. [`ComponentStore::declare`](crate::ComponentStore::declare) refuses the lie
//! anyway, once, when the component type is declared.
//!
//! A component type that carries interior mutability — a `Cell`, a `RefCell`, a `Mutex` — can be
//! changed by anyone who can read it, and no bound on stable Rust expresses "no interior
//! mutability". **A component type must not carry interior mutability.** That is a review
//! convention rather than a mechanism, it is the one rule here a compiler does not enforce, and it
//! is also self-limiting: only the owner of the state can open that hole, by declaring it that way.
//!
//! What a token proves is therefore precise, and worth stating exactly: **given a token, the
//! component types writable through it are exactly the ones its system owns.** That part is the
//! type system's and has no runtime check to get wrong.

use std::collections::BTreeSet;
use std::marker::PhantomData;

use mineworld_contracts::{Component, SystemId};

use crate::error::KernelError;

/// A system, as a type.
///
/// One associated constant, and it is the system's declared name. Being a constant is what makes
/// the ownership machinery work: a component type can name its owner's `ID` in its own
/// declaration, so the two cannot drift apart. The literal is checked while the declaring crate
/// compiles — see [`SystemId::from_static`].
///
/// ```
/// use mineworld_contracts::SystemId;
/// use mineworld_kernel::SystemIdentity;
///
/// /// A system that owns the state of places.
/// struct Places;
///
/// impl SystemIdentity for Places {
///     const ID: SystemId = SystemId::from_static("places");
/// }
/// ```
///
/// This is not the `System` interface. A system's behaviour — what it ticks, which actions it
/// provides, which events it subscribes to — is a separate contract; this trait is only the
/// identity that ownership is expressed against.
pub trait SystemIdentity {
    /// The system's declared name.
    const ID: SystemId;
}

/// The single-writer relationship: `S` is the one system allowed to write `Self`.
///
/// Implemented by [`owned_component!`](crate::owned_component), which generates it alongside the
/// component's own declaration so that the component's [`Component::OWNER`] *is* `S::ID` rather
/// than a second statement that has to agree with it.
///
/// Reads are deliberately not gated (`KD-4`). Any system may read any component, because
/// reacting to another system's state is how independent systems compose at all, and gating
/// reads would mean inventing a query language before anything needs one. What a controller is
/// allowed to perceive is `Observation`'s problem, not the store's.
pub trait OwnedBy<S: SystemIdentity>: Component {}

/// Proof that the holder may write one system's components.
///
/// A token carries no data — it is the type parameter that means something. It cannot be
/// constructed outside this crate, cannot be cloned, and is only ever handed out by
/// [`WriteAccess::grant`]. Writes take it by reference, so a system passes its own token to the
/// store and nothing else can.
pub struct WriteToken<S: SystemIdentity> {
    /// The system the token belongs to. `fn() -> S` rather than `S`, so a token is `Send` and
    /// `Sync` whatever the system type is: the token holds no system, it names one.
    system: PhantomData<fn() -> S>,
}

impl<S: SystemIdentity> WriteToken<S> {
    /// Mints the token. Deliberately crate-private: [`WriteAccess`] is the only issuer, and
    /// forging one is what the whole mechanism exists to prevent.
    pub(crate) const fn new() -> Self {
        Self {
            system: PhantomData,
        }
    }

    /// The name of the system this token belongs to, for diagnostics and for the kernel's own
    /// checks.
    pub fn system(&self) -> SystemId {
        S::ID
    }
}

impl<S: SystemIdentity> core::fmt::Debug for WriteToken<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "WriteToken({})", S::ID)
    }
}

/// One world's issuer of write capabilities.
///
/// Exactly one token exists per system, and this is where it comes from. A second request for a
/// system that already holds one is refused rather than served, so two pieces of code cannot
/// both believe they are the writer of a system's state.
///
/// **There is one of these per world and it cannot be constructed outside this crate** (`BD-1`). A
/// [`World`](crate::World) holds it in a private field, never lends it out, and grants from it only
/// while installing a system. No `Default` is derived, because a derived `Default` is a public
/// second constructor: it would let an external crate build a second issuer and grant itself any
/// system's token, which is precisely the bypass this seal closes.
#[derive(Debug)]
pub struct WriteAccess {
    granted: BTreeSet<SystemId>,
}

impl WriteAccess {
    /// A world in which nothing has write access yet.
    ///
    /// Crate-private: one issuer per world, created by [`World::new`](crate::World::new).
    pub(crate) fn new() -> Self {
        Self {
            granted: BTreeSet::new(),
        }
    }

    /// Issues `S`'s write token, once.
    ///
    /// The `_system` parameter is a possession check rather than data: the caller has to hold a
    /// value of the system type, which is the only thing in Rust that a system's own crate can
    /// keep to itself. Nothing is read from it.
    pub(crate) fn grant<S: SystemIdentity>(
        &mut self,
        _system: &S,
    ) -> Result<WriteToken<S>, KernelError> {
        if !self.granted.insert(S::ID) {
            return Err(KernelError::WriteAccessAlreadyGranted { system: S::ID });
        }
        Ok(WriteToken::new())
    }

    /// Every system that holds a write token, in name order. Reported by
    /// [`World::writers`](crate::World::writers).
    pub(crate) fn granted(&self) -> impl Iterator<Item = &SystemId> {
        self.granted.iter()
    }
}

/// Declares a component type and the system that owns it, in one place.
///
/// This is the intended way to declare a component, because it is the way the two halves of
/// ownership cannot come apart. The macro generates the [`Component`] implementation with
/// `OWNER` taken from the owning system's [`SystemIdentity::ID`], and the
/// [`OwnedBy`] implementation that lets that system — and only that system — write it.
///
/// ```
/// use mineworld_contracts::SystemId;
/// use mineworld_kernel::{
///     Declarations, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
///     owned_component,
/// };
/// use serde::{Deserialize, Serialize};
///
/// struct Places;
/// impl SystemIdentity for Places {
///     const ID: SystemId = SystemId::from_static("places");
/// }
///
/// #[derive(Debug, Serialize, Deserialize)]
/// struct Occupancy {
///     present: u32,
/// }
///
/// owned_component! {
///     component = Occupancy,
///     owner = Places,
///     component_type = "occupancy",
///     schema_version = 1,
/// }
///
/// impl System for Places {
///     const VERSION: SystemVersion = SystemVersion::new(1);
///
///     fn declaration(&self) -> SystemDeclaration {
///         SystemDeclaration::of::<Self>().owning::<Occupancy>()
///     }
///
///     fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
///         tables.component::<Occupancy>()
///     }
/// }
///
/// # fn main() -> Result<(), KernelError> {
/// let mut world = World::new();
/// world.install(Places)?;
///
/// assert!(world.components().is_declared(&Occupancy::COMPONENT_TYPE));
/// # Ok(())
/// # }
/// # use mineworld_contracts::Component;
/// ```
///
/// The table is declared by the system that owns the type, as it is installed, and the write token
/// that authorizes it never leaves the world (`BD-1`). There is no other way in: outside this crate
/// a `WriteAccess` cannot be constructed, so a component type is only ever declared by its owner
/// joining a world.
///
/// A component declared by hand is still a component, and the store still works with it; what a
/// hand-written declaration can do, and this cannot, is state an owner that disagrees with the
/// [`OwnedBy`] implementation. That disagreement is refused when the type is declared — see
/// [`ComponentStore::declare`](crate::ComponentStore::declare).
#[macro_export]
macro_rules! owned_component {
    (
        component = $component:ty,
        owner = $owner:ty,
        component_type = $component_type:literal,
        schema_version = $schema_version:literal $(,)?
    ) => {
        impl $crate::macro_support::Component for $component {
            const COMPONENT_TYPE: $crate::macro_support::ComponentTypeId =
                $crate::macro_support::ComponentTypeId::from_static($component_type);
            const OWNER: $crate::macro_support::SystemId = <$owner as $crate::SystemIdentity>::ID;
            const SCHEMA_VERSION: $crate::macro_support::ComponentSchemaVersion =
                $crate::macro_support::ComponentSchemaVersion::new($schema_version);
        }

        impl $crate::OwnedBy<$owner> for $component {}
    };
}
