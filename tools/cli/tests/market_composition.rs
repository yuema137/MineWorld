//! **`AC-2` at world level for every market pack** (step-10 §4.6 SD-35, P-7, CP-6 widened by QS-61):
//! each of the six packs Market Town installed, taken out again, leaves a world that runs, or one the
//! loader refuses by name — exactly as the packs' declared dependencies say (`ARC-37`, `ARC-38`).
//!
//! The route a World Pack has to leave a system out is its `systems:` list, and a world that does not
//! enable a section's owner refuses the section (`MODULE_SPEC.md` §4.1 rule 6). So each case is a
//! test-time copy of `worlds/market-town` whose `world.yaml` omits one pack and whose files omit the
//! section it owns — the only difference is the pack — run by the real binary.
//!
//! ```text
//! without item           refused: inventory depends on item
//! without inventory      refused: item-transfer, economy, employment and consumption depend on it
//! without item-transfer  runs; nobody gives; buying, eating or drinking, wages and production go on
//! without economy        runs; nothing is bought and no money moves; wages fall due and nobody pays
//!                        them; production, gives and meals go on
//! without employment     runs; nobody is hired, works, is owed a wage or produces; purchases, gives
//!                        and meals go on
//! without consumption    runs; nobody eats or drinks; purchases, wages, production and gives go on
//! ```
//!
//! "Nobody gives" is read from the requests the controllers made: the paced controller attempts an
//! available complete affordance whenever its offer band fires (`ARC-34`), so a `give` that was
//! offered would be requested; with `item-transfer` disabled none is requested and no person's item
//! moves to another by an action.

mod headless;
mod market;

use std::path::{Path, PathBuf};

use headless::{
    Scratch, Tables, every_seat_active_in_every_bucket, fresh, lines, mineworld, stderr,
};
use market::{MARKET_TOWN, MarketFact, Town, market_fact, run_pack};
use mineworld_contracts::{Causation, EventEnvelope};

const DAYS: u64 = 30;

/// Each market pack and the section it owns, if any.
const PACKS: [(&str, Option<&str>, usize); 6] = [
    ("item", Some("item"), 20),
    ("inventory", Some("holdings"), 14),
    ("item-transfer", None, 0),
    ("economy", Some("economy"), 14),
    ("employment", Some("job"), 2),
    ("consumption", None, 0),
];

/// `text` without its top-level `key:` and the indented lines that continue it; whether it had one.
fn strip_section(text: &str, key: &str) -> (String, bool) {
    let mut kept = Vec::new();
    let mut inside = false;
    let mut found = false;
    for line in text.lines() {
        if line.starts_with(&format!("{key}:")) {
            inside = true;
            found = true;
            continue;
        }
        if inside && (line.starts_with(' ') || line.starts_with('-')) {
            continue;
        }
        inside = false;
        kept.push(line);
    }
    (kept.join("\n") + "\n", found)
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

/// Every file of a pack, relative to it, with its text.
fn files(root: &Path) -> Vec<(PathBuf, String)> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).expect("lists") {
            let path = entry.expect("an entry").path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let text = std::fs::read_to_string(&path).expect("reads");
                found.push((path.strip_prefix(root).expect("inside").to_path_buf(), text));
            }
        }
    }
    found.sort();
    found
}

/// A copy of Market Town, named as the pack (a pack's id is its directory), without `pack` and the
/// section it owns; asserted, before it is used, to differ from Market Town by exactly those.
fn without(pack: &str, section: Option<&str>, carriers: usize) -> Scratch {
    let copy = fresh(&format!("market-without-{pack}")).within("market-town");
    copy_dir(Path::new(MARKET_TOWN), &copy);
    let manifest = copy.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("world.yaml reads");
    let line = format!("  - {pack}\n");
    assert_eq!(text.matches(&line).count(), 1, "Market Town enables {pack}");
    std::fs::write(&manifest, text.replace(&line, "")).expect("world.yaml writes");
    let mut stripped = 0;
    if let Some(section) = section {
        for (file, text) in files(&copy) {
            let (kept, found) = strip_section(&text, section);
            if found {
                stripped += 1;
                std::fs::write(copy.join(&file), kept).expect("writes");
            }
        }
    }
    assert_eq!(
        stripped, carriers,
        "{pack}: the files that carried `{section:?}`"
    );

    // The copy differs from Market Town by the pack's line and its section, and by nothing else.
    let original = files(Path::new(MARKET_TOWN));
    let copied = files(&copy);
    assert_eq!(
        original.iter().map(|(file, _)| file).collect::<Vec<_>>(),
        copied.iter().map(|(file, _)| file).collect::<Vec<_>>(),
        "{pack}: the same files"
    );
    for ((file, before), (_, after)) in original.iter().zip(&copied) {
        let expected = if file == Path::new("world.yaml") {
            before.replace(&line, "")
        } else if let Some(section) = section {
            let (kept, found) = strip_section(before, section);
            if found { kept } else { before.clone() }
        } else {
            before.clone()
        };
        assert_eq!(
            *after,
            expected,
            "{pack}: {} differs by more than the pack",
            file.display()
        );
        if let Some(section) = section {
            assert!(
                !after
                    .lines()
                    .any(|line| line.starts_with(&format!("{section}:"))),
                "{pack}: {} still carries `{section}:`",
                file.display()
            );
        }
    }
    copy
}

/// `validate` of a copy must fail, naming every word in `names`.
fn refused(pack: &Path, case: &str, names: &[&str]) {
    let output = mineworld(&["validate", pack.to_str().expect("path")]);
    let complaint = stderr(&output);
    assert!(
        !output.status.success(),
        "without {case}: validate was expected to refuse the world, and it accepted it"
    );
    for name in names {
        assert!(
            complaint.contains(name),
            "without {case}: the refusal does not name {name}: {complaint}"
        );
    }
    assert!(!complaint.contains("panicked"), "{complaint}");
    eprintln!(
        "without {case}: refused — {}",
        complaint.lines().next().unwrap_or("")
    );
}

/// What a 30-day run of a copy did, counted from its printed requests and its saved facts.
struct Ran {
    printed: String,
    facts: Vec<EventEnvelope>,
}

impl Ran {
    fn of(town: &Town, pack: &Path, case: &str) -> Self {
        let save = fresh(&format!("market-without-{case}-save"));
        let printed = run_pack(pack, 7, DAYS, Some(&save));
        assert_eq!(
            lines(&printed, "faults     0").len(),
            1,
            "without {case}: no fault: {printed}"
        );
        let seats: Vec<&str> = town.seats.iter().map(String::as_str).collect();
        assert_eq!(every_seat_active_in_every_bucket(&printed, &seats), 1);
        let facts = Tables::read(&save)
            .facts
            .iter()
            .map(|(_, bytes)| mineworld_persistence::format::decode(bytes, "fact").expect("a fact"))
            .collect();
        Self { printed, facts }
    }

    /// How many `action` requests were made, accepted or not.
    fn requested(&self, action: &str) -> u64 {
        lines(&self.printed, &format!("requests   {action} "))
            .iter()
            .map(|line| {
                line.split_whitespace()
                    .last()
                    .and_then(|count| count.parse::<u64>().ok())
                    .unwrap_or_else(|| panic!("a count at the end of: {line}"))
            })
            .sum()
    }

    fn facts_of(&self, slug: &str) -> usize {
        self.facts
            .iter()
            .filter(|fact| fact.event_type().as_str() == slug)
            .count()
    }

    /// Purchases: a `money-transferred` caused by an action.
    fn purchases(&self) -> usize {
        self.facts
            .iter()
            .filter(|fact| {
                fact.event_type().as_str() == "money-transferred"
                    && matches!(fact.caused_by(), Causation::Action(_))
            })
            .count()
    }

    /// Wages paid: a `money-transferred` caused by a `wage-due`.
    fn wages_paid(&self) -> usize {
        self.facts
            .iter()
            .filter(|fact| {
                fact.event_type().as_str() == "money-transferred"
                    && matches!(fact.caused_by(), Causation::Event(cause)
                        if self.facts.iter().any(|due| due.id() == *cause
                            && due.event_type().as_str() == "wage-due"))
            })
            .count()
    }

    /// Gives: an `items-transferred` from a person, caused by an action.
    fn gives(&self, town: &Town) -> usize {
        self.facts
            .iter()
            .filter(|fact| matches!(fact.caused_by(), Causation::Action(_)))
            .filter_map(market_fact)
            .filter(|fact| {
                matches!(fact, MarketFact::ItemsTransferred { from, .. } if town.people.contains(from))
            })
            .count()
    }

    fn meals(&self) -> u64 {
        self.requested("eat") + self.requested("drink")
    }

    fn report(&self, town: &Town, case: &str) {
        eprintln!(
            "without {case}: purchases {}, wages paid {}, wage-due {}, wage-unpaid {}, \
             items-produced {}, items-consumed {}, gives {}, eat+drink requested {}, give requested \
             {}, buy requested {}, hired {}",
            self.purchases(),
            self.wages_paid(),
            self.facts_of("wage-due"),
            self.facts_of("wage-unpaid"),
            self.facts_of("items-produced"),
            self.facts_of("items-consumed"),
            self.gives(town),
            self.meals(),
            self.requested("give"),
            self.requested("buy"),
            self.facts_of("hired"),
        );
    }
}

/// A count that must be zero, and one that must not.
fn none(case: &str, what: &str, count: usize) {
    assert_eq!(count, 0, "without {case}: {what} occurred {count} time(s)");
}

fn some(case: &str, what: &str, count: usize) {
    assert!(
        count > 0,
        "without {case}: no {what} — the rest of the market stopped"
    );
}

fn as_count(count: u64) -> usize {
    usize::try_from(count).expect("fits")
}

#[test]
fn each_market_pack_removed_runs_or_is_refused_as_its_dependencies_say() {
    let town = Town::read(Path::new(MARKET_TOWN));
    let copies: Vec<(&str, Scratch)> = PACKS
        .iter()
        .map(|(pack, section, carriers)| (*pack, without(pack, *section, *carriers)))
        .collect();
    let copy = |pack: &str| -> &Path {
        copies
            .iter()
            .find(|(case, _)| *case == pack)
            .map(|(_, path)| path.path())
            .expect("a copy")
    };

    std::thread::scope(|scope| {
        scope.spawn(|| refused(copy("item"), "item", &["inventory", "item"]));
        scope.spawn(|| refused(copy("inventory"), "inventory", &["inventory"]));
        scope.spawn(|| {
            let case = "item-transfer";
            let ran = Ran::of(&town, copy(case), case);
            ran.report(&town, case);
            none(case, "a give requested", as_count(ran.requested("give")));
            none(case, "a person's item given by an action", ran.gives(&town));
            some(case, "purchase", ran.purchases());
            some(case, "meal", as_count(ran.meals()));
            some(case, "wage paid", ran.wages_paid());
            some(case, "items-produced", ran.facts_of("items-produced"));
        });
        scope.spawn(|| {
            let case = "economy";
            let ran = Ran::of(&town, copy(case), case);
            ran.report(&town, case);
            none(case, "a buy requested", as_count(ran.requested("buy")));
            for slug in ["funded", "shop-opened", "money-transferred", "wage-unpaid"] {
                none(case, slug, ran.facts_of(slug));
            }
            some(case, "wage-due", ran.facts_of("wage-due"));
            some(case, "items-produced", ran.facts_of("items-produced"));
            some(case, "give", ran.gives(&town));
            some(case, "meal", as_count(ran.meals()));
        });
        scope.spawn(|| {
            let case = "employment";
            let ran = Ran::of(&town, copy(case), case);
            ran.report(&town, case);
            for slug in [
                "hired",
                "shift-started",
                "shift-ended",
                "wage-due",
                "items-produced",
            ] {
                none(case, slug, ran.facts_of(slug));
            }
            some(case, "purchase", ran.purchases());
            some(case, "give", ran.gives(&town));
            some(case, "meal", as_count(ran.meals()));
        });
        scope.spawn(|| {
            let case = "consumption";
            let ran = Ran::of(&town, copy(case), case);
            ran.report(&town, case);
            none(case, "an eat or drink requested", as_count(ran.meals()));
            none(case, "items-consumed", ran.facts_of("items-consumed"));
            some(case, "purchase", ran.purchases());
            some(case, "wage paid", ran.wages_paid());
            some(case, "items-produced", ran.facts_of("items-produced"));
            some(case, "give", ran.gives(&town));
        });
    });
}
