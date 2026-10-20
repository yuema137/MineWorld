//! The lock (`DECISIONS.md` `ARC-123`; `MODULE_SPEC.md` §4.1 "The lock file"; `pr-s30d-sampler.md` §5.9;
//! Q-S30d-2): `locks/<name>.yaml` of a World Pack pins values per person with their origin, decoded as a
//! person-file value is, and written by one canonical writer.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};

use crate::catalogue::Catalogue;
use crate::declaration::{FormatOne, Mutability};
use crate::id::QualifiedId;
use crate::value::{AttributeValue, Loose, decode};

/// How a pinned value came to be pinned.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Pinned {
    /// By a ruling (a casting gate).
    Ruled,
    /// Pinned without a ruling.
    Locked,
}

impl Pinned {
    /// As written.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Ruled => "ruled",
            Self::Locked => "locked",
        }
    }
}

/// One entry as written.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEntry {
    value: Loose,
    origin: Pinned,
    at: String,
    #[serde(default)]
    by: Option<String>,
}

/// The profile a lock was written for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockedProfile {
    /// Its id.
    pub id: String,
    /// Its version.
    pub version: u32,
}

/// One `locks/*.yaml` file, decoded.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LockFile {
    #[allow(dead_code)]
    format: FormatOne,
    #[serde(default)]
    profile: Option<LockedProfile>,
    #[serde(default)]
    seed: Option<String>,
    #[serde(default)]
    people: BTreeMap<String, BTreeMap<String, RawEntry>>,
}

/// One pinned value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LockEntry {
    /// The value.
    pub value: AttributeValue,
    /// Ruled or locked.
    pub origin: Pinned,
    /// When, `YYYY-MM-DD`.
    pub at: String,
    /// By whom or what.
    pub by: Option<String>,
}

/// A lock, checked against the catalogue.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Lock {
    /// The profile it was written for.
    pub profile: Option<LockedProfile>,
    /// The seed it was written with.
    pub seed: Option<String>,
    /// Per person key, per attribute, the entry.
    pub people: BTreeMap<String, BTreeMap<QualifiedId, LockEntry>>,
}

/// Whether `text` is `YYYY-MM-DD`.
fn is_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(at, byte)| at == 4 || at == 7 || byte.is_ascii_digit())
}

impl Lock {
    /// A decoded file, checked: every id the catalogue's, every value its declaration's, the people it names are checked by the resolution, which knows them
    /// (`UnknownPerson`).
    ///
    /// # Errors
    ///
    /// The refusal's name and what: `UnknownPerson`, `UnknownAttribute`, `WrongKind`, `OutOfDomain`,
    /// `OutOfRange`, `BadDeclaration` for a date or a `by`.
    pub fn check(file: &LockFile, catalogue: &Catalogue) -> Result<Self, String> {
        let mut lock = Self {
            profile: file.profile.clone(),
            seed: file.seed.clone(),
            people: BTreeMap::new(),
        };
        for (person, entries) in &file.people {
            let mut pinned = BTreeMap::new();
            for (written, entry) in entries {
                let declaration = catalogue
                    .resolve(written)
                    .map_err(|error| error.to_string())?;
                if declaration.mutability == Mutability::Derived {
                    return Err(format!(
                        "OutOfDomain: '{}' is derived; nothing pins it",
                        declaration.id
                    ));
                }
                let value = decode(&declaration.kind, &entry.value)
                    .map_err(|error| format!("'{}': {error}", declaration.id))?;
                if !is_date(&entry.at) {
                    return Err(format!(
                        "BadDeclaration: '{}' at: {} is not YYYY-MM-DD",
                        declaration.id, entry.at
                    ));
                }
                if entry.by.as_ref().is_some_and(|by| by.len() > 160) {
                    return Err(format!(
                        "BadDeclaration: '{}' by: is at most 160 bytes",
                        declaration.id
                    ));
                }
                pinned.insert(
                    declaration.id.clone(),
                    LockEntry {
                        value,
                        origin: entry.origin,
                        at: entry.at.clone(),
                        by: entry.by.clone(),
                    },
                );
            }
            lock.people.insert(person.clone(), pinned);
        }
        Ok(lock)
    }

    /// The lock in its one canonical form: people and ids sorted, one entry per line, values in the
    /// kind's YAML form. Writing an unchanged lock gives the same bytes.
    pub fn write(&self) -> String {
        let mut out = String::from("format: 1\n");
        if let Some(profile) = &self.profile {
            let _ = writeln!(
                out,
                "profile: {{ id: {}, version: {} }}",
                profile.id, profile.version
            );
        }
        if let Some(seed) = &self.seed {
            let _ = writeln!(out, "seed: {}", quoted(seed));
        }
        out.push_str("people:\n");
        for (person, entries) in &self.people {
            let _ = writeln!(out, "  {person}:");
            for (id, entry) in entries {
                let by = entry
                    .by
                    .as_ref()
                    .map(|by| format!(", by: {}", quoted(by)))
                    .unwrap_or_default();
                let _ = writeln!(
                    out,
                    "    {id}: {{ value: {}, origin: {}, at: \"{}\"{by} }}",
                    yaml_value(&entry.value),
                    entry.origin.as_str(),
                    entry.at
                );
            }
        }
        if self.people.is_empty() {
            out = out.replace("people:\n", "people: {}\n");
        }
        out
    }

    /// FNV-1a 64 of [`Lock::write`]: its identity in `attributes-composed`.
    pub fn digest(&self) -> u64 {
        crate::sample::hash::fnv1a64(self.write().as_bytes())
    }
}

/// A string as a double-quoted YAML scalar.
fn quoted(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_default()
}

/// A value in the form a person file writes it.
fn yaml_value(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Category(slug) => slug.to_string(),
        AttributeValue::Level(level) => level.slug.to_string(),
        AttributeValue::Quantity(quantity) => format!("\"{quantity}\""),
        AttributeValue::Flag(flag) => flag.to_string(),
        AttributeValue::Set(members) => {
            let members: Vec<&str> = members.iter().map(|member| member.as_str()).collect();
            format!("[{}]", members.join(", "))
        }
        AttributeValue::Text(text) => quoted(text.as_str()),
        AttributeValue::Instant(at) => at.seconds().to_string(),
    }
}
