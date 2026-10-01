//! The state this pack owns: which places open onto which, and where the doorway is.

use mineworld_contracts::{LocalPosition, PlaceId};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::system::MovementSystem;

/// One way out of a place: the place it leads to, and where the doorway is on each side.
///
/// Both positions are optional for the reason a [`Location`](mineworld_contracts::Location)'s
/// refinement is: a semantic world says only *the café opens onto the street*, and a world that
/// models position says where the door is. When a side is known, a person must be within a stride
/// of it to pass — which is what makes entering a place a walk through a door rather than a jump.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Passage {
    to: PlaceId,
    here: Option<LocalPosition>,
    there: Option<LocalPosition>,
}

impl Passage {
    /// A way from this place to `to`, with the doorway at `here` in this place's frame and at
    /// `there` in `to`'s.
    pub const fn new(
        to: PlaceId,
        here: Option<LocalPosition>,
        there: Option<LocalPosition>,
    ) -> Self {
        Self { to, here, there }
    }

    /// The place it leads to.
    pub const fn to(&self) -> PlaceId {
        self.to
    }

    /// The doorway, in this place's frame, if the world models it.
    pub const fn here(&self) -> Option<LocalPosition> {
        self.here
    }

    /// The same doorway, in the other place's frame, if the world models it.
    pub const fn there(&self) -> Option<LocalPosition> {
        self.there
    }
}

/// Every way out of one place, ordered by the place each leads to.
///
/// Owned by [`MovementSystem`] and written only while it reduces a
/// [`PassageOpened`](crate::PassageOpened), so the topology is a projection of the log like every
/// other state. It is a fact about where one can *walk*; a later system that opens and closes doors
/// reads it or replaces it with its own.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Passages {
    leads_to: Vec<Passage>,
}

owned_component! {
    component = Passages,
    owner = MovementSystem,
    component_type = "passages",
    schema_version = 1,
}

impl Passages {
    /// The way to `place`, if this place opens onto it.
    pub fn to(&self, place: PlaceId) -> Option<&Passage> {
        self.leads_to.iter().find(|passage| passage.to == place)
    }

    /// Every way out, in [`PlaceId`] order.
    pub fn iter(&self) -> impl Iterator<Item = &Passage> {
        self.leads_to.iter()
    }

    /// Records `passage`, replacing any earlier one to the same place and keeping the order.
    pub(crate) fn open(&mut self, passage: Passage) {
        match self
            .leads_to
            .binary_search_by_key(&passage.to, |existing| existing.to)
        {
            Ok(index) => self.leads_to[index] = passage,
            Err(index) => self.leads_to.insert(index, passage),
        }
    }
}
