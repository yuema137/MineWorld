//! The bundled generators' entries (design §5.1, §5.3): `calendar:` entries, which fire on dates, and
//! `random:` entries, which fire on a seeded per-mille chance per day. Both name a kind and a place;
//! hours, length and audience are the kind's, so an entry cannot invent them.
//!
//! Whether a calendar entry fires is a pure function of a date — calendar's [`CalendarDate`], never
//! the host's clock.

use mineworld_calendar::CalendarDate;
use mineworld_calendar::civil::days_in_month;
use mineworld_contracts::EntityKey;
use serde::{Deserialize, Serialize};

use crate::kind::{HappeningKindId, bounded};

#[cfg(test)]
mod tests;

/// The weekdays' names, Monday first: calendar's weekday 0 is Monday.
const WEEKDAYS: [&str; 7] = [
    "monday",
    "tuesday",
    "wednesday",
    "thursday",
    "friday",
    "saturday",
    "sunday",
];

/// A non-empty set of weekdays, written as their names: `[saturday, sunday]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Vec<String>", into = "Vec<String>")]
pub struct WeekdaySet(u8);

impl WeekdaySet {
    /// Whether `date` falls on one of these weekdays.
    pub const fn admits(self, date: CalendarDate) -> bool {
        self.0 & (1 << date.weekday()) != 0
    }
}

impl TryFrom<Vec<String>> for WeekdaySet {
    type Error = String;

    fn try_from(names: Vec<String>) -> Result<Self, Self::Error> {
        let mut bits = 0_u8;
        for name in &names {
            let Some(index) = WEEKDAYS.iter().position(|weekday| weekday == name) else {
                return Err(format!(
                    "weekdays are named monday … sunday in lowercase, not '{name}'"
                ));
            };
            if bits & (1 << index) != 0 {
                return Err(format!("weekdays lists '{name}' twice"));
            }
            bits |= 1 << index;
        }
        if bits == 0 {
            return Err("weekdays lists at least one weekday".to_owned());
        }
        Ok(Self(bits))
    }
}

impl From<WeekdaySet> for Vec<String> {
    fn from(value: WeekdaySet) -> Self {
        (0..7)
            .filter(|index| value.0 & (1 << index) != 0)
            .map(|index| WEEKDAYS[index].to_owned())
            .collect()
    }
}

/// A non-empty set of months, written as their numbers: `[5, 6, 7, 8, 9]`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(try_from = "Vec<u8>", into = "Vec<u8>")]
pub struct MonthSet(u16);

impl MonthSet {
    /// Whether `date` falls in one of these months.
    pub const fn admits(self, date: CalendarDate) -> bool {
        self.0 & (1 << date.month()) != 0
    }
}

impl TryFrom<Vec<u8>> for MonthSet {
    type Error = String;

    fn try_from(months: Vec<u8>) -> Result<Self, Self::Error> {
        let mut bits = 0_u16;
        for month in months {
            if !(1..=12).contains(&month) {
                return Err(format!("months are 1 to 12, not {month}"));
            }
            if bits & (1 << month) != 0 {
                return Err(format!("months lists {month} twice"));
            }
            bits |= 1 << month;
        }
        if bits == 0 {
            return Err("months lists at least one month".to_owned());
        }
        Ok(Self(bits))
    }
}

impl From<MonthSet> for Vec<u8> {
    fn from(value: MonthSet) -> Self {
        (1..=12)
            .filter(|month| value.0 & (1 << month) != 0)
            .collect()
    }
}

/// When a calendar entry fires: on some weekdays, on one date a year, or on one day of every month.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Recurrence {
    /// Every week, on these weekdays.
    Weekdays(WeekdaySet),
    /// Every year on `month`-`day`; `02-29` fires only in leap years.
    Yearly {
        /// 1 … 12.
        month: u8,
        /// 1 … the month's longest length.
        day: u8,
    },
    /// Every month on `day`, 1 … 28.
    Monthly {
        /// 1 … 28.
        day: u8,
    },
}

impl Recurrence {
    /// Whether it fires on `date`.
    pub const fn fires(self, date: CalendarDate) -> bool {
        match self {
            Self::Weekdays(set) => set.admits(date),
            Self::Yearly { month, day } => date.month() == month && date.day() == day,
            Self::Monthly { day } => date.day() == day,
        }
    }
}

/// A yearly date written `MM-DD`, valid for its month in a leap year.
struct MonthDay(u8, u8);

impl TryFrom<&str> for MonthDay {
    type Error = String;

    fn try_from(text: &str) -> Result<Self, Self::Error> {
        let refused = || {
            format!("date is MM-DD, a day of the year such as \"10-31\" or \"02-29\", not '{text}'")
        };
        let (month, day) = text.split_once('-').ok_or_else(refused)?;
        if month.len() != 2 || day.len() != 2 {
            return Err(refused());
        }
        let (Ok(month), Ok(day)) = (month.parse::<u8>(), day.parse::<u8>()) else {
            return Err(refused());
        };
        // 2000 is a leap year, so February admits its 29th.
        if !(1..=12).contains(&month) || day == 0 || day > days_in_month(2000, month) {
            return Err(refused());
        }
        Ok(Self(month, day))
    }
}

/// One `calendar:` entry: the kind, the place, and when. `P` is how a place is named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CalendarEntry<P> {
    kind: HappeningKindId,
    place: P,
    when: Recurrence,
}

impl<P> CalendarEntry<P> {
    /// The kind it schedules.
    pub const fn kind(&self) -> &HappeningKindId {
        &self.kind
    }

    /// Where.
    pub const fn place(&self) -> &P {
        &self.place
    }

    /// When.
    pub const fn when(&self) -> Recurrence {
        self.when
    }

    /// The same entry at a renamed place.
    pub(crate) fn at<Q>(&self, place: Q) -> CalendarEntry<Q> {
        CalendarEntry {
            kind: self.kind.clone(),
            place,
            when: self.when,
        }
    }
}

/// A calendar entry as the file writes it: exactly one of `weekdays`, `date` and `monthday`.
#[derive(Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
struct RawCalendarEntry {
    /// The kind it schedules, by id.
    kind: HappeningKindId,
    /// Where, by key: one of the kind's places.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    place: EntityKey,
    /// Every week on these weekdays, such as [saturday].
    #[serde(default)]
    weekdays: Option<WeekdaySet>,
    /// Every year on this date, MM-DD, such as "10-31".
    #[serde(default)]
    date: Option<String>,
    /// Every month on this day, 1 to 28.
    #[serde(default)]
    monthday: Option<u8>,
}

/// A calendar entry decoded from the file.
#[derive(Deserialize)]
#[serde(try_from = "RawCalendarEntry")]
pub(crate) struct FileCalendarEntry(pub(crate) CalendarEntry<EntityKey>);

/// The schema is the file's shape: deserializing goes through [`RawCalendarEntry`].
impl schemars::JsonSchema for FileCalendarEntry {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        std::borrow::Cow::Borrowed("CalendarEntry")
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        <RawCalendarEntry as schemars::JsonSchema>::json_schema(generator)
    }
}

impl TryFrom<RawCalendarEntry> for FileCalendarEntry {
    type Error = String;

    fn try_from(raw: RawCalendarEntry) -> Result<Self, Self::Error> {
        let when = match (raw.weekdays, raw.date, raw.monthday) {
            (Some(set), None, None) => Recurrence::Weekdays(set),
            (None, Some(date), None) => {
                let MonthDay(month, day) = MonthDay::try_from(date.as_str())?;
                Recurrence::Yearly { month, day }
            }
            (None, None, Some(day)) if (1..=28).contains(&day) => Recurrence::Monthly { day },
            (None, None, Some(day)) => {
                return Err(format!(
                    "monthday is 1 to 28, a day every month has, not {day}: use a date entry for a \
                     later day"
                ));
            }
            _ => {
                return Err(
                    "a calendar entry names exactly one of weekdays, date and monthday".to_owned(),
                );
            }
        };
        Ok(Self(CalendarEntry {
            kind: raw.kind,
            place: raw.place,
            when,
        }))
    }
}

bounded!(
    /// A chance per day, per mille: 0 never, 1 000 every day.
    PerMille: u16 = "u16", 0, 1_000, 0, "per_mille_per_day"
);

/// One `random:` entry: the kind, the place, its chance per day, and the weekdays and months it may
/// fire in (all, if not given). `P` is how a place is named.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "RandomEntry", bound = "")]
pub struct RandomEntry<P> {
    /// The kind it draws, by id.
    kind: HappeningKindId,
    /// Where, by key: one of the kind's places.
    #[schemars(with = "mineworld_authoring::schema::Key")]
    place: P,
    /// Its chance per day, per mille: 0 never, 1000 every day.
    per_mille_per_day: PerMille,
    /// The weekdays it may fire on; every weekday when unsaid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    weekdays: Option<WeekdaySet>,
    /// The months it may fire in, 1 to 12; every month when unsaid.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    months: Option<MonthSet>,
}

impl<P> RandomEntry<P> {
    /// The kind it draws.
    pub const fn kind(&self) -> &HappeningKindId {
        &self.kind
    }

    /// Where.
    pub const fn place(&self) -> &P {
        &self.place
    }

    /// Its chance per day.
    pub const fn per_mille(&self) -> PerMille {
        self.per_mille_per_day
    }

    /// Whether its weekdays and months admit `date` (every date, when it names neither).
    pub fn admits(&self, date: CalendarDate) -> bool {
        self.weekdays.is_none_or(|set| set.admits(date))
            && self.months.is_none_or(|set| set.admits(date))
    }

    /// The same entry at a renamed place.
    pub(crate) fn at<Q>(&self, place: Q) -> RandomEntry<Q> {
        RandomEntry {
            kind: self.kind.clone(),
            place,
            per_mille_per_day: self.per_mille_per_day,
            weekdays: self.weekdays,
            months: self.months,
        }
    }
}
