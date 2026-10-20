//! The boss policy, decided from observations built by hand (`ARC-287`, §0.5). The `penalize` offer and
//! the manager's `employment` record are written as the world discloses them — slugs and JSON — since
//! this controller may not name the pack that offers them.

use mineworld_contracts::{
    ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType, Observation, PerceivedEntity,
    SimDuration, SpatialRequirement, WorldTime,
};
use serde_json::{Value, json};

use crate::PacedRuleController;
use crate::boss::{Boss, PenalizeFrom};

const ME: u64 = 3;
const CAROL: u64 = 5;
const DAVE: u64 = 6;
const PACE: i64 = 900;
const NOW: i64 = 43_200;

fn person(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "person" })
}

fn penalty(employee: u64, amount: u64) -> Affordance<Value> {
    Affordance::available(
        ActionTypeId::from_static("penalize"),
        None,
        SpatialRequirement::NONE,
    )
    .with_payload(json!({ "employee": person(employee), "amount": amount }))
}

/// My own `employment` record, telling me of `late` staff: (who, count, level, bound).
fn employment(late: &[(u64, u32, &str, u64)]) -> ComponentRecord<Value> {
    let staff: Vec<Value> = late
        .iter()
        .map(|(who, count, level, bound)| {
            json!({ "employee": person(*who), "shift": 28_800, "late_by": 600, "count": count,
                    "level": level, "bound": bound })
        })
        .collect();
    serde_json::from_value(json!({
        "entity": ME.to_string(),
        "component_type": "employment",
        "schema_version": 1,
        "payload": { "job": {}, "shift": null, "late_staff": staff },
    }))
    .expect("a record")
}

fn observation(
    offers: Vec<Affordance<Value>>,
    late: &[(u64, u32, &str, u64)],
) -> Observation<Value> {
    Observation::new(EntityId::from_raw(ME), WorldTime::from_seconds(NOW))
        .perceiving(vec![
            PerceivedEntity::new(EntityId::from_raw(ME), EntityType::Person)
                .with_components(vec![employment(late)]),
        ])
        .offering(offers)
}

fn boss(penalize_from: PenalizeFrom, share: u32) -> PacedRuleController {
    PacedRuleController::new(7, SimDuration::from_seconds(PACE)).bossing(Boss {
        penalize_from,
        share,
    })
}

/// The decided request as (action type, payload).
fn decided(controller: PacedRuleController, observation: &Observation<Value>) -> Option<Value> {
    let request = controller.decide(observation)?;
    assert_eq!(request.action_type().as_str(), "penalize");
    Some(serde_json::from_slice(request.payload().payload()).expect("JSON"))
}

#[test]
fn a_strict_boss_penalizes_the_first_lateness_at_its_share_of_the_bound() {
    let seen = observation(
        vec![penalty(CAROL, 600), penalty(DAVE, 400)],
        &[(CAROL, 1, "notice", 600), (DAVE, 2, "warning", 400)],
    );
    assert_eq!(
        decided(boss(PenalizeFrom::Notice, 500), &seen),
        Some(json!({ "employee": person(CAROL), "amount": 300 }))
    );
    // From a warning: Carol's notice is passed over, Dave's warning taken in full.
    assert_eq!(
        decided(boss(PenalizeFrom::Warning, 1_000), &seen),
        Some(json!({ "employee": person(DAVE), "amount": 400 }))
    );
    assert_eq!(decided(boss(PenalizeFrom::Final, 1_000), &seen), None);
    assert_eq!(decided(boss(PenalizeFrom::Never, 1_000), &seen), None);
    // A tiny share still takes at least one.
    assert_eq!(
        decided(boss(PenalizeFrom::Notice, 1), &seen),
        Some(json!({ "employee": person(CAROL), "amount": 1 }))
    );
}

#[test]
fn without_a_boss_policy_or_an_offer_nothing_is_penalized() {
    let seen = observation(vec![penalty(CAROL, 600)], &[(CAROL, 3, "final", 600)]);
    let plain = PacedRuleController::new(7, SimDuration::from_seconds(PACE));
    assert_eq!(
        plain.decide(&seen),
        None,
        "the offered band never picks a penalty"
    );
    let unoffered = observation(Vec::new(), &[(CAROL, 3, "final", 600)]);
    assert_eq!(decided(boss(PenalizeFrom::Notice, 1_000), &unoffered), None);
}

#[test]
fn the_boss_block_is_decoded_with_bounds_and_defaults() {
    let boss: Boss = serde_json::from_str(r#"{ "penalize_from": "notice" }"#).expect("decodes");
    assert_eq!(
        (boss.penalize_from, boss.share),
        (PenalizeFrom::Notice, 1_000)
    );
    assert_eq!(Boss::default().penalize_from, PenalizeFrom::Warning);
    let refused = serde_json::from_str::<Boss>(r#"{ "share": 0 }"#).expect_err("out of bounds");
    assert!(
        refused.to_string().contains("'share' is 1 … 1000, not 0"),
        "{refused}"
    );
    assert!(serde_json::from_str::<Boss>(r#"{ "strict": true }"#).is_err());
}
