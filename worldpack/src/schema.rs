//! The JSON Schemas of the files the World Pack format itself owns (`docs/DECISIONS.md` `ARC-90`):
//! `world.yaml`, the format's own fields of each kind of content file, `configure/classes.yaml` and
//! `configure/packages.yaml`. The sections and configurations System Packs own are each pack's, asked
//! through the installed set ([`Capability`](crate::Capability)).
//!
//! A content file has no derived `Deserialize` — which keys it may hold depends on the build's packs
//! ([`crate::content`]) — so its framework fields are described by a mirror struct per kind, and a test
//! holds each mirror's keys equal to the list [`crate::content::fields`] reads by.

use mineworld_authoring::schema::TagList;
use mineworld_authoring::{ContentKind, EntityClasses};
use schemars::{JsonSchema, Schema, SchemaGenerator};

use crate::configure::StatedPolicy;
use crate::format::{AuthoredLocation, AuthoredPassage, WorldManifest};

/// The format's own fields of a person file.
#[derive(JsonSchema)]
#[schemars(
    title = "Person file",
    description = "people/<key>.yaml: one Person. Besides these fields, the file may carry the \
                   sections of the System Packs the world enables (see sections.md); any other key is \
                   refused.",
    example = serde_json::json!({ "tags": ["shopkeeper"], "location": { "place": "shop" } })
)]
#[allow(dead_code)]
struct PersonFields {
    /// Labels this person carries, such as shopkeeper.
    #[schemars(with = "Option<TagList>")]
    tags: (),
    /// A note from whoever wrote the file, for a person reading it. No system reads it.
    note: Option<String>,
    /// Where this person starts: a place, and optionally where in it and which way they face. Needs the
    /// presence system.
    location: Option<AuthoredLocation>,
}

/// The format's own fields of a place file.
#[derive(JsonSchema)]
#[schemars(
    title = "Place file",
    description = "places/<key>.yaml: one Place. Besides these fields, the file may carry the \
                   sections of the System Packs the world enables (see sections.md); any other key is \
                   refused.",
    example = serde_json::json!({ "tags": ["shop"], "passages": [{ "to": "home" }] })
)]
#[allow(dead_code)]
struct PlaceFields {
    /// Labels this place carries, such as shop. Clients choose how to draw a place from its tags.
    #[schemars(with = "Option<TagList>")]
    tags: (),
    /// A note from whoever wrote the file, for a person reading it. No system reads it.
    note: Option<String>,
    /// The places this one opens onto, each through one doorway. Needs the movement system.
    passages: Option<Vec<AuthoredPassage>>,
}

/// The format's own fields of an item file.
#[derive(JsonSchema)]
#[schemars(
    title = "Item file",
    description = "items/<key>.yaml: one kind of item, such as bread — not one object. Besides these \
                   fields, the file may carry the sections of the System Packs the world enables (see \
                   sections.md); any other key is refused.",
    example = serde_json::json!({ "tags": ["food"] })
)]
#[allow(dead_code)]
struct ItemFields {
    /// Labels this kind carries, such as food.
    #[schemars(with = "Option<TagList>")]
    tags: (),
    /// A note from whoever wrote the file, for a person reading it. No system reads it.
    note: Option<String>,
}

/// The format's own fields of an organization file.
#[derive(JsonSchema)]
#[schemars(
    title = "Organization file",
    description = "organizations/<key>.yaml: one Organization, such as a company that runs a shop. \
                   Besides these fields, the file may carry the sections of the System Packs the world \
                   enables (see sections.md); any other key is refused.",
    example = serde_json::json!({ "tags": ["company"] })
)]
#[allow(dead_code)]
struct OrganizationFields {
    /// Labels this organization carries, such as company.
    #[schemars(with = "Option<TagList>")]
    tags: (),
    /// A note from whoever wrote the file, for a person reading it. No system reads it.
    note: Option<String>,
}

/// The JSON Schema of `world.yaml`.
#[must_use]
pub fn world_manifest() -> Schema {
    SchemaGenerator::default().into_root_schema_for::<WorldManifest>()
}

/// The JSON Schema of the format's own fields of one kind of content file.
#[must_use]
pub fn content_file(kind: ContentKind) -> Schema {
    let generator = SchemaGenerator::default();
    match kind {
        ContentKind::Person => generator.into_root_schema_for::<PersonFields>(),
        ContentKind::Place => generator.into_root_schema_for::<PlaceFields>(),
        ContentKind::Item => generator.into_root_schema_for::<ItemFields>(),
        ContentKind::Organization => generator.into_root_schema_for::<OrganizationFields>(),
    }
}

/// The JSON Schema of `configure/classes.yaml`, the world's entity classes (`ARC-64`).
#[must_use]
pub fn classes() -> Schema {
    SchemaGenerator::default().into_root_schema_for::<EntityClasses>()
}

/// The JSON Schema of `configure/packages.yaml`, the world's licence policy (`ARC-55` note).
#[must_use]
pub fn packages() -> Schema {
    SchemaGenerator::default().into_root_schema_for::<StatedPolicy>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content::fields;

    /// Each kind's schema lists exactly the format's own fields the reader accepts for that kind, and
    /// requires none of them (every one is optional to the reader).
    #[test]
    fn each_content_files_schema_lists_exactly_the_fields_its_reader_accepts() {
        for kind in ContentKind::ALL {
            let schema = content_file(kind);
            let mut listed: Vec<&str> = schema
                .get("properties")
                .and_then(|properties| properties.as_object())
                .expect("an object schema")
                .keys()
                .map(String::as_str)
                .collect();
            listed.sort_unstable();
            let mut read: Vec<&str> = fields(kind).to_vec();
            read.sort_unstable();
            assert_eq!(listed, read, "{kind}");
            assert!(
                schema.get("required").is_none(),
                "{kind}: nothing is required"
            );
        }
    }
}
