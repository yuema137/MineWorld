//! The JSON Schemas of a package identity's parts, stated for an author (`docs/DECISIONS.md`
//! `ARC-90`, `DEP-49`). Each is a string the type parses on decoding; the description says what parses.

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

use crate::identity::{Compatibility, License, PackId, PackType, Version};

impl JsonSchema for PackId {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("PackId")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "pattern": "^[a-z]([a-z0-9]|-[a-z0-9])*$",
            "minLength": 1,
            "maxLength": 64,
            "description": "A pack's id, such as modern-goods: 1 to 64 of a-z, 0-9 and -, beginning \
                            with a letter, with no -- and no trailing -.",
        })
    }
}

impl JsonSchema for PackType {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("PackType")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "enum": [
                PackType::SystemPack.as_str(),
                PackType::ControllerPack.as_str(),
                PackType::WorldPack.as_str(),
                PackType::PresentationPack.as_str(),
                PackType::EntityPack.as_str(),
            ],
            "description": "What kind of pack this is. A pack.yaml identifies a presentation-pack or \
                            an entity-pack; the other kinds have their own carriers.",
        })
    }
}

impl JsonSchema for Version {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Version")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "A semver version, such as 0.1.0.",
        })
    }
}

impl JsonSchema for Compatibility {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Compatibility")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "A semver range with Cargo's meaning, such as \"^0.1\" (0.1.x and nothing \
                            else). Quote it in YAML.",
        })
    }
}

impl JsonSchema for License {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("License")
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        json_schema!({
            "type": "string",
            "description": "An SPDX licence expression, such as MIT or MIT OR Apache-2.0, checked \
                            against the SPDX licence list.",
        })
    }
}
