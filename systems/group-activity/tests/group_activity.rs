//! Group activity over a hand-built café composed from the real packs (`step-09-social.md` §4.2.3 C2).
//!
//! What each test would pass without proving, and how it is excluded (`ARC-23`): an expiry checked
//! on one side only (both sides of the bound are tested); a wake that "ends" nothing (the facts one
//! second before the end are located as none, and the process is gone after); leaving the place by
//! any route but presence's fact (the member walks through the doorway with `move`).

mod support;

use mineworld_contracts::{
    Action, ActionResult, Affordance, Causation, Event, EventEnvelope, PersonId, Rejection,
};
use mineworld_group_activity::{
    AcceptInvitation, GroupActivityEnded, GroupActivityStarted, INVITATION_LIFETIME, Invite,
    JoinGroupActivity, LeaveGroupActivity, Participation,
};
use mineworld_presence::PersonEnteredPlace;
use support::{CAFE_DOOR, Cafe, STREET_DOOR, at, kind, record, t, types};

fn accepted(result: &ActionResult) -> bool {
    matches!(result, ActionResult::Accepted { .. })
}

fn ended(facts: &[EventEnvelope]) -> GroupActivityEnded {
    let fact = facts
        .iter()
        .find(|fact| *fact.event_type() == GroupActivityEnded::EVENT_TYPE)
        .expect("an ended fact");
    serde_json::from_slice(
        fact.payload()
            .payload_for::<GroupActivityEnded>()
            .expect("its own type"),
    )
    .expect("decodes")
}

fn person(cafe: &Cafe, entity: mineworld_contracts::EntityId) -> PersonId {
    PersonId::new(
        entity,
        cafe.world
            .read()
            .entity(entity)
            .expect("exists")
            .entity_type(),
    )
    .expect("a person")
}

#[test]
fn an_accepted_invitation_starts_an_activity_that_is_a_process_at_the_place() {
    let mut cafe = Cafe::new();
    let (result, facts) = cafe.invite(10, cafe.alice, cafe.bob);
    assert!(accepted(&result), "{result:?}");
    assert_eq!(types(&facts), ["invited"]);
    assert!(
        cafe.invitations(cafe.bob)
            .is_some_and(|held| held.from(person(&cafe, cafe.alice)).is_some())
    );

    let (result, facts) = cafe.accept(20, cafe.bob, cafe.alice);
    assert!(accepted(&result), "{result:?}");
    assert_eq!(
        types(&facts),
        ["invitation-accepted", "group-activity-started"]
    );
    let alices = cafe.participation(cafe.alice).expect("alice is part of it");
    let bobs = cafe.participation(cafe.bob).expect("bob is part of it");
    assert_eq!(alices.activity(), bobs.activity());
    {
        let read = cafe.world.read();
        let process = read.process(alices.activity()).expect("the process runs");
        assert_eq!(process.place(), Some(cafe.cafe));
        assert_eq!(
            process.expected_end(),
            Some(t(20 + 3_600)),
            "one hour from the start"
        );
    }
    assert!(
        cafe.invitations(cafe.bob)
            .is_some_and(|held| held.is_empty()),
        "an answered invitation is gone"
    );
}

#[test]
fn a_declined_invitation_starts_nothing() {
    let mut cafe = Cafe::new();
    cafe.invite(10, cafe.alice, cafe.bob);
    let (result, facts) = cafe.decline(20, cafe.bob, cafe.alice);
    assert!(accepted(&result));
    assert_eq!(types(&facts), ["invitation-declined"]);
    assert!(cafe.participation(cafe.alice).is_none() && cafe.participation(cafe.bob).is_none());
    assert_eq!(cafe.world.read().processes().count(), 0);
}

#[test]
fn an_invitation_can_be_answered_up_to_its_lifetime_and_not_a_second_later() {
    // Both sides of the bound, as literals of the published lifetime (QB-1: 1 800 s).
    assert_eq!(INVITATION_LIFETIME.seconds(), 1_800);
    let mut on_time = Cafe::new();
    on_time.invite(10, on_time.alice, on_time.bob);
    let (result, _) = on_time.accept(10 + 1_800, on_time.bob, on_time.alice);
    assert!(accepted(&result), "at exactly the lifetime: {result:?}");

    let mut late = Cafe::new();
    late.invite(10, late.alice, late.bob);
    let (result, _) = late.accept(10 + 1_801, late.bob, late.alice);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed)
    );
    let (result, _) = late.decline(10 + 1_802, late.bob, late.alice);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed)
    );
}

#[test]
fn every_refusal_is_the_systems_and_named() {
    let mut cafe = Cafe::new();
    // 3 001 mm away: out of invite reach by one millimetre.
    let (result, _) = cafe.invite(10, cafe.alice, cafe.far);
    assert_eq!(result, ActionResult::Rejected(Rejection::TooFarAway));
    // Inviting oneself, or a place.
    let (result, _) = cafe.invite(11, cafe.alice, cafe.alice);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::NoSupportedInteraction)
    );
    let cafe_place = cafe.cafe.entity_id();
    let (result, _) = cafe.invite(12, cafe.alice, cafe_place);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::NoSupportedInteraction)
    );
    // Nothing to accept, nothing to leave.
    let (result, _) = cafe.accept(13, cafe.bob, cafe.alice);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed)
    );
    let (result, _) = cafe.leave(14, cafe.bob);
    assert_eq!(
        result,
        ActionResult::Rejected(Rejection::PreconditionFailed)
    );

    // Alice and Bob together; Carol invites Bob, who is taken.
    cafe.invite(20, cafe.alice, cafe.bob);
    cafe.accept(21, cafe.bob, cafe.alice);
    let (result, _) = cafe.invite(22, cafe.carol, cafe.bob);
    assert_eq!(result, ActionResult::Rejected(Rejection::TargetUnavailable));
    // Joining somebody who is part of nothing.
    let (result, _) = cafe.join(23, cafe.far, cafe.carol);
    assert_eq!(result, ActionResult::Rejected(Rejection::TargetUnavailable));
    // Carol invites the far one, who then joins Alice and Bob: now taken, they cannot accept.
    let (result, _) = cafe.submit(
        t(24),
        cafe.carol,
        Some(cafe.far),
        record(&Invite::new(kind("walk"))),
    );
    assert!(accepted(&result), "{result:?}");
    let (result, _) = cafe.join(25, cafe.far, cafe.bob);
    assert!(accepted(&result), "{result:?}");
    let (result, _) = cafe.accept(26, cafe.far, cafe.carol);
    assert_eq!(result, ActionResult::Rejected(Rejection::Busy));
    // Joining while already a member is Busy too.
    let (result, _) = cafe.join(27, cafe.alice, cafe.bob);
    assert_eq!(result, ActionResult::Rejected(Rejection::Busy));
    // A malformed payload is this pack's own refusal, not a precondition.
    let (result, _) = cafe.submit(
        t(28),
        cafe.carol,
        Some(cafe.bob),
        mineworld_contracts::ActionRecord::new::<Invite>(b"{\"kind\":\"Not A Slug\"}".to_vec()),
    );
    assert!(
        matches!(result, ActionResult::Rejected(Rejection::System { ref code, .. }) if code.as_str() == "malformed-payload"),
        "{result:?}"
    );
}

#[test]
fn a_third_joins_and_the_activity_ends_when_fewer_than_two_remain_naming_all_three() {
    let mut cafe = Cafe::new();
    cafe.invite(10, cafe.alice, cafe.bob);
    cafe.accept(20, cafe.bob, cafe.alice);
    let (result, facts) = cafe.join(30, cafe.carol, cafe.bob);
    assert!(accepted(&result), "{result:?}");
    assert_eq!(types(&facts), ["joined-group-activity"]);
    assert!(cafe.participation(cafe.carol).is_some());

    let (_, facts) = cafe.leave(40, cafe.alice);
    assert_eq!(
        types(&facts),
        ["left-group-activity"],
        "two remain: it goes on"
    );
    assert!(cafe.participation(cafe.alice).is_none());

    let (_, facts) = cafe.leave(50, cafe.bob);
    assert_eq!(
        types(&facts),
        ["left-group-activity", "group-activity-ended"]
    );
    let ended = ended(&facts);
    let everyone: Vec<PersonId> = [cafe.alice, cafe.bob, cafe.carol]
        .into_iter()
        .map(|entity| person(&cafe, entity))
        .collect();
    assert_eq!(
        ended.members(),
        everyone,
        "everyone who took part, in the order they came"
    );
    assert!(
        cafe.participation(cafe.carol).is_none(),
        "the last one is let go too"
    );
    assert_eq!(
        cafe.world.read().processes().count(),
        0,
        "the process is ended"
    );
}

#[test]
fn an_accepted_invitation_from_a_member_joins_the_activity_rather_than_starting_another() {
    let mut cafe = Cafe::new();
    cafe.invite(10, cafe.alice, cafe.bob);
    cafe.accept(20, cafe.bob, cafe.alice);
    cafe.invite(30, cafe.bob, cafe.carol);
    let (_, facts) = cafe.accept(40, cafe.carol, cafe.bob);
    assert_eq!(
        types(&facts),
        ["invitation-accepted", "joined-group-activity"]
    );
    assert_eq!(cafe.world.read().processes().count(), 1);
}

#[test]
fn the_activity_ends_at_its_expected_end_caused_by_its_process() {
    let mut cafe = Cafe::new();
    cafe.invite(10, cafe.alice, cafe.bob);
    cafe.accept(20, cafe.bob, cafe.alice);
    let activity = cafe.participation(cafe.alice).expect("member").activity();

    let before = cafe.world.advance_to(t(20 + 3_599)).expect("advances");
    assert!(
        before.events().is_empty(),
        "nothing one second before the end"
    );
    assert!(cafe.world.read().process(activity).is_some());

    let at_end = cafe.world.advance_to(t(20 + 3_600)).expect("advances");
    assert_eq!(types(at_end.events()), ["group-activity-ended"]);
    assert_eq!(
        *at_end.events()[0].caused_by(),
        Causation::Process(activity)
    );
    assert!(
        cafe.world.read().process(activity).is_none(),
        "ended by its owner"
    );
    assert!(cafe.participation(cafe.alice).is_none() && cafe.participation(cafe.bob).is_none());
}

#[test]
fn a_member_who_walks_into_another_place_leaves_and_the_activity_ends() {
    let mut cafe = Cafe::new();
    cafe.invite(10, cafe.bob, cafe.carol);
    cafe.accept(20, cafe.carol, cafe.bob);
    // Carol walks to the doorway in strides of at most 2 m, then through it.
    let cafe_place = cafe.cafe;
    let street = cafe.street;
    let (result, _) = cafe.walk(30, cafe.carol, at(cafe_place, (4_000, 1_800)));
    assert!(accepted(&result));
    let (result, _) = cafe.walk(31, cafe.carol, at(cafe_place, CAFE_DOOR));
    assert!(accepted(&result));
    let (result, facts) = cafe.walk(32, cafe.carol, at(street, STREET_DOOR));
    assert!(accepted(&result), "{result:?}");
    assert_eq!(
        types(&facts),
        [
            "arrived",
            "person-entered-place",
            "left-group-activity",
            "group-activity-ended"
        ]
    );
    // Each caused by the fact before it: presence's entry is what this pack answered.
    let entered = &facts[1];
    assert_eq!(*entered.event_type(), PersonEnteredPlace::EVENT_TYPE);
    assert_eq!(*facts[2].caused_by(), Causation::Event(entered.id()));
    assert_eq!(*facts[3].caused_by(), Causation::Event(entered.id()));
    assert!(cafe.participation(cafe.carol).is_none() && cafe.participation(cafe.bob).is_none());
}

fn offered<'a>(
    observation: &'a mineworld_contracts::Observation<serde_json::Value>,
    action: &str,
    target: Option<mineworld_contracts::EntityId>,
) -> Option<&'a Affordance> {
    observation.affordances().iter().find(|affordance| {
        affordance.action_type().as_str() == action && affordance.target() == target
    })
}

#[test]
fn offers_and_disclosure_say_what_the_pack_knows_and_no_more() {
    let mut cafe = Cafe::new();
    // Before anything: invite is offered and available to Bob, out of reach to the far one.
    let view = cafe.observation(cafe.alice, 1);
    assert!(
        offered(&view, Invite::ACTION_TYPE.as_str(), Some(cafe.bob))
            .is_some_and(Affordance::is_available)
    );
    assert!(
        offered(&view, Invite::ACTION_TYPE.as_str(), Some(cafe.far))
            .is_some_and(|a| !a.is_available())
    );
    assert!(offered(&view, LeaveGroupActivity::ACTION_TYPE.as_str(), None).is_none());

    cafe.invite(10, cafe.alice, cafe.bob);
    let bobs = cafe.observation(cafe.bob, 11);
    assert!(
        offered(
            &bobs,
            AcceptInvitation::ACTION_TYPE.as_str(),
            Some(cafe.alice)
        )
        .is_some_and(Affordance::is_available)
    );
    let disclosed_to_bob = bobs.entity(cafe.bob).expect("himself").components();
    assert!(
        disclosed_to_bob
            .iter()
            .any(|record| record.component_type().as_str() == "invitations")
    );
    let carols = cafe.observation(cafe.carol, 11);
    assert!(
        carols
            .entity(cafe.bob)
            .expect("bob is perceived")
            .components()
            .iter()
            .all(|record| record.component_type().as_str() != "invitations"),
        "Bob's invitations are Bob's to know"
    );

    cafe.accept(20, cafe.bob, cafe.alice);
    let carols = cafe.observation(cafe.carol, 21);
    let about_bob = carols.entity(cafe.bob).expect("perceived").components();
    let record = about_bob
        .iter()
        .find(|record| record.component_type().as_str() == "participation")
        .expect("that Bob is part of something is visible to anyone who sees him");
    let theirs: Participation = serde_json::from_value(
        record
            .payload_for::<Participation>()
            .expect("labelled")
            .clone(),
    )
    .expect("decodes");
    assert_eq!(theirs.kind().as_str(), "coffee");
    assert!(
        offered(
            &carols,
            JoinGroupActivity::ACTION_TYPE.as_str(),
            Some(cafe.bob)
        )
        .is_some_and(Affordance::is_available)
    );
    assert!(
        offered(&carols, Invite::ACTION_TYPE.as_str(), Some(cafe.bob))
            .is_some_and(|a| !a.is_available())
    );

    let alices = cafe.observation(cafe.alice, 21);
    assert!(
        offered(&alices, LeaveGroupActivity::ACTION_TYPE.as_str(), None)
            .is_some_and(Affordance::is_available)
    );
    assert!(
        offered(
            &alices,
            JoinGroupActivity::ACTION_TYPE.as_str(),
            Some(cafe.bob)
        )
        .is_none(),
        "a member is not offered to join what they are already part of"
    );
}

#[test]
fn the_started_fact_names_both_founders_in_its_payload_and_its_envelope() {
    let mut cafe = Cafe::new();
    cafe.invite(10, cafe.alice, cafe.bob);
    let (_, facts) = cafe.accept(20, cafe.bob, cafe.alice);
    let started = facts
        .iter()
        .find(|fact| *fact.event_type() == GroupActivityStarted::EVENT_TYPE)
        .expect("started");
    let payload: GroupActivityStarted = serde_json::from_slice(
        started
            .payload()
            .payload_for::<GroupActivityStarted>()
            .expect("own"),
    )
    .expect("decodes");
    let founders = vec![person(&cafe, cafe.alice), person(&cafe, cafe.bob)];
    assert_eq!(payload.members(), founders);
    assert_eq!(started.subjects(), [cafe.alice, cafe.bob]);
    assert_eq!(started.participants(), [cafe.alice, cafe.bob]);
    assert_eq!(started.place(), Some(cafe.cafe));
}
