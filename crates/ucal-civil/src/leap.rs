//! The leap-second table (Rule L, §8.4).
//!
//! # Why the table is bundled
//!
//! §8.4 requires offline operation with no runtime network access, and requires
//! `ucal doctor` to report the table's version so that a stale table warns
//! (`UCAL-W0002`) rather than converting silently. Both need the table to be
//! data this crate owns and versions, so it is declared here rather than read
//! from a dependency.
//!
//! # What the table is for, and what it is not for
//!
//! Rule L: **TT is the only pivot.** These values exist solely at the UTC
//! parse/format boundary. No absolute-time arithmetic consults them, and no
//! quantity derived from them ever enters a tick computation — a leap second
//! changes what a UTC *label* means, never how much time has elapsed.
//!
//! # UTC before 1972
//!
//! Between 1961 and 1972, UTC used variable-rate "rubber seconds" and fractional
//! offsets, so `TAI - UTC` was not an integer and UTC was not a uniform scale.
//! Converting a pre-1972 UTC label would require modelling that rate history,
//! which is out of scope and would silently produce a number of unstated
//! accuracy. Such labels are rejected with `UCAL-E0041`. TT and TAI have no such
//! restriction and remain available for any epoch.

use ucal_core::{Code, TimeError};

use crate::calendar::days_from_gregorian;

/// The bundled table's version, reported by `ucal doctor` (§8.4).
///
/// IERS Bulletin C announces leap seconds; the identifier below names the last
/// bulletin this table incorporates. `Bulletin C 70` announced no leap second
/// for the period ending 2026-06-30, so the table is current as of that date.
pub const LEAP_TABLE_VERSION: &str = "IERS Bulletin C 70 (no leap second to 2026-06-30)";

/// The last date through which the bundled table is known to be complete.
///
/// A conversion for a UTC instant after this date is still performed — the table
/// is the best information available — but carries `UCAL-W0002` with the bounded
/// error, which is at most one second per unannounced leap (§8.4).
pub const TABLE_COMPLETE_THROUGH: (i64, u8, u8) = (2026, 6, 30);

/// One entry: from this UTC date at 00:00:00, `TAI - UTC` takes this value.
///
/// A leap second is inserted at `23:59:60` of the day *before* the effective
/// date, which is why the entry dates are all the first of a month.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct LeapEntry {
    /// Effective from this proleptic Gregorian date, at 00:00:00 UTC.
    pub year: i64,
    /// Month, 1-12.
    pub month: u8,
    /// Day, always 1.
    pub day: u8,
    /// `TAI - UTC` in whole seconds from this instant onward.
    pub tai_minus_utc: i64,
}

const fn e(year: i64, month: u8, tai_minus_utc: i64) -> LeapEntry {
    LeapEntry {
        year,
        month,
        day: 1,
        tai_minus_utc,
    }
}

/// The IERS leap-second table, ascending.
///
/// The first entry is the start of modern UTC: from 1972-01-01, `TAI - UTC` is
/// an integer number of seconds and steps by exactly one at each leap.
pub const LEAP_TABLE: &[LeapEntry] = &[
    e(1972, 1, 10),
    e(1972, 7, 11),
    e(1973, 1, 12),
    e(1974, 1, 13),
    e(1975, 1, 14),
    e(1976, 1, 15),
    e(1977, 1, 16),
    e(1978, 1, 17),
    e(1979, 1, 18),
    e(1980, 1, 19),
    e(1981, 7, 20),
    e(1982, 7, 21),
    e(1983, 7, 22),
    e(1985, 7, 23),
    e(1988, 1, 24),
    e(1990, 1, 25),
    e(1991, 1, 26),
    e(1992, 7, 27),
    e(1993, 7, 28),
    e(1994, 7, 29),
    e(1996, 1, 30),
    e(1997, 7, 31),
    e(1999, 1, 32),
    e(2006, 1, 33),
    e(2009, 1, 34),
    e(2012, 7, 35),
    e(2015, 7, 36),
    e(2017, 1, 37),
];

/// `TAI - TT` is exactly -32.184 s by definition, so `TT = TAI + 32.184 s`.
///
/// Expressed in milliseconds because it is exact there: 32184 ms. The tick value
/// is derived in [`crate::si`], where the bridge constant lives.
pub const TT_MINUS_TAI_MILLIS: i64 = 32_184;

/// Seconds from `0000-01-01T00:00:00` to a UTC date at midnight, ignoring leaps.
///
/// This is the "label-linear" count: the number a naive reader would compute by
/// treating every day as 86400 seconds. Adding `TAI - UTC` to it yields TAI
/// exactly, because that offset counts precisely the seconds the naive count
/// omitted.
const fn linear_seconds(year: i64, month: u8, day: u8) -> i64 {
    days_from_gregorian(year, month, day) * 86_400
}

/// The label-linear second count of a table entry's effective instant.
const fn entry_linear(entry: &LeapEntry) -> i64 {
    linear_seconds(entry.year, entry.month, entry.day)
}

/// `TAI - UTC` for a UTC label, and whether the label is inside a leap second.
///
/// `is_leap_label` must be set when the caller's clock reading has `second == 60`.
/// Such a label belongs to the *old* offset: the leap second is the extra second
/// appended to the previous day, and the new offset takes effect at the following
/// midnight. Getting this backwards would place `23:59:60` and the next
/// `00:00:00` at the same instant instead of one second apart.
pub fn tai_minus_utc(
    year: i64,
    month: u8,
    day: u8,
    is_leap_label: bool,
) -> Result<i64, TimeError> {
    let target = days_from_gregorian(year, month, day);
    let first = LEAP_TABLE[0];
    if target < days_from_gregorian(first.year, first.month, first.day) {
        return Err(TimeError::with_context(
            Code::E0041,
            "UTC is not defined before 1972-01-01: it used variable-rate seconds, \
             so TAI - UTC was not an integer. Use the TT or TAI scale instead.",
        ));
    }
    let mut offset = first.tai_minus_utc;
    for entry in LEAP_TABLE {
        let eff = days_from_gregorian(entry.year, entry.month, entry.day);
        if eff <= target {
            offset = entry.tai_minus_utc;
        }
    }

    // A `23:59:60` label needs no adjustment here, and it is worth being explicit
    // about why: the entry that the leap second precedes takes effect on the
    // *following* day, so a lookup keyed on the label's own date already returns
    // the pre-step offset. Adding one here would be the classic off-by-one that
    // collapses `23:59:60` and the next `00:00:00` onto the same instant.
    //
    // The parameter is retained and checked rather than dropped, so that a future
    // change to the table's keying cannot quietly invalidate the reasoning.
    if is_leap_label {
        debug_assert!(
            has_leap_second(year, month, day),
            "a 23:59:60 label must fall on a date the table records a leap for"
        );
        let next_entry = LEAP_TABLE.iter().find(|entry| {
            days_from_gregorian(entry.year, entry.month, entry.day) == target + 1
        });
        debug_assert_eq!(
            next_entry.map(|e| e.tai_minus_utc),
            Some(offset + 1),
            "the offset returned for a leap label must be one below the step it precedes"
        );
    }
    Ok(offset)
}

/// Whether a UTC date has a leap second appended at `23:59:60`.
///
/// True when the *following* day begins a new table entry.
pub fn has_leap_second(year: i64, month: u8, day: u8) -> bool {
    let next = days_from_gregorian(year, month, day) + 1;
    LEAP_TABLE
        .iter()
        .any(|entry| days_from_gregorian(entry.year, entry.month, entry.day) == next)
}

/// The result of resolving a TAI instant back to a UTC label.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct UtcResolution {
    /// Label-linear UTC seconds since `0000-01-01T00:00:00`.
    pub linear_seconds: i64,
    /// `TAI - UTC` in effect.
    pub tai_minus_utc: i64,
    /// Whether the instant falls inside an inserted leap second, in which case
    /// the label's `second` field is 60 (§14.2).
    pub in_leap_second: bool,
}

/// Resolve TAI seconds since `0000-01-01T00:00:00 TAI` to a UTC label.
///
/// The inserted second of entry `i` occupies the TAI half-open interval
/// `[L(D_i) + N_{i-1}, L(D_i) + N_i)`, which is exactly one second wide because
/// consecutive offsets differ by one. An instant in that interval is labelled
/// `23:59:60` on the previous day (§14.2 requires `sec = 60` rather than
/// normalisation).
pub fn utc_from_tai_seconds(tai_seconds: i64) -> Result<UtcResolution, TimeError> {
    let first = LEAP_TABLE[0];
    let floor_tai = entry_linear(&first) + first.tai_minus_utc;
    if tai_seconds < floor_tai {
        return Err(TimeError::with_context(
            Code::E0041,
            "instant precedes modern UTC (1972-01-01); use the TT or TAI scale",
        ));
    }

    let mut offset = first.tai_minus_utc;
    for (i, entry) in LEAP_TABLE.iter().enumerate() {
        if i == 0 {
            continue;
        }
        let prev = LEAP_TABLE[i - 1].tai_minus_utc;
        let step_start = entry_linear(entry) + prev;
        let step_end = entry_linear(entry) + entry.tai_minus_utc;
        if tai_seconds >= step_start && tai_seconds < step_end {
            // Inside the inserted second.
            return Ok(UtcResolution {
                linear_seconds: tai_seconds - prev,
                tai_minus_utc: prev,
                in_leap_second: true,
            });
        }
        if tai_seconds >= step_end {
            offset = entry.tai_minus_utc;
        }
    }
    Ok(UtcResolution {
        linear_seconds: tai_seconds - offset,
        tai_minus_utc: offset,
        in_leap_second: false,
    })
}

/// Whether the table is complete through the given UTC date (§8.4).
pub fn table_covers(year: i64, month: u8, day: u8) -> bool {
    let (ty, tm, td) = TABLE_COMPLETE_THROUGH;
    days_from_gregorian(year, month, day) <= days_from_gregorian(ty, tm, td)
}

/// The table's version string, for `ucal doctor` (§8.4, §14).
pub fn leap_table_version() -> &'static str {
    LEAP_TABLE_VERSION
}

/// The number of leap seconds the table records.
pub fn leap_count() -> usize {
    // The first entry establishes the initial offset rather than recording an
    // insertion, so the count is one fewer than the table length. Written as a
    // branch rather than `saturating_sub` so Rule O's lint can ban that family of
    // tokens outright instead of carrying an exemption list.
    match LEAP_TABLE.len() {
        0 => 0,
        n => n - 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn utc_days(y: i64, m: u8, d: u8) -> i64 {
        days_from_gregorian(y, m, d)
    }

    #[test]
    fn table_is_ascending_and_steps_by_one() {
        for w in LEAP_TABLE.windows(2) {
            assert!(
                utc_days(w[0].year, w[0].month, w[0].day) < utc_days(w[1].year, w[1].month, w[1].day),
                "table out of order at {:?}",
                w[0]
            );
            assert_eq!(
                w[1].tai_minus_utc - w[0].tai_minus_utc,
                1,
                "a leap second is exactly one second: {:?} -> {:?}",
                w[0],
                w[1]
            );
        }
        assert_eq!(LEAP_TABLE[0].tai_minus_utc, 10);
        assert_eq!(LEAP_TABLE[LEAP_TABLE.len() - 1].tai_minus_utc, 37);
        assert_eq!(leap_count(), 27);
    }

    #[test]
    fn entries_fall_on_the_first_of_a_month() {
        // A leap second is appended to the last day of June or December, so every
        // entry takes effect on the first of the following month.
        for entry in LEAP_TABLE {
            assert_eq!(entry.day, 1, "{entry:?}");
            assert!(entry.month == 1 || entry.month == 7, "{entry:?}");
        }
    }

    #[test]
    fn known_offsets() {
        assert_eq!(tai_minus_utc(2024, 3, 1, false).unwrap(), 37);
        assert_eq!(tai_minus_utc(2017, 1, 1, false).unwrap(), 37);
        assert_eq!(tai_minus_utc(2016, 12, 31, false).unwrap(), 36);
        assert_eq!(tai_minus_utc(1999, 1, 1, false).unwrap(), 32);
        assert_eq!(tai_minus_utc(1972, 1, 1, false).unwrap(), 10);
        assert_eq!(tai_minus_utc(1972, 6, 30, false).unwrap(), 10);
        assert_eq!(tai_minus_utc(1972, 7, 1, false).unwrap(), 11);
    }

    #[test]
    fn pre_1972_utc_is_refused_not_guessed() {
        // Rubber seconds: TAI - UTC was not an integer, so any answer here would
        // be of unstated accuracy.
        let err = tai_minus_utc(1971, 12, 31, false).unwrap_err();
        assert_eq!(err.code, Code::E0041);
        assert_eq!(tai_minus_utc(1900, 1, 1, false).unwrap_err().code, Code::E0041);
    }

    #[test]
    fn leap_second_days_are_identified() {
        assert!(has_leap_second(2016, 12, 31));
        assert!(has_leap_second(2015, 6, 30));
        assert!(has_leap_second(1972, 6, 30));
        assert!(!has_leap_second(2016, 12, 30));
        assert!(!has_leap_second(2017, 1, 1));
        assert!(!has_leap_second(2024, 6, 30));
        // Every table entry after the first is preceded by a leap-second day.
        for entry in &LEAP_TABLE[1..] {
            let prev = utc_days(entry.year, entry.month, entry.day) - 1;
            let (y, m, d) = crate::calendar::gregorian_from_days(prev);
            assert!(has_leap_second(y, m, d), "{entry:?}");
        }
    }

    #[test]
    fn the_inserted_second_is_exactly_one_second_wide() {
        // The property Rule L depends on: 23:59:60 and the following 00:00:00 are
        // one second apart in TAI, not the same instant.
        for entry in &LEAP_TABLE[1..] {
            let eff = utc_days(entry.year, entry.month, entry.day) * 86_400;
            let prev_offset = entry.tai_minus_utc - 1;
            let leap_tai = eff + prev_offset;
            let next_tai = eff + entry.tai_minus_utc;
            assert_eq!(next_tai - leap_tai, 1);

            let inside = utc_from_tai_seconds(leap_tai).unwrap();
            assert!(inside.in_leap_second, "{entry:?}");
            assert_eq!(inside.tai_minus_utc, prev_offset);

            let after = utc_from_tai_seconds(next_tai).unwrap();
            assert!(!after.in_leap_second);
            assert_eq!(after.tai_minus_utc, entry.tai_minus_utc);
            // The label steps to midnight of the effective date.
            assert_eq!(after.linear_seconds, eff);
            // ...and the leap label is 23:59:60 of the previous day, which shares
            // the same linear count as the following midnight (§14.2).
            assert_eq!(inside.linear_seconds, eff);
        }
    }

    #[test]
    fn tai_to_utc_round_trips_outside_leaps() {
        for (y, m, d) in [(1972, 1, 1), (1990, 6, 15), (2000, 1, 1), (2024, 7, 4)] {
            let linear = utc_days(y, m, d) * 86_400 + 12 * 3600;
            let offset = tai_minus_utc(y, m, d, false).unwrap();
            let tai = linear + offset;
            let r = utc_from_tai_seconds(tai).unwrap();
            assert!(!r.in_leap_second);
            assert_eq!(r.linear_seconds, linear, "{y}-{m}-{d}");
            assert_eq!(r.tai_minus_utc, offset);
        }
    }

    #[test]
    fn table_coverage_is_reported() {
        assert!(table_covers(2024, 1, 1));
        assert!(table_covers(2026, 6, 30));
        assert!(!table_covers(2026, 7, 1));
        assert!(!table_covers(2030, 1, 1));
        assert!(leap_table_version().contains("Bulletin C"));
    }
}
