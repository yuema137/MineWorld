//! The per-seat request budget (`PROTOCOL.md` §8, `docs/DECISIONS.md` `ARC-97`), over injected wall
//! instants: the burst, the refill, the exact wait, and the bucket kept by seat across every binding
//! transition (design I-a-4, M-a-3).

use std::collections::BTreeMap;
use std::num::NonZeroU32;
use std::time::{Duration, Instant};

use mineworld_contracts::{EntityKey, WorldTime};

use super::{Departure, JoinRequest, SeatTable};
use crate::admission::ResumeSecret;
use crate::host::{RequestBudget, SubscriptionId};
use crate::protocol::SessionId;

const HOLD: Duration = Duration::from_secs(30);
const AT: WorldTime = WorldTime::EPOCH;
/// One token's worth of time at 10 a second.
const TENTH: Duration = Duration::from_millis(100);

fn key(value: &str) -> EntityKey {
    EntityKey::new(value).expect("a legal key")
}

fn sub(raw: u64) -> SubscriptionId {
    SubscriptionId::from_raw(raw)
}

fn secret() -> ResumeSecret {
    ResumeSecret::generate().expect("random bytes")
}

/// `R = 10`, `B = 20`, the default.
fn budget() -> Option<RequestBudget> {
    Some(RequestBudget::DEFAULT)
}

/// `visitor` and `wanderer`, both free, with this budget and this hold.
fn table(budget: Option<RequestBudget>, hold: Duration) -> SeatTable {
    SeatTable::new([key("visitor"), key("wanderer")], BTreeMap::new(), hold, AT).with_budget(budget)
}

fn join(seats: &mut SeatTable, request: &JoinRequest, subscription: u64, now: Instant) {
    seats
        .join(request, sub(subscription), secret(), now, AT)
        .expect("the join is granted");
}

fn plain(seat: &str) -> JoinRequest {
    JoinRequest::plain(key(seat), SessionId::new(1))
}

/// Spends until the bucket refuses, and returns how many were taken.
fn drain(seats: &mut SeatTable, subscription: u64, now: Instant) -> usize {
    (0..1_000)
        .take_while(|_| seats.spend(sub(subscription), now).is_ok())
        .count()
}

#[test]
fn a_seat_takes_its_burst_then_one_request_per_tenth_of_a_second() {
    let mut seats = table(budget(), HOLD);
    let t0 = Instant::now();
    join(&mut seats, &plain("visitor"), 1, t0);

    assert_eq!(drain(&mut seats, 1, t0), 20, "the burst B");
    assert_eq!(
        seats.spend(sub(1), t0),
        Err(TENTH),
        "the wait is exactly 1/R"
    );
    assert_eq!(
        seats.spend(sub(1), t0 + Duration::from_millis(99)),
        Err(Duration::from_millis(1)),
        "and shrinks as wall time passes"
    );
    assert_eq!(
        seats.spend(sub(1), t0 + TENTH),
        Ok(()),
        "one more after 1/R"
    );
    assert!(seats.spend(sub(1), t0 + TENTH).is_err(), "and only one");
    assert_eq!(
        drain(&mut seats, 1, t0 + Duration::from_secs(60)),
        20,
        "a long rest refills the bucket to B and no further"
    );
}

#[test]
fn every_seat_has_a_bucket_of_its_own() {
    let mut seats = table(budget(), HOLD);
    let t0 = Instant::now();
    join(&mut seats, &plain("visitor"), 1, t0);
    join(&mut seats, &plain("wanderer"), 2, t0);
    assert_eq!(drain(&mut seats, 1, t0), 20);
    assert_eq!(
        drain(&mut seats, 2, t0),
        20,
        "the visitor's flood is not the wanderer's"
    );
}

#[test]
fn no_budget_and_no_seat_are_not_limited() {
    let mut unbounded = table(None, HOLD);
    let t0 = Instant::now();
    join(&mut unbounded, &plain("visitor"), 1, t0);
    assert_eq!(
        drain(&mut unbounded, 1, t0),
        1_000,
        "no budget: never refused"
    );

    let mut bounded = table(budget(), HOLD);
    assert_eq!(
        bounded.spend(sub(9), t0),
        Ok(()),
        "a connection with no seat cannot act, so it spends nothing"
    );
}

#[test]
fn a_smaller_budget_is_its_own_rate() {
    let per_second = NonZeroU32::new(2).expect("non-zero");
    let small = RequestBudget {
        per_second,
        burst: per_second,
    };
    let mut seats = table(Some(small), HOLD);
    let t0 = Instant::now();
    join(&mut seats, &plain("visitor"), 1, t0);
    assert_eq!(drain(&mut seats, 1, t0), 2);
    assert_eq!(seats.spend(sub(1), t0), Err(Duration::from_millis(500)));
}

/// I-a-4: reconnecting, resuming, being taken over, kicked or released, or a hold that ends at once —
/// none refills the seat's bucket. The seat, not the connection, owns it (M-a-3).
#[test]
fn the_budget_is_the_seats_across_a_rejoin() {
    let t0 = Instant::now();
    let mut seats = table(budget(), HOLD);
    join(&mut seats, &plain("visitor"), 1, t0);
    assert_eq!(drain(&mut seats, 1, t0), 20);

    // Leave and join again, on a new connection.
    seats.depart(sub(1), Departure::Left, t0, AT);
    join(&mut seats, &plain("visitor"), 2, t0);
    assert!(
        seats.spend(sub(2), t0).is_err(),
        "a fresh join is not refilled"
    );

    // Drop and resume the hold.
    let held = secret();
    seats.depart(sub(2), Departure::Left, t0, AT);
    seats
        .join(&plain("visitor"), sub(3), held.clone(), t0, AT)
        .expect("free");
    seats.depart(sub(3), Departure::Dropped, t0, AT);
    let resume = JoinRequest {
        resume: Some(crate::admission::OfferedResume::new(held.reveal())),
        ..plain("visitor")
    };
    join(&mut seats, &resume, 4, t0);
    assert!(seats.spend(sub(4), t0).is_err(), "a resume is not refilled");

    // Taken over by another connection.
    let take = JoinRequest {
        take_over: true,
        ..plain("visitor")
    };
    join(&mut seats, &take, 5, t0);
    assert!(
        seats.spend(sub(5), t0).is_err(),
        "a takeover is not refilled"
    );

    // Kicked, then joined again.
    let kicked = JoinRequest {
        session: SessionId::new(7),
        ..plain("visitor")
    };
    seats.depart(sub(5), Departure::Left, t0, AT);
    join(&mut seats, &kicked, 6, t0);
    seats.kick(SessionId::new(7), AT).expect("connected");
    join(&mut seats, &plain("visitor"), 7, t0);
    assert!(seats.spend(sub(7), t0).is_err(), "a kick does not refill");

    // Released, then joined again.
    seats.release(&key("visitor"), AT).expect("on the roster");
    join(&mut seats, &plain("visitor"), 8, t0);
    assert!(
        seats.spend(sub(8), t0).is_err(),
        "a release does not refill"
    );

    // A hold that ends (with no hold, at once), then joined again.
    let mut unheld = table(budget(), Duration::ZERO);
    join(&mut unheld, &plain("visitor"), 1, t0);
    assert_eq!(drain(&mut unheld, 1, t0), 20);
    unheld.depart(sub(1), Departure::Dropped, t0, AT);
    unheld.expire(t0, AT);
    join(&mut unheld, &plain("visitor"), 2, t0);
    assert!(
        unheld.spend(sub(2), t0).is_err(),
        "an ended hold does not refill"
    );

    // Only time does.
    assert_eq!(seats.spend(sub(8), t0 + TENTH), Ok(()));
}

/// AA-6 (I-a-10): the 2D walker's recorded trace (`tests/traces/walker-2d.json`, S27-a C0), replayed at
/// its recorded offsets, is never refused. Replayed through the seat table with injected instants, so
/// the claim is exact and takes no wall time; `tests/request_budget.rs` owns the socket path.
#[test]
fn the_recorded_2d_walk_is_never_refused() {
    let trace: serde_json::Value =
        serde_json::from_str(include_str!("../../tests/traces/walker-2d.json")).expect("the trace");
    let offsets: Vec<u64> = trace["requests"]
        .as_array()
        .expect("a request list")
        .iter()
        .map(|request| request["t_ms"].as_u64().expect("an offset"))
        .collect();
    assert!(
        offsets.len() >= 40,
        "the recorded walk: {} requests",
        offsets.len()
    );
    let mut seats = table(budget(), HOLD);
    let t0 = Instant::now();
    join(&mut seats, &plain("visitor"), 1, t0);
    for t_ms in offsets {
        let now = t0 + Duration::from_millis(t_ms);
        assert_eq!(seats.spend(sub(1), now), Ok(()), "refused at {t_ms} ms");
    }
}
