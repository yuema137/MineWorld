//! A manager's `penalize` is a scored choice, never the generic offered one (`ARC-287`, §0.5): its score
//! is `base.penalize` times the `strictness` curve over how often the member of staff was late.

use mineworld_contracts::{
    ActionTypeId, Affordance, ComponentRecord, EntityId, EntityType, Observation, PerceivedEntity,
    SpatialRequirement, WorldTime,
};
use serde_json::{Value, json};

use crate::considerations::{self, Consideration};
use crate::options::{Choice, Context, choices};
use crate::settings::Settings;

const ME: u64 = 3;

fn person(raw: u64) -> Value {
    json!({ "entity": raw.to_string(), "entity_type": "person" })
}

fn manager(count: u32) -> Observation<Value> {
    let record: ComponentRecord<Value> = serde_json::from_value(json!({
        "entity": ME.to_string(),
        "component_type": "employment",
        "schema_version": 1,
        "payload": { "job": {}, "shift": null, "late_staff": [
            { "employee": person(5), "shift": 28_800, "late_by": 600, "count": count,
              "level": "final", "bound": 600 } ] },
    }))
    .expect("a record");
    Observation::new(EntityId::from_raw(ME), WorldTime::from_seconds(43_200))
        .perceiving(vec![
            PerceivedEntity::new(EntityId::from_raw(ME), EntityType::Person)
                .with_components(vec![record]),
        ])
        .offering(vec![
            Affordance::available(
                ActionTypeId::from_static("penalize"),
                None,
                SpatialRequirement::NONE,
            )
            .with_payload(json!({ "employee": person(5), "amount": 600 })),
        ])
}

#[test]
fn a_penalty_is_its_own_choice_scored_by_strictness() {
    let settings = Settings::defaults();
    assert_eq!(settings.base.penalize, 600);
    for (count, strictness) in [(1, 100), (2, 500), (3, 900)] {
        let observation = manager(count);
        let context = Context::new(&observation, &settings).expect("a context");
        let offered = choices(&context);
        assert!(offered.contains(&Choice::Penalize(0)), "{offered:?}");
        assert!(
            !offered.contains(&Choice::Offered(0)),
            "never the generic offer"
        );
        assert_eq!(
            considerations::of(&Choice::Penalize(0), &context, &settings),
            [(Consideration::Strictness, strictness)]
        );
    }
}
