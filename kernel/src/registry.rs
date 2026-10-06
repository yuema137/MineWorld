//! The system registry: which systems a world is composed of, in what order, and which of them
//! answer for what.
//!
//! A world is a set of installed systems, and this is the set. It holds four things, and the reason
//! there are four rather than one is `AC-12` and `INV-10`:
//!
//! ```text
//! order      Vec<SystemId>                registration order — the order reduction runs in
//! entries    BTreeMap<SystemId, _>        lookup by name, and whether each is enabled
//! routes     BTreeMap<ActionTypeId, _>    which enabled system answers for which action
//! ```
//!
//! Order is an explicit `Vec` and never the iteration order of the map (`BD-4`). That is not a
//! micro-optimization: reduction order is observable in the event log, so a world whose reduction
//! order came from a container's iteration would have its history decided by a detail nobody
//! declared — and a rename of a system would silently reorder its past. Every ordered answer this
//! module gives — [`declarations`](SystemRegistry::declarations),
//! [`enabled`](SystemRegistry::enabled), [`subscribers`](SystemRegistry::subscribers) — walks
//! `order` and looks each entry up, so there is one statement of order in the crate and nothing
//! else can disagree with it.
//!
//! # The route map, and why it is a map rather than a search
//!
//! An action is routed by [`ActionTypeId`] through [`routes`](SystemRegistry::provider), built at
//! installation and maintained by [enabling and disabling](SystemRegistry::disable) (`BD-5`). An
//! action type that is not in it is not provided by any *enabled* system, and dispatch answers
//! [`ActionResult::Unavailable`](mineworld_contracts::ActionResult::Unavailable) without consulting
//! a single system (`INV-10`). Searching the declarations instead would give the same answer today
//! and would make the answer depend on how many systems happen to be installed; more importantly,
//! it would put the decision in dispatch rather than in composition, where it belongs.
//!
//! # What the registry refuses
//!
//! Composition errors are decidable from [`SystemDeclaration`]s alone, before any world runs, and
//! refusing them here is what keeps `INV-7` and `INV-10` true at the level of a *world* rather than
//! of a single store:
//!
//! ```text
//! one system per component type       two writers for one piece of state — INV-7
//! one system per action type          two answers for one request — INV-10 has to be decidable
//! every declared dependency present   a system whose dependency is absent is not composable
//! ```
//!
//! All three are checked before anything is granted or declared, so a refused installation leaves
//! the world exactly as it was.
//!
//! # Enabled is a property of a world's configuration, not of a system
//!
//! A system is installed once and stays installed: uninstalling is not removing a Rust value,
//! because the state it owns would be left ownerless. What a world's configuration changes is
//! whether an installed system *acts* — and disabling one removes its actions from dispatch and its
//! reductions from the pipeline, with no edit to any other module (`AC-2`).

use std::collections::BTreeMap;

use mineworld_contracts::{ActionTypeId, EventTypeId, SystemId};

use crate::error::KernelError;
use crate::system::{DynSystem, SystemDeclaration};

/// One installed system: what it declared, whether it is enabled, and the erased handle the world
/// calls it through.
struct Entry {
    declaration: SystemDeclaration,
    system: Box<dyn DynSystem>,
    enabled: bool,
}

/// The systems a world is composed of.
///
/// Held by a [`World`](crate::World), which is the only thing that installs into it: a system is
/// granted its write token as it is registered here, and the token lives in the entry
/// (`BD-1`). There is no way to take a system back out, because uninstalling is not removing a
/// Rust value — the state it owns would be left ownerless. Removing a system's *effect* on a world
/// is what [`World::enable`](crate::World::enable) and [`World::disable`](crate::World::disable)
/// are for.
pub struct SystemRegistry {
    order: Vec<SystemId>,
    entries: BTreeMap<SystemId, Entry>,
    routes: BTreeMap<ActionTypeId, SystemId>,
}

impl SystemRegistry {
    pub(crate) fn new() -> Self {
        Self {
            order: Vec::new(),
            entries: BTreeMap::new(),
            routes: BTreeMap::new(),
        }
    }

    /// Whether this world has that system installed, enabled or not.
    pub fn is_installed(&self, system: &SystemId) -> bool {
        self.entries.contains_key(system)
    }

    /// Whether that system is installed **and** acting: routed to, and reducing what it subscribes
    /// to.
    pub fn is_enabled(&self, system: &SystemId) -> bool {
        self.entries.get(system).is_some_and(|entry| entry.enabled)
    }

    /// What a system declared when it was installed.
    pub fn declaration(&self, system: &SystemId) -> Option<&SystemDeclaration> {
        self.entries.get(system).map(|entry| &entry.declaration)
    }

    /// Every installed system's declaration, in **registration order** — what this world is made
    /// of, in the order it was composed.
    pub fn declarations(&self) -> impl Iterator<Item = &SystemDeclaration> {
        self.order
            .iter()
            .filter_map(|system| self.declaration(system))
    }

    /// The installed systems, in registration order.
    pub fn order(&self) -> &[SystemId] {
        &self.order
    }

    /// The enabled systems, in registration order: the ones a dispatch actually reaches.
    pub fn enabled(&self) -> impl Iterator<Item = &SystemId> {
        self.order.iter().filter(|system| self.is_enabled(system))
    }

    /// The enabled system that answers for that action, if this world has one.
    ///
    /// The whole of routing (`BD-5`): [`None`] is the `INV-10` answer, and no system is consulted
    /// to produce it.
    pub fn provider(&self, action_type: &ActionTypeId) -> Option<&SystemId> {
        self.routes.get(action_type)
    }

    /// Every action this world answers for, and the system that answers, in action-type order.
    pub fn routes(&self) -> impl Iterator<Item = (&ActionTypeId, &SystemId)> {
        self.routes.iter()
    }

    /// The enabled systems that reduce that kind of fact, in registration order.
    ///
    /// Registration order is the order reduction runs in (`BD-4`), and it is stated here by walking
    /// [`order`](SystemRegistry::order) rather than by iterating the entry map.
    pub fn subscribers<'a>(
        &'a self,
        event_type: &'a EventTypeId,
    ) -> impl Iterator<Item = &'a SystemId> {
        self.order.iter().filter(move |system| {
            self.entries
                .get(*system)
                .is_some_and(|entry| entry.enabled && entry.declaration.subscribes_to(event_type))
        })
    }

    /// How many systems this world has installed.
    pub fn len(&self) -> usize {
        self.order.len()
    }

    /// Whether this world has no systems at all — a legal world, and the one every world starts as.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }

    /// Whether a declaration can join this world, decided from declarations alone.
    ///
    /// Called before anything is granted or declared, so that a refusal leaves the world exactly as
    /// it was. Five refusals, in the order a reader of the failure would want them: the name is
    /// free, every dependency is present and enabled, every vocabulary it borrows belongs to one of
    /// those dependencies, no component type is claimed twice, and no action type is answered twice.
    pub(crate) fn check_installable(
        &self,
        declaration: &SystemDeclaration,
    ) -> Result<(), KernelError> {
        if self.is_installed(declaration.system()) {
            return Err(KernelError::SystemAlreadyInstalled {
                system: declaration.system().clone(),
            });
        }
        self.check_dependencies(declaration)?;
        Self::check_emitted_vocabularies(declaration)?;
        self.check_owned_components(declaration)?;
        self.check_provided_actions(declaration)
    }

    /// Every kind of fact this declaration emits in another system's vocabulary names an owner it
    /// declares a dependency on (`ARC-26`).
    ///
    /// Decided from the declaration alone, and before the dependency check's own question — whether
    /// those dependencies are installed and enabled — matters: a system that speaks presence's
    /// vocabulary without depending on presence is a wrong declaration in any world. Together the two
    /// checks guarantee that whenever such a fact can be stated, its owner is present to reduce it.
    /// `enable` needs no repeat of this check: a declaration cannot change after installation, and
    /// `enable` re-checks the dependencies themselves.
    fn check_emitted_vocabularies(declaration: &SystemDeclaration) -> Result<(), KernelError> {
        for (event_type, owner) in declaration.emits_owned_by_others() {
            if !declaration.depends_on().contains(owner) {
                return Err(KernelError::EmittedEventOwnerNotADependency {
                    system: declaration.system().clone(),
                    event_type: event_type.clone(),
                    owner: owner.clone(),
                });
            }
        }
        Ok(())
    }

    /// Every system this declaration depends on is installed and enabled.
    ///
    /// A dependency that is merely installed is not enough: a system whose dependency is disabled
    /// would be running against state nobody is maintaining, which is the composition mistake this
    /// check exists to name rather than to let a world discover at run time.
    fn check_dependencies(&self, declaration: &SystemDeclaration) -> Result<(), KernelError> {
        for dependency in declaration.depends_on() {
            if !self.is_installed(dependency) {
                return Err(KernelError::SystemDependencyMissing {
                    system: declaration.system().clone(),
                    dependency: dependency.clone(),
                });
            }
            if !self.is_enabled(dependency) {
                return Err(KernelError::SystemDependencyDisabled {
                    system: declaration.system().clone(),
                    dependency: dependency.clone(),
                });
            }
        }
        Ok(())
    }

    /// No component type this declaration claims is claimed by another system.
    ///
    /// Two checks, and they answer different questions. A declaration that lists a component type
    /// **owned by somebody else** is refused outright: ownership is a fact about the component
    /// type, and a system cannot claim another's state by listing it. Then
    /// [`ComponentDeclaration::conflicts_with`](mineworld_contracts::ComponentDeclaration::conflicts_with)
    /// decides the remaining case — two systems, each honestly claiming a component type of the
    /// same name — which is the conflict `INV-7` forbids and the reason two packs cannot both
    /// define `"inventory"`.
    fn check_owned_components(&self, declaration: &SystemDeclaration) -> Result<(), KernelError> {
        for owned in declaration.owns() {
            if owned.owner() != declaration.system() {
                return Err(KernelError::ComponentOwnerDisagreesWithDeclaration {
                    component_type: owned.component_type().clone(),
                    declared_owner: owned.owner().clone(),
                    writing_system: declaration.system().clone(),
                });
            }
            for installed in self.declarations() {
                if let Some(existing) = installed
                    .owns()
                    .iter()
                    .find(|existing| existing.conflicts_with(owned))
                {
                    return Err(KernelError::ComponentTypeClaimedByAnotherSystem {
                        component_type: owned.component_type().clone(),
                        declared_by: existing.owner().clone(),
                        claimed_by: owned.owner().clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// No action type this declaration provides is provided by another installed system.
    ///
    /// Checked against declarations rather than against the route map, because a *disabled* system
    /// still provides its actions: admitting a second provider while the first is disabled would
    /// leave a world that cannot be enabled, and `INV-10`'s answer — this action does not exist
    /// here — has to be decidable without asking which of two systems meant it.
    fn check_provided_actions(&self, declaration: &SystemDeclaration) -> Result<(), KernelError> {
        for action_type in declaration.provides() {
            for installed in self.declarations() {
                if installed.provides_action(action_type) {
                    return Err(KernelError::ActionTypeProvidedByAnotherSystem {
                        action_type: action_type.clone(),
                        provided_by: installed.system().clone(),
                        claimed_by: declaration.system().clone(),
                    });
                }
            }
        }
        Ok(())
    }

    /// Records an installed system, enabled. Registration order is appended to, never sorted.
    pub(crate) fn register(&mut self, declaration: SystemDeclaration, system: Box<dyn DynSystem>) {
        let id = declaration.system().clone();
        for action_type in declaration.provides() {
            self.routes.insert(action_type.clone(), id.clone());
        }
        self.order.push(id.clone());
        self.entries.insert(
            id,
            Entry {
                declaration,
                system,
                enabled: true,
            },
        );
    }

    /// The system an action is routed to, together with the handle to call it through.
    ///
    /// One lookup rather than two, so that the pipeline has no unreachable branch to write: an
    /// action type is either routed to an installed system or it is not provided here at all
    /// (`INV-10`).
    pub(crate) fn routed(&self, action_type: &ActionTypeId) -> Option<(&SystemId, &dyn DynSystem)> {
        let (system, entry) = self
            .routes
            .get(action_type)
            .and_then(|system| self.entries.get_key_value(system))?;
        Some((system, entry.system.as_ref()))
    }

    /// The erased handle for one installed system, enabled or not.
    ///
    /// Whether a system is reached at all is decided before this is called — by the route map for
    /// an action, and by [`subscribers`](SystemRegistry::subscribers) for a fact — so this is a
    /// lookup rather than a policy.
    pub(crate) fn system(&self, system: &SystemId) -> Option<&dyn DynSystem> {
        self.entries.get(system).map(|entry| entry.system.as_ref())
    }

    /// Puts an installed system back in the pipeline, restoring its routes.
    ///
    /// Refused if its dependencies are not there to depend on, for the same reason installation is.
    /// Enabling an already-enabled system changes nothing and is not an error: this is a statement
    /// about the configuration a world should be in, not an operation with a history.
    pub(crate) fn enable(&mut self, system: &SystemId) -> Result<(), KernelError> {
        let declaration = self.require(system)?.declaration.clone();
        self.check_dependencies(&declaration)?;

        for action_type in declaration.provides() {
            self.routes.insert(action_type.clone(), system.clone());
        }
        if let Some(entry) = self.entries.get_mut(system) {
            entry.enabled = true;
        }
        Ok(())
    }

    /// Takes an installed system out of the pipeline: its actions stop being routed and its
    /// reductions stop running.
    ///
    /// This is `AC-2` in one method. The system stays installed and its state stays in the world,
    /// because the state is still its state; what changes is that nothing reaches it, so an action
    /// it provided is answered
    /// [`Unavailable`](mineworld_contracts::ActionResult::Unavailable) exactly as it would be in a
    /// world where the system had never been installed.
    ///
    /// Refused while an enabled system depends on it, naming the dependent: a world in which an
    /// enabled system's declared dependency is disabled is a world the registry would have refused
    /// to assemble, and reaching it by disabling would be the same mistake arrived at backwards.
    pub(crate) fn disable(&mut self, system: &SystemId) -> Result<(), KernelError> {
        self.require(system)?;
        if let Some(dependent) = self.enabled_dependent_on(system) {
            return Err(KernelError::SystemRequiredByAnotherSystem {
                system: system.clone(),
                required_by: dependent.clone(),
            });
        }

        self.routes.retain(|_, provider| provider != system);
        if let Some(entry) = self.entries.get_mut(system) {
            entry.enabled = false;
        }
        Ok(())
    }

    /// The first enabled system, in registration order, that declares a dependency on `system`.
    fn enabled_dependent_on(&self, system: &SystemId) -> Option<&SystemId> {
        self.order.iter().find(|candidate| {
            self.entries.get(*candidate).is_some_and(|entry| {
                entry.enabled
                    && entry.declaration.system() != system
                    && entry.declaration.depends_on().contains(system)
            })
        })
    }

    /// The entry, or the refusal naming a system this world never installed.
    fn require(&self, system: &SystemId) -> Result<&Entry, KernelError> {
        self.entries
            .get(system)
            .ok_or_else(|| KernelError::SystemNotInstalled {
                system: system.clone(),
            })
    }
}

impl core::fmt::Debug for SystemRegistry {
    /// A registry cannot print its systems — a system is not required to be `Debug`, and what a
    /// reader of a debug dump wants is the composition anyway: which systems, in which order, and
    /// which of them are acting.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let enabled: Vec<&SystemId> = self.enabled().collect();
        f.debug_struct("SystemRegistry")
            .field("order", &self.order)
            .field("enabled", &enabled)
            .field("routes", &self.routes)
            .finish()
    }
}
