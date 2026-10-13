//! The state this pack owns: which places open onto which, where the doorway is, and who is walking
//! where.

use mineworld_contracts::{EntityId, LocalPosition, Location, PlaceId};
use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::action::Destination;
use crate::system::MovementSystem;

/// A walk in progress: the walker's intention and the route the world planned for it
/// (`DECISIONS.md` `ARC-75`; step-11 SD-N2).
///
/// Owned by [`MovementSystem`], on the walker. Written when `walk-to` is resolved, at each `walk-step`
/// and in this pack's reactions to the walker's own arrivals; removed when the walk ends. It is not a
/// position — where the walker is stays presence's — and not a `Process`: it takes no calendar time,
/// and nothing advances it but its walker's requests. Persisted and replayed like every component, so
/// a resumed world continues the walk where it was.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Walking {
    pub(crate) destination: Destination,
    /// The places still to enter, in order; empty in the destination's own place.
    pub(crate) legs: Vec<PlaceId>,
    /// The current leg's waypoints not yet reached, in order.
    pub(crate) waypoints: Vec<LocalPosition>,
    /// Where the current leg was planned to end; [`None`] until it is planned — after a crossing, or
    /// when it must be planned again.
    pub(crate) goal: Option<LocalPosition>,
    /// The last stride asked for: where from and where to. [`None`] before the first.
    pub(crate) progress: Option<Asked>,
    /// Consecutive steps that made less than `STALL_PROGRESS`.
    pub(crate) stalls: u32,
    /// Re-plans made so far.
    pub(crate) replans: u32,
    /// Who stopped the last stride short, when presence's `stopped-short` named somebody.
    pub(crate) stopped_by: Option<EntityId>,
}

/// One stride as it was asked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Asked {
    pub(crate) from: Location,
    pub(crate) to: Location,
}

owned_component! {
    component = Walking,
    owner = MovementSystem,
    component_type = "walking",
    schema_version = 1,
}

impl Walking {
    /// Where the walk goes.
    pub const fn destination(&self) -> Destination {
        self.destination
    }

    /// The places still to enter, in order.
    pub fn legs(&self) -> &[PlaceId] {
        &self.legs
    }

    /// The current leg's waypoints not yet reached, in order.
    pub fn waypoints(&self) -> &[LocalPosition] {
        &self.waypoints
    }
}

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
