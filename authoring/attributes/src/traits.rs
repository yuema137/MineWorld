//! The `traits` component and typed reads (`DECISIONS.md` `ARC-120`, `ARC-122` item 3; `MODULE_SPEC.md`
//! §3.1 "Reads").
//!
//! `Traits` holds a person's plain, stored traits only: never a derived value and never an owner's
//! (I-S30-1, I-S30-2). It is defined here, with `Component` written by hand against the id `attributes`,
//! so a reader depends on this crate and not on the pack (F-S30c-4); the pack implements
//! `OwnedBy<AttributesSystem>`, so only it writes one.
//!
//! A reader holds constants — `const CHRONOTYPE: Handle<Level> = Handle::declared("core.chronotype");`
//! — and declares the same ids and kinds in the `uses:` of a file it ships, which the catalogue checks
//! at load. `Handle<Text>` does not compile: `Text` is not [`SystemReadable`].

use core::marker::PhantomData;
use std::collections::BTreeMap;

use mineworld_contracts::{Component, ComponentSchemaVersion, ComponentTypeId, EntityId, SystemId};
use mineworld_kernel::WorldRead;
use serde::{Deserialize, Serialize};

use crate::ATTRIBUTES;
use crate::catalogue::Catalogue;
use crate::declaration::Mutability;
use crate::derive;
use crate::facts::RunningCatalogue;
use crate::id::{QualifiedId, is_qualified};
use crate::kind::SystemReadable;
use crate::value::AttributeValue;

/// A person's plain, stored traits: `attributes`-owned, not derived.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Traits(BTreeMap<QualifiedId, AttributeValue>);

impl Component for Traits {
    const COMPONENT_TYPE: ComponentTypeId = ComponentTypeId::from_static("traits");
    const OWNER: SystemId = ATTRIBUTES;
    const SCHEMA_VERSION: ComponentSchemaVersion = ComponentSchemaVersion::new(1);
}

impl Traits {
    /// The traits `values` state, if every one is a plain trait of `catalogue` of the right kind.
    ///
    /// # Errors
    ///
    /// The first value that is unknown, derived, an owner's, or of another kind.
    pub fn stated(
        catalogue: &Catalogue,
        values: impl IntoIterator<Item = (QualifiedId, AttributeValue)>,
    ) -> Result<Self, String> {
        let mut traits = Self::default();
        for (id, value) in values {
            traits.set(catalogue, id, value)?;
        }
        Ok(traits)
    }

    /// Sets one plain trait, checked as [`Traits::stated`] checks each.
    ///
    /// # Errors
    ///
    /// As [`Traits::stated`].
    pub fn set(
        &mut self,
        catalogue: &Catalogue,
        id: QualifiedId,
        value: AttributeValue,
    ) -> Result<(), String> {
        let declaration = catalogue
            .get(&id)
            .ok_or_else(|| format!("'{id}' is declared by no catalogue"))?;
        if declaration.mutability == Mutability::Derived {
            return Err(format!("'{id}' is derived and is never stored"));
        }
        if declaration.owner != ATTRIBUTES {
            return Err(format!("'{id}' is owned by '{}'", declaration.owner));
        }
        if value.kind() != declaration.kind.name() {
            return Err(format!(
                "'{id}' is a {}, not a {}",
                declaration.kind.name(),
                value.kind()
            ));
        }
        self.0.insert(id, value);
        Ok(())
    }

    /// Every stored trait, in id order.
    pub const fn values(&self) -> &BTreeMap<QualifiedId, AttributeValue> {
        &self.0
    }

    /// One stored trait.
    pub fn value(&self, id: &QualifiedId) -> Option<&AttributeValue> {
        self.0.get(id)
    }

    /// A stored trait, through a typed handle.
    pub fn get<K: SystemReadable>(&self, handle: &Handle<K>) -> Option<K::Value> {
        self.0.get(&handle.id()).and_then(K::read)
    }

    /// Whether it holds no trait.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

/// A typed handle to one attribute: a constant of the reading pack, never built from a string at run
/// time (X-S30-10).
#[derive(Debug)]
pub struct Handle<K: SystemReadable> {
    id: &'static str,
    kind: PhantomData<fn() -> K>,
}

impl<K: SystemReadable> Clone for Handle<K> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<K: SystemReadable> Copy for Handle<K> {}

impl<K: SystemReadable> Handle<K> {
    /// The handle to `id`, which the reading pack also declares in its shipped `uses:`.
    ///
    /// # Panics
    ///
    /// When `id` is not a qualified id — at compile time, for a constant.
    pub const fn declared(id: &'static str) -> Self {
        assert!(is_qualified(id), "a handle names a qualified attribute id");
        Self {
            id,
            kind: PhantomData,
        }
    }

    /// The id it names.
    pub fn id(&self) -> QualifiedId {
        QualifiedId::new(self.id)
            .unwrap_or_else(|_| unreachable!("checked when the constant was built"))
    }

    /// The id, as written.
    pub const fn as_str(&self) -> &'static str {
        self.id
    }
}

/// `person`'s value of the attribute `handle` names, stored or derived, at the read's instant; `None`
/// when absent, or when the world does not enable `attributes`.
pub fn read<K: SystemReadable>(
    world: &WorldRead<'_>,
    person: EntityId,
    handle: &Handle<K>,
) -> Option<K::Value> {
    let catalogue = RunningCatalogue::in_world(world)?;
    let traits = world
        .component::<Traits>(person)
        .cloned()
        .unwrap_or_default();
    let id = handle.id();
    let declaration = catalogue.catalogue().get(&id)?;
    let value = match declaration.mutability {
        Mutability::Derived => {
            derive::value(catalogue.catalogue(), declaration, &traits, world.at())?
        }
        Mutability::Fixed | Mutability::Evolving => traits.value(&id)?.clone(),
    };
    K::read(&value)
}
