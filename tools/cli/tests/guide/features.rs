//! `docs/guide/features.toml`: which guide features are merged (`docs/GUIDE_FORMAT.md` §9).

use std::collections::BTreeMap;

use super::report::Verdict;

/// Each guide feature's name, and whether it is merged.
pub type Features = BTreeMap<String, bool>;

/// What the binary shows when a feature is in the build (`docs/GUIDE_FORMAT.md` §9.1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Probe {
    /// A pack of this name in `mineworld packs list` (its second column).
    Pack(&'static str),
    /// A decision model of this id among the models `mineworld decisions` says the build installs.
    Model(&'static str),
    /// This text in this page of the reference `mineworld reference --out` generates.
    Reference {
        page: &'static str,
        text: &'static str,
    },
}

/// The probe of each feature that has one (§9.1). A feature with no probe cannot be checked for presence.
pub fn probe(feature: &str) -> Option<Probe> {
    match feature {
        "calendar" => Some(Probe::Pack("mineworld-calendar")),
        "weather" => Some(Probe::Pack("mineworld-weather")),
        "happenings" => Some(Probe::Pack("mineworld-happenings")),
        "interaction-list" => Some(Probe::Reference {
            page: "interaction-list.md",
            text: "## The shape every section shares",
        }),
        "decisions" => Some(Probe::Reference {
            page: "decisions.md",
            text: "## `decisions.yaml`",
        }),
        "motivated" => Some(Probe::Model("motivated")),
        "lm-controller" => Some(Probe::Model("lm")),
        "relationship-decay" => Some(Probe::Reference {
            page: "interaction-list.md",
            text: "`familiarity_decay`",
        }),
        _ => None,
    }
}

/// What the runner asked the binary, once per run, for the probes to read. Each part is the binary's
/// output, or the reason it could not be had.
#[derive(Debug)]
pub struct Build {
    /// The output of `mineworld packs list`.
    pub packs: Result<String, String>,
    /// The installed decision model ids, read from the refusal `mineworld decisions --seat-model` prints
    /// for a model no build installs (`it installs: rule, motivated`).
    pub models: Result<Vec<String>, String>,
    /// Each generated reference page, by file name, with its text.
    pub reference: Result<BTreeMap<String, String>, String>,
}

/// Whether the build shows `probe`, or why that cannot be said.
fn present(build: &Build, probe: Probe) -> Result<bool, String> {
    match probe {
        Probe::Pack(pack) => {
            let listing = build.packs.as_ref()?;
            Ok(listing
                .lines()
                .any(|line| line.split_whitespace().nth(1) == Some(pack)))
        }
        Probe::Model(id) => Ok(build.models.as_ref()?.iter().any(|model| model == id)),
        Probe::Reference { page, text } => Ok(build
            .reference
            .as_ref()?
            .get(page)
            .is_some_and(|source| source.contains(text))),
    }
}

/// How a probe is named in a verdict.
fn describe(probe: Probe) -> String {
    match probe {
        Probe::Pack(pack) => format!("pack {pack}"),
        Probe::Model(id) => format!("decision model {id}"),
        Probe::Reference { page, text } => format!("{text:?} in the generated reference's {page}"),
    }
}

/// The presence check (§9, rule 3), one verdict per feature: a `false` flag whose probe the build shows
/// fails; a `true` flag whose probe it does not show fails; a feature with no probe is inconclusive. A
/// probe whose evidence the binary could not give fails: the run cannot say the flag is right.
pub fn against_build(features: &Features, build: &Build) -> Vec<(String, Verdict)> {
    features
        .iter()
        .map(|(feature, flag)| {
            let verdict = match probe(feature) {
                None => Verdict::Inconclusive(
                    "no presence probe is defined for this feature, so the build is not checked"
                        .to_owned(),
                ),
                Some(probe) => match (*flag, present(build, probe)) {
                    (_, Err(reason)) => Verdict::Fail(format!(
                        "{feature}: the probe ({}) could not be read: {reason}",
                        describe(probe)
                    )),
                    (false, Ok(true)) => Verdict::Fail(format!(
                        "{feature} = false, but {} is in the build: set the flag to true",
                        describe(probe)
                    )),
                    (true, Ok(false)) => Verdict::Fail(format!(
                        "{feature} = true, but {} is not in the build",
                        describe(probe)
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
