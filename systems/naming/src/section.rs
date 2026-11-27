//! The `name:` section of a person's or a place's file, which this pack owns (`ARC-31`, `ARC-363`).

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, EntityType, PersonId, PlaceId, Rejection};
use mineworld_kernel::Emission;

use crate::event::{Named, PlaceNamed};
use crate::name::NameSpec;
use crate::system::NamingSystem;

impl AuthoredSection for NamingSystem {
    /// `name: Alice Moreau`, or `name: { default: The Copper Kettle, forms: { zh_Hans: 铜壶咖啡馆 } }`.
    const SECTION: SectionName = SectionName::from_static("name");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person, ContentKind::Place];

    /// The name and its forms: decoding it is [`NameSpec`]'s check.
    type Authored = NameSpec;

    /// One `named` fact about a person, or one `place-named` fact about a place, whose file it is.
    /// A person named by the short form states the bytes every earlier `named` did (`ARC-363`).
    fn seed(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        authored: &NameSpec,
    ) -> Result<Vec<Emission>, Rejection> {
        let entity_type = seeding
            .world()
            .entity(subject)
            .map(|record| record.entity_type())
            .ok_or(Rejection::PreconditionFailed)?;
        let name = authored.default_name().clone();
        match entity_type {
            EntityType::Place => {
                let place = PlaceId::new(subject, entity_type)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                Ok(vec![
                    PlaceNamed::new(place, name, authored.forms()).emission(),
                ])
            }
            _ => {
                // A name in another kind of file would be refused by the loader (`CARRIED_BY`);
                // refused here too, because the type of the subject is this pack's to check.
                let person = PersonId::new(subject, entity_type)
                    .map_err(|_| Rejection::PreconditionFailed)?;
                Ok(vec![
                    Named::with_forms(person, name, authored.forms()).emission(),
                ])
            }
        }
    }
}
