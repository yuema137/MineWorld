//! What the reference documents, asked of the types that decode each file (`docs/DECISIONS.md`
//! `ARC-90`): the World Pack format's own files, then every installed pack, through the installed
//! set's [`Capability`] — the generator names no pack.
//!
//! Every schema is normalized here before anything renders it: each `description` becomes its doc
//! comment's first paragraph (`pr-s25a-reference.md` SD-25a-1), and a schema with a field that has no
//! description, or with no example, is refused by name.

use mineworld_worldpack::catalog::AVAILABLE;
use mineworld_worldpack::interactions::Declarations;
use mineworld_worldpack::{Capability, ContentKind, schema};
use serde_json::Value;

/// One documented file or section.
pub(super) struct Entry {
    /// Its anchor and its schema file's stem, such as `section-name`.
    pub slug: String,
    /// Its heading, such as `` `name` ``.
    pub title: String,
    /// Where it is written and who reads it, as Markdown lines.
    pub about: Vec<String>,
    /// Its JSON Schema, normalized.
    pub schema: Value,
    /// The key its example is printed under, for a section of a content file.
    pub wrap: Option<String>,
    /// What its pack declares, for an Interaction List section.
    pub declarations: Option<Declarations>,
}

/// The documented entries of every page, each page's in its order.
pub(super) struct Entries {
    pub world_pack: Vec<Entry>,
    pub sections: Vec<Entry>,
    pub configure: Vec<Entry>,
    pub interaction_list: Vec<Entry>,
    pub decisions: Vec<Entry>,
    pub pack_manifest: Vec<Entry>,
}

/// Everything this build documents.
///
/// # Errors
///
/// A schema with an undescribed field or without an example, named.
pub(super) fn collect() -> Result<Entries, String> {
    let mut world_pack = vec![entry(
        "world",
        "`world.yaml`",
        vec!["The World Pack's manifest, at the root of its directory.".to_owned()],
        schema::world_manifest().to_value(),
    )?];
    for kind in ContentKind::ALL {
        world_pack.push(entry(
            kind.describes(),
            &format!("`{}/<key>.yaml`", kind.directory()),
            vec![format!(
                "One {} per file; the file's name without `.yaml` is its key.",
                kind.describes()
            )],
            schema::content_file(kind).to_value(),
        )?);
    }

    let mut configure = vec![
        entry(
            "configure-classes",
            "`configure/classes.yaml`",
            configured("classes", the_format()),
            schema::classes().to_value(),
        )?,
        entry(
            "configure-packages",
            "`configure/packages.yaml`",
            configured("packages", the_format()),
            schema::packages().to_value(),
        )?,
    ];
    let mut sections = Vec::new();
    let mut interaction_list = Vec::new();
    for capability in AVAILABLE {
        if let (Some(owner), Some(schema)) = (capability.section(), capability.section_schema()) {
            sections.push(section(
                capability,
                owner.name.as_str(),
                owner.carried_by,
                schema.to_value(),
            )?);
        }
        if let Some(schema) = capability.configuration_schema() {
            let id = capability.id();
            let mut configured = entry(
                &format!("configure-{id}"),
                &format!("`configure/{id}.yaml`"),
                configured(id.as_str(), owned_by(capability)),
                schema.to_value(),
            )?;
            match capability.interaction_declarations() {
                Some(declarations) => {
                    configured.declarations = Some(declarations);
                    interaction_list.push(configured);
                }
                None => configure.push(configured),
            }
        }
    }

    let decisions = vec![entry(
        "decisions",
        "`decisions.yaml`",
        vec![
            "The decision list, at the path `world.yaml` names under `decisions:`; `--decisions FILE` \
             replaces it for one run (`docs/MODULE_SPEC.md` §4.1 states its rules)."
                .to_owned(),
            "Read by the hosts with the models this build installs, never seeded into world state."
                .to_owned(),
        ],
        mineworld_decision::file_schema::<mineworld_installed_models::Model>().to_value(),
    )?];

    let pack_manifest = vec![
        entry(
            "pack",
            "`pack.yaml`",
            vec![
                "The package identity of an Entity Pack or a Presentation Pack, at the root of its \
                 directory."
                    .to_owned(),
            ],
            mineworld_packages::pack_file_schema().to_value(),
        )?,
        entry(
            "style",
            "`manifest.yaml` (a style manifest)",
            vec![
                "The fields of a Presentation Pack's style manifest that `mineworld packs validate` \
                 checks. Its other keys are the art direction's (`docs/ART_DIRECTION.md` §12)."
                    .to_owned(),
            ],
            mineworld_packages::style_manifest_schema().to_value(),
        )?,
    ];
    Ok(Entries {
        world_pack,
        sections,
        configure,
        interaction_list,
        decisions,
        pack_manifest,
    })
}

/// A section's entry: the files that carry it and the system that owns it.
fn section(
    capability: Capability,
    key: &str,
    carried_by: &[ContentKind],
    schema: Value,
) -> Result<Entry, String> {
    let files: Vec<String> = carried_by
        .iter()
        .map(|kind| format!("`{}/<key>.yaml`", kind.directory()))
        .collect();
    let mut section = entry(
        &format!("section-{key}"),
        &format!("`{key}`"),
        vec![
            format!("Written under the key `{key}` in {}.", files.join(", ")),
            format!(
                "{} A file whose world does not enable it is refused.",
                owned_by(capability)
            ),
        ],
        schema,
    )?;
    section.wrap = Some(key.to_owned());
    Ok(section)
}

/// Who reads a file: the system, and the pack it comes in.
fn owned_by(capability: Capability) -> String {
    format!(
        "Read by the `{}` system (package `{}`), which a world enables in `systems:`.",
        capability.id(),
        capability.package().name()
    )
}

/// Where a `configure/` file is named, and who reads it.
fn configured(key: &str, reader: String) -> Vec<String> {
    vec![
        format!("Read when `world.yaml` lists `{key}` under `configure:`."),
        reader,
    ]
}

/// The reader of the format's own `configure/` files.
fn the_format() -> String {
    "Read by the World Pack format itself, whatever systems the world enables.".to_owned()
}

/// An entry, its schema normalized and checked.
fn entry(slug: &str, title: &str, about: Vec<String>, schema: Value) -> Result<Entry, String> {
    let schema = normalized(title, schema)?;
    Ok(Entry {
        slug: slug.to_owned(),
        title: title.to_owned(),
        about,
        schema,
        wrap: None,
        declarations: None,
    })
}

/// `schema` with every description cut to its first paragraph, refused when a field has no
/// description or the schema states no example.
///
/// # Errors
///
/// Naming `title`, the type and the field.
pub(super) fn normalized(title: &str, mut schema: Value) -> Result<Value, String> {
    first_paragraphs(&mut schema);
    let root = schema
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("the root")
        .to_owned();
    undescribed(&schema, &root).map_or(Ok(()), |field| {
        Err(format!(
            "[mineworld] reference: {title}: the field {field} has no description; write an \
             author-facing doc comment on it"
        ))
    })?;
    if let Some(defs) = schema.get("$defs").and_then(Value::as_object) {
        for (name, def) in defs {
            if let Some(field) = undescribed(def, name) {
                return Err(format!(
                    "[mineworld] reference: {title}: the field {field} has no description; write \
                     an author-facing doc comment on it"
                ));
            }
        }
    }
    let has_example = schema
        .get("examples")
        .and_then(Value::as_array)
        .is_some_and(|examples| !examples.is_empty());
    if !has_example {
        return Err(format!(
            "[mineworld] reference: {title} ({root}) states no example; give its type one with \
             #[schemars(example = …)]"
        ));
    }
    Ok(schema)
}

/// Keys whose values are data, not schemas: never rewritten, never checked.
const DATA: [&str; 4] = ["examples", "default", "enum", "const"];

/// Every `description` under `value` cut to its first paragraph, its lines joined by a space.
pub(super) fn first_paragraphs(value: &mut Value) {
    match value {
        Value::Object(map) => {
            for (key, child) in map.iter_mut() {
                if DATA.contains(&key.as_str()) {
                    continue;
                }
                match child {
                    Value::String(text) if key == "description" => *text = first_paragraph(text),
                    _ => first_paragraphs(child),
                }
            }
        }
        Value::Array(items) => items.iter_mut().for_each(first_paragraphs),
        _ => {}
    }
}

/// The first paragraph of a doc comment: up to its first blank line, lines trimmed and joined.
pub(super) fn first_paragraph(text: &str) -> String {
    text.lines()
        .map(str::trim)
        .skip_while(|line| line.is_empty())
        .take_while(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

/// The first field under `schema` (outside the `$defs`, which the caller walks) with no description,
/// as `Type.field`.
fn undescribed(schema: &Value, name: &str) -> Option<String> {
    walk(schema, name, false)
}

/// [`undescribed`], knowing whether `schema` is one alternative of a `oneOf` or an `anyOf`.
fn walk(schema: &Value, name: &str, alternative: bool) -> Option<String> {
    let map = schema.as_object()?;
    if let Some(properties) = map.get("properties").and_then(Value::as_object) {
        // An enum's variant written as `{ variant: … }`: its doc comment is the object's.
        let variant = alternative && properties.len() == 1 && has_description(schema);
        for (field, property) in properties {
            if !variant && !has_description(property) {
                return Some(format!("{name}.{field}"));
            }
            if let Some(inner) = walk(property, &format!("{name}.{field}"), false) {
                return Some(inner);
            }
        }
    }
    let nested = ["items", "additionalProperties"]
        .into_iter()
        .filter_map(|key| map.get(key))
        .find_map(|child| walk(child, name, false));
    nested.or_else(|| {
        ["anyOf", "oneOf", "allOf"]
            .into_iter()
            .filter_map(|key| map.get(key).and_then(Value::as_array))
            .flatten()
            .find_map(|child| walk(child, name, true))
    })
}

fn has_description(schema: &Value) -> bool {
    schema
        .get("description")
        .and_then(Value::as_str)
        .is_some_and(|text| !text.trim().is_empty())
}

#[cfg(test)]
mod tests {
    use schemars::{JsonSchema, SchemaGenerator};

    use super::*;

    /// A field the author wrote no doc comment for.
    #[derive(JsonSchema)]
    #[schemars(example = serde_json::json!({ "minutes": 1 }))]
    #[allow(dead_code)]
    struct Spot {
        /// How long, in minutes.
        minutes: u32,
        /// Where.
        at: Inner,
    }

    #[derive(JsonSchema)]
    #[allow(dead_code)]
    struct Inner {
        /// The place.
        place: String,
        depth: u32,
    }

    /// AC-25a-3: a field without a doc comment refuses generation, naming the type and the field —
    /// here one inside a nested type; and a schema without an example is refused by name.
    #[test]
    fn a_field_without_a_description_refuses_generation_by_name() {
        let schema = SchemaGenerator::default()
            .into_root_schema_for::<Spot>()
            .to_value();
        let refusal = normalized("`fishing`", schema).expect_err("refused");
        assert!(refusal.contains("`fishing`"), "{refusal}");
        assert!(refusal.contains("Inner.depth"), "{refusal}");

        let schema = SchemaGenerator::default()
            .into_root_schema_for::<Inner>()
            .to_value();
        let refusal = normalized("`inner`", schema).expect_err("refused");
        assert!(refusal.contains("Inner.depth"), "{refusal}");
    }

    /// The first paragraph is what an author reads; what follows the blank line is for developers.
    #[test]
    fn a_description_is_its_doc_comments_first_paragraph() {
        assert_eq!(
            first_paragraph("How long a catch takes,\n in minutes.\n\nThe decoder checks it."),
            "How long a catch takes, in minutes."
        );
        let mut schema = serde_json::json!({
            "description": "One.\n\nTwo.",
            "properties": { "description": { "description": "A.\nB.\n\nC." } },
            "examples": [{ "description": "kept\n\nwhole" }],
        });
        first_paragraphs(&mut schema);
        assert_eq!(
            schema,
            serde_json::json!({
                "description": "One.",
                "properties": { "description": { "description": "A. B." } },
                "examples": [{ "description": "kept\n\nwhole" }],
            })
        );
    }
}
