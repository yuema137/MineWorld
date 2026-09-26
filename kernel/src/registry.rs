//! The system registry: which systems a world is composed of, and in what order.
//!
//! A world is a set of installed systems, and this is the set. It holds three things, and the
//! reason there are three rather than one is `AC-12`:
//!
//! ```text
//! order      Vec<SystemId>            registration order — the order reduction runs in
//! entries    BTreeMap<SystemId, _>    lookup by name
//! ```
//!
//! Order is an explicit `Vec` and never the iteration order of the map (`BD-4`). That is not a
//! micro-optimization: reduction order is observable in the event log, so a world whose reduction
//! order came from a container's iteration would have its history decided by a detail nobody
//! declared — and a rename of a system would silently reorder its past.
//!
//! # What the registry refuses
//!
//! Composition errors are decidable from [`SystemDeclaration`]s alone, before any world runs, and
//! refusing them here is what keeps `INV-7` and `INV-10` true at the level of a *world* rather than
//! of a single store: one system per component type, one system per action type, and every declared
//! dependency present.

use std::collections::BTreeMap;

use mineworld_contracts::SystemId;

use crate::error::KernelError;
use crate::system::{DynSystem, SystemDeclaration};

/// One installed system: what it declared, and the erased handle the world calls it through.
struct Entry {
    declaration: SystemDeclaration,
    system: Box<dyn DynSystem>,
}

/// The systems a world is composed of.
///
/// Held by a [`World`](crate::World), which is the only thing that installs into it: a system is
/// granted its write token as it is registered here, and the token lives in the entry
/// (`BD-1`). There is no way to take a system back out, because uninstalling is not removing a
/// Rust value — the state it owns would be left ownerless. Removing a system's *effect* on a world
/// is what enable and disable are for.
pub struct SystemRegistry {
    order: Vec<SystemId>,
    entries: BTreeMap<SystemId, Entry>,
}

impl SystemRegistry {
    pub(crate) fn new() -> Self {
        Self {
            order: Vec::new(),
            entries: BTreeMap::new(),
        }
    }

    /// Whether this world has that system installed.
    pub fn is_installed(&self, system: &SystemId) -> bool {
        self.entries.contains_key(system)
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
    /// it was.
    pub(crate) fn check_installable(
        &self,
        declaration: &SystemDeclaration,
    ) -> Result<(), KernelError> {
        if self.is_installed(declaration.system()) {
            return Err(KernelError::SystemAlreadyInstalled {
                system: declaration.system().clone(),
            });
        }
        Ok(())
    }

    /// Records an installed system. Registration order is appended to, never sorted.
    pub(crate) fn register(&mut self, declaration: SystemDeclaration, system: Box<dyn DynSystem>) {
        let id = declaration.system().clone();
        self.order.push(id.clone());
        self.entries.insert(
            id,
            Entry {
                declaration,
                system,
            },
        );
    }

    /// The erased handle for one installed system.
    pub(crate) fn system(&self, system: &SystemId) -> Option<&dyn DynSystem> {
        self.entries.get(system).map(|entry| entry.system.as_ref())
    }
}

impl core::fmt::Debug for SystemRegistry {
    /// A registry cannot print its systems — a system is not required to be `Debug`, and what a
    /// reader of a debug dump wants is the composition anyway: which systems, in which order.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("SystemRegistry")
            .field("order", &self.order)
            .finish()
    }
}
