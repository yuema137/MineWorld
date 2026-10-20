//! `ARC-121` through the SDK: `owns_data!`, `owns_section_in_context!`, `resolves!` and `SHIPS` each define
//! their items together, and a test-local installed set dispatches every one of them — the calls the
//! World Pack loader makes, here with a pack the loader's closed catalog cannot hold (as
//! `systems/installed/tests/installed.rs` expands its own set). `ships_data!`'s `include_str!` is driven
//! end to end by `tests/acceptance/tests/data_seam.rs`, whose crate holds a real file.

use std::any::Any;
use std::collections::BTreeMap;
use std::path::PathBuf;

use mineworld_authoring::{
    ContentKind, DataDirectory, DataOrigin, DataRefusal, DataSource, ErasedSubject, OwnsData,
    ResolveRefusal, ResolvesSubjects, SectionInContext, SectionName, Seeding, ShippedData, Subject,
};
use mineworld_contracts::{EntityId, EntityKey, Rejection, SystemId, Tags};
use mineworld_kernel::{Emission, System, SystemDeclaration, SystemIdentity, SystemVersion};
use mineworld_sdk::{DataOwnerDecl, SystemPack};
use serde::de::value::{Error, MapDeserializer};
use serde::de::{Deserialize, Deserializer, Error as _, MapAccess};

/// The perception trait this test set answers through: nothing, here.
pub trait Perceives {}

const LIMITS: DataDirectory = DataDirectory::from_static("limits");

/// A pack owning `limits/` (each file: the largest value a field may take), a `limited:` section
/// checked against them, and a resolution that counts its subjects.
#[derive(Default)]
pub struct Owner;

impl SystemIdentity for Owner {
    const ID: SystemId = SystemId::from_static("test-owner");
}

impl System for Owner {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

impl Perceives for Owner {}

impl OwnsData for Owner {
    const DIRECTORIES: &'static [DataDirectory] = &[LIMITS];
    type File = BTreeMap<String, u32>;
    type Context = BTreeMap<String, u32>;

    fn decode_file<'de, D: Deserializer<'de>>(
        _: DataDirectory,
        deserializer: D,
    ) -> Result<Self::File, D::Error> {
        BTreeMap::deserialize(deserializer)
    }

    fn context(
        files: Vec<(DataOrigin, Self::File)>,
        _: Option<&dyn Any>,
    ) -> Result<Self::Context, DataRefusal> {
        Ok(files.into_iter().flat_map(|(_, file)| file).collect())
    }
}

impl SectionInContext for Owner {
    const SECTION: SectionName = SectionName::from_static("limited");
    const CARRIED_BY: &'static [ContentKind] = &[ContentKind::Person];
    type Authored = u32;

    /// One value for the field `size`, at most the declared limit.
    fn decode<'de, D: Deserializer<'de>>(
        context: &Self::Context,
        deserializer: D,
    ) -> Result<u32, D::Error> {
        let value = u32::deserialize(deserializer)?;
        let max = context.get("size").copied().unwrap_or(0);
        if value > max {
            return Err(D::Error::custom(format!("at most {max}, not {value}")));
        }
        Ok(value)
    }

    fn seed(
        _: &Seeding<'_, '_>,
        _: EntityId,
        _: &Self::Context,
        _: &u32,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

impl ResolvesSubjects for Owner {
    type Seed = usize;

    /// Each subject's seed is how many subjects there were.
    fn resolve_subjects(
        _: &Self::Context,
        _: Option<&dyn Any>,
        subjects: &[Subject<'_, u32>],
    ) -> Result<BTreeMap<EntityKey, usize>, ResolveRefusal> {
        Ok(subjects
            .iter()
            .map(|subject| (subject.key.clone(), subjects.len()))
            .collect())
    }

    fn seed_resolved(
        _: &Seeding<'_, '_>,
        _: EntityId,
        _: &Self::Context,
        _: &usize,
    ) -> Result<Vec<Emission>, Rejection> {
        Ok(Vec::new())
    }
}

/// What `Owner` ships, written as `ships_data!` expands it but with the text inline.
const SHIPPED: &[ShippedData] = &[ShippedData::new("limits/base.yaml", "size: 5\n")];

impl SystemPack for Owner {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    mineworld_sdk::owns_data!();
    mineworld_sdk::owns_section_in_context!();
    mineworld_sdk::resolves!();
    const SHIPS: &'static [ShippedData] = SHIPPED;
}

/// A pack that declares nothing beyond being one.
#[derive(Default)]
pub struct Bare;

impl SystemIdentity for Bare {
    const ID: SystemId = SystemId::from_static("test-bare");
}

impl System for Bare {
    const VERSION: SystemVersion = SystemVersion::new(1);

    fn declaration(&self) -> SystemDeclaration {
        SystemDeclaration::of::<Self>()
    }
}

impl Perceives for Bare {}

impl SystemPack for Bare {
    const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
}

mineworld_sdk::installed! {
    perception: Perceives;
    Bare => Bare,
    Owner => Owner,
}

/// The `limits/` file declaring `size: <max>`, decoded through the capability.
fn limits(max: u32) -> (DataOrigin, Box<dyn Any + Send + Sync>) {
    let decoded = Capability::Owner
        .decode_data_file(
            LIMITS,
            MapDeserializer::<_, Error>::new([("size", max)].into_iter()),
        )
        .expect("decodes");
    let origin = DataOrigin {
        pack: "a-world".to_owned(),
        source: DataSource::World,
        directory: LIMITS,
        path: PathBuf::from("limits/a.yaml"),
    };
    (origin, decoded)
}

/// Decodes `limited: <value>` through the capability, in `context`.
fn limited(
    capability: Capability,
    value: u32,
    context: Option<&dyn mineworld_authoring::OwnerContext>,
) -> Result<std::sync::Arc<dyn mineworld_authoring::AuthoredContent>, Error> {
    let mut map = MapDeserializer::<_, Error>::new([("limited", value)].into_iter());
    let _: Option<String> = map.next_key().expect("a key");
    capability.decode_section_in(&mut map, context)
}

#[test]
fn each_macro_defines_its_items_together_and_the_installed_set_dispatches_them() {
    // owns_data!: the directories, the decoder and the context.
    assert_eq!(
        Capability::Owner.data(),
        Some(DataOwnerDecl {
            directories: &[LIMITS]
        })
    );
    assert_eq!(
        Capability::owning_data("limits"),
        Some((Capability::Owner, LIMITS))
    );
    assert_eq!(Capability::owning_data("people"), None);
    let context = Capability::Owner
        .data_context(vec![limits(5)], None)
        .expect("the context builds");
    assert_eq!(context.owner(), Owner::ID);

    // owns_section_in_context!: the section the build lists, decoded only in the context.
    let section = Capability::Owner.section().expect("owns a section");
    assert_eq!(section.name.as_str(), "limited");
    assert!(section.carried_by(ContentKind::Person));
    let decoded = limited(Capability::Owner, 5, Some(&*context)).expect("within the limit");
    assert_eq!(decoded.as_any().downcast_ref::<u32>(), Some(&5));
    let refusal = limited(Capability::Owner, 6, Some(&*context)).expect_err("over it");
    assert_eq!(refusal.to_string(), "at most 5, not 6");
    let refusal = limited(Capability::Owner, 1, None).expect_err("no context");
    assert_eq!(
        refusal.to_string(),
        "the 'test-owner' system has no context of its own to decode its section in"
    );
    let mut map = MapDeserializer::<_, Error>::new([("limited", 1_u32)].into_iter());
    let _: Option<String> = map.next_key().expect("a key");
    let refusal = Capability::Owner
        .decode_section(&mut map)
        .expect_err("only in context");
    assert_eq!(
        refusal.to_string(),
        "the 'test-owner' system's section is decoded only in its owner's context"
    );

    // resolves!: every subject resolved once, with or without the section.
    assert!(Capability::Owner.resolves());
    let (ada, bo, tags) = (
        EntityKey::new("ada").expect("a key"),
        EntityKey::new("bo").expect("a key"),
        Tags::default(),
    );
    let subjects = [
        ErasedSubject {
            key: &ada,
            kind: ContentKind::Person,
            tags: &tags,
            authored: Some(&*decoded),
        },
        ErasedSubject {
            key: &bo,
            kind: ContentKind::Person,
            tags: &tags,
            authored: None,
        },
    ];
    let resolved = Capability::Owner
        .resolve_subjects(Some(&*context), None, &subjects)
        .expect("resolves")
        .expect("a resolving pack answers");
    assert_eq!(
        resolved
            .as_any()
            .downcast_ref::<BTreeMap<EntityKey, usize>>(),
        Some(&BTreeMap::from([(ada.clone(), 2), (bo.clone(), 2)]))
    );
    let refusal = Capability::Owner
        .resolve_subjects(None, None, &subjects)
        .expect_err("no context");
    assert_eq!(refusal.subject, None);

    // SHIPS.
    assert_eq!(Capability::Owner.shipped(), SHIPPED);

    // A pack that declares none of it.
    assert_eq!(Capability::Bare.data(), None);
    assert!(!Capability::Bare.resolves());
    assert!(Capability::Bare.shipped().is_empty());
    assert!(matches!(
        Capability::Bare.resolve_subjects(None, None, &subjects),
        Ok(None)
    ));
    let refusal = limited(Capability::Bare, 1, Some(&*context)).expect_err("owns no section");
    assert_eq!(
        refusal.to_string(),
        "the 'test-bare' system owns no section"
    );
    let refusal = Capability::Bare
        .data_context(Vec::new(), None)
        .expect_err("owns no data");
    assert_eq!(
        refusal.reason,
        "the 'test-bare' system owns no data directory"
    );
}

/// A decoder is handed only its own owner's context.
#[test]
fn a_section_is_not_decoded_in_another_owners_context() {
    struct Other;
    impl SystemIdentity for Other {
        const ID: SystemId = SystemId::from_static("test-other");
    }
    impl OwnsData for Other {
        const DIRECTORIES: &'static [DataDirectory] = &[DataDirectory::from_static("others")];
        type File = BTreeMap<String, u32>;
        type Context = BTreeMap<String, u32>;

        fn decode_file<'de, D: Deserializer<'de>>(
            _: DataDirectory,
            deserializer: D,
        ) -> Result<Self::File, D::Error> {
            BTreeMap::deserialize(deserializer)
        }

        fn context(
            _: Vec<(DataOrigin, Self::File)>,
            _: Option<&dyn Any>,
        ) -> Result<Self::Context, DataRefusal> {
            Ok(BTreeMap::from([("size".to_owned(), 100)]))
        }
    }
    let foreign = mineworld_authoring::data_context::<Other>(Vec::new(), None).expect("builds");
    let refusal = limited(Capability::Owner, 50, Some(&*foreign)).expect_err("not its own");
    assert_eq!(
        refusal.to_string(),
        "the 'test-owner' system has no context of its own to decode its section in"
    );
}
