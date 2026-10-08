//! The validated parts of a package identity, and the identity itself (`ARC-53`).

use std::fmt;

use serde::Deserialize;

use crate::declared::Package;
use crate::error::PackageError;

/// A pack's id: 1–64 characters of `a`–`z`, `0`–`9` and `-`, beginning with a letter, with no `--`
/// and no trailing `-`. Every Cargo package name and every World Pack id MineWorld ships is one.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct PackId(String);

impl PackId {
    /// Checks `text` against the rule.
    ///
    /// # Errors
    ///
    /// [`PackageError::Id`], naming which part of the rule `text` breaks.
    pub fn new(text: &str) -> Result<Self, PackageError> {
        let refuse = |why| PackageError::Id {
            text: text.to_owned(),
            why,
        };
        if text.is_empty() || text.len() > 64 {
            return Err(refuse("it must be 1–64 characters"));
        }
        if !text.starts_with(|c: char| c.is_ascii_lowercase()) {
            return Err(refuse("it must begin with a lowercase letter"));
        }
        if !text
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        {
            return Err(refuse("only a–z, 0–9 and '-' are allowed"));
        }
        if text.contains("--") || text.ends_with('-') {
            return Err(refuse("'-' separates words: no '--' and no trailing '-'"));
        }
        Ok(Self(text.to_owned()))
    }

    /// The id.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for PackId {
    type Error = PackageError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(&text)
    }
}

impl fmt::Display for PackId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// What kind of pack this is (`MODULE_SPEC.md` §1). An Asset Pack is a type of the model, and is
/// refused in MVP-0 rather than accepted unread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub enum PackType {
    /// The rules of the world: a code pack identified by its `Cargo.toml`.
    SystemPack,
    /// What decides a Person's actions: a code pack identified by its `Cargo.toml`.
    ControllerPack,
    /// A world: identified by its `world.yaml`.
    WorldPack,
    /// How a world looks: identified by its `pack.yaml`.
    PresentationPack,
    /// What kinds of thing can exist: identified by its `pack.yaml` (read from S16's PR E-d).
    EntityPack,
}

impl PackType {
    /// The type as a manifest writes it.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::SystemPack => "system-pack",
            Self::ControllerPack => "controller-pack",
            Self::WorldPack => "world-pack",
            Self::PresentationPack => "presentation-pack",
            Self::EntityPack => "entity-pack",
        }
    }
}

impl TryFrom<String> for PackType {
    type Error = PackageError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Ok(match text.as_str() {
            "system-pack" => Self::SystemPack,
            "controller-pack" => Self::ControllerPack,
            "world-pack" => Self::WorldPack,
            "presentation-pack" => Self::PresentationPack,
            "entity-pack" => Self::EntityPack,
            "asset-pack" => {
                return Err(PackageError::Type {
                    text,
                    why: "Asset Packs are not read in MVP-0 (PACKAGE_FORMAT.md §8)",
                });
            }
            _ => {
                return Err(PackageError::Type {
                    text,
                    why: "one of system-pack, controller-pack, world-pack, presentation-pack, \
                          entity-pack",
                });
            }
        })
    }
}

impl fmt::Display for PackType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// A semver version, with Cargo's meaning (`DEP-21`).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
#[serde(try_from = "String")]
pub struct Version(semver::Version);

impl Version {
    /// Parses `text`.
    ///
    /// # Errors
    ///
    /// [`PackageError::Version`], naming the text.
    pub fn new(text: &str) -> Result<Self, PackageError> {
        semver::Version::parse(text)
            .map(Self)
            .map_err(|error| PackageError::Version {
                text: text.to_owned(),
                reason: error.to_string(),
            })
    }
}

impl TryFrom<String> for Version {
    type Error = PackageError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(&text)
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// The framework's version, [`FRAMEWORK_VERSION`](crate::FRAMEWORK_VERSION), as a [`Version`].
///
/// # Panics
///
/// Never: Cargo refuses a crate whose version is not semver.
#[must_use]
pub fn framework_version() -> Version {
    Version::new(crate::FRAMEWORK_VERSION).expect("Cargo versions are semver")
}

/// A semver range — the framework versions a pack works with — with Cargo's meaning: `^0.1` admits
/// 0.1.x and nothing else, as it does in a `Cargo.toml` (`DEP-21`).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct Compatibility {
    text: String,
    range: semver::VersionReq,
}

impl Compatibility {
    /// Parses `text`.
    ///
    /// # Errors
    ///
    /// [`PackageError::Range`], naming the text.
    pub fn new(text: &str) -> Result<Self, PackageError> {
        semver::VersionReq::parse(text)
            .map(|range| Self {
                text: text.to_owned(),
                range,
            })
            .map_err(|error| PackageError::Range {
                text: text.to_owned(),
                reason: error.to_string(),
            })
    }

    /// Whether `version` is in the range.
    #[must_use]
    pub fn admits(&self, version: &Version) -> bool {
        self.range.matches(&version.0)
    }

    /// Refuses a range the running framework is not in.
    ///
    /// # Errors
    ///
    /// [`PackageError::FrameworkNotSupported`], naming the range and the framework's version.
    pub fn require_framework(&self) -> Result<(), PackageError> {
        if self.admits(&framework_version()) {
            Ok(())
        } else {
            Err(PackageError::FrameworkNotSupported {
                range: self.text.clone(),
                framework: crate::FRAMEWORK_VERSION.to_owned(),
            })
        }
    }
}

impl TryFrom<String> for Compatibility {
    type Error = PackageError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(&text)
    }
}

impl fmt::Display for Compatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.text)
    }
}

/// An SPDX licence expression, parsed strictly against the SPDX licence list (`DEP-21`). Held as it
/// was written: whether its licences are ones MineWorld may redistribute is a policy over it, not part
/// of it.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(try_from = "String")]
pub struct License(String);

impl License {
    /// Parses `text`.
    ///
    /// # Errors
    ///
    /// [`PackageError::License`], naming the text — an empty one included.
    pub fn new(text: &str) -> Result<Self, PackageError> {
        match spdx::Expression::parse(text) {
            Ok(_) => Ok(Self(text.to_owned())),
            Err(error) => Err(PackageError::License {
                text: text.to_owned(),
                reason: error.reason.to_string(),
            }),
        }
    }

    /// The expression, as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for License {
    type Error = PackageError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        Self::new(&text)
    }
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A pack's validated package identity (`ARC-53`), whichever carrier stated it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Identity {
    /// The pack's id.
    pub id: PackId,
    /// What kind of pack it is.
    pub kind: PackType,
    /// Its release version.
    pub version: Version,
    /// Its licence.
    pub license: License,
    /// Who made it; empty only for a World Pack, whose carrier has no authors field.
    pub authors: Vec<String>,
    /// Where it comes from, when stated.
    pub repository: Option<String>,
    /// The framework versions it works with — stated by a data pack; a code pack's is its Cargo
    /// requirement on the framework, which Cargo enforces at build time.
    pub mineworld: Option<Compatibility>,
}

impl Identity {
    /// The identity a code pack's [`Package`] states, checked.
    ///
    /// # Errors
    ///
    /// [`PackageError::CodePack`], naming the pack and what is wrong: an id outside the rule, a
    /// licence that is not an SPDX expression (an absent one included), or no author.
    pub fn of_code_pack(package: &Package, kind: PackType) -> Result<Self, PackageError> {
        let checked = || -> Result<Self, PackageError> {
            let authors: Vec<String> = package
                .authors()
                .split(':')
                .map(str::trim)
                .filter(|author| !author.is_empty())
                .map(str::to_owned)
                .collect();
            if authors.is_empty() {
                return Err(PackageError::NoAuthors);
            }
            Ok(Self {
                id: PackId::new(package.name())?,
                kind,
                version: Version::new(package.version())?,
                license: License::new(package.license())?,
                authors,
                repository: Some(package.repository())
                    .filter(|repository| !repository.is_empty())
                    .map(str::to_owned),
                mineworld: None,
            })
        };
        checked().map_err(|problem| PackageError::CodePack {
            pack: package.name().to_owned(),
            problem: Box::new(problem),
        })
    }

    /// The identity a World Pack's `world.yaml` states, each field already typed by the loader —
    /// which leaves them optional — and required here.
    ///
    /// # Errors
    ///
    /// [`PackageError::Id`] for an id outside the rule; [`PackageError::Missing`] naming the first
    /// absent field (`world.version`, `world.license`, `mineworld`), and `manifest` as its file.
    pub fn of_world(
        manifest: &std::path::Path,
        id: &str,
        version: Option<Version>,
        license: Option<License>,
        mineworld: Option<Compatibility>,
    ) -> Result<Self, PackageError> {
        let missing = |field| PackageError::Missing {
            path: manifest.to_path_buf(),
            field,
        };
        Ok(Self {
            id: PackId::new(id)?,
            kind: PackType::WorldPack,
            version: version.ok_or_else(|| missing("world.version"))?,
            license: license.ok_or_else(|| missing("world.license"))?,
            authors: Vec::new(),
            repository: None,
            mineworld: Some(mineworld.ok_or_else(|| missing("mineworld"))?),
        })
    }
}

/// Refuses two packs with one id, naming both origins. `packs` is each identity with where it came
/// from, in the order it was found.
///
/// # Errors
///
/// [`PackageError::DuplicateId`] for the first id found twice.
pub fn distinct<'a>(
    packs: impl IntoIterator<Item = (&'a Identity, &'a str)>,
) -> Result<(), PackageError> {
    let mut seen: std::collections::BTreeMap<&PackId, &str> = std::collections::BTreeMap::new();
    for (identity, origin) in packs {
        if let Some(first) = seen.insert(&identity.id, origin) {
            return Err(PackageError::DuplicateId {
                id: identity.id.to_string(),
                first: first.to_owned(),
                second: origin.to_owned(),
            });
        }
    }
    Ok(())
}
