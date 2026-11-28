//! The person file's `attributes:` section, decoded in the catalogue, and the resolution over every
//! person (`ARC-121` items 2, 3; `pr-s30d-sampler.md` §5.7): the person's own value, the lock, roles, the
//! profile's sample, the declaration's default — by the model crate's sampler, which sees only each
//! subject's key, tags and section. Origins are the tools'; facts carry values only.

use std::any::Any;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

use mineworld_attributes_model::sample::{Fixed, Member, Words, resolve};
use mineworld_attributes_model::{
    ATTRIBUTES, AttributeValue, AttributesContext, AttributesHanded, Authored, NewcomerContext,
    QualifiedId, SectionIn, TraitsStated,
};
use mineworld_authoring::{
    ContentKind, ResolveRefusal, ResolvesSubjects, SectionInContext, SectionName, Seeding, Subject,
    SubjectResolution,
};
use mineworld_contracts::{EntityId, EntityType, PersonId, Rejection, SystemId, Visibility};
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
        context: Arc<AttributesContext>,
        _: Option<&dyn Any>,
        subjects: &[Subject<'_, Authored>],
    ) -> Result<SubjectResolution<Self::Context, Self::Seed>, ResolveRefusal> {
        let people: Vec<&Subject<'_, Authored>> = subjects
            .iter()
            .filter(|subject| subject.kind == ContentKind::Person)
            .collect();
        let context = finalize(context, &people)?;
        let people: Vec<_> = people
            .into_iter()
            .filter(|subject| {
                !context
                    .newcomers
                    .as_ref()
                    .is_some_and(|newcomers| newcomers.deferred.contains_key(subject.key))
            })
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
        // Deferred locks are already preserved in the finalized context. The genesis sampler
        // checks lock keys against its own members, so give it only the present people's entries.
        let lock = population
            .and_then(|population| population.lock.as_ref())
            .map(|lock| {
                let mut lock = lock.clone();
                lock.people
                    .retain(|key, _| people.iter().any(|person| person.key.as_str() == key));
                lock
            });
        let resolution = resolve(
            &context.catalogue,
            population.and_then(|population| population.profile.as_ref()),
            lock.as_ref(),
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
        Ok(SubjectResolution { context, seeds })
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

/// Complete the map before any context fact is emitted; no per-subject initialization protocol.
fn finalize(
    context: Arc<AttributesContext>,
    people: &[&Subject<'_, Authored>],
) -> Result<Arc<AttributesContext>, ResolveRefusal> {
    let Some(population) = &context.population else {
        return Ok(context);
    };
    // Validate every lock key before filtering deferred entries out of genesis resolution.
    for key in population.lock.iter().flat_map(|lock| lock.people.keys()) {
        if !people.iter().any(|subject| subject.key.as_str() == key) {
            return Err(ResolveRefusal {
                subject: None,
                reason: format!("UnknownPerson: the lock names '{key}'"),
            });
        }
    }
    let Some(later) = &population.later else {
        return Ok(context);
    };
    let mut deferred = BTreeMap::new();
    for key in later {
        let subject = people
            .iter()
            .find(|subject| subject.key == key)
            .ok_or_else(|| ResolveRefusal {
                subject: Some(key.clone()),
                reason: format!("UnknownPerson: newcomers.later names '{key}'"),
            })?;
        let locked = population
            .lock
            .as_ref()
            .and_then(|lock| lock.people.get(key.as_str()));
        deferred.insert(key.clone(), Fixed::of(subject.authored, locked));
    }
    let newcomers = NewcomerContext {
        profile: population.profile.clone(),
        seed: population.seed.clone(),
        salt: population.salt,
        deferred,
    };
    let mut finalized = (*context).clone();
    finalized.newcomers = Some(newcomers);
    Ok(Arc::new(finalized))
}
