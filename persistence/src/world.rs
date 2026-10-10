//! A world with a save: every input journaled, every fact logged, before anybody is told.

use std::collections::BTreeSet;

use mineworld_contracts::{ActionIntent, EventEnvelope, WorldTime};
use mineworld_kernel::{Advanced, Dispatched, Emission, World, WorldSnapshot};

use crate::backend::{FactRow, ManifestRow, PersistenceBackend, RevisionRow};
use crate::error::PersistError;
use crate::format::{
    Manifest, SAVE_FORMAT, decode, decode_snapshot, encode, encode_snapshot, instance_text,
};
use crate::input::{JournalEntry, Outcome, WorldInput, WorldRevision};
use crate::replay::{self, advanced, check_composition, dispatched};

/// How often a persisted world writes a snapshot, unless told otherwise: every 64 revisions.
pub const DEFAULT_SNAPSHOT_INTERVAL: u64 = 64;

/// How many scheduled snapshots apart two anchors are: an anchor is kept at every multiple of
/// `ANCHOR_EVERY × interval` revisions — 4 096 at the default interval (`ARC-81`, QSR-2).
const ANCHOR_EVERY: u64 = 64;

/// The retention rule (`ARC-81`, design D-SR-3): which stored snapshots to delete once the scheduled
/// snapshot at `n` is committed. Kept are genesis, every anchor, `n` itself and `n − interval`; every
/// other stored revision below `n` goes — an off-lattice checkpoint included, which is kept only until
/// the next scheduled snapshot.
///
/// A function of `n`, the interval and the stored set and of nothing else, so that a killed and
/// resumed run holds exactly the rows an uninterrupted one does (`ARC-27`).
fn retired(
    n: WorldRevision,
    interval: u64,
    stored: &BTreeSet<WorldRevision>,
) -> Vec<WorldRevision> {
    let anchor = ANCHOR_EVERY * interval;
    let kept = |revision: u64| {
        revision == WorldRevision::GENESIS.raw()
            || revision.is_multiple_of(anchor)
            || revision + interval == n.raw()
    };
    stored
        .range(..n)
        .copied()
        .filter(|revision| !kept(revision.raw()))
        .collect()
}

/// What a new save is created with, beyond the world itself.
#[derive(Debug, Clone)]
pub struct Creation {
    /// Which world this is — allocated by the caller, kept for the world's whole life.
    pub instance: u128,
    /// The World Pack it came from, for a person reading the save.
    pub pack: String,
    /// The instant it begins at.
    pub at: WorldTime,
    /// What is true of it as it begins: its genesis facts, not yet applied.
    pub facts: Vec<Emission>,
}

/// How a resumed world was rebuilt — reported, so that a log or a test can see that a snapshot was
/// read and a tail was re-executed, rather than assume it (`ARC-23`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Resumed {
    /// The revision of the snapshot the world was restored from.
    pub snapshot: WorldRevision,
    /// How many journal rows after it were re-executed.
    pub replayed: u64,
    /// How many facts those rows reproduced, byte for byte.
    pub facts: u64,
    /// The head the world resumed at.
    pub head: WorldRevision,
}

/// A world and its save.
///
/// Owns the [`World`] and hands out only reads of it: the two ways it can be moved — a request and
/// an advance of the clock — are this type's methods, and each journals what it did before it
/// returns. A mutation that bypassed them would be state no replay could reproduce, so there is no
/// way to make one (step-06 PD-8).
pub struct PersistentWorld {
    world: World,
    backend: Box<dyn PersistenceBackend>,
    revision: WorldRevision,
    instance: u128,
    snapshot_interval: u64,
    /// The revisions the save holds a snapshot at — read once when the world is created or resumed,
    /// then kept in step with every commit and checkpoint, so that retention needs no query.
    stored: BTreeSet<WorldRevision>,
    /// Set when a commit failed: the world in memory is then ahead of its save, and refuses
    /// every further input.
    ahead_of_save: Option<(WorldRevision, String)>,
    /// The world's instant when the head revision was committed: the instant the state at the head
    /// has, which a checkpoint must match.
    committed_at: WorldTime,
}

impl PersistentWorld {
    /// Creates a save for a world that has been assembled — systems installed, entities created —
    /// and has not begun: genesis runs here, so that what is journaled as genesis is exactly what
    /// was applied. Returns the world and the facts it began with.
    pub fn create(
        mut backend: Box<dyn PersistenceBackend>,
        mut world: World,
        creation: Creation,
    ) -> Result<(Self, Vec<EventEnvelope>), PersistError> {
        let before = world.snapshot()?;
        let began = world.genesis(creation.at, creation.facts.clone())?;
        let manifest = Manifest {
            instance: instance_text(creation.instance),
            pack: creation.pack,
            composition: world.composition(),
        };
        let input = WorldInput::Genesis {
            before: Box::new(before),
            at: creation.at,
            facts: creation.facts,
        };
        let row = revision_row(
            WorldRevision::GENESIS,
            input,
            Outcome::Began,
            &began,
            Some(&world.snapshot()?),
        )?;
        backend.initialize(
            &ManifestRow {
                format: SAVE_FORMAT,
                body: encode(&manifest)?,
            },
            &row,
        )?;
        Ok((
            Self {
                committed_at: world.now(),
                world,
                backend,
                revision: WorldRevision::GENESIS,
                instance: creation.instance,
                snapshot_interval: DEFAULT_SNAPSHOT_INTERVAL,
                stored: BTreeSet::from([WorldRevision::GENESIS]),
                ahead_of_save: None,
            },
            began,
        ))
    }

    /// Rebuilds a saved world into `composed` — a world with its systems installed and nothing else.
    ///
    /// The save's format and composition are checked first, then the newest snapshot is restored and
    /// every journal row after it is re-executed and required to reproduce its logged answer and facts
    /// byte for byte (`ARC-25`). Any difference refuses the load.
    pub fn resume(
        backend: Box<dyn PersistenceBackend>,
        mut composed: World,
    ) -> Result<(Self, Resumed), PersistError> {
        let manifest = replay::manifest(backend.as_ref())?;
        check_composition(&manifest.composition, &composed.composition())?;
        let head = backend.head()?;
        let (snapshot, bytes) =
            backend
                .latest_snapshot(head)?
                .ok_or_else(|| PersistError::Damaged {
                    detail: "the save has no snapshot, not even at genesis".to_owned(),
                })?;
        composed.restore(decode_snapshot(&bytes, snapshot)?)?;
        let (replayed, facts) =
            replay::replay_after(&mut composed, backend.as_ref(), snapshot, |_, _| Ok(()))?;
        let instance = manifest.instance_number()?;
        let stored = backend.snapshot_revisions()?.into_iter().collect();
        Ok((
            Self {
                committed_at: composed.now(),
                world: composed,
                backend,
                revision: head,
                instance,
                snapshot_interval: DEFAULT_SNAPSHOT_INTERVAL,
                stored,
                ahead_of_save: None,
            },
            Resumed {
                snapshot,
                replayed,
                facts,
                head,
            },
        ))
    }

    /// Writes a snapshot every `interval` revisions instead of every [`DEFAULT_SNAPSHOT_INTERVAL`].
    #[must_use]
    pub fn snapshot_every(mut self, interval: u64) -> Self {
        self.snapshot_interval = interval.max(1);
        self
    }

    /// The world, to read.
    pub const fn world(&self) -> &World {
        &self.world
    }

    /// The newest revision committed to the save.
    pub const fn revision(&self) -> WorldRevision {
        self.revision
    }

    /// Which world this is.
    pub const fn instance(&self) -> u128 {
        self.instance
    }

    /// The highest `ActionId` any request in this world's journal carried — where a host's
    /// request allocator resumes, so that no identity is issued twice across a restart.
    pub fn highest_action_id(&self) -> Result<Option<u64>, PersistError> {
        self.backend.highest_action_id()
    }

    /// The newest `count` facts in the log, oldest first.
    pub fn recent_facts(&self, count: usize) -> Result<Vec<EventEnvelope>, PersistError> {
        self.backend
            .last_facts(count)?
            .iter()
            .map(|fact| decode(&fact.bytes, "fact"))
            .collect()
    }

    /// Dispatches a request and journals it, whatever the answer.
    ///
    /// `Err(PersistError::Kernel)` is a system breaking its contract: the fault is journaled as this
    /// request's outcome and the world goes on, exactly as an unpersisted world would. Any other error
    /// means the save could not be written, and the world refuses every later input.
    pub fn dispatch(
        &mut self,
        intent: &ActionIntent,
        at: WorldTime,
    ) -> Result<Dispatched, PersistError> {
        self.refuse_if_ahead()?;
        let result = self.world.dispatch(intent, at);
        let (outcome, facts) = dispatched(&result);
        self.commit(
            WorldInput::Dispatch {
                intent: intent.clone(),
                at,
            },
            outcome,
            &facts,
        )?;
        result.map_err(PersistError::from)
    }

    /// Advances the clock to `until`, journaling the advance if anything was due on the way.
    ///
    /// An advance that fired nothing only moved the clock; it is not a revision (`ARC-25`).
    pub fn advance_to(&mut self, until: WorldTime) -> Result<Advanced, PersistError> {
        self.refuse_if_ahead()?;
        let result = self.world.advance_to(until);
        if let Some((outcome, facts)) = advanced(&result) {
            self.commit(WorldInput::Advance { until }, outcome, &facts)?;
        }
        result.map_err(PersistError::from)
    }

    /// Writes a snapshot of the current revision, unless one is already stored there — what a clean
    /// shutdown does, so that the next start re-executes nothing. Returns whether it wrote.
    ///
    /// Writes nothing if the clock has moved since the last revision. Idle advances are not journaled
    /// (`ARC-25`), so the state *at* the head revision has the head's instant; a snapshot taken after
    /// the clock idled forward would hold an instant no re-execution of the history produces, and
    /// verification would rightly refuse it (step-06 §9, F-12).
    pub fn checkpoint(&mut self) -> Result<bool, PersistError> {
        self.refuse_if_ahead()?;
        if self.world.now() != self.committed_at {
            return Ok(false);
        }
        let snapshot = encode_snapshot(&self.world.snapshot()?)?;
        self.backend.checkpoint(self.revision, &snapshot)?;
        self.stored.insert(self.revision);
        Ok(true)
    }

    fn refuse_if_ahead(&self) -> Result<(), PersistError> {
        match &self.ahead_of_save {
            Some((revision, cause)) => Err(PersistError::WorldAheadOfSave {
                revision: *revision,
                cause: cause.clone(),
            }),
            None => Ok(()),
        }
    }

    /// Commits the next revision, or marks the world as ahead of its save.
    fn commit(
        &mut self,
        input: WorldInput,
        outcome: Outcome,
        facts: &[EventEnvelope],
    ) -> Result<(), PersistError> {
        let revision = self.revision.next();
        let scheduled = revision.raw().is_multiple_of(self.snapshot_interval);
        let committed = (|| {
            let snapshot = if scheduled {
                Some(self.world.snapshot()?)
            } else {
                None
            };
            let mut row = revision_row(revision, input, outcome, facts, snapshot.as_ref())?;
            if scheduled {
                row.retire = retired(revision, self.snapshot_interval, &self.stored);
                // By construction every retired revision is below this one; the check keeps a
                // future edit of the rule from ever deleting the snapshot a resume would read.
                if let Some(late) = row.retire.iter().find(|retired| **retired >= revision) {
                    debug_assert!(false, "the rule retired {late} at {revision}");
                    return Err(PersistError::Damaged {
                        detail: format!("the retention rule retired {late} at {revision}"),
                    });
                }
            }
            self.backend.commit(&row)?;
            Ok(row.retire)
        })();
        match committed {
            Ok(retire) => {
                self.revision = revision;
                self.committed_at = self.world.now();
                if scheduled {
                    self.stored.insert(revision);
                }
                for retired in retire {
                    self.stored.remove(&retired);
                }
                Ok(())
            }
            Err(error) => {
                let cause = error.to_string();
                self.ahead_of_save = Some((revision, cause.clone()));
                Err(PersistError::WorldAheadOfSave { revision, cause })
            }
        }
    }
}

/// Encodes one revision for the backend.
fn revision_row(
    revision: WorldRevision,
    input: WorldInput,
    outcome: Outcome,
    facts: &[EventEnvelope],
    snapshot: Option<&WorldSnapshot>,
) -> Result<RevisionRow, PersistError> {
    let action_id = match &input {
        WorldInput::Dispatch { intent, .. } => Some(intent.action_id().raw()),
        _ => None,
    };
    let at = input.at().seconds();
    Ok(RevisionRow {
        revision,
        at,
        action_id,
        entry: encode(&JournalEntry { input, outcome })?,
        facts: facts
            .iter()
            .map(|fact| {
                Ok(FactRow {
                    id: fact.id().raw(),
                    bytes: encode(fact)?,
                })
            })
            .collect::<Result<_, PersistError>>()?,
        snapshot: snapshot.map(encode_snapshot).transpose()?,
        retire: Vec::new(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn revisions(raw: &[u64]) -> BTreeSet<WorldRevision> {
        raw.iter().copied().map(WorldRevision::from_raw).collect()
    }

    fn retired_raw(n: u64, interval: u64, stored: &[u64]) -> Vec<u64> {
        retired(WorldRevision::from_raw(n), interval, &revisions(stored))
            .into_iter()
            .map(WorldRevision::raw)
            .collect()
    }

    /// One case: what, n, interval, stored before the commit, expected retired.
    type Case = (&'static str, u64, u64, &'static [u64], &'static [u64]);

    /// D-SR-3 over the cases a save can be in: the steady state, the first anchor, an anchor that
    /// has just become older than the newest two, off-lattice checkpoints below and between the
    /// newest two, another interval, and rows the rule never sees (the new snapshot and above).
    #[test]
    fn the_rule_keeps_genesis_anchors_and_the_newest_two_and_retires_the_rest() {
        let cases: [Case; 9] = [
            ("first scheduled", 64, 64, &[1], &[]),
            ("second scheduled", 128, 64, &[1, 64], &[]),
            ("steady state", 320, 64, &[1, 192, 256], &[192]),
            ("at an anchor", 4_096, 64, &[1, 3_968, 4_032], &[3_968]),
            ("past an anchor", 4_224, 64, &[1, 4_096, 4_160], &[]),
            (
                "anchor ages",
                4_288,
                64,
                &[1, 4_096, 4_160, 4_224],
                &[4_160],
            ),
            (
                "checkpoints",
                512,
                64,
                &[1, 384, 400, 448, 470],
                &[384, 400, 470],
            ),
            ("interval 8", 1_032, 8, &[1, 512, 1_016, 1_024], &[1_016]),
            ("never above n", 128, 64, &[1, 64, 128, 130], &[]),
        ];
        for (what, n, interval, stored, expected) in cases {
            assert_eq!(retired_raw(n, interval, stored), expected, "{what}");
        }
    }
}
