//! The host (design §5.3): every draft from every source is admitted or withheld here, in a fixed
//! order, against the happenings already open and those admitted earlier in the same pass. The limits
//! are the host's, so no source can exceed them (INV-WE-6), invent hours, or schedule the past.
//!
//! All decisions are taken when a happening is announced, never when it begins, so an announced
//! happening always begins and ends.

use mineworld_calendar::DAY;
use mineworld_contracts::{PlaceId, WorldTime};
use serde::{Deserialize, Serialize};

use crate::configuration::Limits;
use crate::generate::{DayIndex, HappeningDraft};
use crate::kind::{HappeningKind, HappeningKindId, KindCatalog};

#[cfg(test)]
mod tests;

/// The most drafts one external generator may offer in one pass; the rest are withheld together.
pub const MAX_DRAFTS_PER_GENERATOR: usize = 16;

/// Seconds in a minute.
const MINUTE: i64 = 60;

/// A happening's identity: its kind, its place and its first instant. Unique, by the `Duplicate`
/// check; every fact about it carries it, so a reacting pack keys its own state by a value it can read
/// from any of them.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct HappeningRef {
    kind: HappeningKindId,
    place: PlaceId,
    start: WorldTime,
}

impl HappeningRef {
    /// The kind.
    pub const fn kind(&self) -> &HappeningKindId {
        &self.kind
    }

    /// The place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The instant it begins.
    pub const fn start(&self) -> WorldTime {
        self.start
    }
}

/// A happening announced and not yet ended: its identity and the instant it ends.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct OpenHappening {
    happening: HappeningRef,
    until: WorldTime,
}

impl OpenHappening {
    pub(crate) const fn new(happening: HappeningRef, until: WorldTime) -> Self {
        Self { happening, until }
    }

    /// Its identity.
    pub const fn happening(&self) -> &HappeningRef {
        &self.happening
    }

    /// The instant it ends.
    pub const fn until(&self) -> WorldTime {
        self.until
    }

    /// Its identity, taken out.
    pub(crate) fn into_happening(self) -> HappeningRef {
        self.happening
    }

    /// Whether it overlaps `[start, until)`.
    fn overlaps(&self, start: WorldTime, until: WorldTime) -> bool {
        self.happening.start < until && start < self.until
    }
}

/// Why a draft was not admitted, in the order the checks run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WithheldReason {
    /// The kind is not in the world's catalog.
    UnknownKind,
    /// The place is not one of the kind's places.
    PlaceNotAdmitted,
    /// The day is in the past or beyond the kind's announcement horizon.
    OutsideWindow,
    /// The same kind already happens at the same place from the same instant.
    Duplicate,
    /// `limits.per_day` happenings already begin that day.
    OverDailyLimit,
    /// `limits.per_place_running` happenings already overlap it at the place.
    PlaceBusy,
    /// The generator offered more than [`MAX_DRAFTS_PER_GENERATOR`] drafts; this one and every later
    /// one of that pass.
    OverGeneratorCap,
}

/// The instants a draft of `kind` on day `t` runs between: day `t`'s `from` to day `t + days − 1`'s
/// `until`.
pub(crate) fn span<P: Ord + Clone>(kind: &HappeningKind<P>, t: DayIndex) -> (WorldTime, WorldTime) {
    let hours = kind.hours();
    let start = t.get() * DAY + i64::from(hours.from()) * MINUTE;
    let last = t.get() + i64::from(kind.days().get()) - 1;
    let until = last * DAY + i64::from(hours.until()) * MINUTE;
    (
        WorldTime::from_seconds(start),
        WorldTime::from_seconds(until),
    )
}

/// One pass's admission: the day being opened, its first instant, the catalog and limits, and every
/// happening open or admitted so far.
pub(crate) struct Admission<'c> {
    day: DayIndex,
    now: WorldTime,
    catalog: &'c KindCatalog,
    limits: Limits,
    open: Vec<OpenHappening>,
}

impl<'c> Admission<'c> {
    pub(crate) const fn new(
        day: DayIndex,
        now: WorldTime,
        catalog: &'c KindCatalog,
        limits: Limits,
        open: Vec<OpenHappening>,
    ) -> Self {
        Self {
            day,
            now,
            catalog,
            limits,
            open,
        }
    }

    /// Admits `draft` — returning the happening it becomes, now counted against every later draft — or
    /// says why not. The checks run in the order of [`WithheldReason`].
    pub(crate) fn admit(
        &mut self,
        draft: &HappeningDraft,
    ) -> Result<OpenHappening, WithheldReason> {
        let kind = self
            .catalog
            .get(draft.kind())
            .ok_or(WithheldReason::UnknownKind)?;
        if !kind.admits(&draft.place()) {
            return Err(WithheldReason::PlaceNotAdmitted);
        }
        let horizon = i64::from(kind.announce_days().get());
        let (start, until) = span(kind, draft.day());
        let inside = self.day <= draft.day() && draft.day() <= self.day.plus(horizon);
        if !inside || start < self.now {
            return Err(WithheldReason::OutsideWindow);
        }
        let happening = HappeningRef {
            kind: draft.kind().clone(),
            place: draft.place(),
            start,
        };
        if self.open.iter().any(|open| open.happening == happening) {
            return Err(WithheldReason::Duplicate);
        }
        let beginning_that_day = self
            .open
            .iter()
            .filter(|open| open.happening.start.seconds().div_euclid(DAY) == draft.day().get())
            .count();
        if beginning_that_day >= usize::from(self.limits.per_day().get()) {
            return Err(WithheldReason::OverDailyLimit);
        }
        let overlapping = self
            .open
            .iter()
            .filter(|open| open.happening.place == draft.place() && open.overlaps(start, until))
            .count();
        if overlapping >= usize::from(self.limits.per_place_running().get()) {
            return Err(WithheldReason::PlaceBusy);
        }
        let admitted = OpenHappening::new(happening, until);
        self.open.push(admitted.clone());
        Ok(admitted)
    }
}
