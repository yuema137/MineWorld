//! The state this pack owns: a person's character.

use mineworld_kernel::owned_component;
use serde::{Deserialize, Serialize};

use crate::persona::Persona;
use crate::system::PersonaSystem;

/// A person's persona, held as their own component.
///
/// Serialized as `{ "summary": …, "traits": […], "speech": … }` — an object, because that is what an
/// observation reader takes a component payload to be (`clients/protocol/mineworld/observation.gd`
/// reads only object payloads). The Rust type is `PersonaComponent` because [`Persona`] names the
/// value it wraps; the component's type id is `persona`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PersonaComponent {
    persona: Persona,
}

impl PersonaComponent {
    /// This persona.
    pub const fn new(persona: Persona) -> Self {
        Self { persona }
    }

    /// The persona.
    pub const fn persona(&self) -> &Persona {
        &self.persona
    }
}

owned_component! {
    component = PersonaComponent,
    owner = PersonaSystem,
    component_type = "persona",
    schema_version = 1,
}
