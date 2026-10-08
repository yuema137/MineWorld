//! What a System Pack declares about the world-level configuration it accepts (`DECISIONS.md`
//! `ARC-61`).
//!
//! A section (`ARC-31`) is part of one content file and is about that file's entity. A configuration is
//! one file per pack per world — `configure/<id>.yaml`, listed in `world.yaml`'s `configure:` — and is
//! about no entity. Otherwise the shape is the same, one level up: the pack's own type decodes it, so
//! deserializing is validating; the loader holds it type erased until the world exists; and the pack
//! seeds its own genesis facts from it. The loader never learns what a configuration means.

use std::marker::PhantomData;
use std::sync::Arc;

use mineworld_contracts::{EventTypeId, Rejection, SystemId};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::Deserialize;
use serde::de::{DeserializeOwned, DeserializeSeed, Deserializer};

use crate::section::{Reference, Seeding};

/// A world-level configuration a System Pack accepts (`ARC-61`).
///
/// Implemented by the pack that owns the state the configuration becomes. Its facts should be stated
/// `Visibility::SystemInternal` with no subjects: a world's configuration is nobody's perception and
/// part of no person's history (`INV-13`, `ARC-61` item 8). That is guidance, not a loader rule (QIA-4).
pub trait PackConfiguration: SystemIdentity {
    /// The configuration, as authored. Deserializing it *is* the owner's validation: the loader decodes
    /// it straight from the file, and a refusal carries the file's line and column and this type's own
    /// message.
    type Configuration: DeserializeOwned + core::fmt::Debug + Send + Sync + 'static;

    /// The event types its configuration may seed. The loader refuses a seeded fact of any other type,
    /// and the drift check at resume compares exactly the genesis facts of these types.
    const FACTS: &'static [EventTypeId];

    /// The entities the configuration names by key, each with the entity type it must be.
    fn references(configuration: &Self::Configuration) -> Vec<Reference<'_>> {
        let _ = configuration;
        Vec::new()
    }

    /// The other systems the configuration needs enabled in the same world.
    fn requires(configuration: &Self::Configuration) -> Vec<SystemId> {
        let _ = configuration;
        Vec::new()
    }

    /// The genesis facts the configuration becomes, in this pack's own vocabulary.
    ///
    /// # Errors
    ///
    /// A [`Rejection`] when the world as assembled cannot take the value.
    fn seed(
        seeding: &Seeding<'_, '_>,
        configuration: &Self::Configuration,
    ) -> Result<Vec<Emission>, Rejection>;
}

/// A configuration decoded by its owner's type, with that type erased.
pub trait AuthoredConfiguration: core::fmt::Debug + Send + Sync {
    /// The System Pack that owns it.
    fn owner(&self) -> SystemId;

    /// The entities it names (see [`PackConfiguration::references`]).
    fn references(&self) -> Vec<Reference<'_>>;

    /// The systems it needs enabled (see [`PackConfiguration::requires`]).
    fn requires(&self) -> Vec<SystemId>;

    /// The genesis facts it becomes (see [`PackConfiguration::seed`]).
    ///
    /// # Errors
    ///
    /// The owner's [`Rejection`].
    fn seed(&self, seeding: &Seeding<'_, '_>) -> Result<Vec<Emission>, Rejection>;
}

/// One owner's decoded configuration.
struct Held<P: PackConfiguration>(P::Configuration);

impl<P: PackConfiguration> core::fmt::Debug for Held<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_tuple(P::ID.as_str()).field(&self.0).finish()
    }
}

impl<P: PackConfiguration + 'static> AuthoredConfiguration for Held<P> {
    fn owner(&self) -> SystemId {
        P::ID
    }

    fn references(&self) -> Vec<Reference<'_>> {
        P::references(&self.0)
    }

    fn requires(&self) -> Vec<SystemId> {
        P::requires(&self.0)
    }

    fn seed(&self, seeding: &Seeding<'_, '_>) -> Result<Vec<Emission>, Rejection> {
        P::seed(seeding, &self.0)
    }
}

/// Decodes `P`'s configuration straight from a deserializer — a file's YAML stream, for the loader — so
/// a refusal keeps the deserializer's own position (`DEP-10`).
pub struct DecodeConfiguration<P>(PhantomData<fn() -> P>);

impl<P> DecodeConfiguration<P> {
    /// A decoder for `P`'s configuration.
    pub const fn new() -> Self {
        Self(PhantomData)
    }
}

impl<P> Default for DecodeConfiguration<P> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'de, P: PackConfiguration + 'static> DeserializeSeed<'de> for DecodeConfiguration<P> {
    type Value = Arc<dyn AuthoredConfiguration>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        let configuration = P::Configuration::deserialize(deserializer)?;
        Ok(Arc::new(Held::<P>(configuration)))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use mineworld_contracts::{EntityKey, EntityType, RejectionCode};
    use mineworld_kernel::World;
    use serde::de::value::{Error, MapDeserializer};

    use super::*;

    /// A step per place, each bounded by the type itself.
    type Settings = BTreeMap<EntityKey, Step>;

    /// 1 … 100, refused otherwise: the owner's bound, enforced by decoding.
    #[derive(Debug, Deserialize)]
    #[serde(try_from = "u32")]
    struct Step(u32);

    impl TryFrom<u32> for Step {
        type Error = String;

        fn try_from(value: u32) -> Result<Self, Self::Error> {
            if (1..=100).contains(&value) {
                Ok(Self(value))
            } else {
                Err(format!("a step is 1 to 100, not {value}"))
            }
        }
    }

    /// A pack whose configuration names places, needs another system, and refuses to seed a step
    /// above 50.
    struct Probe;

    impl SystemIdentity for Probe {
        const ID: SystemId = SystemId::from_static("probe");
    }

    impl PackConfiguration for Probe {
        type Configuration = Settings;
        const FACTS: &'static [EventTypeId] = &[];

        fn references(configuration: &Settings) -> Vec<Reference<'_>> {
            configuration
                .keys()
                .map(|key| Reference {
                    key,
                    entity_type: EntityType::Place,
                })
                .collect()
        }

        fn requires(_: &Settings) -> Vec<SystemId> {
            vec![SystemId::from_static("other")]
        }

        fn seed(_: &Seeding<'_, '_>, configuration: &Settings) -> Result<Vec<Emission>, Rejection> {
            match configuration.values().find(|step| step.0 > 50) {
                Some(step) => Err(Rejection::System {
                    code: RejectionCode::from_static("probe-step-too-large"),
                    detail: Some(format!("step {}", step.0)),
                }),
                None => Ok(Vec::new()),
            }
        }
    }

    /// `{ square: <step> }`, decoded through serde's own value deserializer (deviation D-1).
    fn decode(step: u32) -> Result<Arc<dyn AuthoredConfiguration>, Error> {
        let map = MapDeserializer::<_, Error>::new([("square", step)].into_iter());
        DecodeConfiguration::<Probe>::new().deserialize(map)
    }

    /// Decoding goes through the owner's type: its bound refuses with its own message, and a decoded
    /// value answers for its owner, what it names and what it requires.
    #[test]
    fn decoding_is_the_owners_validation_and_the_erased_value_answers_for_its_owner() {
        let refusal = decode(0).expect_err("0 is outside the owner's bound");
        assert!(
            refusal.to_string().contains("a step is 1 to 100, not 0"),
            "the owner's own message: {refusal}"
        );

        let decoded = decode(7).expect("7 is within the bound");
        assert_eq!(decoded.owner(), SystemId::from_static("probe"));
        assert_eq!(decoded.requires(), [SystemId::from_static("other")]);
        let references = decoded.references();
        assert_eq!(references.len(), 1);
        assert_eq!(references[0].key.as_str(), "square");
        assert_eq!(references[0].entity_type, EntityType::Place);
    }

    /// The owner's refusal at seeding reaches the loader unchanged.
    #[test]
    fn a_seed_refusal_is_the_owners_and_is_propagated() {
        let world = World::new();
        let read = world.read();
        let keys = BTreeMap::new();
        let seeding = Seeding::new(&read, &keys);

        assert_eq!(decode(50).expect("decodes").seed(&seeding), Ok(Vec::new()));
        let refusal = decode(51)
            .expect("decodes")
            .seed(&seeding)
            .expect_err("the owner refuses 51");
        assert_eq!(
            refusal,
            Rejection::System {
                code: RejectionCode::from_static("probe-step-too-large"),
                detail: Some("step 51".to_owned()),
            }
        );
    }
}
