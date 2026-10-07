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
