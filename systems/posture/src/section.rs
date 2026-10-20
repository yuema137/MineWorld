//! The `sit-points:` section of a place's file, which this pack owns (`ARC-31`, `ARC-147`).
//!
//! ```yaml
//! sit-points:
//!   - { key: window-bench.1, group: window-bench, at: { x: 2700, y: 1160 }, facing: 180000 }
//!   - { key: window-bench.2, group: window-bench, at: { x: 3300, y: 1160 }, facing: 180000 }
//! ```

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, PlaceId, Rejection};
use mineworld_kernel::Emission;

use crate::event::PlaceSitPoints;
use crate::point::SitPoints;
use crate::system::PostureSystem;

impl AuthoredSection for PostureSystem {
    /// `sit-points:` — a list of sit points, carried by a place's file.
    const SECTION: SectionName = SectionName::from_static("sit-points");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Place];

    /// The list, as written: decoding it is [`SitPoints`]' check, so an invalid list cannot be read.
    type Authored = SitPoints;

    /// One `place-sit-points` fact about the place whose file it is.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &SitPoints,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        // The loader refuses this section outside a place's file (`CARRIED_BY`); refused here too,
        // because the type of the subject is this pack's to check, not the loader's to promise.
        let place =
            PlaceId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![
            PlaceSitPoints::new(place, authored.clone()).emission(),
        ])
    }
}
