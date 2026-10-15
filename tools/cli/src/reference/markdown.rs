//! An entry's schema as Markdown a person reads: what it is, a table of its fields, the types those
//! fields name, and an example (`docs/DECISIONS.md` `ARC-90`). Everything printed is read off the
//! normalized schema; nothing here states a field, a bound or a default of its own.

use std::fmt::Write as _;

use serde_json::{Map, Value};

use super::entries::Entry;
use super::yaml;

/// The refusal a page predicts for a key its file does not list, in serde's own words.
const UNKNOWN_KEY: &str = "unknown field";

/// The same refusal for an Interaction List section, in its decoder's words
/// (`sdk/rust/src/interactions/section.rs` `SectionKey`); the reference test holds the two equal.
const UNKNOWN_SECTION_KEY: &str = "is not a key of a section";

/// One entry: heading, what it is, its fields, its types, its example.
pub(super) fn entry(out: &mut String, entry: &Entry) {
    let schema = &entry.schema;
    introduce(out, entry);
    let mut types = Types::new(&entry.slug, schema);
    body(out, schema, &mut types);
    refusal(out, schema, UNKNOWN_KEY);
    example(out, entry);
    types.render(out);
}

/// An entry's heading, where it is written, and what it is.
fn introduce(out: &mut String, entry: &Entry) {
    let _ = writeln!(out, "<a id=\"{}\"></a>\n", entry.slug);
    let _ = writeln!(out, "## {}\n", entry.title);
    for line in &entry.about {
        let _ = writeln!(out, "{line}");
    }
    out.push('\n');
    if let Some(description) = description(&entry.schema) {
        let _ = writeln!(out, "{description}\n");
    }
}

/// An entry whose shape the page states once (an Interaction List section): its heading, what it is
/// and its example; the caller adds what it declares.
pub(super) fn brief(out: &mut String, entry: &Entry) {
    introduce(out, entry);
    refusal(out, &entry.schema, UNKNOWN_SECTION_KEY);
    example(out, entry);
}

/// The refusal a closed object's file meets for a key it does not list.
fn refusal(out: &mut String, schema: &Value, message: &str) {
    if closed(schema) {
        let _ = writeln!(
            out,
            "Any other key is refused with `{message}`, naming the key.\n"
        );
    }
}

/// A schema's fields and the types they name, with no heading and no example: a parameter block.
pub(super) fn shape(out: &mut String, slug: &str, schema: &Value) {
    let mut types = Types::new(slug, schema);
    body(out, schema, &mut types);
    types.render(out);
}

/// What a schema says about itself, or nothing.
pub(super) fn description(schema: &Value) -> Option<&str> {
    schema
        .get("description")
        .and_then(Value::as_str)
        .filter(|text| !text.is_empty())
}

/// Whether an object schema refuses keys it does not list.
fn closed(schema: &Value) -> bool {
    schema.get("properties").is_some()
        && schema.get("additionalProperties") == Some(&Value::Bool(false))
}

/// A schema's own shape: a field table for an object, else one line saying what it is.
fn body(out: &mut String, schema: &Value, types: &mut Types<'_>) {
    match schema.get("properties").and_then(Value::as_object) {
        Some(properties) => fields(out, schema, properties, types),
        None => {
            let _ = write!(out, "**Type:** {}.", types.type_of(schema));
            let bounds = bounds(schema);
            if !bounds.is_empty() {
                let _ = write!(out, " **Bounds:** {bounds}.");
            }
            out.push_str("\n\n");
            variants(out, schema, types);
        }
    }
}

/// The alternatives of a `oneOf`, each with what it means.
fn variants(out: &mut String, schema: &Value, types: &mut Types<'_>) {
    let Some(alternatives) = schema.get("oneOf").and_then(Value::as_array) else {
        return;
    };
    for alternative in alternatives {
        let shape = types
            .variant(alternative)
            .unwrap_or_else(|| types.type_of(alternative));
        match description(alternative) {
            Some(meaning) => {
                let _ = writeln!(out, "- {shape}: {meaning}");
            }
            None => {
                let _ = writeln!(out, "- {shape}");
            }
        }
    }
    out.push('\n');
}

/// A table of an object's fields, in name order.
fn fields(
    out: &mut String,
    schema: &Value,
    properties: &Map<String, Value>,
    types: &mut Types<'_>,
) {
    let required: Vec<&str> = schema
        .get("required")
        .and_then(Value::as_array)
        .map(|names| names.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    out.push_str("| Field | Type | Required | Default | Bounds | Description |\n");
    out.push_str("| --- | --- | --- | --- | --- | --- |\n");
    for (name, property) in properties {
        types.at = if types.base.is_empty() {
            name.clone()
        } else {
            format!("{}.{name}", types.base)
        };
        let default = property
            .get("default")
            .map_or_else(|| "—".to_owned(), |value| format!("`{value}`"));
        let _ = writeln!(
            out,
            "| `{name}` | {} | {} | {} | {} | {} |",
            cell(&types.type_of(property)),
            if required.contains(&name.as_str()) {
                "yes"
            } else {
                "no"
            },
            cell(&default),
            cell(&or_dash(bounds(property))),
            cell(description(property).unwrap_or("")),
        );
    }
    out.push('\n');
}

/// The entry's example, as its file holds it.
fn example(out: &mut String, entry: &Entry) {
    let Some(example) = example_of(entry) else {
        return;
    };
    out.push_str("Example:\n\n```yaml\n");
    out.push_str(&yaml::document(&example));
    out.push_str("```\n\n");
}

/// The entry's first example, under its key when it is a section of a content file.
pub(super) fn example_of(entry: &Entry) -> Option<Value> {
    let example = entry.schema.get("examples")?.get(0)?.clone();
    Some(match &entry.wrap {
        Some(key) => serde_json::json!({ key: example }),
        None => example,
    })
}

/// A schema's bounds, in words.
pub(super) fn bounds(schema: &Value) -> String {
    let number = |key: &str| schema.get(key).filter(|value| value.is_number());
    let mut parts = Vec::new();
    match (number("minimum"), number("maximum")) {
        (Some(low), Some(high)) => parts.push(format!("{low} to {high}")),
        (Some(low), None) => parts.push(format!("at least {low}")),
        (None, Some(high)) => parts.push(format!("at most {high}")),
        (None, None) => {}
    }
    if let Some(low) = number("exclusiveMinimum") {
        parts.push(format!("more than {low}"));
    }
    if let Some(high) = number("exclusiveMaximum") {
        parts.push(format!("less than {high}"));
    }
    match (number("minLength"), number("maxLength")) {
        (Some(low), Some(high)) if low == high => parts.push(format!("exactly {low} characters")),
        (Some(low), Some(high)) => parts.push(format!("{low} to {high} characters")),
        (Some(low), None) => parts.push(format!("at least {low} characters")),
        (None, Some(high)) => parts.push(format!("at most {high} characters")),
        (None, None) => {}
    }
    match (number("minItems"), number("maxItems")) {
        (Some(low), Some(high)) if low == high => parts.push(format!("exactly {low} entries")),
        (Some(low), Some(high)) => parts.push(format!("{low} to {high} entries")),
        (Some(low), None) => parts.push(format!("at least {low} entries")),
        (None, Some(high)) => parts.push(format!("at most {high} entries")),
        (None, None) => {}
    }
    if schema.get("uniqueItems") == Some(&Value::Bool(true)) {
        parts.push("no entry twice".to_owned());
    }
    if let Some(pattern) = schema.get("pattern").and_then(Value::as_str) {
        parts.push(format!("matches `{pattern}`"));
    }
    parts.join("; ")
}

fn or_dash(text: String) -> String {
    if text.is_empty() {
        "—".to_owned()
    } else {
        text
    }
}

/// Text that stays inside one table cell.
fn cell(text: &str) -> String {
    text.replace('|', "\\|").replace('\n', " ")
}

/// The named types an entry's fields refer to, and the inline objects nested in them, rendered after
/// the entry's own table.
struct Types<'a> {
    slug: &'a str,
    defs: Option<&'a Map<String, Value>>,
    /// Inline objects met while rendering, by the field path they were met at.
    inline: Vec<(String, Value)>,
    /// The type whose fields are being rendered: empty for the entry itself.
    base: String,
    /// The path of the field being rendered, for an inline object's name.
    at: String,
}

impl<'a> Types<'a> {
    fn new(slug: &'a str, root: &'a Value) -> Self {
        Self {
            slug,
            defs: root.get("$defs").and_then(Value::as_object),
            inline: Vec::new(),
            base: String::new(),
            at: String::new(),
        }
    }

    /// The anchor of a named type.
    fn anchor(&self, name: &str) -> String {
        format!("{}-{}", self.slug, name.to_ascii_lowercase())
    }

    /// What a schema is, in words, linking the named types it refers to.
    fn type_of(&mut self, schema: &Value) -> String {
        if let Some(value) = schema.get("const") {
            return format!("`{}`", plain(value));
        }
        // Before a `$ref`: a field whose bound names its choices (a named-choice `ParameterBound`)
        // narrows its type's schema to them, and the choices are what an author writes.
        if let Some(values) = schema.get("enum").and_then(Value::as_array) {
            let values: Vec<String> = values.iter().map(|v| format!("`{}`", plain(v))).collect();
            return format!("one of {}", values.join(", "));
        }
        if let Some(reference) = schema.get("$ref").and_then(Value::as_str) {
            let name = reference.rsplit('/').next().unwrap_or(reference);
            return format!("[{name}](#{})", self.anchor(name));
        }
        for key in ["anyOf", "oneOf"] {
            if let Some(alternatives) = schema.get(key).and_then(Value::as_array) {
                let shapes: Vec<String> = alternatives
                    .iter()
                    .filter(|alternative| !is_null(alternative))
                    .map(|alternative| {
                        self.variant(alternative)
                            .unwrap_or_else(|| self.type_of(alternative))
                    })
                    .collect();
                return shapes.join(" or ");
            }
        }
        let kinds: Vec<&str> = match schema.get("type") {
            Some(Value::String(kind)) => vec![kind.as_str()],
            Some(Value::Array(kinds)) => kinds.iter().filter_map(Value::as_str).collect(),
            _ => Vec::new(),
        };
        let words: Vec<String> = kinds
            .iter()
            .filter(|kind| **kind != "null" || kinds.len() == 1)
            .map(|kind| self.kind(kind, schema))
            .collect();
        if words.is_empty() {
            "anything".to_owned()
        } else {
            words.join(" or ")
        }
    }

    /// An enum's variant written `{ variant: … }`, as `` `variant:` `` and what follows it.
    fn variant(&mut self, alternative: &Value) -> Option<String> {
        let properties = alternative.get("properties")?.as_object()?;
        let (name, inner) = properties.iter().next().filter(|_| properties.len() == 1)?;
        let required = alternative
            .get("required")
            .and_then(Value::as_array)
            .is_some_and(|names| names.len() == 1);
        if !required {
            return None;
        }
        Some(format!("`{name}:` {}", self.valued(inner)))
    }

    /// A type with its own bounds, for a value nested in a map or a variant.
    fn valued(&mut self, schema: &Value) -> String {
        let shape = self.nested(schema, "");
        match bounds(schema) {
            bounds if bounds.is_empty() => shape,
            bounds => format!("{shape} ({bounds})"),
        }
    }

    /// One JSON type, in words.
    fn kind(&mut self, kind: &str, schema: &Value) -> String {
        match kind {
            "string" => "text".to_owned(),
            "integer" => "whole number".to_owned(),
            "number" => "number".to_owned(),
            "boolean" => "true or false".to_owned(),
            "null" => "nothing".to_owned(),
            "array" => match schema.get("items") {
                Some(items) => format!("list of {}", self.nested(items, "[]")),
                None => "list".to_owned(),
            },
            "object" => {
                if schema.get("properties").is_some() {
                    return self.nested(schema, "");
                }
                let patterned = schema
                    .get("patternProperties")
                    .and_then(Value::as_object)
                    .filter(|patterns| patterns.len() == 1)
                    .and_then(|patterns| patterns.iter().next());
                if let Some((pattern, value)) = patterned {
                    return format!(
                        "mapping: each key matches `{pattern}`, each value is {}",
                        self.valued(value)
                    );
                }
                match schema.get("additionalProperties") {
                    Some(value @ Value::Object(_)) => {
                        let keys = schema
                            .get("propertyNames")
                            .map_or_else(|| "text".to_owned(), |names| self.type_of(names));
                        format!(
                            "mapping: each key is {keys}, each value is {}",
                            self.valued(value)
                        )
                    }
                    _ => "mapping".to_owned(),
                }
            }
            other => other.to_owned(),
        }
    }

    /// A subschema's type; an inline object is listed for a table of its own.
    fn nested(&mut self, schema: &Value, suffix: &str) -> String {
        if schema.get("properties").is_some() && schema.get("$ref").is_none() {
            let name = format!("{}{suffix}", self.at);
            let anchor = self.anchor(&name.replace(['[', ']', '{', '}', '.'], "-"));
            if !self.inline.iter().any(|(known, _)| *known == name) {
                self.inline.push((name.clone(), schema.clone()));
            }
            return format!("[`{name}`](#{anchor})");
        }
        self.type_of(schema)
    }

    /// The named and inline types, each with what it is and its fields.
    fn render(mut self, out: &mut String) {
        let defs: Vec<(String, Value)> = self
            .defs
            .map(|defs| defs.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
            .unwrap_or_default();
        let mut done = 0;
        let mut header = false;
        let mut heading = |out: &mut String| {
            if !header {
                out.push_str("### Types used above\n\n");
                header = true;
            }
        };
        for (name, def) in &defs {
            heading(out);
            let _ = writeln!(out, "<a id=\"{}\"></a>\n", self.anchor(name));
            let _ = writeln!(out, "#### `{name}`\n");
            if let Some(description) = description(def) {
                let _ = writeln!(out, "{description}\n");
            }
            self.base = name.clone();
            self.at = name.clone();
            body(out, def, &mut self);
            self.flush(out, &mut done, &mut heading);
        }
        self.flush(out, &mut done, &mut heading);
    }

    /// The inline objects not yet rendered, and any they lead to.
    fn flush(&mut self, out: &mut String, done: &mut usize, heading: &mut impl FnMut(&mut String)) {
        while *done < self.inline.len() {
            let (name, schema) = self.inline[*done].clone();
            *done += 1;
            heading(out);
            let anchor = self.anchor(&name.replace(['[', ']', '{', '}', '.'], "-"));
            let _ = writeln!(out, "<a id=\"{anchor}\"></a>\n");
            let _ = writeln!(out, "#### `{name}`\n");
            if let Some(description) = description(&schema) {
                let _ = writeln!(out, "{description}\n");
            }
            self.base = name.clone();
            self.at = name;
            body(out, &schema, self);
        }
    }
}

fn is_null(schema: &Value) -> bool {
    schema.get("type") == Some(&Value::String("null".to_owned()))
}

/// A value as an author writes it in YAML.
fn plain(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        other => other.to_string(),
    }
}
