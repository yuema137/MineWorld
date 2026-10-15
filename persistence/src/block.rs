//! A sealed block: the journal rows and fact rows of one span of revisions, framed and compressed
//! (`docs/DECISIONS.md` `ARC-89`; design D-SRB-3 … D-SRB-5).
//!
//! A block is a storage representation of opaque rows, as a snapshot frame is of a snapshot's JSON. It
//! decodes no journal entry and no fact: a row's bytes go in and come out unchanged, with their key
//! columns. Replay compares re-executed entries and facts with the rows a block yields, never with the
//! block's compressed bytes, so no codec setting can change a verdict.
//!
//! ```text
//! journal frame   per row, in revision order:
//!                   revision u64 LE · at i64 LE · action id (u8 0 | u8 1 · u64 LE) · length u32 LE · entry
//! facts frame     per row, in EventId order:
//!                   event id u64 LE · revision u64 LE · length u32 LE · fact
//! each frame      zstd level 9, checksum and content size in the frame, single-threaded, no dictionary
//! ```
//!
//! Decoding checks what a block claims about itself: its journal holds every revision of its span once,
//! in order; every fact lies inside the span; event ids strictly increase and match the block's key
//! columns; nothing trails the last row. Any failure is [`PersistError::Damaged`] naming the span.

use crate::backend::{FactRow, RevisionSpan};
use crate::error::PersistError;
use crate::input::WorldRevision;

/// The zstd level every block frame is compressed at (QSRB-2): one level, fixed, so a block's bytes
/// are a function of its rows alone on every platform (`DEP-43`, `AC-8`).
pub const BLOCK_LEVEL: i32 = 9;

/// One journal row as the save stores it: its revision, the input's instant, the request's `ActionId`
/// when the input is a request, and the encoded entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalRow {
    /// Which revision.
    pub revision: WorldRevision,
    /// The instant of its input, in simulated seconds.
    pub at: i64,
    /// The request's `ActionId`, when the input is a request.
    pub action_id: Option<u64>,
    /// The encoded journal entry.
    pub entry: Vec<u8>,
}

/// One fact row as the save stores it: the fact with the revision that recorded it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedFact {
    /// The revision that recorded the fact.
    pub revision: WorldRevision,
    /// The fact's `EventId` and its encoded envelope.
    pub fact: FactRow,
}

/// What a block says about itself in its key columns, so that a reader finds the block it needs
/// without decompressing any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockKeys {
    /// The revisions the block holds, every one of them.
    pub span: RevisionSpan,
    /// The first fact's `EventId`, or `None` when the span recorded no fact.
    pub first_event_id: Option<u64>,
    /// The last fact's `EventId`, or `None` when the span recorded no fact.
    pub last_event_id: Option<u64>,
    /// The highest `ActionId` a request of the span carried, or `None` when it held no request.
    pub max_action_id: Option<u64>,
}

/// A sealed block, ready to store: its key columns and its two compressed frames.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SealedBlock {
    /// The key columns.
    pub keys: BlockKeys,
    /// The journal frame, compressed.
    pub journal: Vec<u8>,
    /// The facts frame, compressed.
    pub facts: Vec<u8>,
}

fn damaged(span: RevisionSpan, what: impl core::fmt::Display) -> PersistError {
    PersistError::Damaged {
        detail: format!("the sealed block {span} {what}"),
    }
}

/// Frames and compresses the rows of `span`: every journal row of the span, in revision order, and
/// every fact the span recorded, in `EventId` order. Rows that do not fill the span exactly, or facts
/// out of order or outside it, are refused rather than sealed.
pub fn encode_block(
    span: RevisionSpan,
    journal: &[JournalRow],
    facts: &[SealedFact],
) -> Result<SealedBlock, PersistError> {
    let max_action_id = check_journal(span, journal)?;
    let (first_event_id, last_event_id) = check_facts(span, facts)?;
    Ok(SealedBlock {
        keys: BlockKeys {
            span,
            first_event_id,
            last_event_id,
            max_action_id,
        },
        journal: compress(span, &frame_journal(span, journal)?)?,
        facts: compress(span, &frame_facts(span, facts)?)?,
    })
}

/// The journal rows a block's journal frame holds, checked against its key columns.
pub fn decode_journal(keys: &BlockKeys, frame: &[u8]) -> Result<Vec<JournalRow>, PersistError> {
    let span = keys.span;
    let bytes = decompress(span, "journal", frame)?;
    let mut reader = Reader::new(span, &bytes);
    let mut rows = Vec::new();
    while !reader.done() {
        let revision = WorldRevision::from_raw(reader.u64()?);
        let at = reader.i64()?;
        let action_id = match reader.u8()? {
            0 => None,
            1 => Some(reader.u64()?),
            other => return Err(damaged(span, format!("has an action-id tag {other}"))),
        };
        let entry = reader.bytes()?;
        rows.push(JournalRow {
            revision,
            at,
            action_id,
            entry,
        });
    }
    if check_journal(span, &rows)? != keys.max_action_id {
        return Err(damaged(
            span,
            "disagrees with its max_action_id column".to_owned(),
        ));
    }
    Ok(rows)
}

/// The fact rows a block's facts frame holds, checked against its key columns.
pub fn decode_facts(keys: &BlockKeys, frame: &[u8]) -> Result<Vec<SealedFact>, PersistError> {
    let span = keys.span;
    let bytes = decompress(span, "facts", frame)?;
    let mut reader = Reader::new(span, &bytes);
    let mut rows = Vec::new();
    while !reader.done() {
        let id = reader.u64()?;
        let revision = WorldRevision::from_raw(reader.u64()?);
        let bytes = reader.bytes()?;
        rows.push(SealedFact {
            revision,
            fact: FactRow { id, bytes },
        });
    }
    if check_facts(span, &rows)? != (keys.first_event_id, keys.last_event_id) {
        return Err(damaged(span, "disagrees with its event-id columns"));
    }
    Ok(rows)
}

/// Requires the journal rows to be every revision of the span, once, in order. Returns the highest
/// `ActionId` they carry.
fn check_journal(span: RevisionSpan, rows: &[JournalRow]) -> Result<Option<u64>, PersistError> {
    let mut expected = span.first.raw();
    for row in rows {
        if row.revision.raw() != expected {
            return Err(damaged(
                span,
                format!(
                    "skips from r{} to {} in its journal",
                    expected - 1,
                    row.revision
                ),
            ));
        }
        expected += 1;
    }
    if expected != span.last.raw() + 1 {
        return Err(damaged(
            span,
            format!("ends its journal at r{}", expected - 1),
        ));
    }
    Ok(rows.iter().filter_map(|row| row.action_id).max())
}

/// Requires every fact to lie inside the span and event ids to strictly increase. Returns the first
/// and last event ids.
fn check_facts(
    span: RevisionSpan,
    rows: &[SealedFact],
) -> Result<(Option<u64>, Option<u64>), PersistError> {
    let mut previous: Option<u64> = None;
    for row in rows {
        if !span.contains(row.revision) {
            return Err(damaged(
                span,
                format!("holds fact {} of {}", row.fact.id, row.revision),
            ));
        }
        if previous.is_some_and(|previous| row.fact.id <= previous) {
            return Err(damaged(
                span,
                format!("holds fact {} out of EventId order", row.fact.id),
            ));
        }
        previous = Some(row.fact.id);
    }
    Ok((rows.first().map(|row| row.fact.id), previous))
}

fn length(span: RevisionSpan, bytes: &[u8]) -> Result<[u8; 4], PersistError> {
    u32::try_from(bytes.len())
        .map(u32::to_le_bytes)
        .map_err(|_| damaged(span, format!("has a row of {} bytes", bytes.len())))
}

fn frame_journal(span: RevisionSpan, rows: &[JournalRow]) -> Result<Vec<u8>, PersistError> {
    let mut frame = Vec::with_capacity(rows.iter().map(|row| 29 + row.entry.len()).sum());
    for row in rows {
        frame.extend_from_slice(&row.revision.raw().to_le_bytes());
        frame.extend_from_slice(&row.at.to_le_bytes());
        match row.action_id {
            None => frame.push(0),
            Some(action) => {
                frame.push(1);
                frame.extend_from_slice(&action.to_le_bytes());
            }
        }
        frame.extend_from_slice(&length(span, &row.entry)?);
        frame.extend_from_slice(&row.entry);
    }
    Ok(frame)
}

fn frame_facts(span: RevisionSpan, rows: &[SealedFact]) -> Result<Vec<u8>, PersistError> {
    let mut frame = Vec::with_capacity(rows.iter().map(|row| 20 + row.fact.bytes.len()).sum());
    for row in rows {
        frame.extend_from_slice(&row.fact.id.to_le_bytes());
        frame.extend_from_slice(&row.revision.raw().to_le_bytes());
        frame.extend_from_slice(&length(span, &row.fact.bytes)?);
        frame.extend_from_slice(&row.fact.bytes);
    }
    Ok(frame)
}

/// zstd at [`BLOCK_LEVEL`], checksum and content size in the frame, no dictionary, single-threaded —
/// as `format::encode_snapshot` compresses a snapshot at its own level.
fn compress(span: RevisionSpan, framed: &[u8]) -> Result<Vec<u8>, PersistError> {
    let failed = |error: std::io::Error| damaged(span, format!("did not compress: {error}"));
    let mut compressor = zstd::bulk::Compressor::new(BLOCK_LEVEL).map_err(failed)?;
    compressor.include_checksum(true).map_err(failed)?;
    compressor.include_contentsize(true).map_err(failed)?;
    compressor.compress(framed).map_err(failed)
}

/// The most a frame may claim to hold: far above any span (`ARC-89` revisits at 64 MiB of rows), and a
/// bound on what one damaged header can make a reader allocate.
const MAX_FRAME: u64 = 1 << 30;

/// Decompresses a frame into one allocation of exactly the size its header states — a block is
/// decoded every few milliseconds by a streaming reader, and a buffer grown by doubling leaves freed
/// regions of every size behind it, which the allocator keeps (F-SRB-2). The checksum is verified.
fn decompress(span: RevisionSpan, which: &str, frame: &[u8]) -> Result<Vec<u8>, PersistError> {
    let failed = |what: String| damaged(span, format!("does not decompress ({which}): {what}"));
    let size = zstd::zstd_safe::get_frame_content_size(frame)
        .map_err(|_| failed("no zstd frame header".to_owned()))?
        .ok_or_else(|| failed("no content size in the frame".to_owned()))?;
    if size > MAX_FRAME {
        return Err(failed(format!("it claims {size} bytes")));
    }
    let capacity = usize::try_from(size).map_err(|_| failed(format!("it claims {size} bytes")))?;
    let bytes =
        zstd::bulk::decompress(frame, capacity).map_err(|error| failed(error.to_string()))?;
    if bytes.len() != capacity {
        return Err(failed(format!(
            "it holds {} bytes, its header {capacity}",
            bytes.len()
        )));
    }
    Ok(bytes)
}

/// Reads a frame's fields in order, refusing a frame that ends inside a row.
struct Reader<'a> {
    span: RevisionSpan,
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    const fn new(span: RevisionSpan, bytes: &'a [u8]) -> Self {
        Self { span, bytes, at: 0 }
    }

    const fn done(&self) -> bool {
        self.at == self.bytes.len()
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], PersistError> {
        let end = self
            .at
            .checked_add(count)
            .filter(|end| *end <= self.bytes.len())
            .ok_or_else(|| damaged(self.span, format!("ends inside a row at byte {}", self.at)))?;
        let taken = &self.bytes[self.at..end];
        self.at = end;
        Ok(taken)
    }

    fn array<const N: usize>(&mut self) -> Result<[u8; N], PersistError> {
        let mut array = [0; N];
        array.copy_from_slice(self.take(N)?);
        Ok(array)
    }

    fn u8(&mut self) -> Result<u8, PersistError> {
        Ok(self.array::<1>()?[0])
    }

    fn u64(&mut self) -> Result<u64, PersistError> {
        Ok(u64::from_le_bytes(self.array()?))
    }

    fn i64(&mut self) -> Result<i64, PersistError> {
        Ok(i64::from_le_bytes(self.array()?))
    }

    fn bytes(&mut self) -> Result<Vec<u8>, PersistError> {
        let length = u32::from_le_bytes(self.array()?);
        let length =
            usize::try_from(length).map_err(|_| damaged(self.span, "has an oversized row"))?;
        Ok(self.take(length)?.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(first: u64, last: u64) -> RevisionSpan {
        RevisionSpan {
            first: WorldRevision::from_raw(first),
            last: WorldRevision::from_raw(last),
        }
    }

    fn journal(first: u64, last: u64) -> Vec<JournalRow> {
        (first..=last)
            .map(|revision| JournalRow {
                revision: WorldRevision::from_raw(revision),
                at: i64::try_from(revision).expect("small") * 37 - 5,
                action_id: (revision % 3 != 0).then_some(revision * 10),
                entry: format!("{{\"entry\":{revision}}}").into_bytes(),
            })
            .collect()
    }

    /// Two facts for every even revision, none for odd ones: some revisions record nothing.
    fn facts(first: u64, last: u64) -> Vec<SealedFact> {
        let mut id = 1_000;
        let mut rows = Vec::new();
        for revision in (first..=last).filter(|revision| revision % 2 == 0) {
            for which in 0..2 {
                id += 1;
                rows.push(SealedFact {
                    revision: WorldRevision::from_raw(revision),
                    fact: FactRow {
                        id,
                        bytes: format!("{{\"fact\":{id},\"of\":{revision},\"n\":{which}}}")
                            .into_bytes(),
                    },
                });
            }
        }
        rows
    }

    fn detail(result: Result<impl core::fmt::Debug, PersistError>) -> String {
        match result {
            Err(PersistError::Damaged { detail }) => detail,
            other => panic!("expected Damaged, got {other:?}"),
        }
    }

    #[test]
    fn a_block_returns_every_row_byte_for_byte_with_its_keys() {
        let (rows, facts) = (journal(5, 104), facts(5, 104));
        let block = encode_block(span(5, 104), &rows, &facts).expect("seals");
        assert_eq!(
            block.keys,
            BlockKeys {
                span: span(5, 104),
                first_event_id: Some(1_001),
                last_event_id: Some(1_100),
                max_action_id: Some(1_040),
            }
        );
        assert_eq!(
            decode_journal(&block.keys, &block.journal).expect("reads"),
            rows
        );
        assert_eq!(
            decode_facts(&block.keys, &block.facts).expect("reads"),
            facts
        );
        let raw: usize = rows.iter().map(|row| row.entry.len()).sum::<usize>()
            + facts.iter().map(|row| row.fact.bytes.len()).sum::<usize>();
        assert!(
            block.journal.len() + block.facts.len() < raw,
            "the frames compress repetitive rows: {} + {} B of {raw} B",
            block.journal.len(),
            block.facts.len()
        );
    }

    #[test]
    fn a_genesis_sized_row_and_a_span_without_facts_round_trip() {
        let mut rows = journal(1, 3);
        rows[0].entry = (0..300_000_u32)
            .map(|index| u8::try_from(index % 251).expect("small"))
            .collect();
        let block = encode_block(span(1, 3), &rows, &[]).expect("seals");
        assert_eq!(
            (block.keys.first_event_id, block.keys.last_event_id),
            (None, None)
        );
        assert_eq!(
            decode_journal(&block.keys, &block.journal).expect("reads"),
            rows
        );
        assert!(
            decode_facts(&block.keys, &block.facts)
                .expect("reads")
                .is_empty()
        );
    }

    #[test]
    fn rows_that_do_not_fill_the_span_or_facts_out_of_order_are_never_sealed() {
        let mut gap = journal(1, 10);
        gap.remove(4);
        assert!(detail(encode_block(span(1, 10), &gap, &[])).contains("skips from r4 to r6"));
        assert!(
            detail(encode_block(span(1, 11), &journal(1, 10), &[]))
                .contains("ends its journal at r10")
        );
        let outside = facts(1, 12);
        assert!(detail(encode_block(span(1, 10), &journal(1, 10), &outside)).contains("of r12"));
        let mut unordered = facts(1, 10);
        unordered.swap(1, 2);
        assert!(
            detail(encode_block(span(1, 10), &journal(1, 10), &unordered))
                .contains("out of EventId order")
        );
    }

    /// Frames written past the encoder's checks — damage, or another writer — are refused by the
    /// decoder, each naming the block's span, and never panic.
    #[test]
    fn every_inconsistent_frame_is_damage_naming_the_span() {
        let whole = span(1, 10);
        let keys = encode_block(whole, &journal(1, 10), &facts(1, 10))
            .expect("seals")
            .keys;
        let journal_frame = |rows: &[JournalRow]| {
            compress(whole, &frame_journal(whole, rows).expect("frames")).expect("compresses")
        };
        let facts_frame = |rows: &[SealedFact]| {
            compress(whole, &frame_facts(whole, rows).expect("frames")).expect("compresses")
        };
        let named = |detail: String, what: &str| {
            assert!(
                detail.starts_with("the sealed block r1–r10 ") && detail.contains(what),
                "{detail}"
            );
        };

        let mut gap = journal(1, 10);
        gap.remove(2);
        named(
            detail(decode_journal(&keys, &journal_frame(&gap))),
            "skips from r2 to r4",
        );
        let mut outside = facts(1, 10);
        outside[3].revision = WorldRevision::from_raw(11);
        named(
            detail(decode_facts(&keys, &facts_frame(&outside))),
            "of r11",
        );
        let mut unordered = facts(1, 10);
        unordered.swap(0, 1);
        named(
            detail(decode_facts(&keys, &facts_frame(&unordered))),
            "out of EventId order",
        );
        let mut fewer = facts(1, 10);
        fewer.pop();
        named(
            detail(decode_facts(&keys, &facts_frame(&fewer))),
            "event-id columns",
        );
        let mut other_action = journal(1, 10);
        other_action[0].action_id = Some(99_999);
        named(
            detail(decode_journal(&keys, &journal_frame(&other_action))),
            "max_action_id column",
        );

        let mut trailing = frame_facts(whole, &facts(1, 10)).expect("frames");
        trailing.push(0);
        named(
            detail(decode_facts(
                &keys,
                &compress(whole, &trailing).expect("compresses"),
            )),
            "ends inside a row",
        );
        let mut truncated = frame_journal(whole, &journal(1, 10)).expect("frames");
        truncated.truncate(truncated.len() - 3);
        named(
            detail(decode_journal(
                &keys,
                &compress(whole, &truncated).expect("compresses"),
            )),
            "ends inside a row",
        );
        let mut tag = frame_journal(whole, &journal(1, 1)).expect("frames");
        tag[16] = 7;
        named(
            detail(decode_journal(
                &keys,
                &compress(whole, &tag).expect("compresses"),
            )),
            "action-id tag 7",
        );

        named(
            detail(decode_facts(&keys, b"{}")),
            "does not decompress (facts)",
        );
        let mut flipped = facts_frame(&facts(1, 10));
        let middle = flipped.len() / 2;
        flipped[middle] ^= 0x55;
        named(
            detail(decode_facts(&keys, &flipped)),
            "does not decompress (facts)",
        );
        let cut = facts_frame(&facts(1, 10));
        named(
            detail(decode_facts(&keys, &cut[..cut.len() - 4])),
            "does not decompress (facts)",
        );
    }
}
