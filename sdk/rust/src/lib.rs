//! Writing a MineWorld **System Pack** in Rust, and installing it into a build.
//!
//! A System Pack is a crate under `systems/` that implements the kernel's `System`. This crate adds
//! the part the *build* needs: what the World Pack loader and the tools must know about a pack
//! beyond `System`, said once by the pack itself (`docs/DECISIONS.md` `ARC-33`).
//!
//! ```text
//! a System Pack     implements SystemPack: its package identity (PACKAGE = package!(), required),
//!                   its biographical event types, the authored section it owns (owns_section!), and
//!                   how that section is decoded, and the world-level configuration it takes
//!                   (configures!, ARC-61) — each but the first defaulting to none
//! the installed set systems/installed: one installed! invocation, one line per pack — the build's
//!                   only list of System Packs, expanded into the catalog the loader reads
//! extension         installed!'s optional lines, one per catalog: a pack-owned trait, its register
//!                   function, and the types other packs implement it with (ARC-62)
//! interactions      a pack's section of the World's Interaction List: InteractionSection,
//!                   parameters!, interactions!, and the lookups permits / parameters / consequence
//!                   (ARC-63 … ARC-65)
//! ```
//!
//! Installing a pack in MVP-0 is a new directory under `systems/`, two lines in `systems/installed/`
//! — its Cargo dependency and its `installed!` line — and a rebuild (`docs/MODULE_SPEC.md` §3.1).
//! Nothing else is edited. Installing without a rebuild is `ARC-8`'s WASM component model, outside
//! MVP-0.
//!
//! # Why a crate of its own
//!
//! Every pack implements [`SystemPack`], so the trait cannot live in a pack or in the loader, which
//! depends on the installed set. It sits below the packs, beside the kernel and the authoring seam,
//! and names no pack and no perception trait: the installed set passes its perception trait to
//! [`installed!`] by path.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod installed;
pub mod interactions;
mod pack;
mod section;

pub use mineworld_packages::{Package, package};
pub use pack::SystemPack;
pub use section::SectionOwner;

/// What the macros expand to, so that an installed set needs no dependency but this crate, its
/// perception trait's crate, and its packs. Not part of the API.
#[doc(hidden)]
pub mod __private {
    pub use std::sync::Arc;

    pub use mineworld_authoring::{
        AuthoredConfiguration, AuthoredContent, Decode, DecodeConfiguration, PackConfiguration,
    };
    pub use mineworld_contracts::{EventTypeId, SystemId};
    pub use mineworld_kernel::{KernelError, System, SystemIdentity, SystemVersion, World};
    pub use serde::de::{DeserializeSeed, Deserializer, Error as DeError, MapAccess};
}
