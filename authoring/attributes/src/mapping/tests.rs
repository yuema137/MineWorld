//! `keeps`, `mapping` and the `map` derivation on a toy backend (`pr-s30h-matraix-backend.md` §§5.7,
//! 5.8; AC-h1-4 … AC-h1-6): every refusal by name, and a view read through the composed catalogue.

use std::path::PathBuf;

use mineworld_authoring::{DataDirectory, DataOrigin, DataSource};
use mineworld_contracts::WorldTime;

use crate::config::AttributesConfiguration;
use crate::context::AttributesContext;
use crate::declaration::{DataFile, Derive, Mutability};
use crate::id::QualifiedId;
use crate::traits::Traits;
use crate::value::AttributeValue;

const CORE: &str = "format: 1
attributes:
  chronotype:
    group: behaviour
    kind: { level: [early-bird, neutral, night-owl] }
    visibility: learnable
    mutability: evolving
    label: Chronotype
    phrase: \"{value}\"
    words: { early-bird: an early bird }
  natal-sex:
    group: background
    kind: { category: [female, male] }
    visibility: private
    mutability: fixed
    label: Natal sex
  smoker:
    group: lifestyle
    kind: flag
    visibility: perceivable
    mutability: evolving
    label: Smoker
  stature:
    group: appearance
    kind: { quantity: { unit: m, scale: 3, min: 0.500, max: 2.500 } }
    visibility: perceivable
    mutability: fixed
    label: Stature
";

const MINEWORLD: &str = "format: 1\nbackend: mineworld\nversion: 1.0.0\nlicense: CC0-1.0\n\
attribution: built-in\ncatalogues: [core]\nmasks: { default: all }\nsampling: [profile]\n\
provenance: { source: mineworld }\n";

const TOY: &str = "format: 1
attributes:
  rhythm:
    group: behaviour
    kind: { category: [dawn, dusk, any] }
    visibility: learnable
    mutability: fixed
    label: Rhythm
  smokes:
    group: lifestyle
    kind: { level: [never, sometimes, daily] }
    visibility: perceivable
    mutability: fixed
    label: Smokes
  diary:
    group: lifestyle
    kind: { text: { max_bytes: 40 } }
    visibility: learnable
    mutability: fixed
    label: Diary
";

const TOY_BACKEND: &str = "format: 1\nbackend: toy\nversion: 0.1.0\nlicense: CC0-1.0\n\
attribution: toy\ncatalogues: [toy]\nmasks: { default: all }\nsampling: [profile]\n\
provenance: { source: mineworld }\nkeeps: [core.natal-sex]\nmapping: toy\n";

const MAPPING: &str = "format: 1
backend: toy
views:
  core.chronotype:
    from: rhythm
    values: { dawn: early-bird, dusk: night-owl, any: neutral }
  core.smoker:
    from: smokes
    values: { never: 'false', sometimes: 'true', daily: 'true' }
";

fn origin(pack: &str, source: DataSource, directory: &'static str, name: &str) -> DataOrigin {
    DataOrigin {
        pack: pack.to_owned(),
        source,
        directory: DataDirectory::from_static(directory),
        path: PathBuf::from(format!("{pack}/{directory}/{name}.yaml")),
    }
}

/// The toy world: `core` (person-attributes, as `core_source` ships it), the toy backend with `backend`,
/// `mapping` and `mask` (`None`: the built-in `all`).
fn compose(
    core_source: DataSource,
    backend: &str,
    mapping: &str,
    mask: Option<&str>,
) -> Result<AttributesContext, String> {
    let mut files = vec![
        (
            origin("person-attributes", core_source, "attributes", "core"),
            DataFile::Attributes(serde_saphyr::from_str(CORE).expect("core")),
        ),
        (
            origin("person-attributes", DataSource::EntityPack, "backends", "mineworld"),
            DataFile::Backend(Box::new(serde_saphyr::from_str(MINEWORLD).expect("mineworld"))),
        ),
        (
            origin("toy-backend", DataSource::EntityPack, "attributes", "toy"),
            DataFile::Attributes(serde_saphyr::from_str(TOY).expect("toy")),
        ),
        (
            origin("toy-backend", DataSource::EntityPack, "backends", "toy"),
            DataFile::Backend(Box::new(serde_saphyr::from_str(backend).expect("backend"))),
        ),
        (
            origin("toy-backend", DataSource::EntityPack, "mappings", "toy"),
            DataFile::Mapping(Box::new(serde_saphyr::from_str(mapping).expect("mapping"))),
        ),
    ];
    if let Some(mask) = mask {
        files.push((
            origin("toy-backend", DataSource::EntityPack, "masks", "small"),
            DataFile::Mask(Box::new(serde_saphyr::from_str(mask).expect("mask"))),
        ));
    }
    let configuration = AttributesConfiguration {
        format: 1,
        profile: None,
        seed: None,
        salt: 0,
        lock: None,
        backend: Some("toy".to_owned()),
        mask: mask.map(|_| "small".to_owned()),
    };
    AttributesContext::build(files, Some(&configuration)).map_err(|refusal| refusal.reason)
}

fn id(text: &str) -> QualifiedId {
    QualifiedId::new(text).expect("an id")
}

fn refused(backend: &str, mapping: &str) -> String {
    compose(DataSource::EntityPack, backend, mapping, None).expect_err("refused")
}

#[test]
fn a_view_is_derived_from_its_source_and_kept_ids_stay_mineworlds() {
    let context = compose(DataSource::EntityPack, TOY_BACKEND, MAPPING, None).expect("composes");
    let catalogue = &context.catalogue;
    let chronotype = catalogue.get(&id("core.chronotype")).expect("a view");
    assert_eq!(chronotype.mutability, Mutability::Derived);
    assert!(matches!(chronotype.derive, Some(Derive::Map { .. })));
    assert_eq!(chronotype.words.len(), 1, "the target's own words");
    assert!(catalogue.get(&id("core.natal-sex")).is_some(), "kept");
    assert!(catalogue.get(&id("core.stature")).is_some(), "appearance");

    let slug = |text: &str| crate::id::ValueSlug::new(text).expect("slug");
    let traits = Traits::stated(
        catalogue,
        [
            (
                id("toy-backend.rhythm"),
                AttributeValue::Category(slug("dusk")),
            ),
            (
                id("toy-backend.smokes"),
                AttributeValue::Level(crate::value::LevelValue {
                    slug: slug("never"),
                    rank: 0,
                }),
            ),
        ],
    )
    .expect("plain traits");
    let stated = Traits::stated(
        catalogue,
        [(
            id("core.chronotype"),
            AttributeValue::Level(crate::value::LevelValue {
                slug: slug("neutral"),
                rank: 1,
            }),
        )],
    )
    .expect_err("a view is never stored");
    assert!(stated.contains("derived"), "{stated}");
    let at = WorldTime::from_seconds(0);
    let read = |attribute: &str| {
        let declaration = catalogue.get(&id(attribute)).expect("declared");
        crate::derive::value(catalogue, declaration, &traits, at)
    };
    assert!(matches!(
        read("core.chronotype"),
        Some(AttributeValue::Level(level)) if level.slug.as_str() == "night-owl" && level.rank == 2
    ));
    assert_eq!(read("core.smoker"), Some(AttributeValue::Flag(false)));
}

#[test]
fn a_masked_out_source_leaves_its_view_absent_and_says_why() {
    let mask = "format: 1\nmask: small\nversion: 1\ninclude:\n  - { id: toy-backend.smokes, why: smoking only }\n";
    let context =
        compose(DataSource::EntityPack, TOY_BACKEND, MAPPING, Some(mask)).expect("composes");
    assert!(context.catalogue.get(&id("core.chronotype")).is_none());
    let why = &context.catalogue.removed()[&id("core.chronotype")];
    assert!(
        why.contains("not provided: the mask leaves out the source 'toy-backend.rhythm'"),
        "{why}"
    );
    assert!(context.catalogue.get(&id("core.smoker")).is_some());
}

#[test]
fn each_mapping_rule_is_refused_by_name() {
    let cases = [
        (
            MAPPING.replacen("any: neutral", "", 1),
            "MappingUncovered",
        ),
        (
            MAPPING.replacen("any: neutral", "any: neutral, noon: neutral", 1),
            "MappingValueUnknown",
        ),
        (
            MAPPING.replacen("any: neutral", "any: sometimes", 1),
            "MappingValueUnknown",
        ),
        (MAPPING.replacen("from: rhythm", "from: nothing", 1), "MappingSourceUnknown"),
        (MAPPING.replacen("from: rhythm", "from: diary", 1), "MappingKind"),
        (
            MAPPING.replacen("core.chronotype:", "core.stature:", 1),
            "MappingTargetNotRemoved",
        ),
        (
            MAPPING.replacen("core.chronotype:", "core.natal-sex:", 1),
            "both kept and mapped",
        ),
        (
            MAPPING.replacen("core.chronotype:", "core.unheard-of:", 1),
            "MappingTargetNotRemoved",
        ),
    ];
    for (mapping, name) in cases {
        let refusal = refused(TOY_BACKEND, &mapping);
        assert!(refusal.contains(name), "{name}: {refusal}");
    }
}

/// `toy-backend.smokes` is perceivable and `core.smoker` perceivable; narrowing the source makes the
/// view wider than it.
#[test]
fn a_view_wider_than_its_source_is_refused() {
    let mapping = MAPPING.replacen(
        "core.smoker:\n    from: smokes",
        "core.smoker:\n    from: rhythm",
        1,
    );
    let mapping = mapping.replacen(
        "{ never: 'false', sometimes: 'true', daily: 'true' }",
        "{ dawn: 'false', dusk: 'true', any: 'true' }",
        1,
    );
    let refusal = refused(TOY_BACKEND, &mapping);
    assert!(refusal.contains("MappingWidens"), "{refusal}");
}

#[test]
fn a_view_of_an_owners_attribute_is_refused() {
    let core = CORE.replacen(
        "    label: Chronotype\n",
        "    label: Chronotype\n    owner: person-attributes\n",
        1,
    );
    let mut files = vec![(
        origin("person-attributes", DataSource::SystemPack, "attributes", "core"),
        DataFile::Attributes(serde_saphyr::from_str(&core).expect("core")),
    )];
    files.extend(
        [
            ("person-attributes", "backends", "mineworld", MINEWORLD),
            ("toy-backend", "backends", "toy", TOY_BACKEND),
        ]
        .map(|(pack, directory, name, text)| {
            (
                origin(pack, DataSource::EntityPack, directory, name),
                DataFile::Backend(Box::new(serde_saphyr::from_str(text).expect("backend"))),
            )
        }),
    );
    files.push((
        origin("toy-backend", DataSource::EntityPack, "attributes", "toy"),
        DataFile::Attributes(serde_saphyr::from_str(TOY).expect("toy")),
    ));
    files.push((
        origin("toy-backend", DataSource::EntityPack, "mappings", "toy"),
        DataFile::Mapping(Box::new(serde_saphyr::from_str(MAPPING).expect("mapping"))),
    ));
    let configuration = AttributesConfiguration {
        format: 1,
        profile: None,
        seed: None,
        salt: 0,
        lock: None,
        backend: Some("toy".to_owned()),
        mask: None,
    };
    let refusal = AttributesContext::build(files, Some(&configuration))
        .expect_err("owned")
        .reason;
    assert!(refusal.contains("MappingTargetOwned"), "{refusal}");
}

#[test]
fn keeps_names_only_what_the_backend_would_remove() {
    for (kept, why) in [
        ("core.stature", "not a `core` attribute this backend removes"),
        ("toy-backend.rhythm", "not a `core` attribute"),
    ] {
        let backend = TOY_BACKEND.replacen("core.natal-sex", kept, 1);
        let refusal = refused(&backend, MAPPING);
        assert!(
            refusal.contains("BackendMalformed") && refusal.contains("keeps") && refusal.contains(why),
            "{kept}: {refusal}"
        );
    }
    let missing = TOY_BACKEND.replacen("mapping: toy", "mapping: other", 1);
    assert!(refused(&missing, MAPPING).contains("no mappings/other.yaml"));
    let foreign = MAPPING.replacen("backend: toy", "backend: elsewhere", 1);
    assert!(refused(TOY_BACKEND, &foreign).contains("the mapping is for 'elsewhere'"));
}
