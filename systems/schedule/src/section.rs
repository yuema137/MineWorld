//! The `routine:` section of a person's file, which this pack owns (`ARC-31`).

use mineworld_authoring::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
use mineworld_contracts::{EntityId, EntityType, PersonId, PlaceId, Rejection};
use mineworld_kernel::Emission;

use crate::event::RoutineAssigned;
use crate::segment::AuthoredRoutine;
use crate::system::ScheduleSystem;

impl AuthoredSection for ScheduleSystem {
    /// `routine: [{ from: "06:30", place: cafe, label: work }, …]`.
    const SECTION: SectionName = SectionName::from_static("routine");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];

    /// The segments, places by key: decoding them is every routine rule (`Segments::new`).
    type Authored = AuthoredRoutine;

    /// Every segment's place, which must be a place this pack declares.
    fn references(authored: &AuthoredRoutine) -> Vec<Reference<'_>> {
        authored
            .segments()
            .iter()
            .map(|segment| Reference {
                key: segment.place(),
                entity_type: EntityType::Place,
            })
            .collect()
    }

    /// One `routine-assigned` about the person whose file it is, places resolved.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &AuthoredRoutine,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        let person =
            PersonId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        let segments = authored.map_places(|key| {
            let place = seeding
                .resolve(key, EntityType::Place)
                .ok_or(Rejection::PreconditionFailed)?;
            PlaceId::new(place, EntityType::Place).map_err(|_| Rejection::PreconditionFailed)
        })?;
        Ok(vec![RoutineAssigned::new(person, segments).emission()])
    }
}
