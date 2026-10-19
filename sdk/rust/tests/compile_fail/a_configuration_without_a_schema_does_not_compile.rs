//! A System Pack whose configuration type states no JSON Schema does not compile (`DECISIONS.md`
//! `ARC-90`, operator ruling QS25-2): the generated reference has an entry for every configuration of
//! every installed pack, so a configuration it could not describe is refused at the pack, not
//! discovered as a gap in the reference.

use mineworld_authoring::{ConfigurationContext, PackConfiguration, Seeding};
use mineworld_contracts::{EventTypeId, Rejection, SystemId};
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
struct Settings {
    minutes: u32,
}

impl PackConfiguration for Undocumented {
    type Configuration = Settings;
    const FACTS: &'static [EventTypeId] = &[];

    fn seed(
        _: &Seeding<'_, '_>,
        _: &Settings,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

fn main() {}
