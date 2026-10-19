//! A System Pack whose section type states no JSON Schema does not compile (`DECISIONS.md` `ARC-90`,
//! operator ruling QS25-2): the generated reference has an entry for every section of every installed
//! pack, so a section it could not describe is refused at the pack, not discovered as a gap in the
//! reference.

use mineworld_authoring::{AuthoredSection, ContentKind, SectionName, Seeding};
use mineworld_contracts::{EntityId, Rejection, SystemId};
use mineworld_kernel::{Emission, System, SystemDeclaration, SystemIdentity, SystemVersion};

#[derive(Default)]
struct Undocumented;

impl SystemIdentity for Undocumented {
    const ID: SystemId = SystemId::from_static("undocumented");
}

impl System for Undocumented {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

/// Decodes, but has no schema.
#[derive(Debug, serde::Deserialize)]
struct Spot {
    minutes: u32,
}

impl AuthoredSection for Undocumented {
    const SECTION: SectionName = SectionName::from_static("spot");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Place];
    type Authored = Spot;

    fn seed(_: &Seeding<'_, '_>, _: EntityId, _: &Spot) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

fn main() {}
