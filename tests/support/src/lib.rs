//! A test's scratch directory, removed when the test ends (`ENGINEERING_STANDARDS.md` §22, "Test
//! scratch"; `DEP-29`).
//!
//! A test asks for scratch with [`scratch!`] and holds the returned [`Scratch`] for as long as it uses
//! the path. The scratch lives at `<CARGO_TARGET_TMPDIR>/mineworld-scratch-<pid>/<name>`:
//!
//! - the last component is exactly `name`, because a World Pack's id is its directory's name;
//! - the pid keeps two processes on one `target/` apart;
//! - one container per process keeps kept scratches side by side, so a directory of kept saves can be
//!   handed back to a test (`BODIES_YARD_SAVES`).
//!
//! Dropping the guard removes the scratch, whether the test passed or failed, unless
//! `MINEWORLD_KEEP_SCRATCH` says to keep it ([`Keep`]). A name is unique among the scratches alive in
//! one process: asking for a live name again panics and names it, rather than letting two tests write
//! one save.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Deref;
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, MutexGuard};

/// The environment variable that keeps scratch: unset or empty, `failed`, or `all`.
pub const KEEP_VARIABLE: &str = "MINEWORLD_KEEP_SCRATCH";

/// A test's scratch: a temporary directory under the calling test crate's `CARGO_TARGET_TMPDIR`.
///
/// `scratch!("name")` gives a path that does not exist yet (for a command that creates it, such as
/// `mineworld run --save` or `mineworld create`); `scratch!(empty "name")` gives an existing, empty
/// directory. `name` may be any expression that is `AsRef<str>`.
#[macro_export]
macro_rules! scratch {
    (empty $name:expr) => {
        $crate::Scratch::empty(env!("CARGO_TARGET_TMPDIR"), $name)
    };
    ($name:expr) => {
        $crate::Scratch::absent(env!("CARGO_TARGET_TMPDIR"), $name)
    };
}

/// Whether a scratch outlives its guard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keep {
    /// Removed when dropped, whether the test passed or failed. The default.
    Never,
    /// Kept when dropped while the thread is panicking — the test failed; removed otherwise.
    Failed,
    /// Always kept.
    All,
}

impl Keep {
    /// Reads [`KEEP_VARIABLE`]. Panics on a value it does not know: a typo must not silently delete
    /// the evidence it was meant to keep.
    pub fn from_environment() -> Self {
        let value =
            std::env::var_os(KEEP_VARIABLE).map(|value| value.to_string_lossy().into_owned());
        Self::parse(value.as_deref()).unwrap_or_else(|refusal| panic!("{refusal}"))
    }

    /// The policy a value of [`KEEP_VARIABLE`] names, or why it names none.
    pub fn parse(value: Option<&str>) -> Result<Self, String> {
        match value {
            None | Some("") => Ok(Self::Never),
            Some("failed") => Ok(Self::Failed),
            Some("all") => Ok(Self::All),
            Some(other) => Err(format!(
                "{KEEP_VARIABLE}={other:?} is not understood: use 'failed' or 'all', or leave it unset"
            )),
        }
    }

    fn keeps(self, panicking: bool) -> bool {
        match self {
            Self::Never => false,
            Self::Failed => panicking,
            Self::All => true,
        }
    }
}

/// Whether a new scratch is a path to be created by the test, or an existing empty directory.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shape {
    /// The path does not exist when the guard is made.
    Absent,
    /// The path is an empty directory when the guard is made.
    Empty,
}

/// The live names of each container in this process. Every creation and removal happens under this
/// lock, so a container is never removed while another scratch is being made in it.
static LIVE: Mutex<BTreeMap<PathBuf, BTreeSet<String>>> = Mutex::new(BTreeMap::new());

fn live() -> MutexGuard<'static, BTreeMap<PathBuf, BTreeSet<String>>> {
    // A panic never happens while the lock is held, but a poisoned lock must not cascade into every
    // later test of the binary.
    LIVE.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

/// A scratch directory, removed when dropped unless its [`Keep`] says otherwise.
#[must_use = "a scratch is removed when its guard is dropped; bind it for as long as the path is used"]
#[derive(Debug)]
pub struct Scratch {
    container: PathBuf,
    name: String,
    path: PathBuf,
    keep: Keep,
}

impl Scratch {
    /// A path under `root` that does not exist yet, kept per [`KEEP_VARIABLE`].
    pub fn absent(root: &str, name: impl AsRef<str>) -> Self {
        Self::with_keep(root, name, Shape::Absent, Keep::from_environment())
    }

    /// An empty directory under `root`, kept per [`KEEP_VARIABLE`].
    pub fn empty(root: &str, name: impl AsRef<str>) -> Self {
        Self::with_keep(root, name, Shape::Empty, Keep::from_environment())
    }

    /// A scratch with an explicit policy, for the helper's own tests: no test mutates the process
    /// environment to choose one.
    #[doc(hidden)]
    pub fn with_keep(root: &str, name: impl AsRef<str>, shape: Shape, keep: Keep) -> Self {
        let name = name.as_ref();
        let mut components = Path::new(name).components();
        let single = matches!(
            (components.next(), components.next()),
            (Some(Component::Normal(_)), None)
        );
        assert!(
            single,
            "a scratch name is one plain path component, not {name:?}"
        );
        let container = Path::new(root).join(format!("mineworld-scratch-{}", std::process::id()));
        let path = container.join(name);
        let mut live = live();
        let names = live.entry(container.clone()).or_default();
        if !names.insert(name.to_owned()) {
            drop(live);
            panic!("scratch {name:?} is already in use by another test in this process");
        }
        // A stale directory of the same name can only be left by a dead process whose pid was reused.
        let _ = std::fs::remove_dir_all(&path);
        let made = match shape {
            Shape::Absent => std::fs::create_dir_all(&container),
            Shape::Empty => std::fs::create_dir_all(&path),
        };
        if let Err(error) = made {
            names.remove(name);
            drop(live);
            panic!("scratch {} cannot be made: {error}", path.display());
        }
        Self {
            container,
            name: name.to_owned(),
            path,
            keep,
        }
    }

    /// The scratch's path.
    pub fn path(&self) -> &Path {
        &self.path
    }
}

impl Deref for Scratch {
    type Target = Path;

    fn deref(&self) -> &Path {
        &self.path
    }
}

impl AsRef<Path> for Scratch {
    fn as_ref(&self) -> &Path {
        &self.path
    }
}

impl Drop for Scratch {
    fn drop(&mut self) {
        let kept = self.keep.keeps(std::thread::panicking());
        let mut live = live();
        if kept {
            eprintln!("[scratch] kept {}", self.path.display());
        } else if let Err(error) = std::fs::remove_dir_all(&self.path)
            && error.kind() != std::io::ErrorKind::NotFound
        {
            eprintln!(
                "[scratch] could not remove {}: {error}",
                self.path.display()
            );
        }
        let last = live.get_mut(&self.container).is_some_and(|names| {
            names.remove(&self.name);
            names.is_empty()
        });
        if last {
            live.remove(&self.container);
            // Fails, harmlessly, while a kept or stale scratch is still inside.
            let _ = std::fs::remove_dir(&self.container);
        }
    }
}
