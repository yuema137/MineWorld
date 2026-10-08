//! **AC-1, the primary criterion** — Market Town is Social Café plus installed System Packs and
//! configuration, with no edit to the kernel, a controller or a renderer (`docs/MVP.md` §2, §9;
//! measured as `docs/DECISIONS.md` `ARC-35` decides, with its notes; step-10 §4.6 SD-29 … SD-32).
//!
//! Three independent checks, one `#[test]` each, so a failing check never hides another (`ARC-23`):
//!
//! ```text
//! check 1  the change set   the two transformation merges, each read against its own first parent:
//!                           only systems/, worlds/, Cargo.lock and Markdown documentation, and no
//!                           Cargo.lock package that is not a path crate under systems/
//! check 2  the structure    from `cargo metadata` at the working tree: only systems/ depends on a
//!                           market pack; no normal or build path leads to one from the framework;
//!                           no code outside systems/, worlds/, tests/acceptance/ names one
//! check 3  the world delta  Market Town is Social Café's configuration, unchanged, plus the six
//!                           market packs, items/, organizations/ and sections they own
//! ```
//!
//! # Fail closed
//!
//! Every check fails, naming the cause, when it cannot run: `git` missing, a directory that is not a
//! repository, a **shallow clone** (check 1 needs the full history: `fetch-depth: 0`), a merge that
//! cannot be found, a `cargo` that does not answer, a pack that does not read. None of them skips.
//!
//! Each check's core is a function over plain inputs — paths, lock text, metadata, YAML values —
//! tested here on hand-written inputs, and called by its `#[test]` with what the repository holds.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// One merge of the measured transformation (`ARC-35` item 1): the PR, its GitHub number and branch,
/// and the merge commit's id, recorded once it existed (the accepted limitation).
struct Transformation {
    pr: &'static str,
    number: u32,
    branch: &'static str,
    merge: &'static str,
}

/// The transformation: exactly these two merges (`ARC-35` item 1; its 11f note, point 1).
const TRANSFORMATION: &[Transformation] = &[
    Transformation {
        pr: "11d",
        number: 43,
        branch: "mvp0/pr-11d-owning-things",
        merge: "70e532f383ff81464d00066ff85e1b7fdc2296b0",
    },
    Transformation {
        pr: "11e",
        number: 46,
        branch: "mvp0/pr-11e-work-money-shops",
        merge: "2dddda86a4347d54de7bd2303d5268a421870b5f",
    },
];

/// The six market packs, by System Pack id (`ARC-35`'s 11e note). Their crates are `mineworld-<id>`.
const MARKET_PACKS: [&str; 6] = [
    "item",
    "inventory",
    "item-transfer",
    "economy",
    "employment",
    "consumption",
];

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("the repository root exists")
}

/// A market pack's crate name, as Cargo spells it.
fn crate_name(pack: &str) -> String {
    format!("mineworld-{pack}")
}

/// Runs `git -C repo args`, or says why it could not.
fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["-c", "core.quotepath=off"])
        .args(args)
        .output()
        .map_err(|error| format!("`git` could not run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed in {}: {}",
            args.join(" "),
            repo.display(),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("`git {}`: {error}", args.join(" ")))
}

// ── Check 1: the change set ──────────────────────────────────────────────────────────────────────

/// Whether `subject` is GitHub's subject for merging PR `number` from `<owner>/<branch>` — exactly:
/// `#43` is not `#430`, and the branch is the whole remainder after the owner.
fn is_merge_of(subject: &str, number: u32, branch: &str) -> bool {
    let Some(rest) = subject.strip_prefix(&format!("Merge pull request #{number} from ")) else {
        return false;
    };
    rest.split_once('/').is_some_and(|(owner, merged)| {
        !owner.is_empty() && !owner.contains(char::is_whitespace) && merged == branch
    })
}

/// The row's merge on `HEAD`'s first-parent chain: found by its subject exactly once, equal to the
/// recorded id, with two parents (`ARC-35`'s 11f note, point 1).
fn transformation_merge(repo: &Path, row: &Transformation) -> Result<String, String> {
    let log = git(
        repo,
        &[
            "log",
            "--first-parent",
            "--merges",
            "--format=%H %P%x09%s",
            "HEAD",
        ],
    )?;
    let found: Vec<(&str, usize)> = log
        .lines()
        .filter_map(|line| {
            let (ids, subject) = line.split_once('\t')?;
            let mut ids = ids.split(' ');
            let merge = ids.next()?;
            is_merge_of(subject, row.number, row.branch).then(|| (merge, ids.count()))
        })
        .collect();
    let [(merge, parents)] = found[..] else {
        return Err(format!(
            "{}: {} merges of #{} from <owner>/{} on the first-parent chain of HEAD, not exactly one: \
             {found:?}",
            row.pr,
            found.len(),
            row.number,
            row.branch
        ));
    };
    if merge != row.merge {
        return Err(format!(
            "{}: the merge of #{} on the first-parent chain is {merge}, not the recorded {}",
            row.pr, row.number, row.merge
        ));
    }
    if parents != 2 {
        return Err(format!(
            "{}: {merge} has {parents} parent(s), not 2 — a squash or a rebase leaves nothing to \
             measure against",
            row.pr
        ));
    }
    Ok(merge.to_owned())
}

/// Whether a path is inside `ARC-35` item 2's allowed set.
fn allowed(path: &str) -> bool {
    path.starts_with("systems/")
        || path.starts_with("worlds/")
        || path == "Cargo.lock"
        || (path.ends_with(".md")
            && (path.starts_with("docs/") || path.starts_with(".structured-coding/plans/")))
}

/// The paths a merge changed against its first parent, renames read as a deletion and an addition.
fn changed_paths(repo: &Path, merge: &str) -> Result<Vec<String>, String> {
    let diff = git(
        repo,
        &[
            "diff",
            "--name-only",
            "--no-renames",
            &format!("{merge}^1"),
            merge,
        ],
    )?;
    Ok(diff.lines().map(str::to_owned).collect())
}

/// One `[[package]]` block of a `Cargo.lock`: every field as written, a list joined by `, `.
#[derive(Debug, Clone, PartialEq, Eq)]
struct LockPackage {
    fields: BTreeMap<String, String>,
}

impl LockPackage {
    fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    fn name(&self) -> &str {
        self.field("name").unwrap_or("")
    }

    fn source(&self) -> Option<&str> {
        self.field("source")
    }
}

/// The packages of a `Cargo.lock`, keyed by `(name, version)` — a lock may hold two versions of one
/// name. A line reader: no TOML dependency is added for this (step-10 F-69).
fn read_lock(text: &str) -> BTreeMap<(String, String), LockPackage> {
    let mut packages = BTreeMap::new();
    let mut current: Option<BTreeMap<String, String>> = None;
    let mut list: Option<(String, Vec<String>)> = None;
    let mut finish = |block: Option<BTreeMap<String, String>>| {
        if let Some(fields) = block {
            let key = (
                fields.get("name").cloned().unwrap_or_default(),
                fields.get("version").cloned().unwrap_or_default(),
            );
            packages.insert(key, LockPackage { fields });
        }
    };
    for line in text.lines() {
        let line = line.trim_end();
        if let Some((key, items)) = &mut list {
            if line == "]" {
                if let Some(fields) = &mut current {
                    fields.insert(key.clone(), items.join(", "));
                }
                list = None;
            } else {
                items.push(
                    line.trim()
                        .trim_end_matches(',')
                        .trim_matches('"')
                        .to_owned(),
                );
            }
            continue;
        }
        if line.starts_with('[') {
            finish(current.take());
            current = (line == "[[package]]").then(BTreeMap::new);
            continue;
        }
        let (Some(fields), Some((key, value))) = (&mut current, line.split_once(" = ")) else {
            continue;
        };
        if value == "[" {
            list = Some((key.to_owned(), Vec::new()));
        } else {
            fields.insert(key.to_owned(), value.trim_matches('"').to_owned());
        }
    }
    finish(current);
    packages
}

/// One package a merge added, removed or changed in any field, and the package as it stands.
#[derive(Debug, PartialEq, Eq)]
struct LockChange {
    what: String,
    package: LockPackage,
}

/// Every package added, removed, or changed in any field between two locks (risk R-S9-6's wording;
/// `ARC-35`'s 11f note, point 2).
fn lock_changes(before: &str, after: &str) -> Vec<LockChange> {
    let (before, after) = (read_lock(before), read_lock(after));
    let mut changes = Vec::new();
    for (key, package) in &after {
        match before.get(key) {
            None => changes.push(LockChange {
                what: "added".to_owned(),
                package: package.clone(),
            }),
            Some(old) if old != package => {
                let fields: BTreeSet<&String> =
                    old.fields.keys().chain(package.fields.keys()).collect();
                let differ: Vec<&str> = fields
                    .into_iter()
                    .filter(|field| old.fields.get(*field) != package.fields.get(*field))
                    .map(String::as_str)
                    .collect();
                changes.push(LockChange {
                    what: format!("changed ({})", differ.join(", ")),
                    package: package.clone(),
                });
            }
            Some(_) => {}
        }
    }
    for (key, package) in &before {
        if !after.contains_key(key) {
            changes.push(LockChange {
                what: "removed".to_owned(),
                package: package.clone(),
            });
        }
    }
    changes
}

/// The lock changes `ARC-35` item 2 refuses: a package with a `source`, or one that is not a crate
/// under `systems/` at the merge.
fn refused_lock_changes(changes: &[LockChange], systems_crates: &BTreeSet<String>) -> Vec<String> {
    changes
        .iter()
        .filter(|change| {
            change.package.source().is_some() || !systems_crates.contains(change.package.name())
        })
        .map(|change| {
            format!(
                "Cargo.lock: {} {} {}{}",
                change.package.name(),
                change.package.field("version").unwrap_or("?"),
                change.what,
                change.package.source().map_or_else(
                    || " — not a crate under systems/".to_owned(),
                    |source| { format!(" — source = {source}") }
                )
            )
        })
        .collect()
}

/// The crate names under `systems/` at `revision`, read from its `systems/*/Cargo.toml`.
fn systems_crates(repo: &Path, revision: &str) -> Result<BTreeSet<String>, String> {
    let names = git(
        repo,
        &[
            "grep",
            "-h",
            "-E",
            "^name = \"[^\"]+\"$",
            revision,
            "--",
            "systems/*/Cargo.toml",
        ],
    )?;
    Ok(names
        .lines()
        .filter_map(|line| line.strip_prefix("name = \""))
        .map(|name| name.trim_end_matches('"').to_owned())
        .collect())
}

/// A file's text at a revision, or empty if the revision has no such file.
fn file_at(repo: &Path, revision: &str, path: &str) -> Result<String, String> {
    let listed = git(repo, &["ls-tree", "--name-only", revision, "--", path])?;
    if listed.trim().is_empty() {
        return Ok(String::new());
    }
    git(repo, &["show", &format!("{revision}:{path}")])
}

/// Check 1 over a repository: every failure, named. Empty means the check holds.
fn change_set_failures(repo: &Path, rows: &[Transformation]) -> Vec<String> {
    match git(repo, &["rev-parse", "--is-inside-work-tree"]) {
        Ok(inside) if inside.trim() == "true" => {}
        Ok(other) => return vec![format!("not a git work tree ({}): {other}", repo.display())],
        Err(error) => return vec![error],
    }
    match git(repo, &["rev-parse", "--is-shallow-repository"]) {
        Ok(shallow) if shallow.trim() == "false" => {}
        Ok(_) => {
            return vec![format!(
                "{} is a shallow clone: check 1 reads the transformation merges from the full \
                 history, which a shallow clone does not have. Fetch it all (`git fetch --unshallow`, \
                 or `fetch-depth: 0` in CI) — the check fails rather than skips",
                repo.display()
            )];
        }
        Err(error) => return vec![error],
    }
    let mut failures = Vec::new();
    for row in rows {
        let measured = transformation_merge(repo, row).and_then(|merge| {
            let mut found: Vec<String> = changed_paths(repo, &merge)?
                .into_iter()
                .filter(|path| !allowed(path))
                .map(|path| format!("a path outside the allowed set: {path}"))
                .collect();
            let before = file_at(repo, &format!("{merge}^1"), "Cargo.lock")?;
            let after = file_at(repo, &merge, "Cargo.lock")?;
            let crates = systems_crates(repo, &merge)?;
            found.extend(refused_lock_changes(
                &lock_changes(&before, &after),
                &crates,
            ));
            Ok((merge, found))
        });
        match measured {
            Ok((merge, found)) => failures.extend(
                found
                    .into_iter()
                    .map(|failure| format!("{} ({merge}): {failure}", row.pr)),
            ),
            Err(error) => failures.push(error),
        }
    }
    failures
}

#[test]
fn check_1_the_change_set() {
    let repo = repository();
    // What was measured, printed for the record; every failure is reported below, by the check.
    for row in TRANSFORMATION {
        let Ok(merge) = transformation_merge(&repo, row) else {
            continue;
        };
        let lock = lock_changes(
            &file_at(&repo, &format!("{merge}^1"), "Cargo.lock").expect("the lock before"),
            &file_at(&repo, &merge, "Cargo.lock").expect("the lock after"),
        );
        eprintln!(
            "{} {merge}: {} paths changed; Cargo.lock: {:?}",
            row.pr,
            changed_paths(&repo, &merge).expect("the diff").len(),
            lock.iter()
                .map(|change| format!("{} {}", change.package.name(), change.what))
                .collect::<Vec<_>>()
        );
    }
    let failures = change_set_failures(&repo, TRANSFORMATION);
    assert!(
        failures.is_empty(),
        "AC-1 check 1 (ARC-35 item 2) fails:\n{}",
        failures.join("\n")
    );
}

/// A scratch repository under the test target directory, made with `git init` and the given commits.
fn scratch_repository(name: &str, commits: &[&str]) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a scratch directory");
    git(&path, &["init", "--quiet", "--initial-branch=main"]).expect("git init");
    for (index, message) in commits.iter().enumerate() {
        std::fs::write(path.join("file.txt"), format!("{index}\n")).expect("a file");
        git(&path, &["add", "file.txt"]).expect("git add");
        git(
            &path,
            &[
                "-c",
                "user.name=scratch",
                "-c",
                "user.email=scratch@example.invalid",
                "commit",
                "--quiet",
                "-m",
                message,
            ],
        )
        .expect("git commit");
    }
    path
}

/// P-2: a history without the merges fails, naming the missing merge — it never passes vacuously.
#[test]
fn check_1_fails_closed_without_the_merges() {
    let repo = scratch_repository("ac1-one-commit", &["the only commit"]);
    let failures = change_set_failures(&repo, TRANSFORMATION);
    assert!(
        failures
            .iter()
            .any(|failure| failure.starts_with("11d: 0 merges of #43")),
        "{failures:?}"
    );
}

/// P-2: a shallow clone fails, naming the shallow history and `fetch-depth: 0` — it never skips.
#[test]
fn check_1_fails_closed_on_a_shallow_clone() {
    let origin = scratch_repository("ac1-two-commits", &["first", "second"]);
    let clone = Path::new(env!("CARGO_TARGET_TMPDIR")).join("ac1-shallow-clone");
    let _ = std::fs::remove_dir_all(&clone);
    let url = format!("file://{}", origin.display());
    git(
        &origin,
        &[
            "clone",
            "--quiet",
            "--depth",
            "1",
            &url,
            clone.to_str().expect("a path"),
        ],
    )
    .expect("a shallow clone");
    let failures = change_set_failures(&clone, TRANSFORMATION);
    assert_eq!(failures.len(), 1, "{failures:?}");
    assert!(
        failures[0].contains("shallow clone") && failures[0].contains("fetch-depth: 0"),
        "{failures:?}"
    );
}

/// The market pack list cannot go stale silently: each is a directory under `systems/` whose
/// manifest names its crate.
#[test]
fn every_market_pack_is_a_crate_under_systems() {
    for pack in MARKET_PACKS {
        let manifest = repository().join("systems").join(pack).join("Cargo.toml");
        let text = std::fs::read_to_string(&manifest)
            .unwrap_or_else(|error| panic!("{}: {error}", manifest.display()));
        assert!(
            text.lines()
                .any(|line| line == format!("name = \"{}\"", crate_name(pack))),
            "{} does not name {}",
            manifest.display(),
            crate_name(pack)
        );
    }
}

#[test]
fn the_subject_match_is_exact() {
    let branch = "mvp0/pr-11d-owning-things";
    assert!(is_merge_of(
        "Merge pull request #43 from yuema137/mvp0/pr-11d-owning-things",
        43,
        branch
    ));
    for subject in [
        "Merge pull request #430 from yuema137/mvp0/pr-11d-owning-things",
        "Merge pull request #4 from yuema137/mvp0/pr-11d-owning-things",
        "Merge pull request #43 from yuema137/mvp0/pr-11d-owning-things-2",
        "Merge pull request #43 from mvp0/pr-11d-owning-things",
        "Merge pull request #43 from /mvp0/pr-11d-owning-things",
        "Merge branch 'mvp0/pr-11d-owning-things'",
    ] {
        assert!(!is_merge_of(subject, 43, branch), "matched: {subject}");
    }
}

#[test]
fn the_allowed_set_is_arc_35_item_2() {
    for path in [
        "systems/economy/src/lib.rs",
        "worlds/market-town/people/alice.yaml",
        "Cargo.lock",
        "docs/DECISIONS.md",
        ".structured-coding/plans/mvp0/step-10-market.md",
    ] {
        assert!(allowed(path), "refused: {path}");
    }
    for path in [
        "kernel/src/lib.rs",
        "Cargo.toml",
        "docs/diagram.svg",
        "README.md",
        "server/PROTOCOL.md",
        ".structured-coding/standards.md",
        "cognition/rule-controller/src/paced.rs",
    ] {
        assert!(!allowed(path), "admitted: {path}");
    }
}

const LOCK_BEFORE: &str = r#"# This file is automatically @generated by Cargo.
version = 4

[[package]]
name = "itoa"
version = "1.0.15"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "aaaa"

[[package]]
name = "mineworld-installed-systems"
version = "0.0.0"
dependencies = [
 "mineworld-sdk",
]

[[package]]
name = "mineworld-sdk"
version = "0.0.0"
"#;

/// The lock reader reads the header, a package with no `dependencies` and a dependency list; the
/// change reader sees an added path package, a changed list, and a changed external checksum.
#[test]
fn the_lock_rule_refuses_any_change_that_is_not_a_path_crate_under_systems() {
    let packages = read_lock(LOCK_BEFORE);
    assert_eq!(packages.len(), 3, "{packages:?}");
    assert_eq!(
        packages[&("mineworld-installed-systems".to_owned(), "0.0.0".to_owned())]
            .field("dependencies"),
        Some("mineworld-sdk")
    );
    assert_eq!(
        packages[&("mineworld-sdk".to_owned(), "0.0.0".to_owned())].field("dependencies"),
        None
    );

    let after = LOCK_BEFORE
        .replace(
            " \"mineworld-sdk\",\n]",
            " \"mineworld-economy\",\n \"mineworld-sdk\",\n]",
        )
        .replace("checksum = \"aaaa\"", "checksum = \"bbbb\"")
        + "\n[[package]]\nname = \"mineworld-economy\"\nversion = \"0.0.0\"\n";
    let changes = lock_changes(LOCK_BEFORE, &after);
    let described: Vec<String> = changes
        .iter()
        .map(|change| format!("{} {}", change.package.name(), change.what))
        .collect();
    assert_eq!(
        described,
        [
            "itoa changed (checksum)",
            "mineworld-economy added",
            "mineworld-installed-systems changed (dependencies)",
        ]
    );
    let systems: BTreeSet<String> = ["mineworld-economy", "mineworld-installed-systems"]
        .map(str::to_owned)
        .into();
    let refused = refused_lock_changes(&changes, &systems);
    assert_eq!(refused.len(), 1, "{refused:?}");
    assert!(
        refused[0].starts_with("Cargo.lock: itoa 1.0.15 changed (checksum) — source = registry+")
    );

    let unknown: BTreeSet<String> = BTreeSet::new();
    assert_eq!(
        refused_lock_changes(&lock_changes(LOCK_BEFORE, &after), &unknown).len(),
        3,
        "a path package that is not a crate under systems/ is refused too"
    );
    assert!(lock_changes(LOCK_BEFORE, LOCK_BEFORE).is_empty());
    assert_eq!(
        lock_changes(LOCK_BEFORE, "version = 4\n")
            .iter()
            .filter(|change| change.what == "removed")
            .count(),
        3
    );
}

// ── Check 2: the structure ───────────────────────────────────────────────────────────────────────

/// The framework crates no normal or build path may lead from to a market pack (`ARC-35` item 3).
const FRAMEWORK: [&str; 7] = [
    "mineworld-kernel",
    "mineworld-contracts",
    "mineworld-persistence",
    "mineworld-server",
    "mineworld-authoring",
    "mineworld-sdk",
    "mineworld-rule-controller",
];

/// Where a code file may name a market crate (`ARC-35` item 3, third bullet).
const MAY_NAME_THE_MARKET: [&str; 3] = ["systems/", "worlds/", "tests/acceptance/"];

/// A dependency's kind, as `cargo metadata` states it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Normal,
    Build,
    Dev,
}

/// One workspace member: its directory relative to the root (with a trailing `/`), and what it
/// declares it depends on.
#[derive(Debug)]
struct Member {
    directory: String,
    dependencies: Vec<(String, Kind)>,
}

/// `cargo metadata --no-deps --format-version 1 --offline` for the workspace at `repo`, run through
/// the `cargo` that built this test. `--no-deps` is enough: only a member can depend on a path crate,
/// so every path to a market pack runs through members (step-10 F-60).
fn metadata(repo: &Path) -> Result<serde_json::Value, String> {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
        ])
        .current_dir(repo)
        .output()
        .map_err(|error| format!("`cargo metadata` could not run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`cargo metadata` failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("`cargo metadata`'s output does not parse: {error}"))
}

/// The members of a workspace, by crate name, read from a metadata value.
fn workspace(metadata: &serde_json::Value) -> Result<BTreeMap<String, Member>, String> {
    let text = |value: &serde_json::Value, what: &str| -> Result<String, String> {
        value
            .as_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("`cargo metadata`: no {what}"))
    };
    let root = text(&metadata["workspace_root"], "workspace_root")?;
    let packages = metadata["packages"]
        .as_array()
        .ok_or("`cargo metadata`: no packages")?;
    let mut members = BTreeMap::new();
    for package in packages {
        let name = text(&package["name"], "package name")?;
        let manifest = text(&package["manifest_path"], "manifest_path")?;
        let directory = manifest
            .strip_prefix(&root)
            .and_then(|rest| rest.strip_suffix("Cargo.toml"))
            .map(|rest| rest.trim_start_matches('/').to_owned())
            .ok_or_else(|| format!("{name}: {manifest} is not under {root}"))?;
        let mut dependencies = Vec::new();
        for dependency in package["dependencies"].as_array().into_iter().flatten() {
            let kind = match dependency["kind"].as_str() {
                None => Kind::Normal,
                Some("build") => Kind::Build,
                Some("dev") => Kind::Dev,
                Some(other) => return Err(format!("{name}: an unknown dependency kind {other}")),
            };
            dependencies.push((text(&dependency["name"], "dependency name")?, kind));
        }
        members.insert(
            name,
            Member {
                directory,
                dependencies,
            },
        );
    }
    Ok(members)
}

/// Bullet 1: every member that declares a market pack, as a dependency of any kind, lives under
/// `systems/`.
fn dependents_outside_systems(
    members: &BTreeMap<String, Member>,
    market: &BTreeSet<String>,
) -> Vec<String> {
    let mut found = Vec::new();
    for (name, member) in members {
        for (dependency, kind) in &member.dependencies {
            if market.contains(dependency) && !member.directory.starts_with("systems/") {
                found.push(format!(
                    "{name} ({}) depends on {dependency} ({kind:?}): only systems/ may",
                    member.directory
                ));
            }
        }
    }
    found
}

/// Bullet 2: every path over **normal and build** edges from a framework crate to a market pack
/// (`ARC-35`'s 11f note, point 3: the operator's QS-54). A framework crate the workspace does not
/// hold is a failure, so a renamed crate cannot silently drop out.
fn linked_paths(
    members: &BTreeMap<String, Member>,
    framework: &[&str],
    market: &BTreeSet<String>,
) -> Vec<String> {
    let mut found = Vec::new();
    for start in framework {
        if !members.contains_key(*start) {
            found.push(format!(
                "{start} is not a workspace member: it cannot be checked"
            ));
            continue;
        }
        let mut came_from: BTreeMap<&str, &str> = BTreeMap::new();
        let mut queue = std::collections::VecDeque::from([*start]);
        while let Some(crate_name) = queue.pop_front() {
            if market.contains(crate_name) {
                let mut path = vec![crate_name];
                while let Some(previous) = came_from.get(path[path.len() - 1]) {
                    path.push(previous);
                }
                path.reverse();
                found.push(format!(
                    "a linked path to a market pack: {}",
                    path.join(" → ")
                ));
                continue;
            }
            let Some(member) = members.get(crate_name) else {
                continue;
            };
            for (dependency, kind) in &member.dependencies {
                let linked = matches!(kind, Kind::Normal | Kind::Build);
                if linked
                    && dependency != start
                    && members.contains_key(dependency.as_str())
                    && !came_from.contains_key(dependency.as_str())
                {
                    came_from.insert(dependency, crate_name);
                    queue.push_back(dependency);
                }
            }
        }
    }
    found
}

/// Bullet 3, on one file's text: each line that contains one of `names` as a word — not inside a
/// longer crate name, so `mineworld-item` does not match in `mineworld-item-transfer`.
fn names_a_market_crate(text: &str, names: &[String]) -> Vec<(usize, String)> {
    let part_of_a_name = |c: char| c.is_ascii_alphanumeric() || c == '_' || c == '-';
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        for name in names {
            let named = line.match_indices(name.as_str()).any(|(at, _)| {
                let before = line[..at].chars().next_back();
                let after = line[at + name.len()..].chars().next();
                !before.is_some_and(part_of_a_name) && !after.is_some_and(part_of_a_name)
            });
            if named {
                found.push((index + 1, name.clone()));
            }
        }
    }
    found
}

/// Bullet 3 over the working tree: tracked files and untracked files Git does not ignore, so an
/// uncommitted edit is seen.
fn code_naming_the_market(repo: &Path, names: &[String]) -> Result<Vec<String>, String> {
    let listed = git(
        repo,
        &[
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ],
    )?;
    let mut found = Vec::new();
    for path in listed.split('\0').filter(|path| !path.is_empty()) {
        let code = path.ends_with(".rs") || path == "Cargo.toml" || path.ends_with("/Cargo.toml");
        if !code
            || MAY_NAME_THE_MARKET
                .iter()
                .any(|directory| path.starts_with(directory))
        {
            continue;
        }
        let text = match std::fs::read(repo.join(path)) {
            Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
            // Deleted in the working tree: it names nothing.
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("{path}: could not be read: {error}")),
        };
        for (line, name) in names_a_market_crate(&text, names) {
            found.push(format!("{path}:{line} names {name}"));
        }
    }
    Ok(found)
}

/// Both spellings of every market crate: `mineworld-economy` and `mineworld_economy`.
fn market_crate_spellings() -> Vec<String> {
    MARKET_PACKS
        .iter()
        .flat_map(|pack| [crate_name(pack), crate_name(pack).replace('-', "_")])
        .collect()
}

#[test]
fn check_2_the_dependency_structure() {
    let repo = repository();
    let members = workspace(&metadata(&repo).unwrap_or_else(|error| panic!("{error}")))
        .unwrap_or_else(|error| panic!("{error}"));
    let market: BTreeSet<String> = MARKET_PACKS.iter().map(|pack| crate_name(pack)).collect();
    for pack in &market {
        assert!(
            members.contains_key(pack),
            "{pack} is not a workspace member"
        );
        let dependents: Vec<&str> = members
            .iter()
            .filter(|(_, member)| member.dependencies.iter().any(|(name, _)| name == pack))
            .map(|(name, _)| name.as_str())
            .collect();
        eprintln!("{pack}: dependents {dependents:?}");
    }

    let mut failures = dependents_outside_systems(&members, &market);
    failures.extend(linked_paths(&members, &FRAMEWORK, &market));
    failures.extend(
        code_naming_the_market(&repo, &market_crate_spellings())
            .unwrap_or_else(|error| panic!("{error}")),
    );
    eprintln!(
        "{} workspace members read; the framework crates checked: {FRAMEWORK:?}",
        members.len()
    );
    assert!(
        failures.is_empty(),
        "AC-1 check 2 (ARC-35 item 3) fails:\n{}",
        failures.join("\n")
    );
}

/// QS-54 on F-58's shape: `persistence` dev-depends on `worldpack`, which links the installed set and
/// so a market pack. Dev edges are no path; the same edge as a normal one is; a dev dependency on a
/// market pack from outside `systems/` is still refused by bullet 1.
#[test]
fn a_dev_edge_is_no_linked_path_but_is_still_a_dependent() {
    let metadata = |persistence_on_worldpack: Option<&str>, persistence_on_economy: &str| {
        serde_json::json!({
            "workspace_root": "/r",
            "packages": [
                { "name": "mineworld-persistence", "manifest_path": "/r/persistence/Cargo.toml",
                  "dependencies": [
                      { "name": "mineworld-kernel", "kind": null },
                      { "name": "mineworld-worldpack", "kind": persistence_on_worldpack },
                      { "name": persistence_on_economy, "kind": "dev" },
                      { "name": "serde", "kind": null } ] },
                { "name": "mineworld-kernel", "manifest_path": "/r/kernel/Cargo.toml",
                  "dependencies": [] },
                { "name": "mineworld-worldpack", "manifest_path": "/r/worldpack/Cargo.toml",
                  "dependencies": [ { "name": "mineworld-installed-systems", "kind": null } ] },
                { "name": "mineworld-installed-systems",
                  "manifest_path": "/r/systems/installed/Cargo.toml",
                  "dependencies": [ { "name": "mineworld-economy", "kind": null } ] },
                { "name": "mineworld-economy", "manifest_path": "/r/systems/economy/Cargo.toml",
                  "dependencies": [ { "name": "mineworld-kernel", "kind": null } ] },
            ]
        })
    };
    let market: BTreeSet<String> = ["mineworld-economy".to_owned()].into();
    let framework = ["mineworld-persistence", "mineworld-kernel"];

    let members = workspace(&metadata(Some("dev"), "serde_json")).expect("reads");
    assert_eq!(
        members["mineworld-installed-systems"].directory,
        "systems/installed/"
    );
    assert!(dependents_outside_systems(&members, &market).is_empty());
    assert!(linked_paths(&members, &framework, &market).is_empty());

    for linked in [None, Some("build")] {
        let members = workspace(&metadata(linked, "serde_json")).expect("reads");
        assert_eq!(
            linked_paths(&members, &framework, &market),
            [
                "a linked path to a market pack: mineworld-persistence → mineworld-worldpack → \
              mineworld-installed-systems → mineworld-economy"
            ],
            "{linked:?}"
        );
    }

    let members = workspace(&metadata(Some("dev"), "mineworld-economy")).expect("reads");
    assert_eq!(
        dependents_outside_systems(&members, &market),
        [
            "mineworld-persistence (persistence/) depends on mineworld-economy (Dev): only systems/ may"
        ]
    );
    assert!(linked_paths(&members, &framework, &market).is_empty());

    assert_eq!(
        linked_paths(&members, &["mineworld-renamed"], &market),
        ["mineworld-renamed is not a workspace member: it cannot be checked"]
    );
}

/// A crate name matches as a word, in either spelling, and never inside a longer crate name.
#[test]
fn a_market_crate_is_named_only_as_a_word() {
    let names = ["mineworld-item".to_owned(), "mineworld_item".to_owned()];
    for text in [
        "mineworld-item = { path = \"../item\" }",
        "use mineworld_item::ItemKind;",
        "// see mineworld-item.",
    ] {
        assert_eq!(
            names_a_market_crate(text, &names).len(),
            1,
            "missed: {text}"
        );
    }
    for text in [
        "mineworld-item-transfer = { path = \"../item-transfer\" }",
        "use mineworld_item_transfer::Give;",
        "\"items-transferred\"",
        "xmineworld-item",
    ] {
        assert!(
            names_a_market_crate(text, &names).is_empty(),
            "matched: {text}"
        );
    }
}

// ── Check 3: the world delta ─────────────────────────────────────────────────────────────────────

const SOCIAL_CAFE: &str = "worlds/social-cafe";
const MARKET_TOWN: &str = "worlds/market-town";

/// The format's own fields of an item or organization file; anything else in one is a section
/// (`MODULE_SPEC.md` §4.1, `ARC-36`). A literal, not the loader's constant: the oracle is the spec.
const ENTITY_FIELDS: [&str; 2] = ["tags", "note"];

type Value = serde_json::Value;

/// A YAML file as a structural value: maps compare by key whatever their order, lists in order.
fn read_yaml(path: &Path) -> Result<Value, String> {
    let text = std::fs::read_to_string(path)
        .map_err(|error| format!("{}: could not be read: {error}", path.display()))?;
    serde_saphyr::from_str(&text).map_err(|error| format!("{}: {error}", path.display()))
}

/// Whether `key` is a section owned, by this build's own catalog, by one of the market packs. A key
/// no installed pack owns is not.
fn market_section(key: &str) -> bool {
    mineworld_worldpack::Capability::owning_section(key)
        .is_some_and(|(capability, _)| MARKET_PACKS.contains(&capability.id().as_str()))
}

/// `systems`: Social Café's list, in order, then exactly the six market packs, as a set.
fn compare_systems(social: Option<&Value>, market: Option<&Value>) -> Vec<String> {
    let names = |value: Option<&Value>| -> Option<Vec<String>> {
        value?
            .as_array()?
            .iter()
            .map(|name| name.as_str().map(str::to_owned))
            .collect()
    };
    let (Some(social), Some(market)) = (names(social), names(market)) else {
        return vec!["world.yaml: `systems` is not a list of names in both packs".to_owned()];
    };
    if !market.starts_with(&social) {
        return vec![format!(
            "world.yaml: `systems` does not begin with Social Café's list, in order: {market:?}"
        )];
    }
    let appended = &market[social.len()..];
    let set: BTreeSet<&str> = appended.iter().map(String::as_str).collect();
    if appended.len() == MARKET_PACKS.len() && set == MARKET_PACKS.into_iter().collect() {
        Vec::new()
    } else {
        vec![format!(
            "world.yaml: the systems appended to Social Café's are {appended:?}, not the six market \
             packs {MARKET_PACKS:?}"
        )]
    }
}

/// The two manifests: `world.id` and `world.name` may differ, `systems` as above, `items` and
/// `organizations` in Market Town only, every other key equal.
fn compare_manifests(social: &Value, market: &Value) -> Vec<String> {
    let (Some(social), Some(market)) = (social.as_object(), market.as_object()) else {
        return vec!["world.yaml: not a map in both packs".to_owned()];
    };
    let mut found = Vec::new();
    let keys: BTreeSet<&String> = social.keys().chain(market.keys()).collect();
    for key in keys {
        let (ours, theirs) = (social.get(key), market.get(key));
        match key.as_str() {
            "world" => {
                fn rest(value: Option<&Value>) -> Option<BTreeMap<&String, &Value>> {
                    value.and_then(Value::as_object).map(|world| {
                        world
                            .iter()
                            .filter(|(field, _)| *field != "id" && *field != "name")
                            .collect()
                    })
                }
                if rest(ours) != rest(theirs) {
                    found.push("world.yaml: `world` differs beyond its id and name".to_owned());
                }
            }
            "systems" => found.extend(compare_systems(ours, theirs)),
            "items" | "organizations" if ours.is_some() || theirs.is_none() => found.push(format!(
                "world.yaml: `{key}` must be absent in Social Café and present in Market Town"
            )),
            "items" | "organizations" => {}
            _ if ours != theirs => found.push(format!("world.yaml: `{key}` differs")),
            _ => {}
        }
    }
    found
}

/// A place or person file of both packs: each of Social Café's keys present with an equal value, and
/// each key Market Town adds a section a market pack owns.
fn compare_content(
    file: &str,
    social: &Value,
    market: &Value,
    owned_by_the_market: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let (Some(social), Some(market)) = (social.as_object(), market.as_object()) else {
        return vec![format!("{file}: not a map in both packs")];
    };
    let mut found = Vec::new();
    for (key, value) in social {
        match market.get(key) {
            None => found.push(format!("{file}: `{key}` is missing in Market Town")),
            Some(theirs) if theirs != value => {
                found.push(format!("{file}: `{key}` differs from Social Café's"));
            }
            Some(_) => {}
        }
    }
    for key in market.keys().filter(|key| !social.contains_key(*key)) {
        if !owned_by_the_market(key) {
            found.push(format!(
                "{file}: `{key}` is added, and is not a section a market pack owns"
            ));
        }
    }
    found
}

/// An item or organization file, which exists in Market Town only: every key is the format's own or
/// a section a market pack owns.
fn market_only_content(
    file: &str,
    market: &Value,
    owned_by_the_market: &dyn Fn(&str) -> bool,
) -> Vec<String> {
    let Some(market) = market.as_object() else {
        return vec![format!("{file}: not a map")];
    };
    market
        .keys()
        .filter(|key| !ENTITY_FIELDS.contains(&key.as_str()) && !owned_by_the_market(key))
        .map(|key| format!("{file}: `{key}` is not a section a market pack owns"))
        .collect()
}

/// The names in a directory, or an empty set if it does not exist.
fn entries(directory: &Path) -> Result<BTreeSet<String>, String> {
    if !directory.exists() {
        return Ok(BTreeSet::new());
    }
    std::fs::read_dir(directory)
        .map_err(|error| format!("{}: could not be listed: {error}", directory.display()))?
        .map(|entry| {
            entry
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .map_err(|error| format!("{}: {error}", directory.display()))
        })
        .collect()
}

/// Check 3 over the two packs under `root`: every difference that is not configuration, named by
/// file and key. Empty means the check holds.
fn world_delta_failures(root: &Path) -> Result<Vec<String>, String> {
    let (social, market) = (root.join(SOCIAL_CAFE), root.join(MARKET_TOWN));
    for pack in [&social, &market] {
        mineworld_worldpack::WorldPack::read(pack.as_path()).map_err(|error| {
            format!("{}: the World Pack does not read: {error}", pack.display())
        })?;
    }
    let mut found = compare_manifests(
        &read_yaml(&social.join("world.yaml"))?,
        &read_yaml(&market.join("world.yaml"))?,
    );

    let not_configuration = |name: &String| name != "README.md";
    let ours: BTreeSet<String> = entries(&social)?
        .into_iter()
        .filter(not_configuration)
        .collect();
    let theirs: BTreeSet<String> = entries(&market)?
        .into_iter()
        .filter(not_configuration)
        .collect();
    let market_only: BTreeSet<String> = ["items", "organizations"].map(str::to_owned).into();
    for name in ours.symmetric_difference(&theirs) {
        if !(market_only.contains(name) && theirs.contains(name)) {
            found.push(format!("{name}: present in one pack only"));
        }
    }
    for name in &market_only {
        if ours.contains(name) || !theirs.contains(name) {
            found.push(format!("{name}/: must exist in Market Town only"));
        }
    }

    for directory in ["places", "people"] {
        let (ours, theirs) = (
            entries(&social.join(directory))?,
            entries(&market.join(directory))?,
        );
        for file in ours.symmetric_difference(&theirs) {
            found.push(format!("{directory}/{file}: present in one pack only"));
        }
        for file in ours.intersection(&theirs) {
            found.extend(compare_content(
                &format!("{directory}/{file}"),
                &read_yaml(&social.join(directory).join(file))?,
                &read_yaml(&market.join(directory).join(file))?,
                &market_section,
            ));
        }
    }
    for directory in &market_only {
        for file in entries(&market.join(directory))? {
            found.extend(market_only_content(
                &format!("{directory}/{file}"),
                &read_yaml(&market.join(directory).join(&file))?,
                &market_section,
            ));
        }
    }
    Ok(found)
}

#[test]
fn check_3_the_world_delta() {
    let failures = world_delta_failures(&repository()).unwrap_or_else(|error| panic!("{error}"));
    let added: BTreeSet<String> = ["people", "places"]
        .into_iter()
        .flat_map(|directory| {
            let (social, market) = (
                repository().join(SOCIAL_CAFE).join(directory),
                repository().join(MARKET_TOWN).join(directory),
            );
            entries(&market)
                .expect("lists")
                .into_iter()
                .flat_map(move |file| {
                    let ours = read_yaml(&social.join(&file)).expect("reads");
                    let theirs = read_yaml(&market.join(&file)).expect("reads");
                    let ours = ours.as_object().cloned().unwrap_or_default();
                    theirs
                        .as_object()
                        .map(|map| {
                            map.keys()
                                .filter(|key| !ours.contains_key(*key))
                                .map(|key| format!("{directory}: {key}"))
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default()
                })
        })
        .collect();
    eprintln!("sections Market Town adds to Social Café's files: {added:?}");
    assert!(
        failures.is_empty(),
        "AC-1 check 3 (ARC-35 item 4) fails:\n{}",
        failures.join("\n")
    );
}

/// The section owners come from the build: the market packs' sections are recognised, Social Café's
/// are not, and a key no installed pack owns is not.
#[test]
fn a_section_is_the_market_s_by_the_build_s_own_catalog() {
    for key in ["item", "holdings", "economy", "job"] {
        assert!(market_section(key), "{key}");
    }
    for key in ["name", "routine", "location", "tags", "no-such-section"] {
        assert!(!market_section(key), "{key}");
    }
}

/// The comparison names a changed value, a removed key and an added key that is not the market's;
/// maps compare whatever their key order, and lists in order.
#[test]
fn the_world_delta_names_every_difference_by_file_and_key() {
    use serde_json::json;
    let market = |key: &str| key == "holdings";
    let social = json!({ "tags": ["a", "b"], "routine": [{ "from": "06:00" }], "note": "n" });
    assert!(
        compare_content(
            "people/x.yaml",
            &social,
            &json!({ "note": "n", "routine": [{ "from": "06:00" }], "tags": ["a", "b"],
                     "holdings": { "k": 1 } }),
            &market
        )
        .is_empty()
    );
    assert_eq!(
        compare_content(
            "people/x.yaml",
            &social,
            &json!({ "tags": ["b", "a"], "routine": [{ "from": "07:00" }], "job": {} }),
            &market
        ),
        [
            "people/x.yaml: `note` is missing in Market Town",
            "people/x.yaml: `routine` differs from Social Café's",
            "people/x.yaml: `tags` differs from Social Café's",
            "people/x.yaml: `job` is added, and is not a section a market pack owns",
        ]
    );
    assert_eq!(
        market_only_content(
            "items/k.yaml",
            &json!({ "tags": [], "routine": 1 }),
            &market
        ),
        ["items/k.yaml: `routine` is not a section a market pack owns"]
    );

    let systems = |list: Value| json!({ "world": { "id": "x", "name": "X" }, "systems": list });
    let base = systems(json!(["presence", "naming"]));
    let mut town = systems(json!([
        "presence",
        "naming",
        "economy",
        "item",
        "inventory",
        "item-transfer",
        "employment",
        "consumption"
    ]));
    town["world"] = json!({ "name": "Y", "id": "y" });
    town["items"] = json!(["k"]);
    town["organizations"] = json!(["o"]);
    assert!(compare_manifests(&base, &town).is_empty());
    let reordered = systems(json!([
        "naming",
        "presence",
        "item",
        "inventory",
        "item-transfer",
        "economy",
        "employment",
        "consumption"
    ]));
    assert_eq!(compare_manifests(&base, &reordered).len(), 1);
    let mut seats = town.clone();
    seats["seats"] = json!(["alice"]);
    assert_eq!(
        compare_manifests(&base, &seats),
        ["world.yaml: `seats` differs"]
    );
    let short = systems(json!(["presence", "naming", "item"]));
    assert!(compare_manifests(&base, &short)[0].contains("not the six market packs"));
}
