//! Masks (`DECISIONS.md` `ARC-126`; `MODULE_SPEC.md` §4.1 "Backends and masks";
//! `pr-s30e-builtin-catalogue.md` §5.2): which of a backend's attributes exist in a world. What a mask
//! does not keep does not exist — the catalogue is frozen after it.
//!
//! ```yaml
//! format: 1
//! mask: core
//! version: 1
//! include:                       # absent: everything
//!   - { id: core.stature, why: "C, OV: height in both clients; tall" }
//!   - { group: appearance, why: "…" }
//!   - { facet: casting, why: "…" }
//! exclude: []
//! ```

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::declaration::Declaration;
use crate::id::QualifiedId;

/// The built-in mask that keeps everything.
pub const ALL: &str = "all";

/// What one entry selects.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Selector {
    /// One attribute.
    Id(QualifiedId),
    /// Every attribute of a group.
    Group(String),
    /// Every attribute carrying a facet.
    Facet(String),
}

/// One mask entry as written: exactly one of `id`, `group`, `facet`, and a `why`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawEntry {
    #[serde(default)]
    id: Option<String>,
    #[serde(default)]
    group: Option<String>,
    #[serde(default)]
    facet: Option<String>,
    #[serde(default)]
    why: String,
}

/// One `masks/<id>.yaml`, as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskFile {
    format: u32,
    mask: String,
    version: u32,
    #[serde(default)]
    include: Option<Vec<RawEntry>>,
    #[serde(default)]
    exclude: Vec<RawEntry>,
}

/// One entry, checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Entry {
    /// What it selects.
    pub selector: Selector,
    /// Why: for a `core` entry of the default mask, the reader line (F-S30e-3).
    pub why: String,
}

/// A mask, checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mask {
    /// Its id.
    pub id: String,
    /// Its version.
    pub version: u32,
    /// What it keeps; `None` keeps everything.
    pub include: Option<Vec<Entry>>,
    /// What it then drops.
    pub exclude: Vec<Entry>,
}

fn entry(mask: &str, raw: &RawEntry) -> Result<Entry, String> {
    let selector = match (&raw.id, &raw.group, &raw.facet) {
        (Some(id), None, None) => Selector::Id(
            QualifiedId::new(id)
                .map_err(|reason| format!("BadMask: mask '{mask}': '{id}': {reason}"))?,
        ),
        (None, Some(group), None) => Selector::Group(group.clone()),
        (None, None, Some(facet)) => Selector::Facet(facet.clone()),
        _ => {
            return Err(format!(
                "BadMask: mask '{mask}': an entry names exactly one of id, group or facet"
            ));
        }
    };
    if raw.why.trim().is_empty() {
        return Err(format!(
            "MaskEntryUnjustified: mask '{mask}': the entry {selector:?} gives no why"
        ));
    }
    Ok(Entry {
        selector,
        why: raw.why.clone(),
    })
}

impl Mask {
    /// The built-in `all`.
    pub fn all() -> Self {
        Self {
            id: ALL.to_owned(),
            version: 1,
            include: None,
            exclude: Vec::new(),
        }
    }

    /// Checks one file.
    ///
    /// # Errors
    ///
    /// `BadMask` or `MaskEntryUnjustified`, naming the mask.
    pub fn check(file: &MaskFile) -> Result<Self, String> {
        if file.format != 1 {
            return Err(format!(
                "BadMask: mask '{}': format {} is not 1",
                file.mask, file.format
            ));
        }
        if file.mask == ALL || !crate::id::is_namespace(&file.mask) {
            return Err(format!(
                "BadMask: '{}' is not a mask id a file may declare",
                file.mask
            ));
        }
        let include = file
            .include
            .as_ref()
            .map(|entries| {
                entries
                    .iter()
                    .map(|raw| entry(&file.mask, raw))
                    .collect::<Result<Vec<_>, _>>()
            })
            .transpose()?;
        let exclude = file
            .exclude
            .iter()
            .map(|raw| entry(&file.mask, raw))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Self {
            id: file.mask.clone(),
            version: file.version,
            include,
            exclude,
        })
    }

    /// Every id an entry names that `known` does not hold.
    pub fn unknown_ids<'m>(&'m self, known: &BTreeSet<QualifiedId>) -> Vec<&'m QualifiedId> {
        self.include
            .iter()
            .flatten()
            .chain(&self.exclude)
            .filter_map(|entry| match &entry.selector {
                Selector::Id(id) if !known.contains(id) => Some(id),
                _ => None,
            })
            .collect()
    }

    /// Whether the mask keeps `declaration`.
    pub fn keeps(&self, declaration: &Declaration) -> bool {
        let selects = |entry: &Entry| match &entry.selector {
            Selector::Id(id) => *id == declaration.id,
            Selector::Group(group) => *group == declaration.group,
            Selector::Facet(facet) => declaration.facets.contains(facet),
        };
        let included = self
            .include
            .as_ref()
            .is_none_or(|entries| entries.iter().any(selects));
        included && !self.exclude.iter().any(selects)
    }

    /// The `why` of the entry naming `id` by id in `include`, if any.
    pub fn why_of(&self, id: &QualifiedId) -> Option<&str> {
        self.include
            .iter()
            .flatten()
            .find_map(|entry| match &entry.selector {
                Selector::Id(named) if named == id => Some(entry.why.as_str()),
                _ => None,
            })
    }
}

#[cfg(test)]
mod tests;
