//! Mappings (`DECISIONS.md` `ARC-122`, `ARC-126`; `MODULE_SPEC.md` §4.1 "Backends and masks";
//! the S30-h design §5.7): how a backend's values become a built-in's values. Under the
//! backend that names it, each view puts back a `core` attribute the backend removed, as a derived
//! attribute computed from one of the backend's own attributes by a value table. A view is never stored
//! and never stated.
//!
//! ```yaml
//! format: 1
//! backend: toy
//! views:
//!   core.chronotype:
//!     from: rhythm                       # an attribute of the backend's catalogue
//!     values: { dawn: early-bird, dusk: night-owl, any: neutral }
//! ```
//!
//! The file's shape is checked here; every rule that needs the catalogue (the target removed by the
//! backend and owned by `attributes`, the source declared, the kinds, coverage, the domains, no
//! widening) is checked when the catalogue is composed, each refused by name.

use std::collections::BTreeMap;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::catalogue::CORE;
use crate::id::{QualifiedId, ValueSlug};

/// One view as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawView {
    from: String,
    values: BTreeMap<String, String>,
}

/// One `mappings/<stem>.yaml`, as written.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingFile {
    format: u32,
    backend: String,
    views: BTreeMap<String, RawView>,
}

/// One view, its ids qualified.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct View {
    /// The backend attribute it reads.
    pub from: QualifiedId,
    /// Each source value, with the target value it becomes.
    pub values: BTreeMap<ValueSlug, ValueSlug>,
}

/// A mapping, its shape checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Mapping {
    /// The file it came from, for a refusal.
    pub path: PathBuf,
    /// The backend it is for.
    pub backend: String,
    /// Each target built-in, with its view.
    pub views: BTreeMap<QualifiedId, View>,
}

fn malformed(pack: &str, reason: impl core::fmt::Display) -> String {
    format!("BackendMalformed: the backend of '{pack}', field 'mapping': {reason}")
}

impl Mapping {
    /// Checks one file of `pack` (the backend's pack): its `from` ids are in the pack's namespace (a
    /// short slug is qualified with it), its targets are `core` ids, no target twice.
    ///
    /// # Errors
    ///
    /// `BackendMalformed` naming the field `mapping`; `DuplicateAttribute` when two views share a
    /// target.
    pub fn check(pack: &str, path: PathBuf, file: &MappingFile) -> Result<Self, String> {
        if file.format != 1 {
            return Err(malformed(pack, format!("format {} is not 1", file.format)));
        }
        if file.views.is_empty() {
            return Err(malformed(pack, "a mapping holds at least one view"));
        }
        let mut views = BTreeMap::new();
        for (target, raw) in &file.views {
            let qualified = if target.starts_with(&format!("{CORE}.")) {
                target.clone()
            } else {
                format!("{CORE}.{target}")
            };
            let target_id = QualifiedId::new(&qualified)
                .map_err(|reason| malformed(pack, format!("'{target}': {reason}")))?;
            let own = format!("{pack}.");
            let from = if raw.from.starts_with(&own) {
                raw.from.clone()
            } else {
                format!("{own}{}", raw.from)
            };
            let from = QualifiedId::new(&from)
                .map_err(|reason| malformed(pack, format!("'{}': {reason}", raw.from)))?;
            let mut values = BTreeMap::new();
            for (source, target_value) in &raw.values {
                let source = ValueSlug::new(source)
                    .map_err(|reason| malformed(pack, format!("'{target}': {reason}")))?;
                let target_value = ValueSlug::new(target_value)
                    .map_err(|reason| malformed(pack, format!("'{target}': {reason}")))?;
                values.insert(source, target_value);
            }
            if views
                .insert(target_id.clone(), View { from, values })
                .is_some()
            {
                return Err(format!(
                    "DuplicateAttribute: the mapping of '{pack}' names the target '{target_id}' twice"
                ));
            }
        }
        Ok(Self {
            path,
            backend: file.backend.clone(),
            views,
        })
    }
}

#[cfg(test)]
mod tests;
