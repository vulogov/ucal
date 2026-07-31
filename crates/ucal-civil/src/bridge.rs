//! Interoperation with `hifitime` (§14).
//!
//! # Why the exact path is ours and not hifitime's
//!
//! §8.1 names `hifitime` for civil time, and it is the right library for the job
//! it does. But two of its properties make it unsuitable as the *exact* path:
//!
//! - Its resolution is nanoseconds. The bridge carries thirty decimal places
//!   (D-3), which is `10^21` times finer, and a tick is finer still.
//! - Most of its accessors return `f64` — `to_seconds`, `to_tai_days`,
//!   `leap_seconds(..) -> Option<f64>`. Rule E forbids a float token anywhere in
//!   a shipped crate, so none of those may be called.
//!
//! This module therefore uses only hifitime's integer surface — `to_gregorian`,
//! `from_gregorian`, and `leap_seconds_iers() -> i32` — and confines the
//! interchange to nanosecond precision, which is what an `Epoch` can represent.
//! Converting *out* of absolute time rounds and says so (Rule R); converting *in*
//! is exact, because a whole number of nanoseconds is exactly representable
//! (§2.4 guarantees 21 trailing base-5 zeros for one).
//!
//! Its real value is as an **independent oracle**: §21.2 requires a differential
//! test against it, and `tests/differential_hifitime.rs` is that test.

use hifitime::{Epoch, TimeScale};
use ucal_core::{Code, Instant, Rounding, TimeError, UC1};

use crate::calendar::CivilCalendar;
use crate::si::{from_civil, to_civil, Scale, SubSecond};

type Result<T> = core::result::Result<T, TimeError>;

/// Nanoseconds are hifitime's resolution, and nine decimal places is exactly
/// representable by the bridge.
const NANOSECOND_DIGITS: u8 = 9;

/// Map a `hifitime::Epoch` to absolute time. **Exact**: an `Epoch` carries whole
/// nanoseconds, and the bridge represents those exactly.
///
/// The TT scale is used for the handover because Rule L makes TT the pivot —
/// going through UTC would drag leap seconds into a conversion that has no
/// business consulting them.
pub fn from_epoch(e: Epoch) -> Result<Instant<UC1>> {
    let (year, month, day, hour, minute, sec, nanos) = e.to_gregorian(TimeScale::TT);
    from_civil(
        year as i64,
        month,
        day,
        hour,
        minute,
        sec,
        SubSecond::new(nanos as u128, NANOSECOND_DIGITS)?,
        Scale::Tt,
        CivilCalendar::Gregorian,
    )
}

/// Map absolute time to a `hifitime::Epoch`, rounding to nanoseconds under an
/// explicit mode (§14, Rule R).
///
/// Lossy by construction: one nanosecond is about 1.855×10¹³ ticks, so all but a
/// vanishing fraction of instants land between two representable epochs. The
/// rounding mode is a required argument for that reason.
pub fn to_epoch(t: &Instant<UC1>, rounding: Rounding) -> Result<(Epoch, bool)> {
    let f = to_civil(
        t,
        Scale::Tt,
        NANOSECOND_DIGITS,
        rounding,
        CivilCalendar::Gregorian,
    )?;
    let year = i32::try_from(f.year).map_err(|_| {
        TimeError::with_context(Code::E0040, "year outside hifitime's representable range")
    })?;
    let nanos = u32::try_from(f.sub.value())
        .map_err(|_| TimeError::with_context(Code::E0040, "nanosecond field overflowed"))?;
    let e = Epoch::from_gregorian(
        year,
        f.month,
        f.day,
        f.hour,
        f.minute,
        f.second,
        nanos,
        TimeScale::TT,
    );
    Ok((e, f.lossy))
}

/// `TAI - UTC` at an epoch, as hifitime reports it.
///
/// Uses the integer accessor deliberately: `leap_seconds(..)` returns
/// `Option<f64>` and is unusable under Rule E.
pub fn hifitime_tai_minus_utc(e: Epoch) -> i32 {
    e.leap_seconds_iers()
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::backend::TickInt;
    use ucal_core::Ticks;

    #[test]
    fn epoch_round_trips_at_nanosecond_resolution() {
        let e = Epoch::from_gregorian(2026, 7, 29, 12, 34, 56, 123_456_789, TimeScale::TT);
        let t = from_epoch(e).unwrap();
        let (back, lossy) = to_epoch(&t, Rounding::HalfEven).unwrap();
        assert!(!lossy, "a whole nanosecond is exactly representable");
        assert_eq!(back.to_gregorian(TimeScale::TT), e.to_gregorian(TimeScale::TT));
    }

    #[test]
    fn sub_nanosecond_detail_is_reported_as_lossy() {
        let e = Epoch::from_gregorian(2026, 7, 29, 0, 0, 0, 0, TimeScale::TT);
        let t = from_epoch(e)
            .unwrap()
            .checked_add(&ucal_core::Delta::one_tick())
            .unwrap();
        let (_, lossy) = to_epoch(&t, Rounding::Trunc).unwrap();
        assert!(lossy, "one tick is far below nanosecond resolution");
    }

    #[test]
    fn a_nanosecond_is_the_expected_number_of_ticks() {
        // Cross-check the handover unit: 1 ns = SECOND / 10^9.
        let a = Epoch::from_gregorian(2026, 7, 29, 0, 0, 0, 0, TimeScale::TT);
        let b = Epoch::from_gregorian(2026, 7, 29, 0, 0, 0, 1, TimeScale::TT);
        let ta = from_epoch(a).unwrap();
        let tb = from_epoch(b).unwrap();
        assert_eq!(tb.since(&ta).unwrap().ticks(), &crate::si::nanosecond());
        // ...which is 18 548 584 399 861 x 10^21 ticks.
        assert_eq!(
            crate::si::nanosecond(),
            <Ticks as TickInt>::from_dec_str("18548584399861000000000000000000000").unwrap()
        );
    }
}
