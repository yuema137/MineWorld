//! MineWorld's saves: a world that survives the death of the process hosting it.
//!
//! A persisted world is three append-only things in one SQLite file, committed together one
//! revision at a time (`docs/DECISIONS.md` `ARC-25`):
//!
//! ```text
//! facts       every Event, in EventId order          HISTORY — what happened (INV-11)
//! journal     every input that moved the world       what it was asked, and when
//! snapshots   the whole state at some revisions      a checkpoint, never an authority alone
//! ```
//!
//! A restart restores the newest snapshot and re-executes the journal after it through the kernel's
//! own pipeline; every re-executed input must reproduce its logged answer and facts byte for byte, or
//! the load is refused. [`verify`] re-executes from genesis and checks every stored snapshot too;
//! [`verify_from`] does the same from a retained snapshot.
//!
//! Facts and journal rows are never deleted. Snapshots are retained by rule — genesis, an anchor every
//! 64 scheduled snapshots, the newest two — and stored as zstd frames of their JSON (`ARC-81`).
//!
//! ```text
//! input      WorldInput, Outcome, WorldRevision      what a journal entry is
//! world      PersistentWorld                         a world and its save; create, resume, drive,
//!                                                    and which snapshots it keeps
//! replay     verify, verify_from                     re-execution and the byte comparison
//! backend    PersistenceBackend                      where rows are kept — DEP-2's isolating trait
//! sqlite     SqliteBackend, Durability               the one backend, and the one file naming SQL
//! format     Manifest, SAVE_FORMAT                   how rows are encoded, and the version refused
//! ```
//!
//! Nothing here reduces a fact, writes a component, or knows a domain concept: state is changed only
//! by the kernel's `genesis`, `dispatch` and `advance_to`, called exactly as a live world calls them.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod backend;
pub mod error;
pub mod format;
pub mod input;
pub mod replay;
pub mod sqlite;
pub mod world;

pub use backend::PersistenceBackend;
pub use error::PersistError;
pub use format::{Manifest, SAVE_FORMAT};
pub use input::{JournalEntry, Outcome, WorldInput, WorldRevision};
pub use replay::{Verified, verify, verify_from};
pub use sqlite::{Durability, SAVE_FILE, SqliteBackend};
pub use world::{Creation, DEFAULT_SNAPSHOT_INTERVAL, PersistentWorld, Resumed};
