//! The `body:` section of a place file, which this pack owns (`ARC-31`; step-11 SD-B3).

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, PlaceId, Rejection};
use mineworld_kernel::Emission;

use crate::component::PlaceShape;
use crate::event::place_shaped;
use crate::system::BodiesSystem;

impl AuthoredSection for BodiesSystem {
    /// ```yaml
    /// body:
    ///   floor: { min: { x: 0, y: 0 }, max: { x: 8320, y: 10320 } }
    ///   solids:
    ///     - { min: { x: 3860, y: 6570 }, max: { x: 8320, y: 7170 }, height: 1100 }
    /// ```
    const SECTION: SectionName = SectionName::from_static("body");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Place];

    /// The shape itself: decoding it is [`PlaceShape`]'s check.
    type Authored = PlaceShape;

    /// One `place-shaped` fact about the place whose file it is.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &PlaceShape,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        // A body in a person's file is refused by the loader (`CARRIED_BY`); refused here too, because
        // the type of the subject is this pack's to check, not the loader's to promise.
        let place =
            PlaceId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![place_shaped(place, authored.clone())])
    }
}
