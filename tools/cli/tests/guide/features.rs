//! `docs/guide/features.toml`: which guide features are merged (`docs/GUIDE_FORMAT.md` §9).

use std::collections::BTreeMap;

use super::report::Verdict;

/// Each guide feature's name, and whether it is merged.
pub type Features = BTreeMap<String, bool>;

/// The pack `mineworld packs list` names when a feature is in the build, for each feature that has one
/// (`docs/GUIDE_FORMAT.md` §9.1). A feature with no probe cannot be checked for presence.
pub fn probe(feature: &str) -> Option<&'static str> {
    match feature {
        "calendar" => Some("mineworld-calendar"),
        "weather" => Some("mineworld-weather"),
        "happenings" => Some("mineworld-happenings"),
        _ => None,
    }
}

/// The presence check (§9, rule 3), one verdict per feature: a `false` flag whose pack is in the build
/// fails; a `true` flag whose pack is not in the build fails; a feature with no probe is inconclusive.
/// `listing` is the output of `mineworld packs list`, whose second column is each pack's name.
pub fn against_build(features: &Features, listing: &str) -> Vec<(String, Verdict)> {
    let built: Vec<&str> = listing
        .lines()
        .filter_map(|line| line.split_whitespace().nth(1))
        .collect();
    features
        .iter()
        .map(|(feature, flag)| {
            let verdict = match probe(feature) {
                None => Verdict::Inconclusive(
                    "no presence probe is defined for this feature, so the build is not checked"
                        .to_owned(),
                ),
                Some(pack) => match (*flag, built.contains(&pack)) {
                    (false, true) => Verdict::Fail(format!(
                        "{feature} = false, but pack {pack} is in the build: set the flag to true"
                    )),
                    (true, false) => Verdict::Fail(format!(
                        "{feature} = true, but pack {pack} is not in the build"
                    )),
                    _ => Verdict::Pass,
                },
            };
            (feature.clone(), verdict)
        })
        .collect()
}

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
