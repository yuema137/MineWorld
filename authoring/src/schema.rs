//! The JSON Schemas of the contract types an authored field may have (`docs/DECISIONS.md` `ARC-90`,
//! `DEP-49`).
//!
//! `mineworld-contracts` does not depend on `schemars`: it is the most depended-on crate and has no use
//! for it. A field of a contract type names its schema here instead, with
//! `#[schemars(with = "mineworld_authoring::schema::Key")]` and the like. Each type below is
//! uninhabited — it is never a value, only the name of a schema — and states the contract's own rule
//! (`contracts/src/ids.rs`, the identifier rule; `contracts/src/spatial.rs`, millimetres).

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};

/// The identifier rule of `mineworld-contracts`: 1 to 64 bytes of `a-z`, `0-9`, `-` and `_`, with no `-`
/// or `_` at either end.
pub const IDENTIFIER_PATTERN: &str = "^[a-z0-9]([a-z0-9_-]*[a-z0-9])?$";

/// The longest identifier, in bytes (`mineworld_contracts::MAX_IDENTIFIER_LENGTH`).
const IDENTIFIER_MAX: usize = 64;

/// A string schema under the identifier rule, described for an author.
fn identifier(description: &str) -> Schema {
    json_schema!({
        "type": "string",
        "pattern": IDENTIFIER_PATTERN,
        "minLength": 1,
        "maxLength": IDENTIFIER_MAX,
        "description": description,
    })
}

macro_rules! schema_type {
    ($(#[$meta:meta])* $Name:ident, $schema_name:literal, $generator:ident => $body:expr) => {
        $(#[$meta])*
        #[derive(Debug)]
        pub enum $Name {}

        impl JsonSchema for $Name {
            fn schema_name() -> Cow<'static, str> {
                Cow::Borrowed($schema_name)
            }

            fn json_schema($generator: &mut SchemaGenerator) -> Schema {
                $body
            }
        }
    };
}

schema_type!(
    /// An entity's authoring key (`mineworld_contracts::EntityKey`): the `alice` of `people/alice.yaml`.
    Key, "EntityKey", _generator => identifier(
        "An authoring key: the name of a file in people/, places/, items/ or organizations/ without \
         .yaml. 1 to 64 lowercase letters, digits, - and _, not starting or ending with - or _."
    )
);

schema_type!(
    /// A system's id (`mineworld_contracts::SystemId`): `presence`, `calendar`.
    SystemName, "SystemId", _generator => identifier(
        "The id of a System Pack, such as presence or calendar. 1 to 64 lowercase letters, digits, - \
         and _, not starting or ending with - or _."
    )
);

schema_type!(
    /// One tag (`mineworld_contracts::Tag`).
    TagName, "Tag", _generator => identifier(
        "One label, such as cafe or night-shift. 1 to 64 lowercase letters, digits, - and _, not \
         starting or ending with - or _."
    )
);

schema_type!(
    /// An entity's tags (`mineworld_contracts::Tags`): a set, written as a list.
    TagList, "Tags", generator => {
        let tag = generator.subschema_for::<TagName>();
        json_schema!({
            "type": "array",
            "items": tag,
            "description": "Labels this entity carries; a tag listed twice counts once. Systems and \
                            clients read tags, never prose.",
        })
    }
);

schema_type!(
    /// A length or a coordinate in millimetres (`mineworld_contracts::Millimetres`).
    Length, "Millimetres", _generator => json_schema!({
        "type": "integer",
        "format": "int32",
        "minimum": i32::MIN,
        "maximum": i32::MAX,
        "description": "A length or a coordinate, in whole millimetres (1 m = 1000).",
    })
);

schema_type!(
    /// One of the four entity types (`mineworld_contracts::EntityType`).
    EntityTypeName, "EntityType", _generator => json_schema!({
        "type": "string",
        "enum": ["person", "place", "item", "organization"],
        "description": "One of the four kinds of entity: person, place, item or organization.",
    })
);

#[cfg(test)]
mod tests {
    use mineworld_contracts::{EntityType, MAX_IDENTIFIER_LENGTH};
    use serde::Deserialize;
    use serde::de::IntoDeserializer;
    use serde::de::value::Error;

    use super::*;

    /// The schemas state the contracts' rules: the identifier bound is the contract's constant, and
    /// every entity type the schema lists is one the contract decodes, all four of them.
    #[test]
    fn the_schemas_state_the_contracts_own_rules() {
        assert_eq!(IDENTIFIER_MAX, MAX_IDENTIFIER_LENGTH);
        let schema = EntityTypeName::json_schema(&mut SchemaGenerator::default());
        let decoded: Vec<EntityType> = schema
            .get("enum")
            .and_then(|names| names.as_array())
            .expect("an enum")
            .iter()
            .map(|name| {
                let name = name.as_str().expect("a name");
                EntityType::deserialize(name.into_deserializer())
                    .map_err(|error: Error| error.to_string())
                    .expect("the contract decodes every listed name")
            })
            .collect();
        assert_eq!(
            decoded,
            [
                EntityType::Person,
                EntityType::Place,
                EntityType::Item,
                EntityType::Organization
            ]
        );
    }
}
