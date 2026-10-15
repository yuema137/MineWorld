//! `docs/guide/features.toml`: which guide features are merged (`docs/GUIDE_FORMAT.md` §9).

use std::collections::BTreeMap;

/// Each guide feature's name, and whether it is merged.
pub type Features = BTreeMap<String, bool>;

/// Reads `features.toml`. Its only table is `[features]`, whose entries are lowercase names with a
/// boolean value; anything else is refused, naming what it is.
pub fn read(text: &str) -> Result<Features, String> {
    let table: toml::Table = text
        .parse()
        .map_err(|error| format!("features.toml: {error}"))?;
    let mut features = None;
    for (key, value) in table {
        let toml::Value::Table(entries) = value else {
            return Err(format!("features.toml: {key:?} is not a [features] table"));
        };
        if key != "features" {
            return Err(format!("features.toml: unknown table {key:?}"));
        }
        let mut read = Features::new();
        for (name, flag) in entries {
            let well_formed = !name.is_empty()
                && name
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            if !well_formed {
                return Err(format!(
                    "features.toml: {name:?} is not a feature name (lowercase letters, digits, hyphens)"
                ));
            }
            let toml::Value::Boolean(flag) = flag else {
                return Err(format!("features.toml: {name:?} must be true or false"));
            };
            read.insert(name, flag);
        }
        features = Some(read);
    }
    features.ok_or_else(|| "features.toml: no [features] table".to_owned())
}
