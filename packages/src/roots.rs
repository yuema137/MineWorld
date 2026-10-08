//! Pack roots: the directories a world's requirements are searched in, and only those
//! (`DECISIONS.md` `ARC-54` point 3).

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use crate::error::PackageError;
use crate::found::{DataPack, packs_in};

/// The environment variable that names pack roots after the command line's.
pub const PACKS_VARIABLE: &str = "MINEWORLD_PACKS";

/// The directories named by `--packs`, in order, then by [`PACKS_VARIABLE`], in order. Nothing else:
/// not the world's own directory, not the working directory.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PackRoots {
    roots: Vec<PathBuf>,
}

impl PackRoots {
    /// No pack root: a world is resolved against the build alone.
    #[must_use]
    pub fn none() -> Self {
        Self::default()
    }

    /// The command line's directories, then the environment's path list (empty entries skipped). The
    /// caller reads the environment, so this is a pure function of both.
    ///
    /// # Errors
    ///
    /// [`PackageError::NoSuchRoot`] for the first entry that is not a directory, naming where it came
    /// from.
    pub fn new(
        command_line: Vec<PathBuf>,
        environment: Option<&OsStr>,
    ) -> Result<Self, PackageError> {
        let from_environment = environment
            .map(|list| std::env::split_paths(list).collect::<Vec<_>>())
            .unwrap_or_default();
        let mut roots = Vec::new();
        for (paths, source_name) in [
            (command_line, "--packs"),
            (from_environment, PACKS_VARIABLE),
        ] {
            for path in paths
                .into_iter()
                .filter(|path| !path.as_os_str().is_empty())
            {
                if !path.is_dir() {
                    return Err(PackageError::NoSuchRoot { path, source_name });
                }
                roots.push(path);
            }
        }
        Ok(Self { roots })
    }

    /// The roots, in search order.
    pub fn dirs(&self) -> impl Iterator<Item = &Path> {
        self.roots.iter().map(PathBuf::as_path)
    }

    /// Every data pack directly inside every root, root by root.
    ///
    /// # Errors
    ///
    /// What [`packs_in`] refuses.
    pub fn packs(&self) -> Result<Vec<DataPack>, PackageError> {
        let mut found = Vec::new();
        for root in &self.roots {
            found.extend(packs_in(root)?);
        }
        Ok(found)
    }

    /// Where a requirement was searched for, for a refusal: the build, then each root — or that no pack
    /// directory was given.
    #[must_use]
    pub fn searched(&self) -> String {
        if self.roots.is_empty() {
            return "this build, and no pack directory was given (--packs, MINEWORLD_PACKS)"
                .to_owned();
        }
        let dirs: Vec<String> = self.roots.iter().map(|d| d.display().to_string()).collect();
        format!("this build or {}", dirs.join(", "))
    }
}
