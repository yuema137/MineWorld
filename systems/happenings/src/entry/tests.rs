//! T2: recurrence against dates read from a calendar, never computed by the code under test. Owns
//! "a calendar entry fires on the wrong day".

use mineworld_calendar::CalendarDate;

use super::{MonthSet, Recurrence, WeekdaySet};

fn date(year: i32, month: u8, day: u8) -> CalendarDate {
    CalendarDate::new(year, month, day).expect("a real date")
}

fn weekdays(names: &[&str]) -> WeekdaySet {
    WeekdaySet::try_from(
        names
            .iter()
            .map(|name| (*name).to_owned())
            .collect::<Vec<_>>(),
    )
    .expect("valid")
}

/// 2026-10-10 and 2026-10-17 are Saturdays, 2026-10-11 a Sunday, 2026-10-08 a Thursday (read from a
/// printed calendar for October 2026).
#[test]
fn a_weekday_entry_fires_on_its_weekdays_only() {
    let saturday = Recurrence::Weekdays(weekdays(&["saturday"]));
    assert!(saturday.fires(date(2026, 10, 10)));
    assert!(saturday.fires(date(2026, 10, 17)));
    assert!(!saturday.fires(date(2026, 10, 11)));
    assert!(!saturday.fires(date(2026, 10, 8)));
    assert!(!saturday.fires(date(2026, 10, 12)), "a Monday");

    let weekend = Recurrence::Weekdays(weekdays(&["sunday", "saturday"]));
    let fired: Vec<u8> = (5..=18)
        .filter(|day| weekend.fires(date(2026, 10, *day)))
        .collect();
    assert_eq!(fired, [10, 11, 17, 18]);
}

/// 10-31 fires once a year; 02-29 fires on 2028-02-29 (a leap year) and on no day of 2027, which has
/// none.
#[test]
fn a_yearly_entry_fires_on_its_date_and_02_29_only_in_leap_years() {
    let halloween = Recurrence::Yearly { month: 10, day: 31 };
    assert!(halloween.fires(date(2026, 10, 31)));
    assert!(halloween.fires(date(2027, 10, 31)));
    assert!(!halloween.fires(date(2026, 10, 30)));
    assert!(!halloween.fires(date(2026, 11, 1)));

    let leap = Recurrence::Yearly { month: 2, day: 29 };
    assert!(leap.fires(date(2028, 2, 29)));
    assert_eq!(CalendarDate::new(2027, 2, 29), None);
    let days_of_2027 = (1..=12_u8).flat_map(|month| (1..=31_u8).map(move |day| (month, day)));
    assert!(
        days_of_2027
            .filter_map(|(month, day)| CalendarDate::new(2027, month, day))
            .all(|day| !leap.fires(day))
    );
}

#[test]
fn a_monthly_entry_fires_on_its_day_of_every_month() {
    let first = Recurrence::Monthly { day: 1 };
    assert!(first.fires(date(2026, 11, 1)));
    assert!(first.fires(date(2027, 2, 1)));
    assert!(!first.fires(date(2026, 10, 31)));
    assert!(!first.fires(date(2026, 11, 2)));
}

/// May … September admits 2026-05-01 and 2026-09-30, not 2026-04-30 or 2026-10-01.
#[test]
fn a_month_set_admits_its_months() {
    let summer = MonthSet::try_from(vec![5, 6, 7, 8, 9]).expect("valid");
    assert!(summer.admits(date(2026, 5, 1)));
    assert!(summer.admits(date(2026, 9, 30)));
    assert!(!summer.admits(date(2026, 4, 30)));
    assert!(!summer.admits(date(2026, 10, 1)));
}

#[test]
fn sets_refuse_empty_repeated_and_unknown_members() {
    assert!(WeekdaySet::try_from(Vec::<String>::new()).is_err());
    assert!(WeekdaySet::try_from(vec!["saturday".to_owned(), "saturday".to_owned()]).is_err());
    assert!(WeekdaySet::try_from(vec!["Saturday".to_owned()]).is_err());
    assert!(MonthSet::try_from(Vec::<u8>::new()).is_err());
    assert!(MonthSet::try_from(vec![0]).is_err());
    assert!(MonthSet::try_from(vec![13]).is_err());
    assert!(MonthSet::try_from(vec![5, 5]).is_err());
}
