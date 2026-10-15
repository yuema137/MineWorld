//! Running a guide command against the real binary, and matching what it prints (`docs/GUIDE_FORMAT.md`
//! §§4.2–4.4). The runner calls `mineworld` directly, with no shell, so a command means the same on
//! every operating system.

use std::io::Read;
use std::path::Path;
use std::process::{Command, Stdio};

use snapbox::{Assert, Data};

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
