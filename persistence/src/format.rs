//! How a save's rows are encoded, and the one version number that covers them all.
//!
//! JSON throughout (`DEP-5`): a journal entry, a fact, a snapshot and the manifest are each the
//! `serde_json` encoding of their contract or kernel type. Every type involved orders its collections
//! by key, so equal values encode to equal bytes — which is what replay compares.
//!
//! A snapshot is stored as a zstd frame of that JSON (`ARC-81`, `DEP-43`): [`encode_snapshot`],
//! [`snapshot_json`], [`decode_snapshot`]. The frame is only how the bytes are kept. Every comparison
//! of state is of the JSON inside it, so no codec setting can change a replay verdict.

use std::io::Cursor;

use mineworld_kernel::{InstalledSystemRecord, WorldSnapshot};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::PersistError;
use crate::input::WorldRevision;

/// The save format this code writes and the only one it reads.
///
/// Raised whenever the encoding of any row changes — including a change to `WorldSnapshot`,
/// `EventEnvelope` or a journal entry's shape. A save of another format is refused, never decoded on
/// a guess (step-06 I-5).
///
/// History: 1 — S5. 2 — S6 (`ARC-26`): a `SystemDeclaration`, stored in every snapshot's composition
/// and in the manifest, gained the owners of the vocabularies it borrows. 3 — SR (`ARC-81`): a stored
/// snapshot is a zstd frame of its JSON, and snapshots are retained by rule.
pub const SAVE_FORMAT: u32 = 3;

/// The zstd level a snapshot is compressed at (`DEP-43`): one level, fixed, so the stored bytes are a
/// function of the JSON alone.
const SNAPSHOT_LEVEL: i32 = 3;

/// What a save says about itself.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    /// Which world this is, as a 32-digit lowercase hexadecimal number: allocated when the world
    /// was created and kept for its whole life, across every restart (`server/PROTOCOL.md` §5).
    pub instance: String,
    /// The World Pack the world was created from, for a person reading the save. Informative: a
    /// resumed world is checked by its composition, not by this name.
    pub pack: String,
    /// The systems the world is composed of, in registration order.
    pub composition: Vec<InstalledSystemRecord>,
}

impl Manifest {
    /// The instance as the number it is.
    pub fn instance_number(&self) -> Result<u128, PersistError> {
        u128::from_str_radix(&self.instance, 16).map_err(|_| PersistError::Damaged {
            detail: format!(
                "the manifest's instance '{}' is not a number",
                self.instance
            ),
        })
    }
}

/// The instance written the way the manifest stores it.
pub fn instance_text(instance: u128) -> String {
    format!("{instance:032x}")
}

/// Encodes a row.
pub fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, PersistError> {
    serde_json::to_vec(value).map_err(|error| PersistError::Damaged {
        detail: format!("a row did not encode: {error}"),
    })
}

/// Decodes a row, naming what it was meant to be when it does not decode.
pub fn decode<T: DeserializeOwned>(bytes: &[u8], what: &str) -> Result<T, PersistError> {
    serde_json::from_slice(bytes).map_err(|error| PersistError::Damaged {
        detail: format!("a stored {what} does not decode: {error}"),
    })
}

/// Encodes a snapshot for storage: a zstd frame — level 3, checksum and content size in the frame,
/// no dictionary, single-threaded — of exactly the bytes [`encode`] gives the snapshot.
pub fn encode_snapshot(snapshot: &WorldSnapshot) -> Result<Vec<u8>, PersistError> {
    let json = encode(snapshot)?;
    let failed = |error: std::io::Error| PersistError::Damaged {
        detail: format!("a snapshot did not compress: {error}"),
    };
    let mut compressor = zstd::bulk::Compressor::new(SNAPSHOT_LEVEL).map_err(failed)?;
    compressor.include_checksum(true).map_err(failed)?;
    compressor.include_contentsize(true).map_err(failed)?;
    compressor.compress(&json).map_err(failed)
}

/// The JSON a stored snapshot holds — the bytes replay and verification compare (`ARC-81`).
pub fn snapshot_json(stored: &[u8], revision: WorldRevision) -> Result<Vec<u8>, PersistError> {
    zstd::stream::decode_all(Cursor::new(stored)).map_err(|error| PersistError::Damaged {
        detail: format!("the snapshot at {revision} does not decompress: {error}"),
    })
}

/// Decodes a stored snapshot: decompressed, then decoded.
pub fn decode_snapshot(
    stored: &[u8],
    revision: WorldRevision,
) -> Result<WorldSnapshot, PersistError> {
    decode(&snapshot_json(stored, revision)?, "snapshot")
}

/// Refuses a save whose format is not this code's.
pub const fn check_format(saved: u32) -> Result<(), PersistError> {
    if saved > SAVE_FORMAT {
        return Err(PersistError::SaveFormatTooNew {
            saved,
            supported: SAVE_FORMAT,
        });
    }
    if saved < SAVE_FORMAT {
        return Err(PersistError::SaveFormatOutdated {
            saved,
            supported: SAVE_FORMAT,
        });
    }
    Ok(())
}
