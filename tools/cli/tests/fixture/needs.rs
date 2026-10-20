//! A test copy of a world with needs and sleep switched on (`pr-s35-nc-controllers.md` §6): the reference
//! world is never edited, and N-d later replaces this helper with real content.
//!
//! ```text
//! systems    `consumption` appended when missing (nothing restores hunger otherwise), then `sleep` and `needs`
//! people     `needs: {}` on every seated person: the step's genesis levels (step-35 §5.6); a person no
//!            controller drives would only grow hungry and tired
//! items      `needs: { nourishment: { hunger } }` on each kind step-35 §5.5's menu names
//! holdings   every seated person's `holdings:` filled up to `up_to` things with `food` — the most
//!            nourishing kind on the menu the world declares — so that the eat band is seen; `rule` does
//!            not shop (step-35 §6.2), and stocking a town's larders is N-d's content
//! ```
//!
//! Nothing here names a world: it reads the copied manifest's lists, so any World Pack can be copied.

#![allow(dead_code)]

use std::path::Path;

use crate::headless::{Scratch, fresh};

/// Step-35 §5.5's nourishment per item kind (authored, fixed before any run). Kinds absent here, or at
/// 0, get no section.
pub const MENU: [(&str, u16); 8] = [
    ("soup", 600),
    ("sandwich", 650),
    ("bread", 500),
    ("cake", 400),
    ("croissant", 350),
    ("apple", 200),
    ("juice", 100),
    ("milk", 150),
];

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("writable");
    for entry in std::fs::read_dir(from).expect("readable") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copied");
        }
    }
}

fn read_lf(path: &Path) -> String {
    std::fs::read_to_string(path)
        .expect("reads")
        .replace("\r\n", "\n")
}

fn append(path: &Path, text: &str) {
    let mut body = read_lf(path);
    if !body.ends_with('\n') {
        body.push('\n');
    }
    body.push_str(text);
    body.push('\n');
    std::fs::write(path, body).expect("writes");
}

/// The entries of the manifest's top-level list `key` (`  - entry` lines, comments skipped), and the
/// index of the line after its last entry.
fn list(lines: &[&str], key: &str) -> (Vec<String>, usize) {
    let start = lines
        .iter()
        .position(|line| *line == format!("{key}:"))
        .unwrap_or_else(|| panic!("the manifest lists {key}"));
    let mut entries = Vec::new();
    let mut end = start + 1;
    for (index, line) in lines.iter().enumerate().skip(start + 1) {
        if let Some(entry) = line.strip_prefix("  - ") {
            entries.push(entry.trim().to_owned());
            end = index + 1;
        } else if !(line.trim().is_empty() || line.trim_start().starts_with('#')) {
            break;
        }
    }
    (entries, end)
}

/// A copy of the world at `world` named `name`, with `consumption` (if missing), `sleep` and `needs`
/// appended to its systems, the menu's nourishment on its kinds, and on every seated person `needs: {}`
/// holdings filled up to `up_to` things with the most nourishing kind on the menu the world declares.
pub fn with_needs(name: &str, world: &Path, up_to: u32) -> Scratch {
    let key = world
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a world directory");
    let scratch = fresh(name).within(key);
    copy_dir(world, &scratch);
    let manifest = scratch.join("world.yaml");
    let text = read_lf(&manifest);
    let lines: Vec<&str> = text.lines().collect();
    let (systems, end) = list(&lines, "systems");
    let mut added: Vec<&str> = Vec::new();
    if !systems.iter().any(|system| system == "consumption") {
        added.push("consumption");
    }
    added.extend(["sleep", "needs"]);
    let mut rewritten: Vec<String> = lines[..end].iter().map(|line| (*line).to_owned()).collect();
    rewritten.extend(added.iter().map(|system| format!("  - {system}")));
    rewritten.extend(lines[end..].iter().map(|line| (*line).to_owned()));
    std::fs::write(&manifest, rewritten.join("\n") + "\n").expect("writes");
    let lines: Vec<&str> = text.lines().collect();
    let (items, _) = list(&lines, "items");
    let food = MENU
        .iter()
        .filter(|(kind, _)| items.iter().any(|item| item == kind))
        .max_by_key(|(_, hunger)| *hunger)
        .map(|(kind, _)| *kind);
    for (kind, hunger) in MENU {
        if items.iter().any(|item| item == kind) {
            append(
                &scratch.join(format!("items/{kind}.yaml")),
                &format!("needs: {{ nourishment: {{ hunger: {hunger} }} }}"),
            );
        }
    }
    let (seats, _) = list(&lines, "seats");
    for person in &seats {
        let file = scratch.join(format!("people/{person}.yaml"));
        let mut body = read_lf(&file);
        // Holding up to nothing: the larder stays as the world wrote it (Q-S35-OP1's fetch check).
        if let Some(food) = food.filter(|_| up_to > 0) {
            body = with_food(&body, food, up_to);
        }
        std::fs::write(&file, body).expect("writes");
        append(&file, "needs: {}");
    }
    scratch
}

/// `body` with its `holdings:` (a one-line flow mapping) filled up to `up_to` things with `food`, when
/// the person holds none of it yet; a `holdings:` line appended when there is none.
fn with_food(body: &str, food: &str, up_to: u32) -> String {
    let mut found = false;
    let mut lines: Vec<String> = body
        .lines()
        .map(|line| match line.strip_prefix("holdings: {") {
            Some(rest) => {
                found = true;
                let held: u32 = rest
                    .trim_end_matches(['}', ' '])
                    .split(',')
                    .filter_map(|entry| entry.split_once(':')?.1.trim().parse::<u32>().ok())
                    .sum();
                let room = up_to.saturating_sub(held);
                if rest.contains(&format!("{food}:")) || room == 0 {
                    line.to_owned()
                } else {
                    format!("holdings: {{ {food}: {room},{rest}")
                }
            }
            None => line.to_owned(),
        })
        .collect();
    if !found {
        lines.push(format!("holdings: {{ {food}: {up_to} }}"));
    }
    lines.join("\n") + "\n"
}
