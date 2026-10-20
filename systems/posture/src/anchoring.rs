//! Posture holds a seated person where they sat (S34-b; `ARC-147` note of 2026-10-19; Q-S34-5): bodies
//! asks this pack, through its `Anchoring` catalog, who may not be nudged or shoved, and which points of a
//! place a person must be able to occupy.

use mineworld_bodies::Anchoring;
use mineworld_contracts::{LocalPosition, PersonId, PlaceId, SystemId};
use mineworld_kernel::{SystemIdentity, WorldRead};

use crate::component::Posture;
use crate::point::SitPoints;
use crate::system::PostureSystem;

impl Anchoring for PostureSystem {
    fn anchoring_of(&self) -> SystemId {
        Self::ID
    }

    /// A seated person is anchored: they have a `Posture` row.
    fn anchored(&self, world: &WorldRead<'_>, person: PersonId) -> bool {
        world.component::<Posture>(person.entity_id()).is_some()
    }

    /// The place's sit points, in key order, each named by its key.
    fn fixed_points(&self, world: &WorldRead<'_>, place: PlaceId) -> Vec<(String, LocalPosition)> {
        world
            .component::<SitPoints>(place.entity_id())
            .map(fixed_points)
            .unwrap_or_default()
    }
}

/// `points` as bodies' fixed points: each key and its position.
pub(crate) fn fixed_points(points: &SitPoints) -> Vec<(String, LocalPosition)> {
    points
        .iter()
        .map(|point| (point.key().as_str().to_owned(), point.at()))
        .collect()
}
