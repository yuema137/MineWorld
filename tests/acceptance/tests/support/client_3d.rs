//! The 3D reference client, named once for this crate's repository scans (S14 PR 16e D-e-3, ARC-347).
//!
//! The 3D client is removable by deletion: a checkout without it builds and passes every Rust test
//! (`scripts/ci_removability.py --target 3d`). A scan that reads both clients therefore reads its 3D
//! half only when the client is there. When it is not, the scan writes one line naming what it skipped,
//! so a skipped half is reported and never passed silently; the 2D half and the rest still run. With
//! the client present, every scan runs exactly as before.

// Each scan that includes this module uses the part it needs.
#![allow(dead_code)]

use std::io::Write;
use std::path::Path;

/// The 3D client's directory, relative to the repository root: `client_3d!()` is the directory and
/// `client_3d!("scripts/town.gd")` a path inside it, both string literals, so tables of paths can use it.
macro_rules! client_3d {
    () => {
        "clients/3d-spike"
    };
    ($path:literal) => {
        concat!(client_3d!(), "/", $path)
    };
}

/// Whether the 3D client is in the checkout at `root`. When it is not, writes
/// `SKIPPED: 3D client absent (clients/3d-spike): <skipped>` to stderr. The write goes past the test
/// harness's output capture, so the line is in every run's log, passing or failing.
pub fn present(root: &Path, skipped: &str) -> bool {
    if root.join(client_3d!()).is_dir() {
        return true;
    }
    let _ = writeln!(
        std::io::stderr(),
        "SKIPPED: 3D client absent ({}): {skipped}",
        client_3d!()
    );
    false
}

/// Whether `path`, relative to the repository root with `/` separators, lies inside the 3D client.
pub fn holds(path: &str) -> bool {
    path.strip_prefix(client_3d!())
        .is_some_and(|rest| rest.starts_with('/'))
}
