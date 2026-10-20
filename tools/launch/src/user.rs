//! The per-user `MineWorld` folder (`LAUNCHER.md` §5): saves, logs and smoke scratch. Nothing is ever
//! written beside the executable, whose folder may be read-only (step-23 F-R8).

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::Failure;
use crate::config::WorldName;

/// How many runs' log files are kept.
const KEPT_RUNS: usize = 5;

/// The per-user folder.
#[derive(Debug)]
pub struct UserDir {
    root: PathBuf,
}

/// One run's identity, `<unix seconds>-<pid>`: the prefix of its log files and its scratch folder.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Run {
    seconds: u64,
    pid: u32,
}

impl Run {
    pub fn now() -> Self {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_or(0, |since| since.as_secs());
        Self {
            seconds,
            pid: std::process::id(),
        }
    }

    /// The run a log file belongs to, from its name: `<seconds>-<pid>-<what>.log`.
    fn of_log(name: &str) -> Option<Self> {
        let stem = name.strip_suffix(".log")?;
        let mut parts = stem.splitn(3, '-');
        let seconds = parts.next()?.parse().ok()?;
        let pid = parts.next()?.parse().ok()?;
        parts.next().filter(|what| !what.is_empty())?;
        Some(Self { seconds, pid })
    }
}

impl std::fmt::Display for Run {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}-{}", self.seconds, self.pid)
    }
}

impl UserDir {
    /// `given`, or the platform's folder (`clients/shared/SETTINGS.md` §4), created if absent.
    pub fn open(given: Option<PathBuf>) -> Result<Self, Failure> {
        let root = match given {
            Some(root) => root,
            None => platform_folder().ok_or_else(|| {
                Failure::new("cannot find your user folder (no APPDATA or HOME is set)".to_owned())
            })?,
        };
        let root = std::path::absolute(&root)
            .map_err(|error| Failure::new(format!("cannot resolve {}: {error}", root.display())))?;
        for folder in [root.join("logs"), root.join("saves")] {
            std::fs::create_dir_all(&folder).map_err(|error| {
                Failure::new(format!("cannot create {}: {error}", folder.display()))
            })?;
        }
        Ok(Self { root })
    }

    /// A world's save folder.
    pub fn save(&self, world: &WorldName) -> PathBuf {
        self.root.join("saves").join(world.as_str())
    }

    /// A smoke run's scratch save, removed when the run ends.
    pub fn scratch(&self, run: Run) -> PathBuf {
        self.root.join("scratch").join(run.to_string())
    }

    /// A log file of `run`: `<run>-<what>.log`.
    pub fn log(&self, run: Run, what: &str) -> PathBuf {
        self.root.join("logs").join(format!("{run}-{what}.log"))
    }

    /// Deletes the log files of every run but the `KEPT_RUNS` most recent. A file whose name is not a
    /// run's log is never touched; a file that cannot be deleted is left.
    pub fn prune_logs(&self) {
        let Ok(entries) = std::fs::read_dir(self.root.join("logs")) else {
            return;
        };
        let logs: Vec<(Run, PathBuf)> = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let run = Run::of_log(entry.file_name().to_str()?)?;
                Some((run, entry.path()))
            })
            .collect();
        let mut runs: Vec<Run> = logs.iter().map(|(run, _)| *run).collect();
        runs.sort_unstable_by(|a, b| b.cmp(a));
        runs.dedup();
        let kept = &runs[..runs.len().min(KEPT_RUNS)];
        for (run, path) in logs {
            if !kept.contains(&run) {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

#[cfg(windows)]
fn platform_folder() -> Option<PathBuf> {
    std::env::var_os("APPDATA").map(|appdata| PathBuf::from(appdata).join("MineWorld"))
}

#[cfg(target_os = "macos")]
fn platform_folder() -> Option<PathBuf> {
    std::env::var_os("HOME")
        .map(|home| PathBuf::from(home).join("Library/Application Support/MineWorld"))
}

#[cfg(all(unix, not(target_os = "macos")))]
fn platform_folder() -> Option<PathBuf> {
    let xdg = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute());
    let data = xdg.or_else(|| {
        std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share"))
    })?;
    Some(data.join("MineWorld"))
}

/// The launcher's own log: every line also goes to standard error.
pub struct Log {
    file: Option<File>,
    path: Option<PathBuf>,
}

impl Log {
    /// A log with no file yet: lines go to standard error only.
    pub fn stderr_only() -> Self {
        Self {
            file: None,
            path: None,
        }
    }

    /// Starts writing to `path` as well.
    pub fn write_to(&mut self, path: &Path) -> Result<(), Failure> {
        let file = File::create(path)
            .map_err(|error| Failure::new(format!("cannot write {}: {error}", path.display())))?;
        self.file = Some(file);
        self.path = Some(path.to_path_buf());
        Ok(())
    }

    /// The file, once there is one.
    pub fn path(&self) -> Option<&Path> {
        self.path.as_deref()
    }

    pub fn line(&mut self, text: &str) {
        eprintln!("[launch] {text}");
        if let Some(file) = &mut self.file {
            let _ = writeln!(file, "{text}");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_log_name_belongs_to_its_run_and_nothing_else_does() {
        assert_eq!(
            Run::of_log("1760000000-4242-client-2d.log"),
            Some(Run {
                seconds: 1_760_000_000,
                pid: 4242
            })
        );
        for other in [
            "notes.log",
            "1760000000-4242.log",
            "1760000000-4242-server.txt",
            "x-4242-server.log",
            "settings.cfg",
        ] {
            assert_eq!(Run::of_log(other), None, "{other}");
        }
    }
}
