//! §20 UC-P7 exit criterion: **10⁶ random civil instants convert with zero
//! rounding.**
//!
//! This is failure mode F3 — "foreign units inject rounding into the core" — and
//! its metric is exactly this sweep. The claim being tested is §8.2's: converting
//! *into* absolute time is `ORIGIN_OFFSET + s x SECOND`, and for any `s` whose
//! denominator divides `10^30` that product is an exact integer.
//!
//! Two independent checks per sample, because a round trip alone could hide a
//! compensating pair of errors:
//!
//! 1. The tick value equals an independently computed
//!    `ORIGIN_OFFSET ± linear_seconds x SECOND + sub_ticks`.
//! 2. Rendering back at the same digit count returns the identical label with
//!    `lossy == false`.

use ucal_civil::calendar::{civil_from_days, days_from_civil, CivilCalendar};
use ucal_civil::si::{second, to_civil, CIVIL_YEAR_MAX, CIVIL_YEAR_MIN};
use ucal_civil::{from_civil, Scale, SubSecond};
use ucal_core::backend::TickInt;
use ucal_core::{Instant, Profile, Rounding, Ticks, UC1};

/// How many samples. The specification names 10⁶; a debug build is roughly
/// twenty times slower than a release build, so the sweep is sized down there and
/// the full count is asserted in release.
fn sample_count() -> usize {
    if cfg!(debug_assertions) {
        20_000
    } else {
        1_000_000
    }
}

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

fn pow10_ticks(e: u32) -> Ticks {
    let ten = <Ticks as TickInt>::from_u64(10);
    let mut acc = <Ticks as TickInt>::one();
    for _ in 0..e {
        acc = acc.try_mul(&ten).expect("10^30 is far inside the domain");
    }
    acc
}

/// Independently recompute the expected tick value, using only the definition in
/// §8.2 and nothing from the conversion under test.
fn expected_ticks(
    year: i64,
    month: u8,
    day: u8,
    hour: u8,
    minute: u8,
    sec: u8,
    sub: &SubSecond,
) -> Ticks {
    let linear = days_from_civil(year, month, day, CivilCalendar::Gregorian) * 86_400
        + hour as i64 * 3_600
        + minute as i64 * 60
        + sec as i64;
    let origin = UC1::origin_offset();
    let magnitude = <Ticks as TickInt>::from_u64(linear.unsigned_abs())
        .try_mul(&second())
        .expect("within domain");
    let base = if linear < 0 {
        origin.try_sub(&magnitude).expect("after the datum")
    } else {
        origin.try_add(&magnitude).expect("within domain")
    };
    // sub x SECOND / 10^d, computed the long way round: multiply first, then
    // divide, and assert the division is exact.
    if sub.is_zero() {
        return base;
    }
    let num = <Ticks as TickInt>::from_u128(sub.value()).expect("fits");
    let prod = num.try_mul(&second()).expect("within domain");
    let (q, r) = prod.quot_rem(&pow10_ticks(sub.digits() as u32));
    assert!(
        r.is_zero_ticks(),
        "sub-second conversion was not exact: {} / 10^{}",
        sub.value(),
        sub.digits()
    );
    base.try_add(&q).expect("within domain")
}

#[test]
fn a_million_civil_instants_convert_with_zero_rounding() {
    let n = sample_count();
    let mut rng = Rng(0x1234_5678_9ABC_DEF0);

    // Day range: wide enough to cross the Gregorian leap-century rules and the
    // year-zero boundary in both directions, and far inside the civil range.
    const DAY_LO: i64 = -1_500_000; // roughly year -4100
    const DAY_SPAN: u64 = 3_000_000; // through roughly year +4100

    let mut exact_checked = 0usize;
    for _ in 0..n {
        let days = DAY_LO + rng.below(DAY_SPAN) as i64;
        let (year, month, day) = civil_from_days(days, CivilCalendar::Gregorian);
        let hour = rng.below(24) as u8;
        let minute = rng.below(60) as u8;
        let sec = rng.below(60) as u8;

        // A fraction of a random width, up to the thirty places the bridge
        // carries exactly. Width 0 means no fractional part at all.
        let digits = rng.below(SubSecond::MAX_DIGITS as u64 + 1) as u8;
        let sub = if digits == 0 {
            SubSecond::zero()
        } else {
            // Uniform over the whole numerator range for this width.
            let limit = 10u128.pow(digits as u32);
            // Composed from two draws rather than by wrapping arithmetic, so the
            // Rule O lint needs no exemption for a test helper.
            let v = ((rng.next() as u128) << 64) | (rng.next() as u128);
            SubSecond::new(v % limit, digits).expect("width is within thirty")
        };

        let got = from_civil(
            year,
            month,
            day,
            hour,
            minute,
            sec,
            sub,
            Scale::Tt,
            CivilCalendar::Gregorian,
        )
        .unwrap_or_else(|e| {
            panic!("{year}-{month}-{day}T{hour}:{minute}:{sec} +{sub:?} failed: {e}")
        });

        // Check 1: the value matches the independent computation exactly.
        let want = expected_ticks(year, month, day, hour, minute, sec, &sub);
        assert_eq!(
            got.ticks(),
            &want,
            "inexact conversion at {year}-{month}-{day}T{hour}:{minute}:{sec} sub={sub:?}"
        );

        // Check 2: rendering back at the same width is lossless and identical.
        let back = to_civil(
            &got,
            Scale::Tt,
            sub.digits(),
            Rounding::Trunc,
            CivilCalendar::Gregorian,
        )
        .expect("render");
        assert!(
            !back.lossy,
            "rendering at the input's own width reported loss: {year}-{month}-{day} sub={sub:?}"
        );
        assert_eq!(
            (
                back.year,
                back.month,
                back.day,
                back.hour,
                back.minute,
                back.second
            ),
            (year, month, day, hour, minute, sec),
            "label did not round trip"
        );
        assert_eq!(back.sub.value(), sub.value(), "fraction did not round trip");
        exact_checked += 1;
    }

    assert_eq!(exact_checked, n);
}

#[test]
fn every_sub_second_width_is_exact_and_the_next_one_is_not() {
    // The boundary D-3 buys: thirty decimal places are exact, and the
    // thirty-first is `UCAL-E0043` rather than a rounded value.
    for digits in 0..=SubSecond::MAX_DIGITS {
        let value = if digits == 0 {
            0
        } else {
            10u128.pow(digits as u32) - 1 // the widest numerator at this width
        };
        let sub = SubSecond::new(value, digits).unwrap();
        let ticks = sub.ticks().unwrap();
        // ticks x 10^digits == value x SECOND, exactly.
        let lhs = ticks.try_mul(&pow10_ticks(digits as u32)).unwrap();
        let rhs = <Ticks as TickInt>::from_u128(value)
            .unwrap()
            .try_mul(&second())
            .unwrap();
        assert_eq!(lhs, rhs, "width {digits} was not exact");
    }
    assert_eq!(
        SubSecond::new(1, 31).unwrap_err().code,
        ucal_core::Code::E0043
    );
}

#[test]
fn conversion_is_exact_across_the_whole_civil_range() {
    // The sweep above concentrates on a few millennia; these are the extremes,
    // where the day arithmetic and the domain bounds are most likely to break.
    let cases: [(i64, u8, u8); 8] = [
        (CIVIL_YEAR_MIN + 1, 1, 1),
        (-13_000_000_000, 6, 15),
        (-4713, 1, 1),
        (-1, 12, 31),
        (0, 1, 1),
        (1, 1, 1),
        (1_000_000, 2, 29),
        (CIVIL_YEAR_MAX - 1, 12, 31),
    ];
    for (year, month, day) in cases {
        let sub = SubSecond::parse("123456789012345678901234567890").unwrap();
        let got = from_civil(
            year,
            month,
            day,
            12,
            34,
            56,
            sub,
            Scale::Tt,
            CivilCalendar::Gregorian,
        );
        match got {
            Ok(v) => {
                let want = expected_ticks(year, month, day, 12, 34, 56, &sub);
                assert_eq!(v.ticks(), &want, "{year}-{month}-{day}");
                // ...and it is a positive tick count, always.
                assert!(v.ticks() > &<Ticks as TickInt>::zero());
            }
            Err(e) => {
                // The only acceptable refusals are the range guard and the datum
                // floor — never a silent approximation.
                assert!(
                    matches!(e.code, ucal_core::Code::E0040 | ucal_core::Code::E0020),
                    "{year}-{month}-{day} failed with an unexpected code: {e}"
                );
            }
        }
    }
}

#[test]
fn the_datum_is_the_floor_of_the_civil_range() {
    // "No time exists before that." Walking backwards, conversion succeeds until
    // it would cross tick 0, and then fails — it never yields a negative value,
    // because there is no such value to yield.
    let mut last_ok: Option<Instant<UC1>> = None;
    let mut first_err_year = None;
    for gyr in 0..20i64 {
        let year = -(gyr * 1_000_000_000);
        match from_civil(
            year,
            1,
            1,
            0,
            0,
            0,
            SubSecond::zero(),
            Scale::Tt,
            CivilCalendar::Gregorian,
        ) {
            Ok(v) => {
                assert!(v.ticks() > &<Ticks as TickInt>::zero());
                if let Some(prev) = &last_ok {
                    assert!(v < *prev, "walking back must decrease");
                }
                last_ok = Some(v);
            }
            Err(e) => {
                assert_eq!(e.code, ucal_core::Code::E0020, "year {year}");
                first_err_year = Some(year);
                break;
            }
        }
    }
    // The crossing happens near -13.787 Gyr, the implied age of the datum.
    let crossed = first_err_year.expect("the walk must eventually reach the datum");
    assert!(
        (-15_000_000_000..=-13_000_000_000).contains(&crossed),
        "the datum floor appeared at year {crossed}, which is not near -13.787 Gyr"
    );
}
