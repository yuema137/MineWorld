//! The resolved decision list as the cognition process reads it: `mineworld decisions <world> --json`
//! (`docs/MODULE_SPEC.md` §8.1; `pr-dl-a-decision-list.md` SD-DLa-15, ruling on QFGb-1).
//!
//! Typed, keys in a fixed order, seats in roster order. `memory` matches FG-b's `MemoryPolicyChoice`
//! field for field (`policy`, `fallback`, `parameters`, `source`); `parameters` is the only free-form
//! object, passed through. `format` is the document's version: a change to its shape bumps it.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::resolve::ResolvedDecisions;

/// The document's version.
pub const JSON_FORMAT: u32 = 1;

/// The whole document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolvedDecisionsJson {
    /// [`JSON_FORMAT`].
    pub format: u32,
    /// The world's id.
    pub world: String,
    /// The digest, 16 hex digits (`ResolvedDecisions::digest`).
    pub digest: String,
    /// Every seat, in roster order.
    pub seats: Vec<SeatJson>,
}

/// One seat.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeatJson {
    /// The seat's key.
    pub seat: String,
    /// Its model.
    pub model: ModelJson,
    /// Its memory choice; `null` when none applies.
    pub memory: Option<MemoryPolicyChoiceJson>,
}

/// A seat's model.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelJson {
    /// The in-process model that drives the seat where no out-of-process model is hosted; `null` when
    /// the chain ends at `none`.
    pub id: Option<String>,
    /// The external model assigned, when one was.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub assigned: Option<String>,
    /// The chain from `assigned` to `id` (or `none`), when an external model was assigned.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub chain: Vec<String>,
}

/// FG-b's `MemoryPolicyChoice`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MemoryPolicyChoiceJson {
    /// The policy id.
    pub policy: String,
    /// Its fallback policy id, or `null`.
    pub fallback: Option<String>,
    /// Its parameters, passed through; `{}` when none were written.
    pub parameters: Map<String, Value>,
    /// Where the winning entry was written.
    pub source: SourceSpanJson,
}

/// A position in the decision list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpanJson {
    /// The file as written in `world.yaml` or passed to `--decisions`.
    pub file: String,
    /// The line, 1-based.
    pub line: u64,
    /// The column, 1-based.
    pub column: u64,
}

impl ResolvedDecisions {
    /// This resolution as the document `mineworld decisions --json` prints, for the world `world`.
    pub fn json(&self, world: &str) -> ResolvedDecisionsJson {
        ResolvedDecisionsJson {
            format: JSON_FORMAT,
            world: world.to_owned(),
            digest: format!("{:016x}", self.digest()),
            seats: self
                .seats()
                .iter()
                .map(|seat| SeatJson {
                    seat: seat.seat.to_string(),
                    model: ModelJson {
                        id: seat
                            .drives
                            .as_ref()
                            .map(|drives| drives.id().as_str().to_owned()),
                        assigned: seat.assigned.as_ref().map(|id| id.as_str().to_owned()),
                        chain: seat.chain.clone(),
                    },
                    memory: seat.memory.as_ref().map(|memory| MemoryPolicyChoiceJson {
                        policy: memory.choice.policy.to_string(),
                        fallback: memory.choice.fallback.as_ref().map(ToString::to_string),
                        parameters: memory.choice.parameters.clone(),
                        source: SourceSpanJson {
                            file: memory.source.file.clone(),
                            line: memory.source.line,
                            column: memory.source.column,
                        },
                    }),
                })
                .collect(),
        }
    }
}
