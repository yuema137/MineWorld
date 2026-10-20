//! The `needs:` section in its two forms (SD-Nb-2): what decodes, what is refused and where, what each
//! form requires, and the genesis facts each becomes in both test worlds.

mod support;

use mineworld_authoring::{AuthoredSection, Seeding};
use mineworld_contracts::{EntityKey, Rejection, SystemId};
use mineworld_needs::{
    AuthoredNeeds, Mode, NeedsAssigned, NeedsSystem, NourishmentAssigned, SECTION_KIND,
};
use support::{GENESIS, Kind, Town, assemble, facts_of};

fn decode(yaml: &str) -> Result<AuthoredNeeds, String> {
    serde_saphyr::from_str::<AuthoredNeeds>(yaml).map_err(|error| error.to_string())
}

#[test]
fn the_person_form_takes_its_defaults_and_the_item_form_its_amount() {
    assert_eq!(
        decode("{}"),
        Ok(AuthoredNeeds::Person {
            hunger: 300,
            tiredness: 700
        })
    );
    assert_eq!(
        decode("hunger: 200"),
        Ok(AuthoredNeeds::Person {
            hunger: 200,
            tiredness: 700
        })
    );
    assert_eq!(
        decode("{ hunger: 0, tiredness: 1000 }"),
        Ok(AuthoredNeeds::Person {
            hunger: 0,
            tiredness: 1_000
        })
    );
    assert_eq!(
        decode("nourishment: { hunger: 600 }"),
        Ok(AuthoredNeeds::Item { hunger: 600 })
    );
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
                (fact.1.levels().hunger, fact.1.levels().tiredness),
                (hunger, tiredness)
            );
            let needs = town.needs(name).expect("stored");
            assert_eq!(
                (needs.hunger().stored(), needs.tiredness().stored()),
                (hunger, tiredness)
            );
            assert_eq!((needs.hunger().at(), needs.mode()), (GENESIS, Mode::Awake));
        }
        let nourished = facts_of::<NourishmentAssigned>(&town.log);
        assert_eq!(nourished.len(), 2, "{kind:?}");
        for (name, hunger) in kinds {
            assert!(
                nourished
                    .iter()
                    .any(|(_, fact)| fact.item() == town.item(name) && fact.hunger() == hunger),
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
