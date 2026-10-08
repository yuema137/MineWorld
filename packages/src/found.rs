//! Which directories are data packs: only those given, and only by their manifest (`ARC-53`).

use std::path::{Path, PathBuf};

use crate::error::PackageError;
use crate::manifest::PACK_FILE;

/// The manifest a World Pack is identified by.
pub const WORLD_FILE: &str = "world.yaml";

/// A directory that is a data pack, by the manifest it holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataPack {
    /// It holds `world.yaml`: the World Pack loader reads it.
    World(PathBuf),
    /// It holds `pack.yaml`: this crate reads it.
    PackFile(PathBuf),
}

impl DataPack {
    /// The pack's directory.
    #[must_use]
    pub fn dir(&self) -> &Path {
        match self {
            Self::World(dir) | Self::PackFile(dir) => dir,
        }
    }

    /// What `dir` is: a World Pack, a `pack.yaml` pack, or not a pack (`None`).
    ///
    /// # Errors
    ///
    /// [`PackageError::TwoManifests`] when it holds both.
    pub fn of(dir: &Path) -> Result<Option<Self>, PackageError> {
        let world = dir.join(WORLD_FILE).is_file();
        let pack = dir.join(PACK_FILE).is_file();
        match (world, pack) {
            (true, true) => Err(PackageError::TwoManifests {
                dir: dir.to_path_buf(),
            }),
            (true, false) => Ok(Some(Self::World(dir.to_path_buf()))),
            (false, true) => Ok(Some(Self::PackFile(dir.to_path_buf()))),
            (false, false) => Ok(None),
        }
    }
}

/// The data packs directly inside `root`, by directory name. A subdirectory holding no manifest is
/// not a pack (a `LICENSES/` beside the packs); a file is not a pack. Nothing below the first level is
/// read, and nothing outside `root`.
///
/// # Errors
///
/// [`PackageError::RootIsAPack`] when `root` itself holds a manifest; [`PackageError::Unreadable`]
/// when it cannot be listed; [`PackageError::TwoManifests`] for a subdirectory holding both.
pub fn packs_in(root: &Path) -> Result<Vec<DataPack>, PackageError> {
    for file in [WORLD_FILE, PACK_FILE] {
        if root.join(file).is_file() {
            return Err(PackageError::RootIsAPack {
                dir: root.to_path_buf(),
                file,
            });
        }
    }
    let unreadable = |error: std::io::Error| PackageError::Unreadable {
        path: root.to_path_buf(),
        reason: error.to_string(),
    };
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(root).map_err(unreadable)? {
        let path = entry.map_err(unreadable)?.path();
        if path.is_dir() {
            dirs.push(path);
        }
    }
    dirs.sort();
    let mut packs = Vec::new();
    for dir in dirs {
        if let Some(pack) = DataPack::of(&dir)? {
            packs.push(pack);
        }
    }
    Ok(packs)
}
