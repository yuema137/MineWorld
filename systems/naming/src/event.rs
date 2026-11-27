//! The fact this pack records: somebody is called something.

use mineworld_contracts::{
    Event, EventSchemaVersion, EventTypeId, PersonId, PlaceId, SystemId, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::{Deserialize, Serialize};

use crate::codec;
use crate::name::{Forms, Name};
use crate::system::NamingSystem;

/// `person` is called `name`.
///
/// Stated at genesis, from the person's authored `name:` section, and reduced by this pack alone into
/// [`DisplayName`](crate::DisplayName). Nothing renames a person in MVP-0; when something does, it
/// states this same fact.
///
/// `forms`, the name in other languages, is left out of the bytes when empty, so a person named by
/// the short form states exactly the bytes every earlier `named` fact did (`ARC-363`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Named {
    person: PersonId,
    name: Name,
    #[serde(default, skip_serializing_if = "Forms::is_empty")]
    forms: Forms,
}

impl Event for Named {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("named");
    const OWNER: SystemId = NamingSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl Named {
    /// States it.
    pub const fn new(person: PersonId, name: Name) -> Self {
        Self {
            person,
            name,
            forms: Forms::new(),
        }
    }

    /// States it with the name's forms in other languages.
    pub const fn with_forms(person: PersonId, name: Name, forms: Forms) -> Self {
        Self {
            person,
            name,
            forms,
        }
    }

    /// What they are called in other languages.
    pub const fn forms(&self) -> &Forms {
        &self.forms
    }

    /// Who.
    pub const fn person(&self) -> PersonId {
        self.person
    }

    /// What they are called.
    pub const fn name(&self) -> &Name {
        &self.name
    }

    /// As a fact ready to record: public, because names are public (`ARC-31`), about the person.
    pub fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::Public)
            .about(vec![self.person.entity_id()])
            .with_participants(vec![self.person.entity_id()])
    }
}

/// `place` is called `name`, and `forms` in other languages (`ARC-363`).
///
/// Stated at genesis, from the place's authored `name:` section, and reduced by this pack alone into
/// the place's [`DisplayName`](crate::DisplayName). A fact of its own rather than a generalised
/// [`Named`], so that every person's `named` fact keeps its bytes. `forms` is left out of the bytes
/// when empty.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlaceNamed {
    place: PlaceId,
    name: Name,
    #[serde(default, skip_serializing_if = "Forms::is_empty")]
    forms: Forms,
}

impl Event for PlaceNamed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("place-named");
    const OWNER: SystemId = NamingSystem::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl PlaceNamed {
    /// States it.
    pub const fn new(place: PlaceId, name: Name, forms: Forms) -> Self {
        Self { place, name, forms }
    }

    /// Which place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// What it is called.
    pub const fn name(&self) -> &Name {
        &self.name
    }

    /// What it is called in other languages.
    pub const fn forms(&self) -> &Forms {
        &self.forms
    }

    /// As a fact ready to record: public, about the place.
    pub fn emission(&self) -> Emission {
        Emission::new::<Self>(codec::encode(self), Visibility::Public)
            .about(vec![self.place.entity_id()])
    }
}
