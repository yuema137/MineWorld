//! What a System Pack says about itself, beyond [`System`], so that the build never has to be told.

use std::sync::Arc;

use mineworld_authoring::{AuthoredConfiguration, AuthoredContent};
use mineworld_contracts::{EventTypeId, SystemId};
use mineworld_kernel::System;
use serde::de::{Deserializer, Error as _, MapAccess};

use crate::section::SectionOwner;

/// A System Pack, as the build sees it: a [`System`] that can be made from nothing, and the facts
/// about itself the World Pack loader and the tools need.
///
/// Each of these used to be an arm in a closed catalog in the World Pack loader, one per pack
/// (finding F-1). Said here, by the pack, they are said once, and installing the pack edits no code
/// that has to learn it (`DECISIONS.md` `ARC-33`).
///
/// Every default is the safe direction: a pack that declares nothing is biographically silent, owns
/// no section, and refuses to decode one. Its package identity has no default, so no pack is
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
