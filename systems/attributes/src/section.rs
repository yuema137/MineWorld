//! The person file's `attributes:` section, decoded in the catalogue, and the resolution over every
//! person (`ARC-121` items 2, 3; `pr-s30d-sampler.md` §5.7): the person's own value, the lock, roles, the
//! profile's sample, the declaration's default — by the model crate's sampler, which sees only each
//! subject's key, tags and section. Origins are the tools'; facts carry values only.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};

use mineworld_attributes_model::sample::{Member, Words, resolve};
use mineworld_attributes_model::{
    ATTRIBUTES, AttributeValue, AttributesContext, AttributesHanded, Authored, QualifiedId,
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
        context: &AttributesContext,
        deserializer: D,
    ) -> Result<Authored, D::Error> {
        SectionIn(&context.catalogue).deserialize(deserializer)
    }

    /// Never called: the pack resolves, and its resolution seeds.
    fn seed(
        _: &Seeding<'_, '_>,
        _: EntityId,
        _: &AttributesContext,
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

impl ResolvesSubjects for AttributesSystem {
    type Seed = Resolved;

    /// Every person, by the sampler's precedence.
    fn resolve_subjects(
        context: &AttributesContext,
        _: Option<&dyn Any>,
        subjects: &[Subject<'_, Authored>],
    ) -> Result<BTreeMap<EntityKey, Resolved>, ResolveRefusal> {
        let people: Vec<&Subject<'_, Authored>> = subjects
            .iter()
            .filter(|subject| subject.kind == ContentKind::Person)
            .collect();
        let members: Vec<Member<'_>> = people
            .iter()
            .map(|subject| Member {
                key: subject.key.as_str(),
                tags: subject.tags,
                authored: subject.authored,
            })
            .collect();
        let population = context.population.as_ref();
        let words = Words::new(
            population.map_or("", |population| population.seed.as_str()),
            population.map_or(0, |population| population.salt),
        );
        let resolution = resolve(
            &context.catalogue,
            population.and_then(|population| population.profile.as_ref()),
            population.and_then(|population| population.lock.as_ref()),
            words,
            &members,
        )
        .map_err(|refusal| ResolveRefusal {
            subject: None,
            reason: refusal.to_string(),
        })?;
        let mut seeds = BTreeMap::new();
        for subject in people {
            let mut resolved = Resolved::default();
            for (id, (value, _)) in resolution
                .people
                .get(subject.key.as_str())
                .into_iter()
                .flatten()
            {
                let Some(declaration) = context.catalogue.get(id) else {
                    continue;
                };
                let target = if declaration.owner == ATTRIBUTES {
                    &mut resolved.plain
                } else {
                    resolved.owned.entry(declaration.owner.clone()).or_default()
                };
                target.insert(id.clone(), value.clone());
            }
            seeds.insert(subject.key.clone(), resolved);
        }
        Ok(seeds)
    }

    /// `traits-stated` (when there is a plain trait), then one `attributes-handed` per owner.
    fn seed_resolved(
        seeding: &Seeding<'_, '_>,
        subject: EntityId,
        _: &AttributesContext,
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
