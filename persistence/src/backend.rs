//! Where a save's rows are kept: the interface `DEP-2` isolates the storage engine behind.
//!
//! A backend stores **encoded rows** and knows nothing about what they mean. Encoding and decoding
//! belong to [`crate::format`]; replay and its byte comparisons to [`crate::replay`]. That split is
//! what lets replay compare a regenerated fact with the logged one byte for byte — the backend hands
//! back exactly the bytes it was given — and what keeps world semantics independent of the engine
//! (`INV-14`): a backend cannot interpret a row, so it cannot change one's meaning.

use crate::error::PersistError;
use crate::input::WorldRevision;

/// One revision, encoded, ready to be committed in one transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RevisionRow {
    /// Which revision.
    pub revision: WorldRevision,
    /// The instant of its input, in simulated seconds.
    pub at: i64,
    /// The request's `ActionId`, when the input is a request — so that the highest one a world has
    /// issued is a query rather than a scan.
    pub action_id: Option<u64>,
    /// The encoded journal entry: the input and what the world answered.
    pub entry: Vec<u8>,
    /// The facts the input recorded, in `EventId` order, each with its identity.
    pub facts: Vec<FactRow>,
    /// The encoded world state after this revision, when this revision is checkpointed.
    pub snapshot: Option<Vec<u8>>,
    /// Snapshot revisions to delete in this revision's transaction, after `snapshot` is written —
    /// the ones the retention rule no longer keeps (`ARC-81`). Never this revision; never a later one.
    /// Each must be stored: a backend refuses to retire a snapshot it does not hold.
    pub retire: Vec<WorldRevision>,
    /// The span whose live rows this revision's transaction seals into one block, after the row, its
    /// snapshot and its retirements are written (`ARC-89`). Always entirely below `revision`; every row
    /// of it must be live, and a backend refuses the revision otherwise.
    pub seal: Option<RevisionSpan>,
}

/// One stored revision as a reader of history receives it: the journal row and the facts it recorded,
/// with their bytes exactly as committed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoredRevision {
    /// Which revision.
    pub revision: WorldRevision,
    /// The instant of its input, in simulated seconds.
    pub at: i64,
    /// The request's `ActionId`, when the input is a request.
    pub action_id: Option<u64>,
    /// The encoded journal entry.
    pub entry: Vec<u8>,
    /// The facts it recorded, in `EventId` order.
    pub facts: Vec<FactRow>,
}

/// A stream of stored revisions, in revision order. It stops after the first error it yields.
pub type Revisions<'a> = Box<dyn Iterator<Item = Result<StoredRevision, PersistError>> + 'a>;

/// A stream of facts, in `EventId` order. It stops after the first error it yields.
pub type Facts<'a> = Box<dyn Iterator<Item = Result<FactRow, PersistError>> + 'a>;

/// A sealed block as a person reading the save sees it: the revisions it covers and what it costs on
/// disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SealedSpan {
    /// The revisions the block holds.
    pub span: RevisionSpan,
    /// Its two compressed frames, in bytes.
    pub stored_bytes: u64,
}

/// A closed range of revisions, `first ..= last` — the span of one sealed block (`ARC-89`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RevisionSpan {
    /// The first revision of the span.
    pub first: WorldRevision,
    /// The last revision of the span, at or after `first`.
    pub last: WorldRevision,
}

impl RevisionSpan {
    /// How many revisions the span covers (never zero).
    pub const fn revisions(self) -> u64 {
        self.last.raw() + 1 - self.first.raw()
    }

    /// Whether `revision` lies inside the span.
    pub fn contains(self, revision: WorldRevision) -> bool {
        self.first <= revision && revision <= self.last
    }
}

impl core::fmt::Display for RevisionSpan {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}–{}", self.first, self.last)
    }
}

/// One fact, encoded, filed under its `EventId`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FactRow {
    /// The fact's `EventId`.
    pub id: u64,
    /// The encoded `EventEnvelope`.
    pub bytes: Vec<u8>,
}

/// The manifest, encoded, with its format version stored beside it so that the version can be read
/// — and a save refused — before anything is decoded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManifestRow {
    /// The save format the save was written in.
    pub format: u32,
    /// The encoded manifest.
    pub body: Vec<u8>,
}

/// A save's storage.
///
/// One implementation today, [`SqliteBackend`](crate::SqliteBackend); `PostgresBackend` is the later
/// one `ARCHITECTURE.md` §8 anticipates. Every write is all-or-nothing: a commit that fails, or a
/// process that dies during one, leaves the previous revision as the head.
pub trait PersistenceBackend {
    /// Writes the manifest and the genesis revision of a new save, together.
    fn initialize(
        &mut self,
        manifest: &ManifestRow,
        genesis: &RevisionRow,
    ) -> Result<(), PersistError>;

    /// The manifest.
    fn manifest(&self) -> Result<ManifestRow, PersistError>;

    /// The newest committed revision.
    fn head(&self) -> Result<WorldRevision, PersistError>;

    /// Commits one revision: its journal row, its facts, its snapshot, the retirement of the
    /// snapshots it names and the seal of the span it names, all or nothing.
    fn commit(&mut self, revision: &RevisionRow) -> Result<(), PersistError>;

    /// Every revision after `after`, in revision order, each with its journal row and its facts — a
    /// stream, not a list (`ARC-89`, D-SRB-6). The cursor reads one consistent history for its whole
    /// life, whatever another connection commits meanwhile, and holds at most one decoded block.
    fn revisions_after(&self, after: WorldRevision) -> Result<Revisions<'_>, PersistError>;

    /// Every fact after the `EventId` `after` (every fact, for `None`), in `EventId` order — a stream,
    /// with the same consistency and memory as [`revisions_after`](Self::revisions_after).
    fn facts_after(&self, after: Option<u64>) -> Result<Facts<'_>, PersistError>;

    /// The facts one revision recorded, in `EventId` order.
    fn facts_of(&self, revision: WorldRevision) -> Result<Vec<FactRow>, PersistError>;

    /// The newest `count` facts, oldest first.
    fn last_facts(&self, count: usize) -> Result<Vec<FactRow>, PersistError>;

    /// The newest snapshot at or below `at_most`: `(revision, encoded snapshot)`.
    fn latest_snapshot(
        &self,
        at_most: WorldRevision,
    ) -> Result<Option<(WorldRevision, Vec<u8>)>, PersistError>;

    /// The revisions that carry a snapshot, in order.
    fn snapshot_revisions(&self) -> Result<Vec<WorldRevision>, PersistError>;

    /// The snapshot stored at exactly `revision`.
    fn snapshot_at(&self, revision: WorldRevision) -> Result<Option<Vec<u8>>, PersistError>;

    /// The highest `ActionId` any journaled request carried.
    fn highest_action_id(&self) -> Result<Option<u64>, PersistError>;

    /// The sealed blocks, in revision order (`ARC-89`). The live tail starts after the last one.
    fn sealed(&self) -> Result<Vec<SealedSpan>, PersistError>;

    /// Writes a snapshot for a revision already committed — a checkpoint, as at a clean shutdown.
    fn checkpoint(&mut self, revision: WorldRevision, snapshot: &[u8]) -> Result<(), PersistError>;
}
