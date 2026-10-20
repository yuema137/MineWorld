//! `configure/needs.yaml` (SD-Nb-4): each bound refused at its line, the base-only fields refused under
//! a role, what the section requires, and the night-shift's actor-scoped rates in force.

mod support;

use mineworld_contracts::SystemId;
use mineworld_needs::NeedsSystem;
use mineworld_needs::interactions::{Thresholds, parameters};
use mineworld_sdk::interactions::{InteractionSection, Section};
use support::{Kind, Town};

fn decode(yaml: &str) -> Result<Section<NeedsSystem>, String> {
    serde_saphyr::from_str::<Section<NeedsSystem>>(yaml).map_err(|error| error.to_string())
}

fn refused(yaml: &str, needle: &str, line: u32) {
    let refusal = decode(yaml).expect_err(yaml);
    assert!(
        refusal.contains(needle) && refusal.contains(&format!("line {line}")),
        "{yaml}: {refusal}"
    );
}

#[test]
fn each_bound_is_refused_at_its_line_in_the_packs_words() {
    refused(
        "parameters:\n  - thresholds: [600, 400, 900]\n",
        "three integers in 1 … 1 000, strictly increasing",
        2,
    );
    refused(
        "parameters:\n  - { thresholds: [0, 400, 900] }\n",
        "strictly increasing",
        2,
    );
    refused(
        "parameters:\n  - {}\n  - { rest_per_hour: 5 }\n",
        "'rest_per_hour' is 10 … 1000, not 5",
        3,
    );
    refused("parameters:\n  - { hunger_per_hour: 501 }\n", "not 501", 2);
    refused(
        "parameters:\n  - { restore: { hunger: [eating, eating] } }\n",
        "each at most once",
        2,
    );
    refused(
        "parameters:\n  - { restore: { tiredness: [eating] } }\n",
        "from [sleeping] for tiredness",
        2,
    );
}

#[test]
fn thresholds_and_restore_are_base_only_and_no_rule_is_declared() {
    for field in ["thresholds: [300, 500, 800]", "restore: { tiredness: [] }"] {
        let yaml = format!("parameters:\n  - {{ actor: night, {field} }}\n");
        let refusal = decode(&yaml).expect_err(&yaml);
        assert!(refusal.contains("line 2"), "{refusal}");
    }
    let rule = decode("rules:\n  - { action: eat, effect: forbid }\n").expect_err("no actions");
    assert!(rule.contains("line 2"), "{rule}");
    decode("parameters:\n  - { actor: night, hunger_per_hour: 90 }\n").expect("a rate by actor");
}

#[test]
fn the_section_requires_what_its_restore_names() {
    let requires =
        |yaml: &str| <NeedsSystem as InteractionSection>::requires(&decode(yaml).expect("decodes"));
    let (eating, sleeping) = (
        SystemId::from_static("consumption"),
        SystemId::from_static("sleep"),
    );
    assert_eq!(
        <NeedsSystem as InteractionSection>::requires(&Section::empty()),
        vec![eating.clone(), sleeping.clone()]
    );
    assert_eq!(
        requires("parameters:\n  - { hunger_per_hour: 90 }\n"),
        vec![eating.clone(), sleeping.clone()]
    );
    assert_eq!(
        requires("parameters:\n  - { restore: { tiredness: [] } }\n"),
        vec![eating]
    );
    assert_eq!(
        requires("parameters:\n  - { restore: { hunger: [] } }\n"),
        vec![sleeping]
    );
    assert!(requires("parameters:\n  - { restore: { hunger: [], tiredness: [] } }\n").is_empty());
}

#[test]
fn night_shift_rates_resolve_for_a_night_person_and_not_for_others() {
    let town = Town::new(Kind::NightShift);
    let read = town.world.read();
    let dee = parameters(&read, town.id("dee"));
    assert_eq!(
        (
            dee.hunger_per_hour,
            dee.hunger_asleep_per_hour,
            dee.tiredness_per_hour,
            dee.rest_per_hour
        ),
        (90, 30, 60, 200)
    );
    assert_eq!(dee.thresholds, Thresholds([300, 500, 800]));
    let eli = parameters(&read, town.id("eli"));
    assert_eq!((eli.hunger_per_hour, eli.rest_per_hour), (120, 125));
    assert_eq!(eli.thresholds, Thresholds([300, 500, 800]));

    let flat = Town::new(Kind::FlatAndCafe);
    let ana = parameters(&flat.world.read(), flat.id("ana"));
    assert_eq!(
        ana,
        mineworld_needs::interactions::NeedsParameters::default()
    );
}
