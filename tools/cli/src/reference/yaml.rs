//! A schema's example, printed as the YAML an author writes (`pr-s25a-reference.md` SD-25a-7): block
//! style, two spaces per level, and a string quoted whenever YAML could read it as anything but that
//! string. The reference test reads every printed example back through the loader's own parser.

use serde_json::Value;

/// `value` as a YAML document, ending in a newline.
pub(super) fn document(value: &Value) -> String {
    let mut out = String::new();
    match value {
        Value::Object(map) if !map.is_empty() => mapping(&mut out, map, 0),
        Value::Array(items) if !items.is_empty() => sequence(&mut out, items, 0),
        scalar => {
            out.push_str(&inline(scalar));
            out.push('\n');
        }
    }
    out
}

/// The entries of a mapping, each on its own line at `indent`.
fn mapping(out: &mut String, map: &serde_json::Map<String, Value>, indent: usize) {
    for (key, value) in map {
        pad(out, indent);
        out.push_str(&string(key));
        out.push(':');
        nested(out, value, indent);
    }
}

/// The items of a sequence, each on its own line at `indent`.
fn sequence(out: &mut String, items: &[Value], indent: usize) {
    for item in items {
        pad(out, indent);
        out.push('-');
        match item {
            // The first entry shares the dash's line; the others line up under it.
            Value::Object(map) if !map.is_empty() => {
                let mut entries = String::new();
                mapping(&mut entries, map, indent + 2);
                out.push(' ');
                out.push_str(entries.trim_start());
            }
            other => nested(out, other, indent),
        }
    }
}

/// What follows a key's colon or an item's dash: a scalar on the same line, or a block below it.
fn nested(out: &mut String, value: &Value, indent: usize) {
    match value {
        Value::Object(map) if !map.is_empty() => {
            out.push('\n');
            mapping(out, map, indent + 2);
        }
        // A list of numbers reads best on one line, in flow style: [8, 28, 54].
        Value::Array(items) if !items.is_empty() && items.iter().all(Value::is_number) => {
            let items: Vec<String> = items.iter().map(inline).collect();
            out.push_str(&format!(" [{}]\n", items.join(", ")));
        }
        Value::Array(items) if !items.is_empty() => {
            out.push('\n');
            sequence(out, items, indent + 2);
        }
        scalar => {
            out.push(' ');
            out.push_str(&inline(scalar));
            out.push('\n');
        }
    }
}

/// A scalar, or an empty collection, on one line.
fn inline(value: &Value) -> String {
    match value {
        Value::Null => "null".to_owned(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => string(text),
        Value::Array(_) => "[]".to_owned(),
        Value::Object(_) => "{}".to_owned(),
    }
}

/// Words a YAML parser may read as something other than a string (YAML 1.1's and 1.2's).
const SPECIAL: [&str; 12] = [
    "true", "false", "yes", "no", "on", "off", "y", "n", "null", "~", "nan", "inf",
];

/// A string, plain when it is plainly a word or a phrase, else double-quoted: a JSON string literal is
/// a valid YAML double-quoted scalar.
fn string(text: &str) -> String {
    let mut characters = text.chars();
    let plain = characters
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && characters
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, ' ' | '_' | '-' | '.' | '/'))
        && !text.ends_with(' ')
        && !SPECIAL.contains(&text.to_ascii_lowercase().as_str());
    if plain {
        text.to_owned()
    } else {
        serde_json::to_string(text).unwrap_or_else(|_| format!("\"{text}\""))
    }
}

fn pad(out: &mut String, indent: usize) {
    out.extend(std::iter::repeat_n(' ', indent));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The printed text is the value: blocks nest, a sequence of mappings shares the dash's line, and
    /// every string YAML would read as a number, a boolean, a time or a null is quoted.
    #[test]
    fn a_printed_example_reads_back_as_the_value() {
        let value = serde_json::json!({
            "name": "Ana Lee",
            "count": 2,
            "open": "08:00",
            "flag": "yes",
            "version": "^0.1",
            "empty": "",
            "ok": true,
            "list": [{ "at": "shop", "prices": { "bread": 250 } }, "plain", 3],
            "none": [],
            "rain": [8, 28, -5],
        });
        let printed = document(&value);
        assert_eq!(
            printed,
            "count: 2\n\
             empty: \"\"\n\
             flag: \"yes\"\n\
             list:\n  \
             - at: shop\n    \
             prices:\n      \
             bread: 250\n  \
             - plain\n  \
             - 3\n\
             name: Ana Lee\n\
             none: []\n\
             ok: true\n\
             open: \"08:00\"\n\
             rain: [8, 28, -5]\n\
             version: \"^0.1\"\n"
        );
        let read: Value = serde_saphyr::from_str(&printed).expect("the printed example parses");
        assert_eq!(read, value);
        assert_eq!(document(&serde_json::json!("Ana Lee")), "Ana Lee\n");
    }
}
