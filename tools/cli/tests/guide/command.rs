//! Running a guide command against the real binary, and matching what it prints (`docs/GUIDE_FORMAT.md`
//! §§4.2–4.4). The runner calls `mineworld` directly, with no shell, so a command means the same on
//! every operating system.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use snapbox::{Assert, Data};

use super::page::Family;

/// The `mineworld` binary this test was built with.
pub const BINARY: &str = env!("CARGO_BIN_EXE_mineworld");

/// Characters a command line may not contain: they would mean something only to a shell, and a reader
/// who sees them expects one.
const SHELL_CHARACTERS: &[char] = &[
    '|', '&', ';', '<', '>', '*', '?', '$', '`', '(', ')', '{', '}', '~', '"', '\'',
];

/// The words of one command line (the text after its `$ `), checked against §4.2: no shell syntax, the
/// first word `mineworld`, and no argument that names a path outside the scratch folder.
pub fn words(command: &str) -> Result<Vec<String>, String> {
    if let Some(character) = command.chars().find(|c| SHELL_CHARACTERS.contains(c)) {
        return Err(format!(
            "{command:?} contains {character:?}, which is shell syntax; a guide command runs without a shell"
        ));
    }
    let words: Vec<String> = command.split_whitespace().map(str::to_owned).collect();
    if words.first().map(String::as_str) != Some("mineworld") {
        return Err(format!("{command:?} does not start with mineworld"));
    }
    for word in &words[1..] {
        if word.contains('\\') || word.starts_with('/') || word.split('/').any(|part| part == "..")
        {
            return Err(format!(
                "argument {word:?} names a path outside the scratch folder, or uses '\\'"
            ));
        }
    }
    Ok(words)
}

/// Runs the binary with `words` (whose first word is `mineworld`) in `scratch`. Returns what it wrote to
/// its standard output and standard error, interleaved in the order it wrote them, and its exit code
/// (-1 when it was ended by a signal).
pub fn run(scratch: &Path, words: &[String]) -> Result<(String, i32), String> {
    let (mut reader, writer) =
        std::io::pipe().map_err(|error| format!("a pipe cannot be made: {error}"))?;
    let mut command = Command::new(BINARY);
    command
        .args(&words[1..])
        .current_dir(scratch)
        .stdin(Stdio::null())
        .stdout(writer.try_clone().map_err(|error| error.to_string())?)
        .stderr(writer);
    let mut child = command
        .spawn()
        .map_err(|error| format!("the binary cannot be started: {error}"))?;
    // The command holds the pipe's write end; it must go before reading, or the read never ends.
    drop(command);
    let mut output = Vec::new();
    reader
        .read_to_end(&mut output)
        .map_err(|error| format!("the output cannot be read: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("the binary cannot be waited for: {error}"))?;
    Ok((
        String::from_utf8_lossy(&output).into_owned(),
        status.code().unwrap_or(-1),
    ))
}

/// Matches `actual` against `expected` with snapbox's text rules (§4.4): `...` on a line of its own matches
/// any lines, `[..]` inside a line matches any text there, and snapbox normalises line endings and path
/// separators. Its other filters (`[EXE]`, JSON wildcards) are in force too; no guide page uses them.
pub fn matches(expected: &str, actual: &str) -> Result<(), String> {
    let name: &dyn std::fmt::Display = &"the command's output";
    Assert::new()
        .try_eq(
            Some(name),
            Data::text(actual.to_owned()),
            Data::text(expected.to_owned()),
        )
        .map_err(|error| error.to_string())
}

/// `actual` as snapbox's own overwrite would record it against `expected`: the wildcards kept up to the first
/// line that differs, every line after it literal. Write mode falls back to it (§11.2).
pub fn snapbox_recorded(expected: &str, actual: &str) -> String {
    let (actual, _) = Assert::new().normalize(
        Data::text(actual.to_owned()),
        Data::text(expected.to_owned()),
    );
    actual.render().unwrap_or_default()
}

/// The address chapter 01 clones. The runner points it at the repository under test (§4.6): the operator's
/// ruling of 2026-10-18 lets `git clone` read the checked-out copy instead of the network, while the command
/// itself runs as written.
pub const CLONE_URL: &str = "https://github.com/yuema137/MineWorld";

/// Runs a `sh` or `powershell` block as a script, in `scratch`, with `home` as the reader's home folder
/// (§4.6). The script is written beside `home`'s clone, never into the scratch folder, so an `equals` check
/// of the scratch folder sees only what the page made. The block passes when the script exits 0; what it
/// printed is returned in the failure.
pub fn shell(
    scratch: &Path,
    home: &Path,
    repo: &Path,
    family: Family,
    text: &str,
) -> Result<(), String> {
    let scripts = home.join(".guide-scripts");
    std::fs::create_dir_all(&scripts)
        .map_err(|error| format!("{} cannot be made: {error}", scripts.display()))?;
    let count = std::fs::read_dir(&scripts)
        .map(Iterator::count)
        .unwrap_or(0);
    let (path, mut command) = match family {
        Family::Posix => {
            let path = scripts.join(format!("block-{count}.sh"));
            let mut command = Command::new("sh");
            command.arg("-e").arg(&path);
            (path, command)
        }
        Family::PowerShell => {
            let path = scripts.join(format!("block-{count}.ps1"));
            let mut command = Command::new("powershell");
            command
                .args([
                    "-NoProfile",
                    "-NonInteractive",
                    "-ExecutionPolicy",
                    "Bypass",
                    "-File",
                ])
                .arg(&path);
            (path, command)
        }
    };
    std::fs::write(&path, script(family, text))
        .map_err(|error| format!("{} cannot be written: {error}", path.display()))?;
    let real_home = std::env::var_os(if cfg!(windows) { "USERPROFILE" } else { "HOME" })
        .map(std::path::PathBuf::from)
        .ok_or("the test process has no home folder to find Cargo and rustup in")?;
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map_or_else(|| real_home.join(".cargo"), std::path::PathBuf::from);
    let rustup_home = std::env::var_os("RUSTUP_HOME")
        .map_or_else(|| real_home.join(".rustup"), std::path::PathBuf::from);
    // The build chapter 01 runs keeps its output beside this test's own target folder, so a second run
    // (and CI's cache) reuses it; the clone's own `target/` stays empty (§4.6).
    let target = Path::new(BINARY)
        .parent()
        .and_then(Path::parent)
        .map(|target| target.join("guide-build"))
        .ok_or("the test binary has no target folder")?;
    let repo_url = lexical(repo).to_string_lossy().replace('\\', "/");
    command
        .current_dir(scratch)
        .stdin(Stdio::null())
        .env("HOME", home)
        .env("CARGO_HOME", cargo_home)
        .env("RUSTUP_HOME", rustup_home)
        .env("CARGO_TARGET_DIR", target)
        .env("GIT_CONFIG_COUNT", "2")
        .env("GIT_CONFIG_KEY_0", format!("url.{repo_url}.insteadOf"))
        .env("GIT_CONFIG_VALUE_0", CLONE_URL)
        .env("GIT_CONFIG_KEY_1", "safe.directory")
        .env("GIT_CONFIG_VALUE_1", "*")
        // The clone reads the checked-out copy, which holds Git LFS pointers only when the checkout did
        // not fetch LFS objects (CI's `lfs: false`); where git-lfs is installed (Windows runners) its
        // smudge would then fail on the cast's art (b805ddaf). Chapter 01 builds the command, which needs
        // none of it, so the clone keeps the pointers.
        .env("GIT_LFS_SKIP_SMUDGE", "1");
    if cfg!(windows) {
        // PowerShell's `$HOME` follows these, as the reader's own profile would.
        let text = home.to_string_lossy().into_owned();
        let (drive, rest) = text.split_at(text.find(':').map_or(0, |colon| colon + 1));
        command
            .env("USERPROFILE", home)
            .env("HOMEDRIVE", drive)
            .env("HOMEPATH", rest);
    }
    let output = command
        .output()
        .map_err(|error| format!("{} cannot be started: {error}", family.word()))?;
    if output.status.success() {
        return Ok(());
    }
    Err(format!(
        "the {} block exited with {}:\n{}{}",
        family.word(),
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    ))
}

/// The script a block becomes. `sh -e` stops at the first command that fails. PowerShell stops at a failing
/// cmdlet with `$ErrorActionPreference`, and a program's exit code is checked after every line, because
/// PowerShell does not stop on one by itself.
fn script(family: Family, text: &str) -> String {
    match family {
        Family::Posix => text.to_owned(),
        Family::PowerShell => {
            let mut script = String::from("$ErrorActionPreference = 'Stop'\n");
            for line in text.lines().filter(|line| !line.trim().is_empty()) {
                script.push_str(line);
                script.push_str("\nif ($LASTEXITCODE) { exit $LASTEXITCODE }\n");
            }
            script
        }
    }
}

/// `path` with its `.` and `..` components folded away, without touching the file system: git reads the
/// repository at this path, and a canonicalised Windows path carries a verbatim prefix git does not take.
fn lexical(path: &Path) -> std::path::PathBuf {
    let mut folded = std::path::PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::CurDir => {}
            std::path::Component::ParentDir => {
                folded.pop();
            }
            other => folded.push(other),
        }
    }
    folded
}
