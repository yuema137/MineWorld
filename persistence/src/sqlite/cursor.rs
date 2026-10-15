//! The two streams a save's history is read through (`ARC-89`, design D-SRB-6).
//!
//! Each cursor opens one read transaction for its whole life (unless its connection is already inside
//! one, whose snapshot it then shares), so a seal another connection commits meanwhile cannot make it
//! miss or repeat a row (I-SRB-5). It walks the sealed blocks its request overlaps in order, holding one
//! decoded block at a time, then pages the live tail by key, [`PAGE`] rows per query. No statement stays
//! open between two calls of `next`.

use std::collections::VecDeque;

use rusqlite::{Connection, OptionalExtension, Transaction};

use super::{
    BLOCK_KEYS, SqliteBackend, block_keys, fact_rows, live_facts_of_span, live_journal, raw_keys,
    signed, storage,
};
use crate::backend::{FactRow, StoredRevision};
use crate::block::{BlockKeys, JournalRow, SealedFact};
use crate::error::PersistError;
use crate::input::WorldRevision;

/// How many live rows one query of the tail reads.
const PAGE: usize = 1_024;

fn skipped(from: u64, to: WorldRevision) -> PersistError {
    PersistError::Damaged {
        detail: format!("the log skips from r{from} to {to}"),
    }
}

/// Pairs journal rows with the facts they recorded: both in order, facts in `EventId` order, which is
/// revision order. A fact left over names the revision it claims.
fn joined(
    journal: Vec<JournalRow>,
    facts: Vec<SealedFact>,
) -> Result<Vec<StoredRevision>, PersistError> {
    let mut facts = facts.into_iter().peekable();
    let mut revisions = Vec::with_capacity(journal.len());
    for row in journal {
        let mut recorded = Vec::new();
        while let Some(fact) = facts.next_if(|fact| fact.revision == row.revision) {
            recorded.push(fact.fact);
        }
        revisions.push(StoredRevision {
            revision: row.revision,
            at: row.at,
            action_id: row.action_id,
            entry: row.entry,
            facts: recorded,
        });
    }
    match facts.next() {
        Some(stray) => Err(PersistError::Damaged {
            detail: format!(
                "fact {} claims {}, out of revision order or with no journal row",
                stray.fact.id, stray.revision
            ),
        }),
        None => Ok(revisions),
    }
}

/// A cursor's place, shared by both kinds.
struct Reading<'c> {
    backend: &'c SqliteBackend,
    /// Held for the cursor's life; dropped (rolled back — it only read) with it.
    _transaction: Option<Transaction<'c>>,
    in_tail: bool,
    finished: bool,
}

impl<'c> Reading<'c> {
    fn open(backend: &'c SqliteBackend) -> Result<Self, PersistError> {
        Ok(Self {
            _transaction: backend.reading()?,
            backend,
            in_tail: false,
            finished: false,
        })
    }

    fn connection(&self) -> &'c Connection {
        &self.backend.connection
    }

    /// The next buffered item, refilling the buffer with `fill` when it is empty; after an error, or
    /// once `fill` finds nothing more, the cursor ends.
    fn step<T>(
        &mut self,
        buffered: &mut VecDeque<T>,
        fill: impl FnOnce(&mut Self, &mut VecDeque<T>) -> Result<(), PersistError>,
    ) -> Option<Result<T, PersistError>> {
        if buffered.is_empty()
            && !self.finished
            && let Err(error) = fill(self, buffered)
        {
            self.finished = true;
            return Some(Err(error));
        }
        let item = buffered.pop_front();
        self.finished |= item.is_none();
        item.map(Ok)
    }
}

/// Every revision after a starting one, journal row and facts together.
pub(super) struct RevisionCursor<'c> {
    reading: Reading<'c>,
    /// The next revision to yield.
    next: u64,
    buffered: VecDeque<StoredRevision>,
}

impl<'c> RevisionCursor<'c> {
    pub(super) fn open(
        backend: &'c SqliteBackend,
        after: WorldRevision,
    ) -> Result<Self, PersistError> {
        Ok(Self {
            reading: Reading::open(backend)?,
            next: after.raw() + 1,
            buffered: VecDeque::new(),
        })
    }

    /// Buffers the next block or page; leaves the buffer empty when the history is exhausted.
    fn fill(
        reading: &mut Reading<'c>,
        next: &mut u64,
        buffered: &mut VecDeque<StoredRevision>,
    ) -> Result<(), PersistError> {
        while buffered.is_empty() {
            if reading.in_tail {
                return Self::page(reading, next, buffered);
            }
            Self::block(reading, next, buffered)?;
        }
        Ok(())
    }

    /// The block holding `next`, if one does; otherwise the tail is next.
    fn block(
        reading: &mut Reading<'c>,
        next: &mut u64,
        buffered: &mut VecDeque<StoredRevision>,
    ) -> Result<(), PersistError> {
        let found = reading
            .connection()
            .query_row(
                &format!(
                    "SELECT {BLOCK_KEYS}, journal, facts FROM sealed WHERE last_revision >= ?1
                     ORDER BY first_revision LIMIT 1"
                ),
                [signed(*next)?],
                |row| {
                    Ok((
                        raw_keys(row)?,
                        row.get::<_, Vec<u8>>(5)?,
                        row.get::<_, Vec<u8>>(6)?,
                    ))
                },
            )
            .optional()
            .map_err(storage)?;
        let Some((keys, journal, facts)) = found else {
            reading.in_tail = true;
            return Ok(());
        };
        let keys: BlockKeys = block_keys(keys)?;
        if keys.span.first.raw() > *next {
            return Err(skipped(*next - 1, keys.span.first));
        }
        let journal = reading.backend.decode_journal(&keys, &journal)?;
        let facts = reading.backend.decode_facts(&keys, &facts)?;
        let floor = *next;
        buffered.extend(
            joined(journal, facts)?
                .into_iter()
                .filter(|stored| stored.revision.raw() >= floor),
        );
        *next = keys.span.last.raw() + 1;
        Ok(())
    }

    /// The next page of the live tail; nothing when it holds nothing more.
    fn page(
        reading: &Reading<'c>,
        next: &mut u64,
        buffered: &mut VecDeque<StoredRevision>,
    ) -> Result<(), PersistError> {
        let connection = reading.connection();
        let journal = live_journal(connection, signed(*next)?, i64::MAX, PAGE)?;
        let (Some(first), Some(last)) = (journal.first(), journal.last()) else {
            return Ok(());
        };
        if first.revision.raw() != *next {
            return Err(skipped(*next - 1, first.revision));
        }
        let (from, to) = (signed(first.revision.raw())?, signed(last.revision.raw())?);
        *next = last.revision.raw() + 1;
        let facts = live_facts_of_span(connection, from, to)?;
        buffered.extend(joined(journal, facts)?);
        Ok(())
    }
}

impl Iterator for RevisionCursor<'_> {
    type Item = Result<StoredRevision, PersistError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            reading,
            next,
            buffered,
        } = self;
        reading.step(buffered, |reading, buffered| {
            Self::fill(reading, next, buffered)
        })
    }
}

/// Every fact after an `EventId`, in `EventId` order.
pub(super) struct FactCursor<'c> {
    reading: Reading<'c>,
    /// The highest `EventId` buffered so far, or the one asked to start after.
    floor: Option<u64>,
    /// The first revision of the last block read.
    last_block: Option<u64>,
    buffered: VecDeque<FactRow>,
}

impl<'c> FactCursor<'c> {
    pub(super) fn open(
        backend: &'c SqliteBackend,
        after: Option<u64>,
    ) -> Result<Self, PersistError> {
        Ok(Self {
            reading: Reading::open(backend)?,
            floor: after,
            last_block: None,
            buffered: VecDeque::new(),
        })
    }

    fn fill(
        reading: &mut Reading<'c>,
        floor: &mut Option<u64>,
        last_block: &mut Option<u64>,
        buffered: &mut VecDeque<FactRow>,
    ) -> Result<(), PersistError> {
        while buffered.is_empty() {
            if reading.in_tail {
                return Self::page(reading, floor, buffered);
            }
            Self::block(reading, floor, last_block, buffered)?;
        }
        Ok(())
    }

    /// The next block holding a fact after `floor`, facts frame only; otherwise the tail is next.
    fn block(
        reading: &mut Reading<'c>,
        floor: &mut Option<u64>,
        last_block: &mut Option<u64>,
        buffered: &mut VecDeque<FactRow>,
    ) -> Result<(), PersistError> {
        let after_block = last_block.map_or(Ok(-1), signed)?;
        let after_fact = floor.map_or(Ok(-1), signed)?;
        let found = reading
            .connection()
            .query_row(
                &format!(
                    "SELECT {BLOCK_KEYS}, facts FROM sealed
                     WHERE first_revision > ?1 AND last_event_id > ?2
                     ORDER BY first_revision LIMIT 1"
                ),
                [after_block, after_fact],
                |row| Ok((raw_keys(row)?, row.get::<_, Vec<u8>>(5)?)),
            )
            .optional()
            .map_err(storage)?;
        let Some((keys, frame)) = found else {
            reading.in_tail = true;
            return Ok(());
        };
        let keys = block_keys(keys)?;
        if last_block.is_some()
            && let (Some(first), Some(floor)) = (keys.first_event_id, *floor)
            && first <= floor
        {
            return Err(PersistError::Damaged {
                detail: format!(
                    "the sealed block {} starts at fact {first}, not after fact {floor}",
                    keys.span
                ),
            });
        }
        let start = *floor;
        buffered.extend(
            reading
                .backend
                .decode_facts(&keys, &frame)?
                .into_iter()
                .map(|sealed| sealed.fact)
                .filter(|fact| start.is_none_or(|start| fact.id > start)),
        );
        *last_block = Some(keys.span.first.raw());
        if let Some(newest) = buffered.back() {
            *floor = Some(newest.id);
        }
        Ok(())
    }

    fn page(
        reading: &Reading<'c>,
        floor: &mut Option<u64>,
        buffered: &mut VecDeque<FactRow>,
    ) -> Result<(), PersistError> {
        let after = floor.map_or(Ok(-1), signed)?;
        let page = fact_rows(
            reading.connection(),
            &format!(
                "SELECT event_id, fact FROM facts WHERE event_id > ?1 ORDER BY event_id LIMIT {PAGE}"
            ),
            after,
        )?;
        if let Some(newest) = page.last() {
            *floor = Some(newest.id);
        }
        buffered.extend(page);
        Ok(())
    }
}

impl Iterator for FactCursor<'_> {
    type Item = Result<FactRow, PersistError>;

    fn next(&mut self) -> Option<Self::Item> {
        let Self {
            reading,
            floor,
            last_block,
            buffered,
        } = self;
        reading.step(buffered, |reading, buffered| {
            Self::fill(reading, floor, last_block, buffered)
        })
    }
}
