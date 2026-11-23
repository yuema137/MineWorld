//! Closed-catalog adapter tests: probe owners use existing capability identities, but run the
//! production subject collection, finalization, installation and assembly paths. No public custom
//! pack loading or runtime registration is claimed here.
use super::*;
use crate::format::{AuthoredPerson, FoundConfiguration};
use mineworld_authoring::{
    Attached, ConfigurationContext, DataDirectory, DataOrigin, DataRefusal, DecodeConfiguration,
    DecodeIn, OwnsData, PackConfiguration, ResolvesSubjects, SectionInContext, SectionName,
    Seeding, Subject, SubjectResolution, data_context, resolve_erased,
};
use mineworld_contracts::{
    EntityId, Event, EventSchemaVersion, EventTypeId, Rejection, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity};
use serde::de::{DeserializeSeed, Deserializer};
use serde::{Deserialize, Serialize};
use std::any::Any;

#[derive(Debug)]
struct Context {
    total: u32,
}

macro_rules! owner {
    ($owner:ident, $fact:ident, $id:literal, $section:literal, $event:literal) => {
        struct $owner;
        impl SystemIdentity for $owner {
            const ID: SystemId = SystemId::from_static($id);
        }
        #[derive(Debug, Serialize, Deserialize)]
        struct $fact;
        impl Event for $fact {
            const EVENT_TYPE: EventTypeId = EventTypeId::from_static($event);
            const OWNER: SystemId = $owner::ID;
            const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
        }
        impl OwnsData for $owner {
            const DIRECTORIES: &'static [DataDirectory] = &[];
            type File = ();
            type Context = Context;
            fn decode_file<'de, D: Deserializer<'de>>(
                _: DataDirectory,
                d: D,
            ) -> Result<(), D::Error> {
                <()>::deserialize(d)
            }
            fn context(
                _: Vec<(DataOrigin, ())>,
                _: Option<&dyn Any>,
            ) -> Result<Context, DataRefusal> {
                Ok(Context { total: 0 })
            }
            const CONTEXT_FACTS: &'static [EventTypeId] = &[$fact::EVENT_TYPE];
            fn seed_context(
                _: &Seeding<'_, '_>,
                context: &Context,
            ) -> Result<Vec<Emission>, Rejection> {
                Ok(vec![Emission::new::<$fact>(
                    context.total.to_string().into_bytes(),
                    Visibility::SystemInternal,
                )])
            }
        }
        impl SectionInContext for $owner {
            const SECTION: SectionName = SectionName::from_static($section);
            const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
            type Authored = u32;
            fn decode<'de, D: Deserializer<'de>>(_: &Context, d: D) -> Result<u32, D::Error> {
                u32::deserialize(d)
            }
            fn seed(
                _: &Seeding<'_, '_>,
                _: EntityId,
                _: &Context,
                _: &u32,
            ) -> Result<Vec<Emission>, Rejection> {
                panic!("initial section must never seed, even when resolution omits it")
            }
        }
        impl ResolvesSubjects for $owner {
            type Seed = u32;
            fn resolve_subjects(
                context: Arc<Context>,
                configuration: Option<&dyn Any>,
                subjects: &[Subject<'_, u32>],
            ) -> Result<SubjectResolution<Context, u32>, ResolveRefusal> {
                assert_eq!(
                    context.total, 0,
                    "only the initial own context reaches resolution"
                );
                assert!(configuration.is_none_or(|c| c.is::<u32>()));
                let total = subjects
                    .iter()
                    .map(|s| s.authored.copied().unwrap_or(1))
                    .sum();
                let seeds = subjects
                    .iter()
                    .filter(|s| s.key.as_str() != "omit")
                    .map(|s| (s.key.clone(), s.authored.copied().unwrap_or(1)))
                    .collect();
                Ok(SubjectResolution {
                    context: Arc::new(Context { total }),
                    seeds,
                })
            }
            fn seed_resolved(
                _: &Seeding<'_, '_>,
                subject: EntityId,
                context: &Context,
                seed: &u32,
            ) -> Result<Vec<Emission>, Rejection> {
                Ok(vec![Emission::new::<$fact>(
                    format!("{}:{}:{}", subject.raw(), context.total, seed).into_bytes(),
                    Visibility::SystemInternal,
                )])
            }
        }
    };
}
owner!(First, FirstFact, "naming", "name", "final-first");
owner!(Second, SecondFact, "schedule", "routine", "final-second");

impl PackConfiguration for First {
    type Configuration = u32;
    const FACTS: &'static [EventTypeId] = &[FirstFact::EVENT_TYPE];
    fn seed(
        _: &Seeding<'_, '_>,
        value: &u32,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![Emission::new::<FirstFact>(
            value.to_string().into_bytes(),
            Visibility::SystemInternal,
        )])
    }
}
const OWNERS: [Capability; 2] = [Capability::Naming, Capability::Schedule];
fn key(s: &str) -> EntityKey {
    EntityKey::new(s).expect("key")
}
fn section<S: SectionInContext<Authored = u32> + 'static>(
    owner: Capability,
    context: &dyn OwnerContext,
    value: u32,
) -> FoundSection {
    FoundSection {
        owner,
        name: S::SECTION,
        state: SectionState::Decoded(
            DecodeIn::<S>::from_erased(context)
                .expect("context")
                .deserialize(serde_json::json!(value))
                .expect("section"),
        ),
    }
}
fn pack() -> WorldPack {
    let first = data_context::<First>(vec![], None).expect("first");
    let second = data_context::<Second>(vec![], None).expect("second");
    let people = [
        ("ada", Some((3, 10))),
        ("bare", None),
        ("omit", Some((5, 20))),
    ]
    .into_iter()
    .map(|(name, values)| {
        let sections = values
            .map(|(a, b)| {
                vec![
                    section::<First>(OWNERS[0], &*first, a),
                    section::<Second>(OWNERS[1], &*second, b),
                ]
            })
            .unwrap_or_default();
        (
            key(name),
            AuthoredPerson {
                sections,
                ..AuthoredPerson::default()
            },
        )
    })
    .collect();
    let configuration = DecodeConfiguration::<First>::new()
        .deserialize(serde_json::json!(99))
        .expect("configuration");
    WorldPack::in_memory(
        OWNERS.to_vec(),
        BTreeMap::new(),
        people,
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .with_contexts(BTreeMap::from([(OWNERS[0], first), (OWNERS[1], second)]))
    .with_configuration(vec![FoundConfiguration {
        owner: OWNERS[0],
        path: "configure/naming.yaml".into(),
        configuration,
        attached: Attached::default(),
    }])
}
fn resolve(
    owner: Capability,
    context: Option<&dyn OwnerContext>,
    configuration: Option<&dyn AuthoredConfiguration>,
    subjects: &[ErasedSubject<'_>],
) -> Result<Option<Arc<dyn ResolvedSeeds>>, ResolveRefusal> {
    match owner {
        Capability::Naming => {
            resolve_erased::<First>(context.expect("first context"), configuration, subjects)
                .map(Some)
        }
        Capability::Schedule => {
            resolve_erased::<Second>(context.expect("second context"), configuration, subjects)
                .map(Some)
        }
        _ => panic!("not a probe owner"),
    }
}

#[test]
fn final_context_reaches_tools_context_facts_and_subjects_in_order() {
    let mut pack = pack();
    finalize_with(&mut pack, &OWNERS, resolve).expect("finalized");
    for (owner, total) in [(OWNERS[0], 9), (OWNERS[1], 31)] {
        let context = pack.context(owner).expect("tool context");
        assert_eq!(
            context
                .as_any()
                .downcast_ref::<Context>()
                .expect("typed")
                .total,
            total
        );
        assert!(Arc::ptr_eq(
            &context.shared(),
            &pack.resolved()[&owner].context().shared()
        ));
    }
    let assembled = pack.assemble().expect("assemble");
    assert_eq!(assembled.facts, pack.assemble().expect("repeated").facts);
    let ada = assembled.ids[&key("ada")].raw();
    let bare = assembled.ids[&key("bare")].raw();
    let expected = [
        "99".to_owned(),
        "9".to_owned(),
        "31".to_owned(),
        format!("{ada}:9:3"),
        format!("{ada}:31:10"),
        format!("{bare}:9:1"),
        format!("{bare}:31:1"),
    ];
    let bytes: Vec<_> = assembled
        .facts
        .iter()
        .map(|fact| {
            if fact.event_type() == &FirstFact::EVENT_TYPE {
                fact.record().payload_for::<FirstFact>().expect("first")
            } else {
                fact.record().payload_for::<SecondFact>().expect("second")
            }
        })
        .collect();
    assert_eq!(
        bytes,
        expected.iter().map(String::as_bytes).collect::<Vec<_>>()
    );
}

#[test]
fn final_context_zero_subjects_and_nonresolving_owner_are_preserved() {
    let mut pack = WorldPack::in_memory(
        OWNERS.to_vec(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
        BTreeMap::new(),
    )
    .with_contexts(BTreeMap::from([
        (
            OWNERS[0],
            data_context::<First>(vec![], None).expect("first"),
        ),
        (
            OWNERS[1],
            data_context::<Second>(vec![], None).expect("second"),
        ),
    ]));
    let untouched = pack.context(OWNERS[1]).expect("second").clone();
    finalize_with(&mut pack, &OWNERS[..1], resolve).expect("empty resolution");
    assert!(Arc::ptr_eq(
        &untouched,
        pack.context(OWNERS[1]).expect("same context")
    ));
    assert_eq!(pack.assemble().expect("empty pack").facts.len(), 2);
}

#[derive(Debug)]
struct InvalidResult {
    inner: Arc<dyn ResolvedSeeds>,
    wrong_owner: bool,
    wrong_section: bool,
    wrong_context: bool,
}
impl ResolvedSeeds for InvalidResult {
    fn owner(&self) -> SystemId {
        if self.wrong_owner {
            First::ID
        } else {
            self.inner.owner()
        }
    }
    fn section(&self) -> SectionName {
        if self.wrong_section {
            First::SECTION
        } else {
            self.inner.section()
        }
    }
    fn context(&self) -> Arc<dyn OwnerContext> {
        if self.wrong_context {
            data_context::<First>(vec![], None).expect("wrong final context")
        } else {
            self.inner.context()
        }
    }
    fn as_any(&self) -> &dyn Any {
        self.inner.as_any()
    }
    fn seed(
        &self,
        s: &Seeding<'_, '_>,
        k: &EntityKey,
        id: EntityId,
    ) -> Option<Result<Vec<Emission>, Rejection>> {
        self.inner.seed(s, k, id)
    }
}
#[test]
fn final_context_later_refusal_or_invalid_result_publishes_neither_owner() {
    for failure in 0..5 {
        let mut pack = pack();
        let before: Vec<_> = OWNERS
            .iter()
            .map(|o| pack.context(*o).expect("initial").clone())
            .collect();
        let error = finalize_with(&mut pack, &OWNERS, |owner, context, config, subjects| {
            if owner == OWNERS[0] {
                return resolve(owner, context, config, subjects);
            }
            match failure {
                0 => Err(ResolveRefusal {
                    subject: None,
                    reason: "later owner refuses".to_owned(),
                }),
                1 => Ok(None),
                _ => Ok(Some(Arc::new(InvalidResult {
                    inner: resolve(owner, context, config, subjects)?.expect("result"),
                    wrong_owner: failure == 2,
                    wrong_section: failure == 3,
                    wrong_context: failure == 4,
                }))),
            }
        })
        .expect_err("must refuse");
        assert!(matches!(error, PackError::ResolveRefused { system, .. } if system == Second::ID));
        assert!(pack.resolved().is_empty(), "no earlier seeds published");
        for (owner, original) in OWNERS.iter().zip(before) {
            assert!(Arc::ptr_eq(
                &original,
                pack.context(*owner).expect("original context")
            ));
        }
    }
}
