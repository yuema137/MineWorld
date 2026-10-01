//! Processes: things that happen over time, stored as state the owning system alone may change.
//!
//! `docs/CORE_CONCEPTS.md` §10: a Process is something that happens **over time** — working,
//! sleeping, having dinner, travelling — and is not an Event (`INV-3`). An Event is an instant,
//! immutable fact; a Process is mutable while it runs, and the facts it produces along the way are
//! Events caused by it ([`Causation::Process`](mineworld_contracts::Causation)).
//!
//! # Stored state, never a coroutine
//!
//! A [`Process`] is a record: identity, type, owner, participants, place, when it started, when it
//! is expected to end, whether it is running or suspended, whether it may be interrupted, and the
//! owner's own state for it. Nothing here is a suspended call stack or a task, which is the reason
//! `DEP-6` rejected every SimPy-style crate: a coroutine cannot be written to SQLite and resumed in
//! a new process, and a record can (SD-5).
//!
//! What a process's state *means* — the course of a dinner, the distance travelled — is the owning
//! system's, encoded by it and never interpreted here (`INV-12`, `BI-1`). Progress is part of that
//! state for the same reason: the kernel cannot know what "40% through dinner" means (step-04 §8
//! F-7).
//!
//! # Only the owner changes a process
//!
//! A process is started by naming its kind, and a kind names its owner as a type:
//! [`ProcessKind::Owner`]. So `start_process::<Dinner>` compiles only inside the system that owns
//! `Dinner`, and so do the typed operations on a running one (`end_process::<Dinner>(id)` …). An id
//! is a value, so those operations also check at run time that the process behind it is that kind
//! and belongs to the writer — the same split relations have, for the same reason (step-04 §8 F-8).
//!
//! # Anyone may ask; the owner decides
//!
//! Another system that needs a process to stop *requests* it
//! ([`WorldView::request_interrupt`](crate::WorldView::request_interrupt)). The request is delivered
//! to the owner's [`System::interrupt`](crate::System::interrupt), which ends it, suspends it, or
//! leaves it running; the requester learns which from the returned [`InterruptOutcome`], read off
//! the store afterwards so the answer cannot misstate what happened (SD-6). A phone call does not
//! end a dinner — the dinner's system does, having been told about the call.

use std::collections::BTreeMap;
use std::marker::PhantomData;

use mineworld_contracts::{
    Causation, EntityId, PlaceId, ProcessId, ProcessTypeId, SystemId, WorldTime,
};
use serde::{Deserialize, Serialize};

use crate::access::SystemIdentity;
use crate::error::KernelError;

/// The first process identity a world allocates. One, for the reason every identity starts at one.
const FIRST_PROCESS_ID: u64 = 1;

/// A kind of process a system runs: its name, and — as a type — which system owns it.
///
/// ```
/// use mineworld_contracts::{ProcessTypeId, SystemId};
/// use mineworld_kernel::{ProcessKind, SystemIdentity};
///
/// struct Dining;
/// impl SystemIdentity for Dining {
///     const ID: SystemId = SystemId::from_static("dining");
/// }
///
/// /// Having dinner: owned by `Dining`, and by nothing else.
/// struct Dinner;
/// impl ProcessKind for Dinner {
///     const PROCESS_TYPE: ProcessTypeId = ProcessTypeId::from_static("having-dinner");
///     type Owner = Dining;
/// }
/// ```
pub trait ProcessKind: 'static {
    /// The kind's declared name — the process's `type` (`CORE_CONCEPTS.md` §10).
    const PROCESS_TYPE: ProcessTypeId;
    /// The system that owns every process of this kind, and the only one that may change one.
    type Owner: SystemIdentity;
}

/// Whether a process is running towards its expected end, or paused by its owner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProcessPhase {
    /// Running: its owner is woken at its expected end, if it has one.
    Running,
    /// Paused by its owner, typically in answer to an interruption. Not woken until resumed.
    Suspended,
}

/// Whether anyone may ask for a process to be interrupted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Interruptibility {
    /// A request is delivered to the owner, who decides.
    Interruptible,
    /// The owner decided in advance: every request is answered
    /// [`InterruptOutcome::Uninterruptible`] without consulting it.
    Uninterruptible,
}

/// A process, as the world stores it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Process {
    id: ProcessId,
    process_type: ProcessTypeId,
    owner: SystemId,
    participants: Vec<EntityId>,
    place: Option<PlaceId>,
    started: WorldTime,
    expected_end: Option<WorldTime>,
    phase: ProcessPhase,
    interruptibility: Interruptibility,
    /// The owner's state for this process, encoded by the owner and never interpreted here.
    state: Vec<u8>,
}

impl Process {
    /// Its identity, allocated by the world.
    pub const fn id(&self) -> ProcessId {
        self.id
    }

    /// What kind of process it is.
    pub const fn process_type(&self) -> &ProcessTypeId {
        &self.process_type
    }

    /// The system that owns it.
    pub const fn owner(&self) -> &SystemId {
        &self.owner
    }

    /// Who is taking part.
    pub fn participants(&self) -> &[EntityId] {
        &self.participants
    }

    /// Where it is happening, if it happens somewhere in particular.
    pub const fn place(&self) -> Option<PlaceId> {
        self.place
    }

    /// When it started — the instant its owner started it, supplied by the world.
    pub const fn started(&self) -> WorldTime {
        self.started
    }

    /// When it is expected to end, if its owner said. Its owner is woken then.
    pub const fn expected_end(&self) -> Option<WorldTime> {
        self.expected_end
    }

    /// Running or suspended.
    pub const fn phase(&self) -> ProcessPhase {
        self.phase
    }

    /// Whether it may be interrupted.
    pub const fn interruptibility(&self) -> Interruptibility {
        self.interruptibility
    }

    /// The owner's state for it, if it is a `P` — the bytes the owner encoded, for the owner to
    /// decode. Refused for another kind, so a process's state is never decoded as something else.
    pub fn state_for<P: ProcessKind>(&self) -> Result<&[u8], KernelError> {
        if self.process_type != P::PROCESS_TYPE {
            return Err(KernelError::ProcessKindMismatch {
                process: self.id,
                expected: P::PROCESS_TYPE,
                actual: self.process_type.clone(),
            });
        }
        Ok(&self.state)
    }
}

/// What starting a `P` states: its state, and optionally who takes part, where, until when, and
/// whether it may be interrupted.
///
/// The identity, the owner, the start instant and the phase are not here: the world supplies them,
/// so a system cannot start a process that claims to have begun at another time or to belong to
/// someone else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStart<P: ProcessKind> {
    state: Vec<u8>,
    participants: Vec<EntityId>,
    place: Option<PlaceId>,
    expected_end: Option<WorldTime>,
    interruptibility: Interruptibility,
    kind: PhantomData<fn() -> P>,
}

impl<P: ProcessKind> ProcessStart<P> {
    /// A `P` with this owner-encoded state: interruptible, open-ended, nobody named yet.
    pub fn new(state: Vec<u8>) -> Self {
        Self {
            state,
            participants: Vec::new(),
            place: None,
            expected_end: None,
            interruptibility: Interruptibility::Interruptible,
            kind: PhantomData,
        }
    }

    /// Names who takes part.
    #[must_use]
    pub fn with_participants(mut self, participants: Vec<EntityId>) -> Self {
        self.participants = participants;
        self
    }

    /// Names where it happens.
    #[must_use]
    pub const fn at_place(mut self, place: PlaceId) -> Self {
        self.place = Some(place);
        self
    }

    /// States when it is expected to end: its owner is woken then. Must be strictly later than the
    /// instant it starts — something that starts and ends in one instant is an event (`INV-3`).
    #[must_use]
    pub const fn ending_at(mut self, at: WorldTime) -> Self {
        self.expected_end = Some(at);
        self
    }

    /// Declares that no one may interrupt it.
    #[must_use]
    pub const fn uninterruptible(mut self) -> Self {
        self.interruptibility = Interruptibility::Uninterruptible;
        self
    }

    /// The expected end it states, if any — what the world schedules a wake for.
    pub(crate) const fn expected_end_instant(&self) -> Option<WorldTime> {
        self.expected_end
    }
}

/// An interruption request, as the owner receives it.
///
/// The requester and the reason are the kernel's statement, not the requester's: the requester is
/// the system whose view asked, and the reason is the causation of the call it asked from — the
/// phone call's event, say — so a request cannot claim to come from elsewhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterruptRequest {
    process: ProcessId,
    requester: SystemId,
    caused_by: Causation,
}

impl InterruptRequest {
    pub(crate) const fn new(process: ProcessId, requester: SystemId, caused_by: Causation) -> Self {
        Self {
            process,
            requester,
            caused_by,
        }
    }

    /// The process the request is about.
    pub const fn process(&self) -> ProcessId {
        self.process
    }

    /// The system asking.
    pub const fn requester(&self) -> &SystemId {
        &self.requester
    }

    /// Why it is asking: what the requester was handling when it asked.
    pub const fn caused_by(&self) -> &Causation {
        &self.caused_by
    }
}

/// What became of an interruption request. Read off the world after the owner decided, so it
/// reports what the owner did rather than what anyone said.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InterruptOutcome {
    /// The owner ended the process.
    Ended,
    /// The owner suspended it.
    Suspended,
    /// The owner left it running: the request was refused.
    Refused,
    /// The process is uninterruptible; the owner was not consulted.
    Uninterruptible,
    /// No such process is running — it has ended, or never existed.
    NotRunning,
    /// Its owner is disabled, so nobody could decide; the process is left as it is.
    OwnerDisabled,
}

/// Every process running in one world, in [`ProcessId`] order.
///
/// Reads are open, like components: any system may see who is having dinner. Writes are the
/// owner's, through [`WorldView`](crate::WorldView); this type's mutating methods are crate-private.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessStore {
    processes: BTreeMap<ProcessId, Process>,
    next: u64,
}

impl Default for ProcessStore {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessStore {
    pub(crate) const fn new() -> Self {
        Self {
            processes: BTreeMap::new(),
            next: FIRST_PROCESS_ID,
        }
    }

    /// The process with this identity, if it is running or suspended.
    pub fn get(&self, process: ProcessId) -> Option<&Process> {
        self.processes.get(&process)
    }

    /// Every process, in identity order — never in the order they were started or changed.
    pub fn iter(&self) -> impl Iterator<Item = &Process> {
        self.processes.values()
    }

    /// How many processes there are.
    pub fn len(&self) -> usize {
        self.processes.len()
    }

    /// Whether there are none.
    pub fn is_empty(&self) -> bool {
        self.processes.is_empty()
    }

    /// Starts a process owned by `owner` at `now`, returning its new identity.
    pub(crate) fn start<P: ProcessKind>(
        &mut self,
        owner: SystemId,
        start: ProcessStart<P>,
        now: WorldTime,
    ) -> Result<ProcessId, KernelError> {
        if let Some(end) = start.expected_end
            && end <= now
        {
            return Err(KernelError::ProcessEndNotInTheFuture { end, now });
        }
        let id = ProcessId::from_raw(self.next);
        self.next = self
            .next
            .checked_add(1)
            .ok_or(KernelError::ProcessIdSpaceExhausted)?;
        self.processes.insert(
            id,
            Process {
                id,
                process_type: P::PROCESS_TYPE,
                owner,
                participants: start.participants,
                place: start.place,
                started: now,
                expected_end: start.expected_end,
                phase: ProcessPhase::Running,
                interruptibility: start.interruptibility,
                state: start.state,
            },
        );
        Ok(id)
    }

    /// The process, for its owner to change: refused unless it exists, belongs to `writer`, and is a
    /// `P`.
    pub(crate) fn owned_mut<P: ProcessKind>(
        &mut self,
        process: ProcessId,
        writer: &SystemId,
    ) -> Result<&mut Process, KernelError> {
        let record = self
            .processes
            .get_mut(&process)
            .ok_or(KernelError::ProcessNotRunning { process })?;
        if record.owner != *writer {
            return Err(KernelError::ProcessNotOwned {
                process,
                owner: record.owner.clone(),
                writing_system: writer.clone(),
            });
        }
        if record.process_type != P::PROCESS_TYPE {
            return Err(KernelError::ProcessKindMismatch {
                process,
                expected: P::PROCESS_TYPE,
                actual: record.process_type.clone(),
            });
        }
        Ok(record)
    }

    /// Ends a process its owner holds, returning its final record.
    pub(crate) fn end<P: ProcessKind>(
        &mut self,
        process: ProcessId,
        writer: &SystemId,
    ) -> Result<Process, KernelError> {
        self.owned_mut::<P>(process, writer)?;
        self.processes
            .remove(&process)
            .ok_or(KernelError::ProcessNotRunning { process })
    }

    /// The next identity this store would allocate, for a snapshot.
    pub(crate) const fn next_id(&self) -> u64 {
        self.next
    }

    /// Every process, for a snapshot.
    pub(crate) fn records(&self) -> Vec<Process> {
        self.processes.values().cloned().collect()
    }

    /// Rebuilds a store from a snapshot, refusing one no store could have produced: an identity used
    /// twice or not below the counter, or a counter below the first identity.
    pub(crate) fn restore(next: u64, records: Vec<Process>) -> Result<Self, KernelError> {
        if next < FIRST_PROCESS_ID {
            return Err(KernelError::PersistedProcessCounterTooLow {
                next,
                first: FIRST_PROCESS_ID,
            });
        }
        let mut processes = BTreeMap::new();
        for record in records {
            if record.id.raw() >= next || record.id.raw() < FIRST_PROCESS_ID {
                return Err(KernelError::PersistedProcessOutsideCounter {
                    process: record.id,
                    next,
                });
            }
            let id = record.id;
            if processes.insert(id, record).is_some() {
                return Err(KernelError::PersistedProcessRepeated { process: id });
            }
        }
        Ok(Self { processes, next })
    }
}

impl Process {
    /// Sets the phase. Crate-private: reached only through the owner's view.
    pub(crate) const fn set_phase(&mut self, phase: ProcessPhase) {
        self.phase = phase;
    }

    /// Sets the expected end. Crate-private: reached only through the owner's view, which checks the
    /// instant is in the future and queues the wake.
    pub(crate) const fn set_expected_end(&mut self, end: Option<WorldTime>) {
        self.expected_end = end;
    }

    /// Replaces the owner's state. Crate-private: reached only through the owner's view.
    pub(crate) fn set_state(&mut self, state: Vec<u8>) {
        self.state = state;
    }
}
