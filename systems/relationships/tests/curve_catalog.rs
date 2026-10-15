//! FA-8 — a user's own curve through relationships' `DecayCurve` catalog (`ARC-62`, `ARC-86`;
//! `pr-fg-a-relationship-forgetting.md` §5.1, `README.md` "Writing your own curve").
//!
//! A catalog is written once per process and Cargo runs each file under `tests/` as its own process, so
//! this file holds one `#[test]`, which performs the README's five steps in order (the rebuild aside):
//!
//! ```text
//! 1  two test packs implement DecayCurve + Default: step-down (half the excess once a time constant
//!    has passed) and greedy (a hostile curve giving back 5 000 more than it was handed)
//! 2  "install": register them, as the installed set's extension line does; a curve named like a
//!    bundled one panics naming it; a second, different list panics naming both
//! 3  a section naming step-down requires its pack, so a world that does not enable it is refused
//! 4  name it: the section decodes; an unknown name is refused listing every curve
//! 5  a scripted world discloses step-down's literal; greedy is clamped to the stored value  M-FA8
//! ```

mod support;

use std::panic::{AssertUnwindSafe, catch_unwind};

use mineworld_contracts::{SimDuration, SystemId};
use mineworld_relationships::{
    CurveName, DecayCurve, RelationshipsSystem, register_curves, registered_curves,
};
use mineworld_sdk::SystemPack;
use support::{ALL, Cafe};

const DAY: i64 = 86_400;

/// Half the excess once a whole time constant has passed; all of it before.
#[derive(Default)]
struct StepDown;

impl DecayCurve for StepDown {
    fn name(&self) -> CurveName {
        CurveName::from_static("step-down")
    }

    fn remaining(&self, excess: u32, elapsed: SimDuration, time_constant: SimDuration) -> u32 {
        if elapsed.seconds() >= time_constant.seconds() {
            excess / 2
        } else {
            excess
        }
    }

    fn curve_of(&self) -> SystemId {
        SystemId::from_static("test-step-down")
    }
}

/// Breaks the contract: more than it was handed.
#[derive(Default)]
struct Greedy;

impl DecayCurve for Greedy {
    fn name(&self) -> CurveName {
        CurveName::from_static("greedy")
    }

    fn remaining(&self, excess: u32, _: SimDuration, _: SimDuration) -> u32 {
        excess + 5_000
    }

    fn curve_of(&self) -> SystemId {
        SystemId::from_static("test-greedy")
    }
}

/// Claims a bundled curve's name.
#[derive(Default)]
struct Impostor;

impl DecayCurve for Impostor {
    fn name(&self) -> CurveName {
        CurveName::HALF_LIFE
    }

    fn remaining(&self, excess: u32, _: SimDuration, _: SimDuration) -> u32 {
        excess
    }

    fn curve_of(&self) -> SystemId {
        SystemId::from_static("test-impostor")
    }
}

fn panic_text(result: std::thread::Result<()>) -> String {
    let payload = result.expect_err("it panics");
    payload
        .downcast_ref::<String>()
        .cloned()
        .or_else(|| {
            payload
                .downcast_ref::<&str>()
                .map(|text| (*text).to_owned())
        })
        .expect("a message")
}

/// The section's requirements as the loader asks them, from the section decoded by its owner.
fn requires(section: &str) -> Result<Vec<SystemId>, String> {
    let held =
        RelationshipsSystem::decode_configuration(&mut serde_json::Deserializer::from_str(section))
            .map_err(|error| error.to_string())?;
    Ok(held.requires())
}

/// What a café configured with `section` shows Alice of Bob's familiarity: after thirty hellos
/// (familiarity 300), then after `waited` seconds with no contact.
fn disclosed_after(section: &str, waited: i64) -> i64 {
    let mut cafe = Cafe::configured(ALL, section);
    let (alice, bob) = (cafe.alice, cafe.bob);
    for _ in 0..30 {
        cafe.talk(alice, bob);
    }
    assert_eq!(cafe.values(alice, bob).expect("known").familiarity(), 300);
    cafe.pass(waited);
    let record = cafe
        .observation(alice)
        .entity(alice)
        .expect("herself")
        .components()
        .iter()
        .find(|record| record.component_type().as_str() == "acquaintances")
        .expect("disclosed")
        .payload()
        .clone();
    record["known"][0]["values"]["familiarity"]
        .as_i64()
        .expect("an integer")
}

#[test]
fn a_users_curve_is_registered_named_required_and_clamped() {
    // 2: a bundled name is refused before anything is registered.
    let impostor = panic_text(catch_unwind(|| register_curves(vec![Box::new(Impostor)])));
    assert!(
        impostor.contains("'half-life'") && impostor.contains("bundled"),
        "{impostor}"
    );
    assert_eq!(registered_curves(), None);

    register_curves(vec![Box::new(StepDown), Box::new(Greedy)]);
    register_curves(vec![Box::new(Greedy), Box::new(StepDown)]); // the same list: nothing happens
    let names = |names: Option<Vec<CurveName>>| -> Vec<String> {
        names
            .unwrap_or_default()
            .iter()
            .map(ToString::to_string)
            .collect()
    };
    assert_eq!(names(registered_curves()), ["greedy", "step-down"]);
    let different = panic_text(catch_unwind(AssertUnwindSafe(|| {
        register_curves(vec![Box::new(StepDown)]);
    })));
    assert!(
        different.contains(r#"called with [CurveName("step-down")]"#)
            && different
                .contains(r#"already registered [CurveName("greedy"), CurveName("step-down")]"#),
        "names both lists: {different}"
    );

    // 3 and 4: named, the section decodes and requires the curve's pack; a bundled curve requires
    // nothing; an unknown name is refused, listing every curve this build has.
    let step_down = r#"{ "parameters": [ { "familiarity_decay": "step-down", "familiarity_decay_days": 1 } ] }"#;
    assert_eq!(
        requires(step_down),
        Ok(vec![SystemId::from_static("test-step-down")])
    );
    assert_eq!(
        requires(r#"{ "parameters": [ { "regard_decay": "power" } ] }"#),
        Ok(Vec::new())
    );
    let unknown =
        requires(r#"{ "parameters": [ { "familiarity_decay": "nope" } ] }"#).expect_err("refused");
    assert!(
        unknown.contains(
            "'familiarity_decay' is one of none, linear, half-life, power, greedy, step-down, not nope"
        ),
        "{unknown}"
    );

    // 5: the curve's own arithmetic, through a scripted world: nothing before a day, half after.
    assert_eq!(disclosed_after(step_down, DAY - 1), 300);
    assert_eq!(disclosed_after(step_down, DAY), 150);
    // The host clamps a curve that gives back more than it was handed: never above the stored value.
    let greedy =
        r#"{ "parameters": [ { "familiarity_decay": "greedy", "familiarity_decay_days": 1 } ] }"#;
    assert_eq!(disclosed_after(greedy, 3 * DAY), 300);
}
