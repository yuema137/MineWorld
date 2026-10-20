//! AC-Na-1 (AC-S35-4): every refusal of `sleep` and `wake`, on both test worlds, records nothing and
//! leaves its entities, components, relations and processes as they were.

mod support;

use mineworld_contracts::{ActionResult, EntityId, Rejection};
use mineworld_sleep::{OUT_OF_BOUND, Sleep, Wake};
use support::{Kind, Town, accepted, record};

/// The world's state as the snapshot holds it — entities, components, relations — and the running
/// `sleep` processes (the consumption `eat.rs` "writes nothing" pattern; the kernel's own dispatch
/// bookkeeping is not world state).
fn state(town: &Town) -> (Vec<u8>, usize) {
    let snapshot = town.world.snapshot().expect("a snapshot");
    (
        serde_json::to_vec(&(
            &snapshot.entities,
            &snapshot.components,
            &snapshot.relations,
        ))
        .expect("encodes"),
        town.sleeping_processes(),
    )
}

/// Submits at the current instant and checks the refusal changed nothing; returns the refusal.
fn refused(
    town: &mut Town,
    actor: EntityId,
    request: mineworld_contracts::ActionRecord,
) -> Rejection {
    let before = state(town);
    let (result, events) = town.submit_now(actor, None, request);
    assert!(events.is_empty(), "a refusal records nothing: {events:?}");
    assert_eq!(state(town), before, "a refusal leaves the world unchanged");
    match result {
        ActionResult::Rejected(reason) => reason,
        other => panic!("expected a refusal, got {other:?}"),
    }
}

fn bound_detail(reason: &Rejection) -> String {
    match reason {
        Rejection::System { code, detail } => {
            assert_eq!(*code, OUT_OF_BOUND);
            detail.clone().expect("the detail names the bounds")
        }
        other => panic!("expected the bound refusal, got {other:?}"),
    }
}

fn every_refusal(kind: Kind, home: &str, elsewhere: &str, longest: u16) {
    let mut town = Town::new(kind);
    let (ana, cy) = (town.person("ana"), town.person("cy"));
    assert_eq!(town.place_of("cy"), Some(town.place(elsewhere)));

    // A place where sleeping is not allowed.
    assert_eq!(
        refused(&mut town, cy, record(&Sleep::new(60))),
        Rejection::PermissionDenied
    );
    // Getting up while awake.
    assert_eq!(
        refused(&mut town, ana, record(&Wake {})),
        Rejection::NoSupportedInteraction
    );
    // `for` out of bounds, at home: the refusal names the bounds in force here.
    let words = format!("15 … {longest} minutes");
    for minutes in [5, 14, longest + 1, 841] {
        let detail = bound_detail(&refused(&mut town, ana, record(&Sleep::new(minutes))));
        assert!(detail.contains(&words), "{detail:?} names {words:?}");
        assert!(detail.contains(&format!("not {minutes}")), "{detail:?}");
    }
    // A target is not something one sleeps on.
    let before = state(&town);
    let (result, events) = town.submit_now(ana, Some(cy), record(&Sleep::new(60)));
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::NoSupportedInteraction)
    );
    assert!(events.is_empty());
    assert_eq!(state(&town), before);
    // A place is not a sleeper.
    let place = town.place(home).entity_id();
    assert_eq!(
        refused(&mut town, place, record(&Sleep::new(60))),
        Rejection::NoSupportedInteraction
    );

    // At the bounds exactly: accepted. Then, asleep, a second sleep is Busy.
    let (result, _) = town.sleep("ana", 15);
    assert!(accepted(&result), "{result:?}");
    assert_eq!(
        refused(&mut town, ana, record(&Sleep::new(60))),
        Rejection::Busy
    );
    let (result, _) = town.wake("ana");
    assert!(accepted(&result), "{result:?}");
    let (result, _) = town.sleep("ana", longest);
    assert!(accepted(&result), "{result:?}");
}

#[test]
fn flat_and_park_refuses_each_request_it_cannot_honour_and_records_nothing() {
    every_refusal(Kind::FlatAndPark, "flat", "street", 840);
}

#[test]
fn hostel_refuses_each_request_it_cannot_honour_with_its_own_bounds() {
    every_refusal(Kind::Hostel, "dorm", "hall", 600);
}
