//! Civil dates in integers: the proleptic Gregorian calendar, a day number, a weekday.
//!
//! `days_from_civil` and `civil_from_days` are Howard Hinnant's public-domain algorithms
//! ("`chrono`-Compatible Low-Level Date Algorithms", <https://howardhinnant.github.io/date_algorithms.html>,
//! released to the public domain by their author), transcribed for `i64`. A day number counts days
//! from 1970-01-01, which is day 0. `DEP-30` records why these two functions are built rather than
//! adopted from `chrono` or `time`.

use serde::{Deserialize, Serialize};

/// A date of the proleptic Gregorian calendar. Valid by construction: [`CalendarDate::new`] refuses a
/// month or day that does not exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CalendarDate {
    year: i32,
    month: u8,
    day: u8,
}

/// Whether `year` is a leap year of the Gregorian calendar.
pub const fn is_leap(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

/// How many days `month` of `year` has.
pub const fn days_in_month(year: i64, month: u8) -> u8 {
    match month {
        2 if is_leap(year) => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

impl CalendarDate {
    /// The date, if it exists.
    pub const fn new(year: i32, month: u8, day: u8) -> Option<Self> {
        if month < 1 || month > 12 || day < 1 || day > days_in_month(year as i64, month) {
            return None;
        }
        Some(Self { year, month, day })
    }

    /// The year.
    pub const fn year(self) -> i32 {
        self.year
    }

    /// The month, 1 … 12.
    pub const fn month(self) -> u8 {
        self.month
    }

    /// The day of the month, 1 … 31.
    pub const fn day(self) -> u8 {
        self.day
    }

    /// Days from 1970-01-01 (Hinnant's `days_from_civil`).
    pub const fn days(self) -> i64 {
        let month = self.month as i64;
        let day = self.day as i64;
        let year = self.year as i64 - if month <= 2 { 1 } else { 0 };
        let era = year.div_euclid(400);
        let year_of_era = year - era * 400; // [0, 399]
        let month_from_march = (month + 9) % 12; // March = 0
        let day_of_year = (153 * month_from_march + 2) / 5 + day - 1; // [0, 365]
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }

    /// The date of a day number (Hinnant's `civil_from_days`). `None` only for a year outside `i32`.
    pub const fn from_days(days: i64) -> Option<Self> {
        let z = days + 719_468;
        let era = z.div_euclid(146_097);
        let day_of_era = z - era * 146_097; // [0, 146096]
        let year_of_era =
            (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
        let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
        let month_from_march = (5 * day_of_year + 2) / 153;
        let day = day_of_year - (153 * month_from_march + 2) / 5 + 1;
        let month = if month_from_march < 10 {
            month_from_march + 3
        } else {
            month_from_march - 9
        };
        let year = year_of_era + era * 400 + if month <= 2 { 1 } else { 0 };
        if year < i32::MIN as i64 || year > i32::MAX as i64 {
            return None;
        }
        Some(Self {
            year: year as i32,
            month: month as u8,
            day: day as u8,
        })
    }

    /// The weekday, 0 = Monday … 6 = Sunday. 1970-01-01 was a Thursday.
    pub const fn weekday(self) -> u8 {
        (self.days() + 3).rem_euclid(7) as u8
    }

    /// The next day.
    pub const fn next(self) -> Option<Self> {
        Self::from_days(self.days() + 1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every day of 1901 … 2099 round-trips, consecutive dates are consecutive day numbers, and the
    /// weekday advances by one each day — an exhaustive check over the configurable range, against
    /// the calendar's own definition rather than against a second copy of the algorithm.
    #[test]
    fn every_day_from_1901_to_2099_round_trips_and_is_one_after_the_last() {
        let mut expected = CalendarDate::new(1901, 1, 1).expect("exists").days();
        let mut weekday = CalendarDate::new(1901, 1, 1).expect("exists").weekday();
        let mut count = 0_i64;
        for year in 1901..=2099_i32 {
            for month in 1..=12_u8 {
                for day in 1..=days_in_month(i64::from(year), month) {
                    let date = CalendarDate::new(year, month, day).expect("exists");
                    assert_eq!(date.days(), expected, "{date:?}");
                    assert_eq!(CalendarDate::from_days(expected), Some(date));
                    assert_eq!(date.weekday(), weekday, "{date:?}");
                    expected += 1;
                    weekday = (weekday + 1) % 7;
                    count += 1;
                }
            }
        }
        // 199 years, 49 of them leap (1904 … 2096 every fourth year; 2000 is one).
        assert_eq!(count, 199 * 365 + 49);
    }

    /// Fixed points read from the calendar, not from this code: 1970-01-01 is day 0 and a Thursday;
    /// 2000-03-01 follows 2000-02-29; 2026-10-08 is a Thursday (step-19 §1.3); 1900 is not a leap year
    /// and 2000 is.
    #[test]
    fn known_dates_have_their_known_day_numbers_and_weekdays() {
        let epoch = CalendarDate::new(1970, 1, 1).expect("exists");
        assert_eq!(epoch.days(), 0);
        assert_eq!(epoch.weekday(), 3, "Thursday");
        let leap = CalendarDate::new(2000, 2, 29).expect("2000 is a leap year");
        assert_eq!(leap.next(), CalendarDate::new(2000, 3, 1));
        assert_eq!(
            CalendarDate::new(1900, 2, 29),
            None,
            "1900 is not a leap year"
        );
        assert_eq!(CalendarDate::new(2026, 2, 29), None);
        let requirement = CalendarDate::new(2026, 10, 8).expect("exists");
        assert_eq!(requirement.weekday(), 3, "8 October 2026 is a Thursday");
        assert_eq!(requirement.days(), 20_734);
        assert_eq!(CalendarDate::new(2026, 13, 1), None);
        assert_eq!(CalendarDate::new(2026, 4, 31), None);
    }
}
