//! A1 — Julian Date, with the time scale required and no default.
//!
//! # Why this exists
//!
//! **Every cited parameter in this project is indexed by JD in its source**, and
//! J2000.0 — the epoch the entire body layer hangs on — *is* JD 2451545.0 TT.
//! Until now there was no way to convert one, so checking a shipped figure
//! against the paper it came from was done by hand.
//!
//! [`S1`] records the wider reason. Astronomy's time bookkeeping is dominated by
//! two error classes and neither is physics; **scale confusion is the first**.
//! `TT − TAI` is 32.184 s exactly, `TT − UTC` is 69.184 s today and changes
//! without warning, and papers publish `JD` and `BJD` without saying which scale
//! they are in. A 69-second error is invisible in a light curve and fatal in a
//! pulsar residual.
//!
//! # The scale is mandatory
//!
//! There is no default and there will not be one. A converter that defaults is a
//! converter that is silently wrong 69 seconds of the time, and the entire value
//! of this function is that the scale becomes something the caller had to type.
//! Rule C's habit — a value without its provenance is not a value — applied to
//! the time scale.
//!
//! # The answer is a window
//!
//! `TT` and `TAI` convert **exactly**, so their window has zero width. `TDB`
//! does not: `TDB − TT` is a periodic series whose evaluation is floating point,
//! and Rule E forbids one here. Rather than ignoring the difference or importing
//! a series, the conversion reports a window of the series' **bound** — and says
//! so. A great many analyses need only that bound; the ones that need more
//! should be using an ephemeris, which [`S1`] records that this project must not
//! become.
//!
//! Rule U: the window is the value. A zero-width window and a ±1.7 ms one are
//! the same kind of answer, differing in what is known.
//!
//! # What is refused, and why
//!
//! **`UTC`.** A Julian Date is a count of days of 86400 SI seconds. A UTC day
//! containing a leap second has 86401, so `JD(UTC)` is not a uniform day count
//! and a fraction inside such a day names no single instant. The refusal points
//! at [`crate::legacy`], which converts UTC calendar times correctly because it
//! has the leap table — which is the honest route rather than an approximation
//! wearing a JD's clothes.
//!
//! **`UT1`.** It needs ΔUT1, which is an observed Earth-orientation quantity
//! published weekly by the IERS and is not in this repository. Rule C forbids
//! using a parameter with no citation, and there is no offline value to cite.
//!
//! [`S1`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/S1-astrophysics-roadmap.md

use ucal_core::backend::TickInt;
use ucal_core::num::Ratio;
use ucal_core::{Code, Delta, Instant, Profile, Ticks, TimeError, Window, UC1};

use crate::si::{second, tt_minus_tai};

type Result<T> = core::result::Result<T, TimeError>;

/// The scale a Julian Date is stated in.
///
/// Deliberately **not** [`crate::si::Scale`]. That one describes a civil
/// calendar label and carries `Utc`, which a Julian Date cannot be in; this one
/// carries `Tdb`, which a civil label is never written in. Two vocabularies that
/// overlap without being equal, kept apart so that neither has a variant its
/// callers must remember is meaningless.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum JdScale {
    /// Terrestrial Time. The pivot, and what an ephemeris epoch is normally in.
    Tt,
    /// International Atomic Time. `TT = TAI + 32.184 s`, exactly.
    Tai,
    /// Barycentric Dynamical Time. Differs from `TT` by a bounded periodic
    /// series that is not evaluated here — see [`TDB_BOUND_NANOS`].
    Tdb,
}

impl JdScale {
    /// The name the CLI and the file format use.
    pub const fn key(self) -> &'static str {
        match self {
            JdScale::Tt => "tt",
            JdScale::Tai => "tai",
            JdScale::Tdb => "tdb",
        }
    }

    /// Parse a scale, refusing the two that have reasons rather than values.
    pub fn parse(s: &str) -> Result<JdScale> {
        match s {
            "tt" | "TT" => Ok(JdScale::Tt),
            "tai" | "TAI" => Ok(JdScale::Tai),
            "tdb" | "TDB" => Ok(JdScale::Tdb),
            "utc" | "UTC" => Err(TimeError::with_context(
                Code::E0016,
                "a Julian Date counts days of 86400 SI seconds, and a UTC day \
                 containing a leap second has 86401 — so `JD(UTC)` is not a \
                 uniform day count and a fraction inside such a day names no \
                 single instant. Use `ucal from-civil <label> --scale utc`, \
                 which has the leap table and converts exactly",
            )),
            "ut1" | "UT1" => Err(TimeError::with_context(
                Code::E0016,
                "UT1 needs ΔUT1, an observed Earth-orientation quantity the \
                 IERS publishes weekly and this repository does not carry. Rule \
                 C forbids a parameter with no citation, and there is no \
                 offline value to cite",
            )),
            _ => Err(TimeError::with_context(
                Code::E0016,
                "no such time scale. `tt`, `tai` and `tdb` convert; `utc` and \
                 `ut1` are refused with reasons",
            )),
        }
    }
}

/// The bound on `|TDB − TT|`, in nanoseconds.
///
/// The periodic difference has an amplitude of about 1.6568 ms; 1.7 ms is the
/// rounded-up bound quoted in IAU 2006 Resolution B3's explanatory material and
/// in the standard references. **Rounded outward**, because a bound rounded
/// inward is not a bound.
///
/// This is the one number here that is an *envelope* rather than a value, and it
/// is why `from_jd` returns a window: evaluating the series would need floating
/// point, and reporting its centre without its width would be claiming a
/// precision this crate does not have.
pub const TDB_BOUND_NANOS: u64 = 1_700_000;

/// `JD` at J2000.0, in TT. The defining epoch.
pub const J2000_JD: u64 = 2_451_545;

/// `JD − MJD`, exactly `2400000.5`.
pub const MJD_OFFSET_TIMES_TWO: u64 = 4_800_001;

/// Ticks in a Julian day of 86400 SI seconds. Exact by construction.
fn julian_day() -> Result<Ticks> {
    second()
        .try_mul(&<Ticks as TickInt>::from_u64(86_400))
        .ok_or(TimeError::new(Code::E0021))
}

/// J2000.0 as absolute time, **derived rather than pasted**.
///
/// `2000-01-01T12:00:00 TT`, through the same civil bridge every other date
/// takes. A hard-coded 61-digit literal for a value the code can compute is a
/// copy, and this project has now twice found that a committed copy of a
/// derivable thing drifts — the schema, and the book's diagnostics appendix.
pub fn j2000() -> Result<Instant<UC1>> {
    crate::si::from_civil(
        2000,
        1,
        1,
        12,
        0,
        0,
        crate::si::SubSecond::zero(),
        crate::si::Scale::Tt,
        crate::calendar::CivilCalendar::Gregorian,
    )
}

/// A Julian Date, in a named scale, as absolute time.
///
/// Exact for `Tt` and `Tai` — the window has zero width. For `Tdb` the window
/// carries [`TDB_BOUND_NANOS`] on each side, because the difference from `TT` is
/// a series this crate does not evaluate.
///
/// `UCAL-E0043` rather than a rounding if the date carries more decimal places
/// than a tick can express, which needs about 34 of them.
pub fn from_jd(jd: &Ratio, scale: JdScale) -> Result<Window<UC1>> {
    let day = julian_day()?;
    let epoch = Ratio::from_u64(J2000_JD);

    // `Ratio` is unsigned (Rule B), so the direction is a branch rather than a
    // sign — the same shape `add_years` takes in `ucal-body`.
    let forward = jd.cmp_exact(&epoch) != core::cmp::Ordering::Less;
    let magnitude = if forward {
        jd.sub(&epoch)?
    } else {
        epoch.sub(jd)?
    };
    let offset = magnitude.mul(&Ratio::from_int(day))?;
    if !offset.is_integer() {
        return Err(TimeError::with_context(
            Code::E0043,
            "that Julian Date is not a whole number of ticks. A tick is \
             5.39e-44 s, so this needs about 34 decimal places to happen — and \
             rounding it would move the answer, which Rule R permits only when \
             rendering",
        ));
    }
    let offset = offset.floor();

    let base = j2000()?;
    let mut ticks = if forward {
        base.ticks()
            .try_add(&offset)
            .ok_or(TimeError::with_context(Code::E0021, "that is past the domain ceiling"))?
    } else {
        base.ticks().try_sub(&offset).ok_or(TimeError::with_context(
            Code::E0020,
            "that lands before the datum. Absolute time is unsigned (Rule B), \
             and a Julian Date that far back precedes tick 0",
        ))?
    };

    // TAI reads 32.184 s behind TT at the same instant, so a date *stated* in
    // TAI names an instant that much later than the same numeral in TT.
    if scale == JdScale::Tai {
        ticks = ticks
            .try_add(&tt_minus_tai())
            .ok_or(TimeError::new(Code::E0021))?;
    }

    let t = Instant::<UC1>::from_ticks(ticks)?;
    match scale {
        JdScale::Tt | JdScale::Tai => Ok(Window::exact(t)),
        JdScale::Tdb => {
            let half = Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .quot_rem(&<Ticks as TickInt>::from_u64(1_000_000_000))
                    .0
                    .try_mul(&<Ticks as TickInt>::from_u64(TDB_BOUND_NANOS))
                    .ok_or(TimeError::new(Code::E0021))?,
            );
            let (w, _clamped) = Window::exact(t).widen(&half)?;
            Ok(w)
        }
    }
}

/// Absolute time as a Julian Date in a named scale. Exact, as a rational.
///
/// The inverse of [`from_jd`], and exact in both directions: a tick count is an
/// integer and a Julian day is an integer number of ticks, so the quotient is a
/// rational and nothing is rounded until it is rendered (Rule R).
///
/// For `Tdb` the returned value is the **TT** Julian Date; the ±1.7 ms envelope
/// is reported by the caller rather than folded into a number that would then
/// look exact. A rational carrying a bound it cannot express is worse than a
/// rational beside one that says what it is.
pub fn to_jd(t: &Instant<UC1>, scale: JdScale) -> Result<Ratio> {
    let day = julian_day()?;
    let base = j2000()?;

    let mut ticks = t.ticks().clone();
    if scale == JdScale::Tai {
        ticks = ticks.try_sub(&tt_minus_tai()).ok_or(TimeError::with_context(
            Code::E0020,
            "that instant is within 32.184 s of the datum, so its TAI reading \
             would precede tick 0",
        ))?;
    }

    let epoch = Ratio::from_u64(J2000_JD);
    let forward = ticks >= *base.ticks();
    let magnitude = if forward {
        ticks.try_sub(base.ticks())
    } else {
        base.ticks().try_sub(&ticks)
    }
    .ok_or(TimeError::new(Code::E0021))?;
    let days = Ratio::new(magnitude, day)?;
    if forward {
        epoch.add(&days)
    } else {
        epoch.sub(&days)
    }
}

/// `MJD = JD − 2400000.5`, exactly.
pub fn jd_to_mjd(jd: &Ratio) -> Result<Ratio> {
    jd.sub(&half_offset()?)
}

/// `JD = MJD + 2400000.5`, exactly.
pub fn mjd_to_jd(mjd: &Ratio) -> Result<Ratio> {
    mjd.add(&half_offset()?)
}

fn half_offset() -> Result<Ratio> {
    Ratio::new(
        <Ticks as TickInt>::from_u64(MJD_OFFSET_TIMES_TWO),
        <Ticks as TickInt>::from_u64(2),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The epoch every shipped parameter is stated at, both ways.
    ///
    /// This is the test that makes the whole module worth having: J2000.0 is
    /// **JD 2451545.0 TT** by definition, and the instant this project has been
    /// carrying as a 61-digit literal since 0.9.0 is the same one.
    #[test]
    fn j2000_is_jd_2451545_tt() {
        let w = from_jd(&Ratio::from_u64(J2000_JD), JdScale::Tt).expect("exact");
        assert_eq!(w.lo().ticks(), w.hi().ticks(), "TT converts exactly");
        assert_eq!(w.lo().ticks(), j2000().expect("epoch").ticks());

        let back = to_jd(&j2000().expect("epoch"), JdScale::Tt).expect("exact");
        assert_eq!(back.cmp_exact(&Ratio::from_u64(J2000_JD)), core::cmp::Ordering::Equal);
    }

    /// Half a day later is noon-to-midnight, exactly.
    #[test]
    fn a_half_day_is_half_a_day() {
        let noon = from_jd(&Ratio::from_u64(J2000_JD), JdScale::Tt).expect("exact");
        let midnight = from_jd(
            &Ratio::from_decimal_str("2451545.5").expect("a date"),
            JdScale::Tt,
        )
        .expect("exact");
        let span = midnight
            .lo()
            .ticks()
            .clone()
            .try_sub(noon.lo().ticks())
            .expect("later");
        let half_day = julian_day().expect("a day").quot_rem(&<Ticks as TickInt>::from_u64(2)).0;
        assert_eq!(span, half_day);
    }

    /// The same numeral in TAI names an instant 32.184 s later than in TT.
    ///
    /// The direction is the part that is easy to get backwards, and getting it
    /// backwards is a 64.368-second error that no test of magnitude would catch.
    #[test]
    fn tai_and_tt_differ_by_exactly_the_offset() {
        let tt = from_jd(&Ratio::from_u64(J2000_JD), JdScale::Tt).expect("exact");
        let tai = from_jd(&Ratio::from_u64(J2000_JD), JdScale::Tai).expect("exact");
        let diff = tai
            .lo()
            .ticks()
            .clone()
            .try_sub(tt.lo().ticks())
            .expect("TAI's numeral is later");
        assert_eq!(diff, tt_minus_tai());
    }

    /// TDB is a window, and the others are not.
    #[test]
    fn tdb_reports_the_bound_it_does_not_evaluate() {
        let w = from_jd(&Ratio::from_u64(J2000_JD), JdScale::Tdb).expect("bounded");
        assert!(w.lo().ticks() < w.hi().ticks(), "TDB is not exact here");
        // 3.4 ms across, being 1.7 on each side.
        let width = w.width().ticks().clone();
        let ms = second().quot_rem(&<Ticks as TickInt>::from_u64(1_000)).0;
        let expect = ms
            .try_mul(&<Ticks as TickInt>::from_u64(34))
            .and_then(|v| v.quot_rem(&<Ticks as TickInt>::from_u64(10)).0.try_add(&<Ticks as TickInt>::zero()))
            .expect("in range");
        assert_eq!(width, expect);
    }

    /// The literal `body_file.rs` used to carry is the instant this derives.
    ///
    /// Kept after the literal was removed, because *that* is the check: the
    /// value was correct, and a pasted copy of a derivable thing is one edit
    /// away from not being. If this ever fails, either the civil bridge moved
    /// or J2000.0 did, and both are worth stopping for.
    #[test]
    fn the_literal_j2000_carried_since_0_9_0_is_this_one() {
        assert_eq!(
            j2000().expect("epoch").ticks().to_dec_string(),
            "8070205173569972963515184424835637180530466139316558837890625"
        );
    }

    /// The scales that are refused are refused with reasons, not silently.
    #[test]
    fn utc_and_ut1_are_refused_and_say_why() {
        let utc = JdScale::parse("utc").expect_err("a UTC day is not 86400 s");
        assert_eq!(utc.code, Code::E0016);
        assert!(format!("{utc}").contains("86401"), "{utc}");
        let ut1 = JdScale::parse("ut1").expect_err("no ΔUT1 here");
        assert!(format!("{ut1}").contains("IERS"), "{ut1}");
    }

    /// MJD is exactly `JD − 2400000.5`.
    #[test]
    fn mjd_is_the_offset_and_nothing_else() {
        let jd = Ratio::from_decimal_str("2451545.0").expect("a date");
        let mjd = jd_to_mjd(&jd).expect("exact");
        assert_eq!(
            mjd.cmp_exact(&Ratio::from_decimal_str("51544.5").expect("mjd")),
            core::cmp::Ordering::Equal
        );
        assert_eq!(
            mjd_to_jd(&mjd).expect("back").cmp_exact(&jd),
            core::cmp::Ordering::Equal
        );
    }

    /// A date needing more precision than a tick is refused, never rounded.
    #[test]
    fn a_date_finer_than_a_tick_is_refused() {
        // 40 decimal places: below 5.39e-44 s only past ~34, so this is inside
        // the range where the product stops being a whole number of ticks.
        let s = format!("2451545.{}", "1".repeat(40));
        let jd = Ratio::from_decimal_str(&s).expect("a date");
        let e = from_jd(&jd, JdScale::Tt).expect_err("finer than a tick");
        assert_eq!(e.code, Code::E0043);
    }
}
