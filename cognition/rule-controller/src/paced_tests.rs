//! What the paced rule does, decided from observations built by hand (step-08 C3).
//!
//! The draws are seeded, so a claim of the form "it answers a line in its window" is checked across
//! a range of seeds — *some* seed answers, and *no* seed answers outside the window — rather than for
//! one seed that happens to roll well. What a real world produces is the CLI's acceptance tests'.

use mineworld_contracts::{
    Action, ActionRequest, Affordance, ComponentRecord, EntityId, EntityType, LocalPosition,
    Location, Millimetres, Observation, PerceivedEntity, PersonId, PlaceId, Rejection, SimDuration,
    WorldTime,
};
use mineworld_conversation::{ConversationHistory, Heard, Talk, Utterance, talk_requirement};
use mineworld_movement::{
    Destination, Move, Passage, Passages, WalkStep, WalkTo, Walking, move_offer_requirement,
};
use serde_json::{Value, json};

use crate::PacedRuleController;

const CAFE: u64 = 1;
const STREET: u64 = 2;
const ME: u64 = 3;
const BOB: u64 = 4;
const SUE: u64 = 5;
const PACE: i64 = 600;
const NOW: i64 = 36_000;
/// Enough seeds that a 75 % or 20 % behaviour certainly occurs, few enough to stay instant.
const SEEDS: u64 = 64;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn place(raw: u64) -> PlaceId {
    PlaceId::new(id(raw), EntityType::Place).expect("a place")
}

fn spot(x: i32, y: i32) -> LocalPosition {
    LocalPosition::on_ground(Millimetres::new(x), Millimetres::new(y))
}

fn in_cafe(x: i32, y: i32) -> Location {
    Location::in_place(place(CAFE)).with_local(spot(x, y))
}

fn said(speaker: u64, words: &str, at: i64) -> Heard {
    Heard::new(
        PersonId::new(id(speaker), EntityType::Person).expect("a person"),
        WorldTime::from_seconds(at),
        Utterance::new(words).expect("a legal utterance"),
    )
}

fn controller(seed: u64) -> PacedRuleController {
    PacedRuleController::new(seed, SimDuration::from_seconds(PACE))
}

/// One view of the café, built as perception builds it.
#[derive(Clone)]
struct View {
    at: i64,
    me: Location,
    heard: Vec<Heard>,
    /// Who is here, and whether the server says I may talk to them.
    people: Vec<(u64, Location, bool)>,
    /// Whether movement offers me `move` and `walk-to` (it offers both, or neither).
    walk_offered: bool,
    /// The doorways the place I stand in discloses.
    doors: Vec<Passage>,
    /// Whose walk is disclosed: mine, with its destination, and Bob's (movement's `walking` record,
    /// step-11 SD-N8).
    my_walk: Option<Destination>,
    bobs_walk: bool,
    /// Whether the world offers me `walk-step` (only to a walker, in the real world).
    walk_step_offered: bool,
}

impl View {
    fn new() -> Self {
        Self {
            at: NOW,
            me: in_cafe(1_000, 1_000),
            heard: Vec::new(),
            people: vec![
                (BOB, in_cafe(2_000, 1_000), true),
                (SUE, in_cafe(6_000, 1_000), false),
            ],
            walk_offered: true,
            doors: vec![Passage::new(
                place(STREET),
                Some(spot(8_000, 1_000)),
                Some(spot(0, 3_000)),
            )],
            my_walk: None,
            bobs_walk: false,
            walk_step_offered: false,
        }
    }

    fn build(&self) -> Observation<Value> {
        let mut history = ConversationHistory::default();
        for entry in &self.heard {
            history.remember(entry.clone());
        }
        let here = self.me.place().entity_id();
        let mut cafe = PerceivedEntity::new(here, EntityType::Place);
        if !self.doors.is_empty() {
            let leads_to: Vec<Value> = self
                .doors
                .iter()
                .map(|door| serde_json::to_value(door).expect("serializes"))
                .collect();
            cafe = cafe.with_components(vec![ComponentRecord::new::<Passages>(
                here,
                json!({ "leads_to": leads_to }),
            )]);
        }
        let mut mine = vec![ComponentRecord::new::<ConversationHistory>(
            id(ME),
            serde_json::to_value(&history).expect("serializes"),
        )];
        if let Some(destination) = self.my_walk {
            mine.push(walk_record(ME, destination));
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(self.me)
            .with_components(mine);
        let mut entities = vec![cafe, me];
        let mut affordances = Vec::new();
        if self.walk_offered {
            for offered in [Move::ACTION_TYPE, WalkTo::ACTION_TYPE] {
                affordances.push(Affordance::available(
                    offered,
                    None,
                    move_offer_requirement(),
                ));
            }
        }
        if self.walk_step_offered {
            affordances.push(Affordance::available(
                WalkStep::ACTION_TYPE,
                None,
                move_offer_requirement(),
            ));
        }
        for (person, location, may_talk) in &self.people {
            let mut other = PerceivedEntity::new(id(*person), EntityType::Person).at(*location);
            if *person == BOB && self.bobs_walk {
                other = other.with_components(vec![walk_record(
                    BOB,
                    Destination::Place(in_cafe(6_000, 6_000)),
                )]);
            }
            entities.push(other);
            affordances.push(if *may_talk {
                Affordance::available(Talk::ACTION_TYPE, Some(id(*person)), talk_requirement())
            } else {
                Affordance::unavailable(
                    Talk::ACTION_TYPE,
                    Some(id(*person)),
                    talk_requirement(),
                    Rejection::TooFarAway,
                )
            });
        }
        Observation::new(id(ME), WorldTime::from_seconds(self.at))
            .at_location(self.me)
            .perceiving(entities)
            .offering(affordances)
    }
}

/// `who`'s walk as movement discloses it: the destination and the next waypoints (step-11 SD-N8).
/// The shape is written out literally — `{ destination, next }` — not taken from the controller.
fn walk_record(who: u64, destination: Destination) -> ComponentRecord<Value> {
    ComponentRecord::new::<Walking>(
        id(who),
        json!({
            "destination": destination,
            "next": [spot(6_000, 6_000)],
        }),
    )
}

/// Where a `walk-to` request asks to go, if it is one.
fn walked_to(request: &ActionRequest) -> Option<Destination> {
    let payload = request.payload().payload_for::<WalkTo>().ok()?;
    let to: WalkTo = serde_json::from_slice(payload).ok()?;
    Some(to.to())
}

/// The place a `walk-to` enters, when its destination is a place and names no point in it.
fn enters(request: &ActionRequest) -> Option<PlaceId> {
    match walked_to(request)? {
        Destination::Place(location) if location.local().is_none() => Some(location.place()),
        _ => None,
    }
}

fn talk(request: &ActionRequest) -> Option<(Option<EntityId>, String)> {
    let payload = request.payload().payload_for::<Talk>().ok()?;
    let talk: Talk = serde_json::from_slice(payload).ok()?;
    Some((request.target(), talk.utterance().as_str().to_owned()))
}

/// No request the paced controller makes is a `move` any more: every walk is a `walk-to`.
fn is_move(request: &ActionRequest) -> bool {
    *request.action_type() == Move::ACTION_TYPE
}

/// Every seed's reply to Bob, if it is a reply (it quotes him).
fn replies_to_bob(view: &View) -> Vec<String> {
    let observation = view.build();
    (0..SEEDS)
        .filter_map(|seed| controller(seed).decide(&observation))
        .filter_map(|request| talk(&request))
        .filter(|(target, words)| *target == Some(id(BOB)) && words.starts_with("I remember you"))
        .map(|(_, words)| words)
        .collect()
}

#[test]
fn a_line_heard_in_the_window_is_answered_and_quoted() {
    let mut view = View::new();
    view.heard = vec![said(BOB, "the coffee is good today", NOW - 10)];
    let replies = replies_to_bob(&view);
    assert!(
        !replies.is_empty(),
        "some seed answers a line heard ten seconds ago"
    );
    assert!(
        replies
            .iter()
            .all(|words| words.contains("the coffee is good today")),
        "{replies:?}"
    );
}

#[test]
fn a_line_is_answered_in_one_window_only_and_never_again() {
    let mut view = View::new();
    view.heard = vec![said(BOB, "the coffee is good today", NOW)];
    assert!(
        !replies_to_bob(&view).is_empty(),
        "heard at the consult instant: inside"
    );

    // One pace later the same history is still disclosed, and no seed answers it again.
    view.at = NOW + PACE;
    assert!(replies_to_bob(&view).is_empty(), "never twice");

    // The boundary itself: heard exactly one pace before the consult belongs to the previous window.
    view.at = NOW;
    view.heard = vec![said(BOB, "the coffee is good today", NOW - PACE)];
    assert!(replies_to_bob(&view).is_empty(), "(t - pace) is outside");
    view.heard = vec![said(BOB, "the coffee is good today", NOW - PACE + 1)];
    assert!(
        !replies_to_bob(&view).is_empty(),
        "(t - pace + 1) is inside"
    );
}

#[test]
fn a_fresh_controller_decides_exactly_as_the_one_it_replaces() {
    let mut view = View::new();
    view.heard = vec![said(BOB, "hello", NOW - 30)];
    for at in (NOW..NOW + 50 * PACE).step_by(usize::try_from(PACE).expect("positive")) {
        view.at = at;
        let observation = view.build();
        for seed in 0..8 {
            let before = controller(seed).decide(&observation);
            let after = controller(seed).decide(&observation);
            assert_eq!(before, after, "seed {seed} at {at}");
        }
    }
}

#[test]
fn the_seed_is_read_and_nobody_is_idle() {
    let forty_consults = |seed: u64| -> Vec<Option<ActionRequest>> {
        (0..40)
            .map(|step| {
                let view = View {
                    at: NOW + step * PACE,
                    ..View::new()
                };
                controller(seed).decide(&view.build())
            })
            .collect()
    };
    let seven = forty_consults(7);
    assert_ne!(
        seven,
        forty_consults(8),
        "seed 7 and seed 8 differ somewhere"
    );
    // Nobody has spoken to it, and it still acts: initiative, not reaction.
    let acted = seven.iter().filter(|decision| decision.is_some()).count();
    assert!(acted > 10, "acted at {acted} of 40 consults");
}

#[test]
fn it_greets_only_whom_the_server_says_it_may_and_walks_only_when_walk_to_is_offered() {
    let mut view = View::new();
    let mut greeted = 0;
    let mut walked = 0;
    for seed in 0..SEEDS {
        for step in 0..8 {
            view.at = NOW + step * PACE;
            if let Some(request) = controller(seed).decide(&view.build()) {
                if let Some((target, _)) = talk(&request) {
                    assert_eq!(target, Some(id(BOB)), "Sue is too far away to talk to");
                    greeted += 1;
                } else {
                    assert!(walked_to(&request).is_some(), "a walk, asked as walk-to");
                    walked += 1;
                }
            }
        }
    }
    assert!(
        greeted > 0 && walked > 0,
        "greeted {greeted}, walked {walked}"
    );

    view.walk_offered = false;
    view.people = vec![(BOB, in_cafe(2_000, 1_000), false)];
    for seed in 0..SEEDS {
        assert_eq!(
            controller(seed).decide(&view.build()),
            None,
            "no walk-to offered and nobody to talk to: nothing to do"
        );
    }
}

/// Every walk names a destination and nothing else (SD-N16; the route and every stride are
/// movement's, `ARC-75`): the place a doorway leads to, entered; a person in my place; or a point
/// within `WANDER_AXIS` (1 400 mm) of me on each axis. No `move` is ever asked.
#[test]
fn every_proposed_walk_names_a_doorway_s_place_a_person_here_or_a_point_nearby() {
    let view = View::new();
    let from = view.me.local().expect("positioned");
    let (mut doorway, mut person, mut nearby) = (0, 0, 0);
    for seed in 0..SEEDS {
        for step in 0..16 {
            let observation = View {
                at: NOW + step * PACE,
                ..View::new()
            }
            .build();
            let Some(request) = controller(seed).decide(&observation) else {
                continue;
            };
            assert!(!is_move(&request), "seed {seed}: a move was asked");
            let Some(to) = walked_to(&request) else {
                continue;
            };
            match to {
                Destination::Place(location) if location.place() == place(STREET) => {
                    assert_eq!(location.local(), None, "the street is entered, at no point");
                    doorway += 1;
                }
                Destination::Place(location) => {
                    assert_eq!(location.place(), place(CAFE), "seed {seed}");
                    let at = location.local().expect("a wander names a point");
                    let (dx, dy) = (
                        i64::from(at.x().value()) - i64::from(from.x().value()),
                        i64::from(at.y().value()) - i64::from(from.y().value()),
                    );
                    assert!(dx.abs() <= 1_400 && dy.abs() <= 1_400, "({dx}, {dy})");
                    nearby += 1;
                }
                Destination::Person(who) => {
                    assert!(
                        [id(BOB), id(SUE)].contains(&who.entity_id()),
                        "somebody in my place"
                    );
                    person += 1;
                }
                _ => panic!("seed {seed}: an unexpected destination {to:?}"),
            }
        }
    }
    println!("walks: {doorway} through the door, {person} to a person, {nearby} nearby");
    assert!(
        doorway > 0 && person > 0 && nearby > 0,
        "each kind of walk occurs"
    );
}

/// A walk my own disclosed walk already makes is not asked again (step-11 §21.13 NW-C1, N-D14):
/// where a seed would ask to walk into the street, it asks for nothing while my walk already goes
/// there — and it asks no other walk instead, because carrying on is the decision.
#[test]
fn already_walking_there_asks_for_nothing() {
    let street = Destination::Place(Location::in_place(place(STREET)));
    let mut carried_on = 0;
    for seed in 0..SEEDS {
        for step in 0..16 {
            let idle = View {
                at: NOW + step * PACE,
                ..View::new()
            };
            let asked = controller(seed).decide(&idle.build());
            if asked.as_ref().and_then(walked_to) != Some(street) {
                continue;
            }
            let walking = View {
                my_walk: Some(street),
                ..idle
            };
            assert_eq!(
                controller(seed).decide(&walking.build()),
                None,
                "seed {seed} at step {step}: already on the way"
            );
            carried_on += 1;
        }
    }
    assert!(carried_on > 0, "the case occurred: {carried_on}");
}

/// `step-09-social.md` C3 / F-6: on a street that five places open onto, a person heads for a door
/// chosen by the draw — every door is chosen by some seed or window, the same door is kept for a
/// whole `DOOR_WINDOW`, and walking on is what people on a street mostly do.
///
/// A decision to leave is a `walk-to` entering the door's place, so its destination says which door
/// was chosen; no distance is computed by the test.
#[test]
fn on_a_street_of_five_doors_every_door_is_chosen_and_each_is_kept_for_a_window() {
    const DOOR_WINDOW: i64 = 21_600;
    let destinations = [10_u64, 11, 12, 13, 14];
    let at_doors = [
        (1_500, 0),
        (-1_500, 0),
        (0, 1_500),
        (0, -1_500),
        (1_000, 1_000),
    ];
    let street = View {
        me: Location::in_place(place(STREET)).with_local(spot(0, 0)),
        people: Vec::new(),
        doors: destinations
            .iter()
            .zip(at_doors)
            .map(|(to, (x, y))| Passage::new(place(*to), Some(spot(x, y)), Some(spot(500, 500))))
            .collect(),
        ..View::new()
    };
    // Consults inside one window: NOW is 36 000 s, the window 21 600 … 43 199.
    let first_window: Vec<i64> = (0..12).map(|step| NOW + step * PACE).collect();
    assert!(
        first_window
            .iter()
            .all(|at| at.div_euclid(DOOR_WINDOW) == NOW.div_euclid(DOOR_WINDOW))
    );

    let mut chosen = std::collections::BTreeSet::new();
    let (mut consults, mut crossings) = (0, 0);
    for seed in 0..SEEDS {
        let mut this_window = std::collections::BTreeSet::new();
        for at in &first_window {
            consults += 1;
            let observation = View {
                at: *at,
                ..street.clone()
            }
            .build();
            let Some(to) = controller(seed)
                .decide(&observation)
                .as_ref()
                .and_then(enters)
            else {
                continue;
            };
            if to != place(STREET) {
                crossings += 1;
                this_window.insert(to.entity_id().raw());
            }
        }
        assert!(
            this_window.len() <= 1,
            "seed {seed} changed door within one window: {this_window:?}"
        );
        chosen.extend(this_window);
        // And the next window is a draw of its own.
        let later = View {
            at: NOW + DOOR_WINDOW,
            ..street.clone()
        }
        .build();
        if let Some(to) = controller(seed).decide(&later).as_ref().and_then(enters)
            && to != place(STREET)
        {
            chosen.insert(to.entity_id().raw());
        }
    }
    println!("{crossings} crossings in {consults} consults; doors chosen {chosen:?}");
    assert_eq!(
        chosen,
        destinations.into_iter().collect(),
        "every door on the street is chosen by some seed or window"
    );
    assert!(
        crossings * 100 > consults * 40,
        "on a street people mostly walk on: {crossings} of {consults} consults crossed"
    );
}

/// SD-N16: `step` asks for the next stride of *my* walk — `walk-step`, by me, with no payload but an
/// empty one — exactly when my walk is disclosed and the world offers `walk-step`; otherwise nothing.
#[test]
fn step_asks_for_my_stride_only_while_my_walk_is_disclosed_and_offered() {
    let cases = [
        (
            true,
            true,
            false,
            true,
            "my walk disclosed, walk-step offered",
        ),
        (true, false, false, false, "walk-step not offered"),
        (false, true, false, false, "no walk of mine disclosed"),
        (false, true, true, false, "only Bob's walk disclosed"),
    ];
    for (my_walk, offered, bobs_walk, steps, what) in cases {
        let view = View {
            my_walk: my_walk.then_some(Destination::Place(in_cafe(6_000, 6_000))),
            walk_step_offered: offered,
            bobs_walk,
            ..View::new()
        };
        let asked = controller(7).step(&view.build());
        assert_eq!(asked.is_some(), steps, "{what}");
        if let Some(request) = asked {
            assert_eq!(*request.action_type(), WalkStep::ACTION_TYPE, "{what}");
            assert_eq!(request.actor(), id(ME), "{what}: asked by me");
            assert_eq!(request.target(), None, "{what}: against nobody");
            assert_eq!(request.payload().payload(), b"{}", "{what}: no payload");
        }
    }
}

/// SD-N16 / F-N14: `step` never answers, greets or draws. With a line waiting in my window it still
/// asks only for the stride; its answer is the same for every seed and at every instant; and asking
/// it twice on one observation gives the same answer.
#[test]
fn step_never_answers_takes_no_draw_and_is_a_function_of_the_observation() {
    let mut view = View {
        my_walk: Some(Destination::Place(in_cafe(6_000, 6_000))),
        walk_step_offered: true,
        heard: vec![said(BOB, "hello", NOW - 30)],
        ..View::new()
    };
    let mut answers = Vec::new();
    for at in (NOW..NOW + 20 * PACE).step_by(97) {
        view.at = at;
        let observation = view.build();
        for seed in 0..SEEDS {
            let first = controller(seed).step(&observation);
            assert_eq!(
                first,
                controller(seed).step(&observation),
                "seed {seed} at {at}"
            );
            answers.push(first);
        }
    }
    assert!(
        answers.windows(2).all(|pair| pair[0] == pair[1]),
        "one answer for every seed and instant"
    );
    assert!(
        answers[0]
            .as_ref()
            .is_some_and(|request| *request.action_type() == WalkStep::ACTION_TYPE),
        "and it is the stride, never a reply"
    );
}
