//! §21.2: differential tests of `ucal-civil::si` against `hifitime`.
//!
//! The two implementations share no code. `ucal-civil` computes day counts with
//! Hinnant's era algorithm over `i64`, and scale offsets from a table declared in
//! `leap.rs`; `hifitime` keeps a `Duration` since its own TAI epoch and carries
//! its own leap-second data. Agreement between them is therefore evidence, not a
//! tautology — which is the whole point of a differential test.
//!
//! Where they disagree by construction, the test says so explicitly rather than
//! papering over it:
//!
//! - **Resolution.** An `Epoch` is nanoseconds; the bridge is thirty decimal
//!   places. Comparisons are made at nanosecond granularity.
//! - **Range.** `hifitime` uses `i32` years and a bounded `Duration`; the bridge
//!   reaches ±10¹¹ years. Only the overlap is compared.
//! - **The 1961-1972 rubber-second era.** This crate implements the published
//!   piecewise-linear definition; `hifitime` applies no pre-1972 offset in scale
//!   conversion. The divergence is asserted against the *published table* rather
//!   than against the oracle, since here the oracle is the less complete of the
//!   two.
//! - **UTC before 1961.** It did not exist; this crate refuses it and `hifitime`
//!   extrapolates.

use hifitime::{Epoch, TimeScale};
use ucal_civil::bridge::{from_epoch, hifitime_tai_minus_utc, to_epoch};
use ucal_civil::calendar::CivilCalendar;
use ucal_civil::leap::{tai_minus_utc, LEAP_TABLE};
use ucal_civil::si::{second, to_civil, Scale, SubSecond};
use ucal_civil::from_civil;
use ucal_core::backend::TickInt;
use ucal_core::{Rounding, Ticks};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

#[test]
fn gregorian_conversion_agrees_with_hifitime() {
    // The calendar arithmetic itself: same label in, same label out, over a wide
    // random sweep inside hifitime's range.
    let mut rng = Rng(0xBEEF_0001);
    let mut checked = 0;
    for _ in 0..20_000 {
        let year = 1900i32 + rng.below(300) as i32;
        let month = 1 + rng.below(12) as u8;
        let day = 1 + rng.below(28) as u8; // safe in every month
        let hour = rng.below(24) as u8;
        let minute = rng.below(60) as u8;
        let sec = rng.below(60) as u8;
        let nanos = rng.below(1_000_000_000) as u32;

        let e = Epoch::from_gregorian(year, month, day, hour, minute, sec, nanos, TimeScale::TT);
        let mine = from_civil(
            year as i64,
            month,
            day,
            hour,
            minute,
            sec,
            SubSecond::new(nanos as u128, 9).unwrap(),
            Scale::Tt,
            CivilCalendar::Gregorian,
        )
        .unwrap();

        // Round trip through the bridge must reproduce hifitime's own label.
        let (back, lossy) = to_epoch(&mine, Rounding::HalfEven).unwrap();
        assert!(!lossy, "whole nanoseconds must be exact");
        assert_eq!(
            back.to_gregorian(TimeScale::TT),
            e.to_gregorian(TimeScale::TT),
            "label mismatch at {year}-{month}-{day}T{hour}:{minute}:{sec}.{nanos}"
        );

        // ...and hifitime's epoch maps to the same instant the bridge computed.
        assert_eq!(from_epoch(e).unwrap(), mine);
        checked += 1;
    }
    assert_eq!(checked, 20_000);
}

#[test]
fn elapsed_time_agrees_with_hifitime() {
    // Durations, not just labels: the number of nanoseconds between two epochs
    // must match the number of ticks between the two instants, exactly.
    let mut rng = Rng(0xBEEF_0002);
    let ns = ucal_civil::si::nanosecond();
    for _ in 0..5_000 {
        let y1 = 1970i32 + rng.below(80) as i32;
        let y2 = 1970i32 + rng.below(80) as i32;
        let (a_y, b_y) = if y1 <= y2 { (y1, y2) } else { (y2, y1) };
        let a = Epoch::from_gregorian(a_y, 1, 1, 0, 0, 0, 0, TimeScale::TT);
        let b = Epoch::from_gregorian(b_y, 1, 1, 0, 0, 0, 0, TimeScale::TT);

        let ta = from_epoch(a).unwrap();
        let tb = from_epoch(b).unwrap();
        let mine_ticks = tb.since(&ta).unwrap();

        let hifi_nanos = (b - a).total_nanoseconds();
        assert!(hifi_nanos >= 0);
        let expect = ns
            .try_mul(&<Ticks as TickInt>::from_u128(hifi_nanos as u128).unwrap())
            .unwrap();
        assert_eq!(
            mine_ticks.ticks(),
            &expect,
            "elapsed mismatch between {a_y} and {b_y}"
        );
    }
}

#[test]
fn leap_second_table_agrees_with_hifitime() {
    // Rule L: the offsets themselves. Two independently maintained tables must
    // report the same TAI - UTC at every step in ours.
    for entry in LEAP_TABLE {
        // Just after the entry takes effect.
        let e = Epoch::from_gregorian(
            entry.year as i32,
            entry.month,
            entry.day,
            12,
            0,
            0,
            0,
            TimeScale::UTC,
        );
        let theirs = hifitime_tai_minus_utc(e);
        let mine = tai_minus_utc(entry.year, entry.month, entry.day, false).unwrap();
        assert_eq!(
            theirs as i64, mine,
            "TAI - UTC disagrees at {}-{}-{}",
            entry.year, entry.month, entry.day
        );
    }
}

#[test]
fn leap_second_instants_agree_with_hifitime() {
    // The inserted second itself, which is where a leap-second implementation is
    // most likely to be off by one.
    for entry in &LEAP_TABLE[1..] {
        // The instant one second before the entry takes effect is 23:59:60 of the
        // previous day in UTC.
        let effective = Epoch::from_gregorian(
            entry.year as i32,
            entry.month,
            entry.day,
            0,
            0,
            0,
            0,
            TimeScale::UTC,
        );
        let mine_effective = from_civil(
            entry.year,
            entry.month,
            entry.day,
            0,
            0,
            0,
            SubSecond::zero(),
            Scale::Utc,
            CivilCalendar::Gregorian,
        )
        .unwrap();
        assert_eq!(
            from_epoch(effective).unwrap(),
            mine_effective,
            "midnight after the leap at {}-{}",
            entry.year,
            entry.month
        );

        // One second earlier is the leap second, and the bridge labels it :60.
        let leap = mine_effective
            .checked_sub(&ucal_core::Delta::from_ticks(second()))
            .unwrap();
        let f = to_civil(&leap, Scale::Utc, 0, Rounding::Trunc, CivilCalendar::Gregorian).unwrap();
        assert_eq!(f.second, 60, "expected a leap label at {}-{}", entry.year, entry.month);
        assert_eq!((f.hour, f.minute), (23, 59));
    }
}

#[test]
fn utc_labels_agree_with_hifitime_after_1972() {
    let mut rng = Rng(0xBEEF_0003);
    for _ in 0..5_000 {
        let year = 1972i32 + rng.below(54) as i32;
        let month = 1 + rng.below(12) as u8;
        let day = 1 + rng.below(28) as u8;
        let hour = rng.below(24) as u8;
        let minute = rng.below(60) as u8;
        let sec = rng.below(60) as u8;

        let mine = from_civil(
            year as i64,
            month,
            day,
            hour,
            minute,
            sec,
            SubSecond::zero(),
            Scale::Utc,
            CivilCalendar::Gregorian,
        )
        .unwrap();
        let theirs =
            Epoch::from_gregorian(year, month, day, hour, minute, sec, 0, TimeScale::UTC);
        assert_eq!(
            from_epoch(theirs).unwrap(),
            mine,
            "UTC mismatch at {year}-{month}-{day}T{hour}:{minute}:{sec}"
        );

        // ...and the label survives the round trip.
        let f = to_civil(&mine, Scale::Utc, 0, Rounding::Trunc, CivilCalendar::Gregorian).unwrap();
        assert_eq!(
            (f.year, f.month, f.day, f.hour, f.minute, f.second),
            (year as i64, month, day, hour, minute, sec)
        );
    }
}

#[test]
fn rubber_second_era_diverges_from_hifitime_deliberately() {
    // A divergence where this implementation is the more complete of the two,
    // recorded so neither side can drift without the test noticing.
    //
    // The 1961-1972 definition is `TAI - UTC = A + (MJD - B) x C`. hifitime
    // carries the constant term `A` but not the rate term, and its *scale
    // conversion* applies no pre-1972 offset at all — `leap_seconds_iers()`
    // returns 0 across the whole era, so a UTC label there converts as though it
    // were TAI. That is a defensible simplification for a library aimed at modern
    // epochs; it is not the published definition, and this crate implements the
    // published definition.
    //
    // The divergence is largest late in the era, where the accumulated rate term
    // dominates the constant.
    for (y, m, d, published_micros) in [
        (1961i32, 1u8, 1u8, 1_422_818i64),   // A only; rate term is zero here
        (1965, 3, 1, 3_716_594),             // A = 3.640130, rate adds 0.076464
        (1968, 2, 1, 6_185_682),             // A = 4.213170, rate adds 1.972512
        (1971, 12, 31, 9_889_650),           // A = 4.213170, rate adds 5.676480
    ] {
        // What hifitime's integer accessor reports for the era.
        let e = Epoch::from_gregorian(y, m, d, 0, 0, 0, 0, TimeScale::UTC);
        assert_eq!(
            hifitime_tai_minus_utc(e),
            0,
            "hifitime is expected to report no offset in the rubber era"
        );

        // What this crate computes, against the published table.
        let utc = from_civil(
            y as i64, m, d, 0, 0, 0,
            SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian,
        )
        .unwrap_or_else(|err| panic!("{y}-{m}-{d}: {err}"));
        let tai = from_civil(
            y as i64, m, d, 0, 0, 0,
            SubSecond::zero(), Scale::Tai, CivilCalendar::Gregorian,
        )
        .unwrap();
        let offset = utc.since(&tai).unwrap();

        // published_micros is the offset to six decimal places; it is a whole
        // number of ticks, so the comparison is exact rather than approximate.
        let (want, r) = ucal_core::num::mul_div(
            &<Ticks as TickInt>::from_u64(published_micros as u64),
            &second(),
            &<Ticks as TickInt>::from_u64(1_000_000),
        )
        .unwrap();
        assert!(r.is_zero_ticks(), "the published value must be exact in ticks");
        assert_eq!(
            offset.ticks(),
            &want,
            "TAI - UTC at {y}-{m}-{d} does not match the published table"
        );
    }
}

#[test]
fn the_two_agree_again_from_1972() {
    // The divergence is confined to the era. From 1972-01-01 the integer
    // leap-second table takes over and the implementations agree exactly, which
    // `utc_labels_agree_with_hifitime_after_1972` already checks in bulk; this
    // pins the boundary itself.
    let mine = from_civil(
        1972, 1, 1, 0, 0, 0,
        SubSecond::zero(), Scale::Utc, CivilCalendar::Gregorian,
    )
    .unwrap();
    let theirs = Epoch::from_gregorian(1972, 1, 1, 0, 0, 0, 0, TimeScale::UTC);
    assert_eq!(from_epoch(theirs).unwrap(), mine);
    assert_eq!(hifitime_tai_minus_utc(theirs), 10);
}

#[test]
fn utc_before_1961_diverges_deliberately() {
    // Documented divergence, asserted so it cannot drift silently. UTC did not
    // exist before 1961-01-01; hifitime extrapolates its scale backwards anyway.
    let err = from_civil(
        1955,
        1,
        1,
        0,
        0,
        0,
        SubSecond::zero(),
        Scale::Utc,
        CivilCalendar::Gregorian,
    )
    .unwrap_err();
    assert_eq!(err.code, ucal_core::Code::E0041);

    // The same instant remains reachable in the uniform scales, at any epoch.
    for scale in [Scale::Tt, Scale::Tai] {
        assert!(from_civil(
            1955,
            1,
            1,
            0,
            0,
            0,
            SubSecond::zero(),
            scale,
            CivilCalendar::Gregorian
        )
        .is_ok());
    }
}
