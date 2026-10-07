//! The `name:` section of a person's file, which this pack owns (`ARC-31`).

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, PersonId, Rejection};
use mineworld_kernel::Emission;

use crate::event::Named;
use crate::name::Name;
use crate::system::NamingSystem;

impl AuthoredSection for NamingSystem {
    /// `name: Alice Moreau`.
    const SECTION: SectionName = SectionName::from_static("name");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];

    /// The name itself: decoding it is [`Name`]'s check.
    type Authored = Name;

    /// One `named` fact about the person whose file it is.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &Name,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        // A name in a place's file would be refused by the loader (`CARRIED_BY`); refused here too,
        // because the type of the subject is this pack's to check, not the loader's to promise.
        let person =
            PersonId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![Named::new(person, authored.clone()).emission()])
    }
}
