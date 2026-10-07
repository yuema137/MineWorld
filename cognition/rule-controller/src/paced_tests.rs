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
use mineworld_movement::{MAX_STRIDE, Move, Passage, Passages, move_offer_requirement};
use serde_json::{Value, json};

use crate::PacedRuleController;
use crate::paced::{distance, toward};

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

/// The exact squared floor distance between two positions — what a stride bound is checked against.
fn squared(a: LocalPosition, b: LocalPosition) -> i64 {
    let dx = i64::from(b.x().value()) - i64::from(a.x().value());
    let dy = i64::from(b.y().value()) - i64::from(a.y().value());
    dx * dx + dy * dy
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
    move_offered: bool,
    /// The doorways the place I stand in discloses.
    doors: Vec<Passage>,
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
            move_offered: true,
            doors: vec![Passage::new(
                place(STREET),
                Some(spot(8_000, 1_000)),
                Some(spot(0, 3_000)),
            )],
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
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(self.me)
            .with_components(vec![ComponentRecord::new::<ConversationHistory>(
                id(ME),
                serde_json::to_value(&history).expect("serializes"),
            )]);
        let mut entities = vec![cafe, me];
        let mut affordances = Vec::new();
        if self.move_offered {
            affordances.push(Affordance::available(
                Move::ACTION_TYPE,
                None,
                move_offer_requirement(),
            ));
        }
        for (person, location, may_talk) in &self.people {
            entities.push(PerceivedEntity::new(id(*person), EntityType::Person).at(*location));
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

fn talk(request: &ActionRequest) -> Option<(Option<EntityId>, String)> {
    let payload = request.payload().payload_for::<Talk>().ok()?;
    let talk: Talk = serde_json::from_slice(payload).ok()?;
    Some((request.target(), talk.utterance().as_str().to_owned()))
}

fn moved_to(request: &ActionRequest) -> Option<Location> {
    let payload = request.payload().payload_for::<Move>().ok()?;
    let to: Move = serde_json::from_slice(payload).ok()?;
    Some(to.to())
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
fn it_greets_only_whom_the_server_says_it_may_and_walks_only_when_move_is_offered() {
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
                    assert!(moved_to(&request).is_some());
                    walked += 1;
                }
            }
        }
    }
    assert!(
        greeted > 0 && walked > 0,
        "greeted {greeted}, walked {walked}"
    );

    view.move_offered = false;
    view.people = vec![(BOB, in_cafe(2_000, 1_000), false)];
    for seed in 0..SEEDS {
        assert_eq!(
            controller(seed).decide(&view.build()),
            None,
            "no move offered and nobody to talk to: nothing to do"
        );
    }
}

#[test]
fn every_proposed_walk_is_one_stride_or_a_crossing_at_a_doorway() {
    let view = View::new();
    let from = view.me.local().expect("positioned");
    let stride = i64::from(MAX_STRIDE.value());
    let mut crossings = 0;
    for seed in 0..SEEDS {
        for step in 0..16 {
            let observation = View {
                at: NOW + step * PACE,
                ..View::new()
            }
            .build();
            let Some(to) = controller(seed)
                .decide(&observation)
                .as_ref()
                .and_then(moved_to)
            else {
                continue;
            };
            if to.place() == place(CAFE) {
                let landed = to.local().expect("positioned");
                assert!(
                    squared(from, landed) <= stride * stride,
                    "a {} mm stride from seed {seed}",
                    distance(from, landed)
                );
            } else {
                crossings += 1;
            }
        }
    }
    // From (1000, 1000) the door at (8000, 1000) is seven metres away: no crossing, only strides.
    assert_eq!(crossings, 0);

    // Standing by the door, leaving is a crossing to the street's side of it.
    let by_the_door = View {
        me: in_cafe(7_000, 1_000),
        ..View::new()
    };
    let crossed: Vec<Location> = (0..SEEDS)
        .filter_map(|seed| controller(seed).decide(&by_the_door.build()))
        .filter_map(|request| moved_to(&request))
        .filter(|to| to.place() == place(STREET))
        .collect();
    assert!(!crossed.is_empty(), "some seed walks out");
    assert!(crossed.iter().all(|to| to.local() == Some(spot(0, 3_000))));
}

/// `step-09-social.md` C3 / F-6: on a street that five places open onto, a person heads for a door
/// chosen by the draw — every door is chosen by some seed or window, the same door is kept for a
/// whole `DOOR_WINDOW`, and walking on is what people on a street mostly do.
///
/// Every door is placed within a stride of the observer, so a decision to leave is a crossing and its
/// destination says which door was chosen; no distance is computed by the test.
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
                .and_then(moved_to)
            else {
                continue;
            };
            if to.place() != place(STREET) {
                crossings += 1;
                this_window.insert(to.place().entity_id().raw());
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
        if let Some(to) = controller(seed).decide(&later).as_ref().and_then(moved_to)
            && to.place() != place(STREET)
        {
            chosen.insert(to.place().entity_id().raw());
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

#[test]
fn a_stride_never_exceeds_the_published_bound_in_any_direction() {
    let stride = i64::from(MAX_STRIDE.value());
    let from = spot(0, 0);
    let targets = [
        (1, 0),
        (0, 1),
        (-1, 0),
        (0, -1),
        (2_001, 0),
        (0, -2_001),
        (5_000, 5_000),
        (-7_123, 3_001),
        (1_414, 1_415),
        (2_000_000, 1),
        (3, -4),
        // Located in step-09 C3: the café door from (−6 232, −1 351) on the street. Divided by the
        // floored root (7 600 for 7 600.6) this proposed (1 640, 1 145) — 2 000.16 mm, refused.
        (6_232, 4_351),
    ];
    // And a sweep of directions and lengths, every one checked exactly.
    let swept = (-9_000..=9_000)
        .step_by(997)
        .flat_map(|x| (-9_000..=9_000).step_by(1_009).map(move |y| (x, y)));
    for (x, y) in targets.into_iter().chain(swept) {
        let to = spot(x, y);
        let before = distance(from, to);
        if let Some(stepped) = toward(from, to, 0) {
            // Exact, on squared integers: a rounded length would call 2 000.16 mm "2 000" and pass
            // the very stride the movement system refuses (`ARC-23`).
            assert!(
                squared(from, stepped) <= stride * stride,
                "{} mm toward ({x}, {y})",
                distance(from, stepped)
            );
            assert!(
                distance(stepped, to) < before,
                "the stride toward ({x}, {y}) gets closer"
            );
        }
        // Stopping short never overshoots the stop.
        if let Some(stepped) = toward(from, to, 1_000) {
            assert!(distance(stepped, to) >= 1_000 - 1, "toward ({x}, {y})");
        }
    }
}
