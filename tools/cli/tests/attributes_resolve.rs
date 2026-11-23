//! S30-d (`pr-s30d-sampler.md` §8): `mineworld attributes resolve` and `lock` through the real binary on
//! the sampled fixture world.

mod headless;

use headless::{mineworld, stderr, stdout};
use serde_json::Value;

/// The fixture world (§5.11).
const WORLD: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/tests/fixtures/attributes-sampled"
);

#[test]
fn deferred_people_are_listed_without_values() {
    let world = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/tests/fixtures/attributes-newcomers"
    );
    let output = mineworld(&["attributes", "resolve", world, "--json"]);
    assert!(output.status.success(), "{}", stderr(&output));
    let document: Value = serde_json::from_str(&stdout(&output)).expect("JSON");
    for name in ["nadia", "omar", "pia", "quin"] {
        assert_eq!(
            document["people"][name],
            serde_json::json!({"deferred": true})
        );
    }
    assert_eq!(document["people"].as_object().unwrap().len(), 12);
    assert!(document["people"]["dev"]["values"].is_object());
}

fn resolved(extra: &[&str]) -> (String, Value) {
    let mut arguments = vec!["attributes", "resolve", WORLD, "--json"];
    arguments.extend_from_slice(extra);
    let output = mineworld(&arguments);
    assert!(output.status.success(), "{}", stderr(&output));
    let text = stdout(&output);
    let document = serde_json::from_str(&text).expect("one JSON document");
    (text, document)
}

fn value<'d>(document: &'d Value, person: &str, attribute: &str) -> &'d Value {
    &document["people"][person]["values"][format!("attributes-sampled.{attribute}")]
}

/// AC-d-1, AC-d-2's local half, AC-d-13: twelve people; authored, ruled, restricted and role values hold;
/// two runs are byte-identical.
#[test]
fn resolve_prints_every_person_with_origins_and_is_reproducible() {
    let (first, document) = resolved(&[]);
    let (second, _) = resolved(&[]);
    assert_eq!(first, second, "the same bytes twice");
    assert_eq!(
        document["people"].as_object().map(serde_json::Map::len),
        Some(12)
    );
    assert_eq!(value(&document, "alice", "stature")["origin"], "authored");
    assert_eq!(
        value(&document, "alice", "stature")["value"]["quantity"],
        "1.660"
    );
    assert_eq!(value(&document, "bob", "stature")["origin"], "ruled");
    assert_eq!(
        value(&document, "bob", "stature")["value"]["quantity"],
        "1.870"
    );
    assert_eq!(
        value(&document, "carol", "hair-colour")["value"]["category"],
        "blond"
    );
    let languages = &value(&document, "dev", "languages")["value"]["set"];
    assert!(
        languages
            .as_array()
            .is_some_and(|members| members.iter().any(|member| member == "italian")),
        "{languages}"
    );
    for person in ["erin", "felix", "grace"] {
        let sex = value(&document, person, "sex");
        assert!(sex["origin"].get("sampled").is_some(), "{person}: {sex}");
    }
    assert!(document["support"]["attributes-sampled.stature"].is_object());
}

/// AC-d-2 (AC-S30-7): the committed golden equals the output on every platform CI runs. Recorded once by
/// running `mineworld attributes resolve tools/cli/tests/fixtures/attributes-sampled --json` (S30-d); it is
/// evidence of reproducibility, not of correctness.
#[test]
fn resolve_matches_the_committed_golden_byte_for_byte() {
    let golden = include_str!("fixtures/attributes-sampled.resolve.json");
    let (text, _) = resolved(&[]);
    assert_eq!(text.trim_end(), golden.trim_end(), "the resolution moved");
}

/// AC-d-3 (X-24): another salt changes only sampled values.
#[test]
fn another_salt_changes_only_sampled_values() {
    let (_, base) = resolved(&[]);
    let (_, salted) = resolved(&["--salt", "1"]);
    let people = base["people"].as_object().expect("people");
    let mut changed = 0;
    for (person, entry) in people {
        for (attribute, held) in entry["values"].as_object().expect("values") {
            let other = &salted["people"][person]["values"][attribute];
            if held["origin"].get("sampled").is_none() {
                assert_eq!(
                    held, other,
                    "{person} {attribute} is not sampled and must not move"
                );
            } else if held != other {
                changed += 1;
            }
        }
    }
    assert!(changed > 0, "a new salt moves some sampled value");
}

/// AC-d-4 (AC-S30-5, X-25): 1 000 synthetic people; stratified marginals within 0.5 pp.
#[test]
fn a_thousand_synthetic_people_match_the_profile() {
    let (_, document) = resolved(&["--synthetic", "1000"]);
    let marginal: f64 = document["worst"]["marginal_pp"]
        .as_str()
        .and_then(|text| text.parse().ok())
        .expect("a marginal");
    assert!(marginal <= 0.5, "{}", document["worst"]);
}

/// Installed-loader compatibility: real CLI create, reopen/continue and full replay of the
/// existing sampled world. The generic derived-context drift case is the schema seam's test.
#[test]
fn attributes_context_survives_cli_save_resume_and_verify() {
    use mineworld_contracts::{Causation, EventEnvelope};
    use mineworld_persistence::format;
    let save = headless::fresh("attributes-context-restart");
    let path = save.path().to_str().expect("path");
    let run = |days| {
        let output = mineworld(&[
            "run",
            WORLD,
            "--headless",
            "--seed",
            "7",
            "--days",
            days,
            "--save",
            path,
        ]);
        assert!(output.status.success(), "{}", stderr(&output));
        stdout(&output)
    };
    let created = run("1");
    assert!(created.contains("created "), "{created}");
    let before = headless::Tables::read(save.path());
    let context_facts = |tables: &headless::Tables| -> Vec<Vec<u8>> {
        tables
            .facts
            .iter()
            .filter_map(|(_, bytes)| {
                let fact: EventEnvelope = format::decode(bytes, "fact").expect("fact");
                (fact.event_type().as_str() == "attributes-composed"
                    && *fact.caused_by() == Causation::WorldGenesis)
                    .then(|| bytes.clone())
            })
            .collect()
    };
    assert_eq!(context_facts(&before).len(), 1);
    let resumed = run("2");
    assert!(
        resumed.contains("resumed ") && resumed.contains("re-executed"),
        "{resumed}"
    );
    let after = headless::Tables::read(save.path());
    assert_eq!(context_facts(&before), context_facts(&after));
    assert!(
        after.facts.starts_with(&before.facts),
        "saved history retained"
    );
    let verified = mineworld(&["replay", WORLD, "--save", path]);
    assert!(verified.status.success(), "{}", stderr(&verified));
    let report = stdout(&verified);
    assert!(
        report.contains("reproduced byte for byte") && report.contains("head revision"),
        "{report}"
    );
    assert_eq!(
        headless::Tables::read(save.path()).facts,
        after.facts,
        "verify does not append"
    );
}
