//! Gravitational time dilation, as a **certified** interval.
//!
//! [`S2`] proposed this and named the primitive it needed:
//! `ucal-core` already has [`isqrt_floor`] and [`isqrt_ceil`], which is exactly
//! what brackets `√(1 − r_s/r)` in exact rationals with no float anywhere.
//!
//! # What it computes
//!
//! For a static observer at Schwarzschild radial coordinate `r` outside a
//! non-rotating mass, with `r_s = 2GM/c²`:
//!
//! ```text
//! dτ/dt = √(1 − r_s/r)          proper time per unit coordinate time
//! z     = 1/√(1 − r_s/r) − 1    the gravitational redshift
//! ```
//!
//! Both are returned as intervals that are **proved to contain** the true value,
//! not as iterates that stopped moving. Same standard the ΛCDM quadrature beside
//! this is held to.
//!
//! # Why exactness earns its keep here, measured
//!
//! Not in the strong field — in the **weak** one, and near the horizon, for the
//! same reason at both ends: catastrophic cancellation.
//!
//! `z = 1/√(1−x) − 1` in `f64`, against the exact value:
//!
//! | | `r_s/r` | correct significant digits in `f64` |
//! |---|---|---|
//! | the Sun | 4.25 × 10⁻⁶ | **~8** of 16 |
//! | Sirius B, a white dwarf | 1.15 × 10⁻⁴ | ~11 |
//! | a neutron star | 0.35 | ~14 |
//! | `r` just outside a horizon | 0.999999 | **~1** |
//!
//! **The two ends lose and the middle is fine.** In the weak field `1/√(1−x)` is
//! a hair above 1 and subtracting 1 throws away half the mantissa; near the
//! horizon `1 − x` is itself the cancellation. A neutron star — the case that
//! *sounds* like it needs care — is where floating point does best.
//!
//! That is the answer to whether this is only a black-hole tool. **The solar
//! gravitational redshift is a measured quantity** (~633 m/s equivalent), white
//! dwarf redshifts are how their masses are checked, and both sit in the band
//! where `f64` has already lost half its digits before anything is compared.
//!
//! # What this is not
//!
//! **Not a claim that `UC-1` measures proper time.** Tick 0 is the FLRW `t → 0`
//! limit, so absolute time here is a cosmological coordinate; this function
//! reports the *ratio between two clocks* and does not assert that either is the
//! one `ucal` keeps. [`S2`] records why giving `UC-1` a stated frame is a 2.0
//! question: a single unsigned integer per instant asserts there is one time,
//! which is the thing general relativity denies.
//!
//! [`S2`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/S2-deep-time.md
//! [`isqrt_floor`]: ucal_core::num::isqrt_floor
//! [`isqrt_ceil`]: ucal_core::num::isqrt_ceil

use ucal_core::backend::TickInt;
use ucal_core::num::{isqrt_ceil, isqrt_floor, RatInterval, Ratio};
use ucal_core::{Code, Ticks, TimeError};

type Result<T> = core::result::Result<T, TimeError>;

/// A certified dilation at one radius.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Dilation {
    /// `r_s/r`, exactly as given.
    pub ratio: Ratio,
    /// `√(1 − r_s/r)` — proper time per unit coordinate time.
    pub factor: RatInterval,
    /// `1/√(1 − r_s/r)` — coordinate time per unit proper time.
    pub inverse: RatInterval,
    /// `1/√(1 − r_s/r) − 1` — the gravitational redshift `z`.
    pub redshift: RatInterval,
    /// Decimal digits the bracketing was carried to.
    pub digits: u32,
}

/// `10^d`, as a tick integer.
fn pow10(d: u32) -> Result<Ticks> {
    let ten = <Ticks as TickInt>::from_u64(10);
    let mut out = <Ticks as TickInt>::one();
    for _ in 0..d {
        out = out.try_mul(&ten).ok_or(TimeError::with_context(
            Code::E0021,
            "that many digits exceeds the domain. The bracketing scales by \
             10^(2d) before taking a square root, so the cost is quadratic in \
             the digits asked for",
        ))?;
    }
    Ok(out)
}

/// A certified enclosure of `√(num/den)`, to `digits` decimal places.
///
/// `√(a/b) = √(a·b)/b`, scaled by `10^d` before the root so the integer square
/// root has something to work with: without the scaling, `√(1/2)` brackets to
/// `[0, 1]`, which is true and useless.
///
/// The two ends use **different** roots — floor for the low, ceiling for the
/// high — so the interval contains the value rather than approximating it from
/// one side. GE-3 forbids narrowing by assumption, and this is where that would
/// happen if the same root were used twice.
fn sqrt_enclosure(num: &Ticks, den: &Ticks, digits: u32) -> Result<RatInterval> {
    if den.is_zero_ticks() {
        return Err(TimeError::new(Code::E0070));
    }
    let scale = pow10(digits)?;
    let inner = num
        .clone()
        .try_mul(den)
        .and_then(|v| v.try_mul(&scale))
        .and_then(|v| v.try_mul(&scale))
        .ok_or(TimeError::with_context(
            Code::E0021,
            "the scaled radicand exceeds the domain; ask for fewer digits",
        ))?;
    let bottom = den.clone().try_mul(&scale).ok_or(TimeError::new(Code::E0021))?;
    RatInterval::new(
        Ratio::new(isqrt_floor(&inner), bottom.clone())?,
        Ratio::new(isqrt_ceil(&inner), bottom)?,
    )
}

/// Gravitational time dilation at `r_s/r`, certified.
///
/// `ratio` is dimensionless and must lie in `[0, 1)`. Deliberately **not**
/// `(M, r)`: that would need `G` and a solar mass as declared constants, and
/// Rule C wants both cited before either is shipped. The formula is
/// `r_s/r = 2GM/(rc²)`, and supplying those constants is a separate piece of
/// data work rather than a line of code.
pub fn schwarzschild(ratio: &Ratio, digits: u32) -> Result<Dilation> {
    if digits == 0 || digits > 60 {
        return Err(TimeError::with_context(
            Code::E0018,
            "ask for between 1 and 60 digits. Zero brackets to something true \
             and useless; the cost of the bracketing is quadratic in the digits",
        ));
    }
    if ratio.cmp_exact(&Ratio::one()) != core::cmp::Ordering::Less {
        return Err(TimeError::with_context(
            Code::E0018,
            "r_s/r must be below 1. At r = r_s the factor is zero and at r < r_s \
             the Schwarzschild radial coordinate is not a clock at all — the \
             coordinate that was time outside the horizon is not time inside it. \
             That is a change of what the question means, not a larger number",
        ));
    }

    // 1 − x, exactly. `Ratio` is unsigned (Rule B) and `x < 1` was just
    // established, so this subtraction cannot go negative.
    let rest = Ratio::one().sub(ratio)?;
    let (a, b) = (rest.numer().clone(), rest.denom().clone());

    let factor = sqrt_enclosure(&a, &b, digits)?;
    let inverse = sqrt_enclosure(&b, &a, digits)?;

    // z = 1/√(1−x) − 1, both ends. The inverse is ≥ 1 for x ≥ 0, and equals 1
    // exactly at x = 0 — where the *low* bracket can fall a hair under 1. Then
    // z's low end is 0 rather than a negative number, which is not a clamp: a
    // gravitational redshift is non-negative outside the horizon, so 0 is the
    // true bound and the bracket was the thing that was loose.
    let one = Ratio::one();
    let z_lo = match inverse.lo().cmp_exact(&one) {
        core::cmp::Ordering::Greater => inverse.lo().sub(&one)?,
        _ => Ratio::zero(),
    };
    let z_hi = match inverse.hi().cmp_exact(&one) {
        core::cmp::Ordering::Greater => inverse.hi().sub(&one)?,
        _ => Ratio::zero(),
    };
    let redshift = RatInterval::new(z_lo, z_hi)?;

    Ok(Dilation {
        ratio: ratio.clone(),
        factor,
        inverse,
        redshift,
        digits,
    })
}

/// **D1 — a circular orbit, where gravitational and kinematic dilation combine.**
///
/// A *static* observer at `r` runs at `√(1 − r_s/r)`. One in a **circular orbit**
/// runs at `√(1 − 3r_s/(2r))`: the extra half comes from its orbital speed, and
/// the factor of 3/2 is the whole difference between the two cases.
///
/// This is the case that covers GPS clocks, the S2 star around Sgr A*, and any
/// pulsar in a binary — which is most of the ones whose timing anybody cares
/// about.
///
/// # Why `r_s/r ≥ 2/3` is refused
///
/// At `r = 1.5 r_s` the factor reaches zero: that is the **photon sphere**,
/// where a circular orbit requires the speed of light. Inside it no circular
/// orbit exists at all, so the question has no answer rather than a small one —
/// a different refusal from [`schwarzschild`]'s horizon, and worth its own
/// message because a caller who confuses them has confused two radii.
pub fn circular_orbit(ratio: &Ratio, digits: u32) -> Result<Dilation> {
    let two_thirds = Ratio::new(
        <Ticks as TickInt>::from_u64(2),
        <Ticks as TickInt>::from_u64(3),
    )?;
    if ratio.cmp_exact(&two_thirds) != core::cmp::Ordering::Less {
        return Err(TimeError::with_context(
            Code::E0018,
            "r_s/r must be below 2/3 for a circular orbit. At r = 1.5 r_s \
             the factor is zero — that is the photon sphere, where a \
             circular orbit would need the speed of light, and inside it \
             no circular orbit exists. This is a different radius from \
             the horizon, and a caller reaching it has confused the two",
        ));
    }
    // The static formula, evaluated at 3x/2. Deliberately reusing it rather
    // than duplicating the bracketing: one square root, one place it can be
    // wrong. The guard above is what makes the argument admissible.
    let effective = ratio
        .mul(&Ratio::new(
            <Ticks as TickInt>::from_u64(3),
            <Ticks as TickInt>::from_u64(2),
        )?)?;
    let mut out = schwarzschild(&effective, digits)?;
    // Report the radius the caller asked about, not the one the formula used.
    out.ratio = ratio.clone();
    Ok(out)
}

/// **V3 — kinematic dilation: a clock that is moving, in flat spacetime.**
///
/// `dτ/dt = √(1 − β²)`, `β = v/c`. The case every physicist meets first, and the
/// one this module was missing: [`schwarzschild`] is a clock deep in a well and
/// [`circular_orbit`] is a clock in a well *and* moving, so this is the third
/// corner and the only one with no gravity in it at all.
///
/// # It is the weak-field cancellation case again
///
/// At `β = 10⁻⁴` — a fast spacecraft — `1 − √(1−β²)` is `5.0 × 10⁻⁹`, and an
/// `f64` computing it by that expression subtracts two numbers agreeing to eight
/// places. The bracket here does not: it works in `1 − β²` throughout and never
/// forms the difference at all.
///
/// # `β ≥ 1` is refused
///
/// Not because the arithmetic breaks — `1 − β²` would go negative and `Ratio` is
/// unsigned, so it breaks too — but because it is a different question. A clock
/// at or above `c` is not a slow clock; it is not a clock.
pub fn kinematic(beta: &Ratio, digits: u32) -> Result<Dilation> {
    if beta.cmp_exact(&Ratio::one()) != core::cmp::Ordering::Less {
        return Err(TimeError::with_context(
            Code::E0018,
            "β = v/c must be below 1. At β = 1 the factor is zero and above it \
             there is no factor: a clock at or beyond the speed of light is not \
             a slow clock, it is not a clock, and that is a change in the \
             question rather than a larger number",
        ));
    }
    // `√(1 − β²)` shares every line below with the gravitational case once the
    // argument is formed, so it is formed and handed over rather than a second
    // square root being written. `β²` is exact; `1 − β²` is exact; the root is
    // the only bracketed step, in one place.
    let b2 = beta.mul(beta)?;
    let mut out = schwarzschild(&b2, digits)?;
    // `schwarzschild` reports the `r_s/r` it was given, which here is `β²`. The
    // caller asked about `β`.
    out.ratio = beta.clone();
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::Rounding;

    fn r(n: u64, d: u64) -> Ratio {
        Ratio::new(
            <Ticks as TickInt>::from_u64(n),
            <Ticks as TickInt>::from_u64(d),
        )
        .expect("a ratio")
    }

    /// **The property: the interval contains the answer.**
    ///
    /// Checked by squaring. `lo² ≤ 1 − x ≤ hi²` is the statement that the
    /// bracket encloses `√(1 − x)`, and it needs no float and no reference
    /// value to check — which is the point of certifying rather than iterating.
    #[test]
    fn the_bracket_encloses_the_root() {
        for (n, d) in [(0u64, 1u64), (1, 2), (1, 3), (35, 100), (999_999, 1_000_000)] {
            let x = r(n, d);
            let out = schwarzschild(&x, 30).expect("in range");
            let rest = Ratio::one().sub(&x).expect("below 1");
            let lo2 = out.factor.lo().mul(out.factor.lo()).expect("in range");
            let hi2 = out.factor.hi().mul(out.factor.hi()).expect("in range");
            assert!(
                lo2.cmp_exact(&rest) != core::cmp::Ordering::Greater,
                "{n}/{d}: low end is above the value"
            );
            assert!(
                hi2.cmp_exact(&rest) != core::cmp::Ordering::Less,
                "{n}/{d}: high end is below the value"
            );
        }
    }

    /// More digits is a narrower interval, and never a wrong one.
    #[test]
    fn asking_for_more_digits_narrows_it() {
        let x = r(1, 2);
        let coarse = schwarzschild(&x, 4).expect("in range");
        let fine = schwarzschild(&x, 30).expect("in range");
        let wc = coarse.factor.width().expect("a width");
        let wf = fine.factor.width().expect("a width");
        assert_eq!(wf.cmp_exact(&wc), core::cmp::Ordering::Less);
        // And the fine bracket sits inside the coarse one.
        assert!(coarse.factor.contains(fine.factor.lo()));
        assert!(coarse.factor.contains(fine.factor.hi()));
    }

    /// At `r_s/r = 0` there is no dilation and no redshift.
    #[test]
    fn flat_space_is_one_and_zero() {
        let out = schwarzschild(&Ratio::zero(), 20).expect("in range");
        assert_eq!(
            out.factor
                .lo()
                .to_decimal_string(6, Rounding::Trunc)
                .expect("rendered"),
            "1.000000"
        );
        assert_eq!(
            out.redshift
                .hi()
                .to_decimal_string(6, Rounding::Trunc)
                .expect("rendered"),
            "0.000000"
        );
    }

    /// **The weak field, which is where `f64` loses half its digits.**
    ///
    /// `r_s/r ≈ 4.2467e-6` is the Sun's surface. The redshift is ~2.1234e-6,
    /// and the exact bracket pins digits an `f64` computation of
    /// `1/√(1−x) − 1` has already lost to cancellation.
    #[test]
    fn the_solar_surface_is_bracketed_past_where_a_float_stops() {
        let x = r(42_467, 10_000_000_000);
        let out = schwarzschild(&x, 40).expect("in range");
        let lo = out
            .redshift
            .lo()
            .to_decimal_string(18, Rounding::Trunc)
            .expect("rendered");
        let hi = out
            .redshift
            .hi()
            .to_decimal_string(18, Rounding::Trunc)
            .expect("rendered");
        assert!(lo.starts_with("0.000002123356"), "{lo}");
        assert!(hi.starts_with("0.000002123356"), "{hi}");
        // Agreeing to 18 places is well past f64's ~8 correct digits here.
        assert_eq!(lo, hi, "the bracket is tighter than a double at this ratio");
    }

    /// At and inside the horizon the question changes, and is refused.
    #[test]
    fn the_horizon_and_inside_it_are_refused() {
        for (n, d) in [(1u64, 1u64), (2, 1)] {
            let e = schwarzschild(&r(n, d), 20).expect_err("not a clock there");
            assert_eq!(e.code, Code::E0018);
            assert!(format!("{e}").contains("not a clock"), "{e}");
        }
    }

    // ---- D1 ----

    /// An orbiting clock runs slower than a static one at the same radius.
    ///
    /// It has the same gravitational dilation and a speed on top, so the factor
    /// must be smaller. If the 3/2 were ever dropped the two would agree, which
    /// is the mistake this catches.
    #[test]
    fn an_orbiting_clock_runs_slower_than_a_static_one() {
        for (n, d) in [(1u64, 100u64), (1, 10), (1, 3), (3, 5)] {
            let x = r(n, d);
            let stat = schwarzschild(&x, 30).expect("in range");
            let orb = circular_orbit(&x, 30).expect("in range");
            assert_eq!(
                orb.factor.hi().cmp_exact(stat.factor.lo()),
                core::cmp::Ordering::Less,
                "{n}/{d}: the orbiting clock must be strictly slower"
            );
            // And it reports the radius asked about, not the effective one.
            assert_eq!(orb.ratio.cmp_exact(&x), core::cmp::Ordering::Equal);
        }
    }

    /// The photon sphere is refused, and says which radius it is.
    #[test]
    fn the_photon_sphere_is_refused_and_named() {
        for (n, d) in [(2u64, 3u64), (7, 10), (1, 1)] {
            let e = circular_orbit(&r(n, d), 20).expect_err("no circular orbit there");
            assert_eq!(e.code, Code::E0018);
            assert!(format!("{e}").contains("photon sphere"), "{e}");
        }
        // And just inside the bound it still answers.
        assert!(circular_orbit(&r(66, 100), 20).is_ok());
    }

    // ---- V3 ----

    /// The textbook values, bracketed.
    ///
    /// `β = 3/5` gives exactly `4/5`, and `β = 12/13` gives exactly `5/13` —
    /// the Pythagorean triples, which are the one place this function has an
    /// exact rational answer and the bracket must therefore pin it.
    #[test]
    fn a_pythagorean_beta_brackets_an_exact_answer() {
        for (bn, bd, wn, wd) in [(3u64, 5u64, 4u64, 5u64), (12, 13, 5, 13)] {
            let out = kinematic(&r(bn, bd), 30).expect("in range");
            let want = r(wn, wd);
            assert!(
                out.factor.contains(&want),
                "{bn}/{bd}: bracket does not contain {wn}/{wd}"
            );
            assert_eq!(out.ratio.cmp_exact(&r(bn, bd)), core::cmp::Ordering::Equal);
        }
    }

    /// A faster clock runs slower, monotonically.
    #[test]
    fn more_speed_is_more_dilation() {
        let mut last: Option<Ratio> = None;
        for (n, d) in [(1u64, 100u64), (1, 10), (1, 2), (9, 10), (99, 100)] {
            let out = kinematic(&r(n, d), 30).expect("in range");
            if let Some(prev) = last {
                assert_eq!(
                    out.factor.hi().cmp_exact(&prev),
                    core::cmp::Ordering::Less,
                    "{n}/{d} must be slower than the one before"
                );
            }
            last = Some(out.factor.lo().clone());
        }
    }

    /// `β ≥ 1` is refused, and says why it is a different question.
    #[test]
    fn light_speed_and_beyond_are_refused() {
        for (n, d) in [(1u64, 1u64), (3, 2)] {
            let e = kinematic(&r(n, d), 20).expect_err("not a clock");
            assert_eq!(e.code, Code::E0018);
            assert!(format!("{e}").contains("not a clock"), "{e}");
        }
    }

    /// Zero digits is refused rather than answering `[0, 1]`.
    #[test]
    fn zero_digits_is_refused() {
        assert!(schwarzschild(&r(1, 2), 0).is_err());
    }
}
