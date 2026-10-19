//! The person file's `attributes:` section, decoded in the catalogue, and the resolution over every
//! person (`ARC-121` items 2, 3; design §5.7): the person's own value, else the declaration's `default`;
//! otherwise absent. S30-d inserts the lock, roles and the profile's sample between the two.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};

use mineworld_attributes_model::{
    ATTRIBUTES, AttributeValue, AttributesHanded, Authored, Catalogue, Mutability, QualifiedId,
    SectionIn, TraitsStated,
};
use mineworld_authoring::{
    ContentKind, ResolveRefusal, ResolvesSubjects, SectionInContext, SectionName, Seeding, Subject,
};
use mineworld_contracts::{
    EntityId, EntityKey, EntityType, PersonId, Rejection, SystemId, Visibility,
};
use mineworld_kernel::Emission;
use serde::de::{DeserializeSeed, Deserializer};

use crate::system::{AttributesSystem, encode};

impl SectionInContext for AttributesSystem {
    const SECTION: SectionName = SectionName::from_static("attributes");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
    type Authored = Authored;

    fn decode<'de, D: Deserializer<'de>>(
        catalogue: &Catalogue,
        deserializer: D,
    ) -> Result<Authored, D::Error> {
        SectionIn(catalogue).deserialize(deserializer)
    }

    /// Never called: the pack resolves, and its resolution seeds.
    fn seed(
        _: &Seeding<'_, '_>,
        _: EntityId,
        _: &Catalogue,
        _: &Authored,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

/// One person's resolved values: the plain traits, and each owner's values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    /// Plain traits, which `traits-stated` states.
    pub plain: BTreeMap<QualifiedId, AttributeValue>,
    /// Each owner's values, which `attributes-handed` hands it.
    pub owned: BTreeMap<SystemId, BTreeMap<QualifiedId, AttributeValue>>,
}

/// One subject's values: authored, else the default; derived attributes never.
pub(crate) fn resolve_one(catalogue: &Catalogue, authored: Option<&Authored>) -> Resolved {
    let mut resolved = Resolved::default();
    for declaration in catalogue.attributes() {
        if declaration.mutability == Mutability::Derived {
            continue;
        }
        let value = authored
            .and_then(|authored| authored.values.get(&declaration.id))
            .or(declaration.default.as_ref());
        let Some(value) = value else {
            continue;
        };
        let target = if declaration.owner == ATTRIBUTES {
            &mut resolved.plain
        } else {
            resolved.owned.entry(declaration.owner.clone()).or_default()
        };
        target.insert(declaration.id.clone(), value.clone());
    }
    resolved
}

impl ResolvesSubjects for AttributesSystem {
    type Seed = Resolved;

    /// Every person, by the precedence S30-c has; the resolution sees only key, kind, tags and section.
    fn resolve_subjects(
        catalogue: &Catalogue,
        _: Option<&dyn Any>,
        subjects: &[Subject<'_, Authored>],
    ) -> Result<BTreeMap<EntityKey, Resolved>, ResolveRefusal> {
        Ok(subjects
            .iter()
            .filter(|subject| subject.kind == ContentKind::Person)
            .map(|subject| {
                (
                    subject.key.clone(),
                    resolve_one(catalogue, subject.authored),
                )
            })
            .collect())
    }

    /// `traits-stated` (when there is a plain trait), then one `attributes-handed` per owner.
    fn seed_resolved(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        _: &Catalogue,
        seed: &Resolved,
    ) -> Result<Vec<Emission>, Rejection> {
        let person = PersonId::new(subject, EntityType::Person)
            .ok()
            .filter(|_| {
                seeding
                    .world()
                    .entity(subject)
                    .is_some_and(|entity| entity.entity_type() == EntityType::Person)
            })
            .ok_or(Rejection::PreconditionFailed)?;
        let mut facts = Vec::new();
        if !seed.plain.is_empty() {
            let stated = TraitsStated {
                person,
                values: seed.plain.clone().into_iter().collect(),
            };
            facts.push(
                Emission::new::<TraitsStated>(
                    encode(&stated),
                    Visibility::Entities(BTreeSet::from([subject])),
                )
                .about(vec![subject]),
            );
        }
        for (owner, values) in &seed.owned {
            let handed = AttributesHanded {
                person,
                owner: owner.clone(),
                values: values.clone().into_iter().collect(),
            };
            facts.push(
                Emission::new::<AttributesHanded>(encode(&handed), Visibility::SystemInternal)
                    .about(vec![subject]),
            );
        }
        Ok(facts)
    }
}
