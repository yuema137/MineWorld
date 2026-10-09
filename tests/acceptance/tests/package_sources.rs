//! Where every package of the build comes from (`docs/DECISIONS.md` `ARC-66`; step-16 §16.5 EC-2).
//!
//! A System Pack from another repository enters the build by two lines pinned to a commit, and names the
//! framework's crates by version, which the build maps to its own source in `.cargo/config.toml`. This
//! file holds what makes that safe, over the real `Cargo.lock`:
//!
//! ```text
//! EC-2 (i)   every package with no source is a workspace member; every registry package has a
//!            checksum; every git package's source is git+<url>?rev=<40 hex>#<the same 40 hex>
//! EC-2 (ii)  no package named mineworld-* comes from a registry — a missing or broken published-surface
//!            table fails here, rather than compiling a stranger's crate of that name
//! ```
//!
//! The lock is read line by line, as `systems/installed/tests/installed.rs` reads its manifest, and a
//! line it cannot read fails the test rather than being skipped. `str::lines` strips a `\r`, so a
//! Windows checkout's CRLF lock reads the same. No pack is named here: a third-party pack is located as a
//! package whose source begins `git+`.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root.
fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// One `[[package]]` of a `Cargo.lock`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Locked {
    name: String,
    version: String,
    source: Option<String>,
    checksum: Option<String>,
    /// The first word of each `dependencies` entry: the depended-on package's name.
    dependencies: Vec<String>,
}

/// Reads a `Cargo.lock`, refusing any line it does not understand, naming the line.
fn read_lock(text: &str) -> Result<Vec<Locked>, String> {
    let mut packages = Vec::new();
    let mut current: Option<Locked> = None;
    let mut in_list = false;
    let mut in_other_table = false;
    for (number, raw) in text.lines().enumerate() {
        let line = raw.trim_end();
        let refuse = || format!("Cargo.lock line {}: cannot read {line:?}", number + 1);
        if in_list {
            if line == "]" {
                in_list = false;
            } else {
                let entry = line
                    .trim()
                    .strip_prefix('"')
                    .and_then(|rest| rest.strip_suffix("\","))
                    .ok_or_else(refuse)?;
                let name = entry.split(' ').next().ok_or_else(refuse)?;
                current
                    .as_mut()
                    .ok_or_else(refuse)?
                    .dependencies
                    .push(name.to_owned());
            }
            continue;
        }
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if line.starts_with('[') {
            packages.extend(current.take());
            in_other_table = line != "[[package]]";
            if !in_other_table {
                current = Some(Locked::default());
            }
            continue;
        }
        let (key, value) = line.split_once(" = ").ok_or_else(refuse)?;
        if in_other_table {
            continue;
        }
        let Some(package) = current.as_mut() else {
            // Before the first table: only the format's `version = <n>`.
            if key == "version" {
                continue;
            }
            return Err(refuse());
        };
        let text = || {
            value
                .strip_prefix('"')
                .and_then(|rest| rest.strip_suffix('"'))
                .map(str::to_owned)
                .ok_or_else(refuse)
        };
        match key {
            "name" => package.name = text()?,
            "version" => package.version = text()?,
            "source" => package.source = Some(text()?),
            "checksum" => package.checksum = Some(text()?),
            "dependencies" if value == "[" => in_list = true,
            "dependencies" => {
                let inline = value
                    .strip_prefix('[')
                    .and_then(|rest| rest.strip_suffix(']'))
                    .ok_or_else(refuse)?;
                for entry in inline.split(',').map(str::trim).filter(|e| !e.is_empty()) {
                    let entry = entry.trim_matches('"');
                    let name = entry.split(' ').next().ok_or_else(refuse)?;
                    package.dependencies.push(name.to_owned());
                }
            }
            _ => return Err(refuse()),
        }
    }
    packages.extend(current);
    if packages.iter().any(|package| package.name.is_empty()) {
        return Err("Cargo.lock: a [[package]] without a name".to_owned());
    }
    Ok(packages)
}

/// Whether `text` is exactly forty lowercase hexadecimal digits: a full commit id.
fn full_commit(text: &str) -> bool {
    text.len() == 40
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

/// A git source's URL and its pinned commit, when it is `git+<url>?rev=<40 hex>#<the same 40 hex>`.
fn pinned_git(source: &str) -> Option<(&str, &str)> {
    let rest = source.strip_prefix("git+")?;
    let (url, pin) = rest.split_once("?rev=")?;
    let (rev, resolved) = pin.split_once('#')?;
    (full_commit(rev) && rev == resolved && !url.is_empty()).then_some((url, rev))
}

/// EC-2 (i) and (ii): each refusal names the package.
fn refused_sources(packages: &[Locked], members: &BTreeSet<String>) -> Vec<String> {
    let mut refused = Vec::new();
    for package in packages {
        let name = &package.name;
        match package.source.as_deref() {
            None if !members.contains(name) => {
                refused.push(format!("{name}: no source, and not a workspace member"));
            }
            None => {}
            Some(source) if source.starts_with("registry+") => {
                if package.checksum.is_none() {
                    refused.push(format!("{name}: a registry package without a checksum"));
                }
                if name.starts_with("mineworld-") {
                    refused.push(format!(
                        "{name}: a framework crate from a registry ({source}); the published-surface \
                         table in .cargo/config.toml must map it to this repository"
                    ));
                }
            }
            Some(source) if pinned_git(source).is_some() => {}
            Some(source) => refused.push(format!(
                "{name}: source {source} is neither a registry with a checksum nor \
                 git+<url>?rev=<40 hex>#<the same>"
            )),
        }
    }
    refused
}

/// `cargo metadata --format-version 1 --offline <extra>` in the repository, through the `cargo` that
/// built this test. Run in the repository, so `.cargo/config.toml` is read (`ARC-66`).
///
/// The members come from `--no-deps`, which resolves nothing and so neither reads nor rewrites the
/// lock: a lock the lock guard must refuse is read as it is, never repaired first.
fn metadata(extra: &str) -> serde_json::Value {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--offline", extra])
        .current_dir(repository())
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr).trim()
    );
    serde_json::from_slice(&output.stdout).expect("cargo metadata's output parses")
}

/// The workspace members' package names, from a metadata value.
fn members(metadata: &serde_json::Value) -> BTreeSet<String> {
    let ids: BTreeSet<&str> = metadata["workspace_members"]
        .as_array()
        .expect("workspace_members")
        .iter()
        .filter_map(serde_json::Value::as_str)
        .collect();
    metadata["packages"]
        .as_array()
        .expect("packages")
        .iter()
        .filter(|package| package["id"].as_str().is_some_and(|id| ids.contains(id)))
        .map(|package| package["name"].as_str().expect("a name").to_owned())
        .collect()
}

fn the_lock() -> Vec<Locked> {
    let path = repository().join("Cargo.lock");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
    read_lock(&text).unwrap_or_else(|error| panic!("{error}"))
}

#[test]
fn every_package_of_the_lock_is_a_member_or_content_addressed_and_no_framework_crate_is_from_a_registry()
 {
    let packages = the_lock();
    let members = members(&metadata("--no-deps"));
    let located: BTreeMap<&str, usize> = [
        (
            "members",
            packages.iter().filter(|p| p.source.is_none()).count(),
        ),
        (
            "registry",
            packages
                .iter()
                .filter(|p| {
                    p.source
                        .as_deref()
                        .is_some_and(|s| s.starts_with("registry+"))
                })
                .count(),
        ),
    ]
    .into_iter()
    .collect();
    println!("{} packages in Cargo.lock: {located:?}", packages.len());
    assert!(
        located["members"] > 0 && located["registry"] > 0,
        "the reader located no package of one kind: {located:?}"
    );
    assert!(
        members.contains("mineworld-sdk") && members.contains("mineworld-inventory"),
        "the published surface's crates are members: {members:?}"
    );
    let refused = refused_sources(&packages, &members);
    assert!(refused.is_empty(), "{refused:#?}");
}

/// A fixed lock, the shape the real one has, for the negative controls.
const LOCK: &str = "# This file is automatically @generated by Cargo.
# It is not intended for manual editing.
version = 4

[[package]]
name = \"acme-pack\"
version = \"0.1.0\"
source = \"git+https://example.org/acme/pack?rev=0123456789abcdef0123456789abcdef01234567#0123456789abcdef0123456789abcdef01234567\"
dependencies = [
 \"mineworld-sdk\",
 \"serde\",
]

[[package]]
name = \"mineworld-sdk\"
version = \"0.1.0\"
dependencies = [
 \"serde 1.0.229\",
]

[[package]]
name = \"serde\"
version = \"1.0.229\"
source = \"registry+https://github.com/rust-lang/crates.io-index\"
checksum = \"0000000000000000000000000000000000000000000000000000000000000000\"
";

fn fixed_members() -> BTreeSet<String> {
    ["mineworld-sdk".to_owned()].into_iter().collect()
}

#[test]
fn the_fixed_lock_passes_and_reads_as_written() {
    let packages = read_lock(LOCK).expect("reads");
    assert_eq!(
        packages
            .iter()
            .map(|p| (p.name.as_str(), p.dependencies.len()))
            .collect::<Vec<_>>(),
        [("acme-pack", 2), ("mineworld-sdk", 1), ("serde", 0)]
    );
    assert_eq!(
        refused_sources(&packages, &fixed_members()),
        Vec::<String>::new()
    );
    let crlf = LOCK.replace('\n', "\r\n");
    assert_eq!(read_lock(&crlf).expect("reads with CRLF"), packages);
}

#[test]
fn a_framework_crate_from_a_registry_is_refused_by_name() {
    let mutated = LOCK.replace(
        "name = \"mineworld-sdk\"\nversion = \"0.1.0\"\n",
        "name = \"mineworld-sdk\"\nversion = \"0.1.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"00\"\n",
    );
    let refused = refused_sources(&read_lock(&mutated).expect("reads"), &BTreeSet::new());
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(refused[0].starts_with("mineworld-sdk: a framework crate from a registry"));
}

#[test]
fn a_git_source_not_pinned_to_one_full_commit_is_refused_by_name() {
    for (from, to) in [
        (
            "?rev=0123456789abcdef0123456789abcdef01234567#",
            "?branch=main#",
        ),
        (
            "?rev=0123456789abcdef0123456789abcdef01234567#",
            "?rev=0123456#",
        ),
        (
            "#0123456789abcdef0123456789abcdef01234567",
            "#ffffffffffffffffffffffffffffffffffffffff",
        ),
        (
            "?rev=0123456789abcdef0123456789abcdef01234567#",
            "?tag=v0.1.0#",
        ),
    ] {
        let mutated = LOCK.replace(from, to);
        assert_ne!(mutated, LOCK);
        let refused = refused_sources(&read_lock(&mutated).expect("reads"), &fixed_members());
        assert_eq!(refused.len(), 1, "{to}: {refused:?}");
        assert!(refused[0].starts_with("acme-pack: source"), "{refused:?}");
    }
}

#[test]
fn a_registry_package_without_a_checksum_and_a_stray_path_package_are_refused() {
    let mutated = LOCK.replace(
        "checksum = \"0000000000000000000000000000000000000000000000000000000000000000\"\n",
        "",
    );
    let refused = refused_sources(&read_lock(&mutated).expect("reads"), &fixed_members());
    assert_eq!(refused, ["serde: a registry package without a checksum"]);
    let refused = refused_sources(&read_lock(LOCK).expect("reads"), &BTreeSet::new());
    assert_eq!(
        refused,
        ["mineworld-sdk: no source, and not a workspace member"]
    );
}

#[test]
fn a_line_the_reader_does_not_understand_fails_naming_it() {
    let mutated = LOCK.replace("checksum = ", "checksum: ");
    let error = read_lock(&mutated).expect_err("refused");
    assert!(error.contains("checksum: "), "{error}");
    let mutated = LOCK.replace("version = 4", "version = 4\nsomething = 1");
    assert!(read_lock(&mutated).is_err());
}
