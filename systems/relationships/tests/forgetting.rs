//! Forgetting in relationships, scripted over a real café (`pr-fg-a-relationship-forgetting.md` §6;
//! `docs/DECISIONS.md` `ARC-86`).
//!
//! ```text
//! FA-5  lazy, and no decay fact: half-life 1 day for everyone. Alice and Bob reach familiarity 300,
//!       regard 140 (Friendly) at t0; a day passes with nobody meeting:
//!   (a) Alice's observation at t0 + 1 d discloses familiarity 150, same record shape as unconfigured
//!   (b) the stored values still hold 300, last contact t0
//!   (c) no relationships fact between t0 and t0 + 1 d
//!   (d) a talk at t0 + 1 d + 1 s stores 150 + 10 = 160 with last contact then, and states
//!       relationship-changed Friendly → Acquaintance, caused by that talk's spoke
//!   (e) the same script unconfigured discloses 300 at t0 + 1 d
//!       M-FA5a (disclosure returns stored) → (a); M-FA5b (contact adds to stored) → (d);
//!       M-FA5c (the stated level read after the rebase) → (d)'s fact missing
//! FA-3  the cap (AC-FG-7, scripted): a 300-person crowd under the default 150 over 100 simulated days
//!       — (a) entries ≤ 150 and edges = entries after every reduction, (b) 5 000 forgotten, each the
//!       oldest tie, caused by the spoke, absent after, (c) the 160 KiB bound on component data;
//!       (d) the eviction order with known_at_most 5
//!       M-FA3a (rank by highest familiarity) → (d); M-FA3b (no unrelate) → (a);
//!       M-FA3c (the newcomer eligible) → (d)
//! ```
//!
//! Hand arithmetic: two hours together are familiarity 2 × 50 = 100 and regard 2 × (50 + 20) = 140;
//! twenty talks add 20 × 10 = 200 more familiarity, all within 40 s of each other — under a 64th of
//! the half-life (1 350 s), so the table's first step, exactly 1 (`HALF[0]`), applies between them.
//! A day later: k = 1, i = 0, 300 >> 1 = 150; one second more is still i = 0.

mod support;

use mineworld_contracts::{ActionResult, Causation, EntityId, EventEnvelope, PersonId, WorldTime};
use mineworld_conversation::ConversationHistory;
use mineworld_relationships::{Acquaintances, Level, RelationshipChanged};
use serde_json::Value;
use support::crowd::Crowd;
use support::{ALL, Cafe, of_type};

const DAY: i64 = 86_400;

const HALF_LIFE_A_DAY: &str =
    r#"{ "parameters": [ { "familiarity_decay": "half-life", "familiarity_decay_days": 1 } ] }"#;

fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

/// Two hours together, then twenty talks: familiarity 300, regard 140 both ways — Friendly.
fn become_friendly(cafe: &mut Cafe) {
    let (alice, bob) = (cafe.alice, cafe.bob);
    for _ in 0..2 {
        assert!(accepted(&cafe.invite(alice, bob).0));
        assert!(accepted(&cafe.accept(bob, alice).0));
        assert!(accepted(&cafe.leave(bob).0));
    }
    for _ in 0..20 {
        assert!(accepted(&cafe.talk(alice, bob).0));
    }
}

/// Alice's own disclosed acquaintances record, as the observation carries it.
fn disclosed(cafe: &Cafe) -> Value {
    let alice = cafe.alice;
    cafe.observation(alice)
        .entity(alice)
        .expect("herself")
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "acquaintances")
        .expect("her acquaintances are disclosed to her")
        .payload()
        .clone()
}

/// The familiarity `record` discloses for its only entry.
fn familiarity(record: &Value) -> i64 {
    record["known"][0]["values"]["familiarity"]
        .as_i64()
        .expect("an integer")
}

/// The shape of a record: its keys, recursively, without the values.
fn shape(value: &Value) -> Value {
    match value {
        Value::Object(map) => Value::Object(
            map.iter()
                .map(|(key, value)| (key.clone(), shape(value)))
                .collect(),
        ),
        Value::Array(items) => Value::Array(items.iter().map(shape).collect()),
        _ => Value::Null,
    }
}

fn changed(fact: &EventEnvelope) -> RelationshipChanged {
    serde_json::from_slice(
        fact.payload()
            .payload_for::<RelationshipChanged>()
            .expect("its own type"),
    )
    .expect("decodes")
}

#[test]
fn a_day_without_contact_halves_familiarity_in_disclosure_and_at_the_next_contact_only() {
    let mut cafe = Cafe::configured(ALL, HALF_LIFE_A_DAY);
    let (alice, bob) = (cafe.alice, cafe.bob);
    become_friendly(&mut cafe);
    let t0 = cafe.now();
    let stored = cafe.values(alice, bob).expect("alice knows bob");
    assert_eq!(
        (stored.familiarity(), stored.regard(), stored.level()),
        (300, 140, Level::Friendly)
    );
    assert_eq!(stored.last_contact(), t0);

    // (c) A day passes with nobody meeting, and relationships states nothing.
    let passed = cafe.pass(DAY);
    assert!(
        passed
            .iter()
            .all(|fact| fact.provenance().emitted_by().as_str() != "relationships"),
        "decay writes no fact: {passed:?}"
    );

    // (a) What Alice is shown has faded; (b) what is stored has not.
    let record = disclosed(&cafe);
    assert_eq!(familiarity(&record), 150);
    assert_eq!(
        record["known"][0]["values"]["regard"], 140,
        "regard does not decay here"
    );
    let still = cafe.values(alice, bob).expect("still known");
    assert_eq!((still.familiarity(), still.last_contact()), (300, t0));

    // (d) The next contact adds to the faded value, and states the level the fade crossed, once.
    let (result, facts) = cafe.talk(alice, bob);
    assert!(accepted(&result));
    let at = cafe.now();
    assert_eq!(at, WorldTime::from_seconds(t0.seconds() + DAY + 1));
    let after = cafe.values(alice, bob).expect("still known");
    assert_eq!(
        (after.familiarity(), after.regard(), after.last_contact()),
        (160, 140, at)
    );
    let spoke = of_type(&facts, "spoke");
    assert_eq!(spoke.len(), 1);
    let crossings: Vec<RelationshipChanged> = of_type(&facts, "relationship-changed")
        .into_iter()
        .inspect(|fact| assert_eq!(*fact.caused_by(), Causation::Event(spoke[0].id())))
        .map(changed)
        .collect();
    assert_eq!(
        crossings,
        [
            RelationshipChanged::new(
                cafe.person(alice),
                cafe.person(bob),
                Level::Friendly,
                Level::Acquaintance
            ),
            RelationshipChanged::new(
                cafe.person(bob),
                cafe.person(alice),
                Level::Friendly,
                Level::Acquaintance
            ),
        ]
    );

    // (e) Unconfigured, the same day changes nothing, and the record has the same shape.
    let mut twin = Cafe::new(ALL);
    become_friendly(&mut twin);
    twin.pass(DAY);
    let unconfigured = disclosed(&twin);
    assert_eq!(familiarity(&unconfigured), 300);
    assert_eq!(shape(&unconfigured), shape(&record));
}

// ---- FA-3: the cap -------------------------------------------------------------------------------

/// The facts of `name` among `facts`, decoded as `(person, counterpart)` entity ids.
fn pairs(facts: &[EventEnvelope], name: &str) -> Vec<(EntityId, EntityId)> {
    of_type(facts, name)
        .into_iter()
        .map(|fact| {
            let payload: Value = serde_json::from_slice(fact.payload().payload()).expect("json");
            let id = |key: &str| {
                serde_json::from_value::<PersonId>(payload[key].clone())
                    .expect("a person")
                    .entity_id()
            };
            (id("person"), id("counterpart"))
        })
        .collect()
}

/// FA-3 (a)–(c), the 300-person crowd under the default cap of 150, over 100 simulated days.
///
/// Layout: people 0 … 99 speak, 100 … 299 listen. In round r = 0 … 199 every speaker, in id order,
/// says hello to listener 100 + r; half a day passes after each round. Each pair talks once, so
/// familiarity is 10 everywhere and nothing ties but the instant. A speaker's 151st acquaintance
/// (round 150) is the first beyond the cap; from there each round evicts exactly one tie per speaker,
/// the oldest — listener 100 + r − 150. A listener hears 100 speakers and never reaches the cap. So:
/// 100 speakers × (200 − 150) rounds = 5 000 `acquaintance-forgotten`, and at the end each speaker
/// knows listeners 150 … 299 and each listener the 100 speakers.
#[test]
fn a_crowd_of_300_never_exceeds_the_cap_and_forgets_its_oldest_ties() {
    let mut crowd = Crowd::new(300, None);
    let people = crowd.people.clone();
    let (speakers, listeners) = people.split_at(100);
    let mut forgotten = 0;
    for (round, listener) in listeners.iter().enumerate() {
        for speaker in speakers {
            let facts = crowd.talk(*speaker, *listener);
            // (a) after every reduction, for both people it touched: within the cap, edges = entries.
            for holder in [*speaker, *listener] {
                let known = crowd.known(holder);
                assert!(known.len() <= 150, "{holder} knows {}", known.len());
                assert_eq!(
                    crowd.edges_from(holder),
                    known,
                    "{holder}'s edges and entries"
                );
            }
            // (b) beyond the cap, exactly the oldest tie is forgotten, caused by this talk's spoke.
            let expected: Vec<(EntityId, EntityId)> = if round >= 150 {
                vec![(*speaker, listeners[round - 150])]
            } else {
                Vec::new()
            };
            assert_eq!(
                pairs(&facts, "acquaintance-forgotten"),
                expected,
                "round {round}"
            );
            if let Some((_, victim)) = expected.first() {
                let spoke = of_type(&facts, "spoke")[0].id();
                assert!(
                    of_type(&facts, "acquaintance-forgotten")
                        .iter()
                        .all(|fact| *fact.caused_by() == Causation::Event(spoke))
                );
                assert!(!crowd.known(*speaker).contains(victim));
                assert!(!crowd.edges_from(*speaker).contains(victim));
                forgotten += 1;
            }
        }
        crowd.pass(DAY / 2);
    }
    assert_eq!(forgotten, 5_000);
    for speaker in speakers {
        assert_eq!(crowd.known(*speaker), listeners[50..].to_vec());
    }
    for listener in listeners {
        assert_eq!(crowd.known(*listener), speakers.to_vec());
    }

    // (c) the bound, on component data (QFGA-7), and the snapshot's encoding beside it.
    let read = crowd.world.read();
    let mut widest = 0;
    for person in &people {
        let known = read
            .component::<Acquaintances>(*person)
            .expect("everyone knows somebody");
        let payload = serde_json::to_vec(known).expect("encodes").len();
        let largest = known
            .known()
            .iter()
            .map(|entry| serde_json::to_vec(entry).expect("encodes").len())
            .max()
            .unwrap_or(0);
        // At most 150 entries of at most `largest` bytes, 149 commas, and `{"known":[]}` (12 bytes).
        assert!(payload <= 150 * largest + 149 + 12, "{person}: {payload} B");
        let history = read
            .component::<ConversationHistory>(*person)
            .map_or(0, |history| {
                serde_json::to_vec(history).expect("encodes").len()
            });
        assert!(
            payload + history <= 160 * 1_024,
            "{person}: {payload} + {history} B"
        );
        let snapshot_json = |bytes: usize| {
            serde_json::to_vec(&vec![255_u8; bytes])
                .expect("encodes")
                .len()
        };
        widest = widest.max(payload + history);
        if *person == speakers[0] {
            println!(
                "FA-3(c) speaker p000: acquaintances {payload} B, history {history} B; as a snapshot \
                 encodes them, at most {} B",
                snapshot_json(payload) + snapshot_json(history)
            );
        }
    }
    println!("FA-3(c) widest person: {widest} B of component data (bound 163 840 B)");
}

/// FA-3 (d): the eviction order, with `known_at_most: 5` and values set by the script. Holder `h`
/// (p000) and seven counterparts p1 … p7, all hellos from `h`:
///
/// ```text
/// A  h → p1 and h → p2 at one instant                      p1 10, p2 10, both last at T_a
/// B  h → p3 ×2, h → p4 ×2, h → p5 ×3                       p3 20, p4 20, p5 30
/// C  h → p6 (the 6th): p1 and p2 tie on familiarity and on the instant; the lower id goes → p1
/// D  h → p2 ×2, h → p6 ×1                                  p2 30, p6 20
///    h → p7 (new, 10 — the lowest of all, never evicted): the 20s are p3, p4, p6; oldest → p3
/// E  h → p1 again: a new formation, stated again; p7 is no longer the one just met, and its 10 is
///    the lowest → p7
/// ```
///
/// (The script's first oracle for E said p4, forgetting that p7's protection lasts only for the
/// reduction that formed it; the run said p7, which is §5.5's rule. Recorded as F-FGA-18.)
#[test]
fn the_cap_evicts_lowest_familiarity_then_oldest_contact_then_lowest_id_and_never_the_newcomer() {
    let section = support::configured(r#"{ "parameters": [ { "known_at_most": 5 } ] }"#, "[]");
    let mut crowd = Crowd::new(8, Some(section));
    let p = crowd.people.clone();
    let h = p[0];
    let mut victims = Vec::new();
    let forget = |facts: &[EventEnvelope]| -> Vec<EntityId> {
        pairs(facts, "acquaintance-forgotten")
            .into_iter()
            .map(|(person, counterpart)| {
                assert_eq!(person, h);
                counterpart
            })
            .collect()
    };
    // A
    assert!(forget(&crowd.talk(h, p[1])).is_empty());
    assert!(forget(&crowd.talk_now(h, p[2])).is_empty());
    // B
    for (who, times) in [(3, 2), (4, 2), (5, 3)] {
        for _ in 0..times {
            assert!(forget(&crowd.talk(h, p[who])).is_empty());
        }
    }
    // C
    victims.extend(forget(&crowd.talk(h, p[6])));
    // D
    for who in [2, 2, 6] {
        assert!(forget(&crowd.talk(h, p[who])).is_empty());
    }
    victims.extend(forget(&crowd.talk(h, p[7])));
    // E
    let again = crowd.talk(h, p[1]);
    assert_eq!(pairs(&again, "became-acquainted"), [(h, p[1])]);
    victims.extend(forget(&again));

    assert_eq!(victims, [p[1], p[3], p[7]]);
    assert_eq!(crowd.known(h), [p[1], p[2], p[4], p[5], p[6]]);
    assert_eq!(crowd.edges_from(h), crowd.known(h));
}
