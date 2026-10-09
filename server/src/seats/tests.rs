//! Every transition of `PROTOCOL.md` §4.2 and its refusals, over an injected clock.

use std::cell::RefCell;
use std::rc::Rc;

use mineworld_contracts::ActionRequest;

use super::*;
use crate::hosted::HostedController;
use crate::protocol::WireObservation;

const HOLD: Duration = Duration::from_secs(30);

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal key")
}

fn sub(raw: u64) -> SubscriptionId {
    SubscriptionId::from_raw(raw)
}

fn secret() -> ResumeSecret {
    ResumeSecret::generate().expect("random bytes")
}

fn offered(secret: &ResumeSecret) -> Option<OfferedResume> {
    Some(OfferedResume::new(secret.reveal()))
}

fn at(seconds: i64) -> WorldTime {
    WorldTime::from_seconds(seconds)
}

/// A controller that wants a consult every `every` seconds after its binding, recording each binding
/// instant so a test can see the factory was called — a rebuild, not a resumption.
struct Every {
    every: i64,
}

impl HostedController for Every {
    fn next_consult(&self, after: WorldTime) -> WorldTime {
        at(after.seconds() + self.every)
    }

    fn decide(&mut self, _observation: &WireObservation) -> Option<ActionRequest> {
        None
    }
}

fn factory(every: i64, bound: &Rc<RefCell<Vec<i64>>>) -> HostedFactory {
    let bound = Rc::clone(bound);
    Box::new(move |instant: WorldTime| {
        bound.borrow_mut().push(instant.seconds());
        Box::new(Every { every }) as Box<dyn HostedController>
    })
}

/// `alice` hosted (every 1 s), `bob` hosted (every 3 s), `visitor` free.
fn table(hold: Duration) -> (SeatTable, Rc<RefCell<Vec<i64>>>) {
    let bound = Rc::new(RefCell::new(Vec::new()));
    let mut factories = BTreeMap::new();
    factories.insert(key("alice"), factory(1, &bound));
    factories.insert(key("bob"), factory(3, &bound));
    let table = SeatTable::new(
        [key("alice"), key("bob"), key("visitor")],
        factories,
        hold,
        at(10),
    );
    (table, bound)
}

fn join(seat: &str) -> JoinRequest {
    JoinRequest::plain(key(seat), SessionId::new(1))
}

fn taking(seat: &str) -> JoinRequest {
    JoinRequest {
        take_over: true,
        ..join(seat)
    }
}

fn resuming(seat: &str, secret: &ResumeSecret) -> JoinRequest {
    JoinRequest {
        resume: offered(secret),
        ..join(seat)
    }
}

fn code(result: Result<Grant, Refusal>) -> RefusalCode {
    result.expect_err("a refusal").code()
}

#[test]
fn a_free_seat_is_granted_once_and_then_occupied_until_taken_over() {
    let (mut seats, _) = table(HOLD);
    let now = Instant::now();
    let first = seats
        .join(&join("visitor"), sub(1), secret(), now, at(10))
        .expect("free");
    assert_eq!(first.took_over, TookOver::None);
    assert_eq!(first.displaced, None);

    assert_eq!(
        code(seats.join(&join("visitor"), sub(2), secret(), now, at(10))),
        RefusalCode::SeatOccupied,
        "a plain join on a connected seat"
    );
    let taken = seats
        .join(&taking("visitor"), sub(3), secret(), now, at(10))
        .expect("take_over");
    assert_eq!(taken.took_over, TookOver::Connection);
    assert_eq!(taken.displaced, Some((sub(1), ClosingReason::TakenOver)));
    assert_eq!(
        seats.depart(sub(1), Departure::Dropped, now, at(10)),
        None,
        "the displaced connection's end changes nothing: it no longer holds the seat"
    );
    assert_eq!(
        code(seats.join(&join("visitor"), sub(4), secret(), now, at(10))),
        RefusalCode::SeatOccupied,
        "the seat is the taker's"
    );
}

#[test]
fn a_hosted_seat_yields_without_a_flag_and_its_controller_is_unbound() {
    let (mut seats, bound) = table(HOLD);
    assert_eq!(
        *bound.borrow(),
        vec![10, 10],
        "alice and bob bound at the start"
    );
    assert_eq!(seats.due(at(11)), vec![(at(11), key("alice"))]);

    let grant = seats
        .join(&join("alice"), sub(1), secret(), Instant::now(), at(11))
        .expect("hosted");
    assert_eq!(grant.took_over, TookOver::Hosted);
    assert_eq!(grant.displaced, None, "a controller is not a connection");
    assert!(seats.hosted_mut(&key("alice")).is_none());
    assert_eq!(seats.due(at(100)), vec![(at(13), key("bob"))], "only bob");
}

#[test]
fn leaving_returns_a_seat_to_a_controller_built_afresh() {
    let (mut seats, bound) = table(HOLD);
    let now = Instant::now();
    seats
        .join(&join("alice"), sub(1), secret(), now, at(11))
        .expect("hosted");
    assert_eq!(
        seats.depart(sub(1), Departure::Left, now, at(42)),
        Some(key("alice"))
    );
    assert_eq!(
        bound.borrow().last(),
        Some(&42),
        "rebuilt by its factory, bound at the release"
    );
    assert_eq!(
        seats.due(at(43)),
        vec![(at(13), key("bob")), (at(43), key("alice"))],
        "alice is next due one second after her new binding, not at her old schedule"
    );

    seats
        .join(&join("visitor"), sub(2), secret(), now, at(42))
        .expect("free");
    seats.depart(sub(2), Departure::Left, now, at(42));
    seats
        .join(&join("visitor"), sub(3), secret(), now, at(42))
        .expect("a left seat without a controller is free again at once");
}

#[test]
fn a_dropped_seat_is_held_until_its_hold_ends_and_not_before() {
    let (mut seats, bound) = table(HOLD);
    let now = Instant::now();
    let held = secret();
    seats
        .join(&join("alice"), sub(1), held.clone(), now, at(10))
        .expect("hosted");
    seats.depart(sub(1), Departure::Dropped, now, at(20));
    let builds = bound.borrow().len();

    assert!(
        seats
            .due(at(1_000))
            .iter()
            .all(|(_, seat)| *seat != key("alice")),
        "nobody drives a held Person, not even its controller"
    );
    assert_eq!(
        code(seats.join(&join("alice"), sub(2), secret(), now, at(20))),
        RefusalCode::SeatOccupied,
        "a plain join on a held seat"
    );
    seats.expire(now + HOLD - Duration::from_millis(1), at(49));
    assert_eq!(bound.borrow().len(), builds, "not before the hold ends");
    seats.expire(now + HOLD, at(50));
    assert_eq!(bound.borrow().last(), Some(&50), "at its end, rebuilt");
    assert_eq!(
        code(seats.join(&resuming("alice", &held), sub(3), secret(), now, at(50))),
        RefusalCode::InvalidResume,
        "an expired hold's secret is dead"
    );
}

#[test]
fn a_held_seat_is_resumed_with_its_secret_and_only_with_it() {
    let (mut seats, _) = table(HOLD);
    let now = Instant::now();
    let held = secret();
    seats
        .join(&join("visitor"), sub(1), held.clone(), now, at(10))
        .expect("free");
    seats.depart(sub(1), Departure::Dropped, now, at(10));

    assert_eq!(
        code(seats.join(
            &resuming("visitor", &secret()),
            sub(2),
            secret(),
            now,
            at(11)
        )),
        RefusalCode::InvalidResume
    );
    assert_eq!(
        code(seats.join(&resuming("bob", &held), sub(2), secret(), now, at(11))),
        RefusalCode::InvalidResume,
        "a secret is for its own seat"
    );
    let resumed = seats
        .join(&resuming("visitor", &held), sub(3), secret(), now, at(11))
        .expect("the right secret, inside the hold");
    assert_eq!(resumed.took_over, TookOver::Held);
    assert_eq!(resumed.displaced, None);
    assert_eq!(
        code(seats.join(&resuming("visitor", &held), sub(4), secret(), now, at(11))),
        RefusalCode::InvalidResume,
        "every welcome's secret replaces the last"
    );
}

#[test]
fn a_resume_on_a_live_connection_supersedes_it() {
    let (mut seats, _) = table(HOLD);
    let now = Instant::now();
    let first = secret();
    seats
        .join(&join("visitor"), sub(1), first.clone(), now, at(10))
        .expect("free");
    let grant = seats
        .join(&resuming("visitor", &first), sub(2), secret(), now, at(10))
        .expect("a half-open socket's own secret");
    assert_eq!(grant.took_over, TookOver::Held);
    assert_eq!(grant.displaced, Some((sub(1), ClosingReason::Superseded)));
}

#[test]
fn a_held_seat_taken_over_kills_the_hold_s_secret() {
    let (mut seats, _) = table(HOLD);
    let now = Instant::now();
    let held = secret();
    seats
        .join(&join("visitor"), sub(1), held.clone(), now, at(10))
        .expect("free");
    seats.depart(sub(1), Departure::Dropped, now, at(10));
    let grant = seats
        .join(&taking("visitor"), sub(2), secret(), now, at(11))
        .expect("take_over on a held seat");
    assert_eq!(grant.took_over, TookOver::Connection);
    assert_eq!(grant.displaced, None, "nobody was connected to close");
    assert_eq!(
        code(seats.join(&resuming("visitor", &held), sub(3), secret(), now, at(11))),
        RefusalCode::InvalidResume
    );
}

#[test]
fn with_no_hold_a_dropped_seat_returns_at_once() {
    let (mut seats, bound) = table(Duration::ZERO);
    let now = Instant::now();
    seats
        .join(&join("alice"), sub(1), secret(), now, at(10))
        .expect("hosted");
    seats.depart(sub(1), Departure::Dropped, now, at(12));
    assert_eq!(bound.borrow().last(), Some(&12));
    assert_eq!(
        seats
            .join(&join("alice"), sub(2), secret(), now, at(12))
            .expect("hosted again")
            .took_over,
        TookOver::Hosted
    );
}

#[test]
fn due_seats_come_in_instant_order_then_seat_order() {
    let bound = Rc::new(RefCell::new(Vec::new()));
    let mut factories = BTreeMap::new();
    factories.insert(key("carol"), factory(2, &bound));
    factories.insert(key("bob"), factory(2, &bound));
    factories.insert(key("alice"), factory(5, &bound));
    let seats = SeatTable::new(
        [key("carol"), key("alice"), key("bob")],
        factories,
        HOLD,
        at(0),
    );
    assert_eq!(seats.due(at(1)), Vec::new());
    assert_eq!(
        seats.due(at(9)),
        vec![
            (at(2), key("bob")),
            (at(2), key("carol")),
            (at(5), key("alice"))
        ]
    );
}

#[test]
fn a_seat_outside_the_roster_is_unknown() {
    let (mut seats, _) = table(HOLD);
    assert_eq!(
        code(seats.join(&join("otto"), sub(1), secret(), Instant::now(), at(10))),
        RefusalCode::UnknownSeat
    );
}

fn state(seats: &SeatTable, seat: &str, now: Instant) -> SeatState {
    seats.state_of(&key(seat), now).expect("a roster seat").state
}

/// A kick returns a connected seat to its default with no hold, rebuilding the controller; it finds
/// nothing for a session that holds no seat, or whose seat is held (step-12 SD-D4).
#[test]
fn a_kick_returns_the_seat_to_its_default_with_no_hold() {
    let (mut seats, bound) = table(HOLD);
    let now = Instant::now();
    let session = |n| JoinRequest {
        session: SessionId::new(n),
        ..join("alice")
    };
    let secret = secret();
    seats
        .join(&session(7), sub(1), secret.clone(), now, at(10))
        .expect("hosted yields");
    assert_eq!(
        state(&seats, "alice", now),
        SeatState::Connected {
            session: SessionId::new(7)
        }
    );
    assert_eq!(seats.kick(SessionId::new(8), at(20)), None, "another session");
    assert_eq!(seats.kick(SessionId::new(7), at(20)), Some((sub(1), key("alice"))));
    assert_eq!(state(&seats, "alice", now), SeatState::Hosted, "no hold");
    assert_eq!(bound.borrow().last(), Some(&20), "a controller built afresh at the kick");
    assert_eq!(
        code(seats.join(&resuming("alice", &secret), sub(2), self::secret(), now, at(20))),
        RefusalCode::InvalidResume,
        "the kicked binding's secret died with it"
    );
    assert_eq!(seats.kick(SessionId::new(7), at(20)), None, "kicked once");

    // A held seat is no connection: there is nothing to kick.
    seats
        .join(&session(9), sub(3), self::secret(), now, at(20))
        .expect("hosted yields");
    seats.depart(sub(3), Departure::Dropped, now, at(20));
    assert_eq!(seats.kick(SessionId::new(9), at(20)), None);
}

/// A release on each of the four states (step-12 SD-D4, QS11D-5).
#[test]
fn a_release_returns_connected_and_held_seats_and_leaves_the_others() {
    let (mut seats, bound) = table(HOLD);
    let now = Instant::now();
    let rebuilt = bound.borrow().len();
    assert_eq!(
        seats.release(&key("alice"), at(10)),
        Some(Released {
            released: false,
            displaced: None
        }),
        "hosted"
    );
    assert_eq!(
        seats.release(&key("visitor"), at(10)),
        Some(Released {
            released: false,
            displaced: None
        }),
        "free"
    );
    assert_eq!(bound.borrow().len(), rebuilt, "no controller rebuilt for a no-op");

    seats
        .join(&join("visitor"), sub(1), secret(), now, at(10))
        .expect("free");
    assert_eq!(
        seats.release(&key("visitor"), at(11)),
        Some(Released {
            released: true,
            displaced: Some(sub(1))
        }),
        "connected"
    );
    assert_eq!(state(&seats, "visitor", now), SeatState::Free);

    let held = secret();
    seats
        .join(&join("bob"), sub(2), held.clone(), now, at(11))
        .expect("hosted yields");
    seats.depart(sub(2), Departure::Dropped, now, at(11));
    assert_eq!(
        state(&seats, "bob", now),
        SeatState::Held { seconds_left: 30 }
    );
    assert_eq!(
        seats.release(&key("bob"), at(12)),
        Some(Released {
            released: true,
            displaced: None
        }),
        "held"
    );
    assert_eq!(state(&seats, "bob", now), SeatState::Hosted);
    assert_eq!(
        code(seats.join(&resuming("bob", &held), sub(3), secret(), now, at(12))),
        RefusalCode::InvalidResume,
        "a released hold's secret dies"
    );
    assert_eq!(seats.release(&key("otto"), at(12)), None, "not a seat");
}

/// The report: every seat in key order, a hold's seconds rounded up.
#[test]
fn the_report_names_every_seat_in_key_order() {
    let (mut seats, _) = table(HOLD);
    let now = Instant::now();
    seats
        .join(&join("visitor"), sub(1), secret(), now, at(10))
        .expect("free");
    seats
        .join(&join("bob"), sub(2), secret(), now, at(10))
        .expect("hosted yields");
    seats.depart(sub(2), Departure::Dropped, now, at(10));
    let later = now + Duration::from_millis(9_500);
    let report: Vec<(String, SeatState)> = seats
        .report(later)
        .into_iter()
        .map(|report| (report.seat.to_string(), report.state))
        .collect();
    assert_eq!(
        report,
        vec![
            ("alice".to_owned(), SeatState::Hosted),
            ("bob".to_owned(), SeatState::Held { seconds_left: 21 }),
            (
                "visitor".to_owned(),
                SeatState::Connected {
                    session: SessionId::new(1)
                }
            ),
        ]
    );
    let json = serde_json::to_value(seats.report(later)).expect("serializes");
    assert_eq!(
        json[1],
        serde_json::json!({ "seat": "bob", "state": "held", "seconds_left": 21 })
    );
    assert_eq!(
        json[2],
        serde_json::json!({ "seat": "visitor", "state": "connected", "session": "1" })
    );
}
