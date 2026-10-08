//! A code pack's identity, recorded from its own `Cargo.toml` at compile time.

/// What a code pack's `Cargo.toml` says about it, recorded where the pack is compiled.
///
/// Five strings, unchecked — Cargo guarantees the name and the version, and nothing else — and whether
/// the pack is **bundled** (`DECISIONS.md` `ARC-54` point 2). The strings become a validated
/// [`Identity`](crate::Identity) through [`Identity::of_code_pack`](crate::Identity::of_code_pack), the
/// one place every carrier's values are checked. Made by [`package!`](crate::package), never by hand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Package {
    name: &'static str,
    version: &'static str,
    license: &'static str,
    authors: &'static str,
    repository: &'static str,
    bundled: bool,
}

/// The framework workspace's root, as this crate's own manifest directory without its last component.
/// Compared at compile time only; no path is stored in a [`Package`].
const PACKAGES_DIR: &str = env!("CARGO_MANIFEST_DIR");

impl Package {
    /// What [`package!`](crate::package) expands to. Not called by hand: a pack's identity is its
    /// Cargo fields, and a second statement of them could disagree. `manifest_dir` is the pack's
    /// `CARGO_MANIFEST_DIR`; only whether it lies under the framework workspace is kept.
    #[doc(hidden)]
    #[must_use]
    pub const fn declared(
        name: &'static str,
        version: &'static str,
        license: &'static str,
        authors: &'static str,
        repository: &'static str,
        manifest_dir: &str,
    ) -> Self {
        Self {
            name,
            version,
            license,
            authors,
            repository,
            bundled: compiled_under(manifest_dir, PACKAGES_DIR),
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

    /// Whether the pack was compiled from the framework's own workspace, and is therefore versioned
    /// with it (`ARC-54` point 2). Every other code pack is third-party.
    #[must_use]
    pub const fn bundled(&self) -> bool {
        self.bundled
    }
}

/// Whether `pack_dir` lies under the workspace root that holds `packages_dir`: the root is
/// `packages_dir` up to and including its last path separator, so `/a/repo2/x` is not under the root of
/// `/a/repo/packages`. A `packages_dir` with no separator has no root, and nothing is under it.
#[doc(hidden)]
#[must_use]
pub const fn compiled_under(pack_dir: &str, packages_dir: &str) -> bool {
    let root = packages_dir.as_bytes();
    let mut length = root.len();
    while length > 0 && root[length - 1] != b'/' && root[length - 1] != b'\\' {
        length -= 1;
    }
    let pack = pack_dir.as_bytes();
    if length == 0 || pack.len() < length {
        return false;
    }
    let mut index = 0;
    while index < length {
        if pack[index] != root[index] {
            return false;
        }
        index += 1;
    }
    true
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
            ::core::env!("CARGO_MANIFEST_DIR"),
        )
    };
}
