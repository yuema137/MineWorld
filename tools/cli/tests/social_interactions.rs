//! IE-2, IE-4 … IE-7 and IE-10 — the social packs' sections through the real binary
//! (`pr-il-e-social.md` §6; `docs/DECISIONS.md` `ARC-63`, `ARC-65`).
//!
//! Scratch copies of `worlds/social-cafe`, made at test time, run 30 days with seed 7 and `--save`.
//! Classes `resident` (people tagged `resident`: carol, dev, erin, otto) and `commuter` (tagged
//! `commuter`: grace, hana). Fixed before measuring:
//!
//! ```text
//! IE-2  the three sections stated as `extends: default` add exactly their three genesis facts; every
//!       later fact equal, its id and every id it refers to offset by 3; request outcomes equal
//!                                                   M-IE2 (relationships' default with declined_regard 0)
//! IE-4  talk forbidden resident ↔ commuter: (a) no such spoke; (b) no talk refused PermissionDenied
//!       (the controller follows offers); (c) every seat talks in every 10-day bucket, no fault;
//!       (d) the unconfigured twin has such a spoke, else INCONCLUSIVE
//!                                                   M-IE1 (talk's offer skips permits) fails (b)
//! IE-5  acquaint forbidden resident → commuter: no fact of that direction; the reverse occurs; the
//!       twin has one, else INCONCLUSIVE             M-IE5 (relationships ignores the rule)
//! IE-6  spoke biography on, off for a resident speaker; became-acquainted off for a commuter holder:
//!       grace's biography has lines but none by a resident and no acquaintance she holds; carol's
//!       still has hers; the log has every excluded fact   M-IE6 (spoke's actor read as the listener)
//! IE-7  spoke narrowed to participants: no seat perceives a line it is not party to; the twin's
//!       seats do, else INCONCLUSIVE; every spoke is stated Participants
//!                                                   M-IE7 (spoke stated with the owner default)
//! IE-10 `mineworld interactions`: relationships "default (compiled)" unconfigured; the IE-4 copy's
//!       talk rule and each person's class
//! ```

mod headless;
mod social;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use headless::{PACK, Scratch, Tables, fresh, lines, mineworld, seats, stderr, stdout};
use mineworld_contracts::{
    Causation, EntityId, EventEnvelope, PerceivedEvent, Visibility, WorldTime,
};
use serde_json::Value;

const DAYS: &str = "30";
const BUCKET: i64 = 10 * 86_400;
const CLASSES: &str = "- { class: resident, of: person, tag: resident }\n\
                       - { class: commuter, of: person, tag: commuter }\n";
const RESIDENTS: [&str; 4] = ["carol", "dev", "erin", "otto"];
const COMMUTERS: [&str; 2] = ["grace", "hana"];

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

/// A copy of social-cafe with `configure:` listing `keys` and these `configure/<key>.yaml` files.
fn configured(name: &str, sections: &[(&str, &str)]) -> Scratch {
    let copy = fresh(name).within("social-cafe");
    copy_dir(Path::new(PACK), &copy);
    if !sections.is_empty() {
        let manifest = copy.join("world.yaml");
        let text = std::fs::read_to_string(&manifest).expect("reads");
        let listed: String = sections
            .iter()
            .map(|(key, _)| format!("  - {key}\n"))
            .collect();
        std::fs::write(&manifest, format!("{text}\nconfigure:\n{listed}")).expect("writes");
        std::fs::create_dir_all(copy.join("configure")).expect("writable");
        for (key, text) in sections {
            std::fs::write(copy.join(format!("configure/{key}.yaml")), text).expect("writes");
        }
    }
    copy
}

/// `sections` with the classes listed first.
fn classed<'a>(sections: &[(&'a str, &'a str)]) -> Vec<(&'a str, &'a str)> {
    let mut all = vec![("classes", CLASSES)];
    all.extend_from_slice(sections);
    all
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

/// A world run: its copy, its save, what it printed and its facts.
struct Ran {
    world: Scratch,
    save: Scratch,
    printed: String,
    facts: Vec<EventEnvelope>,
}

fn ran(name: &str, sections: &[(&str, &str)]) -> Ran {
    let world = configured(name, sections);
    let save = fresh(&format!("{name}-save"));
    let printed = run(&world, &save);
    let facts = social::facts_of(&Tables::read(&save));
    Ran {
        world,
        save,
        printed,
        facts,
    }
}

/// The world's person ids by key (genesis allocates them alike in every copy).
fn people() -> BTreeMap<String, EntityId> {
    let loaded = mineworld_worldpack::WorldPack::read(PACK)
        .expect("reads")
        .load(WorldTime::EPOCH)
        .expect("loads");
    loaded
        .ids()
        .iter()
        .map(|(key, id)| (key.as_str().to_owned(), *id))
        .collect()
}

fn of(people: &BTreeMap<String, EntityId>, keys: &[&str]) -> BTreeSet<EntityId> {
    keys.iter().map(|key| people[*key]).collect()
}

fn of_type<'a>(facts: &'a [EventEnvelope], kind: &str) -> Vec<&'a EventEnvelope> {
    facts
        .iter()
        .filter(|fact| fact.event_type().as_str() == kind)
        .collect()
}

fn faults(printed: &str) -> u64 {
    lines(printed, "faults")
        .first()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|value| value.parse().ok())
        .expect("a faults line")
}

/// Whether a spoke is between one resident and one commuter, either way.
fn across(
    fact: &EventEnvelope,
    residents: &BTreeSet<EntityId>,
    commuters: &BTreeSet<EntityId>,
) -> bool {
    let (speaker, listener) = (fact.participants()[0], fact.participants()[1]);
    (residents.contains(&speaker) && commuters.contains(&listener))
        || (commuters.contains(&speaker) && residents.contains(&listener))
}

/// A relationships fact held by `holders` about `counterparts`.
fn held(
    fact: &EventEnvelope,
    holders: &BTreeSet<EntityId>,
    counterparts: &BTreeSet<EntityId>,
) -> bool {
    holders.contains(&fact.subjects()[0]) && counterparts.contains(&fact.participants()[1])
}

fn relationship_facts(facts: &[EventEnvelope]) -> Vec<&EventEnvelope> {
    facts
        .iter()
        .filter(|fact| {
            matches!(
                fact.event_type().as_str(),
                "became-acquainted" | "relationship-changed"
            )
        })
        .collect()
}

// ---- IE-2 ---------------------------------------------------------------------------------------

#[test]
fn an_explicit_default_for_the_three_social_sections_adds_only_their_genesis_facts() {
    let plain = ran("ie2-plain", &[]);
    let explicit = ran(
        "ie2-explicit",
        &[
            ("conversation", "extends: default\n"),
            ("group-activity", "extends: default\n"),
            ("relationships", "extends: default\n"),
        ],
    );
    assert_eq!(
        lines(&plain.printed, "requests"),
        lines(&explicit.printed, "requests"),
        "every request outcome is the same"
    );
    let configured: Vec<&EventEnvelope> = explicit
        .facts
        .iter()
        .filter(|fact| {
            fact.event_type()
                .as_str()
                .ends_with("-interactions-configured")
        })
        .collect();
    assert_eq!(configured.len(), 3, "exactly the three configuration facts");
    assert!(
        configured
            .iter()
            .all(|fact| *fact.caused_by() == Causation::WorldGenesis)
    );
    let rest: Vec<&EventEnvelope> = explicit
        .facts
        .iter()
        .filter(|fact| {
            !fact
                .event_type()
                .as_str()
                .ends_with("-interactions-configured")
        })
        .collect();
    assert_eq!(rest.len(), plain.facts.len(), "every other fact, once");
    for (index, (x, y)) in plain.facts.iter().zip(&rest).enumerate() {
        // Genesis facts stated before the configuration keep their ids; everything after moves by 3.
        let moved = if x.id().raw() == y.id().raw() { 0 } else { 3 };
        assert_eq!(y.id().raw(), x.id().raw() + moved, "fact {index}");
        assert_eq!(x.event_type(), y.event_type(), "fact {index}");
        assert_eq!(x.at(), y.at(), "fact {index}");
        assert_eq!(x.payload(), y.payload(), "fact {index}: {}", x.event_type());
        assert_eq!(x.subjects(), y.subjects(), "fact {index}");
        assert_eq!(x.participants(), y.participants(), "fact {index}");
        assert_eq!(x.visibility(), y.visibility(), "fact {index}");
        match (x.caused_by(), y.caused_by()) {
            (Causation::Event(cx), Causation::Event(cy)) => {
                assert_eq!(
                    cx.raw() + 3,
                    cy.raw(),
                    "fact {index}: the event it refers to"
                );
            }
            (cx, cy) => assert_eq!(cx, cy, "fact {index}"),
        }
    }
    let moved = rest
        .iter()
        .zip(&plain.facts)
        .filter(|(y, x)| y.id().raw() == x.id().raw() + 3)
        .count();
    assert!(
        moved > 10_000,
        "the run's facts moved by exactly three: {moved}"
    );
}

// ---- IE-4 … IE-7 --------------------------------------------------------------------------------

/// One unconfigured twin, and one configured copy per criterion, each against it.
#[test]
fn the_social_sections_change_the_world_as_configured() {
    let people = people();
    let residents = of(&people, &RESIDENTS);
    let commuters = of(&people, &COMMUTERS);
    let twin = ran("ie-twin", &[]);

    forbidden_talk(&twin, &residents, &commuters, &people);
    forbidden_acquaintance(&twin, &residents, &commuters);
    biography(&residents, &commuters, &people);
    perception(&twin, &people);
}

/// IE-4.
fn forbidden_talk(
    twin: &Ran,
    residents: &BTreeSet<EntityId>,
    commuters: &BTreeSet<EntityId>,
    people: &BTreeMap<String, EntityId>,
) {
    let rules = "rules:\n\
                 \x20 - { action: talk, actor: resident, target: commuter, effect: forbid }\n\
                 \x20 - { action: talk, actor: commuter, target: resident, effect: forbid }\n";
    let copy = ran("ie4-talk", &classed(&[("conversation", rules)]));
    let twin_across = of_type(&twin.facts, "spoke")
        .into_iter()
        .filter(|fact| across(fact, residents, commuters))
        .count();
    assert!(
        twin_across > 0,
        "IE-4 INCONCLUSIVE: the unconfigured twin has no resident–commuter line"
    );
    let configured_across = of_type(&copy.facts, "spoke")
        .into_iter()
        .filter(|fact| across(fact, residents, commuters))
        .count();
    assert_eq!(
        configured_across, 0,
        "IE-4 (a): no forbidden line (twin: {twin_across})"
    );
    assert!(
        lines(&copy.printed, "requests   talk rejected PermissionDenied").is_empty(),
        "IE-4 (b): the controller never attempts a forbidden talk: {:?}",
        lines(&copy.printed, "requests   talk ")
    );
    assert_eq!(faults(&copy.printed), 0, "IE-4 (c): no fault");
    println!(
        "IE-4: twin {twin_across} resident–commuter lines; configured 0; {} lines in all",
        of_type(&copy.facts, "spoke").len()
    );
    let mut spoken: BTreeMap<i64, BTreeSet<EntityId>> = BTreeMap::new();
    for fact in of_type(&copy.facts, "spoke") {
        spoken
            .entry(fact.at().seconds().min(30 * 86_400 - 1) / BUCKET)
            .or_default()
            .insert(fact.participants()[0]);
    }
    assert_eq!(spoken.len(), 3, "three 10-day buckets");
    for seat in seats() {
        for (bucket, speakers) in &spoken {
            assert!(
                speakers.contains(&people[&seat]),
                "IE-4 (c): {seat} talks in bucket {bucket}"
            );
        }
    }
}

/// IE-5, through the binary.
fn forbidden_acquaintance(
    twin: &Ran,
    residents: &BTreeSet<EntityId>,
    commuters: &BTreeSet<EntityId>,
) {
    let rules = "rules:\n\
                 \x20 - { action: acquaint, actor: resident, target: commuter, effect: forbid }\n";
    let copy = ran("ie5-acquaint", &classed(&[("relationships", rules)]));
    let forbidden = |facts: &[EventEnvelope]| {
        relationship_facts(facts)
            .into_iter()
            .filter(|fact| held(fact, residents, commuters))
            .count()
    };
    let twin_forbidden = forbidden(&twin.facts);
    assert!(
        twin_forbidden > 0,
        "IE-5 INCONCLUSIVE: the twin forms no resident → commuter relationship"
    );
    assert_eq!(
        forbidden(&copy.facts),
        0,
        "IE-5: none (twin: {twin_forbidden})"
    );
    let reverse = relationship_facts(&copy.facts)
        .into_iter()
        .filter(|fact| held(fact, commuters, residents))
        .count();
    assert!(reverse > 0, "IE-5: the reverse direction forms");
    println!(
        "IE-5: twin {twin_forbidden} resident → commuter facts; configured 0; reverse {reverse}"
    );
    assert_eq!(faults(&copy.printed), 0);
}

/// `mineworld biography` of `person` in `world`'s save, as event ids.
fn biography_ids(world: &Path, save: &Path, person: &str) -> Vec<u64> {
    let output = mineworld(&[
        "biography",
        world.to_str().expect("a path"),
        "--save",
        save.to_str().expect("a path"),
        "--person",
        person,
        "--json",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
        .lines()
        .map(|line| {
            let entry: Value = serde_json::from_str(line).expect("a JSON entry");
            entry["event_id"].as_u64().expect("an event id")
        })
        .collect()
}

/// IE-6 (QIB-11).
fn biography(
    residents: &BTreeSet<EntityId>,
    commuters: &BTreeSet<EntityId>,
    people: &BTreeMap<String, EntityId>,
) {
    let copy = ran(
        "ie6-biography",
        &classed(&[
            (
                "conversation",
                "consequences:\n\
                 \x20 - { fact: spoke, biography: on }\n\
                 \x20 - { fact: spoke, actor: resident, biography: off }\n",
            ),
            (
                "relationships",
                "consequences:\n  - { fact: became-acquainted, actor: commuter, biography: off }\n",
            ),
        ]),
    );
    let by_id: BTreeMap<u64, &EventEnvelope> = copy
        .facts
        .iter()
        .map(|fact| (fact.id().raw(), fact))
        .collect();
    let grace = people["grace"];
    let hers: Vec<&EventEnvelope> = biography_ids(&copy.world, &copy.save, "grace")
        .iter()
        .map(|id| by_id[id])
        .collect();
    let lines = of_type_ref(&hers, "spoke");
    println!(
        "IE-6: grace's biography {} entries, {} lines",
        hers.len(),
        lines.len()
    );
    assert!(
        lines
            .iter()
            .any(|fact| !residents.contains(&fact.participants()[0])),
        "IE-6 (a): lines by non-residents enter her biography"
    );
    assert!(
        lines
            .iter()
            .all(|fact| !residents.contains(&fact.participants()[0])),
        "IE-6 (a): no line said by a resident, to her or by anyone"
    );
    assert!(
        of_type_ref(&hers, "became-acquainted")
            .iter()
            .all(|fact| fact.subjects()[0] != grace),
        "IE-6 (b): no acquaintance she holds"
    );
    let carol = people["carol"];
    let carols: Vec<&EventEnvelope> = biography_ids(&copy.world, &copy.save, "carol")
        .iter()
        .map(|id| by_id[id])
        .collect();
    assert!(
        of_type_ref(&carols, "became-acquainted")
            .iter()
            .any(|fact| fact.subjects()[0] == carol),
        "IE-6 (b): a resident still holds hers"
    );
    // (c) The log still holds every excluded fact.
    assert!(of_type(&copy.facts, "spoke").iter().any(|fact| {
        residents.contains(&fact.participants()[0]) && fact.participants()[1] == grace
    }));
    assert!(
        of_type(&copy.facts, "became-acquainted")
            .iter()
            .any(|fact| commuters.contains(&fact.subjects()[0]))
    );
}

fn of_type_ref<'a>(facts: &[&'a EventEnvelope], kind: &str) -> Vec<&'a EventEnvelope> {
    facts
        .iter()
        .copied()
        .filter(|fact| fact.event_type().as_str() == kind)
        .collect()
}

/// The `spoke` facts `person` perceived in `world`'s save, through `mineworld perceived`.
fn perceived_lines(world: &Path, save: &Path, person: &str) -> Vec<EventEnvelope<Value>> {
    let output = mineworld(&[
        "perceived",
        world.to_str().expect("a path"),
        "--save",
        save.to_str().expect("a path"),
        "--person",
        person,
        "--json",
    ]);
    assert!(output.status.success(), "{}", stderr(&output));
    stdout(&output)
        .lines()
        .map(|line| {
            serde_json::from_str::<PerceivedEvent<Value>>(line)
                .expect("a PerceivedEvent")
                .envelope()
                .clone()
        })
        .filter(|fact| fact.event_type().as_str() == "spoke")
        .collect()
}

/// IE-7 (QIB-12).
fn perception(twin: &Ran, people: &BTreeMap<String, EntityId>) {
    let copy = ran(
        "ie7-perceived",
        &[(
            "conversation",
            "consequences:\n  - { fact: spoke, audience: participants }\n",
        )],
    );
    let overheard = |ran: &Ran| {
        seats()
            .iter()
            .map(|seat| {
                perceived_lines(&ran.world, &ran.save, seat)
                    .iter()
                    .filter(|fact| !fact.participants().contains(&people[seat.as_str()]))
                    .count()
            })
            .sum::<usize>()
    };
    let twin_overheard = overheard(twin);
    assert!(
        twin_overheard > 0,
        "IE-7 INCONCLUSIVE: no seat of the twin overhears a line"
    );
    assert_eq!(
        overheard(&copy),
        0,
        "IE-7: nobody overhears (twin: {twin_overheard})"
    );
    println!("IE-7: twin seats overheard {twin_overheard} lines; configured 0");
    assert!(
        of_type(&copy.facts, "spoke")
            .iter()
            .all(|fact| *fact.visibility() == Visibility::Participants),
        "IE-7: every line is stated Participants"
    );
}

// ---- IE-10 --------------------------------------------------------------------------------------

#[test]
fn the_interactions_command_shows_the_social_sections() {
    let output = mineworld(&["interactions", PACK, "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let document: Value = serde_json::from_slice(&output.stdout).expect("JSON");
    for pack in ["conversation", "group-activity", "relationships"] {
        assert_eq!(document["sections"][pack], "default (compiled)", "{pack}");
    }

    let rules = "rules:\n  - { action: talk, actor: resident, target: commuter, effect: forbid }\n";
    let copy = configured("ie10-interactions", &classed(&[("conversation", rules)]));
    let output = mineworld(&["interactions", copy.to_str().expect("a path"), "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let document: Value = serde_json::from_slice(&output.stdout).expect("JSON");
    let conversation = document["sections"]["conversation"].to_string();
    for word in ["\"talk\"", "\"forbid\"", "\"resident\"", "\"commuter\""] {
        assert!(conversation.contains(word), "{word} in {conversation}");
    }
    assert_eq!(document["classes"]["carol"], "resident");
    assert_eq!(document["classes"]["grace"], "commuter");
    assert_eq!(document["classes"]["bob"], "person");
}
