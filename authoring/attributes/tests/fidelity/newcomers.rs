//! Sequential-arrival fidelity; expectations come from profile rows, not a second sampler run.
use super::*;
use mineworld_attributes_model::profile::{Profile, Shape};
use mineworld_attributes_model::sample::{Fixed, Newcomer, Resident, sample_newcomer};
use mineworld_attributes_model::{AttributesContext, Mutability, Traits, derive};
use mineworld_contracts::{EventId, WorldTime};

const ARRIVALS: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/cli/tests/fixtures/attributes-newcomers"
);

fn fixture() -> (Catalogue, Profile) {
    let file = serde_saphyr::from_str(
        &std::fs::read_to_string(format!("{ARRIVALS}/attributes/people.yaml")).unwrap(),
    )
    .unwrap();
    let catalogue = Catalogue::build(&[(
        DataOrigin {
            pack: "attributes-newcomers".into(),
            source: DataSource::World,
            directory: DataDirectory::from_static("attributes"),
            path: "attributes/people.yaml".into(),
        },
        DataFile::Attributes(file),
    )])
    .unwrap();
    let config: AttributesConfiguration = serde_saphyr::from_str(
        &std::fs::read_to_string(format!("{ARRIVALS}/configure/attributes.yaml")).unwrap(),
    )
    .unwrap();
    let file = serde_saphyr::from_str(
        &std::fs::read_to_string(format!("{ARRIVALS}/populations/test-town.yaml")).unwrap(),
    )
    .unwrap();
    let profile = build(
        &catalogue,
        &[("populations/test-town.yaml".into(), file)],
        Some(&config),
    )
    .unwrap()
    .unwrap();
    (catalogue, profile)
}

fn builtin() -> (Catalogue, Profile) {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../entities");
    let mut files = Vec::new();
    for pack in ["person-attributes", "populations-us"] {
        for dir in [
            "attributes",
            "vocabularies",
            "populations",
            "backends",
            "masks",
        ] {
            let path = root.join(pack).join(dir);
            if !path.is_dir() {
                continue;
            }
            let mut paths: Vec<_> = std::fs::read_dir(path)
                .unwrap()
                .map(|p| p.unwrap().path())
                .collect();
            paths.sort();
            for path in paths {
                if path.extension().is_none_or(|ext| ext != "yaml") {
                    continue;
                }
                let text = std::fs::read_to_string(&path).unwrap();
                let data = match dir {
                    "attributes" => DataFile::Attributes(serde_saphyr::from_str(&text).unwrap()),
                    "vocabularies" => {
                        DataFile::Vocabularies(serde_saphyr::from_str(&text).unwrap())
                    }
                    "populations" => {
                        DataFile::Profile(Box::new(serde_saphyr::from_str(&text).unwrap()))
                    }
                    "backends" => {
                        DataFile::Backend(Box::new(serde_saphyr::from_str(&text).unwrap()))
                    }
                    "masks" => DataFile::Mask(Box::new(serde_saphyr::from_str(&text).unwrap())),
                    _ => unreachable!(),
                };
                files.push((
                    DataOrigin {
                        pack: pack.to_owned(),
                        source: DataSource::EntityPack,
                        directory: DataDirectory::from_static(dir),
                        path,
                    },
                    data,
                ));
            }
        }
    }
    let config =
        serde_saphyr::from_str("format: 1\nprofile: san-diego-county\nseed: fidelity\n").unwrap();
    let context = AttributesContext::build(files, Some(&config)).unwrap();
    (
        context.catalogue,
        context.population.unwrap().profile.unwrap(),
    )
}
fn traits(
    catalogue: &Catalogue,
    values: BTreeMap<QualifiedId, (AttributeValue, mineworld_attributes_model::sample::Origin)>,
) -> Traits {
    Traits::stated(
        catalogue,
        values
            .into_iter()
            .filter(|(id, _)| catalogue.get(id).unwrap().mutability != Mutability::Derived)
            .map(|(id, (value, _))| (id, value)),
    )
    .unwrap()
}
fn value(
    catalogue: &Catalogue,
    traits: &Traits,
    id: &QualifiedId,
    at: WorldTime,
) -> Option<AttributeValue> {
    traits
        .value(id)
        .cloned()
        .or_else(|| derive::value(catalogue, catalogue.get(id)?, traits, at))
}
fn run(catalogue: &Catalogue, profile: &Profile, initial: usize) -> Vec<(String, Traits)> {
    let tags = Tags::default();
    let keys: Vec<_> = if initial == 8 {
        [
            "dev", "erin", "felix", "grace", "hana", "ivan", "jun", "kim",
        ]
        .map(str::to_owned)
        .to_vec()
    } else {
        (0..initial).map(|n| format!("synthetic-{n:05}")).collect()
    };
    #[derive(serde::Deserialize)]
    struct Person {
        #[serde(default)]
        tags: Tags,
    }
    let genesis_tags: Vec<_> = keys
        .iter()
        .map(|key| {
            if initial == 8 {
                let person: Person = serde_saphyr::from_str(
                    &std::fs::read_to_string(format!("{ARRIVALS}/people/{key}.yaml")).unwrap(),
                )
                .unwrap();
                person.tags
            } else {
                Tags::default()
            }
        })
        .collect();
    let seed = if initial == 8 {
        "fixture-pop-v1"
    } else {
        "fidelity"
    };
    let members: Vec<_> = keys
        .iter()
        .zip(&genesis_tags)
        .map(|(key, tags)| Member {
            key,
            tags,
            authored: None,
        })
        .collect();
    let genesis = resolve(
        catalogue,
        Some(profile),
        None,
        Words::new(seed, 0),
        &members,
    )
    .unwrap();
    let mut population: Vec<_> = genesis
        .people
        .into_iter()
        .map(|(key, values)| (key, traits(catalogue, values)))
        .collect();
    for n in 0..1000 {
        let residents: Vec<_> = population
            .iter()
            .map(|(key, traits)| Resident { key, traits })
            .collect();
        let key = format!("newcomer-{n:05}");
        let drawn = sample_newcomer(
            catalogue,
            Some(profile),
            Words::new(seed, 0),
            Newcomer {
                key: &key,
                tags: &tags,
                fixed: &Fixed::default(),
            },
            &residents,
            WorldTime::from_seconds(n * 60),
            EventId::from_raw(n as u64 + 1),
        )
        .unwrap();
        population.push((key, traits(catalogue, drawn.values)));
    }
    population
}

fn categorical_fidelity(
    catalogue: &Catalogue,
    profile: &Profile,
    population: &[(String, Traits)],
    failures: &mut Vec<String>,
) {
    let at = WorldTime::from_seconds(999 * 60);
    for (id, distribution) in &profile.distributions {
        let eligible = distribution.stratify
            && catalogue.get(id).unwrap().owner == mineworld_attributes_model::ATTRIBUTES;
        let Shape::Prior { rows, adjust, mask } = &distribution.shape else {
            continue;
        };
        let domain = catalogue.get(id).unwrap().kind.values().unwrap();
        let mut groups: BTreeMap<Vec<ValueSlug>, Vec<u64>> = BTreeMap::new();
        for (_, traits) in population {
            let parents: Option<Vec<_>> = distribution
                .given
                .iter()
                .map(|id| {
                    value(catalogue, traits, id, at).and_then(|v| ValueSlug::new(&slug_of(&v)).ok())
                })
                .collect();
            let Some(parents) = parents else {
                continue;
            };
            let Some(v) = value(catalogue, traits, id, at) else {
                continue;
            };
            let index = domain
                .iter()
                .position(|s| s.as_str() == slug_of(&v))
                .unwrap();
            groups.entry(parents).or_insert(vec![0; domain.len()])[index] += 1;
        }
        let mut actual = vec![0i128; domain.len()];
        let mut expected = vec![0i128; domain.len()];
        let mut total = 0i128;
        for (parents, counts) in groups {
            let matches = |when: &BTreeMap<QualifiedId, std::collections::BTreeSet<ValueSlug>>| {
                when.iter().all(|(id, wanted)| {
                    distribution
                        .given
                        .iter()
                        .position(|p| p == id)
                        .is_some_and(|i| wanted.contains(&parents[i]))
                })
            };
            let mut weights: Vec<i128> = rows
                .row(&parents)
                .unwrap()
                .iter()
                .map(|w| i128::from(*w))
                .collect();
            for entry in adjust.iter().filter(|a| matches(&a.when)) {
                for (i, ratio) in &entry.ratios {
                    weights[*i] = weights[*i] * i128::from(*ratio) / 1_000_000;
                }
            }
            for entry in mask.iter().filter(|m| matches(&m.when)) {
                for (i, w) in weights.iter_mut().enumerate() {
                    if !entry.only.contains(&i) {
                        *w = 0;
                    }
                }
            }
            let sum: i128 = weights.iter().sum();
            let n: i128 = counts.iter().map(|n| i128::from(*n)).sum();
            total += n;
            for i in 0..counts.len() {
                let target = n * weights[i] * 1_000_000 / sum;
                let count = i128::from(counts[i]) * 1_000_000;
                println!(
                    "newcomer conditional {} {id} eligible={eligible} {parents:?} {} actual_micropersons{count} expected_micropersons{target} N{n}",
                    profile.id, domain[i]
                );
                if eligible && n >= 50 && (count - target).abs() * 100 > n * 2 * 1_000_000 {
                    failures.push(format!(
                        "{} {id} stratify={} {parents:?} {} count{} N{n} expected_micropersons{target} threshold=2pp",
                        profile.id,
                        distribution.stratify,
                        domain[i],
                        counts[i]
                    ));
                }
                actual[i] += count;
                expected[i] += target;
            }
        }
        for i in 0..domain.len() {
            println!(
                "newcomer marginal {} {id} eligible={eligible} {} actual_micropersons{} expected_micropersons{} N{total}",
                profile.id, domain[i], actual[i], expected[i]
            );
            if eligible && (actual[i] - expected[i]).abs() * 200 > total * 1_000_000 {
                failures.push(format!(
                "{} {id} stratify={} {} marginal actual_micropersons{} expected_micropersons{} N{total} threshold=0.5pp",
                profile.id,
                distribution.stratify,
                domain[i],
                actual[i],
                expected[i]
                ));
            }
        }
    }
}

#[test]
fn san_diego_runtime_state_sizes_are_reported_separately() {
    use mineworld_attributes_model::newcomer::{NewcomerContext, Newcomers};
    use mineworld_contracts::EntityKey;
    let (_, profile) = builtin();
    let profile_bytes = serde_json::to_vec(&profile).unwrap().len();
    let context = NewcomerContext {
        profile: Some(profile),
        seed: "fidelity".into(),
        salt: 0,
        deferred: (0..1000)
            .map(|n| {
                (
                    EntityKey::new(format!("newcomer-{n:05}")).unwrap(),
                    Fixed::default(),
                )
            })
            .collect(),
    };
    let context_bytes = serde_json::to_vec(&context).unwrap().len();
    let mut process = Newcomers::new(context);
    process.arrived = process.context.deferred.keys().cloned().collect();
    println!(
        "San Diego bytes profile={profile_bytes} context={context_bytes} arrived={} process={}",
        serde_json::to_vec(&process.arrived).unwrap().len(),
        serde_json::to_vec(&process).unwrap().len()
    );
    assert!(
        profile_bytes <= 65_536,
        "R-j-1 profile exceeds 64 KiB: {profile_bytes}"
    );
}

#[test]
fn a_thousand_arrivals_preserve_both_profiles_and_reproduce() {
    let (fixture, fixture_profile) = fixture();
    let mut failures = Vec::new();
    for (catalogue, profile, initial) in [(fixture, fixture_profile, 8), {
        let (c, p) = builtin();
        (c, p, 200)
    }] {
        let first = run(&catalogue, &profile, initial);
        categorical_fidelity(&catalogue, &profile, &first, &mut failures);
        deciles::check(&catalogue, &profile, &first, &mut failures);
        let second = run(&catalogue, &profile, initial);
        assert_eq!(first, second);
        let bytes = serde_json::to_vec(&first).unwrap();
        println!(
            "newcomer digest {} {:016x}",
            profile.id,
            mineworld_attributes_model::sample::hash::fnv1a64(&bytes)
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
