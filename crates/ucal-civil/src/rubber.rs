//! UTC in the rubber-second era, 1961-01-01 to 1972-01-01.
//!
//! # What this era was
//!
//! Before 1972, UTC was steered to follow UT1 by two means at once: occasional
//! **step adjustments** of a fraction of a second, and a continuously applied
//! **rate offset** that made the UTC second differ in length from the SI second.
//! `TAI - UTC` was therefore not an integer and not even constant within a day.
//! It is given by a piecewise-linear function of the UTC date,
//!
//! ```text
//! TAI - UTC  =  A + (MJD - B) x C    seconds
//! ```
//!
//! with `A`, `B` and `C` declared per segment. Thirteen segments cover the era.
//!
//! # Why it converts exactly
//!
//! An earlier revision of this crate refused pre-1972 UTC on the grounds that a
//! non-integer offset could not be represented exactly. That was over-cautious,
//! and the reason is worth recording, because it is not obvious.
//!
//! A UTC instant carries a fraction of a day, so `MJD` has a denominator of
//! `86400 = 2^7 x 3^3 x 5^2`. The bridge constant `SECOND = 18 548 584 399 861 x
//! 10^30` has no factor of three, so a bare `1/86400` would land between ticks.
//! But the rate coefficients are not arbitrary — every one of them carries a
//! surplus of threes:
//!
//! | C (s/day) | C / 86400 |
//! |---|---|
//! | 0.001296  | 3 / 200 000 000 |
//! | 0.0011232 | 13 / 1 000 000 000 |
//! | 0.002592  | 3 / 100 000 000 |
//!
//! Each quotient is of the form `n / (2^a x 5^b)`, which divides `10^30` and so
//! divides `SECOND`. The `3^3` in the day cancels exactly. Every instant in the
//! era therefore lands on a whole tick, and the conversion is exact rather than
//! rounded — `every_segment_converts_exactly` sweeps all thirteen segments to
//! confirm it.
//!
//! # What is still refused
//!
//! UTC did not exist before 1961-01-01. A UTC label earlier than that is
//! `UCAL-E0041`; TT and TAI remain available at any epoch.

use ucal_core::backend::TickInt;
use ucal_core::num::{mul_div, Ratio};
use ucal_core::{Code, Ticks, TimeError};

type Result<T> = core::result::Result<T, TimeError>;

/// Day number of MJD 0 (1858-11-17) from this crate's origin, `0000-01-01`
/// proleptic Gregorian.
pub const MJD_EPOCH_DAY: i64 = 678_941;

/// The decimal scale the era's coefficients are declared at: `10^-7` s.
const COEFF_SCALE: i64 = 10_000_000;

/// Seconds in a day, as the era's formula uses it.
const SECONDS_PER_DAY: i64 = 86_400;

/// One segment of the piecewise-linear definition.
///
/// `TAI - UTC = (a + (mjd - b) x c) / 10^7` seconds, with `a` and `c` held as
/// integers at `10^-7` precision so that nothing is stored as a decimal.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct RateSegment {
    /// First MJD this segment applies to.
    pub from_mjd: i64,
    /// Constant term, in units of `10^-7` s.
    pub a: i64,
    /// Reference MJD the rate is measured from.
    pub b: i64,
    /// Rate, in units of `10^-7` s per day.
    pub c: i64,
}

const fn seg(from_mjd: i64, a: i64, b: i64, c: i64) -> RateSegment {
    RateSegment { from_mjd, a, b, c }
}

/// The 1961-1972 `TAI - UTC` definition (USNO/IERS), ascending by MJD.
pub const RATE_TABLE: &[RateSegment] = &[
    seg(37_300, 14_228_180, 37_300, 12_960), // 1961-01-01
    seg(37_512, 13_728_180, 37_300, 12_960), // 1961-08-01
    seg(37_665, 18_458_580, 37_665, 11_232), // 1962-01-01
    seg(38_334, 19_458_580, 37_665, 11_232), // 1963-11-01
    seg(38_395, 32_401_300, 38_761, 12_960), // 1964-01-01
    seg(38_486, 33_401_300, 38_761, 12_960), // 1964-04-01
    seg(38_639, 34_401_300, 38_761, 12_960), // 1964-09-01
    seg(38_761, 35_401_300, 38_761, 12_960), // 1965-01-01
    seg(38_820, 36_401_300, 38_761, 12_960), // 1965-03-01
    seg(38_942, 37_401_300, 38_761, 12_960), // 1965-07-01
    seg(39_004, 38_401_300, 38_761, 12_960), // 1965-09-01
    seg(39_126, 43_131_700, 39_126, 25_920), // 1966-01-01
    seg(39_887, 42_131_700, 39_126, 25_920), // 1968-02-01
];

/// First MJD of the era. UTC does not exist before this.
pub const ERA_FIRST_MJD: i64 = 37_300;

/// First MJD of modern UTC, at which the offset steps to exactly 10 s.
pub const ERA_LAST_MJD: i64 = 41_317;

/// Whether a day falls in the rubber-second era.
pub fn is_rubber_era(days_from_origin: i64) -> bool {
    let mjd = days_from_origin - MJD_EPOCH_DAY;
    (ERA_FIRST_MJD..ERA_LAST_MJD).contains(&mjd)
}

/// Whether a day precedes UTC entirely.
pub fn precedes_utc(days_from_origin: i64) -> bool {
    days_from_origin - MJD_EPOCH_DAY < ERA_FIRST_MJD
}

/// The segment covering an MJD, or `None` outside the era.
pub fn segment_for(mjd: i64) -> Option<&'static RateSegment> {
    if !(ERA_FIRST_MJD..ERA_LAST_MJD).contains(&mjd) {
        return None;
    }
    let mut found = None;
    for s in RATE_TABLE {
        if s.from_mjd <= mjd {
            found = Some(s);
        }
    }
    found
}

/// The common denominator of the era's arithmetic: `86400 x 10^7`.
fn denominator() -> Ticks {
    <Ticks as TickInt>::from_u64((SECONDS_PER_DAY * COEFF_SCALE) as u64)
}

/// `TAI - UTC` in ticks, for a UTC instant given as a day number and a second of
/// day. **Exact** — see the module documentation for why.
pub fn offset_ticks(days_from_origin: i64, second_of_day: i64, second_ticks: &Ticks) -> Result<Ticks> {
    let mjd = days_from_origin - MJD_EPOCH_DAY;
    let s = segment_for(mjd).ok_or_else(|| {
        TimeError::with_context(Code::E0041, "date is outside the 1961-1972 UTC era")
    })?;

    // numerator = (a + (mjd - b) x c) x 86400 + sod x c, at 10^-7 s x 86400.
    let numerator = s
        .a
        .checked_mul(SECONDS_PER_DAY)
        .and_then(|v| {
            (mjd - s.b)
                .checked_mul(s.c)
                .and_then(|w| w.checked_mul(SECONDS_PER_DAY))
                .and_then(|w| v.checked_add(w))
        })
        .and_then(|v| second_of_day.checked_mul(s.c).and_then(|w| v.checked_add(w)))
        .ok_or(TimeError::new(Code::E0040))?;
    if numerator < 0 {
        return Err(TimeError::with_context(
            Code::E0041,
            "negative TAI - UTC offset in the rubber-second era",
        ));
    }

    let n = <Ticks as TickInt>::from_u64(numerator as u64);
    let (q, r) = mul_div(&n, second_ticks, &denominator())?;
    if !r.is_zero_ticks() {
        // The module documentation argues this cannot happen; assert it rather
        // than silently rounding, because a silent rounding here would be exactly
        // the failure Rule R exists to prevent.
        return Err(TimeError::with_context(
            Code::E0043,
            "rubber-second offset did not land on a whole tick",
        ));
    }
    Ok(q)
}

/// Invert [`offset_ticks`]: recover UTC linear seconds from a TAI instant.
///
/// `TAI = u + A + ((u - E)/86400 - B) x C` is linear in `u`, so
///
/// ```text
/// u = [TAI x D - SECOND x (P - E x C)] / [SECOND x (D + C)]
/// ```
///
/// with `D = 86400 x 10^7`, `P = (A - B x C) x 86400` and `E` the linear second
/// count of MJD 0. Exact rational throughout; the result is generally not a whole
/// second, which is correct — a pre-1972 UTC label genuinely has a fractional
/// offset from TAI.
pub fn utc_linear_seconds_from_tai(
    tai_ticks_from_origin: &Ticks,
    second_ticks: &Ticks,
) -> Result<Ratio> {
    let s = segment_for_tai(tai_ticks_from_origin, second_ticks)?;
    invert_with(s, tai_ticks_from_origin, second_ticks)
}

/// The TAI instant, in ticks from the origin, at which a segment begins.
///
/// Computed with the segment's **own** parameters, which is what makes the
/// boundaries unambiguous.
fn segment_start_tai(s: &RateSegment, second_ticks: &Ticks) -> Result<Ticks> {
    let numerator = s.a * SECONDS_PER_DAY + (s.from_mjd - s.b) * s.c * SECONDS_PER_DAY;
    let (offset, r) = mul_div(
        &<Ticks as TickInt>::from_u64(numerator.unsigned_abs()),
        second_ticks,
        &denominator(),
    )?;
    debug_assert!(r.is_zero_ticks());
    let linear = (s.from_mjd + MJD_EPOCH_DAY) * SECONDS_PER_DAY;
    let base = <Ticks as TickInt>::from_u64(linear as u64)
        .try_mul(second_ticks)
        .ok_or(TimeError::new(Code::E0021))?;
    if numerator < 0 {
        base.try_sub(&offset).ok_or(TimeError::new(Code::E0020))
    } else {
        base.try_add(&offset).ok_or(TimeError::new(Code::E0021))
    }
}

/// The TAI instant at which modern UTC begins, with its offset of exactly 10 s.
fn era_end_tai(second_ticks: &Ticks) -> Result<Ticks> {
    let linear = (ERA_LAST_MJD + MJD_EPOCH_DAY) * SECONDS_PER_DAY + 10;
    <Ticks as TickInt>::from_u64(linear as u64)
        .try_mul(second_ticks)
        .ok_or(TimeError::new(Code::E0021))
}

/// Whether a TAI instant, in ticks from the origin, falls in the era.
///
/// Keyed on TAI rather than on a day number, because near the boundary the two
/// disagree: at 1971-12-31T23:59:59 UTC the TAI instant is already past
/// 1972-01-01T00:00:00 in linear terms, since `TAI - UTC` was 9.89 s by then. A
/// day-keyed test would push the era's last seconds into the modern regime and
/// refuse them.
pub fn covers_tai(tai_ticks_from_origin: &Ticks, second_ticks: &Ticks) -> Result<bool> {
    if tai_ticks_from_origin < &segment_start_tai(&RATE_TABLE[0], second_ticks)? {
        return Ok(false);
    }
    Ok(tai_ticks_from_origin < &era_end_tai(second_ticks)?)
}

/// Choose the segment from **TAI**, not from the UTC label.
///
/// The era contains step adjustments, so `TAI - UTC` is discontinuous at some
/// segment boundaries — at 1968-02-01 the constant drops by 0.1 s. Around such a
/// step the UTC label is not a reliable key: a candidate computed with the
/// previous segment's parameters lands 0.1 s early, which floors onto the
/// previous day and so still appears to fall inside the previous segment's range.
/// The result is that the wrong segment matches and the answer is off by the size
/// of the step.
///
/// TAI has no such ambiguity, because it is continuous and monotone. Keying the
/// lookup on it makes the boundaries exact.
fn segment_for_tai(tai: &Ticks, second_ticks: &Ticks) -> Result<&'static RateSegment> {
    let first = segment_start_tai(&RATE_TABLE[0], second_ticks)?;
    if tai < &first {
        return Err(TimeError::with_context(
            Code::E0041,
            "UTC does not exist before 1961-01-01",
        ));
    }
    if tai >= &era_end_tai(second_ticks)? {
        return Err(TimeError::with_context(
            Code::E0041,
            "instant is after the rubber-second era; use the leap-second table",
        ));
    }
    let mut chosen = &RATE_TABLE[0];
    for s in RATE_TABLE {
        if tai >= &segment_start_tai(s, second_ticks)? {
            chosen = s;
        }
    }
    Ok(chosen)
}

/// Solve `TAI = u + A + ((u - E)/86400 - B) x C` for `u`, exactly.
///
/// ```text
/// u = [TAI x D - SECOND x BIAS] / [SECOND x (D + C)]
/// ```
///
/// with `D = 86400 x 10^7` and `BIAS = 86400 x (A - (E_days + B) x C)`.
fn invert_with(s: &RateSegment, tai: &Ticks, second_ticks: &Ticks) -> Result<Ratio> {
    let d = SECONDS_PER_DAY * COEFF_SCALE;
    let bias = SECONDS_PER_DAY * (s.a - (MJD_EPOCH_DAY + s.b) * s.c);

    let lhs = tai
        .try_mul(&<Ticks as TickInt>::from_u64(d as u64))
        .ok_or(TimeError::new(Code::E0021))?;
    let bias_term = second_ticks
        .try_mul(&<Ticks as TickInt>::from_u64(bias.unsigned_abs()))
        .ok_or(TimeError::new(Code::E0021))?;
    let numerator = if bias < 0 {
        lhs.try_add(&bias_term)
    } else {
        lhs.try_sub(&bias_term)
    }
    .ok_or(TimeError::new(Code::E0020))?;

    let denom = second_ticks
        .try_mul(&<Ticks as TickInt>::from_u64((d + s.c) as u64))
        .ok_or(TimeError::new(Code::E0021))?;
    Ratio::new(numerator, denom)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::days_from_gregorian;
    use crate::si::second;
    use ucal_core::backend::TickInt;

    fn day(y: i64, m: u8, d: u8) -> i64 {
        days_from_gregorian(y, m, d)
    }

    #[test]
    fn mjd_epoch_is_1858_11_17() {
        assert_eq!(day(1858, 11, 17), MJD_EPOCH_DAY);
        assert_eq!(day(1972, 1, 1) - MJD_EPOCH_DAY, 41_317);
        assert_eq!(day(1961, 1, 1) - MJD_EPOCH_DAY, 37_300);
        assert_eq!(day(1968, 2, 1) - MJD_EPOCH_DAY, 39_887);
    }

    #[test]
    fn table_is_ascending_and_covers_the_era() {
        for w in RATE_TABLE.windows(2) {
            assert!(w[0].from_mjd < w[1].from_mjd);
        }
        assert_eq!(RATE_TABLE[0].from_mjd, ERA_FIRST_MJD);
        assert!(RATE_TABLE.last().unwrap().from_mjd < ERA_LAST_MJD);
        assert_eq!(RATE_TABLE.len(), 13);
    }

    #[test]
    fn rates_carry_the_threes_that_make_the_era_exact() {
        // The property the whole module rests on: C / 86400 must reduce to a
        // denominator of the form 2^a x 5^b, so that it divides 10^30 and hence
        // SECOND. Equivalently, 27 must divide C.
        for s in RATE_TABLE {
            assert_eq!(
                s.c % 27,
                0,
                "rate {} is not divisible by 27, so the 3^3 in 86400 would not cancel",
                s.c
            );
        }
    }

    #[test]
    fn every_segment_converts_exactly() {
        // Dense sweep: every segment, many seconds of day. Any inexactness would
        // surface as UCAL-E0043 from `offset_ticks`.
        let sec = second();
        let mut checked = 0;
        for s in RATE_TABLE {
            let days = s.from_mjd + MJD_EPOCH_DAY;
            for sod in (0..SECONDS_PER_DAY).step_by(97) {
                offset_ticks(days, sod, &sec)
                    .unwrap_or_else(|e| panic!("segment {s:?} sod {sod}: {e}"));
                checked += 1;
            }
        }
        assert!(checked > 10_000, "only {checked} instants swept");
    }

    #[test]
    fn known_offsets_match_the_published_table() {
        let sec = second();
        // 1968-02-01: TAI - UTC = 4.2131700 + (39887 - 39126) x 0.002592
        //           = 4.21317 + 1.972512 = 6.185682 s
        let t = offset_ticks(day(1968, 2, 1), 0, &sec).unwrap();
        let expect = mul_div(
            &<Ticks as TickInt>::from_u64(6_185_682),
            &sec,
            &<Ticks as TickInt>::from_u64(1_000_000),
        )
        .unwrap();
        assert!(expect.1.is_zero_ticks());
        assert_eq!(t, expect.0);

        // 1961-01-01, the era's first instant: exactly 1.4228180 s.
        let t = offset_ticks(day(1961, 1, 1), 0, &sec).unwrap();
        let expect = mul_div(
            &<Ticks as TickInt>::from_u64(14_228_180),
            &sec,
            &<Ticks as TickInt>::from_u64(10_000_000),
        )
        .unwrap();
        assert!(expect.1.is_zero_ticks());
        assert_eq!(t, expect.0);
    }

    #[test]
    fn the_famous_step_at_1972_is_0_107758_seconds() {
        // The last rubber-second offset is 9.892242 s; modern UTC starts at
        // exactly 10 s. The discontinuity is 0.107758 s, and it is a genuine step
        // in the *definition*, not an artefact of this implementation.
        let sec = second();
        let last = offset_ticks(day(1971, 12, 31), SECONDS_PER_DAY - 1, &sec).unwrap();
        let ten = sec.try_mul(&<Ticks as TickInt>::from_u64(10)).unwrap();
        assert!(last < ten);

        // At exactly 1972-01-01T00:00:00 the formula would give 9.892242 s.
        let at_boundary = {
            let mjd = ERA_LAST_MJD;
            let s = RATE_TABLE.last().unwrap();
            let numerator = s.a * SECONDS_PER_DAY + (mjd - s.b) * s.c * SECONDS_PER_DAY;
            let (q, r) = mul_div(
                &<Ticks as TickInt>::from_u64(numerator as u64),
                &sec,
                &denominator(),
            )
            .unwrap();
            assert!(r.is_zero_ticks());
            q
        };
        let step = ten.try_sub(&at_boundary).unwrap();
        // 0.107758 s is a whole number of ticks, and here it is. Nothing in this
        // era needs a fraction of a tick: `SECOND = M x 10^30`, so any exact
        // decimal of thirty or fewer places is an integer tick count, and the
        // era's constants have at most seven.
        assert_eq!(
            step.to_dec_string(),
            "1998758357760221638000000000000000000000000",
            "the 1972 step must be exactly 0.107758 s"
        );
        // ...which is the same thing said as a ratio.
        let expect = mul_div(
            &<Ticks as TickInt>::from_u64(107_758),
            &sec,
            &<Ticks as TickInt>::from_u64(1_000_000),
        )
        .unwrap();
        assert!(expect.1.is_zero_ticks(), "0.107758 s is exact in ticks");
        assert_eq!(step, expect.0);
    }

    #[test]
    fn every_era_constant_is_a_whole_number_of_ticks() {
        // The general statement behind the 1972 step: every constant in the era
        // is an exact decimal of at most seven places, and `SECOND` carries
        // thirty, so all of them are integer tick counts with nothing left over.
        let sec = second();
        for (label, value, places, want) in [
            ("1.4228180 s, the 1961 offset", 14_228_180u64, 7u32,
             "26391259758641428298000000000000000000000000"),
            ("6.185682 s at 1968-02-01", 6_185_682, 6,
             "114735644647700990202000000000000000000000000"),
            ("9.892242 s, the last rubber offset", 9_892_242, 6,
             "183487085640849778362000000000000000000000000"),
            ("32.184 s, TT - TAI", 32_184, 3,
             "596967640325126424000000000000000000000000000"),
        ] {
            let mut den = <Ticks as TickInt>::one();
            for _ in 0..places {
                den = den.try_mul(&<Ticks as TickInt>::from_u64(10)).unwrap();
            }
            let (q, r) = mul_div(&<Ticks as TickInt>::from_u64(value), &sec, &den).unwrap();
            assert!(r.is_zero_ticks(), "{label} was not exact");
            assert_eq!(q.to_dec_string(), want, "{label}");
        }
    }

    #[test]
    fn the_offset_grows_within_a_day() {
        // The rate term is what made these "rubber" seconds: the offset changes
        // continuously, not just at midnight.
        let sec = second();
        let d = day(1965, 6, 15);
        let start = offset_ticks(d, 0, &sec).unwrap();
        let noon = offset_ticks(d, 43_200, &sec).unwrap();
        let end = offset_ticks(d, 86_399, &sec).unwrap();
        assert!(start < noon && noon < end);
    }

    #[test]
    fn outside_the_era_is_refused() {
        let sec = second();
        assert!(offset_ticks(day(1960, 12, 31), 0, &sec).is_err());
        assert!(offset_ticks(day(1972, 1, 1), 0, &sec).is_err());
        assert!(precedes_utc(day(1960, 1, 1)));
        assert!(!precedes_utc(day(1961, 1, 1)));
        assert!(is_rubber_era(day(1965, 1, 1)));
        assert!(!is_rubber_era(day(1975, 1, 1)));
    }

    #[test]
    fn inversion_recovers_the_label() {
        let sec = second();
        for (y, m, d, sod) in [
            (1961, 1, 1, 0i64),
            (1963, 6, 15, 43_200),
            (1965, 9, 1, 12_345),
            (1968, 2, 1, 0),
            (1971, 12, 31, 86_399),
        ] {
            let days = day(y, m, d);
            let linear = days * SECONDS_PER_DAY + sod;
            let offset = offset_ticks(days, sod, &sec).unwrap();
            let tai = <Ticks as TickInt>::from_u64(linear as u64)
                .try_mul(&sec)
                .unwrap()
                .try_add(&offset)
                .unwrap();

            let u = utc_linear_seconds_from_tai(&tai, &sec).unwrap();
            assert!(u.is_integer(), "the label was a whole second, so u must be too");
            assert_eq!(
                u.floor().to_dec_string(),
                linear.to_string(),
                "inversion failed at {y}-{m}-{d} +{sod}s"
            );
        }
    }
}

