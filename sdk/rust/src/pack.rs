//! What a System Pack says about itself, beyond [`System`], so that the build never has to be told.

use std::sync::Arc;

use mineworld_authoring::AuthoredContent;
use mineworld_contracts::EventTypeId;
use mineworld_kernel::System;
use serde::de::{Error as _, MapAccess};

use crate::section::SectionOwner;

/// A System Pack, as the build sees it: a [`System`] that can be made from nothing, and the facts
/// about itself the World Pack loader and the tools need.
///
/// Each of these used to be an arm in a closed catalog in the World Pack loader, one per pack
/// (finding F-1). Said here, by the pack, they are said once, and installing the pack edits no code
/// that has to learn it (`DECISIONS.md` `ARC-33`).
///
/// Every default is the safe direction: a pack that declares nothing is biographically silent, owns
/// no section, and refuses to decode one.
///
/// `Default` is required because the installed set makes the pack twice — once to install into a
/// world, once to answer perception — and a system holds no fields: its mutable state is the
/// components it owns, which live in the world (`INV-7`).
pub trait SystemPack: System + Default {
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
}

/// Declares, inside an `impl SystemPack`, that the pack owns the section its
/// [`AuthoredSection`](mineworld_authoring::AuthoredSection) impl describes.
///
/// Defines [`SystemPack::SECTION`] and [`SystemPack::decode_section`] together, from that one impl, so
/// the key the build lists and the type a section is decoded with cannot disagree.
///
/// ```text
/// impl SystemPack for NamingSystem {
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

    impl SystemPack for Silent {}

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
}
