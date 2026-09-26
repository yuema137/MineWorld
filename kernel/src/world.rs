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

use mineworld_contracts::{
    Entity, EntityId, EntityKey, EntityType, LifecycleState, Metadata, Relation, SystemId, Tags,
};

use crate::access::WriteAccess;
use crate::components::ComponentStore;
use crate::entities::EntityRegistry;
use crate::error::KernelError;
use crate::registry::SystemRegistry;
use crate::relations::RelationStore;
use crate::system::{DynSystem, InstalledSystem, System};
use crate::view::WorldRead;

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
    /// This world's only issuer of write capability. Never lent out, never returned, and not
    /// constructible outside this crate (`BD-1`).
    access: WriteAccess,
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
            access: WriteAccess::new(),
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
            .finish()
    }
}
