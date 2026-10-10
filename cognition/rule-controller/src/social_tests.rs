//! The paced rule's social initiative, decided from observations built by hand (step-09 C5).
//!
//! Seeded draws, so each claim is checked across seeds and windows: *some* seed does it, and *no* seed
//! does what it must not.

use mineworld_contracts::{
    Action, ActionRequest, Affordance, ComponentRecord, EntityId, EntityType, LocalPosition,
    Location, Millimetres, Observation, PerceivedEntity, PersonId, PlaceId, ProcessId, Rejection,
    SimDuration, WorldTime,
};
use mineworld_group_activity::{
    AcceptInvitation, ActivityKind, DeclineInvitation, Invitation, Invitations, Invite,
    JoinGroupActivity, LeaveGroupActivity, Participation, accept_requirement, decline_requirement,
    invite_requirement, join_requirement, leave_requirement,
};
use mineworld_movement::{Move, Passage, Passages, move_offer_requirement};
use serde_json::{Value, json};

use crate::PacedRuleController;

const CAFE: u64 = 1;
const STREET: u64 = 2;
const ME: u64 = 3;
const BOB: u64 = 4;
const SUE: u64 = 5;
const PACE: i64 = 900;
const NOW: i64 = 36_000;
const SEEDS: u64 = 64;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn person(raw: u64) -> PersonId {
    PersonId::new(id(raw), EntityType::Person).expect("a person")
}

fn place(raw: u64) -> PlaceId {
    PlaceId::new(id(raw), EntityType::Place).expect("a place")
}

fn in_cafe(x: i32, y: i32) -> Location {
    Location::in_place(place(CAFE)).with_local(LocalPosition::on_ground(
        Millimetres::new(x),
        Millimetres::new(y),
    ))
}

fn coffee() -> ActivityKind {
    ActivityKind::new("coffee").expect("a kind")
}

/// What the server offers me about one other person, by action.
#[derive(Clone, Default)]
struct Toward {
    invite: Option<bool>,
    accept: Option<bool>,
    decline: Option<bool>,
    join: Option<bool>,
}

/// One view of the café, as perception would build it with group-activity installed.
#[derive(Clone)]
struct View {
    at: i64,
    invited_by: Vec<(u64, i64)>,
    member_of: Option<u64>,
    leave_offered: bool,
    people: Vec<(u64, Toward)>,
    doors: bool,
}

impl View {
    fn new() -> Self {
        Self {
            at: NOW,
            invited_by: Vec::new(),
            member_of: None,
            leave_offered: false,
            people: vec![(BOB, Toward::default()), (SUE, Toward::default())],
            doors: true,
        }
    }

    fn build(&self) -> Observation<Value> {
        let mut mine = Vec::new();
        if !self.invited_by.is_empty() {
            let mut held = Invitations::default();
            for (from, at) in &self.invited_by {
                held.receive(
                    // Open for the compiled 1 800 s, as an unconfigured world states it.
                    Invitation::new(
                        person(*from),
                        coffee(),
                        WorldTime::from_seconds(*at),
                        WorldTime::from_seconds(*at + 1_800),
                    ),
                    WorldTime::from_seconds(*at),
                );
            }
            mine.push(ComponentRecord::new::<Invitations>(
                id(ME),
                serde_json::to_value(&held).expect("encodes"),
            ));
        }
        if let Some(activity) = self.member_of {
            let participation = Participation::new(
                ProcessId::from_raw(activity),
                coffee(),
                WorldTime::from_seconds(NOW - 100),
            );
            mine.push(ComponentRecord::new::<Participation>(
                id(ME),
                serde_json::to_value(&participation).expect("encodes"),
            ));
        }
        let mut cafe = PerceivedEntity::new(id(CAFE), EntityType::Place);
        if self.doors {
            let door = Passage::new(
                place(STREET),
                Some(LocalPosition::on_ground(
                    Millimetres::new(1_200),
                    Millimetres::new(1_000),
                )),
                Some(LocalPosition::on_ground(
                    Millimetres::new(0),
                    Millimetres::new(3_000),
                )),
            );
            cafe = cafe.with_components(vec![ComponentRecord::new::<Passages>(
                id(CAFE),
                json!({ "leads_to": [serde_json::to_value(door).expect("encodes")] }),
            )]);
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(in_cafe(1_000, 1_000))
            .with_components(mine);
        let mut entities = vec![cafe, me];
        let mut affordances = vec![Affordance::available(
            Move::ACTION_TYPE,
            None,
            move_offer_requirement(),
        )];
        if self.leave_offered {
            affordances.push(Affordance::available(
                LeaveGroupActivity::ACTION_TYPE,
                None,
                leave_requirement(),
            ));
        }
        for (other, toward) in &self.people {
            entities.push(
                PerceivedEntity::new(id(*other), EntityType::Person).at(in_cafe(1_500, 1_000)),
            );
            let mut offer = |action, requirement, available: Option<bool>| {
                if let Some(available) = available {
                    affordances.push(if available {
                        Affordance::available(action, Some(id(*other)), requirement)
                    } else {
                        Affordance::unavailable(
                            action,
                            Some(id(*other)),
                            requirement,
                            Rejection::TargetUnavailable,
                        )
                    });
                }
            };
            offer(Invite::ACTION_TYPE, invite_requirement(), toward.invite);
            offer(
                AcceptInvitation::ACTION_TYPE,
                accept_requirement(),
                toward.accept,
            );
            offer(
                DeclineInvitation::ACTION_TYPE,
                decline_requirement(),
                toward.decline,
            );
            offer(
                JoinGroupActivity::ACTION_TYPE,
                join_requirement(),
                toward.join,
            );
        }
        Observation::new(id(ME), WorldTime::from_seconds(self.at))
            .at_location(in_cafe(1_000, 1_000))
            .perceiving(entities)
            .offering(affordances)
    }
}

fn controller(seed: u64) -> PacedRuleController {
    PacedRuleController::new(seed, SimDuration::from_seconds(PACE))
}

/// Every decision across seeds and consult instants.
fn decisions(view: &View) -> Vec<ActionRequest> {
    let mut all = Vec::new();
    for seed in 0..SEEDS {
        for step in 0..8 {
            let at = view.at + step * PACE;
            let observation = View { at, ..view.clone() }.build();
            all.extend(controller(seed).decide(&observation));
        }
    }
    all
}

fn of<A: Action>(requests: &[ActionRequest]) -> Vec<&ActionRequest> {
    requests
        .iter()
        .filter(|request| *request.payload().action_type() == A::ACTION_TYPE)
        .collect()
}

#[test]
fn an_open_invitation_the_server_lets_me_accept_is_answered_mostly_yes() {
    let mut view = View::new();
    view.invited_by = vec![(BOB, NOW - 60)];
    view.people[0].1 = Toward {
        invite: Some(false),
        accept: Some(true),
        decline: Some(true),
        join: None,
    };
    let (mut accepted, mut declined) = (0, 0);
    for seed in 0..SEEDS {
        let decided = controller(seed)
            .decide(&view.build())
            .expect("an open invitation is always answered");
        assert_eq!(decided.target(), Some(id(BOB)), "answered to the inviter");
        let kind = decided.payload().action_type().clone();
        if kind == AcceptInvitation::ACTION_TYPE {
            accepted += 1;
        } else if kind == DeclineInvitation::ACTION_TYPE {
            declined += 1;
        } else {
            panic!("an invitation comes first, before anything else: {kind}");
        }
    }
    println!("accepted {accepted}, declined {declined} of {SEEDS}");
    assert!(
        declined > 0 && accepted > declined,
        "both answers, yes the likelier"
    );
}

#[test]
fn an_invitation_is_not_answered_when_the_server_says_no_or_when_it_has_expired() {
    let answers = |view: &View| {
        decisions(view)
            .into_iter()
            .filter(|request| {
                let kind = request.payload().action_type();
                *kind == AcceptInvitation::ACTION_TYPE || *kind == DeclineInvitation::ACTION_TYPE
            })
            .count()
    };
    let mut refused = View::new();
    refused.invited_by = vec![(BOB, NOW - 60)];
    refused.people[0].1.accept = Some(false);
    refused.people[0].1.decline = Some(true);
    assert_eq!(
        answers(&refused),
        0,
        "accept priced unavailable: not answered"
    );

    // Older than the lifetime (1 800 s) at every consult of the sweep: never answered. The same
    // invitation one second younger is answered — both sides of the bound.
    let mut stale = View::new();
    stale.people[0].1.accept = Some(true);
    stale.people[0].1.decline = Some(true);
    stale.invited_by = vec![(BOB, NOW - 1_801)];
    let stale_once = stale.build();
    assert!(
        (0..SEEDS).all(|seed| {
            controller(seed).decide(&stale_once).is_none_or(|request| {
                *request.payload().action_type() != AcceptInvitation::ACTION_TYPE
                    && *request.payload().action_type() != DeclineInvitation::ACTION_TYPE
            })
        }),
        "1 801 s old: expired"
    );
    stale.invited_by = vec![(BOB, NOW - 1_800)];
    let fresh_once = stale.build();
    assert!(
        (0..SEEDS).all(|seed| controller(seed)
            .decide(&fresh_once)
            .is_some_and(|request| { request.target() == Some(id(BOB)) })),
        "1 800 s old: still open"
    );
}

#[test]
fn it_invites_only_whom_the_server_says_it_may_and_joins_only_through_an_offer() {
    let mut view = View::new();
    view.people[0].1.invite = Some(true); // Bob: available
    view.people[1].1.invite = Some(false); // Sue: priced unavailable
    let decided = decisions(&view);
    let invites = of::<Invite>(&decided);
    assert!(!invites.is_empty(), "it does invite");
    assert!(
        invites
            .iter()
            .all(|request| request.target() == Some(id(BOB))),
        "never the one the server priced unavailable"
    );
    assert!(
        of::<JoinGroupActivity>(&decided).is_empty(),
        "nothing offered to join"
    );

    let mut joinable = View::new();
    joinable.people[1].1.join = Some(true);
    let joins = decisions(&joinable);
    let joins = of::<JoinGroupActivity>(&joins);
    assert!(!joins.is_empty(), "it does join");
    assert!(
        joins
            .iter()
            .all(|request| request.target() == Some(id(SUE)))
    );
}

#[test]
fn part_of_an_activity_it_sometimes_leaves_and_never_crosses_a_doorway() {
    let doorway_crossings = |requests: &[ActionRequest]| {
        of::<Move>(requests)
            .iter()
            .filter(|request| {
                let to: Move = serde_json::from_slice(request.payload().payload()).expect("a move");
                to.to().place() != place(CAFE)
            })
            .count()
    };
    let mut member = View::new();
    member.member_of = Some(9);
    member.leave_offered = true;
    member.people[0].1.invite = Some(true);
    let decided = decisions(&member);
    assert!(
        !of::<LeaveGroupActivity>(&decided).is_empty(),
        "it does leave, sometimes"
    );
    assert!(
        of::<Invite>(&decided).is_empty(),
        "a member does not start inviting"
    );
    assert_eq!(
        doorway_crossings(&decided),
        0,
        "a member stays in the place"
    );

    // The instrument can see a crossing: the same view, not a member, does cross (ARC-23).
    let outsider = View {
        member_of: None,
        leave_offered: false,
        ..member
    };
    assert!(
        doorway_crossings(&decisions(&outsider)) > 0,
        "a non-member walks out of the door"
    );
}

#[test]
fn a_fresh_controller_decides_exactly_as_the_one_it_replaces_on_social_views() {
    let mut view = View::new();
    view.invited_by = vec![(BOB, NOW - 60)];
    view.people[0].1 = Toward {
        invite: Some(false),
        accept: Some(true),
        decline: Some(true),
        join: Some(true),
    };
    view.people[1].1.invite = Some(true);
    for step in 0..20 {
        let observation = View {
            at: NOW + step * PACE,
            ..view.clone()
        }
        .build();
        for seed in 0..8 {
            assert_eq!(
                controller(seed).decide(&observation),
                controller(seed).decide(&observation),
                "seed {seed} step {step}"
            );
        }
    }
}

#[test]
fn with_nothing_social_in_view_no_social_action_is_ever_proposed() {
    // No group-activity affordance and nothing disclosed: the social module contributes nothing, so
    // the walking-and-talking scheme decides alone — exactly as before C5. (The same holds on a real
    // world without group-activity, byte for byte: step-09 §9 E-B5's parity run.)
    let decided = decisions(&View::new());
    for request in &decided {
        let kind = request.payload().action_type();
        assert!(
            *kind == Move::ACTION_TYPE || kind.as_str() == "talk",
            "only the old repertoire: {kind}"
        );
    }
    assert!(!decided.is_empty());
}
