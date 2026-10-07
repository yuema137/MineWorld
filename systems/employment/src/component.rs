//! The state this pack owns: a person's job, and how the current shift is going.

use mineworld_contracts::{ItemId, OrganizationId, PlaceId, WorldTime};
use mineworld_kernel::owned_component;
use mineworld_schedule::TimeOfDay;
use serde::{Deserialize, Serialize};

use crate::system::EmploymentSystem;

/// Seconds in an hour: a wage is stated per hour and paid per second worked.
const HOUR: u128 = 3_600;

/// One kind a full shift produces for the employer, and how many.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Produces {
    item: ItemId,
    per_shift: u32,
}

impl Produces {
    pub(crate) const fn new(item: ItemId, per_shift: u32) -> Self {
        Self { item, per_shift }
    }

    /// Which kind.
    pub const fn item(&self) -> ItemId {
        self.item
    }

    /// How many a full shift produces; never zero.
    pub const fn per_shift(&self) -> u32 {
        self.per_shift
    }
}

/// A job, as hired: who employs, where, when, for how much, and what a shift makes.
///
/// Built only from an authored `job:` section, whose decoding refuses a shift that does not end after
/// it starts, so `from < until` holds for every job (step-10 QS-50).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    employer: OrganizationId,
    workplace: PlaceId,
    from: TimeOfDay,
    until: TimeOfDay,
    wage: u64,
    produces: Vec<Produces>,
}

impl Job {
    pub(crate) const fn new(
        employer: OrganizationId,
        workplace: PlaceId,
        from: TimeOfDay,
        until: TimeOfDay,
        wage: u64,
        produces: Vec<Produces>,
    ) -> Self {
        Self {
            employer,
            workplace,
            from,
            until,
            wage,
            produces,
        }
    }

    /// The Organization the person works for.
    pub const fn employer(&self) -> OrganizationId {
        self.employer
    }

    /// Where the work is done: being here during the shift is working.
    pub const fn workplace(&self) -> PlaceId {
        self.workplace
    }

    /// When the shift starts, each day.
    pub const fn from(&self) -> TimeOfDay {
        self.from
    }

    /// When it ends, the same day.
    pub const fn until(&self) -> TimeOfDay {
        self.until
    }

    /// The wage, in integer minor units per hour (I-6).
    pub const fn wage(&self) -> u64 {
        self.wage
    }

    /// What a full shift produces for the employer, in item order.
    pub fn produces(&self) -> &[Produces] {
        &self.produces
    }

    /// How long a shift is, in seconds; at least 60, because `from < until` to the minute.
    pub fn shift_seconds(&self) -> u64 {
        u64::try_from(self.until.seconds() - self.from.seconds())
            .expect("a job's shift ends after it starts")
    }

    /// What `worked` seconds are paid: `wage × worked ÷ 3 600`, rounded down — integers only.
    pub fn wage_for(&self, worked: u64) -> u64 {
        let amount = u128::from(self.wage) * u128::from(worked) / HOUR;
        u64::try_from(amount).unwrap_or(u64::MAX)
    }

    /// What `worked` seconds produce, per kind: `per_shift × worked ÷ shift`, rounded down, and only
    /// the kinds whose share is at least one. Absent, nothing is made.
    pub fn produced_for(&self, worked: u64) -> Vec<(ItemId, u32)> {
        let shift = u128::from(self.shift_seconds());
        self.produces
            .iter()
            .filter_map(|line| {
                let count = u128::from(line.per_shift)
                    * u128::from(worked.min(self.shift_seconds()))
                    / shift;
                u32::try_from(count)
                    .ok()
                    .filter(|count| *count > 0)
                    .map(|count| (line.item, count))
            })
            .collect()
    }
}

/// How the shift in progress is going: since when the employee has been at the workplace, if they are
/// there now, and how many seconds they were there before that.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OnShift {
    present_since: Option<WorldTime>,
    worked: u64,
}

impl OnShift {
    /// A shift begun at `at`, with the employee there or not.
    pub(crate) fn begun(at: WorldTime, present: bool) -> Self {
        Self {
            present_since: present.then_some(at),
            worked: 0,
        }
    }

    /// The employee arrived at the workplace at `at`. Arriving while already there changes nothing.
    pub(crate) fn arrived(self, at: WorldTime) -> Self {
        Self {
            present_since: self.present_since.or(Some(at)),
            ..self
        }
    }

    /// The employee left the workplace at `at`: the span they were there is worked.
    pub(crate) fn left(self, at: WorldTime) -> Self {
        Self {
            present_since: None,
            worked: self.worked_until(at),
        }
    }

    /// Since when the employee has been at the workplace, if they are there now.
    pub const fn present_since(&self) -> Option<WorldTime> {
        self.present_since
    }

    /// Seconds worked by `at`: the closed spans, and the open one up to `at`.
    pub fn worked_until(&self, at: WorldTime) -> u64 {
        let open = self.present_since.map_or(0, |since| {
            u64::try_from(at.seconds() - since.seconds()).unwrap_or(0)
        });
        self.worked + open
    }
}

/// A person's job, and the shift in progress if there is one.
///
/// Written only by this pack's reductions of `hired`, `shift-started`, `shift-ended` and presence's
/// `person-entered-place` (`ARC-28`: reacting into its own state).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Employment {
    job: Job,
    shift: Option<OnShift>,
}

impl Employment {
    pub(crate) const fn hired(job: Job) -> Self {
        Self { job, shift: None }
    }

    pub(crate) fn with_shift(self, shift: Option<OnShift>) -> Self {
        Self { shift, ..self }
    }

    /// The job.
    pub const fn job(&self) -> &Job {
        &self.job
    }

    /// The shift in progress, between its start and its end; [`None`] off shift.
    pub const fn shift(&self) -> Option<OnShift> {
        self.shift
    }
}

owned_component! {
    component = Employment,
    owner = EmploymentSystem,
    component_type = "employment",
    schema_version = 1,
}
