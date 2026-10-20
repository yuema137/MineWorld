//! The composition golden (S30-f §5.4), the reader half: merge, order, compose, inspect and sheet,
//! equal to the hand-written expectations the Python and GDScript readers also read.

use serde_json::Value;

use super::*;

const GOLDEN: &str = include_str!("../../tests/fixtures/composition.json");

fn cases() -> Vec<Value> {
    let golden: Value = serde_json::from_str(GOLDEN).expect("the golden is JSON");
    golden["cases"].as_array().expect("cases").clone()
}

fn disclosed(case: &Value) -> Disclosed {
    let components = case["components"].as_array().expect("components");
    Disclosed::from_payloads(
        components
            .iter()
            .map(|component| (component["type"] == "traits", &component["payload"])),
    )
}

fn strings(value: &Value) -> Vec<String> {
    value
        .as_array()
        .expect("a list")
        .iter()
        .map(|item| item.as_str().expect("a string").to_owned())
        .collect()
}

#[test]
fn every_golden_case_merges_composes_and_lists_as_written() {
    for case in cases() {
        let name = case["name"].as_str().expect("a name");
        let disclosed = disclosed(&case);
        let merged: Vec<String> = disclosed
            .entries()
            .iter()
            .map(|entry| entry.attribute.to_string())
            .collect();
        assert_eq!(merged, strings(&case["merged"]), "{name}: merged");
        for composition in case["compose"].as_array().expect("compose") {
            let facet = composition["facet"].as_str();
            let budget = composition["budget"]
                .as_u64()
                .map(|budget| usize::try_from(budget).expect("fits"));
            let text = compose(disclosed.entries(), facet, budget);
            assert_eq!(
                text,
                composition["text"].as_str().expect("text"),
                "{name}: {facet:?} at {budget:?}"
            );
            if let Some(budget) = budget {
                assert!(text.len() <= budget, "{name}: {} > {budget}", text.len());
            }
        }
        assert_eq!(
            inspect(disclosed.entries()),
            strings(&case["inspect"]),
            "{name}: inspect"
        );
        assert_eq!(
            sheet(disclosed.entries()),
            strings(&case["sheet"]),
            "{name}: sheet"
        );
    }
}

#[test]
fn the_typed_reader_answers_by_kind_and_never_across_kinds() {
    let case = cases()
        .into_iter()
        .find(|case| case["name"] == "merge-and-order")
        .expect("the case");
    let disclosed = disclosed(&case);
    let id = |text: &str| QualifiedId::new(text).expect("an id");
    assert_eq!(disclosed.flag(&id("t.glasses")), Some(false));
    assert_eq!(
        disclosed.category(&id("t.mark")).map(ValueSlug::as_str),
        Some("scar")
    );
    assert_eq!(
        disclosed.set(&id("t.languages")).map(BTreeSet::len),
        Some(2)
    );
    // a perceived-as quantity carries words only: no value to read
    assert_eq!(disclosed.quantity(&id("t.stature")), None);
    // another kind reads None
    assert_eq!(disclosed.level(&id("t.mark")), None);
    assert_eq!(disclosed.level(&id("t.absent")), None);
}
