//! The seam's contracts with a probe owner: data decoded and gathered into one context, a section decoded
//! against it, and a resolution over every subject. Driven through serde's own value deserializers, as
//! `configuration.rs`' tests are; the YAML stream's positions are the acceptance test's (`data_seam.rs`).

use std::collections::BTreeMap;
use std::panic::catch_unwind;

use mineworld_contracts::{
    EntityKey, Event, EventSchemaVersion, EventTypeId, Rejection, SystemId, Tag, Tags, Visibility,
};
use mineworld_kernel::{Emission, SystemIdentity, World};
use serde::de::IntoDeserializer;
use serde::de::value::{Error, MapDeserializer};
use serde::{Deserialize, Serialize};

use super::*;
use crate::{ConfigurationContext, DecodeConfiguration, PackConfiguration, Seeding};

/// A pack owning `ranges/`: each file declares fields and the largest value each may take; its
/// configuration is the value a resolution fills an absent field with.
struct Probe;

impl SystemIdentity for Probe {
    const ID: SystemId = SystemId::from_static("probe");
}

const RANGES: DataDirectory = DataDirectory::from_static("ranges");

/// Every declared field's largest value, the fill, and the files it came from, in order.
#[derive(Debug)]
struct Ranges {
    max: BTreeMap<String, u32>,
    fill: u32,
    files: Vec<PathBuf>,
}

impl OwnsData for Probe {
    const DIRECTORIES: &'static [DataDirectory] = &[RANGES];
    type File = BTreeMap<String, u32>;
    type Context = Ranges;

    fn decode_file<'de, D: Deserializer<'de>>(
        _: DataDirectory,
        deserializer: D,
    ) -> Result<Self::File, D::Error> {
        BTreeMap::deserialize(deserializer)
    }

    fn context(
        files: Vec<(DataOrigin, Self::File)>,
        configuration: Option<&dyn Any>,
    ) -> Result<Ranges, DataRefusal> {
        let mut ranges = Ranges {
            max: BTreeMap::new(),
            fill: configuration
                .and_then(|configuration| configuration.downcast_ref::<u32>())
                .copied()
                .unwrap_or(0),
            files: Vec::new(),
        };
        for (origin, file) in files {
            for (field, max) in file {
                if ranges.max.insert(field.clone(), max).is_some() {
                    return Err(DataRefusal {
                        file: Some(origin.path),
                        reason: format!("'{field}' is declared twice"),
                    });
                }
            }
            ranges.files.push(origin.path);
        }
        Ok(ranges)
    }

    const CONTEXT_FACTS: &'static [EventTypeId] = &[Declared::EVENT_TYPE];

    /// How many fields the data declared, and the fill: what a running world would need to know.
    fn seed_context(_: &Seeding<'_, '_>, context: &Ranges) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![declared(context.max.len(), context.fill)])
    }
}

/// The probe's world-level fact (`ARC-121` item 5): how many fields its data declared, and the fill.
#[derive(Debug, Serialize, Deserialize)]
struct Declared;

impl Event for Declared {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("declared");
    const OWNER: SystemId = Probe::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn declared(fields: usize, fill: u32) -> Emission {
    Emission::new::<Declared>(
        format!("{fields}:{fill}").into_bytes(),
        Visibility::SystemInternal,
    )
}

/// A data owner that declares nothing of item 5: it states nothing from its context.
struct Quiet;

impl SystemIdentity for Quiet {
    const ID: SystemId = SystemId::from_static("quiet");
}

impl OwnsData for Quiet {
    const DIRECTORIES: &'static [DataDirectory] = &[DataDirectory::from_static("quiet")];
    type File = ();
    type Context = ();

    fn decode_file<'de, D: Deserializer<'de>>(
        _: DataDirectory,
        deserializer: D,
    ) -> Result<(), D::Error> {
        <()>::deserialize(deserializer)
    }

    fn context(_: Vec<(DataOrigin, ())>, _: Option<&dyn Any>) -> Result<(), DataRefusal> {
        Ok(())
    }
}

impl PackConfiguration for Probe {
    type Configuration = u32;
    const FACTS: &'static [EventTypeId] = &[];

    fn seed(
        _: &Seeding<'_, '_>,
        _: &u32,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

/// The probe's one fact: a subject, and a number its seeding computed.
#[derive(Debug, Serialize, Deserialize)]
struct Stated;

impl Event for Stated {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("stated");
    const OWNER: SystemId = Probe::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

fn stated(subject: EntityId, number: u32) -> Emission {
    Emission::new::<Stated>(
        format!("{}:{number}", subject.raw()).into_bytes(),
        Visibility::SystemInternal,
    )
}

impl SectionInContext for Probe {
    const SECTION: SectionName = SectionName::from_static("probed");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
    type Authored = BTreeMap<String, u32>;

    fn decode<'de, D: Deserializer<'de>>(
        context: &Ranges,
        deserializer: D,
    ) -> Result<Self::Authored, D::Error> {
        let values = BTreeMap::<String, u32>::deserialize(deserializer)?;
        for (field, value) in &values {
            match context.max.get(field) {
                None => return Err(D::Error::custom(format!("'{field}' is not declared"))),
                Some(max) if value > max => {
                    return Err(D::Error::custom(format!(
                        "'{field}' is at most {max}, not {value}"
                    )));
                }
                Some(_) => {}
            }
        }
        Ok(values)
    }

    /// The fill, so a test sees the context reached seeding.
    fn seed(
        _: &Seeding<'_, '_>,
        subject: EntityId,
        context: &Ranges,
        _: &Self::Authored,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![stated(subject, context.fill)])
    }
}

impl ResolvesSubjects for Probe {
    type Seed = BTreeMap<String, u32>;

    /// Every declared field, authored or filled; a subject tagged `invent` makes it seed a stranger.
    fn resolve_subjects(
        context: &Ranges,
        _: Option<&dyn Any>,
        subjects: &[Subject<'_, Self::Authored>],
    ) -> Result<BTreeMap<EntityKey, Self::Seed>, ResolveRefusal> {
        let mut seeds = BTreeMap::new();
        for subject in subjects {
            let mut values = subject.authored.cloned().unwrap_or_default();
            for field in context.max.keys() {
                values.entry(field.clone()).or_insert(context.fill);
            }
            seeds.insert(subject.key.clone(), values);
            if subject.tags.contains(&Tag::new("invent").expect("a tag")) {
                seeds.insert(key("nobody"), BTreeMap::new());
            }
        }
        Ok(seeds)
    }

    /// The seed's sum.
    fn seed_resolved(
        _: &Seeding<'_, '_>,
        subject: EntityId,
        _: &Ranges,
        seed: &Self::Seed,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![stated(subject, seed.values().sum())])
    }
}

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a key")
}

/// `ranges/<name>.yaml` declaring `fields`, decoded and erased as the loader holds it.
fn file(name: &str, fields: &[(&'static str, u32)]) -> (DataOrigin, Box<dyn Any + Send + Sync>) {
    let decoded = decode_data_file::<Probe, _>(
        RANGES,
        MapDeserializer::<_, Error>::new(fields.iter().copied()),
    )
    .expect("a ranges file decodes");
    let origin = DataOrigin {
        pack: "a-world".to_owned(),
        source: DataSource::World,
        directory: RANGES,
        path: PathBuf::from(format!("ranges/{name}.yaml")),
    };
    (origin, decoded)
}

/// The probe's context from `a.yaml` (height ≤ 10) and `b.yaml` (width ≤ 3), configured to fill 7.
fn context() -> Arc<dyn OwnerContext> {
    let configuration = DecodeConfiguration::<Probe>::new()
        .deserialize(7_u32.into_deserializer())
        .map_err(|error: Error| error)
        .expect("the configuration decodes");
    data_context::<Probe>(
        vec![file("a", &[("height", 10)]), file("b", &[("width", 3)])],
        Some(&*configuration),
    )
    .expect("the context builds")
}

fn decode_in(
    context: &dyn OwnerContext,
    values: &[(&'static str, u32)],
) -> Result<Arc<dyn AuthoredContent>, Error> {
    DecodeIn::<Probe>::from_erased(context)
        .expect("the probe's own context")
        .deserialize(MapDeserializer::<_, Error>::new(values.iter().copied()))
}

#[test]
fn a_data_directory_is_one_word_and_none_of_the_formats() {
    assert_eq!(DataDirectory::from_static("ranges").as_str(), "ranges");
    for refused in ["people", "configure", "data", "", "Upper", "a/b", "a_b"] {
        assert!(
            catch_unwind(|| DataDirectory::from_static(refused)).is_err(),
            "'{refused}' is refused"
        );
    }
}

/// M-b-1's test: the context a section decodes in is the one built from every file and the
/// configuration — a field of the second file is admitted, a value over the first file's bound is
/// refused with the owner's words, and seeding sees the configured fill.
#[test]
fn decode_in_context_sees_the_context() {
    let context = context();
    assert_eq!(context.owner(), Probe::ID);
    let ranges = context
        .as_any()
        .downcast_ref::<Ranges>()
        .expect("the owner's own context");
    assert_eq!(
        ranges.files,
        [
            PathBuf::from("ranges/a.yaml"),
            PathBuf::from("ranges/b.yaml")
        ],
        "files reach the owner in the order given"
    );

    let refusal = decode_in(&*context, &[("height", 11)]).expect_err("over the declared bound");
    assert_eq!(refusal.to_string(), "'height' is at most 10, not 11");
    let refusal = decode_in(&*context, &[("depth", 1)]).expect_err("not declared");
    assert_eq!(refusal.to_string(), "'depth' is not declared");

    let decoded = decode_in(&*context, &[("height", 10), ("width", 3)]).expect("within both files");
    assert_eq!(decoded.owner(), Probe::ID);
    assert_eq!(decoded.section(), Probe::SECTION);
    assert!(decoded.references().is_empty());
    assert_eq!(
        decoded.as_any().downcast_ref::<BTreeMap<String, u32>>(),
        Some(&BTreeMap::from([
            ("height".to_owned(), 10),
            ("width".to_owned(), 3)
        ]))
    );

    let world = World::new();
    let read = world.read();
    let keys = BTreeMap::new();
    let subject = EntityId::from_raw(4);
    assert_eq!(
        decoded.seed(&Seeding::new(&read, &keys), subject),
        Ok(vec![stated(subject, 7)]),
        "seeding reaches the owner with its context"
    );
}

#[test]
fn a_context_refusal_is_the_owners_and_names_the_file() {
    let refusal = data_context::<Probe>(
        vec![file("a", &[("height", 10)]), file("b", &[("height", 3)])],
        None,
    )
    .expect_err("height twice");
    assert_eq!(
        refusal,
        DataRefusal {
            file: Some(PathBuf::from("ranges/b.yaml")),
            reason: "'height' is declared twice".to_owned(),
        }
    );

    let refusal = decode_data_file::<Probe, _>(
        DataDirectory::from_static("elsewhere"),
        MapDeserializer::<_, Error>::new([("height", 1_u32)].into_iter()),
    )
    .expect_err("not the probe's directory");
    assert_eq!(
        refusal.to_string(),
        "the 'probe' system does not own the directory 'elsewhere'"
    );
}

/// A resolution sees every subject, with or without the section, and seeds only those it named; a seed
/// for a key it was not given is refused naming the key.
#[test]
fn a_resolution_covers_every_subject_and_names_no_stranger() {
    let context = context();
    let section = decode_in(&*context, &[("height", 4)]).expect("decodes");
    let (ada, bo, tags) = (key("ada"), key("bo"), Tags::default());
    let subjects = [
        ErasedSubject {
            key: &ada,
            kind: ContentKind::Person,
            tags: &tags,
            authored: Some(&*section),
        },
        ErasedSubject {
            key: &bo,
            kind: ContentKind::Person,
            tags: &tags,
            authored: None,
        },
    ];
    let resolved = resolve_erased::<Probe>(&*context, None, &subjects).expect("resolves");
    assert_eq!(resolved.owner(), Probe::ID);
    assert_eq!(resolved.section(), Probe::SECTION);

    let world = World::new();
    let read = world.read();
    let keys = BTreeMap::new();
    let seeding = Seeding::new(&read, &keys);
    let id = EntityId::from_raw(9);
    // ada: height 4 authored, width filled with 7; bo: both filled.
    assert_eq!(
        resolved.seed(&seeding, &ada, id),
        Some(Ok(vec![stated(id, 11)]))
    );
    assert_eq!(
        resolved.seed(&seeding, &bo, id),
        Some(Ok(vec![stated(id, 14)]))
    );
    assert_eq!(
        resolved.seed(&seeding, &key("cy"), id),
        None,
        "not a subject"
    );

    let inventive = Tags::new([Tag::new("invent").expect("a tag")]);
    let subjects = [ErasedSubject {
        key: &ada,
        kind: ContentKind::Person,
        tags: &inventive,
        authored: None,
    }];
    let refusal = resolve_erased::<Probe>(&*context, None, &subjects).expect_err("a stranger");
    assert_eq!(refusal.subject, Some(key("nobody")));
    assert_eq!(
        refusal.reason,
        "'nobody' is not one of the subjects the 'probe' system was given"
    );
}

/// Item 5 (S30-c): the erased context states its owner's world-level facts from the context the owner
/// built, and declares the owner's context facts; an owner that declares neither states nothing.
#[test]
fn a_context_states_its_owners_world_level_facts() {
    let world = World::new();
    let read = world.read();
    let keys = BTreeMap::new();
    let seeding = Seeding::new(&read, &keys);

    let context = context();
    assert_eq!(context.context_facts(), [Declared::EVENT_TYPE]);
    // Two fields declared (height, width), filled with 7.
    assert_eq!(context.seed_context(&seeding), Ok(vec![declared(2, 7)]));

    let quiet = data_context::<Quiet>(Vec::new(), None).expect("a context from no files");
    assert!(quiet.context_facts().is_empty());
    assert_eq!(quiet.seed_context(&seeding), Ok(Vec::new()));
}

#[test]
fn a_shipped_file_lies_directly_in_an_owners_directory() {
    let shipped = ShippedData::new("ranges/base.yaml", "height: 2\n");
    assert_eq!(shipped.directory(), RANGES);
    assert_eq!(shipped.file(), "base.yaml");
    assert_eq!(shipped.path(), "ranges/base.yaml");
    assert_eq!(shipped.text(), "height: 2\n");
    for refused in [
        "ranges.yaml",
        "ranges/inner/base.yaml",
        "ranges/base.yml",
        "ranges/.yaml",
        "people/base.yaml",
    ] {
        assert!(
            catch_unwind(|| ShippedData::new(refused, "")).is_err(),
            "'{refused}' is refused"
        );
    }
}
