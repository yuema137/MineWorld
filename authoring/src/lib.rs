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
//! # Why a crate of its own
//!
//! The kernel and the contracts know nothing about authored files, and a System Pack cannot depend on
//! the loader, which depends on every pack. So the contract sits below the packs, beside the kernel,
//! and names neither a pack nor a file format.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod configuration;
mod content;
mod section;

pub use configuration::{AuthoredConfiguration, DecodeConfiguration, PackConfiguration};
pub use content::{AuthoredContent, Decode};
pub use section::{AuthoredSection, ContentKind, Reference, SectionName, Seeding};
