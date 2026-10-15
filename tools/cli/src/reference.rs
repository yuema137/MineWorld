//! `mineworld reference --out DIR | --check DIR` — the configuration reference, generated from the
//! types that decode a World Pack's files (`docs/DECISIONS.md` `ARC-90`, `DEP-49`;
//! `docs/MODULE_SPEC.md` §8.1).
//!
//! ```text
//! entries    what is documented: the format's own files, then every installed pack through the
//!            installed set, each schema normalized and checked (a field with no description, or a
//!            schema with no example, refuses generation)
//! markdown   one entry as a person reads it; yaml its example; cli_page the parser's surface
//! pages      the files: each page, the JSON Schemas, the index
//! this file  --out writes them into a directory it owns; --check compares a directory with them
//! ```
//!
//! It loads no World Pack and starts no world. What it generates is a function of the build alone,
//! so `--check docs/reference` passing in CI is the proof that the checked-in reference describes the
//! code (`INV-S25-2`).

mod cli_page;
mod entries;
mod markdown;
mod pages;
mod yaml;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use clap::CommandFactory;

/// What `mineworld reference` was asked to do.
#[derive(Debug)]
pub(crate) enum Target {
    /// Write the reference into this directory.
    Out(PathBuf),
    /// Compare this directory with the reference this build generates.
    Check(PathBuf),
}

/// Runs the command.
///
/// # Errors
///
/// A refusal to generate, a directory holding files the command did not generate, an I/O failure,
/// or — for `--check` — every file that differs, is missing or is extra.
pub(crate) fn reference(target: &Target) -> Result<(), String> {
    let generated = generate()?;
    match target {
        Target::Out(dir) => write(dir, &generated),
        Target::Check(dir) => check(dir, &generated),
    }
}

/// Every file of this build's reference, by path under the reference directory.
fn generate() -> Result<BTreeMap<String, String>, String> {
    let entries = entries::collect()?;
    let mut cli = crate::Cli::command();
    cli.build();
    Ok(pages::files(&entries, &cli))
}

/// Writes the reference into `dir`, removing generated files it no longer writes.
fn write(dir: &Path, generated: &BTreeMap<String, String>) -> Result<(), String> {
    let existing = listed(dir)?;
    let mut foreign = Vec::new();
    let mut stale = Vec::new();
    for path in existing.keys() {
        if generated.contains_key(path) {
            continue;
        }
        if is_generated(path, &existing[path]) {
            stale.push(path.clone());
        } else {
            foreign.push(path.clone());
        }
    }
    if !foreign.is_empty() {
        return Err(format!(
            "[mineworld] reference: {} holds files mineworld reference did not generate, so it \
             writes nothing there:\n  {}\nMove them, or name an empty directory.",
            dir.display(),
            foreign.join("\n  ")
        ));
    }
    for path in &stale {
        std::fs::remove_file(dir.join(path)).map_err(|error| io(dir, path, &error))?;
    }
    for (path, text) in generated {
        let file = dir.join(path);
        if let Some(parent) = file.parent() {
            std::fs::create_dir_all(parent).map_err(|error| io(dir, path, &error))?;
        }
        std::fs::write(&file, text).map_err(|error| io(dir, path, &error))?;
    }
    println!(
        "[mineworld] reference: {} file(s) written to {}{}",
        generated.len(),
        dir.display(),
        if stale.is_empty() {
            String::new()
        } else {
            format!(", {} stale file(s) removed", stale.len())
        }
    );
    Ok(())
}

/// Compares `dir` with the reference, byte for byte.
fn check(dir: &Path, generated: &BTreeMap<String, String>) -> Result<(), String> {
    let existing = listed(dir)?;
    let mut problems = Vec::new();
    for (path, text) in generated {
        match existing.get(path) {
            None => problems.push(format!("missing  {path}")),
            Some(found) if found != text.as_bytes() => problems.push(format!("differs  {path}")),
            Some(_) => {}
        }
    }
    for path in existing
        .keys()
        .filter(|path| !generated.contains_key(*path))
    {
        problems.push(format!("extra    {path}"));
    }
    if problems.is_empty() {
        println!(
            "[mineworld] reference: {} matches this build ({} file(s))",
            dir.display(),
            generated.len()
        );
        return Ok(());
    }
    Err(format!(
        "[mineworld] reference: {} is not what this build generates:\n  {}\nRegenerate it with \
         mineworld reference --out {}",
        dir.display(),
        problems.join("\n  "),
        dir.display()
    ))
}

/// Every file under `dir`, by its `/`-separated path, with its bytes; none when `dir` does not
/// exist.
fn listed(dir: &Path) -> Result<BTreeMap<String, Vec<u8>>, String> {
    let mut files = BTreeMap::new();
    if !dir.exists() {
        return Ok(files);
    }
    let mut pending = vec![PathBuf::new()];
    while let Some(relative) = pending.pop() {
        let read = std::fs::read_dir(dir.join(&relative))
            .map_err(|error| io(dir, &relative.display().to_string(), &error))?;
        for found in read {
            let found = found.map_err(|error| io(dir, &relative.display().to_string(), &error))?;
            let path = relative.join(found.file_name());
            let name = path
                .components()
                .map(|part| part.as_os_str().to_string_lossy().into_owned())
                .collect::<Vec<_>>()
                .join("/");
            if found.path().is_dir() {
                pending.push(path);
            } else {
                let bytes = std::fs::read(found.path()).map_err(|error| io(dir, &name, &error))?;
                files.insert(name, bytes);
            }
        }
    }
    Ok(files)
}

/// Whether a file was written by this command: a Markdown page beginning with the banner, or a
/// JSON Schema carrying the generated `$comment`.
fn is_generated(path: &str, bytes: &[u8]) -> bool {
    let text = String::from_utf8_lossy(bytes);
    if path.ends_with(".md") {
        return text.starts_with(&format!("<!-- {}", pages::GENERATED));
    }
    path.ends_with(".schema.json")
        && text.contains(&format!("\"$comment\": \"{}", pages::GENERATED))
}

fn io(dir: &Path, path: &str, error: &std::io::Error) -> String {
    format!(
        "[mineworld] reference: {}: {error}",
        dir.join(path).display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The reference is a function of the build: two generations are byte-equal, every file ends in
    /// one newline and holds no carriage return, and every Markdown page starts with the banner.
    #[test]
    fn the_reference_is_a_function_of_the_build() {
        let first = generate().expect("this build's reference generates");
        let second = generate().expect("again");
        assert_eq!(first, second);
        for (path, text) in &first {
            assert!(!text.contains('\r'), "{path}");
            assert!(text.ends_with('\n') && !text.ends_with("\n\n"), "{path}");
            assert!(is_generated(path, text.as_bytes()), "{path}");
        }
    }
}
