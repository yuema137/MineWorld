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

fn controller(seed: u64) -> PacedRuleController {
    PacedRuleController::new(seed, SimDuration::from_seconds(PACE))
}

/// One view of the café, built as perception builds it.
struct View {
    at: i64,
    me: Location,
    heard: Vec<Heard>,
    /// Who is here, and whether the server says I may talk to them.
    people: Vec<(u64, Location, bool)>,
    move_offered: bool,
    door: Option<Passage>,
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
            door: Some(Passage::new(
                place(STREET),
                Some(spot(8_000, 1_000)),
                Some(spot(0, 3_000)),
            )),
        }
    }

    fn build(&self) -> Observation<Value> {
        let mut history = ConversationHistory::default();
        for entry in &self.heard {
            history.remember(entry.clone());
        }
        let mut cafe = PerceivedEntity::new(id(CAFE), EntityType::Place);
        if let Some(door) = self.door {
            cafe = cafe.with_components(vec![ComponentRecord::new::<Passages>(
                id(CAFE),
                json!({ "leads_to": [serde_json::to_value(door).expect("serializes")] }),
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
                let length = distance(from, to.local().expect("positioned"));
                assert!(length <= stride, "a {length} mm stride from seed {seed}");
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
    ];
    for (x, y) in targets {
        let to = spot(x, y);
        let before = distance(from, to);
        if let Some(stepped) = toward(from, to, 0) {
            let length = distance(from, stepped);
            assert!(length <= stride, "{length} mm toward ({x}, {y})");
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
