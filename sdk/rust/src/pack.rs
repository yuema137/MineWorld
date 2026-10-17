//! What a System Pack says about itself, beyond [`System`], so that the build never has to be told.

use std::any::Any;
use std::sync::Arc;

use mineworld_authoring::{
    AuthoredConfiguration, AuthoredContent, DataDirectory, DataOrigin, DataRefusal, ErasedSubject,
    OwnerContext, ResolveRefusal, ResolvedSeeds, ShippedData,
};
use mineworld_contracts::{EventTypeId, SystemId};
use mineworld_kernel::System;
use serde::de::{Deserializer, Error as _, MapAccess};

use crate::section::{DataOwnerDecl, SectionOwner};

/// A System Pack, as the build sees it: a [`System`] that can be made from nothing, and the facts
/// about itself the World Pack loader and the tools need.
///
/// Each of these used to be an arm in a closed catalog in the World Pack loader, one per pack
/// (finding F-1). Said here, by the pack, they are said once, and installing the pack edits no code
/// that has to learn it (`DECISIONS.md` `ARC-33`).
///
/// Every default is the safe direction: a pack that declares nothing is biographically silent, owns
/// no section, and refuses to decode one; it owns no data directory, resolves nothing and ships nothing
/// (`ARC-121`). Its package identity has no default, so no pack is
/// anonymous.
///
/// `Default` is required because the installed set makes the pack twice — once to install into a
/// world, once to answer perception — and a system holds no fields: its mutable state is the
/// components it owns, which live in the world (`INV-7`).
pub trait SystemPack: System + Default {
    /// This pack's package identity (`DECISIONS.md` `ARC-53`): its Cargo name, version, licence,
    /// authors and repository, recorded at compile time. Always the same line, the first of the
    /// `impl`:
    ///
    /// ```text
    /// const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
    /// ```
    ///
    /// Required: a pack that does not state it does not compile. Read by `mineworld packs` only —
    /// never by a system, so a pack's version never reaches a fact.
    const PACKAGE: crate::Package;

    /// Which of this pack's event types belong in a person's objective biography
    /// (`DECISIONS.md` `ARC-29`) — the pack's own judgement over its own vocabulary.
    const BIOGRAPHICAL: &'static [EventTypeId] = &[];

    /// The section of authored content this pack owns, if any (`ARC-31`).
    ///
    /// Defined only through [`owns_section!`](crate::owns_section), which defines
    /// [`decode_section`](Self::decode_section) with it.
    const SECTION: Option<SectionOwner> = None;

    /// Decodes this pack's section as the next value of an authored file's map, with the pack's own
    /// type — straight from the stream, so a refusal keeps its line and column (`DEP-10`). The loader
    /// holds the result without knowing its type.
    ///
    /// # Errors
    ///
    /// The deserializer's own error when the section is not valid. By default, every call is refused
    /// with "the '`<id>`' system owns no section": a pack that owns nothing is never silently decoded.
    fn decode_section<'de, A: MapAccess<'de>>(
        map: &mut A,
    ) -> Result<Arc<dyn AuthoredContent>, A::Error> {
        let _ = map;
        Err(A::Error::custom(format!(
            "the '{}' system owns no section",
            Self::ID.as_str()
        )))
    }

    /// Its own id when the pack takes a world-level configuration (`DECISIONS.md` `ARC-61`), else
    /// [`None`].
    ///
    /// Defined only through [`configures!`](crate::configures), with the two items below.
    const CONFIGURATION: Option<SystemId> = None;

    /// The event types its configuration may seed: what the loader admits at genesis, and what the
    /// drift check at resume compares (`ARC-61` items 5 and 7).
    const CONFIGURATION_FACTS: &'static [EventTypeId] = &[];

    /// What the build's tools need to know about the pack's section of the World's Interaction List,
    /// if it has one (`ARC-63`). Defined only through [`interactions!`](crate::interactions).
    const INTERACTIONS: Option<crate::interactions::SectionDecl> = None;

    /// Decodes `configure/<id>.yaml` with the pack's own type — straight from the stream, so a refusal
    /// keeps its line and column (`DEP-10`).
    ///
    /// # Errors
    ///
    /// The deserializer's own error when the configuration is not valid. By default, every call is
    /// refused with "the '`<id>`' system takes no configuration": a pack that declares none is never
    /// silently configured.
    fn decode_configuration<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Arc<dyn AuthoredConfiguration>, D::Error> {
        let _ = deserializer;
        Err(D::Error::custom(format!(
            "the '{}' system takes no configuration",
            Self::ID.as_str()
        )))
    }

    /// The data directories this pack owns in World Packs and Entity Packs, if any (`ARC-121` item 1).
    ///
    /// Defined only through [`owns_data!`](crate::owns_data), with
    /// [`decode_data_file`](Self::decode_data_file) and [`data_context`](Self::data_context).
    const DATA: Option<DataOwnerDecl> = None;

    /// Decodes one file of one of this pack's data directories with the pack's own type — straight from
    /// the stream, so a refusal keeps its line and column (`DEP-10`). The loader holds the result without
    /// knowing its type.
    ///
    /// # Errors
    ///
    /// The deserializer's own error. By default, every call is refused with "the '`<id>`' system owns no
    /// data directory".
    fn decode_data_file<'de, D: Deserializer<'de>>(
        directory: DataDirectory,
        deserializer: D,
    ) -> Result<Box<dyn Any + Send + Sync>, D::Error> {
        let _ = (directory, deserializer);
        Err(D::Error::custom(owns_no_data(Self::ID)))
    }

    /// The pack's context: every data file of the world's composition, in collection order, and its own
    /// decoded configuration (`ARC-121` item 1). The loader holds the result without knowing its type.
    ///
    /// # Errors
    ///
    /// The owner's refusal. By default, refused with "the '`<id>`' system owns no data directory" — the
    /// loader never asks a pack whose [`DATA`](Self::DATA) is `None`.
    fn data_context(
        files: Vec<(DataOrigin, Box<dyn Any + Send + Sync>)>,
        configuration: Option<&dyn AuthoredConfiguration>,
    ) -> Result<Arc<dyn OwnerContext>, DataRefusal> {
        let _ = (files, configuration);
        Err(DataRefusal {
            file: None,
            reason: owns_no_data(Self::ID),
        })
    }

    /// Decodes this pack's section as the next value of an authored file's map, in the pack's context
    /// when it has one (`ARC-121` item 2).
    ///
    /// Defined through [`owns_section_in_context!`](crate::owns_section_in_context). By default the context
    /// is ignored and the section decodes as [`decode_section`](Self::decode_section) decodes it, so every
    /// pack that owns a section by [`owns_section!`](crate::owns_section) reads exactly as before.
    ///
    /// # Errors
    ///
    /// As [`decode_section`](Self::decode_section).
    fn decode_section_in<'de, A: MapAccess<'de>>(
        map: &mut A,
        context: Option<&dyn OwnerContext>,
    ) -> Result<Arc<dyn AuthoredContent>, A::Error> {
        let _ = context;
        Self::decode_section(map)
    }

    /// Whether this pack resolves across subjects once, before seeding (`ARC-121` item 3). Defined only
    /// through [`resolves!`](crate::resolves), with [`resolve`](Self::resolve).
    const RESOLVES: bool = false;

    /// The pack's resolution over every subject its section is carried by: called once, after every
    /// section is decoded and every reference checked, before any seeding.
    ///
    /// # Errors
    ///
    /// The owner's refusal. By default, nothing is resolved: `Ok(None)`.
    ///
    /// Named `resolve_subjects`, not `resolve`: every pack is a [`System`], whose `resolve` resolves an
    /// action, and one name for both would make `Self::resolve` ambiguous in the pack's own code.
    fn resolve_subjects(
        context: Option<&dyn OwnerContext>,
        configuration: Option<&dyn AuthoredConfiguration>,
        subjects: &[ErasedSubject<'_>],
    ) -> Result<Option<Arc<dyn ResolvedSeeds>>, ResolveRefusal> {
        let _ = (context, configuration, subjects);
        Ok(None)
    }

    /// The data files this pack ships, compiled in, for an owner's directories (`ARC-121` item 4,
    /// Q-S30b-1). Declared only through [`ships_data!`](crate::ships_data); by default none.
    const SHIPS: &'static [ShippedData] = &[];
}

/// The refusal of a pack that owns no data directory.
fn owns_no_data(id: SystemId) -> String {
    format!("the '{}' system owns no data directory", id.as_str())
}

/// Declares, inside an `impl SystemPack`, that the pack takes the world-level configuration its
/// [`PackConfiguration`](mineworld_authoring::PackConfiguration) impl describes (`DECISIONS.md`
/// `ARC-61`).
///
/// Defines [`SystemPack::CONFIGURATION`], [`SystemPack::CONFIGURATION_FACTS`] and
/// [`SystemPack::decode_configuration`] together, from that one impl, so they cannot disagree.
///
/// ```text
/// impl SystemPack for ExampleSystem {
///     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
///     mineworld_sdk::configures!();
/// }
/// ```
///
/// The pack states its configuration facts `Visibility::SystemInternal` with no subjects: a world's
/// configuration is nobody's perception (`ARC-61` item 8).
#[macro_export]
macro_rules! configures {
    () => {
        const CONFIGURATION: ::core::option::Option<$crate::__private::SystemId> =
            ::core::option::Option::Some(<Self as $crate::__private::SystemIdentity>::ID);

        const CONFIGURATION_FACTS: &'static [$crate::__private::EventTypeId] =
            <Self as $crate::__private::PackConfiguration>::FACTS;

        fn decode_configuration<'de, D: $crate::__private::Deserializer<'de>>(
            deserializer: D,
        ) -> ::core::result::Result<
            $crate::__private::Arc<dyn $crate::__private::AuthoredConfiguration>,
            D::Error,
        > {
            $crate::__private::DeserializeSeed::deserialize(
                $crate::__private::DecodeConfiguration::<Self>::new(),
                deserializer,
            )
        }
    };
}

/// Declares, inside an `impl SystemPack`, that the pack owns the section its
/// [`AuthoredSection`](mineworld_authoring::AuthoredSection) impl describes.
///
/// Defines [`SystemPack::SECTION`] and [`SystemPack::decode_section`] together, from that one impl, so
/// the key the build lists and the type a section is decoded with cannot disagree.
///
/// ```text
/// impl SystemPack for NamingSystem {
///     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
///     const BIOGRAPHICAL: &'static [EventTypeId] = BIOGRAPHICAL;
///     mineworld_sdk::owns_section!();
/// }
/// ```
#[macro_export]
macro_rules! owns_section {
    () => {
        const SECTION: ::core::option::Option<$crate::SectionOwner> =
            ::core::option::Option::Some($crate::SectionOwner::of::<Self>());

        fn decode_section<'de, A: $crate::__private::MapAccess<'de>>(
            map: &mut A,
        ) -> ::core::result::Result<
            $crate::__private::Arc<dyn $crate::__private::AuthoredContent>,
            A::Error,
        > {
            map.next_value_seed($crate::__private::Decode::<Self>::new())
        }
    };
}

/// Declares, inside an `impl SystemPack`, that the pack owns the data directories its
/// [`OwnsData`](mineworld_authoring::OwnsData) impl describes (`DECISIONS.md` `ARC-121` item 1).
///
/// Defines [`SystemPack::DATA`], [`SystemPack::decode_data_file`] and [`SystemPack::data_context`]
/// together, from that one impl, so the directories the build lists and the types their files decode
/// with cannot disagree.
///
/// ```text
/// impl SystemPack for ExampleSystem {
///     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
///     mineworld_sdk::owns_data!();
/// }
/// ```
#[macro_export]
macro_rules! owns_data {
    () => {
        const DATA: ::core::option::Option<$crate::DataOwnerDecl> =
            ::core::option::Option::Some($crate::DataOwnerDecl::of::<Self>());

        fn decode_data_file<'de, D: $crate::__private::Deserializer<'de>>(
            directory: $crate::__private::DataDirectory,
            deserializer: D,
        ) -> ::core::result::Result<
            ::std::boxed::Box<dyn ::core::any::Any + ::core::marker::Send + ::core::marker::Sync>,
            D::Error,
        > {
            $crate::__private::decode_data_file::<Self, D>(directory, deserializer)
        }

        fn data_context(
            files: ::std::vec::Vec<(
                $crate::__private::DataOrigin,
                ::std::boxed::Box<
                    dyn ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
                >,
            )>,
            configuration: ::core::option::Option<&dyn $crate::__private::AuthoredConfiguration>,
        ) -> ::core::result::Result<
            $crate::__private::Arc<dyn $crate::__private::OwnerContext>,
            $crate::__private::DataRefusal,
        > {
            $crate::__private::data_context::<Self>(files, configuration)
        }
    };
}

/// Declares, inside an `impl SystemPack`, that the pack owns the section its
/// [`SectionInContext`](mineworld_authoring::SectionInContext) impl describes, decoded in the pack's
/// context (`DECISIONS.md` `ARC-121` item 2).
///
/// Defines [`SystemPack::SECTION`], [`SystemPack::decode_section`] (refused: the section is decoded only
/// in its context) and [`SystemPack::decode_section_in`] together. It stands where
/// [`owns_section!`](crate::owns_section) would — a pack owns one section, so writing both does not
/// compile — and needs [`owns_data!`](crate::owns_data) beside it, which builds the context.
///
/// ```text
/// impl SystemPack for ExampleSystem {
///     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
///     mineworld_sdk::owns_data!();
///     mineworld_sdk::owns_section_in_context!();
/// }
/// ```
#[macro_export]
macro_rules! owns_section_in_context {
    () => {
        const SECTION: ::core::option::Option<$crate::SectionOwner> =
            ::core::option::Option::Some($crate::SectionOwner::of_in_context::<Self>());

        fn decode_section<'de, A: $crate::__private::MapAccess<'de>>(
            map: &mut A,
        ) -> ::core::result::Result<
            $crate::__private::Arc<dyn $crate::__private::AuthoredContent>,
            A::Error,
        > {
            let _ = map;
            ::core::result::Result::Err(<A::Error as $crate::__private::DeError>::custom(
                ::std::format!(
                    "the '{}' system's section is decoded only in its owner's context",
                    <Self as $crate::__private::SystemIdentity>::ID.as_str()
                ),
            ))
        }

        fn decode_section_in<'de, A: $crate::__private::MapAccess<'de>>(
            map: &mut A,
            context: ::core::option::Option<&dyn $crate::__private::OwnerContext>,
        ) -> ::core::result::Result<
            $crate::__private::Arc<dyn $crate::__private::AuthoredContent>,
            A::Error,
        > {
            let decode = context
                .and_then($crate::__private::DecodeIn::<Self>::from_erased)
                .ok_or_else(|| {
                    <A::Error as $crate::__private::DeError>::custom(::std::format!(
                        "the '{}' system has no context of its own to decode its section in",
                        <Self as $crate::__private::SystemIdentity>::ID.as_str()
                    ))
                })?;
            map.next_value_seed(decode)
        }
    };
}

/// Declares, inside an `impl SystemPack`, that the pack resolves across subjects once, before seeding,
/// as its [`ResolvesSubjects`](mineworld_authoring::ResolvesSubjects) impl describes (`DECISIONS.md`
/// `ARC-121` item 3).
///
/// Defines [`SystemPack::RESOLVES`] and [`SystemPack::resolve_subjects`] together. Needs
/// [`owns_section_in_context!`](crate::owns_section_in_context) and [`owns_data!`](crate::owns_data)
/// beside it.
#[macro_export]
macro_rules! resolves {
    () => {
        const RESOLVES: bool = true;

        fn resolve_subjects(
            context: ::core::option::Option<&dyn $crate::__private::OwnerContext>,
            configuration: ::core::option::Option<&dyn $crate::__private::AuthoredConfiguration>,
            subjects: &[$crate::__private::ErasedSubject<'_>],
        ) -> ::core::result::Result<
            ::core::option::Option<$crate::__private::Arc<dyn $crate::__private::ResolvedSeeds>>,
            $crate::__private::ResolveRefusal,
        > {
            let context = context.ok_or_else(|| $crate::__private::ResolveRefusal {
                subject: ::core::option::Option::None,
                reason: ::std::format!(
                    "the '{}' system has no context of its own to resolve in",
                    <Self as $crate::__private::SystemIdentity>::ID.as_str()
                ),
            })?;
            $crate::__private::resolve_erased::<Self>(context, configuration, subjects)
                .map(::core::option::Option::Some)
        }
    };
}

/// Declares, inside an `impl SystemPack`, the data files the pack ships for an owner's directories
/// (`DECISIONS.md` `ARC-121` item 4, Q-S30b-1): each a path `<directory>/<file>.yaml` relative to the
/// pack's crate directory, compiled in with `include_str!`.
///
/// Defines [`SystemPack::SHIPS`]. A path that is not `<directory>/<file>.yaml` does not compile; a file
/// that is absent does not either.
///
/// ```text
/// impl SystemPack for ExampleSystem {
///     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
///     mineworld_sdk::ships_data!("schemas/example.yaml");
/// }
/// ```
#[macro_export]
macro_rules! ships_data {
    ( $( $path:literal ),+ $(,)? ) => {
        const SHIPS: &'static [$crate::__private::ShippedData] = &[
            $(
                $crate::__private::ShippedData::new(
                    $path,
                    ::core::include_str!(::core::concat!(
                        ::core::env!("CARGO_MANIFEST_DIR"),
                        "/",
                        $path
                    )),
                ),
            )+
        ];
    };
}

#[cfg(test)]
mod tests {
    use mineworld_contracts::SystemId;
    use mineworld_kernel::{SystemDeclaration, SystemIdentity, SystemVersion};
    use serde::de::value::{Error, MapDeserializer};

    use super::*;

    /// A pack that declares nothing beyond being a system.
    #[derive(Default)]
    struct Silent;

    impl SystemIdentity for Silent {
        const ID: SystemId = SystemId::from_static("silent");
    }

    impl System for Silent {
        const VERSION: SystemVersion = SystemVersion::new(1);

        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>()
        }
    }

    impl SystemPack for Silent {
        const PACKAGE: crate::Package = crate::package!();
    }

    /// A pack that owns no section is refused, naming itself, rather than decoded by anything — the
    /// answer the catalog's last arm gave before packs declared themselves.
    #[test]
    fn a_pack_that_owns_no_section_refuses_to_decode_one_and_says_which_pack_it_is() {
        let mut map = MapDeserializer::<_, Error>::new([("anything", 1_u8)].into_iter());
        let key: Option<String> = map.next_key().expect("the map has a key");
        assert_eq!(key.as_deref(), Some("anything"));

        let refusal = Silent::decode_section(&mut map).expect_err("nothing to decode with");
        assert_eq!(refusal.to_string(), "the 'silent' system owns no section");
    }

    /// A pack that owns a section by `owns_section!` and nothing of `ARC-121`.
    #[derive(Default)]
    struct Plain;

    impl SystemIdentity for Plain {
        const ID: SystemId = SystemId::from_static("plain");
    }

    impl System for Plain {
        const VERSION: SystemVersion = SystemVersion::new(1);

        fn declaration(&self) -> SystemDeclaration {
            SystemDeclaration::of::<Self>()
        }
    }

    impl mineworld_authoring::AuthoredSection for Plain {
        const SECTION: mineworld_authoring::SectionName =
            mineworld_authoring::SectionName::from_static("plain");
        const CARRIED_BY: &'static [mineworld_authoring::ContentKind] =
            &[mineworld_authoring::ContentKind::Person];
        type Authored = u8;

        fn seed(
            _: &mineworld_authoring::Seeding<'_, '_>,
            _: mineworld_contracts::EntityId,
            _: &u8,
        ) -> Result<Vec<mineworld_kernel::Emission>, mineworld_contracts::Rejection> {
            Ok(Vec::new())
        }
    }

    impl SystemPack for Plain {
        const PACKAGE: crate::Package = crate::package!();
        crate::owns_section!();
    }

    /// M-b-2's test: every pack that predates `ARC-121` decodes its section in a context exactly as it
    /// decodes it, owns no data, resolves nothing and ships nothing — the defaults that keep every existing
    /// pack and world unchanged.
    #[test]
    fn a_silent_pack_decodes_in_context_as_it_decodes() {
        let mut map = MapDeserializer::<_, Error>::new([("plain", 7_u8)].into_iter());
        let _: Option<String> = map.next_key().expect("the map has a key");
        let decoded = Plain::decode_section_in(&mut map, None).expect("decodes as it decodes");
        assert_eq!(decoded.owner(), Plain::ID);
        assert_eq!(decoded.as_any().downcast_ref::<u8>(), Some(&7));

        let mut map = MapDeserializer::<_, Error>::new([("anything", 1_u8)].into_iter());
        let _: Option<String> = map.next_key().expect("the map has a key");
        let refusal = Silent::decode_section_in(&mut map, None).expect_err("owns no section");
        assert_eq!(refusal.to_string(), "the 'silent' system owns no section");

        assert_eq!(Plain::DATA, None);
        let refusal = Plain::decode_data_file(
            mineworld_authoring::DataDirectory::from_static("anywhere"),
            serde::de::value::UnitDeserializer::<Error>::new(),
        )
        .expect_err("owns no data");
        assert_eq!(
            refusal.to_string(),
            "the 'plain' system owns no data directory"
        );
        let refusal = Plain::data_context(Vec::new(), None).expect_err("owns no data");
        assert_eq!(refusal.reason, "the 'plain' system owns no data directory");
        const { assert!(!Plain::RESOLVES) };
        assert!(matches!(Plain::resolve_subjects(None, None, &[]), Ok(None)));
        assert!(Plain::SHIPS.is_empty());
    }

    /// A pack that declares no configuration is refused one, naming itself, and states no
    /// configuration facts for the drift check to compare (`ARC-61`).
    #[test]
    fn a_pack_that_declares_no_configuration_refuses_one_and_says_which_pack_it_is() {
        let deserializer = serde::de::value::UnitDeserializer::<Error>::new();
        let refusal =
            Silent::decode_configuration(deserializer).expect_err("nothing to decode with");
        assert_eq!(
            refusal.to_string(),
            "the 'silent' system takes no configuration"
        );
        assert_eq!(Silent::CONFIGURATION, None);
        assert!(Silent::CONFIGURATION_FACTS.is_empty());
    }
}
