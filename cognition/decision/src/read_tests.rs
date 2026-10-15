//! The decision list: refusals by name with line and column, precedence, fallback chains, overrides,
//! memory pass-through, the digest and the JSON document — against a test-only catalog with a
//! parameterised model and external models (`pr-dl-a-decision-list.md` C4, AC-DLa-2, AC-DLa-4,
//! AC-DLa-7, AC-DLa-11).

use std::collections::BTreeSet;
use std::path::Path;

use mineworld_contracts::{ActionRequest, EntityKey};
use serde_json::json;

use crate::{
    Catalog, ClassRef, Consult, DecisionList, DecisionModel, Origin, ResolvedDecisions,
    ResolvedDecisionsJson, Roster, SeatEntry, SeatOverride, bounded,
};

mineworld_sdk::parameters! {
    /// The probe's tuning.
    pub struct Patience, partial PatiencePartial {
        /// Consults it waits.
        patience: u32 = 3, 1 ..= 10;
    }
}

impl crate::ModelParameters for Patience {
    fn decode<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        bounded(deserializer)
    }
}

/// A parameterless model.
#[derive(Default)]
struct Steady;

impl DecisionModel for Steady {
    const ID: &'static str = "steady";
    const PACKAGE: crate::Package = mineworld_sdk::package!();
    type Parameters = ();

    fn decide(&self, (): &(), _: &Consult<'_>) -> Option<ActionRequest> {
        None
    }

    fn step(&self, (): &(), _: &Consult<'_>) -> Option<ActionRequest> {
        None
    }
}

/// A model with parameters.
#[derive(Default)]
struct Probe;

impl DecisionModel for Probe {
    const ID: &'static str = "probe";
    const PACKAGE: crate::Package = mineworld_sdk::package!();
    type Parameters = Patience;

    fn decide(&self, _: &Patience, _: &Consult<'_>) -> Option<ActionRequest> {
        None
    }

    fn step(&self, _: &Patience, _: &Consult<'_>) -> Option<ActionRequest> {
        None
    }
}

/// Stands in for `rule`, which this crate cannot depend on: the implicit default's id.
#[derive(Default)]
struct RuleLike;

impl DecisionModel for RuleLike {
    const ID: &'static str = "rule";
    const PACKAGE: crate::Package = mineworld_sdk::package!();
    type Parameters = ();

    fn decide(&self, (): &(), _: &Consult<'_>) -> Option<ActionRequest> {
        None
    }

    fn step(&self, (): &(), _: &Consult<'_>) -> Option<ActionRequest> {
        None
    }
}

mod catalog {
    crate::models! {
        Rule => super::RuleLike,
        Steady => super::Steady,
        Probe => super::Probe,
        external "lm",
        external "far1",
        external "far2",
        external "far3",
        external "far4",
    }
}

use catalog::Model;

fn key(text: &str) -> EntityKey {
    EntityKey::new(text).expect("a key")
}

fn class(text: &str) -> ClassRef {
    ClassRef::new(text).expect("a class")
}

/// alice and dev are unclassed, bob and carol are staff; zed is a person but no seat.
fn roster() -> Roster {
    let seat = |name: &str, of: &str| SeatEntry {
        seat: key(name),
        class: class(of),
    };
    Roster {
        seats: vec![
            seat("alice", "person"),
            seat("bob", "staff"),
            seat("carol", "staff"),
            seat("dev", "person"),
        ],
        classes: BTreeSet::from([class("staff")]),
        implicit: ["person", "place", "item", "organization"]
            .into_iter()
            .map(class)
            .collect(),
        people: ["alice", "bob", "carol", "dev", "zed"]
            .into_iter()
            .map(key)
            .collect(),
    }
}

fn parse(text: &str) -> Result<DecisionList<Model>, crate::DecisionsError> {
    DecisionList::parse(
        text,
        Path::new("w/decisions.yaml"),
        "decisions.yaml",
        &roster(),
    )
}

fn resolved(text: &str, overrides: &[SeatOverride]) -> ResolvedDecisions {
    parse(text)
        .expect("a valid list")
        .resolve(&roster(), Origin::Pack, overrides)
        .expect("resolves")
}

/// Each seat's driving model and, for an external assignment, its chain.
fn models_of(resolved: &ResolvedDecisions) -> Vec<(String, String)> {
    resolved
        .seats()
        .iter()
        .map(|seat| {
            let drives = seat
                .drives
                .as_ref()
                .map_or("idle".to_owned(), |drives| format!("{drives:?}"));
            (seat.seat.to_string(), drives)
        })
        .collect()
}

/// AC-DLa-2: every refusal at the offending node's line and column, naming it. Columns are counted by
/// hand from the text: `(line, column)` of the key or value the comment names.
#[test]
fn every_refusal_names_its_node_with_line_and_column() {
    let cases: &[(&str, &str, u64, u64, &str)] = &[
        (
            "uninstalled model under models:",
            "models:\n  mystery: {}\n",
            2,
            3, // the key `mystery`
            "'mystery' is not a model this build installs; it installs: rule, steady, probe, lm",
        ),
        (
            "uninstalled model in an assignment",
            "assign:\n  - { seat: alice, model: mystery }\n",
            2,
            27, // the value `mystery`
            "assign entry 1: 'mystery' is not a model this build installs",
        ),
        (
            "uninstalled model as default",
            "default: mystery\n",
            1,
            10,
            "default: 'mystery' is not a model this build installs",
        ),
        (
            "unknown class",
            "assign:\n  - { class: kitchen, model: steady }\n",
            2,
            14, // the value `kitchen`
            "class 'kitchen' is not a class of persons defined in configure/classes.yaml",
        ),
        (
            "implicit class",
            "assign:\n  - { class: person, model: steady }\n",
            2,
            14,
            "'person' is an implicit class",
        ),
        (
            "a person who is not a seat",
            "assign:\n  - { seat: zed, model: steady }\n",
            2,
            13, // the value `zed`
            "'zed' is a person but not a seat",
        ),
        (
            "unknown person",
            "assign:\n  - { seat: nobody, model: steady }\n",
            2,
            13,
            "seat 'nobody' names no person of this world",
        ),
        (
            "unknown parameter",
            "models:\n  probe:\n    parameters:\n      patienc: 4\n",
            4,
            7, // the key `patienc`
            "patienc",
        ),
        (
            "out-of-bound parameter",
            "models:\n  probe:\n    parameters:\n      patience: 99\n",
            4,
            7, // the key `patience`
            "'patience' is 1 … 10, not 99",
        ),
        (
            "parameters on a model that takes none",
            "models:\n  steady:\n    parameters: { speed: 2 }\n",
            3,
            19, // the key `speed`
            "speed",
        ),
        (
            "duplicate seat entry",
            "assign:\n  - { seat: alice, model: steady }\n  - { seat: alice, model: probe }\n",
            3,
            5, // the second entry
            "assign entry 2: a second entry for the same seat",
        ),
        (
            "duplicate class entry",
            "assign:\n  - { class: staff, model: steady }\n  - { class: staff, model: probe }\n",
            3,
            5,
            "assign entry 2: a second entry for the same class",
        ),
        (
            "an entry naming both a seat and a class",
            "assign:\n  - { seat: alice, class: staff, model: steady }\n",
            2,
            5,
            "assign entry 1: an entry names exactly one of 'seat' and 'class'",
        ),
        (
            "an entry naming neither",
            "assign:\n  - { model: steady }\n",
            2,
            5,
            "an entry names exactly one of 'seat' and 'class'",
        ),
        (
            "an unknown key in an assignment",
            "assign:\n  - { seat: alice, model: steady, weight: 2 }\n",
            2,
            35, // the key `weight`
            "weight",
        ),
        (
            "an unknown top-level key",
            "default: steady\nmodles: {}\n",
            2,
            1,
            "modles",
        ),
        (
            "an unknown key in a model entry",
            "models:\n  probe:\n    fallbak: none\n",
            3,
            5,
            "fallbak",
        ),
        (
            "fallback on an in-process model",
            "models:\n  steady:\n    fallback: none\n",
            3,
            15, // the value `none`
            "models.steady.fallback: 'steady' runs in process and is always present",
        ),
        (
            "external model without a fallback",
            "models:\n  lm: {}\n",
            2,
            3,
            "models.lm: 'lm' runs out of process and must name a fallback",
        ),
        (
            "external model assigned but not listed",
            "assign:\n  - { seat: alice, model: lm }\n",
            2,
            27,
            "'lm' runs out of process and has no fallback; list it under models: with one",
        ),
        (
            "external model given parameters",
            "models:\n  lm: { fallback: steady, parameters: {} }\n",
            2,
            27, // the key `parameters`
            "models.lm.parameters: 'lm' runs out of process; this build cannot type its parameters",
        ),
        (
            "a fallback naming an uninstalled model",
            "models:\n  lm: { fallback: mystery }\n",
            2,
            19,
            "models.lm.fallback: 'mystery' is not a model this build installs",
        ),
        (
            "a chain of five links",
            "models:\n  lm: { fallback: far1 }\n  far1: { fallback: far2 }\n  far2: { fallback: far3 }\n  \
             far3: { fallback: far4 }\n  far4: { fallback: steady }\n",
            2,
            19, // lm's `fallback` value: the chain from lm is the one too long
            "the fallback chain from 'lm' has more than 4 links (lm → far1 → far2 → far3 → far4 → steady)",
        ),
        (
            "a cyclic chain",
            "models:\n  lm: { fallback: far1 }\n  far1: { fallback: lm }\n",
            3,
            21, // far1's `fallback` value: far1 is checked first, and its chain returns to it
            "models.far1.fallback: the fallback chain from 'far1' returns to 'far1'",
        ),
        (
            "an external link with no fallback of its own",
            "models:\n  lm: { fallback: far1 }\n",
            2,
            19,
            "'far1' runs out of process and has no fallback",
        ),
        (
            "memory: an unknown key",
            "memory:\n  defaults: { policy: consolidate }\n",
            2,
            3,
            "defaults",
        ),
        (
            "memory: an unknown key in the default",
            "memory:\n  default: { policy: consolidate, extra: 1 }\n",
            2,
            35, // the key `extra`
            "unknown field `extra`",
        ),
        (
            "memory: an unknown key in an entry",
            "memory:\n  assign:\n    - { seat: alice, policy: decay, weight: 1 }\n",
            3,
            37,
            "weight",
        ),
        (
            "memory: a malformed policy id",
            "memory:\n  assign:\n    - { seat: alice, policy: Decay }\n",
            3,
            30, // the value `Decay`
            "memory.assign entry 1.policy: 'Decay' is not a policy id",
        ),
        (
            "memory: a malformed fallback id",
            "memory:\n  default: { policy: lm-summarise, fallback: 9lives }\n",
            2,
            46, // the value `9lives`
            "memory.default.fallback: '9lives' is not a policy id",
        ),
        (
            "memory: a person who is not a seat",
            "memory:\n  assign:\n    - { seat: zed, policy: decay }\n",
            3,
            15,
            "memory.assign entry 1: 'zed' is a person but not a seat",
        ),
        (
            "memory: an undefined class",
            "memory:\n  assign:\n    - { class: kitchen, policy: decay }\n",
            3,
            16,
            "class 'kitchen' is not a class of persons",
        ),
        (
            "memory: a duplicate entry",
            "memory:\n  assign:\n    - { seat: bob, policy: decay }\n    - { seat: bob, policy: consolidate }\n",
            4,
            7,
            "memory.assign entry 2: a second entry for the same seat",
        ),
        (
            "memory: parameters that are not a mapping",
            "memory:\n  default: { policy: decay, parameters: 3 }\n",
            2,
            41, // the value `3`
            "expected mapping",
        ),
        (
            "memory: a parameter with a non-string key",
            "memory:\n  default: { policy: decay, parameters: { 1: a } }\n",
            2,
            43,
            "a string key",
        ),
        (
            "memory: a tagged parameter",
            "memory:\n  default: { policy: decay, parameters: { x: !custom 3 } }\n",
            2,
            54, // the tagged scalar `3`
            "the tag '!custom' is refused",
        ),
        (
            "memory: a non-finite parameter",
            "memory:\n  default: { policy: decay, parameters: { x: .nan } }\n",
            2,
            46, // the value `.nan`, refused by the YAML decoder itself
            "non-finite float",
        ),
        (
            "memory: a store bound that is not a number",
            "memory:\n  default: { policy: decay, store_bound_mib: big }\n",
            2,
            46, // the value `big` (pr-fg-b2 §5)
            "invalid u64",
        ),
        (
            "memory: a negative store bound",
            "memory:\n  assign:\n    - { seat: bob, policy: decay, store_bound_mib: -1 }\n",
            3,
            52, // the value `-1`
            "invalid u64",
        ),
    ];
    let mut wrong = Vec::new();
    for (what, text, line, column, said) in cases {
        match parse(text) {
            Ok(_) => wrong.push(format!("{what}: accepted")),
            Err(error) => {
                let place_ok = (error.line, error.column) == (*line, *column);
                let said_ok = error.detail.contains(said);
                let path_ok = error.path == Path::new("w/decisions.yaml");
                if !(place_ok && said_ok && path_ok) {
                    wrong.push(format!(
                        "{what}: expected {line}:{column} containing {said:?}; got {}:{} {:?}",
                        error.line, error.column, error.detail
                    ));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "\n{}", wrong.join("\n"));
}

/// AC-DLa-4: seat > class > default > rule, and `--seat-model` above them all.
#[test]
fn a_seat_entry_beats_its_class_which_beats_the_default() {
    let text = "models:\n  probe:\n    parameters: { patience: 7 }\ndefault: probe\nassign:\n  - { class: \
                staff, model: steady }\n  - { seat: bob, model: rule }\n";
    assert_eq!(
        models_of(&resolved(text, &[])),
        [
            ("alice".to_owned(), "probe {\"patience\":7}".to_owned()),
            ("bob".to_owned(), "rule null".to_owned()),
            ("carol".to_owned(), "steady null".to_owned()),
            ("dev".to_owned(), "probe {\"patience\":7}".to_owned()),
        ],
        "alice and dev take the default with its block; bob his own entry over staff's; carol staff's"
    );
    let overridden = resolved(
        text,
        &[SeatOverride {
            seat: key("carol"),
            model: "probe".to_owned(),
        }],
    );
    assert_eq!(
        models_of(&overridden)[2],
        ("carol".to_owned(), "probe {\"patience\":7}".to_owned()),
        "an override takes the file's block for its model"
    );
    let none = DecisionList::<Model>::none()
        .resolve(&roster(), Origin::Implicit, &[])
        .expect("resolves");
    assert!(
        models_of(&none)
            .iter()
            .all(|(_, model)| model == "rule null"),
        "no list: every seat rule"
    );
    assert!(!none.source().in_effect());
    assert_eq!(none.clause("why"), None, "no list in effect, no clause");
}

/// SD-DLa-8: an external model resolves to the end of its chain, and `none` leaves the seat idle.
#[test]
fn an_external_model_runs_the_end_of_its_chain() {
    let text = "models:\n  lm: { fallback: far1 }\n  far1: { fallback: far2 }\n  far2: { fallback: \
                far3 }\n  far3: { fallback: probe }\n  far4: { fallback: none }\nassign:\n  - { seat: \
                alice, model: lm }\n  - { seat: bob, model: far4 }\n";
    let resolved = resolved(text, &[]);
    let alice = &resolved.seats()[0];
    assert_eq!(alice.assigned.as_ref().map(|id| id.as_str()), Some("lm"));
    assert_eq!(
        alice.chain,
        ["lm", "far1", "far2", "far3", "probe"],
        "four links: accepted"
    );
    assert_eq!(
        format!("{:?}", alice.drives),
        "Some(probe {\"patience\":3})"
    );
    let bob = &resolved.seats()[1];
    assert_eq!(bob.chain, ["far4", "none"]);
    assert!(
        bob.drives.is_none(),
        "a chain ending at none leaves the seat idle"
    );
    let clause = resolved
        .clause("run hosts no out-of-process model")
        .expect("in effect");
    assert!(
        clause.ends_with(
            "; alice: lm → far1 → far2 → far3 → probe (run hosts no out-of-process model); bob: far4 → \
             none (run hosts no out-of-process model)"
        ),
        "{clause}"
    );
    assert!(clause.contains("(1 probe, 1 idle, 2 rule)"), "{clause}");
}

/// SD-DLa-11: `--seat-model` refuses an unknown seat, an unknown model, an external model without a
/// fallback, and one seat twice — each naming the flag.
#[test]
fn an_override_is_refused_by_name() {
    let list = parse("models:\n  lm: { fallback: steady }\n").expect("valid");
    let refused = |overrides: &[(&str, &str)]| -> String {
        let overrides: Vec<SeatOverride> = overrides
            .iter()
            .map(|(seat, model)| SeatOverride {
                seat: key(seat),
                model: (*model).to_owned(),
            })
            .collect();
        list.resolve(&roster(), Origin::File, &overrides)
            .expect_err("refused")
    };
    let unknown_seat = refused(&[("zed", "steady")]);
    assert!(
        unknown_seat.starts_with("--seat-model zed=steady: this world offers no such seat. It offers: alice, bob, carol, dev"),
        "{unknown_seat}"
    );
    let unknown_model = refused(&[("alice", "mystery")]);
    assert!(
        unknown_model
            .contains("--seat-model alice=mystery: 'mystery' is not a model this build installs"),
        "{unknown_model}"
    );
    let unlisted = refused(&[("alice", "far1")]);
    assert!(
        unlisted
            .contains("--seat-model alice=far1: 'far1' runs out of process and has no fallback"),
        "{unlisted}"
    );
    let twice = refused(&[("alice", "steady"), ("alice", "probe")]);
    assert!(twice.contains("seat 'alice' is named twice"), "{twice}");
    let listed = list
        .resolve(
            &roster(),
            Origin::File,
            &[SeatOverride {
                seat: key("alice"),
                model: "lm".to_owned(),
            }],
        )
        .expect("an external model listed with its fallback is accepted");
    assert_eq!(listed.seats()[0].chain, ["lm", "steady"]);
}

/// AC-DLa-7: lists that resolve alike digest alike — `{}` and explicit defaults, entries in another
/// order, a class entry or the same per-seat entries — and a changed parameter changes the digest.
#[test]
fn the_digest_follows_the_resolution_not_the_text() {
    let digest = |text: &str| resolved(text, &[]).digest();
    let a = digest("models:\n  probe: {}\ndefault: probe\n");
    assert_eq!(
        a,
        digest("default: probe\nmodels:\n  probe:\n    parameters: { patience: 3 }\n")
    );
    assert_eq!(
        a,
        digest("default: probe\n"),
        "a model not listed runs with its defaults"
    );
    let by_class = digest("assign:\n  - { class: staff, model: steady }\n");
    let by_seat =
        digest("assign:\n  - { seat: carol, model: steady }\n  - { seat: bob, model: steady }\n");
    assert_eq!(by_class, by_seat);
    assert_ne!(
        a,
        digest("models:\n  probe:\n    parameters: { patience: 4 }\ndefault: probe\n")
    );
    let none = DecisionList::<Model>::none()
        .resolve(&roster(), Origin::Implicit, &[])
        .expect("resolves")
        .digest();
    assert_eq!(
        none,
        digest("default: rule\n"),
        "an all-rule list digests as no list"
    );
    assert_ne!(
        none,
        digest("memory:\n  assign:\n    - { seat: dev, policy: decay }\n"),
        "a memory: section enters the digest"
    );
    assert_eq!(
        digest("memory:\n  assign:\n    - { seat: dev, policy: decay }\n"),
        digest("\n\nmemory:\n  assign:\n\n    - { seat: dev, policy: decay }\n"),
        "positions do not"
    );
    assert_ne!(
        digest("memory:\n  assign:\n    - { seat: dev, policy: decay }\n"),
        digest("memory:\n  assign:\n    - { seat: dev, policy: decay, store_bound_mib: 40 }\n"),
        "a written store bound does (pr-fg-b2 §5)"
    );
}

/// QFGb-1: memory resolves seat > class > default and is passed through uninterpreted — an unknown
/// policy id, its fallback and parameters as written, with the winning entry's position.
#[test]
fn memory_is_resolved_per_seat_and_passed_through() {
    let text = "memory:\n  default: { policy: consolidate }\n  assign:\n    - { class: staff, policy: \
                decay, parameters: { half_life_hours: 336, curve: power, sharp: true, weights: [1, 2.5], \
                none: null } }\n    - { seat: bob, policy: nonexistent-policy, fallback: consolidate, \
                store_bound_mib: 40 }\n";
    let document = resolved(text, &[]).json("cafe");
    let memory: Vec<serde_json::Value> = document
        .seats
        .iter()
        .map(|seat| serde_json::to_value(&seat.memory).expect("serializes"))
        .collect();
    assert_eq!(
        memory[0],
        json!({ "policy": "consolidate", "fallback": null, "parameters": {},
                "store_bound_mib": null,
                "source": { "file": "decisions.yaml", "line": 2, "column": 12 } }),
        "alice: the default"
    );
    assert_eq!(
        memory[1],
        json!({ "policy": "nonexistent-policy", "fallback": "consolidate", "parameters": {},
                "store_bound_mib": 40,
                "source": { "file": "decisions.yaml", "line": 5, "column": 7 } }),
        "bob: his own entry, over staff's; an unknown policy id and his store bound pass through"
    );
    assert_eq!(
        memory[2],
        json!({ "policy": "decay", "fallback": null,
                "parameters": { "half_life_hours": 336, "curve": "power", "sharp": true,
                                "weights": [1, 2.5], "none": null },
                "store_bound_mib": null,
                "source": { "file": "decisions.yaml", "line": 4, "column": 7 } }),
        "carol: staff's, its parameters YAML scalars as JSON scalars"
    );
    let no_default = resolved(
        "memory:\n  assign:\n    - { seat: bob, policy: decay }\n",
        &[],
    )
    .json("cafe");
    assert!(
        no_default.seats[0].memory.is_none(),
        "no entry and no default: null"
    );
    let without = resolved("default: rule\n", &[]).json("cafe");
    assert!(
        without.seats.iter().all(|seat| seat.memory.is_none()),
        "no memory: section: null"
    );
}

/// SD-DLa-15: the document's shape and key order, and its round trip through the typed structs.
#[test]
fn the_json_document_has_a_fixed_shape_and_round_trips() {
    let text = "models:\n  lm: { fallback: none }\nassign:\n  - { seat: alice, model: lm }\n";
    let document = resolved(text, &[]).json("cafe");
    let written = serde_json::to_string(&document).expect("serializes");
    let digest = &document.digest;
    assert_eq!(digest.len(), 16);
    assert_eq!(
        written,
        format!(
            "{{\"format\":2,\"world\":\"cafe\",\"digest\":\"{digest}\",\"seats\":[\
             {{\"seat\":\"alice\",\"model\":{{\"id\":null,\"assigned\":\"lm\",\"chain\":[\"lm\",\"none\"]}},\"memory\":null}},\
             {{\"seat\":\"bob\",\"model\":{{\"id\":\"rule\"}},\"memory\":null}},\
             {{\"seat\":\"carol\",\"model\":{{\"id\":\"rule\"}},\"memory\":null}},\
             {{\"seat\":\"dev\",\"model\":{{\"id\":\"rule\"}},\"memory\":null}}]}}"
        )
    );
    let back: ResolvedDecisionsJson = serde_json::from_str(&written).expect("round trips");
    assert_eq!(back, document);
}

/// The catalog the macro writes: listed order, kinds, packages, defaults.
#[test]
fn the_catalog_lists_in_order_and_knows_each_kind() {
    let ids: Vec<&str> = Model::AVAILABLE.iter().map(|model| model.id()).collect();
    assert_eq!(
        ids,
        [
            "rule", "steady", "probe", "lm", "far1", "far2", "far3", "far4"
        ]
    );
    assert!(Model::find("lm").is_some_and(Catalog::is_external));
    assert!(Model::find("probe").is_some_and(|model| !model.is_external()));
    assert_eq!(Model::find("lm").and_then(Catalog::defaults), None);
    assert_eq!(
        Model::find("probe").and_then(Catalog::type_name),
        Some("mineworld_decision::read_tests::Probe")
    );
}
