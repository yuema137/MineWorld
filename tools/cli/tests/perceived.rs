//! What a Person perceived, from a save: presence's audience rule over a real log, and the offline
//! command that runs it (`DECISIONS.md` `ARC-43`, `MODULE_SPEC.md` §8.1, step-12 §17 C-C3, C-C3b).
//!
//! ```text
//! mineworld run worlds/social-cafe --headless --seed 7 --days 30 --save S
//! mineworld perceived worlds/social-cafe --save S --person KEY [--since ID] [--json]
//! ```
//!
//! The oracles are independent of `audience.rs`: the world the pack authors (its loader's own
//! placement, for genesis), the world the save resumes into (the kernel's reduction, for the fold),
//! and each `arrived` fact's typed payload decoded with presence's published type.

mod headless;

use headless::{PACK, Tables, fresh, mineworld, run, stderr, stdout};
use mineworld_contracts::{
    Causation, EntityId, EntityKey, Event, EventEnvelope, PerceivedEvent, PlaceId, Visibility,
    WorldTime,
};
use mineworld_persistence::{Durability, PersistentWorld, SqliteBackend, format};
use mineworld_presence::audience::Whereabouts;
use mineworld_presence::{Arrived, Presence};
use mineworld_worldpack::WorldPack;
use serde_json::Value;

fn facts_of(save: &std::path::Path) -> Vec<EventEnvelope> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
        .collect()
}

/// `mineworld perceived` for `person`: its exit, stdout lines and stderr.
fn perceived(save: &str, person: &str, extra: &[&str]) -> (bool, Vec<String>, String) {
    let mut arguments = vec!["perceived", PACK, "--save", save, "--person", person];
    arguments.extend_from_slice(extra);
    let output = mineworld(&arguments);
    (
        output.status.success(),
        stdout(&output).lines().map(str::to_owned).collect(),
        stderr(&output),
    )
}

/// The event ids of the plain lines, which begin with one.
fn ids(lines: &[String]) -> Vec<u64> {
    lines
        .iter()
        .map(|line| {
            line.split(' ')
                .next()
                .and_then(|id| id.parse().ok())
                .unwrap_or_else(|| panic!("a line beginning with an event id: {line}"))
        })
        .collect()
}

/// The oracle: which facts of `log` `person` learned of, from the contract's `Visibility` read
/// literally and `person`'s place tracked through `arrived` payloads decoded with presence's own
/// type — a second derivation, sharing no code with `audience.rs`. Also returns the place it put
/// `person` in once the genesis facts were read.
fn learned(log: &[EventEnvelope], person: EntityId) -> (Vec<u64>, Option<PlaceId>) {
    let mut here: Option<PlaceId> = None;
    let mut at_genesis_end = None;
    let mut ids = Vec::new();
    for fact in log {
        if *fact.event_type() == <Arrived as Event>::EVENT_TYPE {
            let arrived: Arrived = serde_json::from_slice(
                fact.payload()
                    .payload_for::<Arrived>()
                    .expect("an arrived record"),
            )
            .expect("an arrived payload");
            if arrived.person().entity_id() == person {
                here = Some(arrived.location().place());
            }
        }
        if *fact.caused_by() != Causation::WorldGenesis && at_genesis_end.is_none() {
            at_genesis_end = Some(here);
        }
        let named = fact.subjects().contains(&person) || fact.participants().contains(&person);
        let heard = match fact.visibility() {
            Visibility::Public => true,
            Visibility::Participants => fact.participants().contains(&person),
            Visibility::Entities(set) => set.contains(&person),
            Visibility::Place(place) => named || here == Some(*place),
            Visibility::SystemInternal => false,
        };
        if heard {
            ids.push(fact.id().raw());
        }
    }
    (ids, at_genesis_end.flatten())
}

/// S10's A-1, the premise of a live fold seeded from components agreeing with an offline fold from
/// facts (step-12 §17.9 R-SC1): over a 30-day run, the fold of every fact equals presence's own
/// components in the world the save resumes into. Failing it is a stop, not a test to adjust.
#[test]
fn the_fold_of_a_whole_log_equals_presence_in_the_resumed_world() {
    let save = fresh("perceived-fold-30");
    run(7, 30, Some(&save));
    let facts = facts_of(&save);

    let mut folded = Whereabouts::new();
    for fact in &facts {
        folded.apply(fact);
    }

    let pack = WorldPack::read(PACK).expect("reads");
    let composed = pack.compose().expect("composes");
    let backend = SqliteBackend::open(&save, Durability::ProcessCrash).expect("opens");
    let (world, _) =
        PersistentWorld::resume(Box::new(backend), composed.world).expect("the save resumes");
    let read = world.world().read();
    let seeded = Whereabouts::from_world(&read);
    assert_eq!(
        folded, seeded,
        "the fold from facts and the components disagree"
    );

    // Not vacuous: everybody is somewhere, and people walked after genesis.
    let people = read.components::<Presence>().count();
    assert!(people >= 10, "only {people} people placed");
    let walked = facts
        .iter()
        .filter(|fact| {
            *fact.event_type() == <Arrived as Event>::EVENT_TYPE
                && *fact.caused_by() != Causation::WorldGenesis
        })
        .count();
    assert!(walked > 100, "only {walked} arrivals after genesis");
    eprintln!(
        "{} facts, {people} people placed, {walked} arrivals after genesis",
        facts.len()
    );
}

/// C-C3b: `mineworld perceived` over a 30-day save is exactly the log judged for one person — sound,
/// complete, ordered, resumable by `--since`, refusing an unknown person by name, and in the server's
/// wire form with `--json`.
#[test]
fn the_export_is_the_log_judged_for_one_person() {
    let save = fresh("perceived-export-30");
    run(7, 30, Some(&save));
    let path = save.to_str().expect("a printable path");
    let log = facts_of(&save);
    let wanderer = WorldPack::read(PACK)
        .expect("reads")
        .load(WorldTime::EPOCH)
        .expect("loads");
    let authored = wanderer
        .world()
        .read()
        .component::<Presence>(
            wanderer
                .id(&EntityKey::new("wanderer").expect("a key"))
                .expect("declared"),
        )
        .map(|presence| presence.location().place());
    let person = wanderer
        .id(&EntityKey::new("wanderer").expect("a key"))
        .expect("declared");

    let (ok, lines, err) = perceived(path, "wanderer", &[]);
    assert!(ok, "{err}");
    let exported = ids(&lines);

    // Sound and complete against the independent oracle; every id a fact of the save.
    let (expected, placed_at_genesis) = learned(&log, person);
    assert_eq!(
        placed_at_genesis, authored,
        "the oracle's genesis placement is the pack's authored one"
    );
    let in_save: std::collections::BTreeSet<u64> = log.iter().map(|f| f.id().raw()).collect();
    assert!(exported.iter().all(|id| in_save.contains(id)));
    assert!(
        exported.windows(2).all(|pair| pair[0] < pair[1]),
        "ascending, no duplicate"
    );
    assert_eq!(exported, expected, "the export and the oracle differ");

    // Not vacuous: beyond genesis, the wanderer overheard Place facts that do not name them.
    let by_id: std::collections::BTreeMap<u64, &EventEnvelope> =
        log.iter().map(|f| (f.id().raw(), f)).collect();
    let overheard = exported
        .iter()
        .filter(|id| {
            let fact = by_id[id];
            *fact.caused_by() != Causation::WorldGenesis
                && matches!(fact.visibility(), Visibility::Place(_))
                && !fact.subjects().contains(&person)
                && !fact.participants().contains(&person)
        })
        .count();
    eprintln!(
        "wanderer perceived {} of {} facts, {overheard} overheard",
        exported.len(),
        log.len()
    );
    assert!(overheard > 0, "nothing overheard after genesis");

    // --since X: exactly the suffix after X.
    let cursor = exported[exported.len() / 2];
    let (ok, lines, err) = perceived(path, "wanderer", &["--since", &cursor.to_string()]);
    assert!(ok, "{err}");
    let suffix: Vec<u64> = exported.iter().copied().filter(|id| *id > cursor).collect();
    assert_eq!(ids(&lines), suffix);

    // --json: one PerceivedEvent per line, the same facts.
    let (ok, lines, err) = perceived(path, "wanderer", &["--json"]);
    assert!(ok, "{err}");
    let decoded: Vec<u64> = lines
        .iter()
        .map(|line| {
            serde_json::from_str::<PerceivedEvent<Value>>(line)
                .unwrap_or_else(|error| panic!("{error}: {line}"))
                .envelope()
                .id()
                .raw()
        })
        .collect();
    assert_eq!(decoded, exported);

    // An unknown person is refused by name.
    let (ok, lines, err) = perceived(path, "nobody", &[]);
    assert!(!ok && lines.is_empty(), "an unknown person is refused");
    assert!(err.contains("'nobody' is not a person"), "{err}");
}
