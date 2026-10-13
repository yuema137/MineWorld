//! IB-2, IB-3 and IB-8's binary half — the real sections through the real binary
//! (step-18-interaction-list §12.5; `docs/DECISIONS.md` `ARC-61`, `ARC-63`).
//!
//! Scratch copies of `worlds/social-cafe`, made at test time, run 30 days with seed 7 and `--save`:
//!
//! ```text
//! IB-2  each section stated explicitly as `extends: default` adds exactly its two genesis facts;
//!       every later fact is the same, its id and every event id it refers to offset by exactly 2,
//!       and every request outcome is the same                       M-IB2 (the base from the
//!                                                                   partial's lowest values) fails it
//! IB-3  (a) gap 3 600: fewer conversations start than unconfigured, every seat still talks in every
//!           10-day bucket, no fault
//!       (b) invitation lifetime 60: no acceptance is refused (the controller never answers an expired
//!           invitation), fewer invitations are accepted, no fault    M-IB3 (the controller judges
//!                                                                   by 1 800 s again) fails (b)
//! IB-8  a save, then its section edited, its referenced class edited, an unreferenced class edited,
//!       its licence policy edited: the first two are refused at resume and at replay as drift, by
//!       name; the last two are not drift                             M-IB8 (the resolved fact
//!                                                                   omits its classes) fails it
//! ```

mod headless;
mod social;

use std::collections::BTreeMap;
use std::path::Path;

use headless::{PACK, Scratch, Tables, fresh, lines, mineworld, stderr, stdout};
use mineworld_contracts::{Causation, EventEnvelope};

const DAYS: &str = "30";
const BUCKET: i64 = 10 * 86_400;

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

/// A copy of social-cafe named as the pack, with `configure:` listing `keys` and these files written.
fn configured(name: &str, keys: &[&str], files: &[(&str, &str)]) -> Scratch {
    let copy = fresh(name).within("social-cafe");
    copy_dir(Path::new(PACK), &copy);
    if !keys.is_empty() {
        let manifest = copy.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("reads");
        let listed: String = keys.iter().map(|key| format!("  - {key}\n")).collect();
        std::fs::write(&manifest, format!("{text}\nconfigure:\n{listed}")).expect("writes");
    }
    std::fs::create_dir_all(copy.join("configure")).expect("writable");
    for (file, text) in files {
        std::fs::write(copy.join(file), text).expect("writes");
    }
    copy
}

/// `run` of `world` for 30 days, seed 7, saved in `save`: what it printed.
fn run(world: &Path, save: &Path) -> String {
    let output = mineworld(&[
        "run",
        world.to_str().expect("a path"),
        "--headless",
        "--seed",
        "7",
        "--days",
        DAYS,
        "--save",
        save.to_str().expect("a path"),
    ]);
    assert!(output.status.success(), "run failed: {}", stderr(&output));
    stdout(&output)
}

fn facts(save: &Path) -> Vec<EventEnvelope> {
    social::facts_of(&Tables::read(save))
}

fn count(facts: &[EventEnvelope], kind: &str) -> usize {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == kind)
        .count()
}

fn faults(printed: &str) -> u64 {
    lines(printed, "faults")
        .first()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse().ok())
        .expect("a faults line")
}

// ---- IB-2 ------------------------------------------------------------------------------------

#[test]
fn an_explicit_default_adds_only_its_genesis_facts() {
    let plain = configured("ib2-plain", &[], &[]);
    let explicit = configured(
        "ib2-explicit",
        &["conversation", "group-activity"],
        &[
            ("configure/conversation.yaml", "extends: default\n"),
            ("configure/group-activity.yaml", "extends: default\n"),
        ],
    );
    let (a, b) = (fresh("ib2-plain-save"), fresh("ib2-explicit-save"));
    let (printed_a, printed_b) = (run(&plain, &a), run(&explicit, &b));
    assert_eq!(
        lines(&printed_a, "requests"),
        lines(&printed_b, "requests"),
        "every request outcome is the same"
    );
    let (a, b) = (facts(&a), facts(&b));
    let configured: Vec<&EventEnvelope> = b
        .iter()
        .filter(|fact| {
            fact.event_type()
                .as_str()
                .ends_with("-interactions-configured")
        })
        .collect();
    assert_eq!(configured.len(), 2, "exactly the two configuration facts");
    assert!(
        configured
            .iter()
            .all(|fact| *fact.caused_by() == Causation::WorldGenesis)
    );
    let rest: Vec<&EventEnvelope> = b
        .iter()
        .filter(|fact| {
            !fact
                .event_type()
                .as_str()
                .ends_with("-interactions-configured")
        })
        .collect();
    assert_eq!(rest.len(), a.len(), "every other fact, once");
    let offset = |id: u64| id + 2;
    for (index, (x, y)) in a.iter().zip(&rest).enumerate() {
        // Genesis facts stated before the configuration keep their ids; everything after moves by 2.
        let moved = if x.id().raw() == y.id().raw() { 0 } else { 2 };
        assert!(
            y.id().raw() == x.id().raw() + moved,
            "fact {index}: id {} vs {}",
            x.id().raw(),
            y.id().raw()
        );
        assert_eq!(x.event_type(), y.event_type(), "fact {index}");
        assert_eq!(x.at(), y.at(), "fact {index}");
        assert_eq!(x.payload(), y.payload(), "fact {index}: {}", x.event_type());
        assert_eq!(x.subjects(), y.subjects(), "fact {index}");
        assert_eq!(x.visibility(), y.visibility(), "fact {index}");
        match (x.caused_by(), y.caused_by()) {
            (Causation::Event(cx), Causation::Event(cy)) => {
                assert_eq!(
                    offset(cx.raw()),
                    cy.raw(),
                    "fact {index}: the event it refers to"
                );
            }
            (cx, cy) => assert_eq!(cx, cy, "fact {index}"),
        }
    }
    let moved = rest
        .iter()
        .zip(&a)
        .filter(|(y, x)| y.id().raw() == x.id().raw() + 2)
        .count();
    println!(
        "IB-2: {} facts unconfigured, {} explicit ({} configuration facts); {moved} moved by 2",
        a.len(),
        b.len(),
        configured.len()
    );
    assert!(moved > 10_000, "the run's facts moved by exactly two");
}

// ---- IB-3 ------------------------------------------------------------------------------------

/// Per 10-day bucket, per speaker key, how many `spoke` facts.
fn spoken_per_bucket(facts: &[EventEnvelope]) -> BTreeMap<i64, BTreeMap<u64, u64>> {
    let mut counted: BTreeMap<i64, BTreeMap<u64, u64>> = BTreeMap::new();
    for fact in facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == "spoke")
    {
        let speaker = fact.participants().first().expect("a speaker").raw();
        *counted
            .entry(fact.at().seconds().min(30 * 86_400 - 1) / BUCKET)
            .or_default()
            .entry(speaker)
            .or_default() += 1;
    }
    counted
}

#[test]
fn a_longer_gap_and_a_shorter_lifetime_change_the_world_as_configured() {
    let plain = configured("ib3-plain", &[], &[]);
    let plain_save = fresh("ib3-plain-save");
    let plain_printed = run(&plain, &plain_save);
    let plain_facts = facts(&plain_save);

    // (a) A gap of an hour: conversations continue across longer silences.
    let gap = configured(
        "ib3-gap",
        &["conversation"],
        &[(
            "configure/conversation.yaml",
            "parameters:\n  - { gap: 3600 }\n",
        )],
    );
    let gap_save = fresh("ib3-gap-save");
    let gap_printed = run(&gap, &gap_save);
    let gap_facts = facts(&gap_save);
    assert_eq!(faults(&gap_printed), 0);
    assert!(
        count(&gap_facts, "conversation-started") < count(&plain_facts, "conversation-started"),
        "{} vs {}",
        count(&gap_facts, "conversation-started"),
        count(&plain_facts, "conversation-started")
    );
    let speakers = spoken_per_bucket(&plain_facts)[&0].len();
    let buckets = spoken_per_bucket(&gap_facts);
    assert_eq!(buckets.len(), 3, "three 10-day buckets");
    for (bucket, speakers_in) in &buckets {
        assert_eq!(
            speakers_in.len(),
            speakers,
            "bucket {bucket}: every speaker of the unconfigured run still talks"
        );
    }

    // (b) Invitations that lapse after a minute.
    let lifetime = configured(
        "ib3-lifetime",
        &["group-activity"],
        &[(
            "configure/group-activity.yaml",
            "parameters:\n  - { invitation_lifetime: 60 }\n",
        )],
    );
    let lifetime_save = fresh("ib3-lifetime-save");
    let lifetime_printed = run(&lifetime, &lifetime_save);
    assert_eq!(faults(&lifetime_printed), 0);
    assert!(
        !lines(&lifetime_printed, "requests   accept-invitation rejected")
            .iter()
            .any(|_| true),
        "the controller never answers an expired invitation: {:?}",
        lines(&lifetime_printed, "requests   accept-invitation")
    );
    let lifetime_facts = facts(&lifetime_save);
    println!(
        "IB-3: conversation-started {} unconfigured, {} with gap 3600; invitation-accepted {} \
         unconfigured, {} with lifetime 60; {:?}",
        count(&plain_facts, "conversation-started"),
        count(&gap_facts, "conversation-started"),
        count(&plain_facts, "invitation-accepted"),
        count(&lifetime_facts, "invitation-accepted"),
        lines(&lifetime_printed, "requests   accept-invitation"),
    );
    assert!(
        count(&lifetime_facts, "invitation-accepted") < count(&plain_facts, "invitation-accepted")
    );
    assert_eq!(faults(&plain_printed), 0);
}

// ---- IB-8 ------------------------------------------------------------------------------------

fn resume_and_replay(world: &Path, save: &Path) -> (std::process::Output, std::process::Output) {
    let (world, save) = (
        world.to_str().expect("a path"),
        save.to_str().expect("a path"),
    );
    let resumed = mineworld(&[
        "run",
        world,
        "--headless",
        "--seed",
        "7",
        "--days",
        "2",
        "--save",
        save,
    ]);
    let replayed = mineworld(&["replay", world, "--save", save]);
    (resumed, replayed)
}

#[test]
fn an_edited_section_or_referenced_class_is_drift_and_nothing_else_is() {
    let classes = "- { class: regular, of: person, tag: regular }\n\
                   - { class: runner, of: person, tag: runner }\n";
    let world = configured(
        "ib8",
        &["classes", "packages", "conversation"],
        &[
            ("configure/classes.yaml", classes),
            ("configure/packages.yaml", "allowed: [MIT]\n"),
            (
                "configure/conversation.yaml",
                "parameters:\n  - { actor: regular, gap: 3600 }\n",
            ),
        ],
    );
    let save = fresh("ib8-save");
    let output = mineworld(&[
        "run",
        world.to_str().expect("a path"),
        "--headless",
        "--seed",
        "7",
        "--days",
        "1",
        "--save",
        save.to_str().expect("a path"),
    ]);
    assert!(output.status.success(), "{}", stderr(&output));

    let refused = |what: &str| {
        let (resumed, replayed) = resume_and_replay(&world, &save);
        for (command, output) in [("run", resumed), ("replay", replayed)] {
            let complaint = stderr(&output);
            assert!(!output.status.success(), "{what}: {command} is refused");
            assert!(
                complaint.contains("configuration differs from the save's")
                    && complaint.contains("'conversation'"),
                "{what}: {command} names the drift: {complaint}"
            );
        }
    };
    let accepted = |what: &str| {
        let (resumed, replayed) = resume_and_replay(&world, &save);
        assert!(
            resumed.status.success(),
            "{what}: run resumes: {}",
            stderr(&resumed)
        );
        assert!(
            replayed.status.success(),
            "{what}: replay: {}",
            stderr(&replayed)
        );
    };

    // Not drift: a class no section can see, and the licence policy.
    std::fs::write(
        world.join("configure/classes.yaml"),
        classes.replace("tag: runner", "tag: resident"),
    )
    .expect("writes");
    std::fs::write(
        world.join("configure/packages.yaml"),
        "allowed: [MIT, ISC]\n",
    )
    .expect("writes");
    accepted("an unreferenced class and the policy edited");

    // Drift: the referenced class.
    std::fs::write(
        world.join("configure/classes.yaml"),
        classes.replace("tag: regular", "tag: resident"),
    )
    .expect("writes");
    refused("the referenced class edited");
    std::fs::write(world.join("configure/classes.yaml"), classes).expect("writes");

    // Drift: the section.
    std::fs::write(
        world.join("configure/conversation.yaml"),
        "parameters:\n  - { actor: regular, gap: 1800 }\n",
    )
    .expect("writes");
    refused("the section edited");
}
