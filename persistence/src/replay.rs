//! Re-executing a journal through the kernel's one pipeline, and refusing any divergence.
//!
//! Nothing here reduces a fact. An input is applied with the same public `World` call that applied it
//! the first time — `genesis`, `dispatch`, `advance_to` — and what that call answers and records is
//! compared, **as bytes**, with what the save recorded (`ARC-25`). A single differing byte refuses the
//! load and names the revision; nothing is repaired.

use mineworld_contracts::EventEnvelope;
use mineworld_kernel::{Advanced, Dispatched, InstalledSystemRecord, KernelError, World};

use crate::backend::{FactRow, PersistenceBackend};
use crate::error::PersistError;
use crate::format::{Manifest, check_format, decode, encode};
use crate::input::{JournalEntry, Outcome, WorldInput, WorldRevision};

/// What a request's dispatch amounts to in the journal: its answer, or the fault, and its facts.
pub(crate) fn dispatched(
    result: &Result<Dispatched, KernelError>,
) -> (Outcome, Vec<EventEnvelope>) {
    match result {
        Ok(dispatched) => (
            Outcome::Dispatched(dispatched.result().clone()),
            dispatched.events().to_vec(),
        ),
        Err(error) => (Outcome::Fault(error.to_string()), Vec::new()),
    }
}

/// What an advance amounts to in the journal — or `None` when it fired nothing, which is not
/// journaled: it only moved the clock, and every later input carries its own instant.
pub(crate) fn advanced(
    result: &Result<Advanced, KernelError>,
) -> Option<(Outcome, Vec<EventEnvelope>)> {
    match result {
        Ok(advanced) if advanced.instants() == 0 => None,
        Ok(advanced) => Some((
            Outcome::Advanced {
                instants: advanced.instants(),
                skipped: advanced.skipped(),
            },
            advanced.events().to_vec(),
        )),
        Err(error) => Some((Outcome::Fault(error.to_string()), Vec::new())),
    }
}

/// Applies one journaled input to `world` through the public kernel call that applied it the first
/// time, returning what it answered and recorded.
fn apply(
    world: &mut World,
    input: &WorldInput,
) -> Result<(Outcome, Vec<EventEnvelope>), PersistError> {
    Ok(match input {
        WorldInput::Genesis { before, at, facts } => {
            world.restore(before.as_ref().clone())?;
            match world.genesis(*at, facts.clone()) {
                Ok(recorded) => (Outcome::Began, recorded),
                Err(error) => (Outcome::Fault(error.to_string()), Vec::new()),
            }
        }
        WorldInput::Dispatch { intent, at } => dispatched(&world.dispatch(intent, *at)),
        WorldInput::Advance { until } => {
            let result = world.advance_to(*until);
            advanced(&result).unwrap_or((
                Outcome::Advanced {
                    instants: 0,
                    skipped: 0,
                },
                Vec::new(),
            ))
        }
    })
}

/// Re-executes one journal row and requires it to reproduce, byte for byte, the entry and the facts
/// the save recorded. Returns how many facts were compared.
fn reproduce(
    world: &mut World,
    backend: &dyn PersistenceBackend,
    revision: WorldRevision,
    stored: &[u8],
) -> Result<usize, PersistError> {
    let entry: JournalEntry = decode(stored, "journal entry")?;
    let (outcome, facts) = apply(world, &entry.input)?;
    let replayed = encode(&JournalEntry {
        input: entry.input,
        outcome: outcome.clone(),
    })?;
    if replayed != stored {
        return Err(PersistError::ReplayDiverged {
            revision,
            detail: format!(
                "the world answered {outcome:?}; the save recorded {:?}",
                entry.outcome
            ),
        });
    }

    let logged = backend.facts_of(revision)?;
    if logged.len() != facts.len() {
        return Err(PersistError::ReplayDiverged {
            revision,
            detail: format!(
                "re-execution recorded {} facts; the save logged {}",
                facts.len(),
                logged.len()
            ),
        });
    }
    for (fact, FactRow { id, bytes }) in facts.iter().zip(&logged) {
        if fact.id().raw() != *id || encode(fact)? != *bytes {
            return Err(PersistError::ReplayDiverged {
                revision,
                detail: format!("fact {} differs from the logged fact {id}", fact.id()),
            });
        }
    }
    Ok(facts.len())
}

/// The manifest of a save, once its format has been accepted.
pub(crate) fn manifest(backend: &dyn PersistenceBackend) -> Result<Manifest, PersistError> {
    let row = backend.manifest()?;
    check_format(row.format)?;
    decode(&row.body, "manifest")
}

/// Refuses a running world composed differently from the saved one, naming the first difference —
/// and, where the difference is only a system's version, which way it goes.
pub(crate) fn check_composition(
    saved: &[InstalledSystemRecord],
    running: &[InstalledSystemRecord],
) -> Result<(), PersistError> {
    for position in 0..saved.len().max(running.len()) {
        let (left, right) = (saved.get(position), running.get(position));
        if left == right {
            continue;
        }
        if let (Some(left), Some(right)) = (left, right) {
            let (was, is) = (&left.declaration, &right.declaration);
            if was.system() == is.system() && was.version() != is.version() {
                let system = is.system().clone();
                let (saved, running) = (was.version(), is.version());
                return Err(if saved > running {
                    PersistError::PersistedSystemTooNew {
                        system,
                        saved,
                        running,
                    }
                } else {
                    PersistError::PersistedSystemOutdated {
                        system,
                        saved,
                        running,
                    }
                });
            }
        }
        return Err(PersistError::CompositionDiffers {
            position,
            saved: left.map(|record| record.declaration.system().clone()),
            running: right.map(|record| record.declaration.system().clone()),
        });
    }
    Ok(())
}

/// Re-executes every journal row after `from` into `world`, in order, each required to reproduce.
/// Calls `after_each` with each revision once it has reproduced. Returns `(rows, facts)` compared.
pub(crate) fn replay_after(
    world: &mut World,
    backend: &dyn PersistenceBackend,
    from: WorldRevision,
    mut after_each: impl FnMut(&World, WorldRevision) -> Result<(), PersistError>,
) -> Result<(u64, u64), PersistError> {
    let mut expected = from.next();
    let (mut rows, mut facts) = (0, 0);
    for (revision, entry) in backend.journal_after(from)? {
        if revision != expected {
            return Err(PersistError::Damaged {
                detail: format!(
                    "the journal skips from {} to {revision}",
                    expected.raw() - 1
                ),
            });
        }
        facts += reproduce(world, backend, revision, &entry)? as u64;
        rows += 1;
        after_each(world, revision)?;
        expected = revision.next();
    }
    Ok((rows, facts))
}

/// What verifying a save from genesis compared.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Verified {
    /// The save's head.
    pub head: WorldRevision,
    /// Journal rows re-executed — every revision, genesis included.
    pub revisions: u64,
    /// Facts compared byte for byte.
    pub facts: u64,
    /// Stored snapshots compared byte for byte with the state the history produces at their revision.
    pub snapshots: u64,
}

/// Re-executes a save's whole journal from genesis into `composed` — a world with its systems
/// installed and nothing else — and requires every revision to reproduce its logged answer and facts,
/// and every stored snapshot to equal the state at its revision (`ARC-25`).
pub fn verify(
    backend: &dyn PersistenceBackend,
    mut composed: World,
) -> Result<Verified, PersistError> {
    let manifest = manifest(backend)?;
    check_composition(&manifest.composition, &composed.composition())?;
    let head = backend.head()?;
    let mut snapshots = 0;
    let (revisions, facts) = replay_after(
        &mut composed,
        backend,
        WorldRevision::from_raw(0),
        |world, revision| {
            if let Some(stored) = backend.snapshot_at(revision)? {
                if encode(&world.snapshot()?)? != stored {
                    return Err(PersistError::SnapshotDisagreesWithHistory { revision });
                }
                snapshots += 1;
            }
            Ok(())
        },
    )?;
    if revisions != head.raw() {
        return Err(PersistError::Damaged {
            detail: format!("re-executed {revisions} revisions; the head is {head}"),
        });
    }
    Ok(Verified {
        head,
        revisions,
        facts,
        snapshots,
    })
}
