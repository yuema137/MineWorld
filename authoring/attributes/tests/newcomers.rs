//! `sample_newcomer` on hand-built inputs (`pr-s30j-runtime-sampling.md` §§5.4, 5.9): fixed values kept, a
//! mask held, a restriction respected, `Unsatisfiable`, a birth relative to the instant of arrival,
//! `NotCovered` at arrival, the deficit rule, and a draw that never depends on the admission's cause.

use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use mineworld_attributes_model::config::AttributesConfiguration;
use mineworld_attributes_model::declaration::AttributesFile;
use mineworld_attributes_model::profile::{Covers, Profile, ProfileFile, build};
use mineworld_attributes_model::sample::{
    Fixed, FixedBy, Newcomer, Origin, Resident, Words, sample_newcomer,
};
use mineworld_attributes_model::{
    AttributeValue, Catalogue, DataFile, LevelValue, QualifiedId, Restriction, ScaledInt, Traits,
    ValueSlug,
};
use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};
use mineworld_contracts::{EventId, Tags, WorldTime};

const FIXTURE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tools/cli/tests/fixtures/attributes-sampled"
);

/// One mean Gregorian year in seconds.
const YEAR: i64 = 146_097 * 86_400 / 400;

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

fn profile(catalogue: &Catalogue) -> Profile {
    let text =
        std::fs::read_to_string(format!("{FIXTURE}/populations/test-town.yaml")).expect("readable");
    let file: ProfileFile = serde_saphyr::from_str(&text).expect("decodes");
    let configuration = AttributesConfiguration {
        format: 1,
        profile: Some("test-town".to_owned()),
        seed: Some("newcomers".to_owned()),
        salt: 0,
        lock: None,
        backend: None,
        mask: None,
        newcomers: None,
    };
    build(
        catalogue,
        &[(PathBuf::from("populations/test-town.yaml"), file)],
        Some(&configuration),
    )
    .expect("the profile builds")
    .expect("it is named")
}

fn id(name: &str) -> QualifiedId {
    QualifiedId::new(&format!("attributes-sampled.{name}")).expect("an id")
}

fn slug(text: &str) -> ValueSlug {
    ValueSlug::new(text).expect("a slug")
}

fn chronotype(name: &str, rank: u16) -> AttributeValue {
    AttributeValue::Level(LevelValue {
        slug: slug(name),
        rank,
    })
}

struct Bench {
    catalogue: Catalogue,
    profile: Profile,
    tags: Tags,
}

impl Bench {
    fn new() -> Self {
        let catalogue = catalogue();
        let profile = profile(&catalogue);
        Self {
            catalogue,
            profile,
            tags: Tags::default(),
        }
    }

    fn draw(
        &self,
        key: &str,
        fixed: &Fixed,
        population: &[(String, Traits)],
        at: WorldTime,
        because: u64,
    ) -> Result<BTreeMap<QualifiedId, (AttributeValue, Origin)>, String> {
        let residents: Vec<Resident<'_>> = population
            .iter()
            .map(|(key, traits)| Resident { key, traits })
            .collect();
        sample_newcomer(
            &self.catalogue,
            Some(&self.profile),
            Words::new("newcomers", 0),
            Newcomer {
                key,
                tags: &self.tags,
                fixed,
            },
            &residents,
            at,
            EventId::from_raw(because),
        )
        .map(|arrival| arrival.values)
        .map_err(|error| error.to_string())
    }

    fn traits(&self, values: Vec<(&str, AttributeValue)>) -> Traits {
        Traits::stated(
            &self.catalogue,
            values.into_iter().map(|(name, value)| (id(name), value)),
        )
        .expect("plain traits")
    }
}

fn fixed(values: Vec<(&str, AttributeValue)>, restrictions: Vec<(&str, Restriction)>) -> Fixed {
    Fixed {
        values: values
            .into_iter()
            .map(|(name, value)| (id(name), (value, FixedBy::Authored)))
            .collect(),
        restrictions: restrictions
            .into_iter()
            .map(|(name, restriction)| (id(name), restriction))
            .collect(),
    }
}

fn category(text: &str) -> AttributeValue {
    AttributeValue::Category(slug(text))
}

/// I-S30-7: a fixed value is kept, with its origin; the first newcomer into an empty world draws by
/// weight and every plain attribute the profile draws is present.
#[test]
fn a_fixed_value_is_kept_and_an_empty_world_draws_by_weight() {
    let bench = Bench::new();
    let stature = AttributeValue::Quantity(ScaledInt::new(1_660, 3));
    let held = fixed(
        vec![("sex", category("female")), ("stature", stature.clone())],
        vec![],
    );
    let values = bench
        .draw("nadia", &held, &[], WorldTime::EPOCH, 1)
        .expect("draws");
    assert_eq!(values[&id("sex")], (category("female"), Origin::Authored));
    assert_eq!(values[&id("stature")], (stature, Origin::Authored));
    for name in ["birth", "hair-colour", "facial-hair", "chronotype"] {
        assert!(values.contains_key(&id(name)), "{name}: {values:?}");
    }
    assert!(
        !values.contains_key(&id("age-band")),
        "derived values are never stored"
    );
}

/// AC-S30-6 at arrival: no female newcomer is ever given a masked value; a restriction is respected;
/// a restriction whose every value is masked is `Unsatisfiable`, naming the person and the cause.
#[test]
fn a_mask_and_a_restriction_hold_and_an_impossible_restriction_is_refused() {
    let bench = Bench::new();
    let female = fixed(vec![("sex", category("female"))], vec![]);
    for index in 0..200 {
        let values = bench
            .draw(&format!("f-{index}"), &female, &[], WorldTime::EPOCH, 1)
            .expect("draws");
        assert_eq!(values[&id("facial-hair")].0, category("none"), "f-{index}");
    }
    let blond = fixed(
        vec![],
        vec![(
            "hair-colour",
            Restriction::Allowed(BTreeSet::from([slug("blond")])),
        )],
    );
    for index in 0..50 {
        let values = bench
            .draw(&format!("b-{index}"), &blond, &[], WorldTime::EPOCH, 1)
            .expect("draws");
        assert_eq!(values[&id("hair-colour")].0, category("blond"));
    }
    let bearded_woman = fixed(
        vec![("sex", category("female"))],
        vec![(
            "facial-hair",
            Restriction::Allowed(BTreeSet::from([slug("beard")])),
        )],
    );
    let refused = bench
        .draw("pia", &bearded_woman, &[], WorldTime::EPOCH, 7)
        .expect_err("no beard for a female under the mask");
    assert!(refused.contains("Unsatisfiable"), "{refused}");
    assert!(refused.contains("pia"), "{refused}");
    assert!(refused.contains("event 7"), "{refused}");
}

/// The deficit rule (§5.4 step 3), hand-computed: one early bird and two neutrals, n = 3; the deficits
/// are early 4·250 000 − 1 000 000 = 0, neutral 4·500 000 − 2 000 000 = 0, night-owl 4·250 000 = 1 000 000,
/// summing to exactly 10⁶, so every newcomer is a night owl whatever its word (M-j-1).
#[test]
fn the_value_the_population_lacks_is_the_one_drawn() {
    let bench = Bench::new();
    let population = vec![
        (
            "r1".to_owned(),
            bench.traits(vec![("chronotype", chronotype("early-bird", 0))]),
        ),
        (
            "r2".to_owned(),
            bench.traits(vec![("chronotype", chronotype("neutral", 1))]),
        ),
        (
            "r3".to_owned(),
            bench.traits(vec![("chronotype", chronotype("neutral", 1))]),
        ),
    ];
    for index in 0..100 {
        let values = bench
            .draw(
                &format!("n-{index}"),
                &Fixed::default(),
                &population,
                WorldTime::EPOCH,
                1,
            )
            .expect("draws");
        let (value, origin) = &values[&id("chronotype")];
        assert_eq!(*value, chronotype("night-owl", 2), "n-{index}");
        assert_eq!(
            *origin,
            Origin::Sampled {
                stratum: String::new(),
                draw: 3
            }
        );
    }
}

/// D-j-5 (M-j-2): a newcomer arriving forty years after the epoch is born within its band's ages of the
/// arrival, not of the epoch.
#[test]
fn a_birth_is_relative_to_the_instant_of_arrival() {
    let bench = Bench::new();
    let at = WorldTime::from_seconds(40 * YEAR);
    for index in 0..50 {
        let values = bench
            .draw(&format!("a-{index}"), &Fixed::default(), &[], at, 1)
            .expect("draws");
        let AttributeValue::Instant(birth) = values[&id("birth")].0 else {
            panic!("a birth");
        };
        let age = (at.seconds() - birth.seconds()) / YEAR;
        assert!(
            (20..=90).contains(&age),
            "a-{index}: {age} years at arrival"
        );
    }
}

/// `covers: adults` is checked at arrival: a person aged ten then is `NotCovered`.
#[test]
fn covers_adults_is_checked_at_arrival() {
    let mut bench = Bench::new();
    bench.profile.covers = Covers::Adults;
    let at = WorldTime::from_seconds(40 * YEAR);
    let child = fixed(
        vec![(
            "birth",
            AttributeValue::Instant(WorldTime::from_seconds(30 * YEAR)),
        )],
        vec![],
    );
    let refused = bench
        .draw("quin", &child, &[], at, 3)
        .expect_err("a child at arrival");
    assert!(refused.contains("NotCovered"), "{refused}");
    // Born ten years after the epoch is no child at the epoch's measure forty years on: an adult arriving
    // sixty years after the epoch is covered.
    let later = WorldTime::from_seconds(60 * YEAR);
    bench
        .draw("quin", &child, &[], later, 3)
        .expect("an adult at arrival");
}

/// I-j-3, D-j-7 (M-j-3): the same key against the same population gives the same values whatever the
/// admission's cause; another key gives other values somewhere.
#[test]
fn the_cause_is_never_mixed_into_the_draw() {
    let bench = Bench::new();
    let population = vec![(
        "r1".to_owned(),
        bench.traits(vec![("chronotype", chronotype("neutral", 1))]),
    )];
    let first = bench
        .draw("omar", &Fixed::default(), &population, WorldTime::EPOCH, 5)
        .expect("draws");
    let again = bench
        .draw(
            "omar",
            &Fixed::default(),
            &population,
            WorldTime::EPOCH,
            9_999,
        )
        .expect("draws");
    assert_eq!(first, again);
    let differs = (0..20).any(|index| {
        bench
            .draw(
                &format!("other-{index}"),
                &Fixed::default(),
                &population,
                WorldTime::EPOCH,
                5,
            )
            .expect("draws")
            != first
    });
    assert!(differs, "the key is in the word");
}
