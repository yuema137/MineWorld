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
//! EC-2 (iii) each git package's commit is the one its line in systems/installed/Cargo.toml states
//! EC-2 (iv)  only the installed set depends on a git package
//! EC-4       genuinely outside, over `cargo metadata --locked --offline`: the git packages are exactly the
//!            installed capabilities that are not bundled; each is not a member, its manifest is not
//!            under the workspace root, its normal dependencies are the published surface (read from
//!            .cargo/config.toml) or registry crates, no dependency has a path, and its manifest says no
//!            `workspace = true`
//! EC-5       nothing in the framework names it: no *.rs and no Cargo.toml but the installed set's spells
//!            a git package's crate name; systems/installed/src/lib.rs names it once
//! ```
//!
//! The lock is read line by line, as `systems/installed/tests/installed.rs` reads its manifest, and a
//! line it cannot read fails the test rather than being skipped. `str::lines` strips a `\r`, so a
//! Windows checkout's CRLF lock reads the same; paths are compared with `Path::starts_with`, and `git
//! ls-files` always prints `/`. No pack is named here: a third-party pack is located as a package whose
//! source begins `git+`.

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
fn metadata(extra: &[&str]) -> serde_json::Value {
    let output = Command::new(env!("CARGO"))
        .args(["metadata", "--format-version", "1", "--offline"])
        .args(extra)
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
    let members = members(&metadata(&["--no-deps"]));
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

// ---------------------------------------------------------------------------------------------
// EC-2 (iii), (iv): the pin is stated once and only the installed set reaches a git package.
// ---------------------------------------------------------------------------------------------

/// The installed set's manifest, relative to the repository.
const INSTALLED_MANIFEST: &str = "systems/installed/Cargo.toml";

/// The git packages of a lock: (name, url, commit).
fn git_packages(packages: &[Locked]) -> Vec<(String, String, String)> {
    packages
        .iter()
        .filter_map(|package| {
            let (url, rev) = pinned_git(package.source.as_deref()?)?;
            Some((package.name.clone(), url.to_owned(), rev.to_owned()))
        })
        .collect()
}

/// The value of `key = "<value>"` inside one manifest line.
fn quoted<'a>(line: &'a str, key: &str) -> Option<&'a str> {
    let rest = &line[line.find(&format!("{key} = \""))? + key.len() + 4..];
    rest.split('"').next()
}

/// EC-2 (iii): each git package's url and commit are the ones its own line of the installed set's
/// manifest states — `<name> = { git = "<url>", rev = "<commit>" }` — exactly once.
fn refused_pins(packages: &[Locked], manifest: &str) -> Vec<String> {
    let mut refused = Vec::new();
    for (name, url, rev) in git_packages(packages) {
        let lines: Vec<&str> = manifest
            .lines()
            .map(str::trim)
            .filter(|line| line.starts_with(&format!("{name} = {{")))
            .collect();
        match lines.as_slice() {
            [line] if quoted(line, "git") == Some(&url) && quoted(line, "rev") == Some(&rev) => {}
            [line] => refused.push(format!(
                "{name}: Cargo.lock pins {url} at {rev}; {INSTALLED_MANIFEST} says {line}"
            )),
            other => refused.push(format!(
                "{name}: {} lines of {INSTALLED_MANIFEST} name it",
                other.len()
            )),
        }
    }
    refused
}

/// EC-2 (iv): nothing but the installed set depends on a git package.
fn refused_dependents(packages: &[Locked]) -> Vec<String> {
    let git: BTreeSet<String> = git_packages(packages)
        .into_iter()
        .map(|(name, ..)| name)
        .collect();
    packages
        .iter()
        .filter(|package| package.name != "mineworld-installed-systems")
        .flat_map(|package| {
            package
                .dependencies
                .iter()
                .filter(|dependency| git.contains(*dependency))
                .map(move |dependency| format!("{} depends on {dependency}", package.name))
        })
        .collect()
}

fn read(relative: &str) -> String {
    let path = repository().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

#[test]
fn each_git_package_is_pinned_where_the_installed_set_says_and_only_it_depends_on_one() {
    let packages = the_lock();
    let git = git_packages(&packages);
    println!("git packages: {git:?}");
    assert!(!git.is_empty(), "no third-party pack located in Cargo.lock");
    assert_eq!(
        refused_pins(&packages, &read(INSTALLED_MANIFEST)),
        Vec::<String>::new()
    );
    assert_eq!(refused_dependents(&packages), Vec::<String>::new());
}

#[test]
fn a_pin_that_differs_from_the_manifest_or_a_second_dependent_is_refused() {
    let packages = read_lock(LOCK).expect("reads");
    let line = "acme-pack = { git = \"https://example.org/acme/pack\", rev = \
                \"0123456789abcdef0123456789abcdef01234567\" }\n";
    assert_eq!(refused_pins(&packages, line), Vec::<String>::new());
    let moved = line.replace("0123456789abcdef0123456789abcdef01234567", &"f".repeat(40));
    assert!(refused_pins(&packages, &moved)[0].starts_with("acme-pack: Cargo.lock pins"));
    assert!(refused_pins(&packages, "")[0].starts_with("acme-pack: 0 lines"));
    let installed = LOCK.replace(
        "name = \"mineworld-sdk\"\nversion = \"0.1.0\"\ndependencies = [\n",
        "name = \"mineworld-sdk\"\nversion = \"0.1.0\"\ndependencies = [\n \"acme-pack\",\n",
    );
    assert_eq!(
        refused_dependents(&read_lock(&installed).expect("reads")),
        ["mineworld-sdk depends on acme-pack"]
    );
}

// ---------------------------------------------------------------------------------------------
// EC-4: genuinely outside.
// ---------------------------------------------------------------------------------------------

/// The published surface: the crate names `.cargo/config.toml`'s `[patch.crates-io]` maps.
fn published_surface(config: &str) -> BTreeSet<String> {
    let mut inside = false;
    let mut surface = BTreeSet::new();
    for line in config.lines().map(str::trim) {
        if line.starts_with('[') {
            inside = line == "[patch.crates-io]";
            continue;
        }
        if inside && !line.is_empty() && !line.starts_with('#') {
            let (name, _) = line
                .split_once(" = ")
                .unwrap_or_else(|| panic!(".cargo/config.toml: cannot read {line:?}"));
            surface.insert(name.to_owned());
        }
    }
    surface
}

/// Every way a git package of `metadata` is not genuinely outside the workspace, naming it.
fn refused_outside(
    metadata: &serde_json::Value,
    git: &BTreeSet<String>,
    surface: &BTreeSet<String>,
) -> Vec<String> {
    let text = |value: &serde_json::Value| value.as_str().unwrap_or_default().to_owned();
    let root = PathBuf::from(text(&metadata["workspace_root"]));
    let members: BTreeSet<String> = metadata["workspace_members"]
        .as_array()
        .into_iter()
        .flatten()
        .map(text)
        .collect();
    let mut refused = Vec::new();
    let mut seen = BTreeSet::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        let name = text(&package["name"]);
        if !git.contains(&name) {
            continue;
        }
        seen.insert(name.clone());
        if members.contains(&text(&package["id"])) {
            refused.push(format!("{name}: a workspace member"));
        }
        let manifest = PathBuf::from(text(&package["manifest_path"]));
        if manifest.starts_with(&root) {
            refused.push(format!(
                "{name}: its manifest {} is under the workspace",
                manifest.display()
            ));
        }
        for dependency in package["dependencies"].as_array().into_iter().flatten() {
            let wanted = text(&dependency["name"]);
            if dependency.get("path").is_some() {
                refused.push(format!("{name}: depends on {wanted} by path"));
            }
            if !dependency["kind"].is_null() || surface.contains(&wanted) {
                continue;
            }
            if wanted.starts_with("mineworld-") {
                refused.push(format!(
                    "{name}: depends on {wanted}, outside the published surface"
                ));
            } else if !text(&dependency["source"]).starts_with("registry+") {
                refused.push(format!("{name}: depends on {wanted}, not from a registry"));
            }
        }
        if let Ok(written) = std::fs::read_to_string(&manifest)
            && written
                .lines()
                .any(|line| line.contains("workspace = true"))
        {
            refused.push(format!("{name}: its manifest says workspace = true"));
        }
    }
    for missing in git.difference(&seen) {
        refused.push(format!("{missing}: in the lock but not in cargo metadata"));
    }
    refused
}

#[test]
fn every_third_party_pack_is_genuinely_outside_and_named_by_the_build_as_third_party() {
    let git: BTreeSet<String> = git_packages(&the_lock())
        .into_iter()
        .map(|(name, ..)| name)
        .collect();
    let third_party: BTreeSet<String> = mineworld_worldpack::catalog::AVAILABLE
        .iter()
        .filter(|capability| !capability.package().bundled())
        .map(|capability| capability.package().name().to_owned())
        .collect();
    println!("located: git {git:?}, third-party capabilities {third_party:?}");
    assert!(!git.is_empty(), "no third-party pack located");
    assert_eq!(
        git, third_party,
        "the lock's git packages are the build's third-party packs"
    );

    let surface = published_surface(&read(".cargo/config.toml"));
    assert!(
        surface.contains("mineworld-sdk"),
        "the published surface is read: {surface:?}"
    );
    let metadata = metadata(&["--locked"]);
    assert_eq!(
        refused_outside(&metadata, &git, &surface),
        Vec::<String>::new()
    );
}

#[test]
fn a_path_dependency_or_one_outside_the_surface_is_refused() {
    let surface: BTreeSet<String> = ["mineworld-sdk".to_owned()].into_iter().collect();
    let git: BTreeSet<String> = ["acme-pack".to_owned()].into_iter().collect();
    let registry = "registry+https://github.com/rust-lang/crates.io-index";
    let metadata = |dependencies: serde_json::Value, manifest: &str| {
        serde_json::json!({
            "workspace_root": "/work/mineworld",
            "workspace_members": ["path+file:///work/mineworld/sdk#mineworld-sdk@0.1.0"],
            "packages": [{
                "name": "acme-pack",
                "id": "git+https://example.org/acme/pack#acme-pack@0.1.0",
                "manifest_path": manifest,
                "dependencies": dependencies,
            }],
        })
    };
    let outside = "/home/me/.cargo/git/checkouts/pack/0123456/Cargo.toml";
    let fine = serde_json::json!([
        {"name": "mineworld-sdk", "source": registry, "kind": null},
        {"name": "serde", "source": registry, "kind": null},
        {"name": "mineworld-item", "source": registry, "kind": "dev"},
    ]);
    assert_eq!(
        refused_outside(&metadata(fine, outside), &git, &surface),
        Vec::<String>::new()
    );
    let by_path = serde_json::json!([
        {"name": "mineworld-sdk", "source": null, "kind": null, "path": "/work/mineworld/sdk"},
    ]);
    assert_eq!(
        refused_outside(&metadata(by_path, outside), &git, &surface),
        ["acme-pack: depends on mineworld-sdk by path"]
    );
    let beyond = serde_json::json!([{"name": "mineworld-item", "source": registry, "kind": null}]);
    assert_eq!(
        refused_outside(&metadata(beyond, outside), &git, &surface),
        ["acme-pack: depends on mineworld-item, outside the published surface"]
    );
    let inside = refused_outside(
        &metadata(
            serde_json::json!([]),
            "/work/mineworld/vendor/pack/Cargo.toml",
        ),
        &git,
        &surface,
    );
    assert_eq!(inside.len(), 1, "{inside:?}");
    assert!(inside[0].contains("is under the workspace"));
}

// ---------------------------------------------------------------------------------------------
// EC-5: nothing in the framework names it.
// ---------------------------------------------------------------------------------------------

/// Every line of a tracked or untracked-but-not-ignored `*.rs` or `Cargo.toml` that names one of
/// `names` as a crate, as `path:line`, except the installed set's manifest; and how many times
/// `systems/installed/src/lib.rs` names each. A sparse checkout's absent files (`git ls-files -t`'s
/// `S`) are skipped: they are not in the working tree to be compiled.
///
/// How a crate is named depends on the file: a manifest names it as the package (`acme-x`) or the
/// library (`acme_x`); Rust source can name it only as the library, `acme_x` — in a path, a `use`, an
/// `extern crate` or a comment. The package spelling in a `*.rs` file is only ever a string, and
/// `packages/`' own tests use such strings as package-id fixtures that link nothing (step-16 §16.8
/// E-Ec6, finding F-Ec2).
fn spellings(names: &BTreeSet<String>) -> (Vec<String>, usize, usize) {
    let output = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "-t",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .current_dir(repository())
        .output()
        .expect("git runs");
    assert!(output.status.success(), "git ls-files failed");
    let listed = String::from_utf8(output.stdout).expect("UTF-8 paths");
    let spelled: Vec<String> = names
        .iter()
        .flat_map(|name| [name.clone(), name.replace('-', "_")])
        .collect();
    let (mut found, mut in_installed, mut scanned) = (Vec::new(), 0, 0);
    let mut seen = BTreeSet::new();
    for entry in listed.split('\0').filter(|entry| !entry.is_empty()) {
        let (tag, path) = entry.split_once(' ').expect("a tagged entry");
        let code = path.ends_with(".rs") || path == "Cargo.toml" || path.ends_with("/Cargo.toml");
        if tag == "S" || !code || path == INSTALLED_MANIFEST || !seen.insert(path.to_owned()) {
            continue;
        }
        scanned += 1;
        let text = read(path);
        for (number, line) in text.lines().enumerate() {
            let rust = path.ends_with(".rs");
            let hits = spelled
                .iter()
                .filter(|s| !(rust && s.contains('-')) && line.contains(s.as_str()))
                .count();
            if hits == 0 {
                continue;
            }
            if path == "systems/installed/src/lib.rs" {
                in_installed += hits;
            } else {
                found.push(format!("{path}:{}: {}", number + 1, line.trim()));
            }
        }
    }
    (found, in_installed, scanned)
}

#[test]
fn no_framework_code_names_a_third_party_pack_and_the_installed_set_names_each_once() {
    let names: BTreeSet<String> = git_packages(&the_lock())
        .into_iter()
        .map(|(name, ..)| name)
        .collect();
    assert!(!names.is_empty(), "no third-party pack located");
    let (found, in_installed, scanned) = spellings(&names);
    println!("{scanned} code files scanned for {names:?}");
    assert!(
        scanned > 100,
        "the scan located the repository's code: {scanned}"
    );
    assert_eq!(
        found,
        Vec::<String>::new(),
        "framework code names a third-party pack"
    );
    assert_eq!(
        in_installed,
        names.len(),
        "systems/installed/src/lib.rs names each once"
    );
}
