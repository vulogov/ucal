//! Civil day arithmetic for the two legacy input calendars (§8.5).
//!
//! Both are **legacy** in the sense of §8.6: declared tables, not derivations.
//! They exist so that empirical inputs which arrive as civil dates can be
//! converted exactly (Rule Y), and for no other reason. Nothing here is a
//! derived calendar and nothing here may be used as one — Rule K's mechanism
//! lives in `ucal-body`, and §12 forbids that crate from depending on this one.
//!
//! Year numbering is astronomical throughout (§2.5): year `0000` **is** 1 BC and
//! `-0001` is 2 BC. Proleptic Gregorian year 0 differs from proleptic Julian
//! year 0 by two days.
//!
//! All arithmetic is integer. Day counts are taken relative to
//! `0000-01-01` proleptic Gregorian, which is the bridge epoch `SI_EPOCH`
//! (§2.1) — a statement about Earth, not about the datum.

use ucal_core::{Code, TimeError};

/// Which declared civil calendar an input date is expressed in (§8.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum CivilCalendar {
    /// Proleptic Gregorian, the default (§8.5).
    #[default]
    Gregorian,
    /// Proleptic Julian. Its year 0 differs from Gregorian year 0 by two days.
    Julian,
}

/// Days per month in a common year.
const MONTH_LENGTHS: [i64; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

/// Whether a year is a leap year in the given calendar.
pub const fn is_leap(year: i64, cal: CivilCalendar) -> bool {
    match cal {
        // The 97/400 rule. Appendix I.1 shows it is *not* a continued-fraction
        // convergent of the tropical year — 8/33 is more accurate with a
        // denominator twelve times smaller. It is declared table data, which is
        // exactly why this calendar is legacy (§8.6).
        CivilCalendar::Gregorian => {
            year.rem_euclid(4) == 0 && (year.rem_euclid(100) != 0 || year.rem_euclid(400) == 0)
        }
        // The 1/4 rule, which *is* convergent 1 (Appendix I.1).
        CivilCalendar::Julian => year.rem_euclid(4) == 0,
    }
}

/// Length of a month in days, or `None` for an invalid month number.
pub const fn month_length(year: i64, month: u8, cal: CivilCalendar) -> Option<i64> {
    if month == 0 || month > 12 {
        return None;
    }
    let base = MONTH_LENGTHS[(month - 1) as usize];
    if month == 2 && is_leap(year, cal) {
        Some(base + 1)
    } else {
        Some(base)
    }
}

/// Days from `0000-01-01` proleptic Gregorian to the given proleptic Gregorian
/// date.
///
/// Howard Hinnant's era algorithm. The `y2 - 399` adjustment exists to make
/// **truncating** division behave like flooring division; Rust's `/` truncates,
/// so it is correct here. Applying the same adjustment in a language whose
/// division already floors double-counts and puts `yoe` outside `[0, 399]` — a
/// mistake that is invisible for non-negative years and wrong by one day for
/// negative ones. `negative_years_are_exact` pins the cases that catch it.
pub const fn days_from_gregorian(year: i64, month: u8, day: u8) -> i64 {
    let y2 = if month <= 2 { year - 1 } else { year };
    let era = if y2 >= 0 { y2 } else { y2 - 399 } / 400;
    let yoe = y2 - era * 400;
    let mp = if month > 2 {
        month as i64 - 3
    } else {
        month as i64 + 9
    };
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    // -719468 puts the origin at 1970-01-01; +719528 moves it to 0000-01-01.
    era * 146097 + doe - 719468 + 719528
}

/// Days from `0000-01-01` proleptic Gregorian to the given proleptic **Julian**
/// date.
///
/// Calibrated by the Julian Day number of the two calendars' year-1 origins:
/// `JDN(Gregorian 0001-01-01) = 1721426` and `JDN(Julian 0001-01-01) = 1721424`,
/// so a Julian label denotes an instant two days earlier than the Gregorian label
/// spelled the same way, at that epoch.
pub const fn days_from_julian(year: i64, month: u8, day: u8) -> i64 {
    let y2 = if month <= 2 { year - 1 } else { year };
    let mp = if month > 2 {
        month as i64 - 3
    } else {
        month as i64 + 9
    };
    let doy = (153 * mp + 2) / 5 + day as i64 - 1;
    // Julian leap rule: every fourth year, no century exception. Flooring
    // division is needed for negative years, which `div_euclid` provides.
    let leaps = y2.div_euclid(4);
    y2 * 365 + leaps + doy + 60 - 2
}

/// Days from `0000-01-01` proleptic Gregorian, in the stated calendar.
pub const fn days_from_civil(year: i64, month: u8, day: u8, cal: CivilCalendar) -> i64 {
    match cal {
        CivilCalendar::Gregorian => days_from_gregorian(year, month, day),
        CivilCalendar::Julian => days_from_julian(year, month, day),
    }
}

/// Inverse of [`days_from_gregorian`].
pub const fn gregorian_from_days(days: i64) -> (i64, u8, u8) {
    // Hinnant's inverse works from an origin of 1970-01-01 shifted by 719468 to
    // put 0000-03-01 first; our days are relative to 0000-01-01, so the net
    // shift is -60.
    let z = days - 60;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
    let month = (if mp < 10 { mp + 3 } else { mp - 9 }) as u8;
    let year = if month <= 2 { y + 1 } else { y };
    (year, month, day)
}

/// Inverse of [`days_from_julian`].
pub const fn julian_from_days(days: i64) -> (i64, u8, u8) {
    // Invert `y2 * 365 + floor(y2/4) + doy + 58`.
    let n = days - 58;
    // Estimate the March-based year, then correct.
    let mut y2 = (4 * n).div_euclid(1461);
    loop {
        let start = y2 * 365 + y2.div_euclid(4);
        let doy = n - start;
        if doy < 0 {
            y2 -= 1;
            continue;
        }
        let year_len = if is_leap(y2 + 1, CivilCalendar::Julian) {
            366
        } else {
            365
        };
        if doy >= year_len {
            y2 += 1;
            continue;
        }
        let mp = (5 * doy + 2) / 153;
        let day = (doy - (153 * mp + 2) / 5 + 1) as u8;
        let month = if mp < 10 { mp + 3 } else { mp - 9 } as u8;
        let year = if month <= 2 { y2 + 1 } else { y2 };
        return (year, month, day);
    }
}

/// Inverse of [`days_from_civil`].
pub const fn civil_from_days(days: i64, cal: CivilCalendar) -> (i64, u8, u8) {
    match cal {
        CivilCalendar::Gregorian => gregorian_from_days(days),
        CivilCalendar::Julian => julian_from_days(days),
    }
}

/// Validate a civil date, returning `UCAL-E0041` if the day does not exist in
/// the stated calendar.
pub fn check_date(year: i64, month: u8, day: u8, cal: CivilCalendar) -> Result<(), TimeError> {
    let Some(len) = month_length(year, month, cal) else {
        return Err(TimeError::with_context(Code::E0041, "month out of range"));
    };
    if day == 0 || day as i64 > len {
        return Err(TimeError::with_context(
            Code::E0041,
            "day out of range for this month",
        ));
    }
    Ok(())
}

/// Validate a clock reading. `second == 60` is permitted here and checked
/// against the leap-second table by the caller (Rule L, `UCAL-E0042`).
pub fn check_time(hour: u8, minute: u8, second: u8) -> Result<(), TimeError> {
    if hour > 23 || minute > 59 || second > 60 {
        return Err(TimeError::with_context(
            Code::E0041,
            "clock reading out of range",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gregorian_origin_and_known_epochs() {
        assert_eq!(days_from_gregorian(0, 1, 1), 0);
        assert_eq!(days_from_gregorian(0, 1, 2), 1);
        // Year 0 is a leap year in the proleptic Gregorian calendar.
        assert!(is_leap(0, CivilCalendar::Gregorian));
        assert_eq!(days_from_gregorian(1, 1, 1), 366);
        // 1970-01-01 is the Unix epoch, 719528 days after 0000-01-01.
        assert_eq!(days_from_gregorian(1970, 1, 1), 719528);
        assert_eq!(days_from_gregorian(2000, 1, 1), 730485);
    }

    #[test]
    fn negative_years_are_exact() {
        // The case that catches the flooring/truncating era-adjustment bug.
        assert_eq!(days_from_gregorian(-43, 3, 15), -15632);
        // Cross-check by an independent route: -43 is not a leap year, so
        // March 15 is 31 + 28 + 15 - 1 = 73 days after January 1 of that year.
        assert!(!is_leap(-43, CivilCalendar::Gregorian));
        assert_eq!(days_from_gregorian(-43, 1, 1) + 73, -15632);
        // ...and every year boundary steps by exactly a year length.
        for y in -50..-40i64 {
            let len = if is_leap(y, CivilCalendar::Gregorian) { 366 } else { 365 };
            assert_eq!(
                days_from_gregorian(y + 1, 1, 1) - days_from_gregorian(y, 1, 1),
                len
            );
        }
    }

    #[test]
    fn gregorian_round_trips_over_a_wide_range() {
        for d in (-800_000..800_000).step_by(97) {
            let (y, m, day) = gregorian_from_days(d);
            assert_eq!(
                days_from_gregorian(y, m, day),
                d,
                "round trip failed at day {d} -> {y}-{m}-{day}"
            );
            assert!(check_date(y, m, day, CivilCalendar::Gregorian).is_ok());
        }
    }

    #[test]
    fn julian_is_two_days_behind_at_year_one() {
        // JDN(Gregorian 0001-01-01) = 1721426, JDN(Julian 0001-01-01) = 1721424.
        let g = days_from_gregorian(1, 1, 1);
        let j = days_from_julian(1, 1, 1);
        assert_eq!(g - j, 2, "Julian year 1 must be two days earlier");
        // §2.5: proleptic Gregorian year 0 differs from proleptic Julian year 0
        // by two days.
        assert_eq!(
            days_from_gregorian(0, 1, 1) - days_from_julian(0, 1, 1),
            2
        );
    }

    #[test]
    fn julian_round_trips() {
        for d in (-800_000..800_000).step_by(101) {
            let (y, m, day) = julian_from_days(d);
            assert_eq!(
                days_from_julian(y, m, day),
                d,
                "julian round trip failed at day {d} -> {y}-{m}-{day}"
            );
            assert!(check_date(y, m, day, CivilCalendar::Julian).is_ok());
        }
    }

    #[test]
    fn the_ides_of_march_is_a_julian_date() {
        // 44 BC-03-15 is a date in the Julian calendar. Appendix C's fixture
        // labels it as proleptic Gregorian, which is a different instant.
        let julian = days_from_julian(-43, 3, 15);
        let gregorian = days_from_gregorian(-43, 3, 15);
        assert_eq!(gregorian - julian, 2);
        // So the historical Ides falls on proleptic Gregorian -0043-03-13.
        assert_eq!(julian, days_from_gregorian(-43, 3, 13));
    }

    #[test]
    fn leap_rules_differ_at_centuries() {
        for y in [1700i64, 1800, 1900, 2100] {
            assert!(!is_leap(y, CivilCalendar::Gregorian));
            assert!(is_leap(y, CivilCalendar::Julian));
        }
        for y in [1600i64, 2000, 2400] {
            assert!(is_leap(y, CivilCalendar::Gregorian));
            assert!(is_leap(y, CivilCalendar::Julian));
        }
    }

    #[test]
    fn invalid_dates_are_rejected() {
        assert_eq!(
            check_date(2023, 2, 29, CivilCalendar::Gregorian)
                .unwrap_err()
                .code,
            Code::E0041
        );
        // ...but 2024 is a leap year.
        assert!(check_date(2024, 2, 29, CivilCalendar::Gregorian).is_ok());
        // 1900 is leap in Julian, not in Gregorian.
        assert!(check_date(1900, 2, 29, CivilCalendar::Julian).is_ok());
        assert!(check_date(1900, 2, 29, CivilCalendar::Gregorian).is_err());
        for (month, day) in [(0u8, 1u8), (13, 1)] {
            assert!(check_date(2023, month, day, CivilCalendar::Gregorian).is_err());
        }
        assert!(check_date(2023, 1, 0, CivilCalendar::Gregorian).is_err());
        assert!(check_date(2023, 4, 31, CivilCalendar::Gregorian).is_err());
        assert_eq!(check_time(24, 0, 0).unwrap_err().code, Code::E0041);
        assert_eq!(check_time(0, 60, 0).unwrap_err().code, Code::E0041);
        assert_eq!(check_time(0, 0, 61).unwrap_err().code, Code::E0041);
        // 60 is allowed here; the leap table decides (Rule L).
        assert!(check_time(23, 59, 60).is_ok());
    }
}
