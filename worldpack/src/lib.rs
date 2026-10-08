//! Reading a **World Pack** and loading the world it describes.
//!
//! A World Pack is what a world author produces: *semantic configuration and population, not code*
//! (`docs/MODULE_SPEC.md` §4). This crate is the other half of that sentence — the code that reads
//! one.
//!
//! ```text
//! worlds/social-cafe/           format    what a pack may say
//! ├── world.yaml                read      directory → WorldPack, refusing a bad one by name
//! ├── people/{alice,bob,…}.yaml load      WorldPack → a running World
//! ├── places/{cafe,…}.yaml      catalog   which System Packs this build provides (the installed
//! ├── items/*.yaml                        set, systems/installed) and the two the format names
//! └── organizations/*.yaml
//! ```
//!
//! `items/` and `organizations/` are optional (`DECISIONS.md` `ARC-36`); an item file declares an item
//! kind, not one object.
//!
//! ```no_run
//! use mineworld_contracts::WorldTime;
//! use mineworld_worldpack::WorldPack;
//!
//! # fn main() -> Result<(), mineworld_worldpack::PackError> {
//! let pack = WorldPack::read("worlds/social-cafe")?;   // validate
//! let loaded = pack.load(WorldTime::EPOCH)?;           // build the world it describes
//!
//! println!("{} — {} entities", pack.name(), loaded.world().entities().len());
//! # Ok(())
//! # }
//! ```
//!
//! # The three things a World Pack may not do, and how each is prevented here
//!
//! ```text
//! no secrets      the format has no field for a key, a token or an endpoint, and unknown fields
//!                 are refused — so a pack cannot carry one even by accident (MODULE_SPEC §4.1)
//! no renderer     nothing in the format names a model, a texture, a colour or a scene; a world
//!                 names a Presentation Pack, which MVP-0 does not load yet
//! no rules        a pack states what is true, never what is allowed. The facts it may state are
//!                 bounded by what its enabled systems own (see `catalog`), so it cannot invent a
//!                 fact its world has no owner for
//! ```
//!
//! # Where this crate sits
//!
//! ```text
//! worldpack  →  systems/installed  →  every installed System Pack  →  kernel  →  contracts
//!     └──────→  systems/{presence,movement}  (the packs `location` and `passages` belong to)
//! ```
//!
//! One way, as `ENGINEERING_STANDARDS.md` §4 requires. It depends on System Packs because composing a
//! world means installing them — through the build's installed set, which names them so this crate
//! does not (`DECISIONS.md` `ARC-33`); **nothing depends on it in the other direction**, and in particular
//! `mineworld-server` does not — a transport that knew what a conversation was would be the layering
//! inverted. The binary that puts the two together is `tools/cli`.
//!
//! # Determinism is this crate's responsibility
//!
//! A pack is loaded into a world whose entity ids and event ids are allocated in the order this crate
//! creates and states things. That order therefore reaches the event log, and `AC-12` requires it to be
//! a property of the pack rather than of the run: see [`load`]'s own documentation for the order and
//! why every container here is a `BTreeMap`.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod catalog;
pub mod content;
pub mod error;
pub mod format;
pub mod load;
pub mod read;

pub use catalog::Capability;
pub use error::{ContentKind, Declared, PackError};
pub use format::{
    AuthoredItem, AuthoredLocation, AuthoredOrganization, AuthoredPerson, AuthoredPlace,
    AuthoredPosition, FoundSection, SectionState, WorldIdentity, WorldManifest,
};
pub use load::{AssembledWorld, ComposedWorld, LoadedWorld, RunningWorld};
pub use read::{MANIFEST, PackageFields, WorldPack};
