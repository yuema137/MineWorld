//! How a save's rows are encoded, and the one version number that covers them all.
//!
//! JSON throughout (`DEP-5`): a journal entry, a fact, a snapshot and the manifest are each the
//! `serde_json` encoding of their contract or kernel type. Every type involved orders its collections
//! by key, so equal values encode to equal bytes — which is what replay compares.

use mineworld_kernel::InstalledSystemRecord;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::PersistError;

/// The save format this code writes and the only one it reads.
///
/// Raised whenever the encoding of any row changes — including a change to `WorldSnapshot`,
/// `EventEnvelope` or a journal entry's shape. A save of another format is refused, never decoded on
/// a guess (step-06 I-5).
pub const SAVE_FORMAT: u32 = 1;

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
            detail: format!("the manifest's instance '{}' is not a number", self.instance),
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
