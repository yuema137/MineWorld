//! The commute: its arithmetic on a literal table, the due test's boundary instants, and the rule's
//! precedence, decided from observations built by hand (`pr-le-leave-early.md` SD-LE-5, §0.4, C4).

use mineworld_contracts::{
    Action, ActionRequest, Affordance, ComponentRecord, EntityId, EntityType, Location,
    Millimetres, Observation, PerceivedEntity, PersonId, PlaceId, ProcessId, SimDuration,
    WorldTime,
};
use mineworld_conversation::{ConversationHistory, Heard, Talk, Utterance, talk_requirement};
use mineworld_movement::{Destination, WalkTo, Walking, Way, Ways, move_offer_requirement};
use mineworld_schedule::{Agenda, AgendaLabel, DisclosedAgenda, NextPart};
use serde_json::{Value, json};

use crate::PacedRuleController;
use crate::commute::{Commute, Mode, Precedence, estimate, holds, strides};

#[test]
fn arrived_early_i_wait_but_hours_before_the_part_in_force_has_its_day() {
    let commute = Commute::default();
    let there = |at| View {
        here: CAFE,
        ..View::at_home(at)
    };
    assert!(holds(&there(FROM - 600).build(), &commute, seconds(PACE)));
    assert!(
        holds(
            &there(FROM - 7_200 - PACE + 1).build(),
            &commute,
            seconds(PACE)
        ),
        "within the cap and a pace"
    );
    assert!(
        !holds(&there(FROM - 7_200 - PACE).build(), &commute, seconds(PACE)),
        "earlier, the part in force calls me home"
    );
    let at_boundary = Commute {
        mode: Mode::AtBoundary,
        ..commute
    };
    assert!(!holds(
        &there(FROM - 600).build(),
        &at_boundary,
        seconds(PACE)
    ));
    // Waiting at the café, nobody walks home to the agenda in force.
    for seed in 0..SEEDS {
        let decided = controller(seed, commute).decide(&there(FROM - 600).build());
        assert!(
            decided
                .as_ref()
                .and_then(walks_into)
                .is_none_or(|to| to == place(CAFE)),
            "seed {seed}: {decided:?}"
        );
    }
}

const CAFE: u64 = 1;
const HOME: u64 = 20;
const ME: u64 = 3;
const BOB: u64 = 4;
const PACE: i64 = 900;
const STRIDE: i64 = 30;
/// When the punctual part begins: 05:30 on day 2.
const FROM: i64 = 86_400 + 5 * 3_600 + 1_800;
/// 13.4 m and one doorway: 10 + 1 + 1 = 12 strides, 360 s at 30 s; lead 360 × 1.5 + 300 = 840 s.
const LENGTH: i32 = 13_400;
const LEAD: i64 = 840;
const SEEDS: u64 = 64;

fn id(raw: u64) -> EntityId {
    EntityId::from_raw(raw)
}

fn place(raw: u64) -> PlaceId {
    PlaceId::new(id(raw), EntityType::Place).expect("a place")
}

fn seconds(value: i64) -> SimDuration {
    SimDuration::from_seconds(value)
}

/// My view: where I am, the next part of my day, my ways, my walk and any line heard.
#[derive(Clone)]
struct View {
    at: i64,
    here: u64,
    punctual: bool,
    ways: bool,
    walking_to: Option<u64>,
    heard: Vec<Heard>,
}

impl View {
    fn at_home(at: i64) -> Self {
        Self {
            at,
            here: HOME,
            punctual: true,
            ways: true,
            walking_to: None,
            heard: Vec::new(),
        }
    }

    fn build(&self) -> Observation<Value> {
        let here = Location::in_place(place(self.here));
        let agenda = Agenda::new(
            place(HOME),
            AgendaLabel::new("home").expect("a label"),
            WorldTime::from_seconds(FROM - 12 * 3_600),
            WorldTime::from_seconds(FROM),
            ProcessId::from_raw(1),
        );
        let disclosed = DisclosedAgenda {
            agenda,
            next: Some(NextPart {
                place: place(CAFE),
                label: AgendaLabel::new("work").expect("a label"),
                from: WorldTime::from_seconds(FROM),
                punctual: self.punctual,
            }),
        };
        let mut history = ConversationHistory::default();
        for entry in &self.heard {
            history.remember(entry.clone());
        }
        let mut mine = vec![
            ComponentRecord::new::<ConversationHistory>(
                id(ME),
                serde_json::to_value(&history).expect("serializes"),
            ),
            ComponentRecord::new::<Agenda>(
                id(ME),
                serde_json::to_value(&disclosed).expect("serializes"),
            ),
        ];
        if self.ways {
            let ways = Ways(vec![Way {
                place: place(CAFE),
                doors: 1,
                length: Millimetres::new(LENGTH),
            }]);
            mine.push(ComponentRecord::new::<Ways>(
                id(ME),
                serde_json::to_value(&ways).expect("serializes"),
            ));
        }
        if let Some(to) = self.walking_to {
            let destination = Destination::Place(Location::in_place(place(to)));
            mine.push(ComponentRecord::new::<Walking>(
                id(ME),
                json!({ "destination": destination, "next": [] }),
            ));
        }
        let me = PerceivedEntity::new(id(ME), EntityType::Person)
            .at(here)
            .with_components(mine);
        let bob = PerceivedEntity::new(id(BOB), EntityType::Person).at(here);
        let affordances = vec![
            Affordance::available(Talk::ACTION_TYPE, Some(id(BOB)), talk_requirement()),
            Affordance::available(WalkTo::ACTION_TYPE, None, move_offer_requirement()),
        ];
        Observation::new(id(ME), WorldTime::from_seconds(self.at))
            .at_location(here)
            .perceiving(vec![me, bob])
            .offering(affordances)
    }
}

fn controller(seed: u64, commute: Commute) -> PacedRuleController {
    PacedRuleController::new(seed, seconds(PACE)).commuting(commute, seconds(STRIDE))
}

fn due(view: &View, commute: Commute) -> Option<PlaceId> {
    commute.due(&view.build(), seconds(PACE), seconds(STRIDE))
}

fn walks_into(request: &ActionRequest) -> Option<PlaceId> {
    let to: WalkTo =
        serde_json::from_slice(request.payload().payload_for::<WalkTo>().ok()?).ok()?;
    match to.to() {
        Destination::Place(location) => Some(location.place()),
        _ => None,
    }
}

fn line(at: i64) -> Heard {
    Heard::new(
        PersonId::new(id(BOB), EntityType::Person).expect("a person"),
        WorldTime::from_seconds(at),
        Utterance::new("off to work?").expect("an utterance"),
    )
}

/// The literal table the Python estimate is checked against too (LE-c): (length mm, doors, stride s)
/// → (strides, estimate s, lead s at the default policy).
const TABLE: [(i32, u8, i64, i64, i64, i64); 6] = [
    (0, 0, 30, 1, 30, 345),
    (1_340, 1, 30, 3, 90, 435),
    (1_341, 1, 30, 4, 120, 480),
    (13_400, 1, 30, 12, 360, 840),
    (13_400, 1, 12, 12, 144, 516),
    (1_000_000, 4, 30, 752, 22_560, 7_200),
];

#[test]
fn strides_estimate_and_lead_follow_the_literal_table() {
    let commute = Commute::default();
    for (length, doors, stride, want_strides, want_estimate, want_lead) in TABLE {
        let way = Way {
            place: place(CAFE),
            doors,
            length: Millimetres::new(length),
        };
        assert_eq!(strides(&way), want_strides, "{length} mm, {doors} doors");
        assert_eq!(estimate(&way, seconds(stride)), want_estimate);
        assert_eq!(commute.lead(Some(&way), seconds(stride)), want_lead);
    }
    assert_eq!(
        commute.lead(None, seconds(STRIDE)),
        7_200,
        "the cap when the way is unknown"
    );
}

#[test]
fn due_from_the_first_consult_at_which_waiting_a_pace_could_make_me_late() {
    let commute = Commute::default();
    let boundary = FROM - LEAD - PACE;
    assert_eq!(
        due(&View::at_home(boundary), commute),
        None,
        "t + pace = from − lead"
    );
    assert_eq!(
        due(&View::at_home(boundary + 1), commute),
        Some(place(CAFE)),
        "one second later"
    );
    assert_eq!(due(&View::at_home(FROM - 1), commute), Some(place(CAFE)));
    assert_eq!(
        due(&View::at_home(FROM), commute),
        None,
        "once it begins, it is the agenda"
    );
}

#[test]
fn not_due_when_there_not_punctual_unknown_or_at_the_boundary_mode() {
    let commute = Commute::default();
    let t = FROM - 600;
    let there = View {
        here: CAFE,
        ..View::at_home(t)
    };
    assert_eq!(due(&there, commute), None, "already there");
    let loose = View {
        punctual: false,
        ..View::at_home(t)
    };
    assert_eq!(due(&loose, commute), None, "not punctual");
    let at_boundary = Commute {
        mode: Mode::AtBoundary,
        ..commute
    };
    assert_eq!(
        due(&View::at_home(t), at_boundary),
        None,
        "at-boundary plans nothing"
    );

    // With no `ways`, the cap: due from FROM − 7 200 − 900.
    let unknown = |at| View {
        ways: false,
        ..View::at_home(at)
    };
    assert_eq!(due(&unknown(FROM - 8_100), commute), None);
    assert_eq!(due(&unknown(FROM - 8_099), commute), Some(place(CAFE)));
}

#[test]
fn a_due_commute_asks_every_seed_to_walk_there_before_answering_a_line() {
    let t = FROM - 600;
    let addressed = View {
        heard: vec![line(t - 10)],
        ..View::at_home(t)
    };
    for seed in 0..SEEDS {
        let decided = controller(seed, Commute::default())
            .decide(&addressed.build())
            .expect("a request");
        assert_eq!(
            walks_into(&decided),
            Some(place(CAFE)),
            "seed {seed}: no draw decides it"
        );
    }
    // After replies: some seed answers the line first.
    let after = Commute {
        precedence: Precedence::AfterReplies,
        ..Commute::default()
    };
    let replied = (0..SEEDS)
        .filter_map(|seed| controller(seed, after).decide(&addressed.build()))
        .filter(|request| *request.action_type() == Talk::ACTION_TYPE)
        .count();
    assert!(replied > 0, "after-replies answers first: {replied}");
}

#[test]
fn on_the_way_nothing_redirects_the_walk() {
    // On the street, walking to the café: only a line may be answered between strides.
    let on_the_way = View {
        here: 2,
        walking_to: Some(CAFE),
        heard: vec![line(FROM - 610)],
        ..View::at_home(FROM - 600)
    };
    for seed in 0..SEEDS {
        let decided = controller(seed, Commute::default()).decide(&on_the_way.build());
        assert!(
            decided
                .as_ref()
                .is_none_or(|request| *request.action_type() == Talk::ACTION_TYPE),
            "seed {seed}: {decided:?}"
        );
    }
}

#[test]
fn with_no_punctual_part_every_decision_is_what_it_was() {
    // I-LE-4: the policy is inert where nothing is punctual.
    for at in (FROM - 4 * 3_600..FROM + 3_600).step_by(300) {
        let view = View {
            punctual: false,
            heard: vec![line(at - 10)],
            ..View::at_home(at)
        };
        let observation = view.build();
        for seed in 0..SEEDS {
            assert_eq!(
                controller(seed, Commute::default()).decide(&observation),
                PacedRuleController::new(seed, seconds(PACE)).decide(&observation),
                "seed {seed} at {at}"
            );
        }
    }
}

#[test]
fn the_policy_decodes_with_defaults_and_refuses_out_of_bounds_by_name() {
    let decoded: Commute = serde_json::from_value(json!({})).expect("defaults");
    assert_eq!(decoded, Commute::default());
    let decoded: Commute = serde_json::from_value(
        json!({ "mode": "at-boundary", "slack": 0, "precedence": "after-replies" }),
    )
    .expect("decodes");
    assert_eq!(
        (decoded.mode, decoded.slack, decoded.precedence),
        (Mode::AtBoundary, 0, Precedence::AfterReplies)
    );
    for (wrong, says) in [
        (json!({ "slack": 2_001 }), "'slack' is 0 … 2000, not 2001"),
        (
            json!({ "cap_minutes": 0 }),
            "'cap_minutes' is 15 … 480, not 0",
        ),
        (
            json!({ "margin_minutes": 61 }),
            "'margin_minutes' is 0 … 60",
        ),
        (json!({ "mode": "early" }), "unknown variant `early`"),
        (json!({ "lead": 3 }), "unknown field `lead`"),
    ] {
        let error = serde_json::from_value::<Commute>(wrong.clone()).expect_err("refused");
        assert!(error.to_string().contains(says), "{wrong}: {error}");
    }
}
