//! The `attributes` pack's vocabulary (`MODULE_SPEC.md` §4.1 "Persistence"), defined here so owners and
//! readers decode it without depending on the pack (F-S30c-4), and the running catalogue.
//!
//! | Fact | Audience | Reduced into |
//! | --- | --- | --- |
//! | `attributes-composed` | `SystemInternal` | the `attributes-catalogue` Process |
//! | `traits-stated` | the holder | `Traits` |
//! | `attributes-handed` | `SystemInternal` | nothing in `attributes`: the owner subscribes |
//! | `attribute-change-proposed` | `SystemInternal` | checked: `trait-changed` or a named refusal |
//! | `trait-changed` | the holder, and the place when perceivable | `Traits` |

use core::marker::PhantomData;

use mineworld_contracts::{
    Event, EventId, EventSchemaVersion, EventTypeId, PersonId, ProcessTypeId, SystemId,
};
use mineworld_kernel::{ProcessKind, SystemIdentity, WorldRead};
use serde::{Deserialize, Serialize};

use crate::ATTRIBUTES;
use crate::catalogue::{Catalogue, Projection};
use crate::id::QualifiedId;
use crate::value::AttributeValue;

/// The schema of `attributes-composed`'s payload.
pub const COMPOSED_SCHEMA: u32 = 1;

/// The world's catalogue, stated once at genesis from `attributes`' data context (`ARC-121` item 5).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributesComposed {
    /// [`COMPOSED_SCHEMA`].
    pub schema: u32,
    /// The frozen catalogue's run-time projection.
    pub catalogue: Projection,
}

impl Event for AttributesComposed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("attributes-composed");
    const OWNER: SystemId = ATTRIBUTES;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// One person's plain traits as genesis states them, sorted by id.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitsStated {
    /// Whose.
    pub person: PersonId,
    /// The values, sorted by id.
    pub values: Vec<(QualifiedId, AttributeValue)>,
}

impl Event for TraitsStated {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("traits-stated");
    const OWNER: SystemId = ATTRIBUTES;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// One person's resolved values of one owner's attributes, handed to that owner at genesis.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributesHanded {
    /// Whose.
    pub person: PersonId,
    /// The owning System Pack.
    pub owner: SystemId,
    /// The values, sorted by id.
    pub values: Vec<(QualifiedId, AttributeValue)>,
}

impl Event for AttributesHanded {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("attributes-handed");
    const OWNER: SystemId = ATTRIBUTES;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// A proposer's request that one plain, evolving trait change, citing its cause.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeChangeProposed {
    /// Whose.
    pub person: PersonId,
    /// Which attribute.
    pub attribute: QualifiedId,
    /// To what.
    pub to: AttributeValue,
    /// The fact that is the reason (INV-15).
    pub because: EventId,
}

impl Event for AttributeChangeProposed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("attribute-change-proposed");
    const OWNER: SystemId = ATTRIBUTES;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// An accepted proposal: the trait changed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraitChanged {
    /// Whose.
    pub person: PersonId,
    /// Which attribute.
    pub attribute: QualifiedId,
    /// The stored value before, if any.
    pub from: Option<AttributeValue>,
    /// The value now.
    pub to: AttributeValue,
    /// The fact that is the reason.
    pub because: EventId,
}

impl Event for TraitChanged {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("trait-changed");
    const OWNER: SystemId = ATTRIBUTES;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The process type of the running catalogue.
pub const CATALOGUE_PROCESS: ProcessTypeId = ProcessTypeId::from_static("attributes-catalogue");

/// The running catalogue's process kind, generic over its owner type: `attributes` starts it as
/// `CatalogueProcess<AttributesSystem>`, every other reader reads it as `CatalogueProcess<AttributesId>`
/// (a process's state is checked by its type alone).
pub struct CatalogueProcess<O>(PhantomData<fn() -> O>);

impl<O: SystemIdentity + 'static> ProcessKind for CatalogueProcess<O> {
    const PROCESS_TYPE: ProcessTypeId = CATALOGUE_PROCESS;
    type Owner = O;
}

/// The identity `attributes` has, for a reader that does not depend on the pack.
#[derive(Debug, Clone, Copy, Default)]
pub struct AttributesId;

impl SystemIdentity for AttributesId {
    const ID: SystemId = ATTRIBUTES;
}

/// The catalogue a running world was composed with, read from its `attributes-catalogue` Process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunningCatalogue(Catalogue);

impl RunningCatalogue {
    /// The world's catalogue; `None` when the world does not enable `attributes` (or has not begun).
    pub fn in_world(world: &WorldRead<'_>) -> Option<Self> {
        let process = world.processes().find(|process| {
            *process.owner() == ATTRIBUTES && *process.process_type() == CATALOGUE_PROCESS
        })?;
        let bytes = process.state_for::<CatalogueProcess<AttributesId>>().ok()?;
        let projection: Projection = serde_json::from_slice(bytes).ok()?;
        Some(Self(Catalogue::from_projection(projection)))
    }

    /// The catalogue.
    pub const fn catalogue(&self) -> &Catalogue {
        &self.0
    }
}
