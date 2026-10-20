//! The model of a Person's attributes (`DECISIONS.md` `ARC-120` … `ARC-125`; `MODULE_SPEC.md` §§3.1, 4.1;
//! `CORE_CONCEPTS.md` §4.5).
//!
//! Every user of attributes depends on this crate and never on the `attributes` System Pack: the pack
//! itself, readers holding typed handles, owners projecting their own state, and the tools.
//!
//! ```text
//! id           Namespace, Slug, QualifiedId, ValueSlug; the nearest ids for a refusal
//! kind         the closed kinds and units; the handle markers and SystemReadable (not Text)
//! value        AttributeValue, ScaledInt (exact decimals), the decoding of an authored value
//! declaration  the attributes/ and vocabularies/ file formats; one Declaration
//! catalogue    Catalogue::build, frozen; AttributeError, every refusal of MODULE_SPEC §4.1
//! section      the person file's attributes: section, decoded against the catalogue
//! traits       the traits component, Handle<K>, read
//! derive       age and band, computed per read and never stored
//! render       phrases, words, and the holder's and another observer's views; the block (schema 2)
//! disclosed    Disclosed, the typed reader of a perceived entity's blocks; the composition
//! owner        AttributeOwner, disclose_owned
//! facts        the pack's five facts, the catalogue Process, RunningCatalogue
//! ```
//!
//! No float is stored, compared or computed (I-c-10).

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![deny(clippy::float_arithmetic)]
// A refusal is the cold path of a load that happens once per world; it carries the file, the line and
// the names a reader needs, and boxing every one of them would buy nothing measurable.
#![allow(clippy::result_large_err)]

use mineworld_contracts::SystemId;

pub mod backend;
pub mod catalogue;
pub mod config;
pub mod context;
pub mod declaration;
pub mod derive;
pub mod disclosed;
pub mod facts;
pub mod id;
pub mod kind;
pub mod lock;
pub mod mask;
pub mod owner;
pub mod profile;
pub mod render;
pub mod sample;
pub mod section;
pub mod traits;
pub mod value;

/// The `attributes` System Pack's id: the owner of the catalogue, the section, `traits` and the facts.
pub const ATTRIBUTES: SystemId = SystemId::from_static("attributes");

pub use catalogue::{AttributeError, Catalogue, Composition, Projection, Uses};
pub use context::{AttributesContext, Population, PopulationIdentity};
pub use declaration::{DataFile, Declaration, Mutability, Visibility};
pub use facts::{
    AttributeChangeProposed, AttributesComposed, AttributesHanded, AttributesId, CatalogueProcess,
    RunningCatalogue, TraitChanged, TraitsStated,
};
pub use id::{QualifiedId, ValueSlug};
pub use kind::{
    Category, Flag, Instant, Kind, KindName, Level, Quantity, Set, SystemReadable, Text,
};
pub use owner::{AttributeOwner, disclose_owned};
pub use disclosed::Disclosed;
pub use render::{Block, Entry, View};
pub use section::{Authored, Restriction, SectionIn};
pub use traits::{Handle, Traits, read};
pub use value::{AttributeValue, LevelValue, ScaledInt};
