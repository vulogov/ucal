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
    /// **Geocentric Coordinate Time.** `TCG − TT` is a *defining* linear rate,
    /// so this converts **exactly** — unlike `TDB`, whose difference from `TT`
    /// is a series. IAU 2000 Resolution B1.9.
    Tcg,
    /// **Barycentric Coordinate Time.** `TCB − TDB` is likewise a defining
    /// linear rate (IAU 2006 Resolution B3), so *that* step is exact; the
    /// remaining `TDB − TT` is the same bounded series, so `TCB` carries
    /// exactly `TDB`'s ±1.7 ms and not a tick more.
    ///
    /// It runs ahead of `TDB` by **0.489 s per Julian year**, which reaches a
    /// minute inside two centuries. Confusing the two is not a rounding.
    Tcb,
}

impl JdScale {
    /// The name the CLI and the file format use.
    pub const fn key(self) -> &'static str {
        match self {
            JdScale::Tt => "tt",
            JdScale::Tai => "tai",
            JdScale::Tdb => "tdb",
            JdScale::Tcg => "tcg",
            JdScale::Tcb => "tcb",
        }
    }

    /// Parse a scale, refusing the two that have reasons rather than values.
    pub fn parse(s: &str) -> Result<JdScale> {
        match s {
            "tt" | "TT" => Ok(JdScale::Tt),
            "tai" | "TAI" => Ok(JdScale::Tai),
            "tdb" | "TDB" => Ok(JdScale::Tdb),
            "tcg" | "TCG" => Ok(JdScale::Tcg),
            "tcb" | "TCB" => Ok(JdScale::Tcb),
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
                "no such time scale. `tt`, `tai`, `tcg`, `tcb` and `tdb` \
                 convert; `utc` and `ut1` are refused with reasons",
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

/// **E1 — the coordinate time scales, which are exactly defined.**
///
/// `L_G` and `L_B` are *defining* constants rather than measurements: a decision
/// the IAU took, not a quantity anybody measured. Both are terminating decimals,
/// so both are exact rationals, so `TT ↔ TCG` and `TDB ↔ TCB` are **exact linear
/// conversions**. This crate is exactly right about them where it can only be
/// bounded about `TDB`.
///
/// The difference is not small. `TCB` runs ahead of `TDB` by 0.489 s per Julian
/// year and `TCG` ahead of `TT` by 0.022 s — half a second a year, in a field
/// whose residuals are microseconds.
///
/// `L_G = 6.969290134 × 10⁻¹⁰`, IAU 2000 Resolution B1.9.
const L_G: (u64, u64) = (6_969_290_134, 10_000_000_000_000_000_000);

/// `L_B = 1.550519768 × 10⁻⁸`, IAU 2006 Resolution B3.
const L_B: (u64, u64) = (1_550_519_768, 100_000_000_000_000_000);

/// `TDB_0 = −6.55 × 10⁻⁵ s`, the constant offset in the same resolution.
///
/// Negative, and `Ticks` is unsigned (Rule B), so the sign lives in the code
/// that applies it rather than in the constant.
const TDB_0_NEG: (u64, u64) = (655, 10_000_000);

/// The epoch both rates run from: 1977 January 1, 0h TAI.
///
/// `JD 2443144.5003725 TT` — and the fraction is exact, because `TT − TAI` is
/// 32.184 s and `32.184/86400 = 149/400000` terminates.
const T0_JD_NUM: u64 = 24_431_445_003_725;
const T0_JD_DEN: u64 = 10_000_000;

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

/// **V2 — the Besselian epoch's two constants.**
///
/// `B = 1900.0 + (JD − 2415020.31352) / 365.242198781`
///
/// The origin is `JD 2415020.31352` and the year is the *tropical* year at 1900,
/// `365.242198781` days — **not** the Julian year of exactly 365.25 that a
/// `J` epoch counts. Both are terminating decimals, so both are exact rationals
/// and the conversion loses nothing.
const BESSELIAN_ORIGIN: (u64, u64) = (241_502_031_352, 100_000);
const BESSELIAN_YEAR: (u64, u64) = (365_242_198_781, 1_000_000_000);

/// The Julian epoch's constants: `J = 2000.0 + (JD − 2451545.0) / 365.25`.
const JULIAN_YEAR_DAYS: (u64, u64) = (36_525, 100);

/// Which epoch notation a figure like `1950.0` is written in.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum EpochKind {
    /// `J` — Julian years of exactly 365.25 days from J2000.0.
    Julian,
    /// `B` — tropical years of 365.242198781 days from a 1900 origin.
    Besselian,
}

/// A Julian or Besselian epoch, as a Julian Date in TT.
///
/// # Why the prefix is required
///
/// `B1950.0` and `J1950.0` are **1.84 hours apart** — `JD 2433282.42346` against
/// `JD 2433282.50000` — because they count different years from different
/// origins. Catalogue positions are still published against B1950 in the older
/// literature, and a bare `1950.0` does not say which is meant.
///
/// So the prefix is required, exactly as `--scale` is required for a Julian
/// Date, and for the same reason: a default here is silently wrong by an amount
/// that looks like nothing. Gaia DR3's `2016.0` is a *Julian* epoch and must be
/// written `J2016.0`.
pub fn epoch_to_jd(text: &str) -> Result<(Ratio, EpochKind)> {
    let t = text.trim();
    let (kind, rest) = match t.as_bytes().first() {
        Some(b'J') | Some(b'j') => (EpochKind::Julian, &t[1..]),
        Some(b'B') | Some(b'b') => (EpochKind::Besselian, &t[1..]),
        _ => {
            return Err(TimeError::with_context(
                Code::E0001,
                "an epoch needs its `J` or `B` prefix. `B1950.0` and `J1950.0` \
                 are 1.84 hours apart, because they count different years from \
                 different origins, and a bare figure does not say which is \
                 meant. Gaia DR3's `2016.0` is Julian: write `J2016.0`",
            ))
        }
    };
    let year = Ratio::from_decimal_str(rest.trim()).map_err(|_| {
        TimeError::with_context(
            Code::E0001,
            "an epoch is a decimal year after its prefix, like `J2000.0`",
        )
    })?;

    let (origin_year, origin_jd, year_days) = match kind {
        EpochKind::Julian => (
            Ratio::from_u64(2000),
            Ratio::from_u64(J2000_JD),
            ratio(JULIAN_YEAR_DAYS)?,
        ),
        EpochKind::Besselian => (
            Ratio::from_u64(1900),
            ratio(BESSELIAN_ORIGIN)?,
            ratio(BESSELIAN_YEAR)?,
        ),
    };

    // `Ratio` is unsigned (Rule B), so an epoch before the origin is a branch.
    let forward = year.cmp_exact(&origin_year) != core::cmp::Ordering::Less;
    let elapsed = if forward {
        year.sub(&origin_year)?
    } else {
        origin_year.sub(&year)?
    };
    let days = elapsed.mul(&year_days)?;
    if forward {
        Ok((origin_jd.add(&days)?, kind))
    } else {
        Ok((origin_jd.sub(&days)?, kind))
    }
}

/// A rational from a `(numerator, denominator)` pair of `u64`.
fn ratio(p: (u64, u64)) -> Result<Ratio> {
    Ratio::new(
        <Ticks as TickInt>::from_u64(p.0),
        <Ticks as TickInt>::from_u64(p.1),
    )
}

/// The 1977 epoch, as a Julian Date in TT.
fn t0_jd() -> Result<Ratio> {
    ratio((T0_JD_NUM, T0_JD_DEN))
}

/// A coordinate-time Julian Date, converted to the dynamical scale under it.
///
/// `TCG` sits over `TT` and `TCB` over `TDB`, and both relations have the same
/// shape: the dynamical scale's elapsed time is the coordinate scale's, scaled
/// by `(1 − L)`. Given a **coordinate** reading, this returns the corresponding
/// **dynamical** reading, both as Julian Dates.
///
/// `TCB` additionally carries `TDB_0`, a defining constant offset.
fn coordinate_to_dynamical(jd: &Ratio, scale: JdScale) -> Result<Ratio> {
    let (l, offset_seconds) = match scale {
        JdScale::Tcg => (ratio(L_G)?, None),
        JdScale::Tcb => (ratio(L_B)?, Some(ratio(TDB_0_NEG)?)),
        _ => return Ok(jd.clone()),
    };
    let t0 = t0_jd()?;
    // Elapsed coordinate time since the 1977 epoch, in days. Before it, the
    // scaling runs the other way; `Ratio` is unsigned so the branch is explicit.
    let forward = jd.cmp_exact(&t0) != core::cmp::Ordering::Less;
    let elapsed = if forward { jd.sub(&t0)? } else { t0.sub(jd)? };
    let scaled = elapsed.mul(&Ratio::one().sub(&l)?)?;
    let mut out = if forward { t0.add(&scaled)? } else { t0.sub(&scaled)? };
    if let Some(secs) = offset_seconds {
        // TDB_0 is negative, and it is a number of seconds against a Julian
        // Date in days.
        let in_days = secs.div(&Ratio::from_u64(86_400))?;
        out = out.sub(&in_days)?;
    }
    Ok(out)
}

/// The inverse: a dynamical Julian Date, as a coordinate one.
fn dynamical_to_coordinate(jd: &Ratio, scale: JdScale) -> Result<Ratio> {
    let (l, offset_seconds) = match scale {
        JdScale::Tcg => (ratio(L_G)?, None),
        JdScale::Tcb => (ratio(L_B)?, Some(ratio(TDB_0_NEG)?)),
        _ => return Ok(jd.clone()),
    };
    let t0 = t0_jd()?;
    let mut jd = jd.clone();
    if let Some(secs) = offset_seconds {
        jd = jd.add(&secs.div(&Ratio::from_u64(86_400))?)?;
    }
    let forward = jd.cmp_exact(&t0) != core::cmp::Ordering::Less;
    let elapsed = if forward { jd.sub(&t0)? } else { t0.sub(&jd)? };
    let scaled = elapsed.div(&Ratio::one().sub(&l)?)?;
    if forward {
        t0.add(&scaled)
    } else {
        t0.sub(&scaled)
    }
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
    // E1 — a coordinate scale is exactly a rate away from the dynamical scale
    // beneath it, so it converts to that and then follows the same path. `TCG`
    // lands on `TT` and is exact; `TCB` lands on `TDB` and inherits its bound,
    // and nothing but its bound — the rate step adds no uncertainty at all.
    let (jd, scale) = match scale {
        JdScale::Tcg => (coordinate_to_dynamical(jd, JdScale::Tcg)?, JdScale::Tt),
        JdScale::Tcb => (coordinate_to_dynamical(jd, JdScale::Tcb)?, JdScale::Tdb),
        other => (jd.clone(), other),
    };
    let jd = &jd;
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
        // The coordinate scales were folded to their dynamical base above and
        // cannot reach here; an explicit arm rather than a wildcard, so that a
        // scale added later fails to compile instead of silently converting
        // exactly when it should not.
        JdScale::Tcg | JdScale::Tcb => Err(TimeError::with_context(
            Code::E0019,
            "a coordinate time scale reached the dynamical conversion, which \
             means the fold above stopped covering it",
        )),
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
    // The inverse of the fold above: compute in the dynamical scale, then lift.
    if matches!(scale, JdScale::Tcg | JdScale::Tcb) {
        let base = to_jd(t, if scale == JdScale::Tcg { JdScale::Tt } else { JdScale::Tdb })?;
        return dynamical_to_coordinate(&base, scale);
    }
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

/// D — an independent implementation, to check this one against.
///
/// # Why an oracle at all
///
/// Every claim A1 makes about exactness is checked *by A1's own arithmetic*:
/// the round trip uses `from_jd` and `to_jd`, and a matched pair of errors in
/// the two would cancel and pass. `S1` records the general form of this — the
/// strongest claim available to a project this size is agreement with an
/// independent implementation, and `ucal verify` already has to say it cannot
/// make one.
///
/// `hifitime` is that implementation. It is already a dependency of this crate,
/// under a permissive licence, written by someone else from the standards
/// rather than from this code. `S1` assessed `siderust` for the same job and
/// **rejected it**: AGPL-3.0-only against this workspace's MPL-2.0 settles it
/// before `f64`, `std`-only and 351 kSLoC each settle it again.
///
/// # What agreement means here, and what it does not
///
/// `hifitime` returns `f64` days. A Julian Date near 2 460 000 has about 2×10⁻¹¹
/// days — roughly **2 microseconds** — of representation spacing in a double, so
/// the oracle cannot check this crate past that. It is not the finer of the two.
///
/// So the assertion is one-directional and stated as such: **the exact value
/// must lie within one double's spacing of the oracle's**. That catches a wrong
/// epoch, a wrong day length, a scale applied backwards and an off-by-one in the
/// MJD offset — every defect that is larger than a microsecond, which is every
/// defect anybody makes here. It cannot catch a sub-microsecond error, and
/// nothing available could.
// ucal-lint-allow-begin(float-free): Rule E permits a float reference
// implementation in test code, marked as such. Everything between this marker
// and its `-end` is `#[cfg(test)]`, unreachable from any shipped artefact, and
// used only to check this crate's exact answer against an independent one.
#[cfg(all(test, feature = "hifitime"))]
mod oracle {
    use super::*;

    /// The oracle's Julian Date for an instant, in a scale.
    ///
    /// **Both scales, deliberately.** The first version of this test looped over
    /// scales while converting back through `Tt` every time, so the TAI offset
    /// cancelled on both sides and the loop compared one path with itself.
    /// Injecting the offset backwards — the classic error, worth 64.368 s —
    /// did not fail it. Found by running the injection rather than by reading
    /// the test, which is the whole reason the injections are run.
    fn hifitime_jd(t: &Instant<UC1>, scale: JdScale) -> Option<f64> {
        let (e, _) = crate::bridge::to_epoch(t, ucal_core::Rounding::HalfEven).ok()?;
        Some(match scale {
            JdScale::Tt | JdScale::Tdb => e.to_jde_tt_days(),
            JdScale::Tai => e.to_jde_tai_days(),
            // hifitime has TCG and TCB too, but its TCB in particular differs
            // in which epoch offset it folds in; comparing them would be
            // comparing two conventions rather than two implementations of one.
            // Left out and said so, rather than quietly asserted.
            JdScale::Tcg | JdScale::Tcb => return None,
        })
    }

    /// One double's spacing at a Julian Date, in days.
    fn spacing(jd: f64) -> f64 {
        // `f64::EPSILON` scaled to the magnitude: the gap between representable
        // neighbours near `jd`. Two of them, because the oracle's own arithmetic
        // rounds at least once on the way.
        2.0 * jd.abs() * f64::EPSILON
    }

    /// This crate's Julian Date agrees with an independent one.
    ///
    /// Across five orders of magnitude of offset from the epoch, both
    /// directions, and both exact scales.
    #[test]
    fn the_julian_date_agrees_with_hifitime() {
        let mut checked = 0usize;
        for days in [0i64, 1, -1, 365, -365, 36_525, -36_525, 730_500] {
            for scale in [JdScale::Tt, JdScale::Tai] {
                let jd = Ratio::from_u64(J2000_JD);
                let jd = if days >= 0 {
                    jd.add(&Ratio::from_u64(days as u64)).expect("in range")
                } else {
                    jd.sub(&Ratio::from_u64(days.unsigned_abs())).expect("in range")
                };
                let jd_f: f64 = jd
                    // The oracle compares against an `f64`, whose spacing
                    // here is ~2 µs — coarser than any two of the four modes
                    // could differ by. There is no caller to take a mode from.
                    .to_decimal_string(9, ucal_core::Rounding::HalfEven) // ucal-lint-allow(rounding-is-declared)
                    .expect("rendered")
                    .parse()
                    .expect("a number");

                // **Forwards**: the instant `from_jd` produces for a date in
                // this scale must read back as that date, to the oracle, in the
                // same scale. This is what exercises `from_jd`'s scale branch —
                // the earlier version converted forwards in `Tt` always, so
                // injecting the TAI offset backwards still passed.
                let w = from_jd(&jd, scale).expect("exact");
                let theirs = hifitime_jd(w.lo(), scale).expect("the oracle answers");
                let gap = (jd_f - theirs).abs();
                assert!(
                    gap <= spacing(theirs),
                    "forwards {scale:?} at J2000{days:+}: asked {jd_f}, hifitime \
                     reads {theirs}, gap {gap} exceeds one double's spacing {}",
                    spacing(theirs)
                );

                // **Backwards**: and `to_jd` returns what was asked for.
                let ours = to_jd(w.lo(), scale).expect("exact");
                let ours_f: f64 = ours
                    // The oracle compares against an `f64`, whose spacing
                    // here is ~2 µs — coarser than any two of the four modes
                    // could differ by. There is no caller to take a mode from.
                    .to_decimal_string(9, ucal_core::Rounding::HalfEven) // ucal-lint-allow(rounding-is-declared)
                    .expect("rendered")
                    .parse()
                    .expect("a number");
                let gap = (ours_f - theirs).abs();
                assert!(
                    gap <= spacing(theirs),
                    "backwards {scale:?} at J2000{days:+}: ours {ours_f}, \
                     hifitime {theirs}, gap {gap}"
                );
                checked += 1;
            }
        }
        assert_eq!(checked, 16, "the sweep stopped early");
    }

    /// And the epoch itself, which is the one value with a defined answer.
    ///
    /// J2000.0 **is** JD 2451545.0 TT by definition, so this is the one case
    /// where the oracle is not the authority — the definition is, and both must
    /// meet it.
    #[test]
    fn both_implementations_put_j2000_at_2451545() {
        let theirs =
            hifitime_jd(&j2000().expect("epoch"), JdScale::Tt).expect("the oracle answers");
        assert!(
            (theirs - 2_451_545.0).abs() <= spacing(2_451_545.0),
            "hifitime puts J2000 at {theirs}"
        );
    }
}
// ucal-lint-allow-end(float-free)

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

    // ---- E1 ----

    /// **`TCG` and `TCB` convert exactly, and `TDB` does not.**
    ///
    /// The whole point of E1 in one assertion: two of the three coordinate-ish
    /// scales are a *defining* rate away from their base and produce a
    /// zero-width window, while `TDB` is a series and produces a bounded one.
    #[test]
    fn tcg_is_exact_and_tcb_is_only_as_uncertain_as_tdb() {
        let jd = Ratio::from_u64(J2000_JD);
        let tcg = from_jd(&jd, JdScale::Tcg).expect("exact");
        assert_eq!(
            tcg.lo().ticks(),
            tcg.hi().ticks(),
            "TCG is a defining rate away from TT, so it converts exactly"
        );

        let tdb = from_jd(&jd, JdScale::Tdb).expect("bounded");
        let tcb = from_jd(&jd, JdScale::Tcb).expect("bounded");
        assert_eq!(
            tcb.width().ticks(),
            tdb.width().ticks(),
            "TCB inherits TDB's bound and adds nothing: the rate step is exact"
        );
    }

    /// `TCG` round-trips **exactly**; `TCB` round-trips to within `TDB`'s bound.
    ///
    /// The asymmetry is the finding, not a defect. `TCG` sits over `TT`, which
    /// converts exactly, so nothing is lost. `TCB` sits over `TDB`, whose window
    /// is 3.4 ms wide, and `from_jd` hands back its low end — so a round trip
    /// through it *cannot* be exact, and a test asserting it was would be
    /// asserting that the bound is not real.
    #[test]
    fn tcg_round_trips_exactly_and_tcb_within_tdbs_bound() {
        for days in [0u64, 1, 3652, 36_525] {
            let jd = Ratio::from_u64(J2000_JD)
                .add(&Ratio::from_u64(days))
                .expect("in range");

            let w = from_jd(&jd, JdScale::Tcg).expect("converts");
            let back = to_jd(w.lo(), JdScale::Tcg).expect("converts");
            assert_eq!(
                back.cmp_exact(&jd),
                core::cmp::Ordering::Equal,
                "TCG at J2000+{days} did not round-trip exactly"
            );

            let w = from_jd(&jd, JdScale::Tcb).expect("converts");
            let back = to_jd(w.lo(), JdScale::Tcb).expect("converts");
            // TDB's 1.7 ms, expressed in TCB — which is `1/(1 − L_B)` larger,
            // because TCB runs faster. The difference is 26 picoseconds and it
            // is why the first version of this assertion failed: a bound is in
            // the units of the scale that states it, and converting one between
            // scales scales it too.
            let bound_days = Ratio::from_u64(TDB_BOUND_NANOS)
                .div(&Ratio::from_u64(1_000_000_000))
                .and_then(|v| v.div(&Ratio::from_u64(86_400)))
                .and_then(|v| v.div(&Ratio::one().sub(&ratio(L_B)?)?))
                .expect("in range");
            let gap = back.abs_diff(&jd).expect("a gap");
            assert!(
                gap.cmp_exact(&bound_days) != core::cmp::Ordering::Greater,
                "TCB at J2000+{days} moved by more than TDB's own bound"
            );
        }
    }

    /// **The measured drift, which is why the distinction matters.**
    ///
    /// `TCB` runs ahead of `TDB` by 0.489 s per Julian year and `TCG` ahead of
    /// `TT` by 0.022 s. Half a second a year, in a field whose residuals are
    /// microseconds — a linear drift, not a rounding.
    #[test]
    fn the_rates_are_the_published_ones() {
        // One Julian year of coordinate time after the 1977 epoch, and how far
        // the dynamical scale beneath it has got.
        let year = Ratio::from_u64(36_525)
            .div(&Ratio::from_u64(100))
            .expect("365.25 d");
        let start = t0_jd().expect("epoch");
        let end = start.add(&year).expect("in range");

        // Microseconds, because the answers are 22 ms and 489 ms and a
        // millisecond count would round the first to nothing useful.
        for (scale, want_us, tol_us) in [
            (JdScale::Tcg, 21_993u64, 10u64),
            (JdScale::Tcb, 489_306, 100),
        ] {
            let base = coordinate_to_dynamical(&end, scale).expect("converts");
            let behind = end.sub(&base).expect("the coordinate scale runs ahead");
            let us = behind
                .mul(&Ratio::from_u64(86_400_000_000))
                .expect("in range")
                .floor()
                .to_dec_string()
                .parse::<u64>()
                .expect("a count");
            assert!(
                us.abs_diff(want_us) < tol_us,
                "{scale:?}: {us} µs per Julian year, expected about {want_us}"
            );
        }
    }

    // ---- V2 ----

    /// J2000.0 is JD 2451545.0 by construction, which is the whole point of it.
    #[test]
    fn j2000_is_the_epoch_it_is_named_for() {
        let (jd, kind) = epoch_to_jd("J2000.0").expect("an epoch");
        assert_eq!(kind, EpochKind::Julian);
        assert_eq!(
            jd.cmp_exact(&Ratio::from_u64(J2000_JD)),
            core::cmp::Ordering::Equal
        );
    }

    /// **The trap, measured.** `B1950.0` and `J1950.0` are 1.84 hours apart.
    ///
    /// Not eighteen, which is what a first draft of the proposal said. The
    /// figures are `JD 2433282.42346` and `JD 2433282.50000`, and the difference
    /// is `0.07654` days.
    #[test]
    fn besselian_and_julian_1950_are_not_the_same_instant() {
        let (b, _) = epoch_to_jd("B1950.0").expect("an epoch");
        let (j, _) = epoch_to_jd("J1950.0").expect("an epoch");
        assert_eq!(
            b.to_decimal_string(5, ucal_core::Rounding::HalfEven).expect("rendered"),
            "2433282.42346"
        );
        assert_eq!(
            j.to_decimal_string(5, ucal_core::Rounding::HalfEven).expect("rendered"),
            "2433282.50000"
        );
        let gap_hours = j
            .sub(&b)
            .expect("J is later")
            .mul(&Ratio::from_u64(24))
            .expect("in range");
        assert_eq!(
            gap_hours.to_decimal_string(2, ucal_core::Rounding::HalfEven).expect("rendered"),
            "1.84"
        );
    }

    /// A bare figure is refused, and the message names the case that motivates it.
    #[test]
    fn an_epoch_without_its_prefix_is_refused() {
        let e = epoch_to_jd("2016.0").expect_err("J or B, not neither");
        assert_eq!(e.code, Code::E0001);
        assert!(format!("{e}").contains("J2016.0"), "{e}");
    }

    /// Epochs before the origin work, since `Ratio` is unsigned and this is a
    /// branch rather than a sign.
    #[test]
    fn an_epoch_before_its_origin_converts() {
        let (early, _) = epoch_to_jd("J1900.0").expect("an epoch");
        let (late, _) = epoch_to_jd("J2000.0").expect("an epoch");
        assert_eq!(early.cmp_exact(&late), core::cmp::Ordering::Less);
        // A century of Julian years is exactly 36525 days.
        let gap = late.sub(&early).expect("later");
        assert_eq!(gap.cmp_exact(&Ratio::from_u64(36_525)), core::cmp::Ordering::Equal);
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
