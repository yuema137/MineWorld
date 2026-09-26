//! The component model: typed state attached to an entity by exactly one system.
//!
//! This module exists to make one sentence of `docs/CORE_CONCEPTS.md` §3.1 mechanical —
//! *"A Component is typed state attached to an Entity by exactly one system"* — so that the
//! single-writer rule (`INV-7`) can be enforced by the kernel instead of remembered by
//! reviewers.
//!
//! Ownership is a fact about the component *type*, not about an instance: implementing
//! [`Component`] is how a system claims a kind of state, and there is no way to implement it
//! without naming an owner. The kernel's registry then refuses a second claim on the same
//! component type by a different system, and the check it needs is
//! [`ComponentDeclaration::conflicts_with`].
//!
//! # The one place a payload is erased
//!
//! [`ComponentRecord`] is the only type in this crate that holds a component's contents in a
//! form it cannot interpret, and it exists because a store and a wire have to: a table row or a
//! network frame carries bytes, not a Rust type. Everything else in the crate — and everything
//! a system writes — works with typed values. A record cannot be built without a component type
//! to label it, and reading one back for a typed component checks both the type and the schema
//! version first.
//!
//! The encoding of those bytes is deliberately *not* decided here. A contract layer that chose
//! JSON, or Protobuf, or a binary codec would bake a wire format into the kernel's vocabulary
//! before the boundary that needs one exists; the payload is therefore a type parameter, and
//! the persistence layer supplies it (defaulting to `Vec<u8>`).

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::ids::{ComponentTypeId, EntityId, SystemId};

/// Which version of a component type's schema a value was written against.
///
/// Ordered, because the useful questions are comparisons: is this record older than the code
/// reading it (migrate), the same (decode), or newer (refuse — a future version knows fields
/// this code would silently drop).
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
#[serde(transparent)]
pub struct ComponentSchemaVersion(u32);

impl ComponentSchemaVersion {
    /// The version numbered `version`. A component type's first schema is version 1 by
    /// convention; the numbers carry no meaning beyond their order.
    pub const fn new(version: u32) -> Self {
        Self(version)
    }

    /// The version number.
    pub const fn value(self) -> u32 {
        self.0
    }
}

impl core::fmt::Display for ComponentSchemaVersion {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "v{}", self.0)
    }
}

/// Typed state one system owns.
///
/// Implementing this trait *is* the ownership declaration (`INV-7`): the three constants are
/// part of the type, so a component cannot exist without an owner, two instances of the same
/// component cannot disagree about who writes them, and no later code can reassign ownership
/// without editing the declaration. The identifiers are literals checked while the declaring
/// crate compiles — see [`ComponentTypeId::from_static`].
///
/// ```
/// use mineworld_contracts::{Component, ComponentSchemaVersion, ComponentTypeId, SystemId};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Serialize, Deserialize)]
/// struct Occupancy {
///     present: u32,
/// }
///
/// impl Component for Occupancy {
///     const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("occupancy");
///     const OWNER: SystemId = SystemId::from_static("places");
///     const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
/// }
/// ```
///
/// The `serde` bounds are not a convenience: world history is the event log, state is rebuilt
/// from it, and a component that cannot be written and read back cannot be persisted or
/// replayed.
pub trait Component: Serialize + DeserializeOwned + Sized {
    /// The name of this kind of state.
    const COMPONENT_TYPE: ComponentTypeId;
    /// The one system allowed to write it.
    const OWNER: SystemId;
    /// The version of this type's schema.
    const SCHEMA_VERSION: ComponentSchemaVersion;
}

/// The same facts a [`Component`] declares, as a value the kernel can hold in a registry.
///
/// A registry cannot store types, so it stores these. Deriving one from a component type is
/// infallible, because the declaration was already checked when the component's crate compiled.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ComponentDeclaration {
    component_type: ComponentTypeId,
    owner: SystemId,
    schema_version: ComponentSchemaVersion,
}

impl ComponentDeclaration {
    /// Reads the declaration off a component type.
    pub fn of<C: Component>() -> Self {
        Self {
            component_type: C::COMPONENT_TYPE,
            owner: C::OWNER,
            schema_version: C::SCHEMA_VERSION,
        }
    }

    /// The kind of state being declared.
    pub const fn component_type(&self) -> &ComponentTypeId {
        &self.component_type
    }

    /// The system that owns it, and therefore the only writer.
    pub const fn owner(&self) -> &SystemId {
        &self.owner
    }

    /// The schema version the owning system currently writes.
    pub const fn schema_version(&self) -> ComponentSchemaVersion {
        self.schema_version
    }

    /// Whether these two declarations claim the same component type for different systems.
    ///
    /// This is the conflict a registry must refuse: two writers for one piece of state is
    /// exactly what `INV-7` forbids, and it is detectable here, from declarations alone, before
    /// any world runs. Two declarations of the same type by the same system are not a conflict —
    /// that is one system registering twice, or two versions of one system's schema.
    pub fn conflicts_with(&self, other: &Self) -> bool {
        self.component_type == other.component_type && self.owner != other.owner
    }
}

/// A component's contents as a store or a wire carries them: labelled, versioned, and opaque.
///
/// This is the single documented payload-erasure boundary of the contract layer. `P` is the
/// encoded form the persistence layer chose — bytes by default — and this crate never
/// interprets it. What it does guarantee is that the label cannot lie: a record can only be
/// built from a component type, and [`ComponentRecord::payload_for`] refuses to hand the
/// payload to a component type it was not written for, or to one whose schema version does not
/// match.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ComponentRecord<P = Vec<u8>> {
    entity: EntityId,
    component_type: ComponentTypeId,
    schema_version: ComponentSchemaVersion,
    payload: P,
}

impl<P> ComponentRecord<P> {
    /// Labels an already-encoded payload with the component type it was encoded from.
    ///
    /// The type and the schema version come from `C` rather than from the caller, so a record
    /// cannot be mislabelled: the only way to claim a component type is to name it as a type
    /// parameter.
    pub fn new<C: Component>(entity: EntityId, payload: P) -> Self {
        Self {
            entity,
            component_type: C::COMPONENT_TYPE,
            schema_version: C::SCHEMA_VERSION,
            payload,
        }
    }

    /// The entity this state belongs to.
    pub const fn entity(&self) -> EntityId {
        self.entity
    }

    /// The component type the payload was written from.
    pub const fn component_type(&self) -> &ComponentTypeId {
        &self.component_type
    }

    /// The schema version the payload was written against.
    pub const fn schema_version(&self) -> ComponentSchemaVersion {
        self.schema_version
    }

    /// The encoded payload, unchecked — for a store moving a record it does not interpret.
    /// Code that intends to decode the payload into a component uses
    /// [`ComponentRecord::payload_for`] instead.
    pub const fn payload(&self) -> &P {
        &self.payload
    }

    /// The payload, if this record was written for `C` at a schema version `C` can read.
    ///
    /// Three outcomes, three distinct errors, because a caller must react differently to each:
    /// a record of a different component type is a bug in whoever routed it; a record from a
    /// newer schema cannot be decoded at all by this code and must not be guessed at; a record
    /// from an older schema is ordinary history that a migration has to bring forward.
    pub fn payload_for<C: Component>(&self) -> Result<&P, ContractError> {
        if self.component_type != C::COMPONENT_TYPE {
            return Err(ContractError::ComponentTypeMismatch {
                expected: C::COMPONENT_TYPE,
                actual: self.component_type.clone(),
            });
        }
        if self.schema_version > C::SCHEMA_VERSION {
            return Err(ContractError::ComponentSchemaTooNew {
                component_type: C::COMPONENT_TYPE,
                record: self.schema_version,
                supported: C::SCHEMA_VERSION,
            });
        }
        if self.schema_version < C::SCHEMA_VERSION {
            return Err(ContractError::ComponentSchemaOutdated {
                component_type: C::COMPONENT_TYPE,
                record: self.schema_version,
                supported: C::SCHEMA_VERSION,
            });
        }
        Ok(&self.payload)
    }
}
