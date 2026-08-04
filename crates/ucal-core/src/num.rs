//! Exact integer numerics (Appendix H).
//!
//! Rule E permits exactly five kinds of numeric machinery, and this module is all
//! five: exact integers, exact rationals over them, scaled fixed-point with a
//! declared scale, integer square root with directed rounding, and interval pairs
//! of any of those. There is no sixth kind and no float.
//!
//! The organising idea is that **rounding never happens implicitly**. Every
//! operation is either exact, or it takes a direction and reports it:
//!
//! - [`mul_div`] is exact and hands back the remainder rather than dropping it,
//!   so the caller decides what the remainder means.
//! - [`isqrt_floor`] and [`isqrt_ceil`] are the two directed square roots; there
//!   is no undirected `isqrt`.
//! - [`Ratio`] arithmetic is exact, and overflow is `UCAL-E0021`, never a wrap.
//! - [`RatInterval`] is where inexactness is allowed to live, and it always
//!   widens outward.
//!
//! H.6: no transcendental function is implemented or called anywhere.

#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::cmp::Ordering;

use crate::backend::{TickInt, Ticks};
use crate::error::{Code, Result, TimeError};
use crate::value::Rounding;

/// §13 names a distinct error type for the numeric surface. It is the same type
/// as the rest of the crate's, because the diagnostic codes are the contract and
/// fragmenting the error type would only make them harder to match on.
pub type NumError = TimeError;

// ---------------------------------------------------------------------------
// H.1 — widening multiply-divide
// ---------------------------------------------------------------------------

/// `a x n / d`, computed through an intermediate twice the backend width so the
/// product cannot overflow (Appendix H.1).
///
/// Returns the truncated quotient **and the remainder**. Nothing rounds
/// implicitly: a caller that wants a directed result inspects the remainder and
/// decides. `UCAL-E0021` if the quotient itself leaves the domain.
///
/// ```
/// # use ucal_core::num::mul_div;
/// # use ucal_core::backend::{TickInt, Ticks};
/// let a = <Ticks as TickInt>::from_u64(7);
/// let n = <Ticks as TickInt>::from_u64(5);
/// let d = <Ticks as TickInt>::from_u64(2);
/// let (q, r) = mul_div(&a, &n, &d).unwrap();
/// assert_eq!(q, <Ticks as TickInt>::from_u64(17)); // 35 / 2
/// assert_eq!(r, <Ticks as TickInt>::from_u64(1));
/// ```
pub fn mul_div(a: &Ticks, n: &Ticks, d: &Ticks) -> Result<(Ticks, Ticks)> {
    if d.is_zero_ticks() {
        return Err(TimeError::with_context(
            Code::E0070,
            "mul_div: zero divisor",
        ));
    }
    let wide = a.wide_mul(n);
    let (q, r) = <Ticks as TickInt>::wide_quot_rem(&wide, d);
    let q = <Ticks as TickInt>::narrow(&q).ok_or(TimeError::with_context(
        Code::E0021,
        "mul_div: quotient exceeds the domain",
    ))?;
    Ok((q, r))
}

/// `a x n / d`, rounded in a stated direction (Rule R).
///
/// Separate from [`mul_div`] so that the exact form stays the default and
/// rounding is always a visible choice at the call site.
pub fn mul_div_rounded(a: &Ticks, n: &Ticks, d: &Ticks, mode: Rounding) -> Result<Ticks> {
    let (q, r) = mul_div(a, n, d)?;
    if r.is_zero_ticks() {
        return Ok(q);
    }
    let up = match mode {
        Rounding::Trunc => false,
        Rounding::Ceil => true,
        Rounding::HalfUp | Rounding::HalfEven => {
            let twice = r
                .try_add(&r)
                .ok_or(TimeError::with_context(Code::E0021, "2r overflowed"))?;
            match twice.cmp(d) {
                Ordering::Greater => true,
                Ordering::Less => false,
                Ordering::Equal => match mode {
                    Rounding::HalfUp => true,
                    _ => q.is_odd(),
                },
            }
        }
    };
    if up {
        q.try_add(&<Ticks as TickInt>::one())
            .ok_or(TimeError::new(Code::E0021))
    } else {
        Ok(q)
    }
}

// ---------------------------------------------------------------------------
// H.2 — integer square root with directed rounding
// ---------------------------------------------------------------------------

/// Largest `r` with `r*r <= x`.
///
/// Integer Newton iteration. The post-condition `r^2 <= x < (r+1)^2` is asserted,
/// not assumed — Appendix H.2 requires it, and a silently wrong square root would
/// corrupt every cosmological enclosure downstream while still looking plausible.
pub fn isqrt_floor(x: &Ticks) -> Ticks {
    let one = <Ticks as TickInt>::one();
    if x.is_zero_ticks() || *x == one {
        return x.clone();
    }
    // Start above the true root: 2^ceil(bits/2) >= sqrt(x).
    let bits = x.bit_len();
    let start_exp = bits.div_ceil(2);
    let mut r = <Ticks as TickInt>::pow2(start_exp)
        .unwrap_or_else(<Ticks as TickInt>::domain_max);
    let two = <Ticks as TickInt>::from_u64(2);
    loop {
        // next = (r + x/r) / 2
        let (q, _) = x.quot_rem(&r);
        let sum = match r.try_add(&q) {
            Some(s) => s,
            // r + x/r can exceed the domain only on the first, deliberately
            // over-large guess; halving the guess keeps the iteration going.
            None => {
                let (half, _) = r.quot_rem(&two);
                r = half;
                continue;
            }
        };
        let (next, _) = sum.quot_rem(&two);
        if next >= r {
            break;
        }
        r = next;
    }
    debug_assert!(
        &r.wide_mul(&r) <= &x.wide_mul(&<Ticks as TickInt>::one()),
        "isqrt_floor post-condition r^2 <= x violated"
    );
    debug_assert!(
        {
            let rp1 = r.try_add(&one).expect("r+1 within domain for any real root");
            rp1.wide_mul(&rp1) > x.wide_mul(&one)
        },
        "isqrt_floor post-condition x < (r+1)^2 violated"
    );
    r
}

/// Smallest `r` with `r*r >= x`.
pub fn isqrt_ceil(x: &Ticks) -> Ticks {
    let f = isqrt_floor(x);
    let one = <Ticks as TickInt>::one();
    if f.wide_mul(&f) == x.wide_mul(&one) {
        f
    } else {
        f.try_add(&one)
            .expect("ceil of a root inside the domain stays inside it")
    }
}

// ---------------------------------------------------------------------------
// Exact rationals
// ---------------------------------------------------------------------------

/// An exact non-negative rational over [`Ticks`], always in lowest terms.
///
/// Non-negative because the value domain is unsigned (Rule Z, N12) and every
/// quantity this crate builds on rationals — durations, ratios of periods,
/// redshifts, density parameters — is non-negative. Differences that could go
/// negative are taken with [`Ratio::abs_diff`], which makes the sign loss
/// explicit at the call site rather than silent in the type.
///
/// Arithmetic is exact. Where a result would leave the domain the operation fails
/// with `UCAL-E0021`; it never wraps and never approximates (Rules O, E). Operands
/// are pre-reduced by common factors before multiplying, which keeps intermediates
/// small enough that overflow is rare in practice without ever making it silent.
#[cfg_attr(feature = "u512", derive(Copy))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Ratio {
    num: Ticks,
    den: Ticks,
}

/// Greatest common divisor by Euclid. Exact and total for non-negative inputs.
pub fn gcd(a: &Ticks, b: &Ticks) -> Ticks {
    let mut a = a.clone();
    let mut b = b.clone();
    while !b.is_zero_ticks() {
        let (_, r) = a.quot_rem(&b);
        a = b;
        b = r;
    }
    a
}

impl Ratio {
    /// Construct and reduce. `UCAL-E0070` if the denominator is zero.
    pub fn new(num: Ticks, den: Ticks) -> Result<Ratio> {
        if den.is_zero_ticks() {
            return Err(TimeError::with_context(
                Code::E0070,
                "rational with a zero denominator",
            ));
        }
        let g = gcd(&num, &den);
        let one = <Ticks as TickInt>::one();
        if g == one || g.is_zero_ticks() {
            return Ok(Ratio { num, den });
        }
        let (n, _) = num.quot_rem(&g);
        let (d, _) = den.quot_rem(&g);
        Ok(Ratio { num: n, den: d })
    }

    /// A whole number.
    pub fn from_int(n: Ticks) -> Ratio {
        Ratio {
            num: n,
            den: <Ticks as TickInt>::one(),
        }
    }

    /// A small whole number.
    pub fn from_u64(n: u64) -> Ratio {
        Ratio::from_int(<Ticks as TickInt>::from_u64(n))
    }

    /// Zero.
    pub fn zero() -> Ratio {
        Ratio::from_u64(0)
    }

    /// One.
    pub fn one() -> Ratio {
        Ratio::from_u64(1)
    }

    /// Parse an exact decimal such as `"365.242190"` or `"1089.80"`.
    ///
    /// §10.2 requires redshift inputs to parse exactly: `1100`, `1089.80` and
    /// `0.5` are all exact rationals, and there is no float path in or out.
    /// Rejects anything that is not digits with at most one decimal point.
    pub fn from_decimal_str(s: &str) -> Result<Ratio> {
        let malformed = TimeError::with_context(Code::E0001, "not an exact decimal");
        let (int, frac) = match s.split_once('.') {
            None => (s, ""),
            Some((i, f)) => (i, f),
        };
        if int.is_empty() && frac.is_empty() {
            return Err(malformed);
        }
        if !int.bytes().all(|b| b.is_ascii_digit()) || !frac.bytes().all(|b| b.is_ascii_digit()) {
            return Err(malformed);
        }
        let scale = frac.len() as u32;
        let mut digits = heapless_concat(int, frac);
        if digits.is_empty() {
            digits = "0".into();
        }
        let num = <Ticks as TickInt>::from_dec_str(&digits).ok_or(malformed)?;
        let den = pow10(scale)?;
        Ratio::new(num, den)
    }

    /// The numerator, in lowest terms.
    pub fn numer(&self) -> &Ticks {
        &self.num
    }

    /// The denominator, in lowest terms. Never zero.
    pub fn denom(&self) -> &Ticks {
        &self.den
    }

    /// Whether this is exactly zero.
    pub fn is_zero(&self) -> bool {
        self.num.is_zero_ticks()
    }

    /// Whether the value is a whole number.
    pub fn is_integer(&self) -> bool {
        self.den == <Ticks as TickInt>::one()
    }

    /// The whole part, truncated toward zero.
    pub fn floor(&self) -> Ticks {
        self.num.quot_rem(&self.den).0
    }

    /// The least integer at or above this value.
    ///
    /// The companion to [`floor`](Ratio::floor), and the two are not
    /// interchangeable at the end of an interval computation: quantising an
    /// enclosure to integers must move the lower bound *down* and the upper
    /// bound *up*, or the result stops containing what it bounded. Flooring both
    /// is the mistake this exists to make impossible to write by accident.
    pub fn ceil(&self) -> Ticks {
        let (q, r) = self.num.quot_rem(&self.den);
        if r.is_zero_ticks() {
            q
        } else {
            q.try_add(&<Ticks as TickInt>::one())
                .unwrap_or_else(|| self.num.quot_rem(&self.den).0)
        }
    }

    /// The fractional part, `self - floor(self)`.
    pub fn frac(&self) -> Ratio {
        let (_, r) = self.num.quot_rem(&self.den);
        Ratio {
            num: r,
            den: self.den.clone(),
        }
    }

    /// `self + other`, exact.
    pub fn add(&self, other: &Ratio) -> Result<Ratio> {
        let g = gcd(&self.den, &other.den);
        let (od, _) = other.den.quot_rem(&g);
        // num = self.num * (other.den/g) + other.num * (self.den/g)
        let (sd, _) = self.den.quot_rem(&g);
        let a = self.num.try_mul(&od).ok_or(overflow())?;
        let b = other.num.try_mul(&sd).ok_or(overflow())?;
        let num = a.try_add(&b).ok_or(overflow())?;
        let den = self.den.try_mul(&od).ok_or(overflow())?;
        Ratio::new(num, den)
    }

    /// `self - other`. `UCAL-E0020` if the result would be negative — the type is
    /// non-negative by construction, so this is Rule Z applied to rationals.
    pub fn sub(&self, other: &Ratio) -> Result<Ratio> {
        if self.cmp_exact(other) == Ordering::Less {
            return Err(TimeError::with_context(
                Code::E0020,
                "rational subtraction would be negative; use abs_diff",
            ));
        }
        let g = gcd(&self.den, &other.den);
        let (od, _) = other.den.quot_rem(&g);
        let (sd, _) = self.den.quot_rem(&g);
        let a = self.num.try_mul(&od).ok_or(overflow())?;
        let b = other.num.try_mul(&sd).ok_or(overflow())?;
        let num = a.try_sub(&b).ok_or(overflow())?;
        let den = self.den.try_mul(&od).ok_or(overflow())?;
        Ratio::new(num, den)
    }

    /// `|self - other|`, exact.
    pub fn abs_diff(&self, other: &Ratio) -> Result<Ratio> {
        match self.cmp_exact(other) {
            Ordering::Less => other.sub(self),
            _ => self.sub(other),
        }
    }

    /// `self * other`, exact.
    pub fn mul(&self, other: &Ratio) -> Result<Ratio> {
        // Cross-reduce first so the products stay as small as possible.
        let g1 = gcd(&self.num, &other.den);
        let g2 = gcd(&other.num, &self.den);
        let (n1, _) = self.num.quot_rem(&g1);
        let (d2, _) = other.den.quot_rem(&g1);
        let (n2, _) = other.num.quot_rem(&g2);
        let (d1, _) = self.den.quot_rem(&g2);
        let num = n1.try_mul(&n2).ok_or(overflow())?;
        let den = d1.try_mul(&d2).ok_or(overflow())?;
        Ratio::new(num, den)
    }

    /// `self / other`. `UCAL-E0070` if `other` is zero.
    pub fn div(&self, other: &Ratio) -> Result<Ratio> {
        if other.is_zero() {
            return Err(TimeError::with_context(
                Code::E0070,
                "rational division by zero",
            ));
        }
        self.mul(&other.recip()?)
    }

    /// `1 / self`. `UCAL-E0070` if `self` is zero.
    pub fn recip(&self) -> Result<Ratio> {
        if self.is_zero() {
            return Err(TimeError::with_context(
                Code::E0070,
                "reciprocal of zero",
            ));
        }
        Ok(Ratio {
            num: self.den.clone(),
            den: self.num.clone(),
        })
    }

    /// Exact comparison.
    ///
    /// Cross-multiplies in the widened type, so comparison is total and can never
    /// fail for overflow — which matters, because a comparison that could error
    /// would make [`Ratio`] unusable as an interval endpoint.
    pub fn cmp_exact(&self, other: &Ratio) -> Ordering {
        self.num
            .wide_mul(&other.den)
            .cmp(&other.num.wide_mul(&self.den))
    }

    /// The closest rational with denominator `10^digits`, rounded in the stated
    /// direction.
    ///
    /// Exact rational accumulation is unbounded. Summing `n` terms whose
    /// denominators are mutually coprime grows the common denominator like their
    /// product, so a few dozen terms is enough to overflow any fixed-width
    /// integer — long before a quadrature sum of thousands of panels finishes.
    ///
    /// A certified sum therefore snaps each partial result back to a fixed
    /// decimal grid, rounding **outward**: [`Rounding::Trunc`] for a value that
    /// bounds from below, [`Rounding::Ceil`] for one that bounds from above. The
    /// accumulator stays bounded and the enclosure stays rigorous, at the cost of
    /// one grid step per snap — which is why callers choose a grid far finer than
    /// the width they are trying to certify.
    ///
    /// This is not a rendering: the result is still an exact [`Ratio`], and Rule R
    /// is untouched. It is a deliberate outward relaxation, and the *only* place
    /// this crate discards information inside a computation rather than at its
    /// edge.
    pub fn snap(&self, digits: u32, mode: Rounding) -> Result<Ratio> {
        let scale = pow10(digits)?;
        let num = mul_div_rounded(&self.num, &scale, &self.den, mode)?;
        Ratio::new(num, scale)
    }

    /// Render to `digits` decimal places under an explicit mode (Rule R).
    ///
    /// This is the only place a `Ratio` becomes inexact, and it is a *rendering*,
    /// never a construction. `UCAL-W0001` territory: the caller is told the mode
    /// because it had to choose one.
    #[cfg(feature = "alloc")]
    pub fn to_decimal_string(&self, digits: u32, mode: Rounding) -> Result<String> {
        use alloc::format;
        let scale = pow10(digits)?;
        let scaled = mul_div_rounded(&self.num, &scale, &self.den, mode)?;
        let s = scaled.to_dec_string();
        if digits == 0 {
            return Ok(s);
        }
        let d = digits as usize;
        Ok(if s.len() <= d {
            format!("0.{}{}", "0".repeat(d - s.len()), s)
        } else {
            format!("{}.{}", &s[..s.len() - d], &s[s.len() - d..])
        })
    }

    /// Whether this rational's decimal expansion **terminates within `digits`**.
    ///
    /// True when the printed digits *are* the value and false when they are a
    /// rounding of something longer. That is the difference between the two
    /// numeric columns of `ucal ladder`, and until 0.4.0 the output rendered
    /// them identically: a tier in beats is a whole power of five and terminates
    /// at once, while the same tier in bridge seconds carries
    /// `18 548 584 399 861` in its denominator, which is neither a power of two
    /// nor of five and therefore never terminates.
    ///
    /// Decided by arithmetic rather than by inspecting the denominator's
    /// factors: the expansion fits in `digits` exactly when `n × 10^digits` is
    /// divisible by `d`.
    pub fn terminates_at(&self, digits: u32) -> Result<bool> {
        let scale = pow10(digits)?;
        let scaled = self
            .num
            .try_mul(&scale)
            .ok_or_else(|| TimeError::new(Code::E0021))?;
        Ok(scaled.quot_rem(&self.den).1.is_zero_ticks())
    }

    /// Render as `numerator/denominator`, which is always exact.
    #[cfg(feature = "alloc")]
    pub fn to_ratio_string(&self) -> String {
        use alloc::format;
        format!("{}/{}", self.num.to_dec_string(), self.den.to_dec_string())
    }
}

impl PartialOrd for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp_exact(other))
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        self.cmp_exact(other)
    }
}

fn overflow() -> TimeError {
    TimeError::with_context(Code::E0021, "exact rational arithmetic left the domain")
}

fn pow10(e: u32) -> Result<Ticks> {
    let ten = <Ticks as TickInt>::from_u64(10);
    let mut acc = <Ticks as TickInt>::one();
    for _ in 0..e {
        acc = acc.try_mul(&ten).ok_or(overflow())?;
    }
    Ok(acc)
}

#[cfg(feature = "alloc")]
fn heapless_concat(a: &str, b: &str) -> String {
    let mut s = String::with_capacity(a.len() + b.len());
    s.push_str(a);
    s.push_str(b);
    s
}

#[cfg(not(feature = "alloc"))]
fn heapless_concat(_a: &str, _b: &str) -> &'static str {
    unreachable!("decimal parsing requires alloc")
}

// ---------------------------------------------------------------------------
// H.3 — interval arithmetic
// ---------------------------------------------------------------------------

/// A closed interval of exact rationals, `lo <= hi` (Appendix H.3).
///
/// Because the endpoints are *exact* rationals, `+`, `-` and `x` on intervals are
/// themselves exact — there is nothing to round outward. Outward rounding enters
/// at exactly two places, and both are explicit:
///
/// - [`RatInterval::sqrt_enclosure`], which uses the directed integer roots;
/// - fixed-point scaling, where the scale is declared and recorded.
///
/// That is a stronger position than a float interval library can take, and it is
/// what lets Rule X promise a *certified* enclosure rather than a tolerance.
#[cfg_attr(feature = "u512", derive(Copy))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RatInterval {
    lo: Ratio,
    hi: Ratio,
}

impl RatInterval {
    /// Construct, rejecting inversion with `UCAL-E0022`.
    pub fn new(lo: Ratio, hi: Ratio) -> Result<RatInterval> {
        if lo.cmp_exact(&hi) == Ordering::Greater {
            return Err(TimeError::new(Code::E0022));
        }
        Ok(RatInterval { lo, hi })
    }

    /// The degenerate interval at an exact value.
    pub fn exact(v: Ratio) -> RatInterval {
        RatInterval {
            lo: v.clone(),
            hi: v,
        }
    }

    /// Lower bound.
    pub fn lo(&self) -> &Ratio {
        &self.lo
    }

    /// Upper bound.
    pub fn hi(&self) -> &Ratio {
        &self.hi
    }

    /// Whether the interval is a single point.
    pub fn is_exact(&self) -> bool {
        self.lo == self.hi
    }

    /// `hi - lo`.
    pub fn width(&self) -> Result<Ratio> {
        self.hi.sub(&self.lo)
    }

    /// Whether the interval contains a value.
    pub fn contains(&self, v: &Ratio) -> bool {
        self.lo.cmp_exact(v) != Ordering::Greater && v.cmp_exact(&self.hi) != Ordering::Greater
    }

    /// Whether zero is inside the interval. Division requires this to be false
    /// (Appendix H.3).
    pub fn contains_zero(&self) -> bool {
        self.lo.is_zero()
    }

    /// Interval addition. Exact: `lo` with `lo`, `hi` with `hi`.
    pub fn add(&self, other: &RatInterval) -> Result<RatInterval> {
        RatInterval::new(self.lo.add(&other.lo)?, self.hi.add(&other.hi)?)
    }

    /// Interval subtraction, `[lo - other.hi, hi - other.lo]`.
    pub fn sub(&self, other: &RatInterval) -> Result<RatInterval> {
        RatInterval::new(self.lo.sub(&other.hi)?, self.hi.sub(&other.lo)?)
    }

    /// Interval multiplication: the min and max of the four endpoint products.
    ///
    /// With non-negative endpoints this reduces to `lo*lo` and `hi*hi`, but the
    /// general form is computed anyway so the method stays correct if the
    /// endpoint domain is ever widened.
    pub fn mul(&self, other: &RatInterval) -> Result<RatInterval> {
        let c = [
            self.lo.mul(&other.lo)?,
            self.lo.mul(&other.hi)?,
            self.hi.mul(&other.lo)?,
            self.hi.mul(&other.hi)?,
        ];
        let mut lo = c[0].clone();
        let mut hi = c[0].clone();
        for v in &c[1..] {
            if v.cmp_exact(&lo) == Ordering::Less {
                lo = v.clone();
            }
            if v.cmp_exact(&hi) == Ordering::Greater {
                hi = v.clone();
            }
        }
        RatInterval::new(lo, hi)
    }

    /// Interval division. `UCAL-E0070` if the divisor interval contains zero.
    pub fn div(&self, other: &RatInterval) -> Result<RatInterval> {
        if other.contains_zero() {
            return Err(TimeError::with_context(
                Code::E0070,
                "interval division by an interval containing zero",
            ));
        }
        let inv = RatInterval::new(other.hi.recip()?, other.lo.recip()?)?;
        self.mul(&inv)
    }

    /// A certified enclosure of the square root, at a declared fixed-point scale
    /// (Appendix H.2).
    ///
    /// Computes `[isqrt_floor(lo x S^2) / S, isqrt_ceil(hi x S^2) / S]`, which
    /// provably contains the true root of every value in the interval. The scale
    /// is a parameter rather than a constant because different queries need
    /// different precision (D-6), and it is returned alongside the result so it
    /// can be recorded — `CosmoResult` carries it as `scale`.
    ///
    /// Widening the interval is always sound; narrowing it would not be. Every
    /// rounding here goes outward.
    pub fn sqrt_enclosure(&self, scale_digits: u32) -> Result<(RatInterval, u32)> {
        let s = pow10(scale_digits)?;
        let s2 = s.try_mul(&s).ok_or(overflow())?;

        // lo: floor(sqrt(lo.num * S^2 / lo.den)) / S  — rounds down, widening left
        let (lo_scaled, _) = mul_div(self.lo.numer(), &s2, self.lo.denom())?;
        let lo_root = isqrt_floor(&lo_scaled);

        // hi: ceil(sqrt(hi.num * S^2 / hi.den)) / S — rounds up, widening right.
        // The dividend is rounded up too, so the enclosure cannot be too narrow.
        let (hi_q, hi_r) = mul_div(self.hi.numer(), &s2, self.hi.denom())?;
        let hi_scaled = if hi_r.is_zero_ticks() {
            hi_q
        } else {
            hi_q.try_add(&<Ticks as TickInt>::one()).ok_or(overflow())?
        };
        let hi_root = isqrt_ceil(&hi_scaled);

        Ok((
            RatInterval::new(Ratio::new(lo_root, s.clone())?, Ratio::new(hi_root, s)?)?,
            scale_digits,
        ))
    }

    /// The smallest interval containing both.
    pub fn hull(&self, other: &RatInterval) -> RatInterval {
        RatInterval {
            lo: if self.lo.cmp_exact(&other.lo) == Ordering::Less {
                self.lo.clone()
            } else {
                other.lo.clone()
            },
            hi: if self.hi.cmp_exact(&other.hi) == Ordering::Greater {
                self.hi.clone()
            } else {
                other.hi.clone()
            },
        }
    }
}

// ---------------------------------------------------------------------------
// H.5 — continued fractions
// ---------------------------------------------------------------------------

/// The continued-fraction expansion of an exact rational.
///
/// Returns the **full** sequence, not just the terms up to some selected
/// convergent: §15.2 requires every derivation to be auditable end to end, and a
/// caller that only sees the chosen answer cannot check the choice.
///
/// Terminates naturally when the remainder reaches zero, or at `max_depth`.
#[cfg(feature = "alloc")]
pub fn cf_expand(r: &Ratio, max_depth: u32) -> Vec<u64> {
    let mut out = Vec::new();
    let mut n = r.numer().clone();
    let mut d = r.denom().clone();
    for _ in 0..max_depth {
        let (a, rem) = n.quot_rem(&d);
        // A continued-fraction term can exceed u64 only for a ratio whose terms
        // are astronomically lopsided; the derivations this serves never are, and
        // saturating here would silently corrupt the expansion.
        let a_small = a.to_dec_string().parse::<u64>();
        match a_small {
            Ok(v) => out.push(v),
            Err(_) => break,
        }
        if rem.is_zero_ticks() {
            break;
        }
        n = d;
        d = rem;
    }
    out
}

/// The convergents of a continued fraction, in order.
///
/// `h[k] = a[k]*h[k-1] + h[k-2]`, `k[k] = a[k]*k[k-1] + k[k-2]`, exact throughout.
/// Stops early if a convergent would leave the domain rather than wrapping.
#[cfg(feature = "alloc")]
pub fn convergents(cf: &[u64]) -> Vec<Ratio> {
    let one = <Ticks as TickInt>::one();
    let zero = <Ticks as TickInt>::zero();
    let (mut hm1, mut hm2) = (one.clone(), zero.clone());
    let (mut km1, mut km2) = (zero, one);
    let mut out = Vec::with_capacity(cf.len());
    for &a in cf {
        let a = <Ticks as TickInt>::from_u64(a);
        let Some(h) = a.try_mul(&hm1).and_then(|v| v.try_add(&hm2)) else {
            break;
        };
        let Some(k) = a.try_mul(&km1).and_then(|v| v.try_add(&km2)) else {
            break;
        };
        let Ok(c) = Ratio::new(h.clone(), k.clone()) else {
            break;
        };
        out.push(c);
        hm2 = hm1;
        hm1 = h;
        km2 = km1;
        km1 = k;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn t(n: u64) -> Ticks {
        <Ticks as TickInt>::from_u64(n)
    }
    fn r(n: u64, d: u64) -> Ratio {
        Ratio::new(t(n), t(d)).unwrap()
    }

    // ---- H.1 ----

    #[test]
    fn mul_div_is_exact_and_returns_the_remainder() {
        assert_eq!(mul_div(&t(7), &t(5), &t(2)).unwrap(), (t(17), t(1)));
        assert_eq!(mul_div(&t(10), &t(10), &t(5)).unwrap(), (t(20), t(0)));
        assert_eq!(
            mul_div(&t(1), &t(1), &t(0)).unwrap_err().code,
            Code::E0070
        );
    }

    #[test]
    fn mul_div_intermediate_cannot_overflow() {
        // The whole point of Appendix H.1: a product that exceeds the domain is
        // fine as long as the quotient does not. domain_max^2 / domain_max is
        // exactly domain_max, and a 512-bit backend cannot hold the intermediate.
        let m = <Ticks as TickInt>::domain_max();
        assert!(m.try_mul(&m).is_none(), "the product must not fit");
        let (q, rem) = mul_div(&m, &m, &m).unwrap();
        assert_eq!(q, m);
        assert!(rem.is_zero_ticks());
    }

    #[test]
    fn mul_div_reports_a_quotient_that_leaves_the_domain() {
        let m = <Ticks as TickInt>::domain_max();
        assert_eq!(
            mul_div(&m, &t(2), &t(1)).unwrap_err().code,
            Code::E0021
        );
    }

    #[test]
    fn mul_div_rounded_honours_the_mode() {
        // 7*1/2 = 3.5 exactly — a tie, so the modes separate.
        assert_eq!(mul_div_rounded(&t(7), &t(1), &t(2), Rounding::Trunc).unwrap(), t(3));
        assert_eq!(mul_div_rounded(&t(7), &t(1), &t(2), Rounding::Ceil).unwrap(), t(4));
        assert_eq!(mul_div_rounded(&t(7), &t(1), &t(2), Rounding::HalfUp).unwrap(), t(4));
        // half-even: quotient 3 is odd, so it rounds up to 4
        assert_eq!(mul_div_rounded(&t(7), &t(1), &t(2), Rounding::HalfEven).unwrap(), t(4));
        // 5/2 = 2.5, quotient 2 is even, so half-even stays at 2
        assert_eq!(mul_div_rounded(&t(5), &t(1), &t(2), Rounding::HalfEven).unwrap(), t(2));
        assert_eq!(mul_div_rounded(&t(5), &t(1), &t(2), Rounding::HalfUp).unwrap(), t(3));
    }

    // ---- H.2 ----

    #[test]
    fn isqrt_post_conditions_hold() {
        for n in 0..200u64 {
            let x = t(n);
            let f = isqrt_floor(&x);
            let one = <Ticks as TickInt>::one();
            // r^2 <= x < (r+1)^2
            assert!(f.wide_mul(&f) <= x.wide_mul(&one), "floor^2 > x at {n}");
            let fp1 = f.try_add(&one).unwrap();
            assert!(fp1.wide_mul(&fp1) > x.wide_mul(&one), "x >= (floor+1)^2 at {n}");

            let c = isqrt_ceil(&x);
            assert!(c.wide_mul(&c) >= x.wide_mul(&one), "ceil^2 < x at {n}");
            if f.wide_mul(&f) == x.wide_mul(&one) {
                assert_eq!(c, f, "a perfect square must have equal floor and ceil");
            } else {
                assert_eq!(c, fp1);
            }
        }
    }

    #[test]
    fn isqrt_on_perfect_squares_and_large_values() {
        for n in [0u64, 1, 4, 9, 16, 10_000, 1u64 << 40] {
            let sq = t(n).try_mul(&t(n)).unwrap();
            assert_eq!(isqrt_floor(&sq), t(n));
            assert_eq!(isqrt_ceil(&sq), t(n));
        }
        // Near the top of the domain.
        let m = <Ticks as TickInt>::domain_max();
        let f = isqrt_floor(&m);
        let one = <Ticks as TickInt>::one();
        assert!(f.wide_mul(&f) <= m.wide_mul(&one));
        let fp1 = f.try_add(&one).unwrap();
        assert!(fp1.wide_mul(&fp1) > m.wide_mul(&one));
        // 2^512-1 has a root of 2^256-1.
        assert_eq!(f.bit_len(), 256);
    }

    // ---- rationals ----

    #[test]
    fn rationals_reduce_and_compare_exactly() {
        assert_eq!(r(2, 4), r(1, 2));
        assert_eq!(r(6, 3), Ratio::from_u64(2));
        assert!(r(1, 3).cmp_exact(&r(1, 2)) == Ordering::Less);
        assert!(r(2, 3).cmp_exact(&r(1, 2)) == Ordering::Greater);
        assert_eq!(r(3, 6).cmp_exact(&r(1, 2)), Ordering::Equal);
        assert_eq!(Ratio::new(t(1), t(0)).unwrap_err().code, Code::E0070);
    }

    #[test]
    fn rational_comparison_cannot_overflow() {
        // Cross-multiplication of two near-domain-max rationals overflows a
        // narrow product but must still compare correctly.
        let m = <Ticks as TickInt>::domain_max();
        let a = Ratio::new(m.clone(), m.clone()).unwrap(); // == 1
        let b = Ratio::new(m.clone(), <Ticks as TickInt>::one()).unwrap();
        assert_eq!(a, Ratio::one());
        assert_eq!(a.cmp_exact(&b), Ordering::Less);
    }

    #[test]
    fn rational_arithmetic_is_exact() {
        assert_eq!(r(1, 2).add(&r(1, 3)).unwrap(), r(5, 6));
        assert_eq!(r(1, 2).sub(&r(1, 3)).unwrap(), r(1, 6));
        assert_eq!(r(2, 3).mul(&r(3, 4)).unwrap(), r(1, 2));
        assert_eq!(r(2, 3).div(&r(4, 9)).unwrap(), r(3, 2));
        assert_eq!(r(1, 3).abs_diff(&r(1, 2)).unwrap(), r(1, 6));
        assert_eq!(r(1, 2).abs_diff(&r(1, 3)).unwrap(), r(1, 6));
        // Rule Z for rationals: a negative result is an error, not a sign.
        assert_eq!(r(1, 3).sub(&r(1, 2)).unwrap_err().code, Code::E0020);
        assert_eq!(r(1, 2).div(&Ratio::zero()).unwrap_err().code, Code::E0070);
        assert_eq!(Ratio::zero().recip().unwrap_err().code, Code::E0070);
    }

    #[test]
    fn decimal_parsing_is_exact() {
        // §10.2: 1100, 1089.80 and 0.5 must all be exact.
        assert_eq!(Ratio::from_decimal_str("1100").unwrap(), Ratio::from_u64(1100));
        assert_eq!(Ratio::from_decimal_str("0.5").unwrap(), r(1, 2));
        assert_eq!(Ratio::from_decimal_str("1089.80").unwrap(), r(10898, 10));
        let earth = Ratio::from_decimal_str("365.242190").unwrap();
        assert_eq!(earth.to_ratio_string(), "36524219/100000");
        assert_eq!(earth.floor(), t(365));
        assert_eq!(earth.frac(), r(24219, 100000));
        for bad in ["", "1.2.3", "abc", "1e5", "-1", "1.2f"] {
            assert!(Ratio::from_decimal_str(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn decimal_rendering_states_its_mode() {
        let third = r(1, 3);
        assert_eq!(third.to_decimal_string(5, Rounding::Trunc).unwrap(), "0.33333");
        assert_eq!(third.to_decimal_string(5, Rounding::Ceil).unwrap(), "0.33334");
        let two_thirds = r(2, 3);
        assert_eq!(two_thirds.to_decimal_string(5, Rounding::Trunc).unwrap(), "0.66666");
        assert_eq!(two_thirds.to_decimal_string(5, Rounding::HalfEven).unwrap(), "0.66667");
        // Values below 1 keep their leading zero and pad correctly.
        assert_eq!(r(1, 100).to_decimal_string(4, Rounding::Trunc).unwrap(), "0.0100");
        assert_eq!(Ratio::from_u64(7).to_decimal_string(0, Rounding::Trunc).unwrap(), "7");
    }

    // ---- H.3 ----

    #[test]
    fn interval_arithmetic_widens_and_rejects_inversion() {
        let a = RatInterval::new(r(1, 2), r(3, 2)).unwrap();
        let b = RatInterval::new(r(1, 4), r(1, 2)).unwrap();
        let s = a.add(&b).unwrap();
        assert_eq!(s.lo(), &r(3, 4));
        assert_eq!(s.hi(), &Ratio::from_u64(2));
        let p = a.mul(&b).unwrap();
        assert_eq!(p.lo(), &r(1, 8));
        assert_eq!(p.hi(), &r(3, 4));
        assert!(a.contains(&Ratio::one()));
        assert!(!a.contains(&Ratio::from_u64(2)));
        assert_eq!(
            RatInterval::new(r(3, 2), r(1, 2)).unwrap_err().code,
            Code::E0022
        );
    }

    #[test]
    fn interval_division_rejects_a_divisor_spanning_zero() {
        let a = RatInterval::new(r(1, 2), r(3, 2)).unwrap();
        let spans_zero = RatInterval::new(Ratio::zero(), Ratio::one()).unwrap();
        assert_eq!(a.div(&spans_zero).unwrap_err().code, Code::E0070);
        let ok = RatInterval::new(r(1, 2), Ratio::one()).unwrap();
        let q = a.div(&ok).unwrap();
        assert_eq!(q.lo(), &r(1, 2));
        assert_eq!(q.hi(), &Ratio::from_u64(3));
    }

    #[test]
    fn sqrt_enclosure_contains_the_true_root_and_narrows_with_scale() {
        // sqrt(2) is irrational, so the enclosure must be strict and must bracket.
        let two = RatInterval::exact(Ratio::from_u64(2));
        let mut prev_width: Option<Ratio> = None;
        for digits in [3u32, 6, 9, 12] {
            let (e, s) = two.sqrt_enclosure(digits).unwrap();
            assert_eq!(s, digits);
            // lo^2 <= 2 <= hi^2
            assert!(e.lo().mul(e.lo()).unwrap().cmp_exact(&Ratio::from_u64(2)) != Ordering::Greater);
            assert!(e.hi().mul(e.hi()).unwrap().cmp_exact(&Ratio::from_u64(2)) != Ordering::Less);
            let w = e.width().unwrap();
            if let Some(p) = prev_width {
                assert!(w.cmp_exact(&p) == Ordering::Less, "enclosure did not narrow");
            }
            prev_width = Some(w);
        }
        // A perfect square encloses exactly.
        let four = RatInterval::exact(Ratio::from_u64(4));
        let (e, _) = four.sqrt_enclosure(6).unwrap();
        assert!(e.contains(&Ratio::from_u64(2)));
    }

    // ---- H.5: Appendix I reproduction, the UC-P3 exit criterion ----

    #[test]
    fn appendix_i1_earth_intercalation() {
        let ratio = Ratio::from_decimal_str("365.242190").unwrap();
        let frac = ratio.frac();
        let cf = cf_expand(&frac, 32);
        assert_eq!(cf[..9], [0, 4, 7, 1, 3, 24, 6, 2, 2]);

        let cv = convergents(&cf);
        let want = [(1u64, 4u64), (7, 29), (8, 33), (31, 128), (752, 3105), (4543, 18758)];
        for (i, (n, d)) in want.iter().enumerate() {
            assert_eq!(cv[i + 1], r(*n, *d), "convergent {}", i + 1);
        }

        // §21.3-6: 1/4 (the Julian rule) is convergent 1...
        assert_eq!(cv[1], r(1, 4));
        // ...and 97/400 (Gregorian) appears at NO depth.
        let gregorian = r(97, 400);
        assert!(
            !cv.iter().any(|c| *c == gregorian),
            "97/400 must not appear as a convergent at any depth"
        );

        // The RFC's two accuracy claims about 97/400, verified exactly.
        let e_greg = gregorian.abs_diff(&frac).unwrap();
        let e_8_33 = r(8, 33).abs_diff(&frac).unwrap();
        let e_31_128 = r(31, 128).abs_diff(&frac).unwrap();
        assert!(
            e_8_33.cmp_exact(&e_greg) == Ordering::Less,
            "8/33 must be more accurate than 97/400, with a denominator 12x smaller"
        );
        assert!(e_31_128.cmp_exact(&e_greg) == Ordering::Less);
        // 31/128 is ~124x more accurate.
        let times = e_greg.div(&e_31_128).unwrap();
        assert_eq!(times.to_decimal_string(1, Rounding::Trunc).unwrap(), "124.0");
    }

    #[test]
    fn appendix_i2_earth_grouping_derives_the_metonic_cycle() {
        let ratio = Ratio::from_decimal_str("12.368266761").unwrap();
        let cf = cf_expand(&ratio, 32);
        assert_eq!(cf[..9], [12, 2, 1, 2, 1, 1, 17, 2, 1]);
        let cv = convergents(&cf);
        let want = [(12u64, 1u64), (25, 2), (37, 3), (99, 8), (136, 11), (235, 19), (4131, 334)];
        for (i, (n, d)) in want.iter().enumerate() {
            assert_eq!(cv[i], r(*n, *d), "convergent {}", i + 1);
        }
        // §21.3-7: the Metonic cycle falls out with no special-casing.
        assert!(cv.iter().any(|c| *c == r(235, 19)));
    }

    #[test]
    fn appendix_i3_mars_intercalation() {
        let frac = Ratio::from_decimal_str("668.592165627").unwrap().frac();
        let cf = cf_expand(&frac, 32);
        assert_eq!(cf[..9], [0, 1, 1, 2, 4, 1, 2, 2, 1]);
        let cv = convergents(&cf);
        let want = [(1u64, 1u64), (1, 2), (3, 5), (13, 22), (16, 27), (45, 76), (106, 179)];
        for (i, (n, d)) in want.iter().enumerate() {
            assert_eq!(cv[i + 1], r(*n, *d), "convergent {}", i + 1);
        }
    }

    #[test]
    fn appendix_i5_titan_intercalation() {
        let frac = Ratio::from_decimal_str("673.983719443").unwrap().frac();
        let cf = cf_expand(&frac, 32);
        assert_eq!(cf[..9], [0, 1, 60, 2, 2, 1, 2, 1, 11]);
        let cv = convergents(&cf);
        let want = [(1u64, 1u64), (60, 61), (121, 123), (302, 307), (423, 430)];
        for (i, (n, d)) in want.iter().enumerate() {
            assert_eq!(cv[i + 1], r(*n, *d), "convergent {}", i + 1);
        }
    }

    #[test]
    fn convergents_alternate_around_the_value_and_improve() {
        // A structural property of continued fractions, and a good check that the
        // recurrence is right: successive convergents must strictly improve.
        let frac = Ratio::from_decimal_str("365.242190").unwrap().frac();
        let cv = convergents(&cf_expand(&frac, 32));
        let mut prev: Option<Ratio> = None;
        for c in cv.iter().skip(1) {
            let e = c.abs_diff(&frac).unwrap();
            if let Some(p) = prev {
                assert!(
                    e.cmp_exact(&p) == Ordering::Less,
                    "convergent {} did not improve",
                    c.to_ratio_string()
                );
            }
            prev = Some(e);
        }
        // The last convergent of an exact decimal reproduces it exactly.
        assert_eq!(cv.last().unwrap(), &frac);
    }

    #[test]
    fn cf_expand_returns_the_full_sequence() {
        // §15.2: derivations must be auditable, so the whole walk is returned.
        let cf = cf_expand(&r(649, 200), 32);
        let cv = convergents(&cf);
        assert_eq!(cv.last().unwrap(), &r(649, 200));
        // A whole number has a one-term expansion.
        assert_eq!(cf_expand(&Ratio::from_u64(7), 32), vec![7]);
        // max_depth truncates rather than looping.
        assert_eq!(cf_expand(&frac_of("365.242190"), 3).len(), 3);
    }

    fn frac_of(s: &str) -> Ratio {
        Ratio::from_decimal_str(s).unwrap().frac()
    }
}
