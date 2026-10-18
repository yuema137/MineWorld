//! `test-schema`: a test-only System Pack that uses every capability of the authoring seam's data half
//! (`docs/DECISIONS.md` `ARC-121`; S30-b §9 C6).
//!
//! ```text
//! data            owns schemas/: each file declares fields, { <name>: { min, max } }; the context is
//!                 every declared field (a field declared twice is refused naming the second file) and
//!                 the configured default
//! configuration   configure/test-schema.yaml: { default: <n> }, the value an absent field is filled with
//! section         schema: on a person file, { <name>: <n> } — decoded in the context, so an undeclared
//!                 field or a value outside its declared range is refused at the value's own line
//! resolution      once, over every person: each declared field authored or filled with the default; a
//!                 person with no schema: section is resolved too
//! seeding         schema-stated { subject, values }, reduced into the subject's SchemaValues
//! ships           schemas/shipped.yaml, compiled in from this crate's directory
//! ```
//!
//! It names no word of any installed pack's vocabulary: it exists to show the seam carries what a pack
//! declares in data, checks against it and resolves from it, and nothing else.

#![allow(dead_code)]

use std::any::Any;
use std::collections::BTreeMap;
use std::path::PathBuf;

use mineworld_authoring::{
    ConfigurationContext, ContentKind, DataDirectory, DataOrigin, DataRefusal, DataSource,
    OwnsData, PackConfiguration, ResolveRefusal, ResolvesSubjects, SectionInContext, SectionName,
    Seeding, Subject,
};
use mineworld_contracts::{
    EntityId, EntityKey, Event, EventEnvelope, EventSchemaVersion, EventTypeId, Rejection,
    SystemId, Visibility,
};
use mineworld_kernel::{
    Declarations, Emission, KernelError, System, SystemDeclaration, SystemIdentity, SystemVersion,
    WorldView, owned_component,
};
use mineworld_sdk::SystemPack;
use serde::de::{self, DeserializeSeed, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};

/// The pack.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SchemaPack;

impl SystemIdentity for SchemaPack {
    const ID: SystemId = SystemId::from_static("test-schema");
}

/// The directory it owns.
pub const SCHEMAS: DataDirectory = DataDirectory::from_static("schemas");

/// One field's declared range, inclusive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Range {
    pub min: i64,
    pub max: i64,
}

/// One `schemas/*.yaml` file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaFile {
    pub fields: BTreeMap<String, Range>,
}

/// Every declared field, the configured default, and where each file came from, in order.
#[derive(Debug)]
pub struct Schemas {
    pub fields: BTreeMap<String, Range>,
    pub default: i64,
    pub origins: Vec<(String, DataSource, PathBuf)>,
}

/// `configure/test-schema.yaml`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SchemaSettings {
    pub default: i64,
}

impl OwnsData for SchemaPack {
    const DIRECTORIES: &'static [DataDirectory] = &[SCHEMAS];
    type File = SchemaFile;
    type Context = Schemas;

    fn decode_file<'de, D: Deserializer<'de>>(
        _: DataDirectory,
        deserializer: D,
    ) -> Result<SchemaFile, D::Error> {
        SchemaFile::deserialize(deserializer)
    }

    fn context(
        files: Vec<(DataOrigin, SchemaFile)>,
        configuration: Option<&dyn Any>,
    ) -> Result<Schemas, DataRefusal> {
        let default = match configuration {
            None => 0,
            Some(configuration) => {
                configuration
                    .downcast_ref::<SchemaSettings>()
                    .ok_or_else(|| DataRefusal {
                        file: None,
                        reason: "not test-schema's configuration".to_owned(),
                    })?
                    .default
            }
        };
        let mut schemas = Schemas {
            fields: BTreeMap::new(),
            default,
            origins: Vec::new(),
        };
        for (origin, file) in files {
            for (field, range) in file.fields {
                if schemas.fields.insert(field.clone(), range).is_some() {
                    return Err(DataRefusal {
                        file: Some(origin.path),
                        reason: format!("the field '{field}' is declared twice"),
                    });
                }
            }
            schemas
                .origins
                .push((origin.pack, origin.source, origin.path));
        }
        Ok(schemas)
    }
}

impl PackConfiguration for SchemaPack {
    type Configuration = SchemaSettings;
    const FACTS: &'static [EventTypeId] = &[];

    fn seed(
        _: &Seeding<'_, '_>,
        _: &SchemaSettings,
        _: &ConfigurationContext<'_>,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

/// A person's section: one value per declared field.
pub type Values = BTreeMap<String, i64>;

/// Decodes `schema:` against the declared fields: a key the context does not declare is refused at the
/// key, a value outside its range at the value.
struct ValuesIn<'c>(&'c Schemas);

impl<'de> Visitor<'de> for ValuesIn<'_> {
    type Value = Values;

    fn expecting(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter.write_str("a map of declared fields to integers")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Values, A::Error> {
        let mut values = Values::new();
        while let Some(field) = map.next_key::<String>()? {
            let Some(range) = self.0.fields.get(&field) else {
                return Err(de::Error::custom(format!(
                    "'{field}' is not a field any schemas/ file declares"
                )));
            };
            let value = map.next_value_seed(Bounded {
                field: &field,
                range: *range,
            })?;
            values.insert(field, value);
        }
        Ok(values)
    }
}

/// One value, checked against its range while it is read, so the refusal keeps its position.
struct Bounded<'f> {
    field: &'f str,
    range: Range,
}

impl<'de> DeserializeSeed<'de> for Bounded<'_> {
    type Value = i64;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<i64, D::Error> {
        let value = i64::deserialize(deserializer)?;
        if value < self.range.min || value > self.range.max {
            return Err(de::Error::custom(format!(
                "'{}' is {} … {}, not {value}",
                self.field, self.range.min, self.range.max
            )));
        }
        Ok(value)
    }
}

impl SectionInContext for SchemaPack {
    const SECTION: SectionName = SectionName::from_static("schema");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
    type Authored = Values;

    fn decode<'de, D: Deserializer<'de>>(
        context: &Schemas,
        deserializer: D,
    ) -> Result<Values, D::Error> {
        deserializer.deserialize_map(ValuesIn(context))
    }

    /// Never called: the pack resolves, and its resolution seeds.
    fn seed(
        _: &Seeding<'_, '_>,
        _: EntityId,
        _: &Schemas,
        _: &Values,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

impl ResolvesSubjects for SchemaPack {
    type Seed = Values;

    fn resolve(
        context: &Schemas,
        _: Option<&dyn Any>,
        subjects: &[Subject<'_, Values>],
    ) -> Result<BTreeMap<EntityKey, Values>, ResolveRefusal> {
        Ok(subjects
            .iter()
            .map(|subject| {
                let mut values = subject.authored.cloned().unwrap_or_default();
                for field in context.fields.keys() {
                    values.entry(field.clone()).or_insert(context.default);
                }
                (subject.key.clone(), values)
            })
            .collect())
    }

    fn seed_resolved(
        _: &Seeding<'_, '_>,
        subject: EntityId,
        _: &Schemas,
        values: &Values,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(vec![stated(subject, values)])
    }
}

/// `schema-stated`: a subject's resolved values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaStated {
    pub subject: EntityId,
    pub values: Values,
}

impl Event for SchemaStated {
    const EVENT_TYPE: EventTypeId = EventTypeId::from_static("schema-stated");
    const OWNER: SystemId = SchemaPack::ID;
    const SCHEMA_VERSION: EventSchemaVersion = EventSchemaVersion::new(1);
}

/// The fact a subject's values become.
pub fn stated(subject: EntityId, values: &Values) -> Emission {
    let fact = SchemaStated {
        subject,
        values: values.clone(),
    };
    Emission::new::<SchemaStated>(
        serde_json::to_vec(&fact).expect("a fact encodes"),
        Visibility::SystemInternal,
    )
}

/// A subject's values, reduced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaValues {
    pub values: Values,
}

owned_component! {
    component = SchemaValues,
    owner = SchemaPack,
    component_type = "test-schema-values",
    schema_version = 1,
}

impl System for SchemaPack {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
            .owning::<SchemaValues>()
            .emitting::<SchemaStated>()
            .subscribing_to::<SchemaStated>()
    }

    fn install(&self, tables: &mut Declarations<'_, Self>) -> Result<(), KernelError> {
        tables.component::<SchemaValues>()
    }

    fn react(
        &self,
        world: &mut WorldView<'_, Self>,
        event: &EventEnvelope,
    ) -> Result<Vec<Emission>, KernelError> {
        if *event.event_type() == SchemaStated::EVENT_TYPE {
            let bytes = event.payload().payload_for::<SchemaStated>()?;
            let fact: SchemaStated = serde_json::from_slice(bytes).map_err(|_| {
                KernelError::Contract(mineworld_contracts::ContractError::EventTypeMismatch {
                    expected: SchemaStated::EVENT_TYPE,
                    actual: event.event_type().clone(),
                })
            })?;
            world.insert(
                fact.subject,
                SchemaValues {
                    values: fact.values,
                },
            )?;
        }
        Ok(Vec::new())
    }
}

impl SystemPack for SchemaPack {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::configures!();
    mineworld_sdk::owns_data!();
    mineworld_sdk::owns_section_in_context!();
    mineworld_sdk::resolves!();
    mineworld_sdk::ships_data!("schemas/shipped.yaml");
}
