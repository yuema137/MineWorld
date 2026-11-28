//! Removing a scratch tree on every platform (`DEP-29`).
//!
//! `std::fs::remove_dir_all` alone leaves a tree behind on Windows in two cases a guide run meets: a file a
//! scanner or a just-exited child still holds open for a moment (a sharing violation), and a read-only file
//! on a volume where the delete cannot ignore the attribute (Git marks its object files read-only). A tree
//! that is left behind fills the runner's disk and turns `scripts/check_scratch.py left` red, so removal
//! retries with a short backoff and, on Windows, clears read-only attributes between attempts.

use std::io;
use std::path::Path;
use std::time::Duration;

/// Attempts before removal gives up. The waits between them add up to a little under three seconds.
const ATTEMPTS: u32 = 8;

/// Removes `path` and everything under it. A path that does not exist is already removed. The last
/// error is returned when every attempt fails.
pub fn remove_tree(path: &Path) -> io::Result<()> {
    let mut attempt = 1;
    loop {
        match std::fs::remove_dir_all(path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) if attempt == ATTEMPTS => return Err(error),
            Err(_) => {
                clear_read_only(path);
                std::thread::sleep(Duration::from_millis(100 * u64::from(attempt)));
                attempt += 1;
            }
        }
    }
}

/// Clears the read-only attribute of every file under `path`, ignoring what cannot be read.
#[cfg(windows)]
fn clear_read_only(path: &Path) {
    let Ok(entries) = std::fs::read_dir(path) else {
        return;
    };
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        if kind.is_dir() {
            clear_read_only(&entry.path());
        } else if let Ok(metadata) = entry.metadata() {
            let mut permissions = metadata.permissions();
            if permissions.readonly() {
                // On Windows this clears one attribute bit; the unix world-writable concern of the lint does
                // not apply.
                #[allow(clippy::permissions_set_readonly_false)]
                permissions.set_readonly(false);
                let _ = std::fs::set_permissions(entry.path(), permissions);
            }
        }
    }
}

/// On other systems a read-only file in a writable directory is removed like any other.
#[cfg(not(windows))]
fn clear_read_only(_path: &Path) {}
