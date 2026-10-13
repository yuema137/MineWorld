//! The SQLite backend: the only file in MineWorld that names a SQL type (`DEP-2`).
//!
//! One file, four tables, one transaction per revision:
//!
//! ```text
//! manifest    one row: the save format and the encoded manifest
//! journal     revision → instant, ActionId of a request, encoded entry
//! facts       EventId → revision, encoded EventEnvelope
//! snapshots   revision → encoded WorldSnapshot (a zstd frame of its JSON, `ARC-81`)
//! ```
//!
//! `facts` and `journal` are only ever inserted into. A row of `snapshots` is deleted only when a
//! revision's [`RevisionRow::retire`] names it, inside that revision's transaction; the freed pages are
//! reused by later inserts, and nothing here runs `VACUUM`.
//!
//! WAL mode, so a reader (a verifying tool, a test) never blocks the writer. `synchronous` is chosen
//! by [`Durability`]: `FULL` for a hosted world, whose revisions are told to clients; `NORMAL` for a
//! headless bulk run, which in WAL mode is still durable against the process dying and loses recent
//! commits only to a power failure (SQLite, "Write-Ahead Logging" §§ on durability).

use std::path::{Path, PathBuf};
use std::time::Duration;

use rusqlite::{Connection, OptionalExtension, params};

use crate::backend::{FactRow, ManifestRow, PersistenceBackend, RevisionRow};
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
    CREATE TABLE snapshots (
        revision  INTEGER PRIMARY KEY REFERENCES journal (revision),
        snapshot  BLOB    NOT NULL
    );
";

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
        Ok(Self { connection })
    }

    /// Opens the save in `directory`, refusing if there is none.
    pub fn open(directory: &Path, durability: Durability) -> Result<Self, PersistError> {
        let path = directory.join(SAVE_FILE);
        if !path.exists() {
            return Err(PersistError::NoSave {
                path: path.display().to_string(),
            });
        }
        Ok(Self {
            connection: Self::connect(&path, durability)?,
        })
    }

    /// Whether `directory` holds a save.
    pub fn exists(directory: &Path) -> bool {
        Self::file(directory).exists()
    }

    /// The file a save in `directory` lives in.
    pub fn file(directory: &Path) -> PathBuf {
        directory.join(SAVE_FILE)
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
        Ok(())
    }

    fn fact_rows(&self, sql: &str, parameter: i64) -> Result<Vec<FactRow>, PersistError> {
        let mut statement = self.connection.prepare(sql).map_err(storage)?;
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
            .query_row("SELECT MAX(revision) FROM journal", [], |row| row.get(0))
            .map_err(storage)?;
        let head = head.ok_or_else(|| PersistError::Damaged {
            detail: "the save has no journal".to_owned(),
        })?;
        Ok(WorldRevision::from_raw(unsigned(head)?))
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
        let mut statement = self
            .connection
            .prepare("SELECT revision, entry FROM journal WHERE revision > ?1 ORDER BY revision")
            .map_err(storage)?;
        let rows = statement
            .query_map([signed(after.raw())?], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, Vec<u8>>(1)?))
            })
            .map_err(storage)?;
        let mut entries = Vec::new();
        for row in rows {
            let (revision, entry) = row.map_err(storage)?;
            entries.push((WorldRevision::from_raw(unsigned(revision)?), entry));
        }
        Ok(entries)
    }

    fn facts_of(&self, revision: WorldRevision) -> Result<Vec<FactRow>, PersistError> {
        self.fact_rows(
            "SELECT event_id, fact FROM facts WHERE revision = ?1 ORDER BY event_id",
            signed(revision.raw())?,
        )
    }

    fn last_facts(&self, count: usize) -> Result<Vec<FactRow>, PersistError> {
        let limit = signed(u64::try_from(count).unwrap_or(u64::MAX >> 1))?;
        let mut facts = self.fact_rows(
            "SELECT event_id, fact FROM facts ORDER BY event_id DESC LIMIT ?1",
            limit,
        )?;
        facts.reverse();
        Ok(facts)
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
            .map(|(revision, bytes)| Ok((WorldRevision::from_raw(unsigned(revision)?), bytes)))
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
            revisions.push(WorldRevision::from_raw(unsigned(row.map_err(storage)?)?));
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
            .query_row("SELECT MAX(action_id) FROM journal", [], |row| row.get(0))
            .map_err(storage)?;
        highest.map(unsigned).transpose()
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
