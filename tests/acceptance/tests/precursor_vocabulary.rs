//! **I-2: the precursors know no market** (`docs/DECISIONS.md` `ARC-35` item 7; step-10 §1.3).
//!
//! PRs 11a, 11b and 11c change the framework before the market exists — installable packs, items and
//! organizations as content, complete affordances — and are outside the range `AC-1` measures. They
//! are bounded instead: each must be justified without the market, so none may add a market concept.
//! This test reads every line each of them added and refuses any that names one.
//!
//! # What is scanned
//!
//! Every line a precursor added, and the path of every file it added, in every file except Markdown.
//! Code, comments, manifests, `Cargo.lock` and fixtures are all scanned. Documentation is not: it must
//! be able to discuss the market, and `ARC-35` does.
//!
//! # Which lines are "the PR's added lines"
//!
//! [`PRECURSORS`] records each precursor's base commit and branch.
//!
//! - **Merged:** if the first-parent history of `HEAD` holds the branch's merge commit `M` — subject
//!   `Merge pull request #N from <owner>/<branch>`, the form every merge into the protected `main`
//!   takes (`ARC-5`) — the range is `base..M^2`: what the PR itself added, whatever merged later.
//! - **Not merged:** the range is from `base` to the working tree — tracked changes against `base`,
//!   plus every untracked file Git does not ignore — so a line is scanned before it is committed.
//!
//! # Fail closed
//!
//! The test fails, naming the cause, when `git` cannot run, the directory is not a repository, a
//! recorded base is missing (a shallow clone), or `HEAD` does not descend from it. It never skips.
//!
//! # Extending it
//!
//! A precursor adds its row to [`PRECURSORS`] and its entries to [`ALLOWED`]. An allow-list entry
//! admits only a match that is not a market concept, and carries its reason; an entry with no reason,
//! or one that matches nothing, fails the test. An entry admits named **words**, not lines: every
//! other market word on a line it covers is still refused, and only this file may admit any word. A precursor that needs a market word is a material
//! stop for the operator, never an entry added to pass.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The market vocabulary. A word matches when it begins with one of these, so plurals and
/// derivations (`items`, `employer`, `ShopFront`) match too.
const VOCABULARY: &[&str] = &[
    "item",
    "inventory",
    "money",
    "price",
    "wage",
    "job",
    "shift",
    "shop",
    "economy",
    "employ",
];

/// One precursor PR: where it started, and the branch it merges from.
struct Precursor {
    pr: &'static str,
    base: &'static str,
    branch: &'static str,
}

/// Every precursor, in order. 11c adds its row.
const PRECURSORS: &[Precursor] = &[
    Precursor {
        pr: "11a",
        base: "b53e19d4182ebd2cc3357dacfe7d7248b29732ff",
        branch: "mvp0/pr-11a-installable",
    },
    Precursor {
        pr: "11b",
        base: "da316134e8bf8a82d1f65bbeaab62f3368222a3d",
        branch: "mvp0/pr-11b-content-kinds",
    },
];

/// This file. It alone may admit any word, because it names the vocabulary it looks for.
const THIS_SCAN: &str = "tests/acceptance/tests/precursor_vocabulary.rs";

/// The words an allow-list entry admits (`ARC-35`'s note of 2026-10-07: words, not lines).
enum Words {
    /// Every word. Legal only for [`THIS_SCAN`].
    Any,
    /// Exactly these lowercase words, and no word that merely begins with one of them.
    Only(&'static [&'static str]),
}

impl Words {
    fn admit(&self, word: &str) -> bool {
        match self {
            Words::Any => true,
            Words::Only(words) => words.contains(&word),
        }
    }
}

/// A match a precursor may add because it is not a market concept.
struct Allowed {
    pr: &'static str,
    /// The file, relative to the repository root.
    path: &'static str,
    /// A substring of the line; empty covers every line of the file.
    contains: &'static str,
    /// The market words the entry admits on the lines it covers; every other one is still refused.
    words: Words,
    reason: &'static str,
}

/// The allow-list, per precursor, each entry with its reason.
const ALLOWED: &[Allowed] = &[
    Allowed {
        pr: "11a",
        path: THIS_SCAN,
        contains: "",
        words: Words::Any,
        reason: "this scan: it names the vocabulary it looks for, and its own checks use it",
    },
    Allowed {
        pr: "11b",
        path: THIS_SCAN,
        contains: "",
        words: Words::Any,
        reason: "this scan: its allow-list names the words it admits",
    },
    Allowed {
        pr: "11b",
        path: "authoring/src/section.rs",
        contains: "",
        words: ITEM,
        reason: "ContentKind::Item, its directory `items` and its description `item` (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/src/format.rs",
        contains: "",
        words: ITEM,
        reason: "world.yaml's `items:` list and AuthoredItem (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/src/content.rs",
        contains: "",
        words: ITEM,
        reason: "ITEM_FIELDS and ContentFile::item (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/src/error.rs",
        contains: "",
        words: ITEM,
        reason: "Declared::Items, displayed `items`, and the kind's article `an item` (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/src/read.rs",
        contains: "",
        words: ITEM,
        reason: "the items map, its accessor, its reading and the per-file order (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/src/lib.rs",
        contains: "",
        words: ITEM,
        reason: "the layout diagram and the AuthoredItem re-export (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/tests/refusals.rs",
        contains: "",
        words: ITEM,
        reason: "refusal fixtures for items/<key>.yaml (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/src/load.rs",
        contains: "",
        words: ITEM,
        reason: "creating Item entities, their genesis order, and the probe tests (ARC-36)",
    },
    Allowed {
        pr: "11b",
        path: "worldpack/tests/content_kinds.rs",
        contains: "",
        words: ITEM,
        reason: "the loading fixture's item kinds (ARC-36)",
    },
];

/// The defined term `Item` (`CORE_CONCEPTS.md` §7, `EntityType::Item`) and its plural, the directory
/// and list name of `MODULE_SPEC.md` §4's frozen layout: the only words 11b admits (step-10 §4.2.2).
const ITEM: Words = Words::Only(&["item", "items"]);

/// One added line (or added file path) that names a market word.
#[derive(Debug)]
struct Hit {
    pr: &'static str,
    path: String,
    line: Option<u32>,
    word: String,
    text: String,
}

fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Runs `git` in the repository, or says why it could not.
fn git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repository())
        .args(["-c", "core.quotepath=off"])
        .args(args)
        .output()
        .map_err(|error| format!("`git` could not run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`git {}` failed: {}",
            args.join(" "),
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|error| format!("`git {}`: {error}", args.join(" ")))
}

/// The words of a line: split at every character that is not an ASCII letter or digit, and at every
/// lower-to-upper case boundary, lowercased.
fn words(line: &str) -> Vec<String> {
    let mut words = Vec::new();
    for run in line.split(|c: char| !c.is_ascii_alphanumeric()) {
        let mut word = String::new();
        let mut after_lower = false;
        for c in run.chars() {
            if c.is_ascii_uppercase() && after_lower {
                words.push(std::mem::take(&mut word));
            }
            after_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
            word.push(c.to_ascii_lowercase());
        }
        if !word.is_empty() {
            words.push(word);
        }
    }
    words
}

/// Every market word in `text`, in order.
fn market_words(text: &str) -> Vec<String> {
    words(text)
        .into_iter()
        .filter(|word| VOCABULARY.iter().any(|market| word.starts_with(market)))
        .collect()
}

/// The market words of one added line (or added path) that no entry admits. Each word is admitted
/// only by an entry for this precursor and file, whose substring the line contains, and which admits
/// that exact word; every entry that admits a word is marked in `used`.
fn refused_words(
    pr: &str,
    path: &str,
    text: &str,
    allowed: &[Allowed],
    used: &mut [bool],
) -> Vec<String> {
    let mut refused = Vec::new();
    for word in market_words(text) {
        let admitted = allowed.iter().position(|entry| {
            entry.pr == pr
                && entry.path == path
                && text.contains(entry.contains)
                && entry.words.admit(&word)
        });
        match admitted {
            Some(index) => used[index] = true,
            None => refused.push(word),
        }
    }
    refused
}

fn scanned(path: &str) -> bool {
    !path.to_ascii_lowercase().ends_with(".md")
}

/// The added lines of a `git diff --unified=0` and the paths of the files it adds, as
/// `(path, line number or None for the path itself, text)`.
fn added_in_diff(diff: &str) -> Vec<(String, Option<u32>, String)> {
    let mut added = Vec::new();
    let mut path = String::new();
    let mut in_header = false;
    let mut new_file = false;
    let mut next_line = 0_u32;
    for line in diff.lines() {
        if line.starts_with("diff --git ") {
            in_header = true;
            new_file = false;
            path.clear();
        } else if in_header {
            if line == "--- /dev/null" {
                new_file = true;
            } else if let Some(target) = line.strip_prefix("+++ ") {
                path = target.strip_prefix("b/").unwrap_or(target).to_owned();
                if new_file {
                    added.push((path.clone(), None, path.clone()));
                }
            } else if let Some(hunk) = line.strip_prefix("@@ ") {
                in_header = false;
                next_line = hunk_start(hunk);
            }
        } else if let Some(hunk) = line.strip_prefix("@@ ") {
            next_line = hunk_start(hunk);
        } else if let Some(text) = line.strip_prefix('+') {
            added.push((path.clone(), Some(next_line), text.to_owned()));
            next_line += 1;
        }
    }
    added
}

/// The first new-file line number of a hunk header `-a,b +c,d @@`.
fn hunk_start(hunk: &str) -> u32 {
    hunk.split_whitespace()
        .find_map(|part| part.strip_prefix('+'))
        .and_then(|range| range.split(',').next())
        .and_then(|start| start.parse().ok())
        .unwrap_or_else(|| panic!("an unreadable hunk header: @@ {hunk}"))
}

/// Where a precursor's range ends: its merged head, or the working tree while it is unmerged.
fn merged_head(precursor: &Precursor) -> Result<Option<String>, String> {
    let log = git(&[
        "log",
        "--first-parent",
        "--merges",
        "--format=%H %s",
        "HEAD",
    ])?;
    let suffix = format!("/{}", precursor.branch);
    Ok(log.lines().find_map(|line| {
        let (merge, subject) = line.split_once(' ')?;
        (subject.starts_with("Merge pull request #") && subject.ends_with(&suffix))
            .then(|| format!("{merge}^2"))
    }))
}

/// Every line and file path a precursor added, outside Markdown.
fn added_by(precursor: &Precursor) -> Result<Vec<(String, Option<u32>, String)>, String> {
    let base = precursor.base;
    git(&["cat-file", "-e", &format!("{base}^{{commit}}")]).map_err(|error| {
        format!(
            "{}: its base {base} is not in this repository's history (a shallow clone?): {error}",
            precursor.pr
        )
    })?;
    git(&["merge-base", "--is-ancestor", base, "HEAD"]).map_err(|error| {
        format!(
            "{}: HEAD does not descend from {base}: {error}",
            precursor.pr
        )
    })?;

    let diff_args = [
        "diff",
        "--unified=0",
        "--no-color",
        "--no-ext-diff",
        "--no-renames",
    ];
    let mut added = match merged_head(precursor)? {
        Some(head) => added_in_diff(&git(&[&diff_args[..], &[base, head.as_str()]].concat())?),
        None => {
            let mut added = added_in_diff(&git(&[&diff_args[..], &[base, "--"]].concat())?);
            let untracked = git(&["ls-files", "--others", "--exclude-standard", "-z"])?;
            for path in untracked.split('\0').filter(|path| !path.is_empty()) {
                added.push((path.to_owned(), None, path.to_owned()));
                let bytes = std::fs::read(repository().join(path))
                    .map_err(|error| format!("{path}: could not be read: {error}"))?;
                for (number, text) in String::from_utf8_lossy(&bytes).lines().enumerate() {
                    let number = u32::try_from(number + 1).expect("a file shorter than 2^32 lines");
                    added.push((path.to_owned(), Some(number), text.to_owned()));
                }
            }
            added
        }
    };
    added.retain(|(path, _, _)| scanned(path));
    Ok(added)
}

#[test]
fn the_precursors_add_no_market_concept() {
    let inside =
        git(&["rev-parse", "--is-inside-work-tree"]).unwrap_or_else(|error| panic!("{error}"));
    assert_eq!(
        inside.trim(),
        "true",
        "not a git work tree: {}",
        repository().display()
    );

    for entry in ALLOWED {
        assert!(
            !entry.reason.trim().is_empty(),
            "an allow-list entry with no reason: {}",
            entry.path
        );
        assert!(
            PRECURSORS.iter().any(|precursor| precursor.pr == entry.pr),
            "an allow-list entry for an unknown precursor: {}",
            entry.pr
        );
        assert!(
            matches!(entry.words, Words::Only(_)) || entry.path == THIS_SCAN,
            "an allow-list entry admits any word outside this scan's own file: {} ({})",
            entry.path,
            entry.pr
        );
    }

    let mut used = vec![false; ALLOWED.len()];
    let mut refused: Vec<Hit> = Vec::new();
    for precursor in PRECURSORS {
        let added = added_by(precursor).unwrap_or_else(|error| panic!("{error}"));
        for (path, line, text) in added {
            for word in refused_words(precursor.pr, &path, &text, ALLOWED, &mut used) {
                refused.push(Hit {
                    pr: precursor.pr,
                    path: path.clone(),
                    line,
                    word,
                    text: text.clone(),
                });
            }
        }
    }

    let report: Vec<String> = refused
        .iter()
        .map(|hit| match hit.line {
            Some(line) => format!(
                "{}: {}:{line}: `{}` in: {}",
                hit.pr,
                hit.path,
                hit.word,
                hit.text.trim()
            ),
            None => format!(
                "{}: the added file {} is named `{}`",
                hit.pr, hit.path, hit.word
            ),
        })
        .collect();
    assert!(
        report.is_empty(),
        "a precursor adds a market concept (I-2, ARC-35):\n{}",
        report.join("\n")
    );

    let stale: Vec<&str> = ALLOWED
        .iter()
        .zip(&used)
        .filter(|(_, used)| !**used)
        .map(|(entry, _)| entry.path)
        .collect();
    assert!(
        stale.is_empty(),
        "allow-list entries that match nothing: {stale:?}"
    );
}

/// The matcher sees the forms a market word takes in code, and nothing that merely contains one.
#[test]
fn the_matcher_sees_every_form_of_a_market_word() {
    for text in [
        "let items = held.len();",
        "struct ShopFront;",
        "fn pay_wages(employer: EntityId)",
        "const JOB_BOARD: &str = \"job-board\";",
        "  - economy",
        "worlds/market-town/places/shop.yaml",
    ] {
        assert!(!market_words(text).is_empty(), "missed: {text}");
    }
    for text in [
        "let iterate = 1;",
        "workshop",
        "a priority list",
        "SystemId",
    ] {
        let found = market_words(text);
        assert!(found.is_empty(), "matched {found:?} in: {text}");
    }
}

/// An entry admits the words it names and nothing else on the same line: an admitted `item` does not
/// hide `price` (`ARC-35`'s note of 2026-10-07, step-10 QS-16).
#[test]
fn an_admitted_word_admits_no_other() {
    let allowed = [Allowed {
        pr: "11x",
        path: "a.rs",
        contains: "",
        words: Words::Only(&["item", "items"]),
        reason: "the defined term",
    }];
    for (text, expected) in [
        ("let item_price = 1;", vec!["price"]),
        ("struct ItemPrice;", vec!["price"]),
        ("items: [wage]", vec!["wage"]),
        ("let itemprice = 1;", vec!["itemprice"]),
        ("let items = 1;", vec![]),
    ] {
        let mut used = [false];
        assert_eq!(
            refused_words("11x", "a.rs", text, &allowed, &mut used),
            expected,
            "in: {text}"
        );
    }

    let mut used = [false];
    assert_eq!(
        refused_words("11x", "b.rs", "let item = 1;", &allowed, &mut used),
        vec!["item"],
        "an entry admits only in its own file"
    );
    assert_eq!(used, [false]);
}

/// The diff reader finds added lines with their numbers, and the paths of added files.
#[test]
fn the_diff_reader_finds_added_lines_and_added_files() {
    let diff = "\
diff --git a/a.rs b/a.rs
index 1..2 100644
--- a/a.rs
+++ b/a.rs
@@ -3,0 +4,2 @@ fn f() {
+let one = 1;
++++ starts with plus signs
diff --git a/new.yaml b/new.yaml
new file mode 100644
--- /dev/null
+++ b/new.yaml
@@ -0,0 +1 @@
+key: value
";
    let added = added_in_diff(diff);
    assert_eq!(
        added,
        vec![
            ("a.rs".to_owned(), Some(4), "let one = 1;".to_owned()),
            (
                "a.rs".to_owned(),
                Some(5),
                "+++ starts with plus signs".to_owned()
            ),
            ("new.yaml".to_owned(), None, "new.yaml".to_owned()),
            ("new.yaml".to_owned(), Some(1), "key: value".to_owned()),
        ]
    );
}
