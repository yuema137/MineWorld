//! The `needs:` section in its two forms (SD-Nb-2): what decodes, what is refused and where, what each
//! form requires, and the genesis facts each becomes in both test worlds.

mod support;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{EntityKey, Rejection, SystemId};
use mineworld_needs::{
    AuthoredNeeds, Hunger, Mode, NeedsAssigned, NeedsSystem, Nourishment, NourishmentAssigned,
    Quantity, QuantityUnit, SECTION_KIND, Serving, StartingHunger,
};
use support::{GENESIS, Kind, Town, assemble, facts_of};

fn decode(yaml: &str) -> Result<AuthoredNeeds, String> {
    serde_saphyr::from_str::<AuthoredNeeds>(yaml).map_err(|error| error.to_string())
}

fn unitless(hunger: u16, tiredness: u16) -> AuthoredNeeds {
    AuthoredNeeds::Person {
        hunger: StartingHunger::Unitless { hunger },
        tiredness,
    }
}

#[test]
fn the_person_form_takes_its_defaults_and_the_item_form_its_amount() {
    assert_eq!(decode("{}"), Ok(unitless(300, 700)));
    assert_eq!(decode("hunger: 200"), Ok(unitless(200, 700)));
    assert_eq!(
        decode("{ hunger: 0, tiredness: 1000 }"),
        Ok(unitless(0, 1_000))
    );
    assert_eq!(
        decode("nourishment: { hunger: 600 }"),
        Ok(AuthoredNeeds::Item(Nourishment::Unitless { hunger: 600 }))
    );
}

/// The food-energy forms (`ARC-370`): a person's `food_energy.initial_kcal` (tiredness keeps its
/// default) and an item's `energy_kcal` with its `serving`.
#[test]
fn the_food_energy_forms_decode_with_their_serving() {
    assert_eq!(
        decode("food_energy: { initial_kcal: 1500 }"),
        Ok(AuthoredNeeds::Person {
            hunger: StartingHunger::FoodEnergy {
                initial_kcal: 1_500
            },
            tiredness: 700
        })
    );
    assert_eq!(
        decode(
            "nourishment: { energy_kcal: 0, serving: { label: \"1 glass\", quantity: { unit: ml, amount: 250 } } }"
        ),
        Ok(AuthoredNeeds::Item(Nourishment::FoodEnergy {
            energy_kcal: 0,
            serving: Serving {
                label: "1 glass".to_owned(),
                quantity: Quantity {
                    unit: QuantityUnit::Ml,
                    amount: 250
                }
            }
        }))
    );
}

/// NE-1's decode refusals: the models' keys mixed, energy without a serving (and the reverse), a kcal
/// or amount out of bound, a bad label — each at the section, the bounds at their line.
#[test]
fn the_food_energy_forms_are_refused_for_mixed_keys_and_bounds() {
    let serving = "serving: { label: cup, quantity: { unit: ml, amount: 200 } }";
    for (yaml, words) in [
        (
            "{ hunger: 1, food_energy: { initial_kcal: 1 } }".to_owned(),
            "never both",
        ),
        (
            format!("nourishment: {{ hunger: 1, energy_kcal: 5, {serving} }}"),
            "never both",
        ),
        (
            "nourishment: { energy_kcal: 5 }".to_owned(),
            "needs a `serving`",
        ),
        (
            format!("nourishment: {{ {serving} }}"),
            "needs an `energy_kcal`",
        ),
        (
            format!("nourishment:\n  energy_kcal: 100001\n  {serving}\n"),
            "not 100001",
        ),
        (
            "food_energy:\n  initial_kcal: 100001\n".to_owned(),
            "not 100001",
        ),
        (
            "nourishment: { energy_kcal: 5, serving: { label: \"\", quantity: { unit: g, amount: 1 } } }"
                .to_owned(),
            "label",
        ),
        (
            "nourishment: { energy_kcal: 5, serving: { label: x, quantity: { unit: g, amount: 0 } } }"
                .to_owned(),
            "amount",
        ),
        (
            "nourishment: { energy_kcal: 5, serving: { label: x, quantity: { unit: oz, amount: 1 } } }"
                .to_owned(),
            "oz",
        ),
    ] {
        let refusal = decode(&yaml).expect_err(&yaml);
        assert!(refusal.contains(words), "{yaml}: {refusal}");
    }
    let refusal = decode("nourishment:\n  energy_kcal: 100001\n  serving: { label: cup, quantity: { unit: ml, amount: 200 } }\n")
        .expect_err("over 100 000");
    assert!(refusal.contains("line 2"), "at its line: {refusal}");
}

#[test]
fn a_number_out_of_range_is_refused_at_its_line() {
    let refusal = decode("nourishment:\n  hunger: 1001\n").expect_err("over 1 000");
    assert!(
        refusal.contains("0 … 1 000, not 1001") && refusal.contains("line 2"),
        "{refusal}"
    );
    let refusal = decode("tiredness: 5\nhunger: 2000\n").expect_err("over 1 000");
    assert!(
        refusal.contains("not 2000") && refusal.contains("line 2"),
        "{refusal}"
    );
}

#[test]
fn mixed_forms_and_unknown_keys_are_refused_naming_the_keys() {
    let mixed = decode("{ hunger: 1, nourishment: { hunger: 1 } }").expect_err("both forms");
    assert!(mixed.contains("never both"), "{mixed}");
    let unknown = decode("{ thirst: 3 }").expect_err("no thirst");
    assert!(unknown.contains("thirst"), "{unknown}");
    let inner = decode("nourishment: { hunger: 1, thirst: 2 }").expect_err("no thirst");
    assert!(inner.contains("thirst"), "{inner}");
    let empty = decode("nourishment: {}").expect_err("needs hunger");
    assert!(empty.contains("hunger"), "{empty}");
}

#[test]
fn the_item_form_requires_consumption_and_the_person_form_nothing() {
    let item = decode("nourishment: { hunger: 600 }").expect("decodes");
    assert_eq!(
        <NeedsSystem as AuthoredSection>::requires(&item),
        vec![SystemId::from_static("consumption")]
    );
    let person = decode("{}").expect("decodes");
    assert!(<NeedsSystem as AuthoredSection>::requires(&person).is_empty());
}

#[test]
fn a_form_on_the_other_kind_of_file_is_refused_with_its_code() {
    let assembled = assemble(Kind::FlatAndCafe, true);
    let read = assembled.world.read();
    let keys = assembled
        .keys
        .iter()
        .map(|(key, id)| (EntityKey::new(key.as_str()).expect("a key"), *id))
        .collect();
    let seeding = Seeding::new(&read, &keys);
    let code = |result: Result<_, Rejection>| match result {
        Err(Rejection::System { code, detail }) => (code, detail.unwrap_or_default()),
        other => panic!("expected needs-section-kind, got {other:?}"),
    };
    let person_form = decode("{}").expect("decodes");
    let (refused, detail) = code(NeedsSystem::seed(
        &seeding,
        assembled.keys["soup"],
        &person_form,
    ));
    assert_eq!(refused, SECTION_KIND);
    assert!(detail.contains("person form"), "{detail}");
    let item_form = decode("nourishment: { hunger: 5 }").expect("decodes");
    let (refused, detail) = code(NeedsSystem::seed(
        &seeding,
        assembled.keys["ana"],
        &item_form,
    ));
    assert_eq!(refused, SECTION_KIND);
    assert!(detail.contains("item"), "{detail}");
    let (refused, _) = code(NeedsSystem::seed(
        &seeding,
        assembled.keys["flat"],
        &person_form,
    ));
    assert_eq!(refused, SECTION_KIND);
}

#[test]
fn genesis_states_one_fact_per_form_with_the_authored_numbers() {
    for (kind, people, kinds) in [
        (
            Kind::FlatAndCafe,
            [("ana", 300, 700), ("ben", 900, 700)],
            [("soup", 600), ("apple", 200)],
        ),
        (
            Kind::NightShift,
            [("dee", 300, 700), ("eli", 300, 700)],
            [("stew", 500), ("bread", 300)],
        ),
    ] {
        let town = Town::new(kind);
        let assigned = facts_of::<NeedsAssigned>(&town.log);
        assert_eq!(assigned.len(), 2, "{kind:?}");
        for (name, hunger, tiredness) in people {
            let fact = assigned
                .iter()
                .find(|(_, fact)| fact.person().entity_id() == town.id(name))
                .expect("assigned");
            assert_eq!(fact.0, GENESIS);
            assert_eq!(
                (fact.1.hunger(), fact.1.tiredness()),
                (StartingHunger::Unitless { hunger }, tiredness)
            );
            let needs = town.needs(name).expect("stored");
            let Hunger::Unitless(stored) = needs.hunger() else {
                panic!("a unitless world stores a unitless hunger");
            };
            assert_eq!(
                (stored.stored(), needs.tiredness().stored()),
                (hunger, tiredness)
            );
            assert_eq!((stored.at(), needs.mode()), (GENESIS, Mode::Awake));
        }
        let nourished = facts_of::<NourishmentAssigned>(&town.log);
        assert_eq!(nourished.len(), 2, "{kind:?}");
        for (name, hunger) in kinds {
            assert!(
                nourished
                    .iter()
                    .any(|(_, fact)| fact.item() == town.item(name)
                        && *fact.nourishment() == Nourishment::Unitless { hunger }),
                "{name}"
            );
        }
        let third = if kind == Kind::FlatAndCafe {
            "cy"
        } else {
            "fay"
        };
        assert!(town.needs(third).is_none(), "no section, no needs");
    }
}
