//! The data seam, end to end with a test-only pack (`docs/DECISIONS.md` `ARC-121`; S30-b's integration
//! checkpoint, AC-b-1, AC-b-2, AC-b-7, AC-b-8).
//!
//! `test-schema` (`schema/mod.rs`) is driven through exactly the calls the World Pack loader makes — every
//! file straight from its YAML stream through `serde_saphyr::with_deserializer_from_str`, then
//! `SystemPack::decode_data_file`, `data_context`, `decode_section_in` and `resolve_subjects` — and its
//! seeds through `World::genesis`, because the loader's `Capability` is closed and a test-only pack cannot
//! join it (as `configuration_seam.rs` does for `ARC-61`). The order the loader collects files in, the
//! directory rule and where resolved facts sit among other sections are held by worldpack's in-crate
//! tests (`worldpack/src/data/tests.rs`, `worldpack/src/load.rs`).

mod schema;

use std::any::Any;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::Arc;

use mineworld_authoring::{
    AuthoredConfiguration, AuthoredContent, ContentKind, DataOrigin, DataSource, ErasedSubject,
    OwnerContext, ResolvedSeeds, Seeding,
};
use mineworld_contracts::{EntityId, EntityKey, EntityType, Tags, WorldTime};
use mineworld_kernel::{SystemIdentity, World};
use mineworld_sdk::SystemPack;
use schema::{SCHEMAS, SchemaPack, SchemaValues, Schemas, Values};
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};

const WORLD_SCHEMA: &str = "fields:\n  mood: { min: 1, max: 9 }\n";
const CONFIGURED: &str = "default: 4\n";

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a key")
}

/// Every data file test-schema is handed, in the loader's order: what it ships, then the world's
/// `schemas/body.yaml` — each decoded by the pack, straight from its stream.
fn files(world_schema: &str) -> Vec<(DataOrigin, Box<dyn Any + Send + Sync>)> {
    let decode = |text: &str| {
        serde_saphyr::with_deserializer_from_str(text, |file| {
            SchemaPack::decode_data_file(SCHEMAS, file)
        })
        .expect("a schemas/ file decodes")
    };
    let mut files = Vec::new();
    for shipped in SchemaPack::SHIPS {
        files.push((
            DataOrigin {
                pack: SchemaPack::ID.as_str().to_owned(),
                source: DataSource::SystemPack,
                directory: shipped.directory(),
                path: PathBuf::from(shipped.path()),
            },
            decode(shipped.text()),
        ));
    }
    files.push((
        DataOrigin {
            pack: "a-world".to_owned(),
            source: DataSource::World,
            directory: SCHEMAS,
            path: PathBuf::from("schemas/body.yaml"),
        },
        decode(world_schema),
    ));
    files
}

/// `configure/test-schema.yaml`, decoded by the pack.
fn configuration() -> Arc<dyn AuthoredConfiguration> {
    serde_saphyr::with_deserializer_from_str(CONFIGURED, |file| {
        SchemaPack::decode_configuration(file)
    })
    .expect("the configuration decodes")
}

/// test-schema's context, as read step 4d builds it.
fn context() -> Arc<dyn OwnerContext> {
    let configuration = configuration();
    SchemaPack::data_context(files(WORLD_SCHEMA), Some(&*configuration))
        .expect("the context builds")
}

/// A person file holding only (optionally) a `schema:` section, read as the loader's `ContentFile`
/// reads a section: the key, then the owner's `decode_section_in` with its context.
struct PersonFile<'c>(&'c dyn OwnerContext);

impl<'de> DeserializeSeed<'de> for PersonFile<'_> {
    type Value = Option<Arc<dyn AuthoredContent>>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de> Visitor<'de> for PersonFile<'_> {
    type Value = Option<Arc<dyn AuthoredContent>>;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a person file")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(None)
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        let mut section = None;
        while let Some(field) = map.next_key::<String>()? {
            if field != "schema" {
                return Err(de::Error::custom(format!("unknown field `{field}`")));
            }
            section = Some(SchemaPack::decode_section_in(&mut map, Some(self.0))?);
        }
        Ok(section)
    }
}

fn person(
    context: &dyn OwnerContext,
    text: &str,
) -> Result<Option<Arc<dyn AuthoredContent>>, String> {
    serde_saphyr::with_deserializer_from_str(text, |file| PersonFile(context).deserialize(file))
        .map_err(|error| error.to_string())
}

/// AC-b-1: a person-file value outside the range a `schemas/*.yaml` file declares is refused with the
/// line and column of the value and the owner's message naming the range; an undeclared field is
/// refused naming it; a value inside the range is taken.
#[test]
fn a_person_file_value_is_checked_at_its_own_line_against_a_data_declared_schema() {
    let context = context();
    let refusal = person(
        &*context,
        "# people/ada.yaml\nschema:\n  patience: 2\n  mood: 12\n",
    )
    .expect_err("12 is outside mood's 1 … 9");
    assert!(
        refusal.contains("'mood' is 1 … 9, not 12"),
        "the owner's words: {refusal}"
    );
    assert!(
        refusal.contains("line 4") && refusal.contains("column 9"),
        "the value's own position: {refusal}"
    );

    let refusal = person(&*context, "schema:\n  height: 3\n").expect_err("undeclared");
    assert!(
        refusal.contains("'height' is not a field any schemas/ file declares")
            && refusal.contains("line 2"),
        "{refusal}"
    );

    let taken = person(&*context, "schema:\n  mood: 9\n")
        .expect("within the range")
        .expect("a section");
    assert_eq!(taken.owner(), SchemaPack::ID);
    assert_eq!(
        taken.as_any().downcast_ref::<Values>(),
        Some(&BTreeMap::from([("mood".to_owned(), 9)]))
    );
}

/// AC-b-2: three people — one with no `schema:` section — are resolved once; each is seeded with its
/// values, authored or the configured default; genesis reduces them, in the people's key order.
#[test]
fn every_person_is_resolved_once_and_seeded_with_authored_or_default_values() {
    let context = context();
    let configuration = configuration();
    let sections: Vec<(EntityKey, Option<Arc<dyn AuthoredContent>>)> = [
        ("ada", "schema:\n  mood: 2\n"),
        ("bo", "schema:\n  patience: 5\n  mood: 7\n"),
        ("cy", ""),
    ]
    .into_iter()
    .map(|(name, text)| (key(name), person(&*context, text).expect("reads")))
    .collect();
    let tags = Tags::default();
    let subjects: Vec<ErasedSubject<'_>> = sections
        .iter()
        .map(|(key, section)| ErasedSubject {
            key,
            kind: ContentKind::Person,
            tags: &tags,
            authored: section.as_deref(),
        })
        .collect();
    let resolved: Arc<dyn ResolvedSeeds> =
        SchemaPack::resolve_subjects(Some(&*context), Some(&*configuration), &subjects)
            .expect("resolves")
            .expect("test-schema resolves");

    let mut world = World::new();
    world.install(SchemaPack).expect("installs");
    let mut ids: BTreeMap<EntityKey, EntityId> = BTreeMap::new();
    for (key, _) in &sections {
        let id = world
            .create_entity(key.clone(), EntityType::Person)
            .expect("a person");
        ids.insert(key.clone(), id);
    }
    // Genesis's sections pass: people in key order, each its resolved seed.
    let facts: Vec<_> = {
        let read = world.read();
        let seeding = Seeding::new(&read, &ids);
        sections
            .iter()
            .flat_map(|(key, _)| {
                resolved
                    .seed(&seeding, key, ids[key])
                    .expect("every subject has a seed")
                    .expect("seeds")
            })
            .collect()
    };
    let values = |pairs: &[(&str, i64)]| -> Values {
        pairs
            .iter()
            .map(|(field, value)| ((*field).to_owned(), *value))
            .collect()
    };
    let expected = [
        values(&[("mood", 2), ("patience", 4)]),
        values(&[("mood", 7), ("patience", 5)]),
        values(&[("mood", 4), ("patience", 4)]),
    ];
    assert_eq!(
        facts,
        sections
            .iter()
            .zip(&expected)
            .map(|((key, _), values)| schema::stated(ids[key], values))
            .collect::<Vec<_>>(),
        "one fact per person, in key order"
    );

    world
        .genesis(WorldTime::EPOCH, facts)
        .expect("genesis reduces them");
    let read = world.read();
    for ((key, _), values) in sections.iter().zip(expected) {
        assert_eq!(
            read.component::<SchemaValues>(ids[key]),
            Some(&SchemaValues { values }),
            "{key}"
        );
    }
}

/// AC-b-8: the shipped file is compiled in from this crate's directory and reaches the owner first, its
/// origin naming the shipping pack and its crate-relative path; a field the world declares again is
/// refused naming the world's file.
#[test]
fn shipped_data_reaches_the_owner_first_naming_its_pack_and_path() {
    let context = context();
    let schemas = context
        .as_any()
        .downcast_ref::<Schemas>()
        .expect("test-schema's own context");
    assert_eq!(
        schemas.origins,
        [
            (
                "test-schema".to_owned(),
                DataSource::SystemPack,
                PathBuf::from("schemas/shipped.yaml")
            ),
            (
                "a-world".to_owned(),
                DataSource::World,
                PathBuf::from("schemas/body.yaml")
            ),
        ]
    );
    assert_eq!(
        schemas.fields.len(),
        2,
        "patience (shipped) and mood (the world's)"
    );
    assert_eq!(schemas.default, 4, "the configuration reached the context");

    let refusal =
        SchemaPack::data_context(files("fields:\n  patience: { min: 0, max: 1 }\n"), None)
            .expect_err("patience twice");
    assert_eq!(refusal.file, Some(PathBuf::from("schemas/body.yaml")));
    assert!(refusal.reason.contains("'patience'"), "{}", refusal.reason);
}

/// AC-b-7's package half: an Entity Pack of `pack.yaml` and `schemas/a.yaml` only declares something.
/// (Required by a world, its files are collected after the shipped ones and before the world's:
/// `worldpack/src/data/tests.rs` `data_is_collected_in_composition_order`.)
#[test]
fn an_entity_pack_of_data_only_passes_the_package_check() {
    let scratch = mineworld_test_support::scratch!(empty "data-seam-entity").within("schemas-only");
    let write = |relative: &str, text: &str| {
        let path = relative
            .split('/')
            .fold(scratch.path().to_path_buf(), |path, part| path.join(part));
        std::fs::create_dir_all(path.parent().expect("a parent")).expect("writable");
        std::fs::write(path, text).expect("writable");
    };
    write(
        "pack.yaml",
        "id: schemas-only\ntype: entity-pack\nversion: 0.1.0\nmineworld: \"^0.1\"\nlicense: MIT\n\
         authors: [Someone]\n",
    );
    write("schemas/a.yaml", WORLD_SCHEMA);
    mineworld_packages::check_entity_layout(scratch.path())
        .expect("a data file declares something");
}
