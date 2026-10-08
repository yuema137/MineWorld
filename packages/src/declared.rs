//! A code pack's identity, recorded from its own `Cargo.toml` at compile time.

/// What a code pack's `Cargo.toml` says about it, recorded where the pack is compiled.
///
/// Five strings, unchecked: Cargo guarantees the name and the version, and nothing else. They become a
/// validated [`Identity`](crate::Identity) through
/// [`Identity::of_code_pack`](crate::Identity::of_code_pack), the one place every carrier's values are
/// checked. Made by [`package!`](crate::package), never by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Package {
    name: &'static str,
    version: &'static str,
    license: &'static str,
    authors: &'static str,
    repository: &'static str,
}

impl Package {
    /// What [`package!`](crate::package) expands to. Not called by hand: a pack's identity is its
    /// Cargo fields, and a second statement of them could disagree.
    #[doc(hidden)]
    #[must_use]
    pub const fn declared(
        name: &'static str,
        version: &'static str,
        license: &'static str,
        authors: &'static str,
        repository: &'static str,
    ) -> Self {
        Self {
            name,
            version,
            license,
            authors,
            repository,
        }
    }

    /// The Cargo package name, which is the pack's id.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        self.name
    }

    /// The Cargo version.
    #[must_use]
    pub const fn version(&self) -> &'static str {
        self.version
    }

    /// The Cargo `license` field, as written; empty when the manifest states none.
    #[must_use]
    pub const fn license(&self) -> &'static str {
        self.license
    }

    /// The Cargo `authors`, as Cargo joins them: separated by `:`; empty when none are stated.
    #[must_use]
    pub const fn authors(&self) -> &'static str {
        self.authors
    }

    /// The Cargo `repository` field; empty when the manifest states none.
    #[must_use]
    pub const fn repository(&self) -> &'static str {
        self.repository
    }
}

/// The [`Package`] of the crate this is written in, from its own `Cargo.toml`.
///
/// An expression, so one form serves every carrier:
///
/// ```text
/// impl SystemPack for ConversationSystem {
///     const PACKAGE: mineworld_sdk::Package = mineworld_sdk::package!();
/// }
///
/// pub const PACKAGE: mineworld_packages::Package = mineworld_packages::package!();
/// ```
///
/// `env!` is expanded where this macro is invoked, so each pack records its own crate's fields, at
/// compile time; the binary never runs Cargo or reads a source tree.
#[macro_export]
macro_rules! package {
    () => {
        $crate::Package::declared(
            ::core::env!("CARGO_PKG_NAME"),
            ::core::env!("CARGO_PKG_VERSION"),
            ::core::env!("CARGO_PKG_LICENSE"),
            ::core::env!("CARGO_PKG_AUTHORS"),
            ::core::env!("CARGO_PKG_REPOSITORY"),
        )
    };
}
