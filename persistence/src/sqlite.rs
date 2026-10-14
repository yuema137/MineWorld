//! The SQLite backend: the only file in MineWorld that names a SQL type (`DEP-2`).
//!
//! One file, five tables, one transaction per revision:
//!
//! ```text
//! manifest    one row: the save format and the encoded manifest
//! journal     revision → instant, ActionId of a request, encoded entry          the live tail
//! facts       EventId → revision, encoded EventEnvelope                         the live tail
//! sealed      first revision → the span's journal and facts, one block (`ARC-89`)
//! snapshots   revision → encoded WorldSnapshot (a zstd frame of its JSON, `ARC-81`)
//! ```
//!
//! `facts` and `journal` are only ever inserted into, and moved unchanged into a sealed block: when a
//! revision's [`RevisionRow::seal`] names a span, its rows are framed and compressed into one `sealed`
//! row, the block is decoded and compared with them, and only then are they deleted — all inside that
//! revision's transaction ([`crate::block`], D-SRB-11). A row of `snapshots` is deleted only when a
//! revision's [`RevisionRow::retire`] names it. The freed pages are reused by later inserts, and nothing
//! here runs `VACUUM`.
//!
//! A read that spans several queries — a block and the live tail behind it — runs inside one read
//! transaction, so a seal committed by another connection meanwhile cannot make it miss or repeat a row.
//!
//! WAL mode, so a reader (a verifying tool, a test) never blocks the writer. `synchronous` is chosen
//! by [`Durability`]: `FULL` for a hosted world, whose revisions are told to clients; `NORMAL` for a
//! headless bulk run, which in WAL mode is still durable against the process dying and loses recent
//! commits only to a power failure (SQLite, "Write-Ahead Logging" §§ on durability).

use std::cell::Cell;
use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::backend::{
    FactRow, ManifestRow, PersistenceBackend, RevisionRow, RevisionSpan, SealedSpan,
};
use crate::block::{self, BlockKeys, JournalRow, SealedFact};
use crate::error::PersistError;
use crate::input::WorldRevision;

/// The name of the file a save directory holds.
pub const SAVE_FILE: &str = "world.sqlite";

/// How hard a commit tries to survive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Durability {
    /// `synchronous = FULL`: a committed revision survives power loss. For hosted worlds, whose
    /// revisions are told to clients (step-06 I-6).
    PowerLoss,
    /// `synchronous = NORMAL` in WAL mode: a committed revision survives the process being killed,
    /// and only a power failure can lose the newest commits. For headless bulk runs.
    ProcessCrash,
}

/// A save in one SQLite file.
pub struct SqliteBackend {
    connection: Connection,
    /// How many block frames this connection has decompressed — observable, so that a test can show a
    /// resume reads no block (`ARC-89` I-SRB-4) rather than assume it.
    decoded: Cell<u64>,
}

const SCHEMA: &str = "
    CREATE TABLE manifest (
        id      INTEGER PRIMARY KEY CHECK (id = 1),
        format  INTEGER NOT NULL,
        body    BLOB    NOT NULL
    );
    CREATE TABLE journal (
        revision   INTEGER PRIMARY KEY,
        at         INTEGER NOT NULL,
        action_id  INTEGER,
        entry      BLOB    NOT NULL
    );
    CREATE TABLE facts (
        event_id  INTEGER PRIMARY KEY,
        revision  INTEGER NOT NULL REFERENCES journal (revision),
        fact      BLOB    NOT NULL
    );
    CREATE INDEX facts_by_revision ON facts (revision);
    CREATE TABLE sealed (
        first_revision  INTEGER PRIMARY KEY,
        last_revision   INTEGER NOT NULL,
        first_event_id  INTEGER,
        last_event_id   INTEGER,
        max_action_id   INTEGER,
        journal         BLOB    NOT NULL,
        facts           BLOB    NOT NULL,
        CHECK (first_revision <= last_revision)
    );
    CREATE TABLE snapshots (
        revision  INTEGER PRIMARY KEY,
        snapshot  BLOB    NOT NULL
    );
";

/// The key columns of `sealed`, in the order every query here selects them.
const BLOCK_KEYS: &str =
    "first_revision, last_revision, first_event_id, last_event_id, max_action_id";

fn storage(error: rusqlite::Error) -> PersistError {
    PersistError::Storage {
        detail: error.to_string(),
    }
}

/// SQLite stores integers as `i64`; every number this crate stores is a non-negative `u64` far below
/// `i64::MAX`, and a value that is not is refused rather than wrapped.
fn signed(value: u64) -> Result<i64, PersistError> {
    i64::try_from(value).map_err(|_| PersistError::Damaged {
        detail: format!("{value} does not fit a stored integer"),
    })
}

fn unsigned(value: i64) -> Result<u64, PersistError> {
    u64::try_from(value).map_err(|_| PersistError::Damaged {
        detail: format!("a stored integer is negative: {value}"),
    })
}

fn revision(value: i64) -> Result<WorldRevision, PersistError> {
    unsigned(value).map(WorldRevision::from_raw)
}

/// A block's key columns as stored, before they are checked.
type RawKeys = (i64, i64, Option<i64>, Option<i64>, Option<i64>);

fn raw_keys(row: &Row<'_>) -> rusqlite::Result<RawKeys> {
    Ok((
        row.get(0)?,
        row.get(1)?,
        row.get(2)?,
        row.get(3)?,
        row.get(4)?,
    ))
}

fn block_keys(
    (first, last, first_id, last_id, max_action): RawKeys,
) -> Result<BlockKeys, PersistError> {
    Ok(BlockKeys {
        span: RevisionSpan {
            first: revision(first)?,
            last: revision(last)?,
        },
        first_event_id: first_id.map(unsigned).transpose()?,
        last_event_id: last_id.map(unsigned).transpose()?,
        max_action_id: max_action.map(unsigned).transpose()?,
    })
}

impl SqliteBackend {
    /// Creates a new, empty save in `directory`, refusing if one is already there.
    pub fn create(directory: &Path, durability: Durability) -> Result<Self, PersistError> {
        let path = directory.join(SAVE_FILE);
        if path.exists() {
            return Err(PersistError::SaveExists {
                path: path.display().to_string(),
            });
        }
        std::fs::create_dir_all(directory).map_err(|error| PersistError::Storage {
            detail: format!("{}: {error}", directory.display()),
        })?;
        let connection = Self::connect(&path, durability)?;
        connection.execute_batch(SCHEMA).map_err(storage)?;
        Ok(Self::with(connection))
    }

    /// Opens the save in `directory`, refusing if there is none.
    pub fn open(directory: &Path, durability: Durability) -> Result<Self, PersistError> {
        let path = directory.join(SAVE_FILE);
        if !path.exists() {
            return Err(PersistError::NoSave {
                path: path.display().to_string(),
            });
        }
        Ok(Self::with(Self::connect(&path, durability)?))
    }

    const fn with(connection: Connection) -> Self {
        Self {
            connection,
            decoded: Cell::new(0),
        }
    }

    /// Whether `directory` holds a save.
    pub fn exists(directory: &Path) -> bool {
        Self::file(directory).exists()
    }

    /// The file a save in `directory` lives in.
    pub fn file(directory: &Path) -> PathBuf {
        directory.join(SAVE_FILE)
    }

    /// How many block frames this connection has decompressed since it was opened.
    pub fn blocks_decoded(&self) -> u64 {
        self.decoded.get()
    }

    fn connect(path: &Path, durability: Durability) -> Result<Connection, PersistError> {
        let connection = Connection::open(path).map_err(storage)?;
        let mode: String = connection
            .query_row("PRAGMA journal_mode = WAL", [], |row| row.get(0))
            .map_err(storage)?;
        if !mode.eq_ignore_ascii_case("wal") {
            return Err(PersistError::Storage {
                detail: format!("SQLite refused WAL mode and stayed in {mode}"),
            });
        }
        let synchronous = match durability {
            Durability::PowerLoss => "FULL",
            Durability::ProcessCrash => "NORMAL",
        };
        connection
            .execute_batch(&format!(
                "PRAGMA synchronous = {synchronous}; PRAGMA foreign_keys = ON;"
            ))
            .map_err(storage)?;
        connection
            .busy_timeout(Duration::from_secs(5))
            .map_err(storage)?;
        Ok(connection)
    }

    /// A read transaction for a read of several queries, unless this connection is already inside one
    /// (a cursor's), whose snapshot the read then shares.
    fn reading(&self) -> Result<Option<rusqlite::Transaction<'_>>, PersistError> {
        if self.connection.is_autocommit() {
            self.connection
                .unchecked_transaction()
                .map(Some)
                .map_err(storage)
        } else {
            Ok(None)
        }
    }

    fn write_revision(
        transaction: &rusqlite::Transaction<'_>,
        row: &RevisionRow,
    ) -> Result<(), PersistError> {
        let action_id = row.action_id.map(signed).transpose()?;
        transaction
            .execute(
                "INSERT INTO journal (revision, at, action_id, entry) VALUES (?1, ?2, ?3, ?4)",
                params![signed(row.revision.raw())?, row.at, action_id, row.entry],
            )
            .map_err(storage)?;
        for fact in &row.facts {
            transaction
                .execute(
                    "INSERT INTO facts (event_id, revision, fact) VALUES (?1, ?2, ?3)",
                    params![signed(fact.id)?, signed(row.revision.raw())?, fact.bytes],
                )
                .map_err(storage)?;
        }
        if let Some(snapshot) = &row.snapshot {
            transaction
                .execute(
                    "INSERT INTO snapshots (revision, snapshot) VALUES (?1, ?2)",
                    params![signed(row.revision.raw())?, snapshot],
                )
                .map_err(storage)?;
        }
        // Retired after the new snapshot is in, inside the same transaction: the save never holds
        // fewer snapshots than the rule keeps, even for a moment another reader could see (`ARC-81`).
        for retired in &row.retire {
            let deleted = transaction
                .execute(
                    "DELETE FROM snapshots WHERE revision = ?1",
                    [signed(retired.raw())?],
                )
                .map_err(storage)?;
            if deleted != 1 {
                return Err(PersistError::Damaged {
                    detail: format!(
                        "asked to retire the snapshot at {retired}, which is not stored"
                    ),
                });
            }
        }
        if let Some(span) = row.seal {
            Self::seal(transaction, span)?;
        }
        Ok(())
    }

    /// Moves the live rows of `span` into one sealed block (`ARC-89`, D-SRB-11): reads them, frames and
    /// compresses them, decodes its own frames and compares every row with its source, stores the block,
    /// then deletes the rows — facts first, since they reference the journal — and requires the deleted
    /// counts to be the block's. Any disagreement fails the whole revision.
    fn seal(
        transaction: &rusqlite::Transaction<'_>,
        span: RevisionSpan,
    ) -> Result<(), PersistError> {
        let (first, last) = (signed(span.first.raw())?, signed(span.last.raw())?);
        let journal = live_journal(transaction, first, last, usize::MAX)?;
        let facts = live_facts_of_span(transaction, first, last)?;
        let sealed = block::encode_block(span, &journal, &facts)?;
        if block::decode_journal(&sealed.keys, &sealed.journal)? != journal
            || block::decode_facts(&sealed.keys, &sealed.facts)? != facts
        {
            return Err(PersistError::Damaged {
                detail: format!("the block {span} does not decode to the rows it seals"),
            });
        }
        let keys = sealed.keys;
        transaction
            .execute(
                "INSERT INTO sealed (first_revision, last_revision, first_event_id, last_event_id,
                                     max_action_id, journal, facts)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    first,
                    last,
                    keys.first_event_id.map(signed).transpose()?,
                    keys.last_event_id.map(signed).transpose()?,
                    keys.max_action_id.map(signed).transpose()?,
                    sealed.journal,
                    sealed.facts
                ],
            )
            .map_err(storage)?;
        let deleted_facts = transaction
            .execute(
                "DELETE FROM facts WHERE revision BETWEEN ?1 AND ?2",
                [first, last],
            )
            .map_err(storage)?;
        let deleted_journal = transaction
            .execute(
                "DELETE FROM journal WHERE revision BETWEEN ?1 AND ?2",
                [first, last],
            )
            .map_err(storage)?;
        if deleted_facts != facts.len() || deleted_journal != journal.len() {
            return Err(PersistError::Damaged {
                detail: format!(
                    "sealing {span} deleted {deleted_journal} journal rows and {deleted_facts} facts; \
                     the block holds {} and {}",
                    journal.len(),
                    facts.len()
                ),
            });
        }
        Ok(())
    }

    /// The block holding `revision`, with its facts frame and, when asked, its journal frame.
    fn block_holding(
        &self,
        revision: WorldRevision,
    ) -> Result<Option<(BlockKeys, Vec<u8>)>, PersistError> {
        let found = self
            .connection
            .query_row(
                &format!(
                    "SELECT {BLOCK_KEYS}, facts FROM sealed
                     WHERE first_revision <= ?1 AND last_revision >= ?1"
                ),
                [signed(revision.raw())?],
                |row| Ok((raw_keys(row)?, row.get::<_, Vec<u8>>(5)?)),
            )
            .optional()
            .map_err(storage)?;
        found
            .map(|(keys, facts)| Ok((block_keys(keys)?, facts)))
            .transpose()
    }

    fn decode_facts(
        &self,
        keys: &BlockKeys,
        frame: &[u8],
    ) -> Result<Vec<SealedFact>, PersistError> {
        self.decoded.set(self.decoded.get() + 1);
        block::decode_facts(keys, frame)
    }

    fn decode_journal(
        &self,
        keys: &BlockKeys,
        frame: &[u8],
    ) -> Result<Vec<JournalRow>, PersistError> {
        self.decoded.set(self.decoded.get() + 1);
        block::decode_journal(keys, frame)
    }

    /// The blocks whose span ends after `after`, in revision order, with their journal frames.
    fn blocks_after(
        &self,
        after: WorldRevision,
    ) -> Result<Vec<(BlockKeys, Vec<u8>)>, PersistError> {
        let mut statement = self
            .connection
            .prepare(&format!(
                "SELECT {BLOCK_KEYS}, journal FROM sealed WHERE last_revision > ?1
                 ORDER BY first_revision"
            ))
            .map_err(storage)?;
        let rows = statement
            .query_map([signed(after.raw())?], |row| {
                Ok((raw_keys(row)?, row.get(5)?))
            })
            .map_err(storage)?;
        let mut blocks = Vec::new();
        for row in rows {
            let (keys, journal) = row.map_err(storage)?;
            blocks.push((block_keys(keys)?, journal));
        }
        Ok(blocks)
    }
}

/// Up to `limit` live journal rows of revisions `first ..= last`, in revision order.
fn live_journal(
    connection: &Connection,
    first: i64,
    last: i64,
    limit: usize,
) -> Result<Vec<JournalRow>, PersistError> {
    let mut statement = connection
        .prepare(
            "SELECT revision, at, action_id, entry FROM journal
             WHERE revision BETWEEN ?1 AND ?2 ORDER BY revision LIMIT ?3",
        )
        .map_err(storage)?;
    let limit = i64::try_from(limit).unwrap_or(i64::MAX);
    let rows = statement
        .query_map([first, last, limit], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<i64>>(2)?,
                row.get::<_, Vec<u8>>(3)?,
            ))
        })
        .map_err(storage)?;
    let mut journal = Vec::new();
    for row in rows {
        let (stored, at, action_id, entry) = row.map_err(storage)?;
        journal.push(JournalRow {
            revision: revision(stored)?,
            at,
            action_id: action_id.map(unsigned).transpose()?,
            entry,
        });
    }
    Ok(journal)
}

/// Every live fact recorded by revisions `first ..= last`, in `EventId` order.
fn live_facts_of_span(
    connection: &Connection,
    first: i64,
    last: i64,
) -> Result<Vec<SealedFact>, PersistError> {
    let mut statement = connection
        .prepare(
            "SELECT event_id, revision, fact FROM facts
             WHERE revision BETWEEN ?1 AND ?2 ORDER BY event_id",
        )
        .map_err(storage)?;
    let rows = statement
        .query_map([first, last], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Vec<u8>>(2)?,
            ))
        })
        .map_err(storage)?;
    let mut facts = Vec::new();
    for row in rows {
        let (id, stored, bytes) = row.map_err(storage)?;
        facts.push(SealedFact {
            revision: revision(stored)?,
            fact: FactRow {
                id: unsigned(id)?,
                bytes,
            },
        });
    }
    Ok(facts)
}

fn fact_rows(
    connection: &Connection,
    sql: &str,
    parameter: i64,
) -> Result<Vec<FactRow>, PersistError> {
    let mut statement = connection.prepare(sql).map_err(storage)?;
    let rows = statement
        .query_map([parameter], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
        })
        .map_err(storage)?;
    let mut facts = Vec::new();
    for row in rows {
        let (id, bytes) = row.map_err(storage)?;
        facts.push(FactRow {
            id: unsigned(id)?,
            bytes,
        });
    }
    Ok(facts)
}

impl PersistenceBackend for SqliteBackend {
    fn initialize(
        &mut self,
        manifest: &ManifestRow,
        genesis: &RevisionRow,
    ) -> Result<(), PersistError> {
        let transaction = self.connection.transaction().map_err(storage)?;
        transaction
            .execute(
                "INSERT INTO manifest (id, format, body) VALUES (1, ?1, ?2)",
                params![manifest.format, manifest.body],
            )
            .map_err(storage)?;
        Self::write_revision(&transaction, genesis)?;
        transaction.commit().map_err(storage)
    }

    fn manifest(&self) -> Result<ManifestRow, PersistError> {
        self.connection
            .query_row(
                "SELECT format, body FROM manifest WHERE id = 1",
                [],
                |row| {
                    Ok(ManifestRow {
                        format: row.get(0)?,
                        body: row.get(1)?,
                    })
                },
            )
            .optional()
            .map_err(storage)?
            .ok_or_else(|| PersistError::Damaged {
                detail: "the save has no manifest".to_owned(),
            })
    }

    fn head(&self) -> Result<WorldRevision, PersistError> {
        let head: Option<i64> = self
            .connection
            .query_row(
                "SELECT MAX(head) FROM (SELECT MAX(revision) AS head FROM journal
                                        UNION ALL SELECT MAX(last_revision) FROM sealed)",
                [],
                |row| row.get(0),
            )
            .map_err(storage)?;
        let head = head.ok_or_else(|| PersistError::Damaged {
            detail: "the save has no journal".to_owned(),
        })?;
        revision(head)
    }

    fn commit(&mut self, revision: &RevisionRow) -> Result<(), PersistError> {
        let transaction = self.connection.transaction().map_err(storage)?;
        Self::write_revision(&transaction, revision)?;
        transaction.commit().map_err(storage)
    }

    fn journal_after(
        &self,
        after: WorldRevision,
    ) -> Result<Vec<(WorldRevision, Vec<u8>)>, PersistError> {
        let _read = self.reading()?;
        let mut entries = Vec::new();
        for (keys, journal) in self.blocks_after(after)? {
            for row in self.decode_journal(&keys, &journal)? {
                if row.revision > after {
                    entries.push((row.revision, row.entry));
                }
            }
        }
        let from = entries.last().map_or(after, |(revision, _)| *revision);
        for row in live_journal(
            &self.connection,
            signed(from.raw())? + 1,
            i64::MAX,
            usize::MAX,
        )? {
            entries.push((row.revision, row.entry));
        }
        Ok(entries)
    }

    fn facts_of(&self, revision: WorldRevision) -> Result<Vec<FactRow>, PersistError> {
        let _read = self.reading()?;
        if let Some((keys, frame)) = self.block_holding(revision)? {
            return Ok(self
                .decode_facts(&keys, &frame)?
                .into_iter()
                .filter(|sealed| sealed.revision == revision)
                .map(|sealed| sealed.fact)
                .collect());
        }
        fact_rows(
            &self.connection,
            "SELECT event_id, fact FROM facts WHERE revision = ?1 ORDER BY event_id",
            signed(revision.raw())?,
        )
    }

    fn last_facts(&self, count: usize) -> Result<Vec<FactRow>, PersistError> {
        let _read = self.reading()?;
        let limit = signed(u64::try_from(count).unwrap_or(u64::MAX >> 1))?;
        let mut live = fact_rows(
            &self.connection,
            "SELECT event_id, fact FROM facts ORDER BY event_id DESC LIMIT ?1",
            limit,
        )?;
        live.reverse();
        // Older facts than the tail holds come from blocks, newest block first, only as many as asked.
        let mut older: Vec<Vec<FactRow>> = Vec::new();
        let mut wanted = count.saturating_sub(live.len());
        if wanted > 0 {
            let mut statement = self
                .connection
                .prepare(&format!(
                    "SELECT {BLOCK_KEYS}, facts FROM sealed WHERE last_event_id IS NOT NULL
                     ORDER BY first_revision DESC"
                ))
                .map_err(storage)?;
            let mut rows = statement.query([]).map_err(storage)?;
            while wanted > 0 {
                let Some(row) = rows.next().map_err(storage)? else {
                    break;
                };
                let keys = block_keys(raw_keys(row).map_err(storage)?)?;
                let frame: Vec<u8> = row.get(5).map_err(storage)?;
                let facts = self.decode_facts(&keys, &frame)?;
                let skip = facts.len().saturating_sub(wanted);
                wanted -= facts.len() - skip;
                older.push(
                    facts
                        .into_iter()
                        .skip(skip)
                        .map(|sealed| sealed.fact)
                        .collect(),
                );
            }
        }
        Ok(older.into_iter().rev().flatten().chain(live).collect())
    }

    fn latest_snapshot(
        &self,
        at_most: WorldRevision,
    ) -> Result<Option<(WorldRevision, Vec<u8>)>, PersistError> {
        let found: Option<(i64, Vec<u8>)> = self
            .connection
            .query_row(
                "SELECT revision, snapshot FROM snapshots WHERE revision <= ?1
                 ORDER BY revision DESC LIMIT 1",
                [signed(at_most.raw())?],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()
            .map_err(storage)?;
        found
            .map(|(stored, bytes)| Ok((revision(stored)?, bytes)))
            .transpose()
    }

    fn snapshot_revisions(&self) -> Result<Vec<WorldRevision>, PersistError> {
        let mut statement = self
            .connection
            .prepare("SELECT revision FROM snapshots ORDER BY revision")
            .map_err(storage)?;
        let rows = statement
            .query_map([], |row| row.get::<_, i64>(0))
            .map_err(storage)?;
        let mut revisions = Vec::new();
        for row in rows {
            revisions.push(revision(row.map_err(storage)?)?);
        }
        Ok(revisions)
    }

    fn snapshot_at(&self, revision: WorldRevision) -> Result<Option<Vec<u8>>, PersistError> {
        self.connection
            .query_row(
                "SELECT snapshot FROM snapshots WHERE revision = ?1",
                [signed(revision.raw())?],
                |row| row.get(0),
            )
            .optional()
            .map_err(storage)
    }

    fn highest_action_id(&self) -> Result<Option<u64>, PersistError> {
        let highest: Option<i64> = self
            .connection
            .query_row(
                "SELECT MAX(highest) FROM (SELECT MAX(action_id) AS highest FROM journal
                                           UNION ALL SELECT MAX(max_action_id) FROM sealed)",
                [],
                |row| row.get(0),
            )
            .map_err(storage)?;
        highest.map(unsigned).transpose()
    }

    fn sealed(&self) -> Result<Vec<SealedSpan>, PersistError> {
        let mut statement = self
            .connection
            .prepare(
                "SELECT first_revision, last_revision, length(journal) + length(facts) FROM sealed
                 ORDER BY first_revision",
            )
            .map_err(storage)?;
        let rows = statement
            .query_map([], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, i64>(1)?,
                    row.get::<_, i64>(2)?,
                ))
            })
            .map_err(storage)?;
        let mut spans = Vec::new();
        for row in rows {
            let (first, last, stored) = row.map_err(storage)?;
            spans.push(SealedSpan {
                span: RevisionSpan {
                    first: revision(first)?,
                    last: revision(last)?,
                },
                stored_bytes: unsigned(stored)?,
            });
        }
        Ok(spans)
    }

    fn checkpoint(&mut self, revision: WorldRevision, snapshot: &[u8]) -> Result<(), PersistError> {
        self.connection
            .execute(
                // A snapshot already at this revision is the same state by construction; it is
                // kept rather than rewritten.
                "INSERT OR IGNORE INTO snapshots (revision, snapshot) VALUES (?1, ?2)",
                params![signed(revision.raw())?, snapshot],
            )
            .map_err(storage)?;
        Ok(())
    }
}
