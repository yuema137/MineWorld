//! The state this pack owns: a person's job, and how the current shift is going.

use mineworld_contracts::{ItemId, OrganizationId, PersonId, PlaceId, WorldTime};
use mineworld_kernel::owned_component;
use mineworld_schedule::TimeOfDay;
use serde::{Deserialize, Serialize};
use serde_json::Value;

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
    /// Whether this employee manages the employer's staff (`ARC-287`): serialized only when true, so
    /// a job without a manager keeps the bytes it had.
    #[serde(default, skip_serializing_if = "core::ops::Not::not")]
    manages: bool,
    /// What the job is called, if its author said (S30-f): serialized only when present, so a job
    /// without a title keeps the bytes it had.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    title: Option<String>,
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
            manages: false,
            title: None,
        }
    }

    /// The same job, with its title.
    #[must_use]
    pub(crate) fn titled(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    /// What the job is called, if its author said.
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// The same job, held by the employer's manager.
    pub(crate) const fn managing(mut self, manages: bool) -> Self {
        self.manages = manages;
        self
    }

    /// Whether this employee is the employer's manager: the one person offered `penalize` for the
    /// lateness of the employer's other staff (`ARC-287`).
    pub const fn manages(&self) -> bool {
        self.manages
    }

    /// The instant the shift containing `at`'s day begins: that day's midnight plus `from`.
    pub fn shift_start(&self, at: WorldTime) -> WorldTime {
        let midnight = at.seconds() - TimeOfDay::of(at);
        WorldTime::from_seconds(midnight + self.from.seconds())
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

/// A lateness the employer's manager may still penalize (`ARC-287`): which shift, how late, how many
/// times within the window, and the most a penalty may take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenLateness {
    shift: WorldTime,
    late_by: u64,
    count: u32,
    level: Level,
    bound: u64,
}

impl OpenLateness {
    pub(crate) const fn new(
        shift: WorldTime,
        late_by: u64,
        count: u32,
        level: Level,
        bound: u64,
    ) -> Self {
        Self {
            shift,
            late_by,
            count,
            level,
            bound,
        }
    }

    /// The instant the shift began.
    pub const fn shift(&self) -> WorldTime {
        self.shift
    }

    /// Seconds late.
    pub const fn late_by(&self) -> u64 {
        self.late_by
    }

    /// Latenesses within the window, this one included.
    pub const fn count(&self) -> u32 {
        self.count
    }

    /// How serious, from the count.
    pub const fn level(&self) -> Level {
        self.level
    }

    /// The most a `penalize` for it may take, in minor units; at least 1.
    pub const fn bound(&self) -> u64 {
        self.bound
    }
}

/// How serious a lateness is, from how many there were within the window (`ARC-287`). In MVP-0 `final`
/// is a stronger complaint and nothing more: nobody is dismissed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Level {
    /// Below the warning threshold.
    Notice,
    /// At or above the warning threshold.
    Warning,
    /// At or above the final threshold.
    Final,
}

/// A person's job, the shift in progress if there is one, and their recent lateness.
///
/// Written only by this pack's reductions of `hired`, `shift-started`, `shift-ended`,
/// `late-for-shift`, `wage-deduction-due` and presence's `person-entered-place` (`ARC-28`: reacting
/// into its own state). The lateness fields are serialized only when non-empty, so a world where
/// nobody is late keeps its bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Employment {
    job: Job,
    shift: Option<OnShift>,
    /// The shift instants of recent latenesses, oldest first; at most [`LATE_KEPT`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    late: Vec<WorldTime>,
    /// The lateness the manager may still penalize, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    open: Option<OpenLateness>,
}

/// The most lateness instants kept: the largest `final_at` a world may set, so the count can always
/// reach the final threshold.
pub const LATE_KEPT: usize = 20;

impl Employment {
    pub(crate) const fn hired(job: Job) -> Self {
        Self {
            job,
            shift: None,
            late: Vec::new(),
            open: None,
        }
    }

    pub(crate) fn with_shift(self, shift: Option<OnShift>) -> Self {
        Self { shift, ..self }
    }

    /// The same, with a lateness at `shift` recorded and the open lateness set to `open`.
    pub(crate) fn with_lateness(mut self, shift: WorldTime, open: Option<OpenLateness>) -> Self {
        if !self.late.contains(&shift) {
            self.late.push(shift);
        }
        if self.late.len() > LATE_KEPT {
            let excess = self.late.len() - LATE_KEPT;
            self.late.drain(..excess);
        }
        self.open = open;
        self
    }

    /// The same, with the open lateness closed.
    pub(crate) fn closed(self) -> Self {
        Self { open: None, ..self }
    }

    /// The job.
    pub const fn job(&self) -> &Job {
        &self.job
    }

    /// The shift in progress, between its start and its end; [`None`] off shift.
    pub const fn shift(&self) -> Option<OnShift> {
        self.shift
    }

    /// The shift instants of recent latenesses, oldest first.
    pub fn late(&self) -> &[WorldTime] {
        &self.late
    }

    /// The open lateness, however old.
    pub const fn open(&self) -> Option<OpenLateness> {
        self.open
    }

    /// The lateness the manager may still penalize at `now`: open and within `window` seconds of its
    /// shift.
    pub fn open_at(&self, now: WorldTime, window: i64) -> Option<OpenLateness> {
        self.open
            .filter(|open| now.seconds() < open.shift.seconds().saturating_add(window))
    }
}

owned_component! {
    component = Employment,
    owner = EmploymentSystem,
    component_type = "employment",
    schema_version = 1,
}

/// One of the employer's staff whose lateness the manager may still penalize, as the manager is told
/// it (`ARC-287`): who, which shift, how late, how many times, how serious, and the most a penalty may
/// take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LateStaff {
    employee: PersonId,
    #[serde(flatten)]
    lateness: OpenLateness,
}

impl LateStaff {
    pub(crate) const fn new(employee: PersonId, lateness: OpenLateness) -> Self {
        Self { employee, lateness }
    }

    /// Who was late.
    pub const fn employee(&self) -> PersonId {
        self.employee
    }

    /// The lateness.
    pub const fn lateness(&self) -> &OpenLateness {
        &self.lateness
    }
}

/// [`Employment`] as its holder is told it: the record itself and, for a manager, every member of
/// staff whose lateness may still be penalized, in entity order (`late_staff`, serialized only when
/// non-empty, so everybody else's disclosure keeps its shape).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DisclosedEmployment {
    #[serde(flatten)]
    employment: Employment,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    late_staff: Vec<LateStaff>,
    /// The holder's attributes this pack owns, as an attribute block (S30-f D-f-1): present only where
    /// the world enables `attributes` and something is disclosable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    attributes: Option<Value>,
}

impl DisclosedEmployment {
    pub(crate) const fn new(
        employment: Employment,
        late_staff: Vec<LateStaff>,
        attributes: Option<Value>,
    ) -> Self {
        Self {
            employment,
            late_staff,
            attributes,
        }
    }

    /// The attribute block this pack disclosed with it, if any.
    pub const fn attributes(&self) -> Option<&Value> {
        self.attributes.as_ref()
    }

    /// The record.
    pub const fn employment(&self) -> &Employment {
        &self.employment
    }

    /// The staff a manager may still penalize.
    pub fn late_staff(&self) -> &[LateStaff] {
        &self.late_staff
    }
}
