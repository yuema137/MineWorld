//! A decoded section, held without its type until the world it seeds exists.
//!
//! A pack is read before its world is assembled, so a section is decoded (validated) at read time and
//! seeded later. Between the two the loader holds it as [`AuthoredContent`]: the owner's typed value
//! behind a trait object, so the loader can ask what it names and what it becomes without ever
//! learning its type.

use std::any::Any;
use std::marker::PhantomData;
use std::sync::Arc;

use mineworld_contracts::{EntityId, Rejection, SystemId};
use mineworld_kernel::Emission;
use serde::Deserialize;
use serde::de::{DeserializeSeed, Deserializer};

use crate::section::{AuthoredSection, Reference, SectionName, Seeding};

/// A section decoded by its owner's type, with that type erased.
pub trait AuthoredContent: core::fmt::Debug + Send + Sync {
    /// Which section it is.
    fn section(&self) -> SectionName;

    /// The System Pack that owns it.
    fn owner(&self) -> SystemId;

    /// The other entities it names (see [`AuthoredSection::references`]).
    fn references(&self) -> Vec<Reference<'_>>;

    /// The genesis facts it becomes (see [`AuthoredSection::seed`]).
    ///
    /// # Errors
    ///
    /// The owner's [`Rejection`].
    fn seed(
        &self,
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
    ) -> Result<Vec<Emission>, Rejection>;

    /// The owner's own decoded value (its `Authored`), for the owner to downcast when the loader hands
    /// the section back to it — a resolution's subjects (`ARC-121` item 3).
    ///
    /// Provided, answering `()`, so an implementation outside this crate keeps compiling; one that does
    /// not answer is simply not downcastable, and the downcast's failure is a named refusal.
    fn as_any(&self) -> &dyn Any {
        &()
    }
}

/// One owner's decoded value.
struct Held<S: AuthoredSection>(S::Authored);

impl<S: AuthoredSection> core::fmt::Debug for Held<S> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(S::SECTION.as_str()).field(&self.0).finish()
    }
}

impl<S: AuthoredSection + 'static> AuthoredContent for Held<S> {
    fn section(&self) -> SectionName {
        S::SECTION
    }

    fn owner(&self) -> SystemId {
        S::ID
    }

    fn references(&self) -> Vec<Reference<'_>> {
        S::references(&self.0)
    }

    fn seed(
        &self,
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
    ) -> Result<Vec<Emission>, Rejection> {
        S::seed(seeding, subject, &self.0)
    }

    fn as_any(&self) -> &dyn Any {
        &self.0
    }
}

/// Decodes `S`'s section straight from a deserializer — a file's YAML stream, for the loader — so a
/// refusal keeps the deserializer's own position (`DEP-10`).
pub struct Decode<S>(PhantomData<fn() -> S>);

impl<S> Decode<S> {
    /// A decoder for `S`'s section.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<S> Default for Decode<S> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, S: AuthoredSection + 'static> DeserializeSeed<'de> for Decode<S> {
    type Value = Arc<dyn AuthoredContent>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let authored = S::Authored::deserialize(deserializer)?;
        Ok(Arc::new(Held::<S>(authored)))
    }
}
