//! The state this pack owns: what a person or a place is called.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::name::{Forms, Name};
use crate::system::NamingSystem;

/// A person's or a place's display name, with its forms in other languages.
///
/// Serialized as `{ "name": "…" }` — an object, not a bare string, because that is what an observation
/// reader takes a component payload to be (`clients/protocol/mineworld/observation.gd` reads only object
/// payloads) — plus `"forms": { "zh_Hans": "…" }` when the author wrote any. Without forms the
/// payload is byte-identical to the one before forms existed, so `schema_version` stays 1 and every
/// reader that reads `name` keeps working (`ARC-363`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisplayName {
    name: Name,
    #[serde(default, skip_serializing_if = "Forms::is_empty")]
    forms: Forms,
}

impl DisplayName {
    /// This name, with no other form.
    pub const fn new(name: Name) -> Self {
        Self {
            name,
            forms: Forms::new(),
        }
    }

    /// This name, with its forms in other languages.
    pub const fn with_forms(name: Name, forms: Forms) -> Self {
        Self { name, forms }
    }

    /// The default name.
    pub const fn name(&self) -> &Name {
        &self.name
    }

    /// The forms in other languages, by locale.
    pub const fn forms(&self) -> &Forms {
        &self.forms
    }
}

owned_component! {
    component = DisplayName,
    owner = NamingSystem,
    component_type = "display-name",
    schema_version = 1,
}
