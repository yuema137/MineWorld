//! The bundle as the launcher reads it (`LAUNCHER.md` §4): every path into it is named here, and only
//! here, so that a change of the assembled layout is a change to this file.

use std::path::{Path, PathBuf};

use crate::Failure;
use crate::config::WorldName;

/// A bundle root whose `runtime/` holds the server and the Godot runtime.
#[derive(Debug)]
pub struct Bundle {
    runtime: PathBuf,
}

/// The two exported clients.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Client {
    TwoD,
    ThreeD,
}

impl Client {
    /// The short name used in the pack's file name and in log names: `2d`, `3d`.
    pub fn label(self) -> &'static str {
        match self {
            Self::TwoD => "2d",
            Self::ThreeD => "3d",
        }
    }
}

impl Bundle {
    /// The bundle at `root`, or at the root found from this executable's path (`LAUNCHER.md` §4), checked
    /// to hold the server and the Godot runtime.
    pub fn locate(root: Option<PathBuf>) -> Result<Self, Failure> {
        let root = match root {
            Some(root) => root,
            None => {
                let exe = std::env::current_exe().map_err(|error| {
                    Failure::new(format!("cannot tell where this launcher is: {error}"))
                })?;
                root_of(&exe)
            }
        };
        let root = std::path::absolute(&root)
            .map_err(|error| Failure::new(format!("cannot resolve {}: {error}", root.display())))?;
        let bundle = Self {
            runtime: root.join("runtime"),
        };
        for needed in [bundle.server(), bundle.engine()] {
            exists(&needed)?;
        }
        Ok(bundle)
    }

    /// `runtime/`, which the clients are told is their root (`--root=`, step-23 §6.5).
    pub fn runtime(&self) -> &Path {
        &self.runtime
    }

    /// The server binary.
    pub fn server(&self) -> PathBuf {
        self.runtime
            .join(format!("mineworld{}", std::env::consts::EXE_SUFFIX))
    }

    /// The Godot runtime the exported clients run in.
    pub fn engine(&self) -> PathBuf {
        let godot = self.runtime.join("godot");
        if cfg!(target_os = "macos") {
            godot.join("Godot.app/Contents/MacOS/Godot")
        } else if cfg!(windows) {
            godot.join("godot.exe")
        } else {
            godot.join("godot")
        }
    }

    /// A world's directory, checked to exist.
    pub fn world(&self, world: &WorldName) -> Result<PathBuf, Failure> {
        let path = self.runtime.join("worlds").join(world.as_str());
        exists(&path)?;
        Ok(path)
    }

    /// A client's exported pack, checked to exist.
    pub fn pack(&self, client: Client) -> Result<PathBuf, Failure> {
        let path = self
            .runtime
            .join("clients")
            .join(format!("{}.pck", client.label()));
        exists(&path)?;
        Ok(path)
    }

    /// The optional overrides file (`LAUNCHER.md` §6).
    pub fn overrides(&self) -> PathBuf {
        self.runtime.join("launch.toml")
    }
}

fn exists(path: &Path) -> Result<(), Failure> {
    if path.exists() {
        Ok(())
    } else {
        Err(Failure::new(format!(
            "this bundle is incomplete: {} is missing",
            path.display()
        )))
    }
}

/// The bundle root for an executable at `exe`: its folder, or the folder that holds the `.app` when
/// `exe` is `<name>.app/Contents/MacOS/<exe>`.
pub fn root_of(exe: &Path) -> PathBuf {
    let folder = exe.parent().unwrap_or(Path::new("."));
    let app = folder
        .parent()
        .filter(|_| folder.file_name().is_some_and(|name| name == "MacOS"))
        .filter(|contents| contents.file_name().is_some_and(|name| name == "Contents"))
        .and_then(Path::parent)
        .filter(|app| app.extension().is_some_and(|extension| extension == "app"));
    match app.and_then(Path::parent) {
        Some(holder) => holder.to_path_buf(),
        None => folder.to_path_buf(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_root_is_the_executables_folder_or_the_folder_holding_its_app() {
        let plain = Path::new("/x/MineWorld-0.1.0/MineWorld 2D.exe");
        assert_eq!(root_of(plain), Path::new("/x/MineWorld-0.1.0"));

        let app =
            Path::new("/x/MineWorld 世界/MineWorld 3D.app/Contents/MacOS/mineworld-3d-launch");
        assert_eq!(root_of(app), Path::new("/x/MineWorld 世界"));

        // A MacOS/Contents chain that is not inside an .app is just a folder.
        let lookalike = Path::new("/x/Contents/MacOS/launch");
        assert_eq!(root_of(lookalike), Path::new("/x/Contents/MacOS"));
    }
}
