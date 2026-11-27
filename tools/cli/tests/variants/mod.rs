//! Generated world variants for S35 N-h's acceptance (`pr-s35-nh-hands-and-errands.md` §5): built at test
//! time from the files the tests already read, never checked in. Each one changes the content generically
//! (every person, every shop, every routine part), so a fix that only makes one town pass cannot pass them.
//!
//! ```text
//! full-hands Commute Town   every resident starts holding six goods (no food, no drink); item-transfer
//!                           is installed, so putting a thing down is possible
//! late-shop Commute Town    every shop's stock starts at zero; every producing shift ends at 13:00
//! no-flag Market Town       every routine part's `punctual` flag is removed; jobs are unchanged
//! ```

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::headless::{Scratch, fresh};

pub const MARKET: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../../worlds/market-town");
pub const COMMUTE: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/worlds/commute-town");

/// A generated world: its scratch directory (kept alive) and the World Pack's path inside it.
pub struct Variant {
    _scratch: Scratch,
    pub path: PathBuf,
}

impl Variant {
    pub fn path(&self) -> &str {
        self.path.to_str().expect("a printable path")
    }
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copied");
        }
    }
}

/// `base` copied into a fresh scratch directory as `name`.
fn copied(base: &str, name: &str) -> Variant {
    // One scratch per call: two tests of one binary may build the same variant at once.
    static MADE: AtomicU64 = AtomicU64::new(0);
    let made = MADE.fetch_add(1, Ordering::Relaxed);
    let scratch = fresh(&format!("variant-{name}-{made}"));
    // A pack's id is its directory's name: the copy keeps the base's.
    let path = scratch.join(Path::new(base).file_name().expect("a named world"));
    copy_dir(Path::new(base), &path);
    Variant {
        _scratch: scratch,
        path,
    }
}

/// Every YAML file of `dir`, in name order.
fn files(dir: &Path) -> Vec<PathBuf> {
    let mut found: Vec<PathBuf> = std::fs::read_dir(dir)
        .expect("readable")
        .map(|entry| entry.expect("an entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yaml"))
        .collect();
    found.sort();
    found
}

/// Rewrites one file through `edit`.
fn rewrite(path: &Path, edit: impl Fn(&str) -> String) {
    let text = std::fs::read_to_string(path).expect("readable");
    std::fs::write(path, edit(&text)).expect("writable");
}

/// The lines of `text`, each through `edit` (a `None` drops the line).
fn lines(text: &str, edit: impl FnMut(&str) -> Option<String>) -> String {
    let mut out: String = text.lines().filter_map(edit).collect::<Vec<_>>().join("\n");
    out.push('\n');
    out
}

/// full-hands Commute Town: a goods kind, and six of it in every resident's hands.
pub fn full_hands() -> Variant {
    let variant = copied(COMMUTE, "full-hands");
    std::fs::write(
        variant.path.join("items/pebble.yaml"),
        "tags: [goods]\nitem: { category: goods, name: Pebble }\n",
    )
    .expect("writable");
    rewrite(&variant.path.join("world.yaml"), |text| {
        // The pebble kind, and item-transfer so that a full person has a way to put a thing down.
        let text = text.replacen("items:\n  - bread\n", "items:\n  - bread\n  - pebble\n", 1);
        if text.contains("  - item-transfer\n") {
            text
        } else {
            text.replacen("  - inventory\n", "  - inventory\n  - item-transfer\n", 1)
        }
    });
    for person in files(&variant.path.join("people")) {
        rewrite(&person, |text| {
            assert!(
                !text.contains("\nholdings:"),
                "{person:?} holds nothing yet"
            );
            format!("{text}\nholdings: {{ pebble: 6 }}\n")
        });
    }
    variant
}

/// late-shop Commute Town: no organization starts with stock, and every producing shift ends at 13:00.
pub fn late_shop() -> Variant {
    let variant = copied(COMMUTE, "late-shop");
    for organization in files(&variant.path.join("organizations")) {
        rewrite(&organization, |text| {
            lines(text, |line| {
                (!line.starts_with("holdings:")).then(|| line.to_owned())
            })
        });
    }
    for person in files(&variant.path.join("people")) {
        rewrite(&person, |text| {
            if !text.contains("produces:") {
                return text.to_owned();
            }
            lines(text, |line| {
                Some(if line.trim_start().starts_with("until:") {
                    "  until: \"13:00\"".to_owned()
                } else {
                    line.to_owned()
                })
            })
        });
    }
    variant
}

/// no-flag Market Town: no routine part is punctual.
pub fn no_flag() -> Variant {
    let variant = copied(MARKET, "no-flag");
    for person in files(&variant.path.join("people")) {
        rewrite(&person, |text| text.replace(", punctual: true", ""));
    }
    let text = std::fs::read_to_string(variant.path.join("people/alice.yaml")).expect("readable");
    assert!(!text.contains("punctual: true"), "every flag removed");
    variant
}

// ------------------------------------------------------------------ posture variants (S34-d1 SD-d-6)

/// Rewrites every place file's `sit-points:` section of `variant` through `edit`, which receives the
/// section's point lines in file order and returns the lines to write instead.
fn sit_points(variant: &Variant, edit: impl Fn(&[String]) -> Vec<String>) {
    for place in files(&variant.path.join("places")) {
        let text = std::fs::read_to_string(&place).expect("readable");
        if !text.contains("\nsit-points:\n") {
            continue;
        }
        let mut out = Vec::new();
        let mut section = Vec::new();
        let mut inside = false;
        for line in text.lines() {
            if inside && line.starts_with("  - ") {
                section.push(line.to_owned());
                continue;
            }
            if inside {
                out.extend(edit(&section));
                section.clear();
            }
            inside = line == "sit-points:";
            out.push(line.to_owned());
        }
        if inside {
            out.extend(edit(&section));
        }
        let mut written = out.join("\n");
        written.push('\n');
        std::fs::write(&place, written).expect("writable");
    }
}

/// The value of `field: value,` or `field: value }` in a flow mapping line.
fn flow_value<'a>(line: &'a str, field: &str) -> Option<&'a str> {
    let start = line.find(&format!("{field}: "))? + field.len() + 2;
    let rest = &line[start..];
    let end = rest.find([',', ' ', '}']).unwrap_or(rest.len());
    Some(&rest[..end])
}

/// renamed-seats: every sit point's key and group rewritten to names that share nothing with the
/// authored ones, and each place's list written in reverse order, so the key order is reversed too. A
/// test or a controller that names an authored key, or relies on the authored order, fails here.
pub fn renamed_seats(base: &str) -> Variant {
    let variant = copied(base, "renamed-seats");
    sit_points(&variant, |lines| {
        let n = lines.len();
        let mut renamed: Vec<String> = lines
            .iter()
            .enumerate()
            .map(|(index, line)| {
                let key = flow_value(line, "key").expect("every sit point has a key");
                // The last authored point gets the smallest new key: key order is reversed.
                let mut line = line.replacen(
                    &format!("key: {key}"),
                    &format!("key: q{:02}.seat", n - 1 - index),
                    1,
                );
                if let Some(group) = flow_value(&line, "group").map(str::to_owned) {
                    line = line.replacen(
                        &format!("group: {group}"),
                        &format!("group: renamed-{}", group.len()),
                        1,
                    );
                }
                line
            })
            .collect();
        renamed.reverse();
        renamed
    });
    variant
}

/// one-seat: every place keeps only the first sit point it lists, so contention is high.
pub fn one_seat(base: &str) -> Variant {
    let variant = copied(base, "one-seat");
    sit_points(&variant, |lines| lines.iter().take(1).cloned().collect());
    variant
}

/// no-posture: `posture` removed from `systems:` and every `sit-points:` section removed, as the world's
/// own comment says a world may (I-S34-6).
pub fn no_posture(base: &str) -> Variant {
    let variant = copied(base, "no-posture");
    rewrite(&variant.path.join("world.yaml"), |text| {
        assert!(text.contains("\n  - posture\n"), "{base} enables posture");
        text.replacen("\n  - posture\n", "\n", 1)
    });
    for place in files(&variant.path.join("places")) {
        rewrite(&place, |text| {
            let mut skipping = false;
            lines(text, |line| {
                if line == "sit-points:" {
                    skipping = true;
                    return None;
                }
                if skipping && line.starts_with("  - ") {
                    return None;
                }
                skipping = false;
                Some(line.to_owned())
            })
        });
    }
    variant
}
