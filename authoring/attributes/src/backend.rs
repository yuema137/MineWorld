//! Person-generation backends (`DECISIONS.md` `ARC-126`; `MODULE_SPEC.md` §4.1 "Backends and masks";
//! `pr-s30e-builtin-catalogue.md` §§5.1, 5.3): the contract a backend pack carries in `backends/<id>.yaml`,
//! and which backend a world uses.
//!
//! ```yaml
//! format: 1
//! backend: mineworld
//! version: 1.0.0
//! license: CC0-1.0
//! attribution: "MineWorld's built-in person attributes"
//! catalogues: [appearance, background]        # this pack's attributes/ files, by stem
//! masks: { default: core, ships: [core, appearance] }
//! sampling: [profile]
//! provenance: { source: mineworld }            # or upstream's: source, version, commit, files
//! keeps: []                                    # `core` ids a foreign backend leaves as MineWorld's
//! mapping: <stem>                              # mappings/<stem>.yaml: views of built-ins (S30-h1)
//! ```
//!
//! An upstream provenance is checked for its shape only (a 40-hex commit, 64-hex SHA-256 digests); the
//! digests themselves are verified by the backend pack's own import check, never at load (D-h-6).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

/// The built-in backend's id.
pub const MINEWORLD: &str = "mineworld";

/// The sampling modes this engine knows (`log-linear` is S30-h2's).
pub const SAMPLING: [&str; 1] = ["profile"];

/// A backend's masks: the default and the ones it ships.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BackendMasks {
    /// The mask a world gets when it names none.
    pub default: String,
    /// The masks this pack ships in `masks/`.
    #[serde(default)]
    pub ships: Vec<String>,
}

/// One `backends/<id>.yaml`, as written. Every field is checked by [`Backend::check`], so each refusal
/// names its field (`BackendMalformed`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BackendFile {
    format: u32,
    backend: String,
    version: String,
    license: String,
    attribution: String,
    catalogues: Vec<String>,
    masks: BackendMasks,
    sampling: Vec<String>,
    #[serde(default)]
    provenance: Provenance,
    #[serde(default)]
    keeps: Vec<String>,
    #[serde(default)]
    mapping: Option<String>,
}

/// Where a backend's catalogue comes from: `{ source: mineworld }`, or an upstream release — its
/// source, version, commit and the SHA-256 of every imported file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provenance {
    /// `mineworld`, or where the upstream lives.
    pub source: String,
    /// The upstream's version.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The upstream commit, 40 hexadecimal digits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    /// Each imported file, by its path in the pack, with its SHA-256 (64 hexadecimal digits).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub files: BTreeMap<String, String>,
}

impl Provenance {
    /// Whether this is MineWorld's own catalogue.
    pub fn is_mineworld(&self) -> bool {
        self.source == MINEWORLD
            && self.version.is_none()
            && self.commit.is_none()
            && self.files.is_empty()
    }

    /// The shape rule: `{ source: mineworld }`, or a source, a version, a 40-hex commit and at least one
    /// file with a 64-hex digest.
    fn check(&self) -> Result<(), String> {
        if self.is_mineworld() {
            return Ok(());
        }
        let hex = |text: &str, length: usize| {
            text.len() == length
                && text
                    .bytes()
                    .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        };
        if self.source.trim().is_empty() || self.source == MINEWORLD {
            return Err(
                "{ source: mineworld } alone, or an upstream source, version, commit and files"
                    .to_owned(),
            );
        }
        if self.version.as_deref().is_none_or(|v| v.trim().is_empty()) {
            return Err("an upstream provenance states its version".to_owned());
        }
        if !self.commit.as_deref().is_some_and(|commit| hex(commit, 40)) {
            return Err("an upstream commit is 40 lower-case hexadecimal digits".to_owned());
        }
        if self.files.is_empty() {
            return Err("an upstream provenance lists the files it imported".to_owned());
        }
        if let Some((file, _)) = self.files.iter().find(|(_, digest)| !hex(digest, 64)) {
            return Err(format!(
                "the digest of '{file}' is not 64 lower-case hexadecimal digits (SHA-256)"
            ));
        }
        Ok(())
    }
}

impl core::fmt::Display for Provenance {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match (&self.version, &self.commit) {
            (Some(version), Some(commit)) => write!(
                f,
                "{} {version} at {commit} ({} files)",
                self.source,
                self.files.len()
            ),
            _ => f.write_str(&self.source),
        }
    }
}

/// A backend, checked.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Backend {
    /// Its id.
    pub id: String,
    /// The pack that carries it.
    pub pack: String,
    /// Its version.
    pub version: String,
    /// Its licence.
    pub license: String,
    /// Its attribution.
    pub attribution: String,
    /// Its `attributes/` files, by stem.
    pub catalogues: BTreeSet<String>,
    /// Its masks.
    pub masks: BackendMasks,
    /// Its sampling modes.
    pub sampling: BTreeSet<String>,
    /// Its provenance.
    pub provenance: Provenance,
    /// The `core` attributes it leaves as MineWorld's (`ARC-126`; Q-S30h-4).
    pub keeps: BTreeSet<crate::id::QualifiedId>,
    /// Its mapping's file stem in `mappings/`, when it maps values onto built-ins.
    pub mapping: Option<String>,
}

/// `BackendMalformed { pack, field }`, with the reason.
fn malformed(pack: &str, field: &str, reason: impl core::fmt::Display) -> String {
    format!("BackendMalformed: the backend of '{pack}', field '{field}': {reason}")
}

impl Backend {
    /// Checks one file of `pack`.
    ///
    /// # Errors
    ///
    /// `BackendMalformed` naming the pack and the field.
    pub fn check(pack: &str, file: &BackendFile) -> Result<Self, String> {
        if file.format != 1 {
            return Err(malformed(
                pack,
                "format",
                format!("{} is not 1", file.format),
            ));
        }
        if !crate::id::is_namespace(&file.backend) {
            return Err(malformed(
                pack,
                "backend",
                "an id of lower-case letters, digits and '-'",
            ));
        }
        for (field, text) in [
            ("version", &file.version),
            ("license", &file.license),
            ("attribution", &file.attribution),
        ] {
            if text.trim().is_empty() {
                return Err(malformed(pack, field, "it is empty"));
            }
        }
        if file.catalogues.is_empty() {
            return Err(malformed(
                pack,
                "catalogues",
                "a backend lists at least one attributes/ file",
            ));
        }
        if !file.masks.ships.contains(&file.masks.default) && file.masks.default != crate::mask::ALL
        {
            return Err(malformed(
                pack,
                "masks",
                format!(
                    "the default '{}' is not among the masks it ships",
                    file.masks.default
                ),
            ));
        }
        if let Some(mode) = file
            .sampling
            .iter()
            .find(|mode| !SAMPLING.contains(&mode.as_str()))
        {
            return Err(malformed(
                pack,
                "sampling",
                format!("'{mode}' is not one of {SAMPLING:?}"),
            ));
        }
        if let Some(stem) = &file.mapping
            && !crate::id::is_namespace(stem)
        {
            return Err(malformed(
                pack,
                "mapping",
                format!("'{stem}' is not a file stem of lower-case letters, digits and '-'"),
            ));
        }
        file.provenance
            .check()
            .map_err(|reason| malformed(pack, "provenance", reason))?;
        let mut keeps = BTreeSet::new();
        for kept in &file.keeps {
            let id = crate::id::QualifiedId::new(kept)
                .map_err(|reason| malformed(pack, "keeps", format!("'{kept}': {reason}")))?;
            if id.namespace() != crate::catalogue::CORE {
                return Err(malformed(
                    pack,
                    "keeps",
                    format!("'{kept}' is not a `core` attribute"),
                ));
            }
            if !keeps.insert(id) {
                return Err(malformed(pack, "keeps", format!("'{kept}' is named twice")));
            }
        }
        Ok(Self {
            id: file.backend.clone(),
            pack: pack.to_owned(),
            version: file.version.clone(),
            license: file.license.clone(),
            attribution: file.attribution.clone(),
            catalogues: file.catalogues.iter().cloned().collect(),
            masks: file.masks.clone(),
            sampling: file.sampling.iter().cloned().collect(),
            provenance: file.provenance.clone(),
            keeps,
            mapping: file.mapping.clone(),
        })
    }

    /// Whether `path` (an `attributes/` file of this backend's pack) is one of its catalogues.
    pub fn lists(&self, path: &Path) -> bool {
        path.file_stem()
            .is_some_and(|stem| self.catalogues.contains(stem.to_string_lossy().as_ref()))
    }
}

/// Which backend a world uses (F-S30e-8): the one it names; else the only one its packs provide; else
/// none.
///
/// # Errors
///
/// `UnknownBackend` when the named one is not provided; `AmbiguousBackend` when several are and none is
/// named; `BackendMalformed` when one pack carries two.
pub fn choose<'b>(
    backends: &'b [(PathBuf, Backend)],
    named: Option<&str>,
) -> Result<Option<&'b Backend>, String> {
    let mut packs = BTreeSet::new();
    for (path, backend) in backends {
        if !packs.insert(backend.pack.as_str()) {
            return Err(format!(
                "BackendMalformed: '{}' carries more than one backends/ file ({})",
                backend.pack,
                path.display()
            ));
        }
    }
    let ids: Vec<&str> = backends
        .iter()
        .map(|(_, backend)| backend.id.as_str())
        .collect();
    match named {
        Some(name) => backends
            .iter()
            .map(|(_, backend)| backend)
            .find(|backend| backend.id == name)
            .map(Some)
            .ok_or_else(|| format!("UnknownBackend: no required pack provides the backend '{name}' (provided: {ids:?})")),
        None => match backends {
            [] => Ok(None),
            [(_, one)] => Ok(Some(one)),
            _ => Err(format!(
                "AmbiguousBackend: the world's packs provide {ids:?}; name one in configure/attributes.yaml: backend: <id>"
            )),
        },
    }
}

/// The most attributes the built-in catalogue may hold (RR-S30-3).
pub const CORE_CAP: usize = 100;

/// The reader codes a `core` reader line begins with (step-30 §6.1; F-S30e-3).
pub const READER_CODES: [&str; 6] = ["C", "OV", "ME", "CV", "DM", "S31"];

/// Whether `why` is a reader line: `<code>(, <code>)*: <text>`.
pub fn is_reader_line(why: &str) -> bool {
    let Some((codes, text)) = why.split_once(": ") else {
        return false;
    };
    !text.trim().is_empty() && codes.split(", ").all(|code| READER_CODES.contains(&code))
}

/// The built-in catalogue check (X-S30-20, `pr-s30e-builtin-catalogue.md` §5.6): `core` holds at most
/// [`CORE_CAP`] attributes, and the `mineworld` backend's default mask names each by id with a reader
/// line. A release check: loading a world is not refused by it.
///
/// # Errors
///
/// Every refusal, each naming what it is about.
pub fn check_core<'c>(
    core: impl IntoIterator<Item = &'c crate::id::QualifiedId>,
    default_mask: &crate::mask::Mask,
) -> Result<usize, Vec<String>> {
    let ids: Vec<&crate::id::QualifiedId> = core.into_iter().collect();
    let mut refusals = Vec::new();
    if ids.len() > CORE_CAP {
        refusals.push(format!(
            "core holds {} attributes; the built-in catalogue is curated to at most {CORE_CAP}",
            ids.len()
        ));
    }
    for id in &ids {
        if !default_mask.why_of(id).is_some_and(is_reader_line) {
            refusals.push(format!(
                "{id} has no reader line in the mask '{}'",
                default_mask.id
            ));
        }
    }
    if refusals.is_empty() {
        Ok(ids.len())
    } else {
        Err(refusals)
    }
}

#[cfg(test)]
mod tests;
