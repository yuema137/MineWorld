//! AC-j-2 revision A: deterministic independent draws, not a finite-sample quota.
use std::collections::{BTreeMap, BTreeSet};

use mineworld_attributes_model::profile::{
    Adjust, Covers, Distribution, Mask, Profile, Rows, Shape,
};
use mineworld_attributes_model::sample::{
    Fixed, FixedBy, Newcomer, Origin, Resident, Words, sample_newcomer,
};
use mineworld_attributes_model::{
    AttributeValue, Catalogue, DataFile, QualifiedId, Restriction, ScaledInt, Traits, ValueSlug,
};
use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};
use mineworld_contracts::{EventId, Tags, WorldTime};

const SEED: &str = "independent-contract-v1";
// Independently evaluated FNV-1a/SplitMix64 vectors; derivation is retained in the PR evidence.
const VECTORS: [(&str, u64, i64, u64, i64); 4] = [
    (
        "alpha",
        6_392_475_687_494_453_744,
        346_536,
        25_613_028_571_060_416,
        1_388,
    ),
    (
        "beta",
        15_493_963_867_027_636_056,
        839_929,
        17_776_683_587_060_003_643,
        963_675,
    ),
    (
        "gamma",
        13_571_761_069_762_628_465,
        735_726,
        7_860_676_389_489_779_799,
        426_128,
    ),
    (
        "delta",
        13_109_769_343_693_117_701,
        710_682,
        15_832_175_594_541_766_627,
        858_263,
    ),
];

fn id(name: &str) -> QualifiedId {
    QualifiedId::new(&format!("oracle.{name}")).unwrap()
}
fn slug(name: &str) -> ValueSlug {
    ValueSlug::new(name).unwrap()
}
fn category(name: &str) -> AttributeValue {
    AttributeValue::Category(slug(name))
}

fn catalogue(external: bool) -> Catalogue {
    let owner = if external { "oracle" } else { "attributes" };
    let text = format!(
        r#"format: 1
attributes:
  parent:
    group: background
    kind: {{ category: [p, q] }}
    visibility: private
    mutability: fixed
    label: Parent
  choice:
    group: background
    kind: {{ category: [a, b, c] }}
    visibility: private
    mutability: fixed
    owner: {owner}
    label: Choice
  amount:
    group: background
    kind: {{ quantity: {{ unit: count, scale: 0, min: 0, max: 999999 }} }}
    visibility: private
    mutability: fixed
    owner: {owner}
    label: Amount
"#
    );
    Catalogue::build(&[(
        DataOrigin {
            pack: "oracle".into(),
            source: DataSource::SystemPack,
            directory: DataDirectory::from_static("attributes"),
            path: "attributes/oracle.yaml".into(),
        },
        DataFile::Attributes(serde_saphyr::from_str(&text).unwrap()),
    )])
    .unwrap()
}

fn profile(external: bool) -> Profile {
    let when = BTreeMap::from([(id("parent"), BTreeSet::from([slug("p")]))]);
    let categorical = Distribution {
        given: vec![id("parent")],
        shape: Shape::Prior {
            rows: Rows::Node {
                by: BTreeMap::from([
                    (slug("p"), Rows::Leaf(vec![100_000, 600_000, 300_000])),
                    (slug("q"), Rows::Leaf(vec![600_000, 100_000, 300_000])),
                ]),
                otherwise: None,
            },
            adjust: vec![Adjust {
                when: when.clone(),
                ratios: BTreeMap::from([(0, 4_000_000)]),
            }],
            mask: vec![Mask {
                when,
                only: BTreeSet::from([0, 1]),
            }],
        },
        // External ownership must force independence even with this flag enabled.
        stratify: external,
        source: "authored".into(),
        verify: None,
    };
    let numeric = Distribution {
        given: vec![],
        shape: Shape::Uniform {
            min: 0,
            max: 999_999,
        },
        stratify: external,
        source: "authored".into(),
        verify: None,
    };
    Profile {
        id: "oracle".into(),
        version: 1,
        covers: Covers::All,
        sources: BTreeMap::new(),
        distributions: BTreeMap::from([(id("choice"), categorical), (id("amount"), numeric)]),
        roles: vec![],
        order: vec![id("choice"), id("amount")],
    }
}

fn fixed(parent: &str) -> Fixed {
    Fixed {
        values: BTreeMap::from([(id("parent"), (category(parent), FixedBy::Authored))]),
        ..Fixed::default()
    }
}

fn draw(
    catalogue: &Catalogue,
    profile: &Profile,
    key: &str,
    fixed: &Fixed,
    residents: &[Resident<'_>],
) -> Result<BTreeMap<QualifiedId, (AttributeValue, Origin)>, String> {
    sample_newcomer(
        catalogue,
        Some(profile),
        Words::new(SEED, 0),
        Newcomer {
            key,
            tags: &Tags::default(),
            fixed,
        },
        residents,
        WorldTime::from_seconds(60),
        EventId::from_raw(7),
    )
    .map(|arrival| arrival.values)
    .map_err(|error| error.to_string())
}

#[test]
fn independent_rows_masks_adjustments_and_uniforms_match_known_vectors() {
    for external in [false, true] {
        let catalogue = catalogue(external);
        let profile = profile(external);
        for (key, categorical_word, point, numeric_word, amount) in VECTORS {
            assert_eq!(
                Words::new(SEED, 0).draw(&id("choice"), key, 0),
                categorical_word
            );
            assert_eq!(
                Words::new(SEED, 0).draw(&id("amount"), key, 0),
                numeric_word
            );
            for parent in ["p", "q"] {
                let values = draw(&catalogue, &profile, key, &fixed(parent), &[]).unwrap();
                // p: [0.1,0.6,0.3] -> adjust a*4 -> mask c -> [0.4,0.6,0].
                // Without adjustment alpha becomes b; without masking beta becomes c.
                // Integer shares are [400000,600000,0]; q remains [600000,100000,300000].
                let expected = if parent == "p" {
                    if point < 400_000 { "a" } else { "b" }
                } else if point < 600_000 {
                    "a"
                } else if point < 700_000 {
                    "b"
                } else {
                    "c"
                };
                assert_eq!(
                    values[&id("choice")].0,
                    category(expected),
                    "{external} {key} {parent}"
                );
                // Uniform [0,999999] at the midpoint of ppm bucket k maps exactly to k.
                assert_eq!(
                    values[&id("amount")].0,
                    AttributeValue::Quantity(ScaledInt::new(amount, 0))
                );
                assert!((0..=999_999).contains(&amount));
            }
        }
    }
}

#[test]
fn cumulative_boundary_is_exclusive_at_the_integer_point() {
    for external in [false, true] {
        let catalogue = catalogue(external);
        for (threshold, expected) in [(346_535, "b"), (346_536, "b"), (346_537, "a")] {
            let mut profile = profile(external);
            profile.distributions.get_mut(&id("choice")).unwrap().shape = Shape::Prior {
                rows: Rows::Leaf(vec![threshold, 1_000_000 - threshold, 0]),
                adjust: vec![],
                mask: vec![],
            };
            let values = draw(&catalogue, &profile, "alpha", &fixed("p"), &[]).unwrap();
            assert_eq!(
                values[&id("choice")].0,
                category(expected),
                "{external} threshold{threshold}"
            );
        }
    }
}

#[test]
fn changing_or_reordering_residents_cannot_change_independent_draws() {
    for external in [false, true] {
        let catalogue = catalogue(external);
        let profile = profile(external);
        let resident = |parent, choice, amount| {
            let mut values = vec![(id("parent"), category(parent))];
            if !external {
                values.extend([
                    (id("choice"), category(choice)),
                    (
                        id("amount"),
                        AttributeValue::Quantity(ScaledInt::new(amount, 0)),
                    ),
                ]);
            }
            // Never fabricate external-owner Traits to simulate foreign state.
            Traits::stated(&catalogue, values).unwrap()
        };
        let left = resident("p", "a", 0);
        let right = resident("q", "b", 999_999);
        let residents = [
            Resident {
                key: "resident-a",
                traits: &left,
            },
            Resident {
                key: "resident-b",
                traits: &right,
            },
        ];
        let reversed = [residents[1], residents[0]];
        let replaced = [Resident {
            key: "replacement",
            traits: &right,
        }; 20];
        for (key, ..) in VECTORS {
            let fixed = fixed("p");
            let baseline = draw(&catalogue, &profile, key, &fixed, &[]).unwrap();
            for population in [&residents[..], &reversed[..], &replaced[..]] {
                assert_eq!(
                    draw(&catalogue, &profile, key, &fixed, population).unwrap(),
                    baseline,
                    "external={external} key={key}"
                );
            }
        }
    }
}

#[test]
fn restrictions_fixed_origins_and_empty_support_remain_hard_constraints() {
    for external in [false, true] {
        let catalogue = catalogue(external);
        let profile = profile(external);
        let mut restricted = fixed("p");
        restricted.restrictions.insert(
            id("choice"),
            Restriction::Allowed(BTreeSet::from([slug("b"), slug("c")])),
        );
        for (key, ..) in VECTORS {
            let values = draw(&catalogue, &profile, key, &restricted, &[]).unwrap();
            assert_eq!(values[&id("choice")].0, category("b")); // c is masked out.
        }
        restricted.restrictions.insert(
            id("choice"),
            Restriction::Allowed(BTreeSet::from([slug("c")])),
        );
        assert!(
            draw(&catalogue, &profile, "alpha", &restricted, &[])
                .unwrap_err()
                .contains("Unsatisfiable")
        );
        for (by, origin) in [
            (FixedBy::Authored, Origin::Authored),
            (FixedBy::Locked, Origin::Locked),
        ] {
            let mut fixed = fixed("p");
            fixed.values.insert(id("choice"), (category("b"), by));
            fixed.values.insert(
                id("amount"),
                (AttributeValue::Quantity(ScaledInt::new(42, 0)), by),
            );
            let values = draw(&catalogue, &profile, "alpha", &fixed, &[]).unwrap();
            assert_eq!(values[&id("choice")], (category("b"), origin.clone()));
            assert_eq!(
                values[&id("amount")],
                (AttributeValue::Quantity(ScaledInt::new(42, 0)), origin)
            );
        }
    }
}
