//! A world's requirements, gathered for the resolver (`DECISIONS.md` `ARC-54`).
//!
//! This module only gathers: what the world states, what this build holds (through the installed set,
//! naming no pack) and what the pack roots hold. What is allowed is `mineworld_packages::resolve`'s
//! and the licence policy's. Nothing here is world state: no genesis fact is seeded from it, and the
//! loader that builds a world never reads it.

use std::path::Path;

use mineworld_packages::{
    CodePack, Composition, DataPack, FoundPack, Identity, Installed, LicencePolicy, PackRoots,
    PackType, PackageError, WorldRequirements, read_pack_file,
};

use crate::catalog::{AVAILABLE, Capability};
use crate::error::PackError;
use crate::format::WorldManifest;
use crate::read::parse;

/// Resolves the world `manifest` (read from `manifest_path`) against this build and `roots`, under the
/// world's licence policy — the default, or the world's `configure/packages.yaml` (`ARC-55` note).
pub(crate) fn resolve(
    manifest_path: &Path,
    manifest: &WorldManifest,
    systems: &[Capability],
    roots: &PackRoots,
    policy: &LicencePolicy,
) -> Result<Composition, PackError> {
    let refused = |refusal| PackError::Requirements {
        path: manifest_path.to_path_buf(),
        refusal: Box::new(refusal),
    };
    let build = build().map_err(refused)?;
    let found = found(roots).map_err(|refusal| match refusal {
        Found::Package(refusal) => refused(refusal),
        Found::World(refusal) => *refusal,
    })?;
    let enabled: Vec<String> = systems.iter().map(|c| c.id().to_string()).collect();
    let enabled: Vec<&str> = enabled.iter().map(String::as_str).collect();
    let searched = roots.searched();
    mineworld_packages::resolve(
        &WorldRequirements {
            id: &manifest.world.id,
            license: manifest.world.license.as_ref(),
            mineworld: manifest.mineworld.as_ref(),
            requires: &manifest.requires,
        },
        &enabled,
        &Installed {
            build: &build,
            found: &found,
            searched: &searched,
        },
        policy,
    )
    .map_err(refused)
}

/// Every System Pack this build installs, with its identity and whether it is bundled.
fn build() -> Result<Vec<CodePack>, PackageError> {
    AVAILABLE
        .iter()
        .map(|capability| {
            let package = capability.package();
            Ok(CodePack {
                identity: Identity::of_code_pack(&package, PackType::SystemPack)?,
                bundled: package.bundled(),
                system: capability.id().to_string(),
            })
        })
        .collect()
}

/// Every data pack directly inside the roots. A World Pack found there is identified from its
/// `world.yaml` alone — never read as a world, so a world in a root is never resolved, and no
/// resolution can recurse.
fn found(roots: &PackRoots) -> Result<Vec<FoundPack>, Found> {
    let mut found = Vec::new();
    for pack in roots.packs().map_err(Found::Package)? {
        let identity = match &pack {
            DataPack::PackFile(dir) => read_pack_file(dir).map_err(Found::Package)?,
            DataPack::World(dir) => {
                let path = dir.join(crate::read::MANIFEST);
                let world: WorldManifest =
                    parse(&path, "world").map_err(|refusal| Found::World(Box::new(refusal)))?;
                Identity::of_world(
                    &path,
                    &world.world.id,
                    world.world.version,
                    world.world.license,
                    world.mineworld,
                )
                .map_err(Found::Package)?
            }
        };
        found.push(FoundPack {
            identity,
            dir: pack.dir().to_path_buf(),
        });
    }
    Ok(found)
}

/// Why a pack in a root could not be identified: a package fact, or a `world.yaml` that does not parse
/// (refused as the loader refuses any world file, with its line and column).
enum Found {
    Package(PackageError),
    World(Box<PackError>),
}
