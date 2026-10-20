//! AC-S30-5 and AC-S30-6 (`pr-s30d-sampler.md` AC-d-4, AC-d-5): synthetic populations of the fixture
//! profile match its marginals and conditionals, and no masked value is ever drawn.

use std::collections::BTreeMap;
use std::path::PathBuf;

use mineworld_attributes_model::config::AttributesConfiguration;
use mineworld_attributes_model::declaration::AttributesFile;
use mineworld_attributes_model::profile::{ProfileFile, build};
use mineworld_attributes_model::sample::{Member, Words, resolve};
use mineworld_attributes_model::{AttributeValue, Catalogue, DataFile, QualifiedId, ValueSlug};
use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};
use mineworld_contracts::Tags;

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/cli/tests/fixtures/attributes-sampled"
);

fn catalogue() -> Catalogue {
    let text =
        std::fs::read_to_string(format!("{FIXTURE}/attributes/people.yaml")).expect("readable");
    let file: AttributesFile = serde_saphyr::from_str(&text).expect("decodes");
    let origin = DataOrigin {
        pack: "attributes-sampled".to_owned(),
        source: DataSource::World,
        directory: DataDirectory::from_static("attributes"),
        path: PathBuf::from("attributes/people.yaml"),
    };
    Catalogue::build(&[(origin, DataFile::Attributes(file))]).expect("builds")
}

fn profile(catalogue: &Catalogue) -> mineworld_attributes_model::profile::Profile {
    let text =
        std::fs::read_to_string(format!("{FIXTURE}/populations/test-town.yaml")).expect("readable");
    let file: ProfileFile = serde_saphyr::from_str(&text).expect("decodes");
    let configuration = AttributesConfiguration {
        format: 1,
        profile: Some("test-town".to_owned()),
        seed: Some("fidelity".to_owned()),
        salt: 0,
        lock: None,
        backend: None,
        mask: None,
    };
    build(
        catalogue,
        &[(PathBuf::from("populations/test-town.yaml"), file)],
        Some(&configuration),
    )
    .expect("the profile builds")
    .expect("it is named")
}

fn id(text: &str) -> QualifiedId {
    QualifiedId::new(text).expect("an id")
}

fn slug_of(value: &AttributeValue) -> String {
    match value {
        AttributeValue::Category(slug) => slug.to_string(),
        AttributeValue::Level(level) => level.slug.to_string(),
        other => other.display(),
    }
}

/// Counts of each value of `attribute` among `n` synthetic people, and the counts per `given` value.
fn counts(
    n: usize,
    attribute: &str,
    given: Option<&str>,
) -> (BTreeMap<String, u64>, BTreeMap<(String, String), u64>) {
    let catalogue = catalogue();
    let profile = profile(&catalogue);
    let keys: Vec<String> = (1..=n)
        .map(|index| format!("synthetic-{index:05}"))
        .collect();
    let tags = Tags::default();
    let members: Vec<Member<'_>> = keys
        .iter()
        .map(|key| Member {
            key,
            tags: &tags,
            authored: None,
        })
        .collect();
    let resolution = resolve(
        &catalogue,
        Some(&profile),
        None,
        Words::new("fidelity", 0),
        &members,
    )
    .expect("resolves");
    let mut marginal = BTreeMap::new();
    let mut conditional = BTreeMap::new();
    for values in resolution.people.values() {
        let Some((value, _)) = values.get(&id(attribute)) else {
            continue;
        };
        *marginal.entry(slug_of(value)).or_insert(0) += 1;
        if let Some(given) = given
            && let Some((parent, _)) = values.get(&id(given))
        {
            *conditional
                .entry((slug_of(parent), slug_of(value)))
                .or_insert(0) += 1;
        }
    }
    (marginal, conditional)
}

/// AC-S30-5: 1 000 people; stratified marginals within 0.5 pp of the prior.
#[test]
fn a_thousand_people_match_the_profiles_marginals() {
    let (sex, _) = counts(1000, "attributes-sampled.sex", None);
    for value in ["female", "male"] {
        let count = sex.get(value).copied().unwrap_or(0);
        assert!((495..=505).contains(&count), "{value}: {count} of 1000");
    }
    let (chronotype, _) = counts(1000, "attributes-sampled.chronotype", None);
    for (value, target) in [("early-bird", 250), ("neutral", 500), ("night-owl", 250)] {
        let count = chronotype.get(value).copied().unwrap_or(0);
        assert!(
            count.abs_diff(target) <= 5,
            "{value}: {count}, target {target}"
        );
    }
}

/// AC-S30-6: 10 000 people; no female has a beard or stubble under the fixture's mask.
#[test]
fn no_masked_value_is_drawn_in_ten_thousand() {
    let (_, by_sex) = counts(
        10_000,
        "attributes-sampled.facial-hair",
        Some("attributes-sampled.sex"),
    );
    for value in ["beard", "stubble"] {
        assert_eq!(
            by_sex.get(&("female".to_owned(), value.to_owned())),
            None,
            "{value}"
        );
    }
    let males: u64 = by_sex
        .iter()
        .filter(|((sex, _), _)| sex == "male")
        .map(|(_, count)| count)
        .sum();
    assert!(males > 4000);
    let _ = ValueSlug::new("none");
}
