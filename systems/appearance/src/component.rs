//! The state this pack owns: what a person looks like, as a key.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::look::LookKey;
use crate::system::AppearanceSystem;

/// A person's look.
///
/// Serialized as `{ "look": "alice-moreau" }`, an object and not a bare string, because that is what an
/// observation reader takes a component payload to be (`clients/protocol/mineworld/observation.gd`
/// reads only object payloads). The value is a key a Presentation Pack resolves (`ARC-92`); it is not
/// a name, and no client shows it as text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Look {
    look: LookKey,
}

impl Look {
    /// This look.
    pub const fn new(look: LookKey) -> Self {
        Self { look }
    }

    /// The key.
    pub const fn look(&self) -> &LookKey {
        &self.look
    }
}

owned_component! {
    component = Look,
    owner = AppearanceSystem,
    component_type = "look",
    schema_version = 1,
}
