//! The macro a build's installed set of System Packs is written in (`DECISIONS.md` `ARC-33`).
//!
//! It expands to the closed catalog the World Pack loader reads — an enum with one variant per pack,
//! the list in a fixed order, and one `match` per question — exactly the catalog that was written by
//! hand, one arm per pack, before packs declared themselves. Generating it rather than registering
//! packs at startup keeps every code path monomorphic: a section is still decoded straight from the
//! YAML stream, with its line and column (`DEP-10`), and `DEP-12` records why the registration crates
//! were declined.

/// Declares a build's installed set: every System Pack the build provides, one line each.
///
/// ```text
/// mineworld_sdk::installed! {
///     perception: mineworld_presence::PerceptionProvider;
///     Presence => mineworld_presence::PresenceSystem,
///     Movement => mineworld_movement::MovementSystem,
/// }
/// ```
///
/// `perception` names the trait every pack answers perception through, so this crate names no pack.
/// Each line names a variant and the pack's system type, which must implement
/// [`SystemPack`](crate::SystemPack) and the perception trait; a type that does not is refused by
/// the compiler here, at the list.
///
/// The expansion, in the invoking crate:
///
/// ```text
/// pub enum Capability { … }                 one variant per line, in the listed order
/// pub const AVAILABLE: [Capability; N]      every capability, in the listed order
/// impl Capability {
///     resolve, id, section, owning_section, decode_section, biographical, install, provider,
///     type_name
/// }
/// impl Display for Capability               its id
/// ```
#[macro_export]
macro_rules! installed {
    (
        perception: $perception:path;
        $( $Variant:ident => $System:ty ),+ $(,)?
    ) => {
        /// One System Pack this build can install.
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub enum Capability {
            $(
                #[doc = concat!("The System Pack `", stringify!($System), "`.")]
                $Variant,
            )+
        }

        /// Every system this build provides, in a fixed order — the order an error message lists
        /// them in, and the order of the installed set's list.
        pub const AVAILABLE: [Capability; [$( stringify!($Variant) ),+].len()] =
            [$( Capability::$Variant ),+];

        impl Capability {
            /// Which capability a pack is asking for, or [`None`] if this build has no such system.
            pub fn resolve(name: &$crate::__private::SystemId) -> ::core::option::Option<Self> {
                AVAILABLE
                    .into_iter()
                    .find(|capability| capability.id() == *name)
            }

            /// The name this capability is enabled by.
            pub fn id(self) -> $crate::__private::SystemId {
                match self {
                    $( Self::$Variant => <$System as $crate::__private::SystemIdentity>::ID, )+
                }
            }

            /// The section of authored content this capability owns, if any (`ARC-31`).
            pub const fn section(self) -> ::core::option::Option<$crate::SectionOwner> {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::SECTION, )+
                }
            }

            /// The capability that owns the section with this key, and that section, if this build
            /// has one.
            pub fn owning_section(
                key: &str,
            ) -> ::core::option::Option<(Self, $crate::SectionOwner)> {
                AVAILABLE.into_iter().find_map(|capability| {
                    capability
                        .section()
                        .filter(|section| section.name.as_str() == key)
                        .map(|section| (capability, section))
                })
            }

            /// Decodes this capability's section as the next value of an authored file's map, with
            /// the owner's own type — straight from the stream, so a refusal keeps its line and
            /// column (`DEP-10`). The loader holds the result without knowing its type.
            ///
            /// # Errors
            ///
            /// The deserializer's error for an invalid section, or a refusal naming the system when
            /// it owns no section.
            pub fn decode_section<'de, A: $crate::__private::MapAccess<'de>>(
                self,
                map: &mut A,
            ) -> ::core::result::Result<
                $crate::__private::Arc<dyn $crate::__private::AuthoredContent>,
                A::Error,
            > {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::decode_section(map), )+
                }
            }

            /// Which of this capability's event types belong in a person's objective biography
            /// (`ARC-29`) — each pack's own judgement over its own vocabulary, aggregated here.
            pub fn biographical(self) -> &'static [$crate::__private::EventTypeId] {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::BIOGRAPHICAL, )+
                }
            }

            /// Installs it into a world under assembly.
            ///
            /// Takes the system by value, as `World::install` requires: installation is the world
            /// assembler's act, and a system that is already installed cannot be installed again
            /// under the same name.
            ///
            /// # Errors
            ///
            /// Whatever the kernel's registry refuses: a duplicate, or a missing dependency.
            pub fn install(
                self,
                world: &mut $crate::__private::World,
            ) -> ::core::result::Result<(), $crate::__private::KernelError> {
                match self {
                    $(
                        Self::$Variant => world.install(
                            <$System as ::core::default::Default>::default(),
                        ),
                    )+
                }
            }

            /// This capability as an answerer of "what may this observer attempt against that
            /// target".
            ///
            /// A second value of the system type, which is safe by construction rather than merely
            /// convenient: a system holds no fields, because its mutable state is the components it
            /// owns and those live in the world.
            pub fn provider(self) -> ::std::boxed::Box<dyn $perception> {
                match self {
                    $(
                        Self::$Variant => ::std::boxed::Box::new(
                            <$System as ::core::default::Default>::default(),
                        ),
                    )+
                }
            }

            /// The Rust path of this capability's system type, for the installed set's own
            /// consistency guard. Not an identity: [`Capability::id`] is.
            pub fn type_name(self) -> &'static str {
                match self {
                    $( Self::$Variant => ::core::any::type_name::<$System>(), )+
                }
            }
        }

        impl ::core::fmt::Display for Capability {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                f.write_str(self.id().as_str())
            }
        }
    };
}
