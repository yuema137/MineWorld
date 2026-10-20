//! Where drafts come from (design §5.3, §5.5): a source proposes `HappeningDraft { kind, place, day }`
//! and the host ([`crate::host`]) admits or withholds each. Hours, length and audience are the kind's,
//! so no source can invent them.
//!
//! Two sources ship with the pack and are called directly: `calendar` (pure functions of the date) and
//! `random` (one seeded per-mille draw per entry and day). Both implement [`HappeningGenerator`], the
//! trait other System Packs implement to contribute a generator through the `ARC-62` catalog (C3).
//!
//! **Look-ahead.** Each entry is evaluated for one target day per pass, `t = d + announce_days(kind)`,
//! so a festival announced a week ahead is drafted, admitted and announced at the midnight a week
//! before it. On the world's first pass each entry is evaluated for every `t` in `d ..= d +
//! announce_days(kind)`, so a happening inside the first horizon is announced late rather than lost.

use mineworld_calendar::CalendarDate;
use mineworld_contracts::{PlaceId, SystemId};
use mineworld_kernel::WorldRead;
use serde::{Deserialize, Serialize};

use crate::draw::{self, DRAW_FIRES};
use crate::entry::CalendarEntry;
use crate::event::KeyedRandom;
use crate::kind::{HappeningKindId, KindCatalog};

/// A world day: its local midnight is `day × 86 400` seconds after instant 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DayIndex(i64);

impl DayIndex {
    /// The day numbered `day`.
    pub const fn new(day: i64) -> Self {
        Self(day)
    }

    /// Its number.
    pub const fn get(self) -> i64 {
        self.0
    }

    /// The day `days` later.
    #[must_use]
    pub const fn plus(self, days: i64) -> Self {
        Self(self.0 + days)
    }
}

/// What a source proposes: this kind, at this place, on this day. Everything else is the kind's.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HappeningDraft {
    kind: HappeningKindId,
    place: PlaceId,
    day: DayIndex,
}

impl HappeningDraft {
    /// A draft of `kind` at `place` on `day`.
    pub const fn new(kind: HappeningKindId, place: PlaceId, day: DayIndex) -> Self {
        Self { kind, place, day }
    }

    /// The kind.
    pub const fn kind(&self) -> &HappeningKindId {
        &self.kind
    }

    /// The place.
    pub const fn place(&self) -> PlaceId {
        self.place
    }

    /// The day it would begin on.
    pub const fn day(&self) -> DayIndex {
        self.day
    }
}

/// Who proposed a draft: one of the bundled sources, or an external generator by its pack's id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DraftSource {
    /// A `calendar:` entry.
    Calendar,
    /// A `random:` entry.
    Random,
    /// An external generator (`ARC-62`), by its pack's system id.
    Generator(SystemId),
}

/// What a generator is told: the day being opened and the catalog it must draft within. Built only by
/// this pack, from the calendar's fact and its own state.
pub struct DraftAsk<'a> {
    /// The day being opened, `d`.
    pub day: DayIndex,
    /// `d`'s local date (calendar's).
    pub date: CalendarDate,
    /// Whether this is the world's first pass, the catch-up (module documentation).
    pub first_pass: bool,
    /// The kinds, their places and their horizons.
    pub catalog: &'a KindCatalog,
    seed: u64,
    generator: u64,
}

impl<'a> DraftAsk<'a> {
    pub(crate) fn new(
        day: DayIndex,
        date: CalendarDate,
        first_pass: bool,
        catalog: &'a KindCatalog,
        seed: u64,
    ) -> Self {
        Self {
            day,
            date,
            first_pass,
            catalog,
            seed,
            generator: 0,
        }
    }

    /// The same ask, its draws keyed by `generator`, so two generators never share a draw.
    pub(crate) fn for_generator(&self, generator: &SystemId) -> Self {
        Self {
            day: self.day,
            date: self.date,
            first_pass: self.first_pass,
            catalog: self.catalog,
            seed: self.seed,
            generator: draw::key(&["generator", generator.as_str()]),
        }
    }

    /// Draw `index` for target day `t`, per mille: counter-based over (the world's seed, `t`, the
    /// generator's id, `index`), so a generator never needs, or keeps, a random source.
    pub const fn permille(&self, t: DayIndex, index: u64) -> u16 {
        draw::draw(self.seed, t.get(), self.generator, index)
    }

    /// The local date of day `t`, counted from `d`'s date; [`None`] only beyond the calendar's range.
    pub const fn date_of(&self, t: DayIndex) -> Option<CalendarDate> {
        CalendarDate::from_days(self.date.days() + (t.get() - self.day.get()))
    }

    /// The target days of an entry of `kind` on this pass: `d + announce_days`, or on the first pass
    /// every day from `d` to it. Empty for a kind the catalog lacks.
    pub fn targets(&self, kind: &HappeningKindId) -> impl Iterator<Item = DayIndex> + use<> {
        let horizon = self
            .catalog
            .get(kind)
            .map(|kind| i64::from(kind.announce_days().get()));
        let (first, last) = match horizon {
            Some(horizon) if self.first_pass => (self.day.get(), self.day.get() + horizon),
            Some(horizon) => (self.day.get() + horizon, self.day.get() + horizon),
            None => (1, 0),
        };
        (first..=last).map(DayIndex)
    }
}

/// A source of happening drafts, owned by `happenings` and implemented by other System Packs
/// (`ARC-62`).
///
/// The contract an implementation keeps (`ARC-62` item 4): **pure** — its drafts are a function of
/// `world` and `ask` alone, no state kept between calls, no clock, no random source but
/// [`DraftAsk::permille`]; **inert where its state is absent** — it returns nothing where its own pack's
/// state is absent; **a proposal, never a decision** — every draft passes this pack's admission, which
/// enforces the kind's places and horizon and the world's limits.
pub trait HappeningGenerator: Send + Sync {
    /// The implementing pack's system id: the name a world selects in `generators:`. Generators are
    /// asked in ascending order of it.
    fn generator_of(&self) -> SystemId;

    /// The drafts for `ask`. At most [`MAX_DRAFTS_PER_GENERATOR`](crate::host::MAX_DRAFTS_PER_GENERATOR)
    /// are considered; the rest are withheld.
    fn drafts(&self, world: &WorldRead<'_>, ask: &DraftAsk<'_>) -> Vec<HappeningDraft>;
}

/// The bundled `calendar` source: each entry on each target day whose date its recurrence names.
pub(crate) struct CalendarGenerator<'c>(pub(crate) &'c [CalendarEntry<PlaceId>]);

impl HappeningGenerator for CalendarGenerator<'_> {
    fn generator_of(&self) -> SystemId {
        SystemId::from_static("calendar")
    }

    fn drafts(&self, _: &WorldRead<'_>, ask: &DraftAsk<'_>) -> Vec<HappeningDraft> {
        self.0
            .iter()
            .flat_map(|entry| {
                ask.targets(entry.kind())
                    .filter(|t| ask.date_of(*t).is_some_and(|date| entry.when().fires(date)))
                    .map(|t| HappeningDraft::new(entry.kind().clone(), *entry.place(), t))
            })
            .collect()
    }
}

/// The bundled `random` source: each entry on each target day its weekdays and months admit, if that
/// day's draw for its key falls below its chance.
pub(crate) struct RandomGenerator<'c> {
    pub(crate) seed: u64,
    pub(crate) entries: &'c [KeyedRandom],
}

impl RandomGenerator<'_> {
    /// Whether `entry` fires on day `t`: `draw(seed, t, key, DRAW_FIRES) < per_mille_per_day`.
    pub(crate) const fn fires(seed: u64, entry: &KeyedRandom, t: DayIndex) -> bool {
        draw::draw(seed, t.get(), entry.key(), DRAW_FIRES) < entry.entry().per_mille().get()
    }
}

impl HappeningGenerator for RandomGenerator<'_> {
    fn generator_of(&self) -> SystemId {
        SystemId::from_static("random")
    }

    fn drafts(&self, _: &WorldRead<'_>, ask: &DraftAsk<'_>) -> Vec<HappeningDraft> {
        self.entries
            .iter()
            .flat_map(|keyed| {
                let entry = keyed.entry();
                ask.targets(entry.kind())
                    .filter(|t| ask.date_of(*t).is_some_and(|date| entry.admits(date)))
                    .filter(|t| Self::fires(self.seed, keyed, *t))
                    .map(|t| HappeningDraft::new(entry.kind().clone(), *entry.place(), t))
            })
            .collect()
    }
}
