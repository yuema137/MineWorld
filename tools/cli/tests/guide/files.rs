//! The paths a guide page names, and the writes they lead to (`docs/GUIDE_FORMAT.md` §3.2).
//!
//! A page is text anyone can edit, so a path in it is untrusted: it must stay inside the page's scratch
//! folder, on every platform, before anything is written.

use std::fs;
use std::path::{Path, PathBuf};

/// The path `relative` names inside a scratch folder, or the reason it names none (§3.2).
pub fn confine(relative: &str) -> Result<PathBuf, String> {
    let refused = |reason: &str| format!("path {relative:?} is refused: {reason}");
    if relative.is_empty() {
        return Err(refused("it is empty"));
    }
    if relative.contains('\\') {
        return Err(refused("paths use '/' as their only separator"));
    }
    if relative.contains(':') {
        return Err(refused("a colon can name a drive or an alternate stream"));
    }
    if relative.starts_with('/') {
        return Err(refused("it is absolute"));
    }
    let mut path = PathBuf::new();
    for component in relative.split('/') {
        match component {
            "" => return Err(refused("it has an empty component")),
            "." | ".." => return Err(refused("'.' and '..' are not allowed")),
            name => path.push(name),
        }
    }
    Ok(path)
}

/// Writes `text` to `relative` inside `scratch`, creating its parent folders and replacing a file that
/// is there. A symbolic link on the way is refused rather than followed: it could lead outside (§3.2).
pub fn write(scratch: &Path, relative: &str, text: &str) -> Result<(), String> {
    let relative = confine(relative)?;
    let mut target = scratch.to_path_buf();
    for component in relative.components() {
        target.push(component);
        let is_link = fs::symlink_metadata(&target).is_ok_and(|meta| meta.file_type().is_symlink());
        if is_link {
            return Err(format!(
                "{} is a symbolic link; a guide page never writes through one",
                target.display()
            ));
        }
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)
            .map_err(|error| format!("{} cannot be made: {error}", parent.display()))?;
    }
    fs::write(&target, text)
        .map_err(|error| format!("{} cannot be written: {error}", target.display()))
}
