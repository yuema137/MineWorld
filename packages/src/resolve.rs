//! A world's requirements, resolved: checked against what is installed, never chosen
//! (`DECISIONS.md` `ARC-54` point 4).
//!
//! A pure function of what the world states, what the build holds, what the pack roots hold and the
//! licence policy. The World Pack loader gathers those and calls [`resolve`]; nothing here reads a
//! file, the environment or a world's content.

use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::error::PackageError;
use crate::identity::{Compatibility, Identity, License, PackId, PackType, Version, distinct};
use crate::policy::LicencePolicy;

/// What a world states about its composition.
#[derive(Debug, Clone, Copy)]
pub struct WorldRequirements<'a> {
    /// The world's id.
    pub id: &'a str,
    /// The world's licence, when stated.
    pub license: Option<&'a License>,
    /// The world's `mineworld:` range, when stated (already checked against the framework).
    pub mineworld: Option<&'a Compatibility>,
    /// `requires:`, in id order.
    pub requires: &'a BTreeMap<PackId, Compatibility>,
}

/// A code pack this build holds: its identity, whether it is bundled, and its system's id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodePack {
    /// The pack's identity.
    pub identity: Identity,
    /// Compiled from the framework's own workspace.
    pub bundled: bool,
    /// The system it provides.
    pub system: String,
}

/// A data pack found in a pack root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundPack {
    /// The pack's identity.
    pub identity: Identity,
    /// Its directory.
    pub dir: PathBuf,
}

/// Where a resolved pack comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// This build; `bundled` or third-party.
    Build {
        /// Compiled from the framework's own workspace.
        bundled: bool,
    },
    /// A pack root's subdirectory.
    Directory(PathBuf),
}

/// One requirement and the pack that met it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolved {
    /// The range required.
    pub range: Compatibility,
    /// The pack that met it.
    pub identity: Identity,
    /// Where it was found.
    pub source: Source,
}

/// One enabled system and its pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SystemPackUsed {
    /// The system's id.
    pub system: String,
    /// Its pack's identity.
    pub identity: Identity,
    /// Compiled from the framework's own workspace.
    pub bundled: bool,
}

/// A world's resolved composition. Never world state: printed, never recorded (`ARC-54` point 6).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composition {
    /// The framework's version.
    pub framework: Version,
    /// The world's `mineworld:` range, when stated.
    pub mineworld: Option<Compatibility>,
    /// Each requirement, in id order, with what met it.
    pub required: Vec<Resolved>,
    /// Each enabled system, in the world's order, with its pack.
    pub systems: Vec<SystemPackUsed>,
}

/// The inputs a resolution reads besides the world's own statements.
#[derive(Debug, Clone, Copy)]
pub struct Installed<'a> {
    /// Every code pack this build holds.
    pub build: &'a [CodePack],
    /// Every data pack found in the pack roots.
    pub found: &'a [FoundPack],
    /// Where a requirement was searched, for a refusal.
    pub searched: &'a str,
}

/// Resolves `world` against `installed` under `policy`, refusing the first failure by name, in the
/// order of `ARC-54` point 4: a duplicate id; per requirement, in id order, absent, a type a world
/// cannot require, bundled, out of range, a data pack whose own `mineworld:` range excludes the
/// framework (`ARC-54` note); an enabled third-party system not required; a licence outside the policy.
/// `enabled` is the world's `systems`, in its order.
///
/// # Errors
///
/// The [`PackageError`] naming the first failure.
pub fn resolve(
    world: &WorldRequirements<'_>,
    enabled: &[&str],
    installed: &Installed<'_>,
    policy: &LicencePolicy,
) -> Result<Composition, PackageError> {
    let origins: Vec<String> = installed
        .build
        .iter()
        .map(|pack| format!("this build (system {})", pack.system))
        .chain(
            installed
                .found
                .iter()
                .map(|pack| pack.dir.display().to_string()),
        )
        .collect();
    distinct(
        installed
            .build
            .iter()
            .map(|pack| &pack.identity)
            .chain(installed.found.iter().map(|pack| &pack.identity))
            .zip(origins.iter().map(String::as_str)),
    )?;

    let mut required = Vec::new();
    for (id, range) in world.requires {
        required.push(requirement(id, range, installed)?);
    }

    let mut systems = Vec::new();
    for system in enabled {
        let Some(pack) = installed.build.iter().find(|pack| pack.system == *system) else {
            continue;
        };
        if !pack.bundled && !world.requires.contains_key(&pack.identity.id) {
            return Err(PackageError::ThirdPartyNotRequired {
                system: (*system).to_owned(),
                pack: pack.identity.id.to_string(),
            });
        }
        systems.push(SystemPackUsed {
            system: (*system).to_owned(),
            identity: pack.identity.clone(),
            bundled: pack.bundled,
        });
    }

    if let Some(license) = world.license {
        policy.judge(world.id, license)?;
    }
    for pack in required
        .iter()
        .map(|r| &r.identity)
        .chain(systems.iter().map(|s| &s.identity))
    {
        policy.judge(pack.id.as_str(), &pack.license)?;
    }

    Ok(Composition {
        framework: crate::framework_version(),
        mineworld: world.mineworld.cloned(),
        required,
        systems,
    })
}

/// One requirement: found, of a type a world may require, not bundled, in range, and — for a data
/// pack — stating a framework range this framework is in.
fn requirement(
    id: &PackId,
    range: &Compatibility,
    installed: &Installed<'_>,
) -> Result<Resolved, PackageError> {
    let (identity, source) = installed
        .build
        .iter()
        .find(|pack| &pack.identity.id == id)
        .map(|pack| {
            (
                &pack.identity,
                Source::Build {
                    bundled: pack.bundled,
                },
            )
        })
        .or_else(|| {
            installed
                .found
                .iter()
                .find(|pack| &pack.identity.id == id)
                .map(|pack| (&pack.identity, Source::Directory(pack.dir.clone())))
        })
        .ok_or_else(|| PackageError::Absent {
            id: id.to_string(),
            searched: installed.searched.to_owned(),
        })?;
    let wrong = |why| PackageError::WrongType {
        id: id.to_string(),
        kind: identity.kind.to_string(),
        why,
    };
    match identity.kind {
        PackType::WorldPack => return Err(wrong("a world is not a part of another world")),
        PackType::ControllerPack => {
            return Err(wrong(
                "controllers are chosen by the host, not by the world (S10)",
            ));
        }
        PackType::SystemPack if matches!(source, Source::Build { bundled: true }) => {
            return Err(PackageError::BundledRequired { id: id.to_string() });
        }
        PackType::SystemPack | PackType::PresentationPack | PackType::EntityPack => {}
    }
    if !range.admits(&identity.version) {
        return Err(PackageError::OutOfRange {
            id: id.to_string(),
            range: range.to_string(),
            found: identity.version.to_string(),
        });
    }
    // A data pack states the frameworks it works with; one this framework is not in is refused like the
    // world's own range (`ARC-54` note, F-Ed1). A code pack states none.
    identity.require_framework()?;
    Ok(Resolved {
        range: range.clone(),
        identity: identity.clone(),
        source,
    })
}
