//! What a pack's perception provider may be built with (`docs/DECISIONS.md` `ARC-121` note, `ARC-129`):
//! a read-only view of the composition's owner contexts, so a provider can keep presentation data —
//! label packs — that is never world state.
//!
//! The data is immutable, decoded once when the world is read, the same on creation and on resume. A
//! provider that keeps it holds no state and states no fact from it.

use std::collections::BTreeMap;

use mineworld_contracts::SystemId;

use crate::data::OwnerContext;

/// The composition's owner contexts, by owner, borrowed for the time providers are built.
#[derive(Debug, Default)]
pub struct ProviderContexts<'a> {
    contexts: BTreeMap<SystemId, &'a dyn OwnerContext>,
}

impl<'a> ProviderContexts<'a> {
    /// No context: what a provider is built with outside a composed world.
    pub fn none() -> Self {
        Self::default()
    }

    /// The contexts, each under its owner's id.
    pub fn new(contexts: impl IntoIterator<Item = &'a dyn OwnerContext>) -> Self {
        Self {
            contexts: contexts
                .into_iter()
                .map(|context| (context.owner(), context))
                .collect(),
        }
    }

    /// The context of `owner`, when the world enables it and it owns data.
    pub fn of(&self, owner: &SystemId) -> Option<&'a dyn OwnerContext> {
        self.contexts.get(owner).copied()
    }
}
