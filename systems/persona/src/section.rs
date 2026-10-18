//! The `persona:` section of a person's file, which this pack owns (`ARC-31`, `ARC-74`).

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, PersonId, Rejection};
use mineworld_kernel::Emission;

use crate::event::PersonaAuthored;
use crate::persona::Persona;
use crate::system::PersonaSystem;

impl AuthoredSection for PersonaSystem {
    /// `persona: { summary: …, traits: […], speech: … }`.
    const SECTION: SectionName = SectionName::from_static("persona");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];

    /// The character itself: decoding it is [`Persona`]'s check, with every bound.
    type Authored = Persona;

    /// One `persona-authored` fact about the person whose file it is.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &Persona,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        // A persona in a place's file would be refused by the loader (`CARRIED_BY`); refused here too,
        // because the type of the subject is this pack's to check, not the loader's to promise.
        let person =
            PersonId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![
            PersonaAuthored::new(person, authored.clone()).emission(),
        ])
    }
}
