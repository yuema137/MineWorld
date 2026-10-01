//! The schedule: what is to happen later, in the one order replay can reproduce.
//!
//! A discrete-event queue (`DEP-6`). Every entry is filed under `(WorldTime, Sequence)`: the instant
//! it is due, and a monotonic number assigned when it was queued. That pair is the whole of the
//! ordering rule, stated once here because everything downstream depends on it:
//!
//! ```text
//! earlier instant first                        the clock only moves forward
//! at one instant, lower sequence first         the order things were asked for — the only
//!                                              tie-break that is both natural and reproducible
//! ```
//!
//! Nothing is ordered by a hash, an address or a wall clock, so the same world given the same
//! inputs drains the same entries in the same order on every run (`AC-12`).
//!
//! # Idle time costs nothing
//!
//! The queue is asked for its *next* instant, and the world jumps there. There is no empty tick: a
//! world where nothing is due between 02:00 and 06:00 spends no work on those four hours (SD-2,
//! `AC-11`).
//!
//! # Why an ordered map rather than a heap
//!
//! `DEP-6` describes a binary heap keyed by `(WorldTime, sequence)`. This is the same key and the
//! same order in a `BTreeMap`, for one reason: a heap's internal layout depends on the history of
//! its insertions, so two queues holding the same entries could serialize to different bytes. An
//! ordered map serializes canonically, which a queue that S5 must save and compare across runs needs
//! (step-04 §8 F-9; the decision record carries a dated note).
//!
//! # Systems never see this
//!
//! A system asks for a fact *later* through [`WorldView::defer`](crate::WorldView::defer) and runs a
//! process with an expected end; the world files both here. No system holds the queue, reorders it
//! or reads what else is in it (`DEP-6`'s isolating interface).

#[cfg(test)]
mod tests;

use std::collections::{BTreeMap, BTreeSet};

use mineworld_contracts::{SystemId, WorldTime};
use serde::{Deserialize, Serialize};

use crate::error::KernelError;
use crate::system::Deferral;

/// The first sequence number a schedule assigns. One, not zero, for the reason identities start at
/// one: zero is what an absent or defaulted number looks like in a serialized record.
const FIRST_SEQUENCE: u64 = 1;

/// The order in which entries due at one instant are fired: the order they were queued in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Sequence(u64);

impl Sequence {
    /// The number, for diagnostics and persistence.
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl core::fmt::Display for Sequence {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "#{}", self.0)
    }
}

/// What can be due at an instant.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scheduled {
    /// A fact a system asked to happen at this instant.
    Fact(Deferral),
}

impl Scheduled {
    /// The system that acts when this entry fires, if it names one directly.
    pub(crate) fn deferring_system(&self) -> Option<&SystemId> {
        match self {
            Self::Fact(deferral) => Some(deferral.emitter()),
        }
    }
}

/// One entry, as a snapshot lists it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledEntry {
    /// When it is due.
    pub at: WorldTime,
    /// Its place among the entries due at that instant.
    pub sequence: Sequence,
    /// What is due.
    pub item: Scheduled,
}

/// A world's time, as saved: what [`World::schedule_snapshot`](crate::World::schedule_snapshot)
/// produces and [`World::restore_schedule`](crate::World::restore_schedule) validates.
///
/// Plain data with public fields, because it is a persistence format rather than a live object: the
/// world refuses a snapshot that could not have come from a world, whoever built it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduleSnapshot {
    /// The instant the world was at.
    pub now: WorldTime,
    /// The next sequence number its schedule would have assigned.
    pub next_sequence: u64,
    /// Everything that was waiting, in firing order.
    pub entries: Vec<ScheduledEntry>,
    /// The next event identity the world would have allocated.
    pub next_event: u64,
}

/// The queue a world advances through.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Schedule {
    entries: BTreeMap<(WorldTime, Sequence), Scheduled>,
    next_sequence: u64,
}

impl Schedule {
    pub(crate) const fn new() -> Self {
        Self {
            entries: BTreeMap::new(),
            next_sequence: FIRST_SEQUENCE,
        }
    }

    /// Files `item` at `at`, after everything already filed at that instant.
    ///
    /// The schedule does not know the clock, so it does not refuse a past instant itself: every
    /// path that schedules — deferral, a process's expected end, restoring a snapshot — refuses
    /// one before reaching here, each with its own named error.
    pub(crate) fn insert(
        &mut self,
        at: WorldTime,
        item: Scheduled,
    ) -> Result<Sequence, KernelError> {
        let sequence = Sequence(self.next_sequence);
        self.next_sequence = self
            .next_sequence
            .checked_add(1)
            .ok_or(KernelError::ScheduleSequenceExhausted)?;
        self.entries.insert((at, sequence), item);
        Ok(sequence)
    }

    /// The earliest instant anything is due, if anything is.
    pub(crate) fn next_instant(&self) -> Option<WorldTime> {
        self.entries.keys().next().map(|(at, _)| *at)
    }

    /// Removes and returns the first entry due exactly at `at`, in sequence order.
    ///
    /// Called repeatedly while a world fires one instant. Entries cannot be added *at* the instant
    /// being fired — deferral and process ends must be strictly later — so the loop that calls this
    /// terminates with the instant's entries, and only those.
    pub(crate) fn pop_at(&mut self, at: WorldTime) -> Option<(Sequence, Scheduled)> {
        let first = self.entries.first_entry()?;
        if first.key().0 != at {
            return None;
        }
        let ((_, sequence), item) = first.remove_entry();
        Some((sequence, item))
    }

    /// How many entries are waiting.
    pub(crate) fn len(&self) -> usize {
        self.entries.len()
    }

    /// Every entry in firing order, for a snapshot.
    pub(crate) fn entries(&self) -> Vec<ScheduledEntry> {
        self.entries
            .iter()
            .map(|((at, sequence), item)| ScheduledEntry {
                at: *at,
                sequence: *sequence,
                item: item.clone(),
            })
            .collect()
    }

    /// The next sequence number this schedule would assign.
    pub(crate) const fn next_sequence(&self) -> u64 {
        self.next_sequence
    }

    /// Rebuilds a schedule from a snapshot's entries, refusing one that could not have been
    /// produced by a schedule: an entry before `now`, a sequence used twice, or a counter that would
    /// hand out a sequence already used.
    pub(crate) fn restore(
        now: WorldTime,
        next_sequence: u64,
        entries: Vec<ScheduledEntry>,
    ) -> Result<Self, KernelError> {
        if next_sequence < FIRST_SEQUENCE {
            return Err(KernelError::PersistedSequenceCounterTooLow {
                next: next_sequence,
                first: FIRST_SEQUENCE,
            });
        }
        let mut restored = BTreeMap::new();
        let mut seen = BTreeSet::new();
        for entry in entries {
            if entry.at < now {
                return Err(KernelError::PersistedEntryBeforeNow { at: entry.at, now });
            }
            if entry.sequence.0 >= next_sequence || entry.sequence.0 < FIRST_SEQUENCE {
                return Err(KernelError::PersistedSequenceOutsideCounter {
                    sequence: entry.sequence,
                    next: next_sequence,
                });
            }
            // A sequence is assigned once, whatever instant it was filed at: two entries sharing one
            // would have no defined order between them if they ever fell due together.
            if !seen.insert(entry.sequence) {
                return Err(KernelError::PersistedSequenceRepeated {
                    sequence: entry.sequence,
                });
            }
            restored.insert((entry.at, entry.sequence), entry.item);
        }
        Ok(Self {
            entries: restored,
            next_sequence,
        })
    }
}
