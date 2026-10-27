//! Market Town settles atomically across kills and renames its coin with no code (S31-a, `ARC-111`,
//! `ARC-112`; AC-a-4, AC-a-6, AC-a-7's binary arm, and AC-a-9's measurement).
//!
//! ```text
//! kills        run … --days 10 --save K, SIGKILLed at 20 wall offsets (SplitMix64 seed 31, uniform in
//!              [0.05, 0.95] × the control's wall time), each re-run by a new process: every save holds
//!              atomic sales and its money, and equals the control byte for byte
//! rename       a copy whose currency is { crown, cr, 0, after }: the same facts but the one
//!              currency-declared payload
//! undeclared   the copy without configure/economy.yaml fails `validate` naming the code
//! vocabulary   (ignored; S31A_BASE_SAVE) the base save's journal choices equal the head's
//! ```
//!
//! What would make the kills pass without proving anything, and how each is excluded (`ARC-23`): the
//! victim was never killed — its status is a signal and the save's head is short of the control's; no
//! kill re-executed anything — at least five kills must land with a non-empty re-executed tail.

mod headless;
mod market;

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use headless::{BINARY, Tables, fresh, mineworld, stderr, stdout};
use market::{MARKET_TOWN, genesis_money, run_pack, sales_are_atomic};
use mineworld_contracts::EventEnvelope;
use mineworld_test_support::process;

const SEED: u64 = 7;

/// SplitMix64 (Steele, Lea and Flood 2014): the kill offsets, reproducibly.
struct SplitMix64(u64);

impl SplitMix64 {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    fn unit(&mut self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let value = (self.next() >> 11) as f64 / (1_u64 << 53) as f64;
        value
    }
}

fn facts_of(tables: &Tables) -> Vec<EventEnvelope> {
    tables
        .facts
        .iter()
        .map(|(_, bytes)| {
            mineworld_persistence::format::decode(bytes, "fact").expect("a stored fact decodes")
        })
        .collect()
}

/// The arguments of the one command every run here repeats.
fn arguments(pack: &Path, days: u64, save: &Path) -> Vec<String> {
    vec![
        "run".to_owned(),
        pack.to_str().expect("a path").to_owned(),
        "--headless".to_owned(),
        "--seed".to_owned(),
        SEED.to_string(),
        "--days".to_owned(),
        days.to_string(),
        "--save".to_owned(),
        save.to_str().expect("a path").to_owned(),
    ]
}

/// Starts the run and SIGKILLs it after `after`; whether it was killed before it finished.
fn killed_after(save: &Path, after: Duration) -> bool {
    let mut child = Command::new(BINARY)
        .args(arguments(Path::new(MARKET_TOWN), 10, save))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("the binary starts");
    std::thread::sleep(after);
    process::kill(&mut child).killed()
}

/// A save's money and sales hold: Σ wallets moved only by atomic sales and wages (I-a-3, I-a-5).
fn sound(save: &Path, what: &str) -> Tables {
    let tables = Tables::read(save);
    let facts = facts_of(&tables);
    assert!(genesis_money(&facts) > 0, "{what}: the town is funded");
    sales_are_atomic(&facts);
    tables
}

/// AC-a-4: no half-trade across a kill.
#[test]
fn market_town_killed_anywhere_resumes_with_every_sale_whole() {
    let control = fresh("settlement-control");
    let started = Instant::now();
    run_pack(Path::new(MARKET_TOWN), SEED, 10, Some(&control));
    let wall = started.elapsed();
    let control_tables = sound(&control, "the control");
    let control_head = control_tables.journal.len();
    assert!(
        sales_are_atomic(&facts_of(&control_tables)) > 0,
        "the control sells something"
    );

    let mut offsets = SplitMix64(31);
    let mut re_executed = 0;
    for kill in 0..20 {
        let fraction = 0.05 + 0.9 * offsets.unit();
        let save = fresh(&format!("settlement-killed-{kill}"));
        if !killed_after(&save, wall.mul_f64(fraction)) {
            eprintln!("kill {kill} at {fraction:.2}: the run finished first");
            continue;
        }
        if !save.exists() {
            eprintln!("kill {kill} at {fraction:.2}: killed before the save existed");
            continue;
        }
        let at_kill = sound(&save, &format!("kill {kill}, as left"));
        assert!(
            at_kill.journal.len() <= control_head,
            "kill {kill}: no longer than the control"
        );
        let output = mineworld(
            &arguments(Path::new(MARKET_TOWN), 10, &save)
                .iter()
                .map(String::as_str)
                .collect::<Vec<_>>(),
        );
        assert!(
            output.status.success(),
            "kill {kill}: the re-run succeeds: {}",
            stderr(&output)
        );
        if stdout(&output)
            .lines()
            .next()
            .is_some_and(|header| header.contains("resumed") && !header.contains("+ 0 re-executed"))
        {
            re_executed += 1;
        }
        sound(&save, &format!("kill {kill}, resumed"))
            .assert_same_history(&control_tables, &format!("kill {kill}"));
    }
    assert!(
        re_executed >= 5,
        "at least five kills re-executed a tail: {re_executed}"
    );
}

/// A scratch copy of Market Town, its `configure/economy.yaml` rewritten by `edit`.
fn copy_with(name: &str, edit: impl FnOnce(&Path)) -> (headless::Scratch, PathBuf) {
    let scratch = fresh(name);
    std::fs::create_dir_all(scratch.path()).expect("a directory");
    let world = scratch.join("market-town");
    copy_dir(Path::new(MARKET_TOWN), &world);
    edit(&world);
    (scratch, world)
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("a directory");
    for entry in std::fs::read_dir(from).expect("the pack lists") {
        let entry = entry.expect("an entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("a type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("a copy");
        }
    }
}

/// The world's economy file with its `currency:` block replaced.
fn with_currency(world: &Path, currency: &str) {
    let file = world.join("configure").join("economy.yaml");
    let text = std::fs::read_to_string(&file).expect("readable");
    let start = text.find("currency:").expect("a currency block");
    let end = text[start..]
        .find("\n\n")
        .map_or(text.len(), |offset| start + offset);
    let replaced = format!("{}{currency}{}", &text[..start], &text[end..]);
    std::fs::write(&file, replaced).expect("writable");
}

/// AC-a-6: the currency renames with zero code — the facts differ only in `currency-declared`.
#[test]
fn market_town_renames_its_coin_and_nothing_else_changes() {
    let original = fresh("settlement-dollar");
    run_pack(Path::new(MARKET_TOWN), SEED, 30, Some(&original));
    let (_scratch, crown) = copy_with("settlement-crown", |world| {
        with_currency(
            world,
            "currency:\n  name: crown\n  symbol: cr\n  exponent: 0\n  symbol_at: after",
        );
    });
    let renamed = fresh("settlement-crown-save");
    run_pack(&crown, SEED, 30, Some(&renamed));
    let (ours, theirs) = (
        facts_of(&Tables::read(&original)),
        facts_of(&Tables::read(&renamed)),
    );
    assert_eq!(ours.len(), theirs.len(), "as many facts");
    let mut differing = Vec::new();
    for (a, b) in ours.iter().zip(&theirs) {
        if a != b {
            differing.push(a.event_type().as_str().to_owned());
        }
    }
    assert_eq!(
        differing,
        ["currency-declared"],
        "only the declaration of the currency differs"
    );
}

/// AC-a-7's binary arm: without its currency the copy fails `validate` naming the code.
#[test]
fn market_town_without_its_currency_fails_validate_by_name() {
    let (_scratch, world) = copy_with("settlement-undeclared", |world| {
        std::fs::remove_file(world.join("configure").join("economy.yaml")).expect("removed");
        let manifest = world.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("readable");
        std::fs::write(
            &manifest,
            text.replacen("  - economy\n  - sleep", "  - sleep", 1),
        )
        .expect("writable");
    });
    let output = mineworld(&["validate", world.to_str().expect("a path")]);
    let said = format!("{}{}", stdout(&output), stderr(&output));
    assert!(!output.status.success(), "refused: {said}");
    assert!(
        said.contains("economy-currency-undeclared"),
        "names the code: {said}"
    );
}

/// AC-a-9 (a measurement, run by hand at the milestone): Market Town's journal choices on the base
/// save (`S31A_BASE_SAVE`, written by the base binary, `pr-s31a-currency-asks.md` §15 Evidence) equal
/// the head's — the same (actor, action type, instant, outcome kind) in order.
#[test]
#[ignore = "a measurement against the base save; set S31A_BASE_SAVE"]
fn vocabulary_only() {
    let base = PathBuf::from(std::env::var("S31A_BASE_SAVE").expect("S31A_BASE_SAVE"));
    let head = fresh("settlement-vocabulary");
    run_pack(Path::new(MARKET_TOWN), SEED, 30, Some(&head));
    let choices = |save: &Path| -> Vec<String> {
        Tables::read(save)
            .journal
            .iter()
            .map(|(_, bytes)| {
                let entry: mineworld_persistence::JournalEntry =
                    mineworld_persistence::format::decode(bytes, "journal entry").expect("decodes");
                let input = format!("{:?}", entry.input);
                input.split(", payload").next().unwrap_or(&input).to_owned()
            })
            .collect()
    };
    assert_eq!(
        choices(&base),
        choices(&head),
        "the controllers chose the same"
    );
}
