//! S30-c AC-c-3 (`pr-s30c-attributes-pack.md` §8): a test-only reader and proposer, `test-early-bird`,
//! with the real `attributes` pack in a kernel world composed by hand. It reads
//! `attributes-town.chronotype` through a const handle (its `open-shop` is refused for a night owl) and
//! proposes `attributes-town.hair-colour` from its own `dyed` fact, which `attributes` accepts as
//! `trait-changed` citing that fact; a proposal for an attribute it did not list is refused by code.

use std::collections::BTreeSet;
use std::path::PathBuf;

use mineworld_attributes::AttributesSystem;
use mineworld_attributes_model::declaration::AttributesFile;
use mineworld_attributes_model::{
    ATTRIBUTES, AttributeChangeProposed, AttributeValue, AttributesComposed, Catalogue, DataFile,
    Handle, Level, LevelValue, QualifiedId, TraitChanged, Traits, TraitsStated, ValueSlug,
};
use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};
use mineworld_contracts::{
    Action, ActionId, ActionIntent, ActionRecord, ActionRequest, ActionResult, ActionTypeId,
    EntityId, EntityKey, EntityType, Event, EventEnvelope, EventSchemaVersion, EventTypeId,
    PersonId, Rejection, SystemId, Visibility, WorldTime,
};
use mineworld_kernel::{
    Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion, World,
    WorldRead, WorldView,
};
use serde::{Deserialize, Serialize};

const CATALOGUE: &str =
    include_str!("../../../tools/cli/tests/fixtures/attributes-town/attributes/test.yaml");
const VOCABULARY: &str = "format: 1\nvocabularies:\n  languages: [english, spanish, tagalog, vietnamese]\n";
const USES: &str = "format: 1\nuses:\n  reads: { attributes-town.chronotype: level }\n  proposes: [attributes-town.hair-colour]\n";

/// The reader's const handle (X-S30-10: a constant, never a string at run time).
const CHRONOTYPE: Handle<Level> = Handle::declared("attributes-town.chronotype");

/// The test-only reader and proposer.
#[derive(Debug, Default)]
struct EarlyBird;

impl SystemIdentity for EarlyBird {
    const ID: SystemId = SystemId::from_static("test-early-bird");
}

#[derive(Debug, Serialize, Deserialize)]
struct OpenShop;

impl Action for OpenShop {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("open-shop");
    const OWNER: SystemId = EarlyBird::ID;
}

#[derive(Debug, Serialize, Deserialize)]
struct Dye {
    colour: String,
}

impl Action for Dye {
    const ACTION_TYPE: ActionTypeId = ActionTypeId::from_static("dye");
    const OWNER: SystemId = EarlyBird::ID;
}

/// The reader's own fact: a person dyed their hair.
#[derive(Debug, Serialize, Deserialize)]
struct Dyed {
    person: PersonId,
    colour: String,
}

impl Event for Dyed {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("dyed");
    const OWNER: SystemId = EarlyBird::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The reader's own fact: the shop opened.
#[derive(Debug, Serialize, Deserialize)]
struct Opened {
    by: EntityId,
}

impl Event for Opened {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("shop-opened");
    const OWNER: SystemId = EarlyBird::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

impl System for EarlyBird {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .depending_on([ATTRIBUTES])
            .providing::<OpenShop>()
            .providing::<Dye>()
            .emitting::<Opened>()
            .emitting::<Dyed>()
            .emitting::<AttributeChangeProposed>()
            .subscribing_to::<Dyed>()
    }

    /// `open-shop` only for an early bird, read through the handle; `dye` by anyone.
    fn validate(&self, world: &WorldRead<'_>, intent: &ActionIntent) -> Result<(), Rejection> {
        if *intent.action_type() == OpenShop::ACTION_TYPE {
            let traits = world
                .component::<Traits>(intent.actor())
                .ok_or(Rejection::PreconditionFailed)?;
            let rank = traits.get(&CHRONOTYPE).map(|level| level.rank);
            return if rank == Some(0) {
                Ok(())
            } else {
                Err(Rejection::PreconditionFailed)
            };
        }
        // The owner's checked constructor (`ARC-26`): a proposal `attributes` would refuse is refused
        // here, as an ordinary rejection of this request.
        let dye: Dye = serde_json::from_slice(intent.payload().payload())
            .map_err(|_| Rejection::PreconditionFailed)?;
        let (attribute, to) = proposal(&dye.colour);
        mineworld_attributes_model::facts::admit_proposal(world, &Self::ID, &attribute, &to)
    }

    fn resolve(
        &self,
        _: &mut WorldView<'_, Self>,
        intent: &ActionIntent,
    ) -> Result<Vec<Emission>, KernelError> {
        let actor = intent.actor();
        if *intent.action_type() == OpenShop::ACTION_TYPE {
            return Ok(vec![Emission::new::<Opened>(
                serde_json::to_vec(&Opened { by: actor }).expect("encodes"),
                Visibility::Entities(BTreeSet::from([actor])),
            )]);
        }
        let dye: Dye = serde_json::from_slice(intent.payload().payload()).expect("a dye request");
        let person = PersonId::new(actor, EntityType::Person).expect("a person");
        Ok(vec![Emission::new::<Dyed>(
            serde_json::to_vec(&Dyed {
                person,
                colour: dye.colour,
            })
            .expect("encodes"),
            Visibility::Entities(BTreeSet::from([actor])),
        )])
    }

    /// Its own `dyed` → a proposal citing it.
    fn react(
        &self,
        _: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        let dyed: Dyed = serde_json::from_slice(event.payload().payload()).expect("a dyed fact");
        let (attribute, to) = proposal(&dyed.colour);
        let proposed = AttributeChangeProposed {
            person: dyed.person,
            attribute,
            to,
            because: event.id(),
        };
        Ok(vec![Emission::new::<AttributeChangeProposed>(
            serde_json::to_vec(&proposed).expect("encodes"),
            Visibility::SystemInternal,
        )])
    }
}

/// What dyeing to `colour` proposes: hair-colour (listed), or chronotype (not listed) for `night`.
fn proposal(colour: &str) -> (QualifiedId, AttributeValue) {
    if colour == "night" {
        (
            QualifiedId::new("attributes-town.chronotype").expect("an id"),
            AttributeValue::Level(LevelValue {
                slug: slug("night-owl"),
                rank: 2,
            }),
        )
    } else {
        (
            QualifiedId::new("attributes-town.hair-colour").expect("an id"),
            AttributeValue::Category(slug(colour)),
        )
    }
}

fn slug(text: &str) -> ValueSlug {
    ValueSlug::new(text).expect("a slug")
}

fn file(
    pack: &str,
    source: DataSource,
    directory: &'static str,
    text: &str,
) -> (DataOrigin, DataFile) {
    let directory = DataDirectory::from_static(directory);
    let origin = DataOrigin {
        pack: pack.to_owned(),
        source,
        directory,
        path: PathBuf::from(format!("{directory}/{pack}.yaml")),
    };
    let decoded = if directory.as_str() == "attributes" {
        DataFile::Attributes(serde_saphyr::from_str::<AttributesFile>(text).expect("decodes"))
    } else {
        DataFile::Vocabularies(serde_saphyr::from_str(text).expect("decodes"))
    };
    (origin, decoded)
}

/// Alice (brown hair, early bird) and Bob (night owl), with `attributes` and `test-early-bird`.
fn world() -> (World, EntityId, EntityId) {
    let catalogue = Catalogue::build(&[
        file(
            "test-early-bird",
            DataSource::SystemPack,
            "attributes",
            USES,
        ),
        file(
            "attributes-town",
            DataSource::World,
            "vocabularies",
            VOCABULARY,
        ),
        file(
            "attributes-town",
            DataSource::World,
            "attributes",
            CATALOGUE,
        ),
    ])
    .expect("the catalogue builds");
    let mut world = World::new();
    world.install(AttributesSystem).expect("attributes");
    world
        .install(EarlyBird)
        .expect("the reader depends on attributes");
    let alice = world
        .create_entity(EntityKey::new("alice").expect("a key"), EntityType::Person)
        .expect("alice");
    let bob = world
        .create_entity(EntityKey::new("bob").expect("a key"), EntityType::Person)
        .expect("bob");
    let stated = |person: EntityId, values: Vec<(&str, AttributeValue)>| {
        let stated = TraitsStated {
            person: PersonId::new(person, EntityType::Person).expect("a person"),
            values: values
                .into_iter()
                .map(|(id, value)| (QualifiedId::new(id).expect("an id"), value))
                .collect(),
        };
        Emission::new::<TraitsStated>(
            serde_json::to_vec(&stated).expect("encodes"),
            Visibility::Entities(BTreeSet::from([person])),
        )
    };
    let composed = AttributesComposed {
        schema: 1,
        catalogue: catalogue.projection(),
        population: None,
    };
    let facts = vec![
        Emission::new::<AttributesComposed>(
            serde_json::to_vec(&composed).expect("encodes"),
            Visibility::SystemInternal,
        ),
        stated(
            alice,
            vec![
                (
                    "attributes-town.chronotype",
                    AttributeValue::Level(LevelValue {
                        slug: slug("early-bird"),
                        rank: 0,
                    }),
                ),
                (
                    "attributes-town.hair-colour",
                    AttributeValue::Category(slug("brown")),
                ),
            ],
        ),
        stated(
            bob,
            vec![(
                "attributes-town.chronotype",
                AttributeValue::Level(LevelValue {
                    slug: slug("night-owl"),
                    rank: 2,
                }),
            )],
        ),
    ];
    world.genesis(WorldTime::EPOCH, facts).expect("genesis");
    (world, alice, bob)
}

fn dispatch<A: Action + Serialize>(
    world: &mut World,
    actor: EntityId,
    action: &A,
    id: u64,
) -> mineworld_kernel::Dispatched {
    let record = ActionRecord::new::<A>(serde_json::to_vec(action).expect("encodes"));
    let intent = ActionIntent::allocate(
        ActionRequest::new(actor, record),
        ActionId::from_raw(id),
        WorldTime::from_seconds(60),
    );
    world
        .dispatch(&intent, WorldTime::from_seconds(60))
        .expect("dispatch answers")
}

#[test]
fn a_read_through_a_const_handle_refuses_a_night_owl() {
    let (mut world, alice, bob) = world();
    assert!(matches!(
        dispatch(&mut world, alice, &OpenShop, 1).result(),
        ActionResult::Accepted { .. }
    ));
    assert!(!matches!(
        dispatch(&mut world, bob, &OpenShop, 2).result(),
        ActionResult::Accepted { .. }
    ));
}

#[test]
fn a_listed_proposal_changes_an_evolving_trait_citing_its_cause_and_an_unlisted_one_is_refused() {
    let (mut world, alice, _) = world();
    let dyed = dispatch(
        &mut world,
        alice,
        &Dye {
            colour: "auburn".to_owned(),
        },
        1,
    );
    assert!(
        matches!(dyed.result(), ActionResult::Accepted { .. }),
        "{:?}",
        dyed.result()
    );
    let cause = dyed
        .events()
        .iter()
        .find(|fact| *fact.event_type() == Dyed::EVENT_TYPE)
        .expect("dyed")
        .id();
    let changed = dyed
        .events()
        .iter()
        .find(|fact| *fact.event_type() == TraitChanged::EVENT_TYPE)
        .expect("trait-changed");
    let changed: TraitChanged =
        serde_json::from_slice(changed.payload().payload()).expect("decodes");
    assert_eq!(changed.from, Some(AttributeValue::Category(slug("brown"))));
    assert_eq!(changed.to, AttributeValue::Category(slug("auburn")));
    assert_eq!(changed.because, cause);
    let traits = world
        .read()
        .component::<Traits>(alice)
        .cloned()
        .expect("traits");
    assert_eq!(
        traits.value(&QualifiedId::new("attributes-town.hair-colour").expect("an id")),
        Some(&AttributeValue::Category(slug("auburn")))
    );

    let refused = dispatch(
        &mut world,
        alice,
        &Dye {
            colour: "night".to_owned(),
        },
        2,
    );
    let rendered = format!("{:?}", refused.result());
    assert!(
        !matches!(refused.result(), ActionResult::Accepted { .. }),
        "{rendered}"
    );
    assert!(rendered.contains("not-a-proposer"), "{rendered}");
}
