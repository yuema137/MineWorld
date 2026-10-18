//! The `appearance:` section of a person's file, which this pack owns (`ARC-31`, `ARC-91`).

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, PersonId, Rejection};
use mineworld_kernel::Emission;
use serde::Deserialize;

use crate::event::Appeared;
use crate::look::LookKey;
use crate::system::AppearanceSystem;

/// `appearance: { look: alice-moreau }`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    look: LookKey,
}

impl Appearance {
    /// This appearance.
    pub const fn new(look: LookKey) -> Self {
        Self { look }
    }

    /// The look.
    pub const fn look(&self) -> &LookKey {
        &self.look
    }
}

impl AuthoredSection for AppearanceSystem {
    /// `appearance: { look: alice-moreau }`.
    const SECTION: SectionName = SectionName::from_static("appearance");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];

    /// The look and its key: decoding them is [`Appearance`]'s and [`LookKey`]'s check.
    type Authored = Appearance;

    /// One `appeared` fact about the person whose file it is.
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &Appearance,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        // An appearance in a place's file would be refused by the loader (`CARRIED_BY`); refused here
        // too, because the type of the subject is this pack's to check, not the loader's to promise.
        let person =
            PersonId::new(subject, entity_type).map_err(|_| Rejection::PreconditionFailed)?;
        Ok(vec![
            Appeared::new(person, authored.look().clone()).emission(),
        ])
    }
}
