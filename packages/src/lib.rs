//! What a MineWorld **package** is: the identity every pack states, and where it states it
//! (`docs/DECISIONS.md` `ARC-53`, `docs/PACKAGE_FORMAT.md` §5.0).
//!
//! ```text
//! id          1–64 of a–z, 0–9, '-'; a letter first; no '--'; no trailing '-'
//! version     semver
//! type        system-pack · controller-pack · world-pack · presentation-pack · entity-pack
//! mineworld   the framework versions a data pack works with (a semver range)
//! license     an SPDX expression
//! provenance  authors; the repository where stated
//! ```
//!
//! Each fact is stated once, by the carrier that already says who the pack is:
//!
//! ```text
//! a code pack        its Cargo.toml, recorded at compile time by package!() into a Package
//! a World Pack       its world.yaml (read by the World Pack loader, which hands the fields here)
//! a data pack        its pack.yaml (read here)
//! ```
//!
//! Whatever the carrier, the values become one validated [`Identity`] through the same types, so a
//! version or a licence means the same thing whoever states it.
//!
//! # A leaf
//!
//! This crate depends on no MineWorld crate. The SDK re-exports [`Package`] and [`package!`] for
//! System Packs; the rule controller uses them directly, so that naming its identity does not put the
//! kernel into a crate that reaches none. A package identity is never world state: no system reads
//! it, and no fact or save carries it.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod declared;
mod error;
mod found;
mod identity;
mod manifest;
mod policy;
mod resolve;
mod roots;

pub use declared::{Package, compiled_under};
pub use error::PackageError;
pub use found::{DataPack, WORLD_FILE, packs_in};
pub use identity::{
    Compatibility, Identity, License, PackId, PackType, Version, distinct, framework_version,
};
pub use manifest::{
    ENTITY_ITEM_EXTENSION, ENTITY_ITEMS, PACK_FILE, STYLE_FILE, check_entity_layout,
    check_style_manifest, read_pack_file,
};
pub use policy::LicencePolicy;
pub use resolve::{
    CodePack, Composition, FoundPack, Installed, Resolved, Source, SystemPackUsed,
    WorldRequirements, resolve,
};
pub use roots::{PACKS_VARIABLE, PackRoots};

/// The framework's release version: the version every framework crate and every bundled pack share
/// (`ARC-53`). This crate is one of them, so its own Cargo version is the framework's.
pub const FRAMEWORK_VERSION: &str = env!("CARGO_PKG_VERSION");
