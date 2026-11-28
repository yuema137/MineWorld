//! A test copy of a world with needs and sleep switched **off** (`pr-s35-nd-content.md` §6, AC-S35-2 (b)).
//!
//! N-c's copy that switched the packs on is gone: Market Town and Commute Town carry them as content
//! since N-d. What stays is its removal form — the step's §7 "both removed" row:
//!
//! ```text
//! systems, configure   the `sleep` and `needs` entries dropped
//! configure/           sleep.yaml and needs.yaml deleted
//! people/, items/      every `needs:` line dropped (each is a one-line section in the worlds that
//!                      carry one)
//! ```
//!
//! Everything else — supply, prices, wallets — is kept, so the copy is the base world plus N-d's sizing.
//! Nothing here names a world: it reads the copied files, so any World Pack can be copied.

#![allow(dead_code)]

use std::path::Path;

use crate::headless::{Scratch, fresh};

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

/// `path` without the lines `drop` selects; returns how many it dropped.
fn without_lines(path: &Path, drop: impl Fn(&str) -> bool) -> usize {
    let text = read_lf(path);
    let kept: Vec<&str> = text.lines().filter(|line| !drop(line)).collect();
    let dropped = text.lines().count() - kept.len();
    std::fs::write(path, kept.join("\n") + "\n").expect("writes");
    dropped
}

/// An unchanged copy of the world at `world` named `name`, for a test that needs the packs on. The worlds
/// carry them as content since N-d, so the copy changes nothing; it fails if the world does not enable them.
pub fn copied(name: &str, world: &Path) -> Scratch {
    let key = world
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a world directory");
    let scratch = fresh(name).within(key);
    copy_dir(world, &scratch);
    let manifest = read_lf(&scratch.join("world.yaml"));
    assert!(
        manifest.lines().any(|line| line == "  - sleep")
            && manifest.lines().any(|line| line == "  - needs"),
        "{key} enables sleep and needs"
    );
    scratch
}

/// A copy of the world at `world` named `name`, with `sleep` and `needs` removed with their files and
/// sections. Fails if the world did not enable them: a removal of nothing proves nothing.
pub fn without_needs(name: &str, world: &Path) -> Scratch {
    let key = world
        .file_name()
        .and_then(|name| name.to_str())
        .expect("a world directory");
    let scratch = fresh(name).within(key);
    copy_dir(world, &scratch);
    let dropped = without_lines(&scratch.join("world.yaml"), |line| {
        line == "  - sleep" || line == "  - needs"
    });
    assert!(dropped >= 2, "{key} enables sleep and needs");
    for file in ["configure/sleep.yaml", "configure/needs.yaml"] {
        let path = scratch.join(file);
        if path.exists() {
            std::fs::remove_file(path).expect("removable");
        }
    }
    let mut sections = 0;
    for directory in ["people", "items"] {
        for entry in std::fs::read_dir(scratch.join(directory)).expect("lists") {
            let path = entry.expect("an entry").path();
            sections += without_lines(&path, |line| line.starts_with("needs:"));
        }
    }
    assert!(sections > 0, "{key} carries needs: sections");
    scratch
}

/// How a food-energy copy is made (`ARC-370`, PR FE-a): the base daily use and whether the currency is
/// replaced by a fictional exponent-0 one. Every value written is synthetic test content, never a retail
/// or nutrition claim.
#[derive(Debug, Clone, Copy)]
pub struct FoodEnergyCopy {
    /// `energy_kcal_per_day` in the base entry.
    pub daily_kcal: u32,
    /// Every needs person's `food_energy.initial_kcal`.
    pub initial_kcal: u32,
    /// Replace the currency with "Shell", symbol "§", exponent 0.
    pub whole_currency: bool,
}

/// The `configure/needs.yaml` a food-energy copy writes: the base daily use, tiredness as before.
fn food_energy_section(daily_kcal: u32) -> String {
    format!(
        "# A synthetic food-energy configuration made by tools/cli/tests/fixture/needs.rs (FE-a).\n\
         parameters:\n  \
           - hunger_model: food-energy\n    \
             energy_kcal_per_day: {daily_kcal}\n    \
             tiredness_per_hour: 60\n    \
             rest_per_hour: 125\n    \
             thresholds: [400, 600, 900]\n    \
             restore: {{ hunger: [eating], tiredness: [sleeping] }}\n"
    )
}

/// An item's food-energy form: its unitless amount reused as synthetic kcal per one serving.
fn energy_item(line: &str) -> Option<String> {
    let amount: String = line
        .strip_prefix("needs: { nourishment: { hunger: ")?
        .chars()
        .take_while(char::is_ascii_digit)
        .collect();
    Some(format!(
        "needs: {{ nourishment: {{ energy_kcal: {amount}, serving: {{ label: \"1 serving (synthetic)\", \
         quantity: {{ unit: g, amount: 100 }} }} }} }}"
    ))
}

/// A copy of the world at `world` named `name` switched to the food-energy model: its
/// `configure/needs.yaml` replaced, every person's `needs: {}` given `food_energy`, every item's
/// unitless nourishment rewritten as kcal per serving, and optionally a whole-unit currency. Fails if
/// the world carries none of those lines: a switch of nothing proves nothing.
pub fn food_energy(name: &str, world: &Path, copy: FoodEnergyCopy) -> Scratch {
    let scratch = copied(name, world);
    std::fs::write(
        scratch.join("configure/needs.yaml"),
        food_energy_section(copy.daily_kcal),
    )
    .expect("writes");
    let mut people = 0;
    let mut items = 0;
    for directory in ["people", "items"] {
        for entry in std::fs::read_dir(scratch.join(directory)).expect("lists") {
            let path = entry.expect("an entry").path();
            let text = read_lf(&path);
            let rewritten: Vec<String> = text
                .lines()
                .map(|line| {
                    if line == "needs: {}" {
                        people += 1;
                        format!(
                            "needs: {{ food_energy: {{ initial_kcal: {} }} }}",
                            copy.initial_kcal
                        )
                    } else if let Some(item) = energy_item(line) {
                        items += 1;
                        item
                    } else {
                        line.to_owned()
                    }
                })
                .collect();
            std::fs::write(&path, rewritten.join("\n") + "\n").expect("writes");
        }
    }
    assert!(people > 0 && items > 0, "the world carries needs sections");
    if copy.whole_currency {
        let path = scratch.join("configure/economy.yaml");
        let text = read_lf(&path)
            .replacen("  name: US dollar\n", "  name: Shell\n", 1)
            .replacen("  symbol: \"$\"\n", "  symbol: \"§\"\n", 1)
            .replacen("  exponent: 2\n", "  exponent: 0\n", 1);
        assert!(
            text.contains("  exponent: 0\n"),
            "the currency was replaced"
        );
        std::fs::write(&path, text).expect("writes");
    }
    scratch
}
