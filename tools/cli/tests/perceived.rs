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

use headless::{PACK, Tables, fresh, run};
use mineworld_contracts::{Causation, Event, EventEnvelope};
use mineworld_persistence::{Durability, PersistentWorld, SqliteBackend, format};
use mineworld_presence::audience::Whereabouts;
use mineworld_presence::{Arrived, Presence};
use mineworld_worldpack::WorldPack;

fn facts_of(save: &std::path::Path) -> Vec<EventEnvelope> {
    Tables::read(save)
        .facts
        .iter()
        .map(|(_, bytes)| format::decode(bytes, "fact").expect("a fact"))
        .collect()
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
