//! A component must not be able to exist without naming the system that owns it: the
//! single-writer rule is only mechanical if the declaration cannot be left out.

use mineworld_contracts::{Component, ComponentSchemaVersion, ComponentTypeId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Unowned {
    amount: u32,
}

impl Component for Unowned {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("unowned");
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

fn main() {}
