//! `rule`'s needs policy and bands, decided from observations built by hand (`pr-s35-nc-controllers.md`
//! SD-Nc-1 … SD-Nc-6, AC-Nc-2). The `needs` record is written as the world discloses it — slug and JSON —
//! since this controller may not name the pack's crate (F-Nc-3).

use mineworld_decision::ModelParameters;
use serde_json::json;

use crate::RuleParameters;
use crate::needs::{ClockTime, NeedsPolicy, Night};

fn policy(value: serde_json::Value) -> Result<NeedsPolicy, serde_json::Error> {
    serde_json::from_value(value)
}

#[test]
fn the_needs_block_decodes_with_the_step_defaults() {
    let decoded = policy(json!({})).expect("defaults");
    assert_eq!(decoded, NeedsPolicy::default());
    assert_eq!(
        (
            decoded.eat_from,
            decoded.tired_from,
            decoded.exhausted_from,
            decoded.night.from.minutes(),
            decoded.night.until.minutes(),
            decoded.bedtime_spread_minutes,
            decoded.wake_for_commute,
        ),
        (600, 600, 900, 22 * 60, 6 * 60, 60, true)
    );
    let owls = policy(json!({
        "night": { "from": "01:00", "until": "09:00" },
        "eat_from": 1, "tired_from": 1_000, "exhausted_from": 1_000,
        "bedtime_spread_minutes": 240, "wake_for_commute": false,
    }))
    .expect("a town of night owls");
    assert_eq!(
        owls.night,
        Night {
            from: ClockTime::new(1, 0).expect("a time"),
            until: ClockTime::new(9, 0).expect("a time"),
        }
    );
    assert!(!owls.wake_for_commute);
    let parameters = RuleParameters::decode(json!({ "needs": { "eat_from": 700 } }))
        .expect("the rule block carries needs");
    assert_eq!(parameters.needs.eat_from, 700);
    assert_eq!(parameters.commute, crate::commute::Commute::default());
}

#[test]
fn the_needs_block_refuses_each_bound_by_name() {
    for (wrong, names) in [
        (json!({ "eat_from": 0 }), "'eat_from' is 1 … 1000, not 0"),
        (json!({ "eat_from": 1_001 }), "'eat_from' is 1 … 1000"),
        (json!({ "tired_from": 0 }), "'tired_from' is 1 … 1000"),
        (
            json!({ "exhausted_from": 1_001 }),
            "'exhausted_from' is 1 … 1000",
        ),
        (
            json!({ "bedtime_spread_minutes": 241 }),
            "'bedtime_spread_minutes' is 0 … 240",
        ),
        (
            json!({ "night": { "from": "22:00", "until": "22:00" } }),
            "the night is empty",
        ),
        (json!({ "night": { "from": "24:00" } }), "not a time of day"),
        (json!({ "night": { "from": "7:00" } }), "not a time of day"),
        (json!({ "night": { "at": "22:00" } }), "unknown field `at`"),
        (json!({ "sleepy_from": 3 }), "unknown field `sleepy_from`"),
        (
            json!({ "tired_from": 700, "exhausted_from": 650 }),
            "'exhausted_from' (650) is below 'tired_from' (700)",
        ),
        (
            json!({ "exhausted_from": 650, "tired_from": 700 }),
            "'exhausted_from' (650) is below 'tired_from' (700)",
        ),
        (
            json!({ "exhausted_from": 500 }),
            "'exhausted_from' (500) is below 'tired_from' (600)",
        ),
    ] {
        let error = policy(wrong.clone()).expect_err("refused").to_string();
        assert!(error.contains(names), "{wrong}: {error}");
    }
}
