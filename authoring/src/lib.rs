//! The seam between authored World Pack content and the System Pack that owns it
//! (`docs/DECISIONS.md` `ARC-31`, `docs/MODULE_SPEC.md` §4.1).
//!
//! Authored content becomes world state only as a genesis fact its owner reduces (`ARC-15`). A person
//! file's `name:` is `naming`'s, its `routine:` is `schedule`'s, and neither the World Pack loader nor
//! this crate knows what either means:
//!
//! ```text
//! a System Pack     implements AuthoredSection: the key it owns, the files that may carry it, the
//!                   type it is validated by, the entities it names, and the facts it becomes
//! the loader        decodes the section with that type, checks what every section shares (owner
//!                   enabled, file kind, references declared), and records what `seed` returns
//! ```
//!
//! The same shape serves a world-level configuration (`ARC-61`): a pack implements
//! [`PackConfiguration`] for `configure/<id>.yaml`, which is about no entity, and the loader holds it as
//! [`AuthoredConfiguration`] until the world it seeds exists.
//!
//! And a pack may own **data** (`ARC-121`): directories of World and Entity Packs whose files it decodes
//! into one context ([`OwnsData`]), a section decoded in that context ([`SectionInContext`]), one
//! resolution across every subject before seeding ([`ResolvesSubjects`]), and files it ships compiled in
//! ([`ShippedData`]). The loader holds each type erased, as it holds a section.
//!
//! # Why a crate of its own
//!
//! The kernel and the contracts know nothing about authored files, and a System Pack cannot depend on
//! the loader, which depends on every pack. So the contract sits below the packs, beside the kernel,
//! and names neither a pack nor a file format.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod attachment;
mod classes;
mod configuration;
mod content;
mod data;
mod section;

pub use attachment::{ATTACHMENT_DIRECTORY, ATTACHMENT_MAX_BYTES, Attached, Attachment};
pub use classes::{ClassDefinition, ClassName, Classed, EntityClasses};
pub use configuration::{
    AuthoredConfiguration, ConfigurationContext, ConfigurationRefusal, DecodeConfiguration,
    EntryAt, PackConfiguration,
};
pub use content::{AuthoredContent, Decode};
pub use data::{
    DataDirectory, DataOrigin, DataRefusal, DataSource, DecodeIn, ErasedSubject, OwnerContext,
    OwnsData, ResolveRefusal, ResolvedSeeds, ResolvesSubjects, SectionInContext, ShippedData,
    Subject, data_context, decode_data_file, resolve_erased,
};
pub use section::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
