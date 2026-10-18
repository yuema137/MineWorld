//! Relationships over a real café, changed only by the facts conversation and group-activity state
//! (`step-09-social.md` §4.2.3 C3).
//!
//! What would pass without proving anything, and how each is excluded (`ARC-23`):
//!
//! ```text
//! values that only ever rise        a decline's decrease is asserted, and a downward crossing
//! a fact per value change           Q5 is located: talks below a boundary state nothing; the one
//!                                   that crosses states exactly one fact per direction, caused by it
//! an edge without its entry         checked both ways after every step of a mixed script
//! a dependency hiding as a type     a world without conversation and group-activity installs it
//! ```

mod support;

use mineworld_contracts::{ActionResult, Causation, EventEnvelope};
use mineworld_relationships::{
    ACCEPTED_REGARD, ACTIVITY_FAMILIARITY, ACTIVITY_REGARD, DECLINED_REGARD, Level,
    RelationshipChanged, SPOKE_FAMILIARITY,
};
use support::{ALL, Cafe, Packs, conversation_id, of_type};

fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

fn changed(fact: &EventEnvelope) -> RelationshipChanged {
    serde_json::from_slice(
        fact.payload()
            .payload_for::<RelationshipChanged>()
            .expect("its own type"),
    )
    .expect("decodes")
}

/// One accepted invitation, then the activity ended by Bob leaving.
fn an_hour_together(cafe: &mut Cafe) -> Vec<EventEnvelope> {
    let (alice, bob) = (cafe.alice, cafe.bob);
    let mut facts = cafe.invite(alice, bob).1;
    let (result, accepted_facts) = cafe.accept(bob, alice);
    assert!(accepted(&result), "{result:?}");
    facts.extend(accepted_facts);
    let (result, left) = cafe.leave(bob);
    assert!(accepted(&result), "{result:?}");
    facts.extend(left);
    facts
}

#[test]
fn the_edge_and_the_entry_agree_after_every_reduction() {
    let mut cafe = Cafe::new(ALL);
    let (alice, bob) = (cafe.alice, cafe.bob);
    let check = |cafe: &Cafe, step: &str| {
        let mut edges = cafe.edges();
        edges.sort();
        assert_eq!(edges, cafe.entries(), "after {step}");
    };
    check(&cafe, "genesis");
    cafe.invite(alice, bob);
    check(&cafe, "an invitation (no reduction yet)");
    cafe.decline(bob, alice);
    check(&cafe, "a decline: alice → bob only");
    assert_eq!(cafe.entries(), [(alice, bob)]);
    cafe.talk(bob, alice);
    check(&cafe, "a talk: both directions");
    an_hour_together(&mut cafe);
    check(&cafe, "an activity");
    for _ in 0..5 {
        cafe.talk(alice, bob);
        check(&cafe, "talks");
    }
    assert_eq!(cafe.entries(), [(alice, bob), (bob, alice)]);
}

#[test]
fn a_level_fact_is_stated_only_when_a_boundary_is_crossed_and_by_the_fact_that_crossed_it() {
    let mut cafe = Cafe::new(ALL);
    let (alice, bob) = (cafe.alice, cafe.bob);
    // Two hours together: regard 2 × (50 + 20) = 140, familiarity 2 × 50 = 100.
    let mut before_talks = an_hour_together(&mut cafe);
    before_talks.extend(an_hour_together(&mut cafe));
    let values = cafe.values(alice, bob).expect("known");
    assert_eq!(
        (values.familiarity(), values.regard()),
        (
            2 * ACTIVITY_FAMILIARITY,
            2 * (ACCEPTED_REGARD + ACTIVITY_REGARD)
        )
    );
    assert_eq!(values.activities_shared(), 2);
    assert!(
        of_type(&before_talks, "relationship-changed").is_empty(),
        "below every boundary: no level fact"
    );
    assert_eq!(
        of_type(&before_talks, "became-acquainted").len(),
        2,
        "the edge formed once per direction"
    );

    // Familiarity 100 → 300 needs 20 talks of +10; the 20th crosses into Friendly.
    let mut crossings = Vec::new();
    for talk in 1..=25 {
        let (result, facts) = cafe.talk(alice, bob);
        assert!(accepted(&result), "talk {talk}: {result:?}");
        let level_facts = of_type(&facts, "relationship-changed");
        if talk == 20 {
            let spoke = of_type(&facts, "spoke")[0].id();
            assert_eq!(
                level_facts.len(),
                2,
                "one per direction, at the crossing talk"
            );
            for fact in &level_facts {
                assert_eq!(
                    *fact.caused_by(),
                    Causation::Event(spoke),
                    "caused by that spoke"
                );
                let crossing = changed(fact);
                assert_eq!(
                    (crossing.from(), crossing.to()),
                    (Level::Acquaintance, Level::Friendly)
                );
            }
            crossings.extend(level_facts.into_iter().cloned());
        } else {
            assert!(level_facts.is_empty(), "talk {talk} crosses nothing");
        }
    }
    assert_eq!(crossings.len(), 2);
    let values = cafe.values(alice, bob).expect("known");
    assert_eq!(values.familiarity(), 100 + 25 * SPOKE_FAMILIARITY);
    assert_eq!(values.exchanges(), 25);
}

#[test]
fn a_decline_lowers_only_the_inviters_regard_and_can_cross_a_level_downwards() {
    let mut cafe = Cafe::new(ALL);
    let (alice, bob) = (cafe.alice, cafe.bob);
    an_hour_together(&mut cafe);
    an_hour_together(&mut cafe);
    for _ in 0..20 {
        cafe.talk(alice, bob);
    }
    let alices = cafe.values(alice, bob).expect("known");
    let bobs = cafe.values(bob, alice).expect("known");
    assert_eq!((alices.regard(), alices.level()), (140, Level::Friendly));
    assert_eq!(bobs.level(), Level::Friendly);

    // Bob declines: Alice thinks less of Bob; Bob's view of Alice is untouched.
    cafe.invite(alice, bob);
    let (_, facts) = cafe.decline(bob, alice);
    assert_eq!(
        cafe.values(alice, bob).expect("known").regard(),
        140 + DECLINED_REGARD
    );
    assert_eq!(cafe.values(bob, alice).expect("known"), bobs, "asymmetric");
    assert!(
        of_type(&facts, "relationship-changed").is_empty(),
        "110: still Friendly"
    );

    // A second decline takes regard to 80, under Friendly's 100: a downward crossing, Alice's only.
    cafe.invite(alice, bob);
    let (_, facts) = cafe.decline(bob, alice);
    let level_facts = of_type(&facts, "relationship-changed");
    assert_eq!(level_facts.len(), 1);
    let crossing = changed(level_facts[0]);
    assert_eq!(crossing.person(), cafe.person(alice));
    assert_eq!(
        (crossing.from(), crossing.to()),
        (Level::Friendly, Level::Acquaintance)
    );
    assert!(crossing.from() > crossing.to(), "down");
    let declined = of_type(&facts, "invitation-declined")[0].id();
    assert_eq!(*level_facts[0].caused_by(), Causation::Event(declined));
}

#[test]
fn relationships_needs_neither_pack_it_listens_to() {
    // Installed with neither conversation nor group-activity: no registry dependency (Q6).
    let mut alone = Cafe::new(Packs {
        conversation: false,
        group_activity: false,
    });
    let (alice, bob) = (alone.alice, alone.bob);
    let (result, _) = alone.talk(alice, bob);
    assert_eq!(
        result,
        ActionResult::Unavailable,
        "no conversation in this world"
    );
    assert!(alone.entries().is_empty(), "and nothing to hear");

    // With conversation installed and then disabled, relationships stays enabled and hears nothing;
    // enabled again, the next talk is reduced (AC-2's direction).
    let mut cafe = Cafe::new(Packs {
        conversation: true,
        group_activity: false,
    });
    let (alice, bob) = (cafe.alice, cafe.bob);
    cafe.disable(&conversation_id());
    let (result, _) = cafe.talk(alice, bob);
    assert_eq!(result, ActionResult::Unavailable);
    assert!(cafe.entries().is_empty());
    cafe.enable(&conversation_id());
    let (result, facts) = cafe.talk(alice, bob);
    assert!(accepted(&result));
    assert_eq!(of_type(&facts, "became-acquainted").len(), 2);
}

/// F-IE-13: this pack looks its section up at the place of the fact it reduces. Every cause it
/// subscribes to is stated with a place, so a configured world never falls back to the compiled
/// default for lack of one — and this pack's own facts carry the same place.
#[test]
fn every_cause_this_pack_reduces_is_stated_at_a_place() {
    let mut cafe = Cafe::new(ALL);
    let (alice, bob) = (cafe.alice, cafe.bob);
    let mut facts = cafe.invite(alice, bob).1;
    facts.extend(cafe.decline(bob, alice).1);
    facts.extend(cafe.talk(alice, bob).1);
    facts.extend(an_hour_together(&mut cafe));
    for kind in [
        "spoke",
        "invitation-accepted",
        "invitation-declined",
        "group-activity-ended",
        "became-acquainted",
    ] {
        let found = of_type(&facts, kind);
        assert!(!found.is_empty(), "the script states a {kind}");
        assert!(
            found.iter().all(|fact| fact.place().is_some()),
            "every {kind} is stated at a place"
        );
    }
}

#[test]
fn how_alice_regards_bob_is_disclosed_to_alice_only() {
    let mut cafe = Cafe::new(ALL);
    let (alice, bob) = (cafe.alice, cafe.bob);
    cafe.talk(alice, bob);
    let hers = cafe.observation(alice);
    assert!(
        hers.entity(alice)
            .expect("herself")
            .components()
            .iter()
            .any(|record| record.component_type().as_str() == "acquaintances")
    );
    let his = cafe.observation(bob);
    assert!(
        his.entity(alice)
            .expect("bob perceives alice")
            .components()
            .iter()
            .all(|record| record.component_type().as_str() != "acquaintances"),
        "Alice's regard for Bob is not Bob's to read"
    );
}
