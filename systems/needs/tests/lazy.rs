//! AC-Nb-3 (AC-S35-5) and AC-Nb-4: decay is lazy and survives a restart.
//!
//! A seeded script — test code, its own draws, never the pack's — drives every person of each test
//! world through the real dispatch: every 15 world minutes each awake person may eat a food they might
//! hold (consumption decides), or may sleep where they are for 6 … 9 hours (sleep decides). Over the
//! run `needs` states exactly its genesis facts — one `needs-assigned` per person with the section, one
//! `nourishment-assigned` per item kind with the item form, its configured fact when configured — and
//! nothing after genesis; no Process is ever owned by `needs`; and a person's stored `Needs` changes
//! only at a nourishing meal, at falling asleep or at waking (M-Nb-3's byte check). The script moves
//! nobody: each person with needs starts in the residential place, where sleep is allowed.

mod support;

use std::collections::BTreeMap;

use mineworld_contracts::{EventEnvelope, WorldTime};
use mineworld_kernel::World;
use mineworld_needs::{Needs, NeedsSystem};
use mineworld_persistence::{Creation, Durability, PersistentWorld, SqliteBackend};
use mineworld_presence::PerceptionProvider;
use mineworld_sleep::Asleep;
use support::{Assembled, GENESIS, HOUR, Kind, Town, accepted, assemble, compose, hour};

/// A test-local generator (SplitMix64): the script's draws.
struct Draws(u64);

impl Draws {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

/// Each world's people and the foods its people might hold.
fn cast(kind: Kind) -> ([&'static str; 3], [&'static str; 2]) {
    match kind {
        Kind::FlatAndCafe => (["ana", "ben", "cy"], ["soup", "apple"]),
        Kind::NightShift => (["dee", "eli", "fay"], ["stew", "bread"]),
    }
}

/// `needs`' own fact types.
const OWN: [&str; 3] = [
    "needs-assigned",
    "nourishment-assigned",
    "needs-interactions-configured",
];

/// What a run saw: meals and sleeps per person.
#[derive(Default)]
struct Seen {
    meals: BTreeMap<&'static str, u32>,
    sleeps: BTreeMap<&'static str, u32>,
}

/// Whether `facts` hold a reason for `who`'s `Needs` to change: a meal of a nourishing kind, or
/// falling asleep, or waking.
fn explains(town: &Town, facts: &[EventEnvelope], who: &str, nourishing: &[&str]) -> bool {
    let id = town.id(who);
    let foods: Vec<_> = nourishing.iter().map(|food| town.item(food)).collect();
    facts.iter().any(|fact| match fact.event_type().as_str() {
        "items-consumed" => {
            let consumed: mineworld_inventory::ItemsConsumed =
                serde_json::from_slice(fact.payload().payload()).expect("decodes");
            consumed.holder() == id && foods.contains(&consumed.item())
        }
        "fell-asleep" | "woke" => fact.participants().contains(&id),
        _ => false,
    })
}

fn run(kind: Kind, seed: u64, days: i64) -> Seen {
    let mut town = Town::new(kind);
    let (people, foods) = cast(kind);
    let genesis = town.log.len();
    let mut draws = Draws(seed);
    let mut seen = Seen::default();
    let steps = days * 24 * 4;
    for step in 1..=steps {
        let mut facts = town.pass_to(WorldTime::from_seconds(step * 15 * 60));
        let before: Vec<Option<Needs>> = people.iter().map(|who| town.needs(who)).collect();
        for who in people {
            let asleep = town
                .world
                .read()
                .component::<Asleep>(town.id(who))
                .is_some();
            if asleep {
                continue;
            }
            match draws.below(16) {
                0 | 1 => {
                    let food = foods[usize::try_from(draws.below(2)).expect("small")];
                    let len = town.log.len();
                    if accepted(&town.eat(who, food)) {
                        *seen.meals.entry(who).or_default() += 1;
                    }
                    facts.extend(town.log[len..].iter().cloned());
                }
                2 => {
                    let minutes = 360 + u16::try_from(draws.below(181)).expect("small");
                    let len = town.log.len();
                    if accepted(&town.sleep(who, minutes)) {
                        *seen.sleeps.entry(who).or_default() += 1;
                    }
                    facts.extend(town.log[len..].iter().cloned());
                }
                _ => {}
            }
        }
        for (who, was) in people.iter().zip(before) {
            if town.needs(who) != was {
                assert!(
                    explains(&town, &facts, who, &foods),
                    "{kind:?} seed {seed}: {who}'s Needs changed with no meal or sleep fact"
                );
            }
        }
        if step % 16 == 0 {
            assert!(
                town.world.read().processes().all(|process| *process.owner()
                    != <NeedsSystem as mineworld_kernel::SystemIdentity>::ID),
                "no needs Process"
            );
        }
    }

    let own: Vec<(&str, WorldTime)> = town
        .log
        .iter()
        .filter(|fact| OWN.contains(&fact.event_type().as_str()))
        .map(|fact| (fact.event_type().as_str(), fact.at()))
        .collect();
    let count = |name: &str| own.iter().filter(|(kind, _)| *kind == name).count();
    assert_eq!(count("needs-assigned"), 2, "{kind:?}");
    assert_eq!(count("nourishment-assigned"), 2, "{kind:?}");
    assert_eq!(
        count("needs-interactions-configured"),
        usize::from(kind == Kind::NightShift),
        "{kind:?}"
    );
    assert!(own.iter().all(|(_, at)| *at == GENESIS), "genesis only");
    assert!(
        town.log[genesis..]
            .iter()
            .all(|fact| !OWN.contains(&fact.event_type().as_str())),
        "{kind:?}: no fact after genesis"
    );
    seen
}

#[test]
fn needs_states_only_its_genesis_facts_over_two_days() {
    for kind in [Kind::FlatAndCafe, Kind::NightShift] {
        for seed in [7, 42] {
            let seen = run(kind, seed, 2);
            let (people, _) = cast(kind);
            for who in &people[..2] {
                assert!(
                    seen.meals.get(who).copied().unwrap_or(0) >= 1,
                    "{kind:?} seed {seed}: {who} ate"
                );
                assert!(
                    seen.sleeps.get(who).copied().unwrap_or(0) >= 1,
                    "{kind:?} seed {seed}: {who} slept"
                );
            }
        }
    }
}

/// The milestone run (AC-S35-5): 30 days per world and seed.
#[test]
#[ignore = "milestone batch: 30 simulated days per world and seed"]
fn needs_states_only_its_genesis_facts_over_thirty_days() {
    for kind in [Kind::FlatAndCafe, Kind::NightShift] {
        for seed in [7, 42] {
            run(kind, seed, 30);
        }
    }
}

// ---- AC-Nb-4: kill and resume --------------------------------------------------------------------

const INSTANCE: u128 = 0x5eed_0000_0000_0000_0000_0000_0035_000b;

fn create(
    scratch: &mineworld_test_support::Scratch,
) -> (
    PersistentWorld,
    BTreeMap<String, mineworld_contracts::EntityId>,
) {
    let Assembled {
        world, keys, facts, ..
    } = assemble(Kind::FlatAndCafe, true);
    let backend = SqliteBackend::create(scratch, Durability::ProcessCrash).expect("creates");
    let (persisted, _) = PersistentWorld::create(
        Box::new(backend),
        world,
        Creation {
            instance: INSTANCE,
            pack: "needs-test".to_owned(),
            at: GENESIS,
            facts,
        },
    )
    .expect("a save is created");
    (persisted, keys)
}

fn request<A: mineworld_contracts::Action + serde::Serialize>(
    persisted: &mut PersistentWorld,
    who: mineworld_contracts::EntityId,
    action: &A,
    id: u64,
) {
    let at = persisted.world().read().at();
    let intent = mineworld_contracts::ActionIntent::new(
        mineworld_contracts::ActionId::from_raw(id),
        who,
        support::record(action),
        at,
    );
    let done = persisted.dispatch(&intent, at).expect("dispatches");
    assert!(accepted(done.result()));
}

/// Until 05:30: ana eats a soup at 05:00.
fn before_the_stop(
    persisted: &mut PersistentWorld,
    keys: &BTreeMap<String, mineworld_contracts::EntityId>,
) {
    let _ = persisted.advance_to(hour(5)).expect("advances");
    let soup =
        mineworld_contracts::ItemId::new(keys["soup"], mineworld_contracts::EntityType::Item)
            .expect("an item");
    request(
        persisted,
        keys["ana"],
        &mineworld_consumption::Eat::new(soup),
        1,
    );
    let _ = persisted
        .advance_to(WorldTime::from_seconds(5 * HOUR + 1_800))
        .expect("advances");
}

/// Disclosed `(hunger, tiredness)` of each person with needs at 06:00, 12:00 and 23:00, with ana
/// sleeping eight hours at 06:00 and ben eating his apple at 12:00.
fn after_the_stop(
    persisted: &mut PersistentWorld,
    keys: &BTreeMap<String, mineworld_contracts::EntityId>,
) -> Vec<(u64, u64)> {
    let providers = compose(true).1;
    let mut read = Vec::new();
    for h in [6, 12, 23] {
        let _ = persisted.advance_to(hour(h)).expect("advances");
        for who in ["ana", "ben"] {
            read.push(levels(persisted.world(), keys[who], hour(h), &providers));
        }
        if h == 6 {
            request(persisted, keys["ana"], &mineworld_sleep::Sleep::new(480), 2);
        }
        if h == 12 {
            let apple = mineworld_contracts::ItemId::new(
                keys["apple"],
                mineworld_contracts::EntityType::Item,
            )
            .expect("an item");
            request(
                persisted,
                keys["ben"],
                &mineworld_consumption::Eat::new(apple),
                3,
            );
        }
    }
    read
}

fn levels(
    world: &World,
    who: mineworld_contracts::EntityId,
    at: WorldTime,
    providers: &[Box<dyn PerceptionProvider>],
) -> (u64, u64) {
    let providers: Vec<&dyn PerceptionProvider> = providers.iter().map(AsRef::as_ref).collect();
    let observation = mineworld_presence::observe(world, who, at, &providers);
    let record = observation
        .entities()
        .iter()
        .flat_map(|entity| entity.components())
        .find(|record| record.entity() == who && record.component_type().as_str() == "needs")
        .expect("disclosed to oneself")
        .payload()
        .clone();
    (
        record["hunger"].as_u64().expect("a number"),
        record["tiredness"].as_u64().expect("a number"),
    )
}

#[test]
fn a_world_stopped_and_resumed_discloses_the_same_levels() {
    let straight = mineworld_test_support::scratch!(empty "needs-straight");
    let (mut uninterrupted, keys) = create(&straight);
    before_the_stop(&mut uninterrupted, &keys);
    let expected = after_the_stop(&mut uninterrupted, &keys);

    let stopped = mineworld_test_support::scratch!(empty "needs-stopped");
    let (mut first, keys) = create(&stopped);
    before_the_stop(&mut first, &keys);
    let stored = first
        .world()
        .read()
        .component::<Needs>(keys["ana"])
        .copied();
    drop(first);

    let backend = SqliteBackend::open(&stopped, Durability::ProcessCrash).expect("opens");
    let (mut resumed, _) =
        PersistentWorld::resume(Box::new(backend), compose(true).0).expect("resumes");
    assert_eq!(
        resumed
            .world()
            .read()
            .component::<Needs>(keys["ana"])
            .copied(),
        stored,
        "Needs survived"
    );
    let got = after_the_stop(&mut resumed, &keys);
    // By hand, ana: 06:00 (420, 1 000); asleep, 12:00 (600, 82); the alarm at 14:00 (660, 35), then
    // awake 9 h → 23:00 (1 000, 575). Tiredness recovers in proportion to its level (Q-Nc3-5).
    // ben: 06:00 (1 000, 1 000); 12:00 (1 000, 1 000), apple → 800; 23:00 1 000 (clamped), 1 000.
    assert_eq!(
        got,
        vec![
            (420, 1_000),
            (1_000, 1_000),
            (600, 82),
            (1_000, 1_000),
            (1_000, 575),
            (1_000, 1_000),
        ]
    );
    assert_eq!(got, expected, "the uninterrupted world's levels");
}
