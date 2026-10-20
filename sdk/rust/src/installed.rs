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
/// Zero or more **extension lines**, after `perception` and before the pack lines, list the build's
/// extension catalogs (`DECISIONS.md` `ARC-62`): for each, a trait a pack owns, that pack's register
/// function, and the types other packs implement the trait with, each followed by a comma.
///
/// ```text
/// mineworld_sdk::installed! {
///     perception: mineworld_presence::PerceptionProvider;
///     extension mineworld_presence::ArrivalResolver => mineworld_presence::register_resolvers: [];
///     Presence => mineworld_presence::PresenceSystem,
/// }
/// ```
///
/// A listed type must implement that trait and `Default`; one that does not is refused by the
/// compiler, at the list. A register function takes `Vec<Box<dyn Trait>>`. Whether each listed type is
/// also an installed pack is the installed set's own test, because the compiler cannot see it.
///
/// The expansion, in the invoking crate:
///
/// ```text
/// pub enum Capability { … }                 one variant per line, in the listed order
/// pub const AVAILABLE: [Capability; N]      every capability, in the listed order
/// impl Capability {
///     resolve, id, section, owning_section, decode_section, decode_section_in, data, owning_data,
///     decode_data_file, data_context, resolves, resolve_subjects, shipped, configuration,
///     decode_configuration, configuration_facts, context_facts, interaction_section, section_schema,
///     configuration_schema, interaction_declarations, biographical, package, version, install,
///     provider, type_name
///     register_extensions                   calls each extension line's register function once,
///                                           with one value of each listed type, lines and types
///                                           in the listed order — what a host does before it
///                                           composes any world
///     extension_types                       each line's trait and its types' Rust paths, for the
///                                           installed set's guard
/// }
/// impl Display for Capability               its id
/// ```
///
/// The macro names no trait and no pack: a new catalog is one more line in the invocation.
#[macro_export]
macro_rules! installed {
    (
        perception: $perception:path;
        $( $rest:tt )+
    ) => {
        $crate::installed! { @lines [$perception] [] $( $rest )+ }
    };
    (
        @lines [$perception:path] [ $( $done:tt )* ]
        extension $Trait:path => $register:path : [ $( $Type:ty , )* ];
        $( $rest:tt )+
    ) => {
        $crate::installed! {
            @lines [$perception] [ $( $done )* { $Trait ; $register ; $( $Type , )* } ]
            $( $rest )+
        }
    };
    (
        @lines [$perception:path] [ $( { $Trait:path ; $register:path ; $( $Type:ty , )* } )* ]
        $( $Variant:ident => $System:ty ),+ $(,)?
    ) => {
        $crate::installed! {
            @catalog
            perception: $perception;
            $( $Variant => $System ),+
        }

        impl Capability {
            /// Registers every extension catalog this build lists (`ARC-62`): each line's register
            /// function, called once with one value of each listed type, lines and types in the listed
            /// order. What a host does before it composes any world; each catalog's owner keeps its
            /// own rules (write-once, a different list refused).
            pub fn register_extensions() {
                $(
                    $register(::std::vec![
                        $(
                            ::std::boxed::Box::new(<$Type as ::core::default::Default>::default())
                                as ::std::boxed::Box<dyn $Trait>,
                        )*
                    ]);
                )*
            }

            /// Each extension line's trait, and the Rust paths of the types it lists, in the listed
            /// order: for the installed set's own guard, which refuses a type that is not an installed
            /// pack, or one listed twice on a line.
            pub fn extension_types()
            -> ::std::vec::Vec<(&'static str, ::std::vec::Vec<&'static str>)> {
                ::std::vec![
                    $(
                        (
                            stringify!($Trait),
                            ::std::vec![ $( ::core::any::type_name::<$Type>(), )* ],
                        ),
                    )*
                ]
            }
        }
    };
    (
        @catalog
        perception: $perception:path;
        $( $Variant:ident => $System:ty ),+
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

            /// Decodes this capability's section as the next value of an authored file's map, in its
            /// context when it has one (`ARC-121` item 2); a pack that owns its section by
            /// `owns_section!` decodes exactly as [`Capability::decode_section`] does.
            ///
            /// # Errors
            ///
            /// As [`Capability::decode_section`], or a refusal naming the system when its section
            /// needs a context it was not given.
            pub fn decode_section_in<'de, A: $crate::__private::MapAccess<'de>>(
                self,
                map: &mut A,
                context: ::core::option::Option<&dyn $crate::__private::OwnerContext>,
            ) -> ::core::result::Result<
                $crate::__private::Arc<dyn $crate::__private::AuthoredContent>,
                A::Error,
            > {
                match self {
                    $(
                        Self::$Variant => {
                            <$System as $crate::SystemPack>::decode_section_in(map, context)
                        }
                    )+
                }
            }

            /// The data directories this capability owns, if any (`ARC-121` item 1).
            pub const fn data(self) -> ::core::option::Option<$crate::DataOwnerDecl> {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::DATA, )+
                }
            }

            /// The capability that owns the data directory written `name`, and that directory, if
            /// this build has one — the first in the listed order (the installed set's guard refuses
            /// two owners of one directory).
            pub fn owning_data(
                name: &str,
            ) -> ::core::option::Option<(Self, $crate::__private::DataDirectory)> {
                AVAILABLE.into_iter().find_map(|capability| {
                    capability
                        .data()
                        .and_then(|data| data.directory(name))
                        .map(|directory| (capability, directory))
                })
            }

            /// Decodes one data file of this capability's `directory` with the owner's own type,
            /// straight from the stream (`DEP-10`). The loader holds the result without knowing its
            /// type.
            ///
            /// # Errors
            ///
            /// The deserializer's error for an invalid file, or a refusal naming the system when it
            /// owns no such directory.
            pub fn decode_data_file<'de, D: $crate::__private::Deserializer<'de>>(
                self,
                directory: $crate::__private::DataDirectory,
                deserializer: D,
            ) -> ::core::result::Result<
                ::std::boxed::Box<dyn ::core::any::Any + ::core::marker::Send + ::core::marker::Sync>,
                D::Error,
            > {
                match self {
                    $(
                        Self::$Variant => <$System as $crate::SystemPack>::decode_data_file(
                            directory,
                            deserializer,
                        ),
                    )+
                }
            }

            /// This capability's context, from its decoded data files in collection order and its
            /// own decoded configuration (`ARC-121` item 1).
            ///
            /// # Errors
            ///
            /// The owner's refusal.
            pub fn data_context(
                self,
                files: ::std::vec::Vec<(
                    $crate::__private::DataOrigin,
                    ::std::boxed::Box<
                        dyn ::core::any::Any + ::core::marker::Send + ::core::marker::Sync,
                    >,
                )>,
                configuration: ::core::option::Option<
                    &dyn $crate::__private::AuthoredConfiguration,
                >,
            ) -> ::core::result::Result<
                $crate::__private::Arc<dyn $crate::__private::OwnerContext>,
                $crate::__private::DataRefusal,
            > {
                match self {
                    $(
                        Self::$Variant => {
                            <$System as $crate::SystemPack>::data_context(files, configuration)
                        }
                    )+
                }
            }

            /// Whether this capability resolves across subjects before seeding (`ARC-121` item 3).
            pub const fn resolves(self) -> bool {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::RESOLVES, )+
                }
            }

            /// This capability's resolution over `subjects`, once, before any seeding; [`None`] for a
            /// capability that does not resolve (`SystemPack::resolve_subjects`).
            ///
            /// # Errors
            ///
            /// The owner's refusal.
            pub fn resolve_subjects(
                self,
                context: ::core::option::Option<&dyn $crate::__private::OwnerContext>,
                configuration: ::core::option::Option<
                    &dyn $crate::__private::AuthoredConfiguration,
                >,
                subjects: &[$crate::__private::ErasedSubject<'_>],
            ) -> ::core::result::Result<
                ::core::option::Option<
                    $crate::__private::Arc<dyn $crate::__private::ResolvedSeeds>,
                >,
                $crate::__private::ResolveRefusal,
            > {
                match self {
                    $(
                        Self::$Variant => <$System as $crate::SystemPack>::resolve_subjects(
                            context,
                            configuration,
                            subjects,
                        ),
                    )+
                }
            }

            /// The data files this capability's pack ships for an owner's directories (`ARC-121`
            /// item 4).
            pub const fn shipped(self) -> &'static [$crate::__private::ShippedData] {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::SHIPS, )+
                }
            }

            /// This capability's id when it takes a world-level configuration (`ARC-61`), else
            /// [`None`].
            pub fn configuration(self) -> ::core::option::Option<$crate::__private::SystemId> {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::CONFIGURATION, )+
                }
            }

            /// Decodes this capability's `configure/<id>.yaml` with the owner's own type, straight
            /// from the stream (`DEP-10`). The loader holds the result without knowing its type.
            ///
            /// # Errors
            ///
            /// The deserializer's error for an invalid configuration, or a refusal naming the system
            /// when it takes none.
            pub fn decode_configuration<'de, D: $crate::__private::Deserializer<'de>>(
                self,
                deserializer: D,
            ) -> ::core::result::Result<
                $crate::__private::Arc<dyn $crate::__private::AuthoredConfiguration>,
                D::Error,
            > {
                match self {
                    $(
                        Self::$Variant => {
                            <$System as $crate::SystemPack>::decode_configuration(deserializer)
                        }
                    )+
                }
            }

            /// The event types this capability's configuration may seed: what the loader admits at
            /// genesis and the drift check compares (`ARC-61`).
            pub fn configuration_facts(self) -> &'static [$crate::__private::EventTypeId] {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::CONFIGURATION_FACTS, )+
                }
            }

            /// The event types this capability states from its data context at genesis (`ARC-121`
            /// item 5): what the drift check compares beside its configuration facts.
            pub fn context_facts(self) -> &'static [$crate::__private::EventTypeId] {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::CONTEXT_FACTS, )+
                }
            }

            /// What this capability's section of the World's Interaction List declares, if it has
            /// one (`ARC-63`): its configured fact, actions and facts — for the tools.
            pub const fn interaction_section(
                self,
            ) -> ::core::option::Option<$crate::interactions::SectionDecl> {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::INTERACTIONS, )+
                }
            }

            /// The JSON Schema of the section this capability owns, if any, for the generated
            /// reference (`ARC-90`).
            pub fn section_schema(self) -> ::core::option::Option<$crate::__private::schemars::Schema> {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::section_schema(), )+
                }
            }

            /// The JSON Schema of this capability's `configure/<id>.yaml`, if it takes one
            /// (`ARC-90`): its configuration, or its section of the World's Interaction List.
            pub fn configuration_schema(
                self,
            ) -> ::core::option::Option<$crate::__private::schemars::Schema> {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::configuration_schema(), )+
                }
            }

            /// What this capability's section of the World's Interaction List declares, with its
            /// parameter and knob schemas, if it has one (`ARC-90`).
            pub fn interaction_declarations(
                self,
            ) -> ::core::option::Option<$crate::interactions::Declarations> {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::interaction_declarations(), )+
                }
            }

            /// Which of this capability's event types belong in a person's objective biography
            /// (`ARC-29`) — each pack's own judgement over its own vocabulary, aggregated here.
            pub fn biographical(self) -> &'static [$crate::__private::EventTypeId] {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::BIOGRAPHICAL, )+
                }
            }

            /// The package identity its pack states (`ARC-53`), for `mineworld packs` — never for a
            /// system: a version is not world state.
            pub const fn package(self) -> $crate::Package {
                match self {
                    $( Self::$Variant => <$System as $crate::SystemPack>::PACKAGE, )+
                }
            }

            /// Its system's contract version: what a save is checked against, a different thing
            /// from its package's release version (`ARC-53`).
            pub fn version(self) -> $crate::__private::SystemVersion {
                match self {
                    $( Self::$Variant => <$System as $crate::__private::System>::VERSION, )+
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
