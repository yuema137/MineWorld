//! What a pack's section of the World's Interaction List may say, as JSON Schema and as data, for the
//! generated reference (`docs/DECISIONS.md` `ARC-90`, `DEP-49`).
//!
//! ```text
//! Section<S>     its JSON Schema: the six keys, built from S's declared actions, facts, roles,
//!                parameters and knobs — what the section's decoder accepts
//! Declarations   the same declarations as data, for tables a person reads
//! field, block   what parameters! expands its JsonSchema impls into
//! ```

use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::Serialize;
use serde_json::{Map, Value};

use super::InteractionSection;
use super::decl::{ActionDecl, FactDecl, Role};
use super::section::{Partial, Section};

/// What a pack's section declares, for the tools: its actions, facts and parameter roles, the schemas
/// of its parameter block and knobs, and the names of its reference lists.
#[derive(Debug, Clone)]
pub struct Declarations {
    /// Each action a rule may name, with its roles and whether a region may scope it.
    pub actions: &'static [ActionDecl],
    /// Each fact a consequence may name, with its audiences and whether its biography is configurable.
    pub facts: &'static [FactDecl],
    /// The roles a parameter entry may scope by.
    pub parameter_roles: &'static [Role],
    /// The parameter block: every field with its type, default, bounds and description.
    pub parameters: Schema,
    /// The pack's consequence knobs, in the same form; no properties when it has none.
    pub knobs: Schema,
    /// The names of the reference lists a section may extend, `default` first.
    pub reference_lists: Vec<&'static str>,
}

impl Declarations {
    /// What `S` declares.
    pub fn of<S: InteractionSection>() -> Self {
        Self {
            actions: S::ACTIONS,
            facts: S::FACTS,
            parameter_roles: S::PARAMETER_ROLES,
            parameters: SchemaGenerator::default().into_root_schema_for::<S::Parameters>(),
            knobs: SchemaGenerator::default().into_root_schema_for::<S::Knobs>(),
            reference_lists: S::reference_lists()
                .into_iter()
                .map(|(name, _)| name)
                .collect(),
        }
    }
}

/// One field of a parameter block, as [`parameters!`](crate::parameters) states it.
#[doc(hidden)]
pub struct Field {
    name: &'static str,
    schema: Value,
}

/// A parameter block field's schema: its type's, with `description` (the doc comment), `default`, and
/// for a number `minimum` and `maximum`. Used by [`parameters!`](crate::parameters).
#[doc(hidden)]
pub fn field<T: JsonSchema + Serialize>(
    generator: &mut SchemaGenerator,
    name: &'static str,
    doc: &str,
    default: &T,
    low: &T,
    high: &T,
) -> Field {
    let mut schema = generator.subschema_for::<T>();
    let description: Vec<&str> = doc
        .lines()
        .map(|line| line.strip_prefix(' ').unwrap_or(line))
        .collect();
    let description = description.join("\n");
    if !description.trim().is_empty() {
        schema.insert("description".to_owned(), Value::String(description));
    }
    let value = |value: &T| serde_json::to_value(value).unwrap_or(Value::Null);
    schema.insert("default".to_owned(), value(default));
    let (low, high) = (value(low), value(high));
    if low.is_number() && high.is_number() {
        schema.insert("minimum".to_owned(), low);
        schema.insert("maximum".to_owned(), high);
    }
    Field {
        name,
        schema: schema.to_value(),
    }
}

/// A parameter block's schema: an object of `fields`, none required, no other key. Used by
/// [`parameters!`](crate::parameters).
#[doc(hidden)]
pub fn block(fields: Vec<Field>) -> Schema {
    let properties: Map<String, Value> = fields
        .into_iter()
        .map(|field| (field.name.to_owned(), field.schema))
        .collect();
    json_schema!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false,
    })
}

/// The schema of one role's selector.
fn selector() -> Value {
    serde_json::json!({
        "type": "string",
        "pattern": "^(\\*|[a-z0-9]([a-z0-9_-]*[a-z0-9])?)$",
        "description": "* for anyone, or an entity class: one defined in configure/classes.yaml, or \
                        an implicit class (person, place, item, organization).",
    })
}

/// The `properties` of an object schema, or none.
fn properties_of(schema: &Schema) -> Map<String, Value> {
    schema
        .get("properties")
        .and_then(Value::as_object)
        .cloned()
        .unwrap_or_default()
}

/// Each role in `roles`, once, in [`Role::ALL`]'s order, as a selector property.
fn role_properties(roles: impl Iterator<Item = Role>) -> Map<String, Value> {
    let mut named: Vec<Role> = roles.collect();
    named.sort_unstable();
    named.dedup();
    named
        .into_iter()
        .map(|role| (role.name().to_owned(), selector()))
        .collect()
}

/// The schema of a rule, in or out of a region.
fn rule<S: InteractionSection>(in_region: bool) -> Value {
    let actions: Vec<&str> = S::ACTIONS
        .iter()
        .filter(|action| !in_region || action.regional)
        .map(|action| action.action.as_str())
        .collect();
    let mut properties = role_properties(S::ACTIONS.iter().flat_map(|a| a.roles.iter().copied()));
    properties.insert(
        "action".to_owned(),
        serde_json::json!({
            "type": "string",
            "enum": actions,
            "description": if in_region {
                "The action the rule is about: one this pack lets a region scope."
            } else {
                "The action the rule is about: one this pack declares."
            },
        }),
    );
    properties.insert(
        "effect".to_owned(),
        effect("What the rule says: permit or forbid."),
    );
    serde_json::json!({
        "type": "object",
        "description": "A rule: an action, a selector per role it names, and permit or forbid. The \
                        most specific matching rule wins; between equals, forbid.",
        "properties": properties,
        "required": ["action", "effect"],
        "additionalProperties": false,
    })
}

/// The schema of `permit` or `forbid`.
fn effect(description: &str) -> Value {
    serde_json::json!({ "type": "string", "enum": ["permit", "forbid"], "description": description })
}

/// The schema of a parameter entry.
fn parameter_entry<S: InteractionSection>(partial: &Schema) -> Value {
    let mut properties = role_properties(S::PARAMETER_ROLES.iter().copied());
    properties.extend(properties_of(partial));
    serde_json::json!({
        "type": "object",
        "description": "A parameter entry: a selector per role it names, and the parameters it sets. \
                        With no role named it is the section's base.",
        "properties": properties,
        "additionalProperties": false,
    })
}

/// The schema of a consequence entry.
fn consequence_entry<S: InteractionSection>(knobs: &Schema) -> Value {
    let facts: Vec<&str> = S::FACTS.iter().map(|fact| fact.fact.as_str()).collect();
    let mut properties = role_properties(
        S::FACTS
            .iter()
            .flat_map(|fact| fact.roles.iter().map(|(role, _)| *role)),
    );
    properties.extend(properties_of(knobs));
    properties.insert(
        "fact".to_owned(),
        serde_json::json!({
            "type": "string",
            "enum": facts,
            "description": "The fact the entry is about: one this pack lets a list govern.",
        }),
    );
    properties.insert(
        "audience".to_owned(),
        serde_json::json!({
            "type": "string",
            "enum": ["public", "place", "participants"],
            "description": "Who may perceive the fact: only narrower than its owner's audience, and \
                            no narrower than the narrowest the owner allows.",
        }),
    );
    properties.insert(
        "biography".to_owned(),
        serde_json::json!({
            "type": ["string", "boolean"],
            "enum": ["on", "off", true, false],
            "description": "Whether the fact enters the biographies of the people in it: on or off, \
                            where the owner lets a list choose.",
        }),
    );
    serde_json::json!({
        "type": "object",
        "description": "A consequence entry: for one fact and a selector per role it names, its \
                        audience, whether it is biographical, and the pack's knobs.",
        "properties": properties,
        "required": ["fact"],
        "additionalProperties": false,
    })
}

/// The minimal valid section the reference prints: one permitting rule for the first action, and the
/// first parameter at its default.
fn example<S: InteractionSection>(parameters: &Schema) -> Value {
    let mut example = Map::new();
    if let Some(action) = S::ACTIONS.first() {
        example.insert(
            "rules".to_owned(),
            serde_json::json!([{ "action": action.action.as_str(), "effect": "permit" }]),
        );
    }
    if let Some((name, field)) = properties_of(parameters).into_iter().next() {
        let value = field.get("default").cloned().unwrap_or(Value::Null);
        example.insert(
            "parameters".to_owned(),
            serde_json::json!([{ name: value }]),
        );
    }
    Value::Object(example)
}

/// A section's schema: what [`Section`]'s decoder accepts for `S` (`ARC-63` items 1–4).
impl<S: InteractionSection> JsonSchema for Section<S> {
    fn schema_name() -> Cow<'static, str> {
        Cow::Owned(format!(
            "{}Section",
            <S as crate::__private::SystemIdentity>::ID
        ))
    }

    fn json_schema(_: &mut SchemaGenerator) -> Schema {
        // Each part as its own root, so its properties can be merged into the entry they belong to.
        let partial = SchemaGenerator::default().into_root_schema_for::<Partial<S>>();
        let knobs = SchemaGenerator::default().into_root_schema_for::<S::Knobs>();
        let parameters = SchemaGenerator::default().into_root_schema_for::<S::Parameters>();
        let lists: Vec<&str> = S::reference_lists()
            .into_iter()
            .map(|(name, _)| name)
            .collect();
        let entries = |in_region: bool| {
            serde_json::json!({
                "rules": { "type": "array", "items": rule::<S>(in_region),
                           "description": "Who may do what to whom: rules about this pack's actions." },
                "parameters": { "type": "array", "items": parameter_entry::<S>(&partial),
                                "description": "This pack's numbers, for everyone or for the classes \
                                                the entry names." },
                "consequences": { "type": "array", "items": consequence_entry::<S>(&knobs),
                                  "description": "What this pack's facts mean for who perceives them \
                                                  and for history." },
            })
        };
        let mut properties = entries(false).as_object().cloned().unwrap_or_default();
        properties.insert(
            "extends".to_owned(),
            serde_json::json!({
                "type": "string",
                "enum": lists,
                "description": "A reference list of this pack to start from; the section's entries \
                                are added after it. Chains of at most four lists, with no cycle.",
            }),
        );
        properties.insert(
            "default".to_owned(),
            effect("What an action no rule matches gets: permit (when unsaid) or forbid."),
        );
        properties.insert(
            "regions".to_owned(),
            serde_json::json!({
                "type": "object",
                "description": "Places whose own entries override the world's there, by place key.",
                "additionalProperties": {
                    "type": "object",
                    "properties": entries(true),
                    "additionalProperties": false,
                },
            }),
        );
        json_schema!({
            "type": ["object", "null"],
            "description": format!(
                "configure/{}.yaml: this pack's section of the World's Interaction List. Every key is \
                 optional; an empty file is today's behaviour.",
                <S as crate::__private::SystemIdentity>::ID
            ),
            "properties": properties,
            "additionalProperties": false,
            "examples": [example::<S>(&parameters)],
        })
    }
}
