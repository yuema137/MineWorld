//! The S6 checkpoint in a headless world of real packs (step-07 §1, CP-1 … CP-3), driven through the
//! whole pipeline.
//!
//! ```text
//! CP-1  a stride and a doorway crossing are accepted, stated as presence's `arrived` by movement,
//!       and reduced by presence into Presence and the present-in edge
//! CP-2  with movement disabled, `move` is Unavailable and the world is otherwise the world that never
//!       had it — with a negative control that shows the comparison fails while movement is reachable
//! CP-3  a stride one millimetre too long, a doorway one millimetre too far, and a place that does not
//!       adjoin are refused TooFarAway by the system, and the refusal changes nothing
//! Q4    a client following the reporting rule at jog speed is never refused; one ignoring it is
//! ```
//!
//! Every bound below is a literal from the layout in `support` (`ARC-23` rule 2) and every equality
//! is preceded by a printed, located count (`ARC-23` rule 1).

mod support;

use mineworld_contracts::{
    Action, ActionRecord, ActionResult, ComponentRecord, EntityType, Event, EventEnvelope,
    Location, PlaceId, Rejection, RejectionCode,
};
use mineworld_conversation::{Talk, Utterance};
use mineworld_kernel::{EntityRegistrySnapshot, RelationStoreSnapshot, SystemIdentity};
use mineworld_movement::{Move, MovementSystem};
use mineworld_presence::{Arrived, Presence, PresenceSystem};
use serde::Serialize;
use support::{Layout, Movement, NOW, Town, at, encode};

fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

// ---------------------------------------------------------------------------------------------
// CP-1 — a move changes presence and occupancy, through presence.
// ---------------------------------------------------------------------------------------------

/// A stride inside the café, a walk to the doorway, and a crossing into the street.
///
/// Checked at each layer: the answer, the one fact (presence's type, movement's provenance), the
/// position presence holds, and the `present-in` edge set, printed before and after the crossing.
#[test]
fn a_stride_and_a_crossing_are_stated_by_movement_and_reduced_by_presence() {
    let mut town = Town::joined(|town| at(town.cafe, 2_600, 2_000));
    let visitor = town.visitor;
    let (cafe, street) = (town.cafe, town.street);

    for to in [at(cafe, 3_600, 2_000), at(cafe, 4_600, 2_000)] {
        let (result, events) = town.r#move(visitor, to);
        assert!(accepted(&result), "a stride inside the café: {result:?}");
        println!("stride to {to:?}: {} fact(s)", events.len());
        assert_eq!(events.len(), 1, "one fact per move");
        assert_eq!(*events[0].event_type(), Arrived::EVENT_TYPE);
        assert_eq!(events[0].provenance().emitted_by(), &MovementSystem::ID);
        assert_eq!(town.presence(visitor), Some(to));
        assert_eq!(town.present_in(visitor), vec![cafe.entity_id()]);
    }

    let before = town.present_in(visitor);
    let onto_the_street = at(street, 1_000, 2_000);
    let (result, events) = town.r#move(visitor, onto_the_street);
    let after = town.present_in(visitor);
    println!(
        "crossing: present-in before {before:?}, after {after:?}; {} fact(s)",
        events.len()
    );
    assert!(accepted(&result), "through the doorway: {result:?}");
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].provenance().emitted_by(), &MovementSystem::ID);
    assert_eq!(before, vec![cafe.entity_id()]);
    assert_eq!(
        after,
        vec![street.entity_id()],
        "occupancy moved, and only one edge is left"
    );
    assert_eq!(town.presence(visitor), Some(onto_the_street));
}

// ---------------------------------------------------------------------------------------------
// CP-3 — refused for distance, by the system, changing nothing.
// ---------------------------------------------------------------------------------------------

/// One case of the boundary: a visitor begun at `from`, asking for `to`, and the expected answer.
struct Case {
    name: &'static str,
    from: fn(&Town) -> Location,
    to: fn(&Town) -> Location,
    expected: Option<Rejection>,
}

#[test]
fn the_stride_and_the_doorway_are_bounded_to_the_millimetre_and_a_refusal_changes_nothing() {
    let cases = [
        Case {
            name: "exactly 2 000 mm inside the café",
            from: |t| at(t.cafe, 2_600, 2_000),
            to: |t| at(t.cafe, 4_600, 2_000),
            expected: None,
        },
        Case {
            name: "2 001 mm inside the café",
            from: |t| at(t.cafe, 2_600, 2_000),
            to: |t| at(t.cafe, 4_601, 2_000),
            expected: Some(Rejection::TooFarAway),
        },
        Case {
            name: "through the doorway, exactly 2 000 mm from it on both sides",
            from: |t| at(t.cafe, 2_600, 2_000),
            to: |t| at(t.street, 2_000, 2_000),
            expected: None,
        },
        Case {
            name: "through the doorway from 2 001 mm off it",
            from: |t| at(t.cafe, 2_599, 2_000),
            to: |t| at(t.street, 0, 2_000),
            expected: Some(Rejection::TooFarAway),
        },
        Case {
            name: "landing 2 001 mm off the far side of the doorway",
            from: |t| at(t.cafe, 4_600, 2_000),
            to: |t| at(t.street, 2_001, 2_000),
            expected: Some(Rejection::TooFarAway),
        },
        Case {
            name: "a place with no passage, from beside the doorway",
            from: |t| at(t.cafe, 4_600, 2_000),
            to: |t| at(t.attic, 0, 0),
            expected: Some(Rejection::TooFarAway),
        },
    ];

    let mut refusals = 0;
    for case in &cases {
        let mut town = Town::joined(case.from);
        let to = (case.to)(&town);
        let start = (case.from)(&town);
        let before = town.snapshot_bytes();
        let (result, events) = town.r#move(town.visitor, to);
        println!("{}: {result:?}", case.name);
        match &case.expected {
            None => {
                assert!(accepted(&result), "{}: {result:?}", case.name);
                assert_eq!(town.presence(town.visitor), Some(to), "{}", case.name);
            }
            Some(rejection) => {
                refusals += 1;
                assert_eq!(
                    result,
                    ActionResult::Rejected(rejection.clone()),
                    "{}",
                    case.name
                );
                assert!(
                    events.is_empty(),
                    "{}: a refusal records nothing",
                    case.name
                );
                assert_eq!(town.presence(town.visitor), Some(start), "{}", case.name);
                assert_eq!(
                    town.snapshot_bytes(),
                    before,
                    "{}: and changes nothing",
                    case.name
                );
            }
        }
    }
    println!("{} cases, {refusals} refused", cases.len());
    assert_eq!((cases.len(), refusals), (6, 4));
}

/// The refusals that are not about distance: nobody to move, nowhere to go, nothing that can walk,
/// and a request this pack cannot read.
#[test]
fn a_move_that_makes_no_sense_is_refused_for_what_is_wrong_with_it() {
    let mut town = Town::joined(|town| at(town.cafe, 2_600, 2_000));
    let (cafe, stranger, visitor) = (town.cafe, town.stranger, town.visitor);

    let (result, _) = town.r#move(stranger, at(cafe, 0, 0));
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed),
        "somebody presence was never told about has nowhere to move from — placement is genesis"
    );

    let not_a_place = PlaceId::new(town.alice, EntityType::Place).expect("the id trusts its claim");
    let (result, _) = town.r#move(visitor, Location::in_place(not_a_place));
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed)
    );

    let (result, _) = town.r#move(cafe.entity_id(), at(cafe, 0, 0));
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::NoSupportedInteraction),
        "a place does not walk"
    );

    let (result, events) = town.submit(visitor, ActionRecord::new::<Move>(b"not a move".to_vec()));
    assert!(events.is_empty());
    match result {
        ActionResult::Rejected(Rejection::System { code, .. }) => {
            assert_eq!(code, RejectionCode::from_static("malformed-payload"));
        }
        other => panic!("an unreadable payload is this pack's own refusal, not {other:?}"),
    }
}

/// A world that models no position: a move within a place and through a passage are accepted, and a
/// place that does not adjoin is still too far — the contract's degeneracy, stated as behaviour.
#[test]
fn a_semantic_world_moves_within_places_and_through_passages_and_no_further() {
    let mut town = Town::new(Movement::Enabled, Layout::Semantic, |town| {
        Location::in_place(town.cafe)
    });
    let visitor = town.visitor;
    let (cafe, street, attic) = (town.cafe, town.street, town.attic);

    let (within, _) = town.r#move(visitor, Location::in_place(cafe));
    let (through, _) = town.r#move(visitor, Location::in_place(street));
    let after_through = town.present_in(visitor);
    let (beyond, _) = town.r#move(visitor, Location::in_place(attic));
    println!("within {within:?}, through {through:?}, beyond {beyond:?}");
    assert!(accepted(&within));
    assert!(accepted(&through));
    assert_eq!(after_through, vec![street.entity_id()]);
    assert_eq!(beyond, ActionResult::Rejected(Rejection::TooFarAway));
    assert_eq!(town.presence(visitor), Some(Location::in_place(street)));
}

// ---------------------------------------------------------------------------------------------
// CP-2 — AC-2: removing movement removes `move`, and nothing else.
// ---------------------------------------------------------------------------------------------

/// Everything a world holds that is not movement's own: identity, presence, the edges, every
/// recorded fact, and what each person is told.
#[derive(Debug, PartialEq, Eq, Serialize)]
struct EverythingElse {
    entities: EntityRegistrySnapshot,
    presence: Vec<ComponentRecord<serde_json::Value>>,
    relations: RelationStoreSnapshot,
    facts: Vec<serde_json::Value>,
    results: Vec<String>,
    observations: Vec<serde_json::Value>,
}

/// Where the visitor asks to step: 1 709 mm from where they begin, (4 600, 200), so a world with
/// walking accepts it — and 1 811 mm from Alice at (1 200, 1 000), inside talking range, where the
/// starting point is 3 493 mm away and outside it. A reachable movement system therefore changes the
/// facts, the positions and the talk's answer, which is what lets the negative control fail loudly.
fn toward_alice(town: &Town) -> Location {
    at(town.cafe, 3_000, 800)
}

/// The one script every configuration runs: the visitor asks to take a stride toward Alice, then
/// talks to her. Without walking, the stride is unavailable and the talk is answered as it would be
/// in a world that never had walking.
fn run(town: &mut Town) -> EverythingElse {
    let mut facts: Vec<EventEnvelope> = town.genesis.clone();
    let mut results = Vec::new();
    let toward_alice = toward_alice(town);
    let (result, events) = town.r#move(town.visitor, toward_alice);
    results.push(format!("{result:?}"));
    facts.extend(events);
    let talk = Talk::new(Utterance::new("good morning").expect("an utterance"));
    let alice = town.alice;
    let id = town.next_id();
    let intent = mineworld_contracts::ActionIntent::new(
        id,
        town.visitor,
        ActionRecord::new::<Talk>(encode(&talk)),
        NOW,
    )
    .with_target(alice);
    let dispatched = town.world.dispatch(&intent, NOW).expect("dispatch answers");
    results.push(format!("{:?}", dispatched.result()));
    facts.extend(dispatched.events().to_vec());

    EverythingElse {
        entities: EntityRegistrySnapshot::from(town.world.entities()),
        presence: town
            .world
            .components()
            .iter::<Presence>()
            .map(|(entity, presence)| {
                ComponentRecord::new::<Presence>(
                    entity,
                    serde_json::to_value(presence).expect("json"),
                )
            })
            .collect(),
        relations: RelationStoreSnapshot::from(town.world.relations()),
        facts: facts
            .iter()
            .map(|fact| serde_json::to_value(fact).expect("json"))
            .collect(),
        results,
        observations: [town.visitor, town.alice]
            .into_iter()
            .map(|observer| serde_json::to_value(town.observation(observer)).expect("json"))
            .collect(),
    }
}

/// Which of the AC-2 claims `candidate` breaks, measured against the world that never had movement.
///
/// The claims are stated once, here, so the test can apply the same claims to a world in which they
/// must hold and to one in which they must not — the second is what shows the first could fail.
fn removability_violations(candidate: Movement) -> Vec<String> {
    let mut violations = Vec::new();
    let mut town = Town::new(candidate, Layout::CafeOnly, |town| {
        at(town.cafe, 4_600, 200)
    });
    let mut reference = Town::new(Movement::Absent, Layout::CafeOnly, |town| {
        at(town.cafe, 4_600, 200)
    });

    let (answer, events) = town.r#move(town.visitor, toward_alice(&town));
    if answer != ActionResult::Unavailable {
        violations.push(format!("move was answered {answer:?}, not Unavailable"));
    }
    if !events.is_empty() {
        violations.push(format!("move recorded {} fact(s)", events.len()));
    }
    if town
        .observation(town.visitor)
        .affordances()
        .iter()
        .any(|affordance| *affordance.action_type() == Move::ACTION_TYPE)
    {
        violations.push("move is still offered".to_owned());
    }

    // The same script from the start, against both worlds.
    let mut town = Town::new(candidate, Layout::CafeOnly, |town| {
        at(town.cafe, 4_600, 200)
    });
    let ours = run(&mut town);
    let theirs = run(&mut reference);
    for (field, same) in [
        ("entities", ours.entities == theirs.entities),
        ("presence", ours.presence == theirs.presence),
        ("relations", ours.relations == theirs.relations),
        ("facts", ours.facts == theirs.facts),
        ("results", ours.results == theirs.results),
        ("observations", ours.observations == theirs.observations),
    ] {
        if !same {
            violations.push(format!(
                "{field} differ from the world that never had movement"
            ));
        }
    }
    violations
}

/// `AC-2` for movement, with its own negative control (`ARC-23`).
///
/// A disabled movement system must leave a world indistinguishable — in identity, positions, edges,
/// every fact, every answer and every observation — from the same world composed without it, and its
/// action must be `Unavailable` and unoffered. The same claims applied to a world where movement is
/// still reachable must be broken, or the comparison is not one that could fail.
#[test]
fn disabling_movement_makes_move_unavailable_and_changes_nothing_else() {
    let disabled = removability_violations(Movement::Disabled);
    let reachable = removability_violations(Movement::Enabled);
    println!("disabled: {} violation(s) {disabled:?}", disabled.len());
    println!(
        "reachable (negative control): {} violation(s) {reachable:?}",
        reachable.len()
    );
    assert!(disabled.is_empty(), "AC-2: {disabled:#?}");
    for expected in [
        "move was answered",
        "move is still offered",
        "presence differ",
        "facts differ",
    ] {
        assert!(
            reachable
                .iter()
                .any(|violation| violation.starts_with(expected)),
            "the claims must fail for a world where movement is reachable — `{expected}`: \
             {reachable:#?}"
        );
    }
}

// ---------------------------------------------------------------------------------------------
// Q4 — the client reporting rule, at jog speed.
// ---------------------------------------------------------------------------------------------

/// A body jogging along a straight line at 2 600 mm/s for 20 s, sampled every 100 ms: 260 mm per
/// sample, 200 samples, from x = 0 to x = 52 000 in one long place. Literals throughout.
const PER_SAMPLE: i32 = 260;
const SAMPLES: i32 = 200;

/// Runs one client and returns (accepted, refused).
fn jog(reports_at: impl Fn(i32, i32) -> bool) -> (u32, u32, Town) {
    let mut town = Town::joined(|town| at(town.street, 0, 0));
    let (mut accepted_count, mut refused) = (0, 0);
    let mut last_accepted = 0;
    for sample in 1..=SAMPLES {
        let body = sample * PER_SAMPLE;
        if !reports_at(sample, last_accepted) {
            continue;
        }
        let (result, _) = town.r#move(town.visitor, at(town.street, body, 0));
        if accepted(&result) {
            accepted_count += 1;
            last_accepted = body;
        } else {
            assert_eq!(result, ActionResult::Rejected(Rejection::TooFarAway));
            refused += 1;
        }
    }
    (accepted_count, refused, town)
}

#[test]
fn a_client_that_follows_the_reporting_rule_at_jog_speed_is_never_refused_and_one_that_ignores_it_is()
 {
    // The rule (`server/PROTOCOL.md` §6.2): report before the body has travelled 2 000 mm since the
    // last accepted position — that is, now, if the next sample would carry it further — and on the
    // final sample.
    let (following_accepted, following_refused, following) =
        jog(|sample, last| sample == SAMPLES || (sample + 1) * PER_SAMPLE - last > 2_000);
    // Once a second: every tenth sample, 2 600 mm at a time.
    let (ignoring_accepted, ignoring_refused, ignoring) = jog(|sample, _| sample % 10 == 0);

    println!("following: {following_accepted} accepted, {following_refused} refused");
    println!("ignoring:  {ignoring_accepted} accepted, {ignoring_refused} refused");
    // 7 samples (1 820 mm) per report → reports at samples 7, 14, …, 196, then the final one at 200.
    assert_eq!((following_accepted, following_refused), (29, 0));
    assert_eq!(
        following.presence(following.visitor),
        Some(at(following.street, 52_000, 0))
    );
    // The first report is 2 600 mm from the start and refused; with no accepted position to advance
    // from, every later one is further still.
    assert_eq!((ignoring_accepted, ignoring_refused), (0, 20));
    assert_eq!(
        ignoring.presence(ignoring.visitor),
        Some(at(ignoring.street, 0, 0))
    );
}

// ---------------------------------------------------------------------------------------------
// Structure: this pack's isolation, checked rather than trusted.
// ---------------------------------------------------------------------------------------------

/// Scans `dir` for any of `words` (case-sensitive), returning (files scanned, offending lines).
fn scan(dir: &std::path::Path, words: &[&str]) -> (usize, Vec<String>) {
    let mut scanned = 0;
    let mut offences = Vec::new();
    for entry in std::fs::read_dir(dir).expect("a readable src/") {
        let path = entry.expect("an entry").path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let text = std::fs::read_to_string(&path).expect("a readable source");
        for (number, line) in text.lines().enumerate() {
            if words.iter().any(|word| line.contains(word)) {
                offences.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    number + 1,
                    line.trim()
                ));
            }
        }
        scanned += 1;
    }
    (scanned, offences)
}

/// No float in this pack, and nothing of this pack in the packs it must not be known by: presence
/// and conversation name none of its identifiers, so removing it cannot break them (`I-4`).
///
/// The words are this pack's *identifiers*. Prose in those crates that says "a movement system" in
/// general — as `ENGINEERING_RULES.md` §6 does — is not a dependency; a type or constant name is.
#[test]
fn this_pack_has_no_float_and_presence_and_conversation_know_nothing_of_it() {
    let systems = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let (own, floats) = scan(&systems.join("movement/src"), &["f32", "f64"]);
    assert!(own >= 6, "every module of this pack is scanned: {own}");
    assert!(floats.is_empty(), "no floating point: {floats:#?}");

    let identifiers = [
        "mineworld_movement",
        "MovementSystem",
        "Passage",
        "passage",
        "MAX_STRIDE",
        "stride",
    ];
    for pack in ["presence", "conversation"] {
        let (scanned, offences) = scan(&systems.join(pack).join("src"), &identifiers);
        println!(
            "{pack}: {scanned} files scanned, {} offence(s)",
            offences.len()
        );
        assert!(scanned >= 7, "{pack}: every module is scanned");
        assert!(offences.is_empty(), "{pack} names movement: {offences:#?}");
    }
    let _ = PresenceSystem::ID;
}
