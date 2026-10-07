//! The state this pack owns: what a person is called.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::name::Name;
use crate::system::NamingSystem;

/// A person's display name.
///
/// Serialized as `{ "name": "…" }` — an object, not a bare string, because that is what an observation
/// reader takes a component payload to be (`clients/protocol/mineworld/observation.gd` reads only object
/// payloads), and because a name may later carry more than one form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayName {
    name: Name,
}

impl DisplayName {
    /// This name.
    pub const fn new(name: Name) -> Self {
        Self { name }
    }

    /// The name.
    pub const fn name(&self) -> &Name {
        &self.name
    }
}

owned_component! {
    component = DisplayName,
    owner = NamingSystem,
    component_type = "display-name",
    schema_version = 1,
}
