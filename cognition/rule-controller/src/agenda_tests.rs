//! The paced rule following its day, decided from observations built by hand (`step-09-social.md`
//! C7). Across 64 seeds, as the other paced tests do: *some* seed does a thing, *no* seed does what
//! must never happen.

use mineworld_contracts::{
    Action, ActionRequest, Affordance, ComponentRecord, EntityId, EntityType, LocalPosition,
    Location, Millimetres, Observation, PerceivedEntity, PersonId, PlaceId, ProcessId, SimDuration,
    WorldTime,
};
use mineworld_conversation::{ConversationHistory, Heard, Talk, Utterance, talk_requirement};
use mineworld_movement::{Move, Passage, Passages, move_offer_requirement};
use mineworld_schedule::{Agenda, AgendaLabel};
use serde_json::{Value, json};

use crate::PacedRuleController;

const CAFE: u64 = 1;
const STREET: u64 = 2;
const ME: u64 = 3;
const BOB: u64 = 4;
const PARK: u64 = 12;
const PACE: i64 = 900;
const NOW: i64 = 36_000;
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

fn controller(seed: u64) -> PacedRuleController {
    PacedRuleController::new(seed, SimDuration::from_seconds(PACE))
}

/// A view of one place: me at `me`, Bob beside me, the place's doorways, and — if given — my agenda.
#[derive(Clone)]
struct Day {
    at: i64,
    me: Location,
    doors: Vec<Passage>,
    agenda: Option<PlaceId>,
    move_offered: bool,
    heard: Vec<Heard>,
}

impl Day {
    /// In the café, whose one door onto the street is seven metres east.
    fn in_cafe() -> Self {
        Self {
            at: NOW,
            me: Location::in_place(place(CAFE)).with_local(spot(1_000, 1_000)),
            doors: vec![Passage::new(
                place(STREET),
                Some(spot(8_000, 1_000)),
                Some(spot(0, 3_000)),
            )],
            agenda: None,
            move_offered: true,
            heard: Vec::new(),
        }
    }

    /// On the street, five doors each within a stride, so a heading is a crossing whose destination
    /// says which door was taken.
    fn on_street() -> Self {
        let at_doors = [
            (1_500, 0),
            (-1_500, 0),
            (0, 1_500),
            (0, -1_500),
            (1_000, 1_000),
        ];
        Self {
            me: Location::in_place(place(STREET)).with_local(spot(0, 0)),
            doors: [10_u64, 11, PARK, 13, 14]
                .iter()
                .zip(at_doors)
                .map(|(to, (x, y))| {
                    Passage::new(place(*to), Some(spot(x, y)), Some(spot(500, 500)))
                })
                .collect(),
            ..Self::in_cafe()
        }
    }

    fn build(&self) -> Observation<Value> {
        let here = self.me.place().entity_id();
        let leads_to: Vec<Value> = self
            .doors
            .iter()
            .map(|door| serde_json::to_value(door).expect("serializes"))
            .collect();
        let the_place = PerceivedEntity::new(here, EntityType::Place).with_components(vec![
            ComponentRecord::new::<Passages>(here, json!({ "leads_to": leads_to })),
        ]);
        let mut history = ConversationHistory::default();
        for entry in &self.heard {
            history.remember(entry.clone());
        }
        let mut mine = vec![ComponentRecord::new::<ConversationHistory>(
            id(ME),
            serde_json::to_value(&history).expect("serializes"),
        )];
        if let Some(agenda) = self.agenda {
            let agenda = Agenda::new(
                agenda,
                AgendaLabel::new("work").expect("a label"),
                WorldTime::from_seconds(NOW - 600),
                WorldTime::from_seconds(NOW + 7 * 3_600),
                ProcessId::from_raw(1),
            );
            mine.push(ComponentRecord::new::<Agenda>(
                id(ME),
                serde_json::to_value(&agenda).expect("serializes"),
            ));
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(self.me)
            .with_components(mine);
        let bob_at = self.me.with_local(spot(
            self.me.local().map_or(0, |at| at.x().value()) + 1_000,
            0,
        ));
        let bob = PerceivedEntity::new(id(BOB), EntityType::Person).at(bob_at);
        let mut affordances = vec![Affordance::available(
            Talk::ACTION_TYPE,
            Some(id(BOB)),
            talk_requirement(),
        )];
        if self.move_offered {
            affordances.push(Affordance::available(
                Move::ACTION_TYPE,
                None,
                move_offer_requirement(),
            ));
        }
        Observation::new(id(ME), WorldTime::from_seconds(self.at))
            .at_location(self.me)
            .perceiving(vec![the_place, me, bob])
            .offering(affordances)
    }
}

fn moved_to(request: &ActionRequest) -> Option<Location> {
    let payload = request.payload().payload_for::<Move>().ok()?;
    let to: Move = serde_json::from_slice(payload).ok()?;
    Some(to.to())
}

/// Whether the request answers a line — a reply, not a greeting, which is also a `talk`.
fn replied(request: &ActionRequest) -> bool {
    request
        .payload()
        .payload_for::<Talk>()
        .ok()
        .and_then(|payload| serde_json::from_slice::<Talk>(payload).ok())
        .is_some_and(|talk| talk.utterance().as_str().starts_with("I remember you"))
}

/// Every seed's decision on `day`, over `windows` consults a pace apart.
fn decisions(day: &Day, windows: i64) -> Vec<Option<ActionRequest>> {
    (0..SEEDS)
        .flat_map(|seed| {
            (0..windows).map(move |step| {
                let at = Day {
                    at: day.at + step * PACE,
                    ..day.clone()
                };
                controller(seed).decide(&at.build())
            })
        })
        .collect()
}

#[test]
fn away_from_the_agenda_in_a_place_with_one_door_most_seeds_head_for_that_door() {
    let away = Day {
        agenda: Some(place(PARK)),
        ..Day::in_cafe()
    };
    let door = spot(8_000, 1_000);
    let from = away.me.local().expect("positioned");
    let gap = |at: LocalPosition| {
        let (dx, dy) = (
            i64::from(at.x().value()) - i64::from(door.x().value()),
            i64::from(at.y().value()) - i64::from(door.y().value()),
        );
        dx * dx + dy * dy
    };
    let all = decisions(&away, 1);
    let toward: usize = all
        .iter()
        .flatten()
        .filter_map(moved_to)
        .filter(|to| to.place() == place(CAFE) && to.local().is_some_and(|at| gap(at) < gap(from)))
        .count();
    println!("{toward} of {SEEDS} seeds stride toward the door");
    assert!(
        toward * 100 >= all.len() * 75,
        "the agenda band (90 of 100) shows: {toward} of {}",
        all.len()
    );
}

#[test]
fn on_the_street_the_door_taken_is_the_one_that_leads_to_the_agenda_and_never_another() {
    let away = Day {
        agenda: Some(place(PARK)),
        ..Day::on_street()
    };
    let crossings: Vec<u64> = decisions(&away, 12)
        .iter()
        .flatten()
        .filter_map(moved_to)
        .filter(|to| to.place() != place(STREET))
        .map(|to| to.place().entity_id().raw())
        .collect();
    println!(
        "{} crossings, all to {:?}",
        crossings.len(),
        crossings.iter().collect::<std::collections::BTreeSet<_>>()
    );
    assert!(
        !crossings.is_empty(),
        "located: people cross toward their agenda"
    );
    assert!(
        crossings.iter().all(|to| *to == PARK),
        "every crossing is into the park, the agenda's place: {crossings:?}"
    );
}

#[test]
fn at_the_agenda_nobody_walks_out_even_standing_at_the_door() {
    // By the door, without an agenda, some seed walks out (paced_tests); with the agenda here, none.
    let by_the_door = Day {
        me: Location::in_place(place(CAFE)).with_local(spot(7_000, 1_000)),
        ..Day::in_cafe()
    };
    let out = |day: &Day| {
        decisions(day, 16)
            .iter()
            .flatten()
            .filter_map(moved_to)
            .filter(|to| to.place() != place(CAFE))
            .count()
    };
    assert!(
        out(&by_the_door) > 0,
        "without an agenda, people do walk out"
    );
    let at_work = Day {
        agenda: Some(place(CAFE)),
        ..by_the_door
    };
    assert_eq!(out(&at_work), 0, "at the agenda's place, nobody leaves it");
    let acted = decisions(&at_work, 16).iter().flatten().count();
    assert!(acted > 0, "and they still act there: {acted}");
}

#[test]
fn with_no_move_offered_an_agenda_proposes_no_walk() {
    let stuck = Day {
        agenda: Some(place(PARK)),
        move_offered: false,
        ..Day::in_cafe()
    };
    assert!(
        decisions(&stuck, 8)
            .iter()
            .flatten()
            .all(|request| moved_to(request).is_none()),
        "the server's verdict is the whole answer (ENGINEERING_RULES §8)"
    );
}

#[test]
fn being_addressed_still_comes_before_the_agenda() {
    let line = Heard::new(
        PersonId::new(id(BOB), EntityType::Person).expect("a person"),
        WorldTime::from_seconds(NOW - 10),
        Utterance::new("off to the park?").expect("an utterance"),
    );
    let addressed = |agenda| Day {
        agenda,
        heard: vec![line.clone()],
        ..Day::in_cafe()
    };
    let replies = |day: &Day| {
        (0..SEEDS)
            .map(|seed| controller(seed).decide(&day.build()))
            .map(|request| request.as_ref().is_some_and(replied))
            .collect::<Vec<bool>>()
    };
    let without = replies(&addressed(None));
    let with = replies(&addressed(Some(place(PARK))));
    assert!(
        without.iter().any(|replied| *replied),
        "located: some seed replies"
    );
    assert_eq!(
        with, without,
        "every seed that answers the line without an agenda answers it with one"
    );
}

#[test]
fn a_controller_rebuilt_decides_identically() {
    let away = Day {
        agenda: Some(place(PARK)),
        ..Day::on_street()
    };
    for seed in 0..SEEDS {
        let observation = away.build();
        assert_eq!(
            controller(seed).decide(&observation),
            controller(seed).decide(&observation),
            "a pure function of seed, pace and observation (ARC-27)"
        );
    }
}
