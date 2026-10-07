//! `AC-2` for the two new packs: a world without one of them still runs, and every other system is
//! unchanged (`step-09-social.md` C7, `DECISIONS.md` `ARC-28`).
//!
//! The route a World Pack has to leave a system out is its `systems:` list — there is no pack-level
//! disable flag (`worldpack/src/read.rs`) — so each configuration is a test-time copy of
//! `worlds/social-cafe` whose `world.yaml` omits one system, run by the real binary.
//!
//! ```text
//! without relationships    every fact of every other system equals the full run's, in order, by
//!                          (type, instant, subjects, participants, place, payload bytes); the
//!                          requests, activity and consults printed are equal. Relationships only
//!                          ever listened, so taking it away takes nothing else away.
//! without group-activity   relationships stays installed and enabled; the world runs, every seat
//!                          moves and talks; nobody invites or joins (the controller reads the
//!                          affordances it is offered); people still become acquainted, by talking
//! without naming           every other fact is the same row for row; only a reply's words may
//!                          differ, and the bare world's replies name nobody (10c C4)
//! ```
//!
//! The comparison is shown able to see a difference (`ARC-23`): the same projection of seed 7 and
//! seed 8 differs, at a located row.

mod headless;
mod social;

use std::path::{Path, PathBuf};

use headless::{
    PACK, Tables, every_seat_active_in_every_bucket, fresh, lines, mineworld, run, seats, stderr,
    stdout,
};
use mineworld_contracts::{Causation, EntityId, EventEnvelope};

const DAYS: u64 = 30;

/// A copy of the pack, named as the pack (a pack's id is its directory), with `system` left out.
fn without(system: &str) -> PathBuf {
    without_owning(system, None)
}

/// [`without`], and with the section `system` owns removed from every person's file: a world that
/// does not enable a section's owner refuses the section (`MODULE_SPEC.md` §4.1 rule 6).
fn without_owning(system: &str, section: Option<&str>) -> PathBuf {
    let root = fresh(&format!("composition-without-{system}"));
    let copy = root.join("social-cafe");
    copy_dir(Path::new(PACK), &copy);
    let manifest = copy.join("world.yaml");
    let text = std::fs::read_to_string(&manifest).expect("world.yaml reads");
    let line = format!("  - {system}\n");
    assert!(text.contains(&line), "the pack enables {system}");
    std::fs::write(&manifest, text.replace(&line, "")).expect("world.yaml writes");
    if let Some(section) = section {
        let mut removed = 0;
        for entry in std::fs::read_dir(copy.join("people")).expect("people/ lists") {
            let path = entry.expect("an entry").path();
            let text = std::fs::read_to_string(&path).expect("reads");
            let (kept, found) = strip_section(&text, section);
            removed += usize::from(found);
            std::fs::write(&path, kept).expect("writes");
        }
        assert_eq!(removed, 12, "every person carried a `{section}:` section");
    }
    copy
}

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

/// `run` of a given pack directory, saved; its stdout.
fn run_pack(pack: &Path, seed: u64, save: &Path) -> String {
    let output = mineworld(&[
        "run",
        pack.to_str().expect("path"),
        "--headless",
        "--seed",
        &seed.to_string(),
        "--days",
        &DAYS.to_string(),
        "--save",
        save.to_str().expect("path"),
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
}

/// What a fact says, without the identities that depend on how many facts came before it.
type Said = (
    String,
    i64,
    Vec<EntityId>,
    Vec<EntityId>,
    Option<EntityId>,
    Vec<u8>,
);

fn said(fact: &EventEnvelope) -> Said {
    (
        fact.event_type().as_str().to_owned(),
        fact.at().seconds(),
        fact.subjects().to_vec(),
        fact.participants().to_vec(),
        fact.place().map(|place| place.entity_id()),
        fact.payload().payload().clone(),
    )
}

fn relationships_fact(fact: &EventEnvelope) -> bool {
    matches!(
        fact.event_type().as_str(),
        "became-acquainted" | "relationship-changed"
    )
}

/// The first row at which two projections differ, or the shorter length if one is a prefix.
fn first_difference(a: &[Said], b: &[Said]) -> Option<usize> {
    a.iter()
        .zip(b)
        .position(|(x, y)| x != y)
        .or_else(|| (a.len() != b.len()).then(|| a.len().min(b.len())))
}

#[test]
fn without_relationships_every_other_system_is_unchanged() {
    let full_save = fresh("composition-full");
    let full = run(7, DAYS, Some(&full_save));
    let full_facts = social::facts_of(&Tables::read(&full_save));
    social::precondition(&full_facts, 30, true);

    let pack = without("relationships");
    let save = fresh("composition-no-relationships-save");
    let printed = run_pack(&pack, 7, &save);
    assert_eq!(
        lines(&printed, "faults     0").len(),
        1,
        "no fault: {printed}"
    );
    let facts = social::facts_of(&Tables::read(&save));
    assert!(
        !facts.iter().any(relationships_fact),
        "no relationship fact in a world without relationships"
    );

    let others: Vec<Said> = full_facts
        .iter()
        .filter(|fact| !relationships_fact(fact))
        .map(said)
        .collect();
    let without_them: Vec<Said> = facts.iter().map(said).collect();
    eprintln!(
        "full run: {} facts, {} of them not relationships'; without relationships: {} facts",
        full_facts.len(),
        others.len(),
        without_them.len()
    );
    assert!(
        others.len() > 10_000,
        "a busy month was compared: {}",
        others.len()
    );
    assert_eq!(
        first_difference(&others, &without_them),
        None,
        "every other system's facts are the same, in the same order"
    );
    for prefix in ["requests ", "activity ", "consults "] {
        assert_eq!(
            lines(&full, prefix),
            lines(&printed, prefix),
            "{prefix}lines"
        );
    }

    // The comparison can see a difference: seed 8's world differs from seed 7's, at a located row.
    let eight_save = fresh("composition-full-seed-8");
    run(8, DAYS, Some(&eight_save));
    let eight: Vec<Said> = social::facts_of(&Tables::read(&eight_save))
        .iter()
        .filter(|fact| !relationships_fact(fact))
        .map(said)
        .collect();
    let differs_at = first_difference(&others, &eight).expect("seed 8 differs from seed 7");
    eprintln!(
        "seed 7 and seed 8 first differ at row {differs_at}: {} vs {}",
        others[differs_at].0, eight[differs_at].0
    );
}

/// `AC-2` for `naming` (10c C4): without it, every other system's facts are the same — except that
/// a reply which named somebody now says "someone else", because the controller is told no name. The
/// controller is not a system; what it says is the one thing that may move, and only by that one
/// substitution.
#[test]
fn without_naming_every_other_system_is_unchanged_but_for_the_names_people_say() {
    let full_save = fresh("composition-full-for-naming");
    let full = run(7, DAYS, Some(&full_save));
    let full_facts = social::facts_of(&Tables::read(&full_save));

    let pack = without_owning("naming", Some("name"));
    let save = fresh("composition-no-naming-save");
    let printed = run_pack(&pack, 7, &save);
    assert_eq!(
        lines(&printed, "faults     0").len(),
        1,
        "no fault: {printed}"
    );
    let bare = social::facts_of(&Tables::read(&save));
    assert!(
        !bare
            .iter()
            .any(|fact| fact.event_type().as_str() == "named"),
        "nobody is named in a world without naming"
    );

    let others: Vec<&EventEnvelope> = full_facts
        .iter()
        .filter(|fact| fact.event_type().as_str() != "named")
        .collect();
    assert_eq!(
        others.len(),
        bare.len(),
        "the same number of every other fact"
    );
    assert!(others.len() > 10_000, "a busy month: {}", others.len());
    let mut substituted = 0;
    for (row, (full, bare)) in others.iter().zip(&bare).enumerate() {
        let (mut a, mut b) = (said(full), said(bare));
        if a.0 == "spoke" {
            // Not "equal but for one substituted name", as first planned: a reply quotes earlier
            // replies, each cut at 80 characters, so a name inside a quotation may itself be cut
            // ("Ali…" against "som…") and a substituted name moves where later cuts fall. The text
            // cannot be mapped one onto the other. What holds, and is asserted: who spoke to whom,
            // when, is the same on every row (below); words that are not a reply are byte-equal; and
            // a reply differs only between two replies, the bare one naming nobody (step-09 §9
            // E-C4, the oracle as corrected).
            let full_words = spoken(full);
            let bare_words = spoken(bare);
            if full_words != bare_words {
                assert!(
                    full_words.starts_with(REPLY) && bare_words.starts_with(REPLY),
                    "row {row}: only a reply's words may differ: {full_words:?} vs {bare_words:?}"
                );
                assert!(
                    !says_a_given_name(&bare_words),
                    "row {row}: a name in a world that names nobody: {bare_words:?}"
                );
                substituted += 1;
            }
            // Compared above, by the words; everything else about the fact is compared below.
            a.5.clear();
            b.5.clear();
        }
        assert_eq!(a, b, "row {row}: every other system's fact is the same");
    }
    assert!(
        substituted > 0,
        "located: some reply named somebody in the full world"
    );
    let bare_names = bare
        .iter()
        .filter(|fact| fact.event_type().as_str() == "spoke")
        .filter(|fact| says_a_given_name(&spoken(fact)))
        .count();
    assert_eq!(bare_names, 0, "and nobody says a name nobody was told");
    let full_names = others
        .iter()
        .filter(|fact| fact.event_type().as_str() == "spoke")
        .filter(|fact| {
            let words = spoken(fact);
            social::AUTHORED_NAMES
                .iter()
                .any(|(_, name)| words.contains(&format!("Earlier, {name} said")))
        })
        .count();
    assert!(
        full_names > 0,
        "located: in the full world a reply names the earlier speaker in full"
    );
    for prefix in ["requests ", "activity ", "consults "] {
        assert_eq!(
            lines(&full, prefix),
            lines(&printed, prefix),
            "{prefix}lines"
        );
    }
    eprintln!(
        "without naming: {} facts compared; {substituted} replies differ, the bare ones naming \
         nobody; {full_names} full-world replies say a full name",
        others.len()
    );
}

/// How every reply the rule controllers make begins.
const REPLY: &str = "I remember you. You said ";

/// Whether `words` contain any person's given name, as a word of its own — the part of a name that
/// survives a quotation cut short.
fn says_a_given_name(words: &str) -> bool {
    social::AUTHORED_NAMES.iter().any(|(_, name)| {
        let given = name.split(' ').next().unwrap_or(name);
        words
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| word == given)
    })
}

/// The words of a `spoke`, decoded with conversation's own type.
fn spoken(fact: &EventEnvelope) -> String {
    social::decoded::<mineworld_conversation::Spoke>(fact)
        .expect("a spoke")
        .utterance()
        .as_str()
        .to_owned()
}

#[test]
fn without_group_activity_relationships_stays_and_the_world_runs() {
    let pack = without("group-activity");
    let save = fresh("composition-no-group-activity-save");
    let printed = run_pack(&pack, 7, &save);
    assert_eq!(
        lines(&printed, "faults     0").len(),
        1,
        "no fault: {printed}"
    );
    let seats = seats();
    let seats: Vec<&str> = seats.iter().map(String::as_str).collect();
    assert_eq!(every_seat_active_in_every_bucket(&printed, &seats), 1);
    for line in lines(&printed, "requests ") {
        for action in [
            "invite",
            "accept-invitation",
            "decline-invitation",
            "join-group-activity",
        ] {
            assert!(
                !line.starts_with(&format!("requests   {action} ")),
                "nobody is asked to do something together in a world without it: {line}"
            );
        }
    }

    let inspected = mineworld(&["inspect", save.to_str().expect("path"), "--last", "0"]);
    assert!(inspected.status.success(), "{}", stderr(&inspected));
    let report = stdout(&inspected);
    let systems = lines(&report, "systems ");
    assert_eq!(
        systems,
        [
            "systems    presence v2, movement v1, conversation v1, relationships v1, naming v1, \
             schedule v1"
        ],
        "relationships is installed and enabled, group-activity is not there"
    );

    let facts = social::facts_of(&Tables::read(&save));
    let types: std::collections::BTreeMap<u64, &str> = facts
        .iter()
        .map(|fact| (fact.id().raw(), fact.event_type().as_str()))
        .collect();
    let acquainted = facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "became-acquainted")
        .count();
    assert!(
        acquainted > 0,
        "people still come to know each other, by talking"
    );
    for fact in facts.iter().filter(|fact| relationships_fact(fact)) {
        let Causation::Event(cause) = fact.caused_by() else {
            panic!("a relationship fact not caused by a fact");
        };
        assert_eq!(
            types.get(&cause.raw()).copied(),
            Some("spoke"),
            "with no activities, speech is all relationships hears"
        );
    }
    assert!(
        !facts
            .iter()
            .any(|fact| fact.event_type().as_str().contains("group-activity")
                || fact.event_type().as_str().starts_with("invit")),
        "no group-activity fact"
    );
    eprintln!("without group-activity: {acquainted} became-acquainted, all caused by spoke");
}
