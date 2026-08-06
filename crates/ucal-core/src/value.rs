//! The value types (§5): `Instant`, `Delta`, `Signed`, `SignedWindow`, `Window`,
//! `Precision`, `Rounding`.
//!
//! Three rules shape this module and are worth stating before the code:
//!
//! - **Rule Z / Rule O.** The domain is unsigned and closed. `Sub` is *not*
//!   implemented on `Instant`; subtraction is [`Instant::since`], which fails
//!   with `UCAL-E0020`, or [`Instant::between`], which returns a [`Signed`].
//!   Nothing wraps and nothing saturates.
//! - **Rule T.** A value stated to tier precision denotes a closed interval.
//!   [`Instant::window_at`] materialises it, and no parse path may hand back a
//!   bare tick-precision `Instant` from truncated input.
//! - **Rule Q.3.** [`SignedWindow`] is metadata. It has no arithmetic operators
//!   and no conversion into [`Delta`], [`Instant`] or [`Window`]. §21.3-3 requires
//!   a compile-fail test proving that lifting the restriction breaks the build.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
use core::cmp::Ordering;
use core::marker::PhantomData;

use crate::backend::{TickInt, Ticks, CANONICAL_BYTES};
use crate::error::{Code, Result, TimeError};
use crate::profile::Profile;
use crate::tier::{Tier, GROUP_BASE};

/// How to round when rendering an exact internal value into a coarser form.
///
/// Rule R: rounding exists **only** when rendering. No API that constructs
/// absolute time may round — construction from a foreign unit is exact or it is
/// `UCAL-E0043`.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub enum Rounding {
    /// Toward the datum.
    Trunc,
    /// Away from the datum.
    Ceil,
    /// Nearest, ties to even. The default, and the mode §2.2 uses for the datum.
    #[default]
    HalfEven,
    /// Nearest, ties away from the datum.
    HalfUp,
}

/// The precision at which a value is stated.
///
/// D-13 makes this a runtime field rather than a type parameter: type-level
/// precision would infect every signature for little safety gain over Rule T.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Precision {
    /// Exact to one tick. The finest addressable precision (G2).
    Tick,
    /// Stated to a tier, denoting the closed interval `[v, v + 5^e - 1]`.
    Tier(Tier),
}

impl Precision {
    /// The tier this precision corresponds to.
    pub fn tier(self) -> Tier {
        match self {
            Precision::Tick => Tier::TICK,
            Precision::Tier(t) => t,
        }
    }

    /// Whether this precision denotes a single tick.
    pub fn is_exact(self) -> bool {
        matches!(self, Precision::Tick) || self.tier().is_tick()
    }
}

/// Sign of a [`Signed`] magnitude.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sign {
    /// Zero or forward in time.
    Positive,
    /// Backward in time.
    Negative,
}

// ---------------------------------------------------------------------------
// Delta — an unsigned magnitude in ticks
// ---------------------------------------------------------------------------

/// An unsigned magnitude in ticks. Not tied to a profile: a count of ticks means
/// the same thing under any profile that shares the tick (§5).
#[cfg_attr(feature = "u512", derive(Copy))]
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug, Default)]
pub struct Delta {
    ticks: Ticks,
}

impl Delta {
    /// Zero ticks.
    pub fn zero() -> Self {
        Delta {
            ticks: <Ticks as TickInt>::zero(),
        }
    }

    /// One tick — the finest representable interval (G2, N10).
    pub fn one_tick() -> Self {
        Delta {
            ticks: <Ticks as TickInt>::one(),
        }
    }

    /// From a raw tick count.
    pub fn from_ticks(ticks: Ticks) -> Self {
        Delta { ticks }
    }

    /// From a small tick count.
    pub fn from_u64(v: u64) -> Self {
        Delta {
            ticks: <Ticks as TickInt>::from_u64(v),
        }
    }

    /// Whole multiples of a tier: `count x 5^e`.
    pub fn from_tier(tier: Tier, count: u64) -> Result<Self> {
        let t = tier.ticks();
        let c = <Ticks as TickInt>::from_u64(count);
        t.try_mul(&c)
            .map(|ticks| Delta { ticks })
            .ok_or(TimeError::new(Code::E0021))
    }

    /// The raw tick count.
    pub fn ticks(&self) -> &Ticks {
        &self.ticks
    }

    /// Whether this is zero ticks.
    pub fn is_zero(&self) -> bool {
        self.ticks.is_zero_ticks()
    }

    /// `self + other`, failing on domain exit (Rule O).
    pub fn checked_add(&self, other: &Delta) -> Result<Delta> {
        self.ticks
            .try_add(&other.ticks)
            .map(|ticks| Delta { ticks })
            .ok_or(TimeError::new(Code::E0021))
    }

    /// `self - other`, failing rather than wrapping (Rules Z, O).
    pub fn checked_sub(&self, other: &Delta) -> Result<Delta> {
        self.ticks
            .try_sub(&other.ticks)
            .map(|ticks| Delta { ticks })
            .ok_or(TimeError::new(Code::E0020))
    }

    /// `self x n`, failing on domain exit.
    pub fn mul_u64(&self, n: u64) -> Result<Delta> {
        self.ticks
            .try_mul(&<Ticks as TickInt>::from_u64(n))
            .map(|ticks| Delta { ticks })
            .ok_or(TimeError::new(Code::E0021))
    }

    /// Truncating `self / n`. Panics only if `n` is zero.
    pub fn div_u64(&self, n: u64) -> Result<Delta> {
        if n == 0 {
            return Err(TimeError::with_context(Code::E0021, "division by zero"));
        }
        let (q, _) = self.ticks.quot_rem(&<Ticks as TickInt>::from_u64(n));
        Ok(Delta { ticks: q })
    }

    /// Quotient and remainder against another magnitude.
    pub fn divmod(&self, divisor: &Delta) -> Result<(Delta, Delta)> {
        if divisor.is_zero() {
            return Err(TimeError::with_context(Code::E0021, "division by zero"));
        }
        let (q, r) = self.ticks.quot_rem(&divisor.ticks);
        Ok((Delta { ticks: q }, Delta { ticks: r }))
    }

    /// The largest tier no greater than `self`, or `None` if `self` is zero or
    /// smaller than one tick (which cannot happen — one tick is the floor).
    pub fn tier_of(&self) -> Option<Tier> {
        if self.is_zero() {
            return None;
        }
        Tier::all_descending().find(|t| t.ticks() <= self.ticks)
    }

    /// Whole count of a tier contained in `self`, and the remainder in ticks.
    pub fn in_tier(&self, tier: Tier) -> (Ticks, Ticks) {
        self.ticks.quot_rem(&tier.ticks())
    }
}

// ---------------------------------------------------------------------------
// Signed — a difference, which may be negative
// ---------------------------------------------------------------------------

/// A signed difference between two instants. The domain itself is unsigned
/// (Rule Z, N12), so this type exists to let [`Instant::between`] be total.
#[cfg_attr(feature = "u512", derive(Copy))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Signed {
    sign: Sign,
    mag: Delta,
}

impl Signed {
    /// Construct from a sign and magnitude. A zero magnitude is normalised to
    /// positive, so `Signed` has no negative zero.
    pub fn new(sign: Sign, mag: Delta) -> Self {
        if mag.is_zero() {
            Signed {
                sign: Sign::Positive,
                mag,
            }
        } else {
            Signed { sign, mag }
        }
    }

    /// Zero.
    pub fn zero() -> Self {
        Signed::new(Sign::Positive, Delta::zero())
    }

    /// The sign.
    pub fn sign(&self) -> Sign {
        self.sign
    }

    /// The magnitude.
    pub fn magnitude(&self) -> &Delta {
        &self.mag
    }

    /// Whether this is zero.
    pub fn is_zero(&self) -> bool {
        self.mag.is_zero()
    }

    /// The magnitude, discarding the sign. Named so that dropping the sign is a
    /// visible act at the call site.
    pub fn into_magnitude(self) -> Delta {
        self.mag
    }
}

impl PartialOrd for Signed {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Signed {
    fn cmp(&self, other: &Self) -> Ordering {
        match (self.sign, other.sign) {
            (Sign::Positive, Sign::Positive) => self.mag.cmp(&other.mag),
            (Sign::Negative, Sign::Negative) => other.mag.cmp(&self.mag),
            (Sign::Positive, Sign::Negative) => Ordering::Greater,
            (Sign::Negative, Sign::Positive) => Ordering::Less,
        }
    }
}

// ---------------------------------------------------------------------------
// SignedWindow — metadata only (Rule Q.3)
// ---------------------------------------------------------------------------

/// A signed interval, used **only** for profile metadata such as
/// `BIG_BANG_CLAIM` (Rule Q.3).
///
/// The window is signed because the FLRW t→0 limit may lie *before* the datum,
/// which is not representable as a tick (N12).
///
/// This type deliberately has:
///
/// - no arithmetic operators of any kind,
/// - no `From<SignedWindow>` for [`Delta`], [`Instant`] or [`Window`],
/// - no method returning any of those types.
///
/// It cannot be added to an `Instant` and cannot be widened into one. Attempting
/// to use it as an operand is `UCAL-E0025`, and the type system is what makes
/// that unreachable rather than a runtime check. §21.3-3 requires a compile-fail
/// test; see `tests/compile_fail/`.
#[cfg_attr(feature = "u512", derive(Copy))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SignedWindow {
    lo: Signed,
    hi: Signed,
}

impl SignedWindow {
    /// Construct from bounds. Fails with `UCAL-E0022` if inverted.
    pub fn new(lo: Signed, hi: Signed) -> Result<Self> {
        if lo > hi {
            return Err(TimeError::new(Code::E0022));
        }
        Ok(SignedWindow { lo, hi })
    }

    /// A symmetric window `+/- half_width` about zero. D-16: the half-width form
    /// matches how the Planck 2018 uncertainty is published, without foreclosing
    /// asymmetric windows, which [`SignedWindow::new`] still permits.
    pub fn symmetric(half_width: Delta) -> Self {
        SignedWindow {
            lo: Signed::new(Sign::Negative, half_width.clone()),
            hi: Signed::new(Sign::Positive, half_width),
        }
    }

    /// The lower bound. Returns a [`Signed`], which is itself inert.
    pub fn lo(&self) -> &Signed {
        &self.lo
    }

    /// The upper bound.
    pub fn hi(&self) -> &Signed {
        &self.hi
    }

    /// Render for reporting. This is the only intended consumer: `ucal datum`,
    /// `ucal doctor`, `ucal explain --claim`.
    #[cfg(feature = "alloc")]
    pub fn describe(&self) -> alloc::string::String {
        use alloc::format;
        let s = |v: &Signed| {
            let m = v.magnitude().ticks().to_dec_string();
            match v.sign() {
                Sign::Negative => format!("-{m}"),
                Sign::Positive => m,
            }
        };
        format!("[{}, {}] ticks", s(&self.lo), s(&self.hi))
    }
}

// ---------------------------------------------------------------------------
// Instant
// ---------------------------------------------------------------------------

/// A point in absolute time: an unsigned integer count of ticks since the datum,
/// parameterised by profile at the type level (Rule P).
///
/// Cross-profile arithmetic and comparison do not compile. `Instant<UC1>` and
/// `Instant<UC2>` are distinct types with no coercion between them; conversion is
/// available only through [`Instant::rebase`], which reports the constant shift.
pub struct Instant<P: Profile> {
    ticks: Ticks,
    _p: PhantomData<P>,
}

// These are written by hand rather than derived. `derive` would place a bound on
// `P` for each trait, but `P` appears only in `PhantomData` and carries no data,
// so an `Instant` is comparable exactly when its tick count is. Deriving would,
// for instance, demand `P: Ord` — which no profile has any reason to be.
impl<P: Profile> Clone for Instant<P> {
    fn clone(&self) -> Self {
        Instant {
            ticks: self.ticks.clone(),
            _p: PhantomData,
        }
    }
}

#[cfg(feature = "u512")]
impl<P: Profile> Copy for Instant<P> {}

impl<P: Profile> PartialEq for Instant<P> {
    fn eq(&self, other: &Self) -> bool {
        self.ticks == other.ticks
    }
}

impl<P: Profile> Eq for Instant<P> {}

impl<P: Profile> PartialOrd for Instant<P> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Rule M: for instants in one profile, exactly one of `a < b`, `a = b`, `a > b`
/// holds, and it is the chronological order. Rule P makes the "in one profile"
/// part a type-level guarantee rather than a runtime check.
impl<P: Profile> Ord for Instant<P> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.ticks.cmp(&other.ticks)
    }
}

impl<P: Profile> core::hash::Hash for Instant<P> {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.ticks.hash(state);
    }
}

impl<P: Profile> core::fmt::Debug for Instant<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        // The profile tag is part of the identity of the value (Rule P), so it
        // belongs in the debug output.
        write!(f, "Instant<{}>({:?})", P::TAG, self.ticks)
    }
}

/// The datum, as a `const`, on the default backend only.
///
/// §13 specifies `pub const ZERO: Self`. That is achievable when `Ticks` is
/// `const`-constructible, which it is on the default backend and is not on
/// `bigint` (a heap value cannot be a `const`). [`Instant::zero`] is the portable
/// form and is available on both.
#[cfg(feature = "u512")]
impl<P: Profile> Instant<P> {
    /// Tick 0 — the datum (Rule Z, Rule Q).
    pub const ZERO: Self = Instant {
        ticks: Ticks::ZERO,
        _p: PhantomData,
    };
}

impl<P: Profile> Instant<P> {
    /// Tick 0 — the datum. A **stipulated** reference point, conventionally
    /// identified with the FLRW t→0 limit; not a measurement and not an observed
    /// event (Rule Q, N17).
    pub fn zero() -> Self {
        Instant {
            ticks: <Ticks as TickInt>::zero(),
            _p: PhantomData,
        }
    }

    /// Construct from a tick count, rejecting values outside the profile domain.
    pub fn from_ticks(ticks: Ticks) -> Result<Self> {
        if ticks > P::domain_max() {
            return Err(TimeError::new(Code::E0021));
        }
        Ok(Instant {
            ticks,
            _p: PhantomData,
        })
    }

    /// Construct from a small tick count.
    pub fn from_u64(v: u64) -> Result<Self> {
        Self::from_ticks(<Ticks as TickInt>::from_u64(v))
    }

    /// The raw tick count.
    pub fn ticks(&self) -> &Ticks {
        &self.ticks
    }

    /// The decimal group value at a tier, in `0..=3124`.
    pub fn tier_value(&self, tier: Tier) -> u16 {
        let (shifted, _) = self.ticks.quot_rem(&tier.ticks());
        let (_, r) = shifted.quot_rem(&<Ticks as TickInt>::from_u64(GROUP_BASE as u64));
        // r < 3125 fits u16; the conversion goes through the canonical bytes so
        // that it needs no backend-specific cast.
        let b = r.to_canonical_bytes();
        u16::from_be_bytes([b[CANONICAL_BYTES - 2], b[CANONICAL_BYTES - 1]])
    }

    /// Group values for tiers `from` down to `to`, most significant first.
    #[cfg(feature = "alloc")]
    pub fn groups(&self, from: Tier, to: Tier) -> Result<Vec<u16>> {
        if from < to {
            return Err(TimeError::with_context(
                Code::E0006,
                "group range must descend",
            ));
        }
        let mut out = Vec::new();
        let mut k = from.index();
        while k >= to.index() {
            out.push(self.tier_value(Tier::new(k)?));
            k -= 1;
        }
        Ok(out)
    }

    /// Truncate toward the datum. Truncation *is* rounding (G3, Rule G).
    pub fn floor_to(&self, tier: Tier) -> Self {
        let t = tier.ticks();
        let (q, _) = self.ticks.quot_rem(&t);
        let ticks = q
            .try_mul(&t)
            .expect("floor of an in-domain value is in domain");
        Instant {
            ticks,
            _p: PhantomData,
        }
    }

    /// Round away from the datum, failing on domain exit.
    pub fn ceil_to(&self, tier: Tier) -> Result<Self> {
        let t = tier.ticks();
        let (q, r) = self.ticks.quot_rem(&t);
        if r.is_zero_ticks() {
            return Ok(self.clone());
        }
        let next = q
            .try_add(&<Ticks as TickInt>::one())
            .and_then(|n| n.try_mul(&t))
            .ok_or(TimeError::new(Code::E0021))?;
        Self::from_ticks(next)
    }

    /// Round to a tier under an explicit mode (Rule R).
    pub fn round_to(&self, tier: Tier, mode: Rounding) -> Result<Self> {
        let t = tier.ticks();
        let (q, r) = self.ticks.quot_rem(&t);
        if r.is_zero_ticks() {
            return Ok(self.clone());
        }
        let up = match mode {
            Rounding::Trunc => false,
            Rounding::Ceil => true,
            Rounding::HalfUp | Rounding::HalfEven => {
                let twice = r
                    .try_add(&r)
                    .expect("2r < 2 x tier, which is in domain");
                match twice.cmp(&t) {
                    Ordering::Greater => true,
                    Ordering::Less => false,
                    Ordering::Equal => match mode {
                        Rounding::HalfUp => true,
                        // ties to even
                        _ => q.is_odd(),
                    },
                }
            }
        };
        if up {
            self.ceil_to(tier)
        } else {
            Ok(self.floor_to(tier))
        }
    }

    /// The closed interval a value stated at this precision denotes (Rule T).
    ///
    /// For [`Precision::Tick`] this is the degenerate window `[v, v]`. For a tier
    /// it is `[floor, floor + 5^e - 1]` — never a bare instant, which is what
    /// keeps failure mode F2 closed.
    pub fn window_at(&self, precision: Precision) -> Result<Window<P>> {
        match precision {
            Precision::Tick => Window::new(self.clone(), self.clone()),
            Precision::Tier(tier) => {
                if tier.is_tick() {
                    return Window::new(self.clone(), self.clone());
                }
                let lo = self.floor_to(tier);
                let span = tier
                    .ticks()
                    .try_sub(&<Ticks as TickInt>::one())
                    .expect("a tier is at least one tick");
                // Rule T's interval is intersected with the closed domain. Near
                // the ceiling, `floor + 5^e - 1` can run past `domain_max`; since
                // no representable instant lies beyond it, clamping keeps the
                // window a sound enclosure and makes it a tighter one. Failing
                // here instead would mean a legitimate coarse statement about a
                // late instant had no representable meaning.
                let hi_ticks = match lo.ticks.try_add(&span) {
                    Some(t) if t <= P::domain_max() => t,
                    _ => P::domain_max(),
                };
                Window::new(lo, Self::from_ticks(hi_ticks)?)
            }
        }
    }

    /// Elapsed ticks since an earlier instant. `UCAL-E0020` if `earlier` is later
    /// than `self` — Rule Z admits no negative result.
    pub fn since(&self, earlier: &Self) -> Result<Delta> {
        self.ticks
            .try_sub(&earlier.ticks)
            .map(Delta::from_ticks)
            .ok_or(TimeError::new(Code::E0020))
    }

    /// The signed difference `self - other`. Always succeeds.
    pub fn between(&self, other: &Self) -> Signed {
        match self.ticks.cmp(&other.ticks) {
            Ordering::Greater | Ordering::Equal => Signed::new(
                Sign::Positive,
                Delta::from_ticks(
                    self.ticks
                        .try_sub(&other.ticks)
                        .expect("self >= other"),
                ),
            ),
            Ordering::Less => Signed::new(
                Sign::Negative,
                Delta::from_ticks(
                    other
                        .ticks
                        .try_sub(&self.ticks)
                        .expect("other > self"),
                ),
            ),
        }
    }

    /// Advance by a magnitude, failing on domain exit (`UCAL-E0021`).
    pub fn checked_add(&self, d: &Delta) -> Result<Self> {
        let ticks = self
            .ticks
            .try_add(d.ticks())
            .ok_or(TimeError::new(Code::E0021))?;
        Self::from_ticks(ticks)
    }

    /// Retreat by a magnitude, failing before the datum (`UCAL-E0020`).
    pub fn checked_sub(&self, d: &Delta) -> Result<Self> {
        self.ticks
            .try_sub(d.ticks())
            .map(|ticks| Instant {
                ticks,
                _p: PhantomData,
            })
            .ok_or(TimeError::new(Code::E0020))
    }

    /// Canonical binary form: 64 bytes, big-endian, zero-padded (§7.1, Rule B).
    ///
    /// Byte order is chronological order, so the encoding is directly usable as a
    /// database key. Identical on every backend, which is what makes the
    /// cross-backend differential test a conformance test (Rule W).
    pub fn to_bytes(&self) -> [u8; CANONICAL_BYTES] {
        self.ticks.to_canonical_bytes()
    }

    /// Inverse of [`Instant::to_bytes`].
    pub fn from_bytes(bytes: &[u8; CANONICAL_BYTES]) -> Result<Self> {
        let ticks =
            <Ticks as TickInt>::from_canonical_bytes(bytes).ok_or(TimeError::new(Code::E0021))?;
        Self::from_ticks(ticks)
    }

    /// Reinterpret under another profile, reporting the constant tick shift
    /// (D-14, Rule P).
    ///
    /// The shift is `Q::origin_offset() - P::origin_offset()`: both profiles date
    /// from their own datum, so rebasing is a translation. Returns the new instant
    /// and the shift that was applied, because a caller that does not see the
    /// shift cannot audit the conversion.
    pub fn rebase<Q: Profile>(&self) -> Result<(Instant<Q>, Signed)> {
        let from = P::origin_offset();
        let to = Q::origin_offset();
        let (shift, ticks) = match to.cmp(&from) {
            Ordering::Greater | Ordering::Equal => {
                let d = to.try_sub(&from).expect("to >= from");
                (
                    Signed::new(Sign::Positive, Delta::from_ticks(d.clone())),
                    self.ticks
                        .try_add(&d)
                        .ok_or(TimeError::new(Code::E0021))?,
                )
            }
            Ordering::Less => {
                let d = from.try_sub(&to).expect("from > to");
                (
                    Signed::new(Sign::Negative, Delta::from_ticks(d.clone())),
                    self.ticks
                        .try_sub(&d)
                        .ok_or(TimeError::new(Code::E0020))?,
                )
            }
        };
        Ok((Instant::<Q>::from_ticks(ticks)?, shift))
    }
}

// ---------------------------------------------------------------------------
// Window — a closed interval (Rules T, U)
// ---------------------------------------------------------------------------

/// A closed interval in absolute time, `lo <= hi`.
///
/// Rule U: window arithmetic is interval arithmetic with outward rounding.
/// Windows are never silently collapsed — [`Window::midpoint`] must be called
/// explicitly and takes a [`Rounding`].
pub struct Window<P: Profile> {
    lo: Instant<P>,
    hi: Instant<P>,
}

// Hand-written for the same reason as `Instant`'s: `P` carries no data.
impl<P: Profile> Clone for Window<P> {
    fn clone(&self) -> Self {
        Window {
            lo: self.lo.clone(),
            hi: self.hi.clone(),
        }
    }
}

#[cfg(feature = "u512")]
impl<P: Profile> Copy for Window<P> {}

impl<P: Profile> PartialEq for Window<P> {
    fn eq(&self, other: &Self) -> bool {
        self.lo == other.lo && self.hi == other.hi
    }
}

impl<P: Profile> Eq for Window<P> {}

impl<P: Profile> core::fmt::Debug for Window<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "Window<{}>[{:?}, {:?}]", P::TAG, self.lo.ticks, self.hi.ticks)
    }
}

// Deliberately NOT implemented for `Window`: `PartialOrd` and `Ord`.
//
// Rule T requires comparison across unequal precision to use interval semantics
// and to be able to return indeterminate. A total order on windows would silently
// resolve overlapping intervals, which is the whole failure the rule exists to
// prevent. Use `Window::compare` or `Window::try_compare` instead.

/// The result of comparing two values whose precisions differ (Rule T).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum IntervalOrdering {
    /// Strictly earlier: `self.hi < other.lo`.
    Before,
    /// Strictly later: `self.lo > other.hi`.
    After,
    /// The same single tick.
    EqualExact,
    /// The intervals overlap, so the order is not determined. `UCAL-E0023`.
    Indeterminate,
}

impl<P: Profile> Window<P> {
    /// Construct from bounds, rejecting inversion with `UCAL-E0022`.
    pub fn new(lo: Instant<P>, hi: Instant<P>) -> Result<Self> {
        if lo > hi {
            return Err(TimeError::new(Code::E0022));
        }
        Ok(Window { lo, hi })
    }

    /// The degenerate window at a single tick.
    pub fn exact(at: Instant<P>) -> Self {
        Window {
            lo: at.clone(),
            hi: at,
        }
    }

    /// Lower bound.
    pub fn lo(&self) -> &Instant<P> {
        &self.lo
    }

    /// Upper bound.
    pub fn hi(&self) -> &Instant<P> {
        &self.hi
    }

    /// Width in ticks. A degenerate window has width zero, not one: it spans one
    /// tick, and the width is the difference between its bounds.
    pub fn width(&self) -> Delta {
        self.hi
            .since(&self.lo)
            .expect("Window maintains lo <= hi")
    }

    /// Whether the window spans exactly one tick.
    pub fn is_exact(&self) -> bool {
        self.lo == self.hi
    }

    /// Whether an instant lies within the closed interval.
    pub fn contains(&self, t: &Instant<P>) -> bool {
        self.lo <= *t && *t <= self.hi
    }

    /// Whether two windows overlap.
    pub fn overlaps(&self, other: &Self) -> bool {
        self.lo <= other.hi && other.lo <= self.hi
    }

    /// Interval-aware comparison, which may be indeterminate (Rule T).
    pub fn compare(&self, other: &Self) -> IntervalOrdering {
        if self.is_exact() && other.is_exact() && self.lo == other.lo {
            return IntervalOrdering::EqualExact;
        }
        if self.hi < other.lo {
            IntervalOrdering::Before
        } else if self.lo > other.hi {
            IntervalOrdering::After
        } else {
            IntervalOrdering::Indeterminate
        }
    }

    /// Interval-aware comparison as a `Result`, so that indeterminacy surfaces as
    /// `UCAL-E0023` at call sites that require a total order.
    pub fn try_compare(&self, other: &Self) -> Result<Ordering> {
        match self.compare(other) {
            IntervalOrdering::Before => Ok(Ordering::Less),
            IntervalOrdering::After => Ok(Ordering::Greater),
            IntervalOrdering::EqualExact => Ok(Ordering::Equal),
            IntervalOrdering::Indeterminate => Err(TimeError::new(Code::E0023)),
        }
    }

    /// Shift the whole window by a magnitude. Outward rounding is trivial here
    /// because a translation preserves width exactly (Rule U).
    pub fn checked_add(&self, d: &Delta) -> Result<Self> {
        Window::new(self.lo.checked_add(d)?, self.hi.checked_add(d)?)
    }

    /// Shift the window back by a magnitude.
    pub fn checked_sub(&self, d: &Delta) -> Result<Self> {
        Window::new(self.lo.checked_sub(d)?, self.hi.checked_sub(d)?)
    }

    /// Widen outward by a magnitude on both sides. The lower bound saturates at
    /// the datum rather than failing, because a window clipped by Rule Z is still
    /// a sound enclosure; the clipping is reported by the returned flag.
    pub fn widen(&self, d: &Delta) -> Result<(Self, bool)> {
        let hi = self.hi.checked_add(d)?;
        match self.lo.checked_sub(d) {
            Ok(lo) => Ok((Window::new(lo, hi)?, false)),
            Err(_) => Ok((Window::new(Instant::zero(), hi)?, true)),
        }
    }

    /// Shift by an *uncertain* magnitude: `lo` with `lo`, `hi` with `hi` (Rule U).
    ///
    /// The result is at least as wide as either input, which is the point — an
    /// uncertain instant displaced by an uncertain duration cannot be known better
    /// than either. This is the operation Rule J.2 needs when an anchor window is
    /// carried forward into a derived field.
    pub fn checked_add_span(&self, s: &Span) -> Result<Self> {
        Window::new(self.lo.checked_add(s.lo())?, self.hi.checked_add(s.hi())?)
    }

    /// Shift back by an uncertain magnitude: `[lo - s.hi, hi - s.lo]`, outward.
    ///
    /// Strict at the datum — `UCAL-E0020` rather than a clamp — because a window
    /// that has been silently clipped is no longer the interval the caller asked
    /// for. Use [`Window::widen`] where clipping is acceptable and reported.
    pub fn checked_sub_span(&self, s: &Span) -> Result<Self> {
        Window::new(self.lo.checked_sub(s.hi())?, self.hi.checked_sub(s.lo())?)
    }

    /// Elapsed time from an earlier window to this one, as an uncertain
    /// magnitude.
    ///
    /// `[max(0, self.lo - earlier.hi), self.hi - earlier.lo]`. The lower bound
    /// clamps at zero because overlapping windows genuinely admit zero elapsed
    /// time; the flag reports whether that happened. `UCAL-E0020` when this window
    /// lies wholly before `earlier`, which is Rule Z applied to intervals.
    pub fn since_window(&self, earlier: &Self) -> Result<(Span, bool)> {
        let hi = self.hi.since(&earlier.lo)?;
        match self.lo.since(&earlier.hi) {
            Ok(lo) => Ok((Span::new(lo, hi)?, false)),
            Err(_) => Ok((Span::new(Delta::zero(), hi)?, true)),
        }
    }

    /// The union enclosure of two windows — the smallest window containing both.
    pub fn hull(&self, other: &Self) -> Self {
        Window {
            lo: if self.lo <= other.lo {
                self.lo.clone()
            } else {
                other.lo.clone()
            },
            hi: if self.hi >= other.hi {
                self.hi.clone()
            } else {
                other.hi.clone()
            },
        }
    }

    /// The intersection, or `None` if the windows are disjoint.
    pub fn intersect(&self, other: &Self) -> Option<Self> {
        if !self.overlaps(other) {
            return None;
        }
        Some(Window {
            lo: if self.lo >= other.lo {
                self.lo.clone()
            } else {
                other.lo.clone()
            },
            hi: if self.hi <= other.hi {
                self.hi.clone()
            } else {
                other.hi.clone()
            },
        })
    }

    /// Collapse to a single instant under an explicit rounding mode.
    ///
    /// Rule U forbids collapsing a window silently, so this is deliberately a
    /// named call that cannot be reached by coercion.
    pub fn midpoint(&self, mode: Rounding) -> Result<Instant<P>> {
        let width = self.width();
        let (half, rem) = width.divmod(&Delta::from_u64(2))?;
        let mut ticks = self
            .lo
            .ticks
            .try_add(half.ticks())
            .ok_or(TimeError::new(Code::E0021))?;
        if !rem.is_zero() {
            let bump = match mode {
                Rounding::Trunc => false,
                Rounding::Ceil | Rounding::HalfUp => true,
                Rounding::HalfEven => ticks.is_odd(),
            };
            if bump {
                ticks = ticks
                    .try_add(&<Ticks as TickInt>::one())
                    .ok_or(TimeError::new(Code::E0021))?;
            }
        }
        Instant::from_ticks(ticks)
    }
}

// ---------------------------------------------------------------------------
// Span — an uncertain magnitude (Rule U)
// ---------------------------------------------------------------------------

/// A closed interval of durations: a magnitude that is known only to within a
/// range.
///
/// [`Window`] is to [`Instant`] as `Span` is to [`Delta`], and the pairing is
/// what lets Rule U's propagation actually happen. Rule J.2 requires an anchor's
/// uncertainty to reach every derived field, and an anchor is a window; the
/// elapsed time from an uncertain anchor to an uncertain instant is therefore not
/// a `Delta` but a `Span`, and typing it as one keeps the uncertainty from being
/// dropped on the way through.
#[cfg_attr(feature = "u512", derive(Copy))]
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Span {
    lo: Delta,
    hi: Delta,
}

impl Span {
    /// Construct from bounds. `UCAL-E0022` if inverted.
    pub fn new(lo: Delta, hi: Delta) -> Result<Span> {
        if lo > hi {
            return Err(TimeError::new(Code::E0022));
        }
        Ok(Span { lo, hi })
    }

    /// A magnitude known exactly.
    pub fn exact(d: Delta) -> Span {
        Span {
            lo: d.clone(),
            hi: d,
        }
    }

    /// Zero, exactly.
    pub fn zero() -> Span {
        Span::exact(Delta::zero())
    }

    /// Lower bound.
    pub fn lo(&self) -> &Delta {
        &self.lo
    }

    /// Upper bound.
    pub fn hi(&self) -> &Delta {
        &self.hi
    }

    /// How wide the uncertainty is.
    pub fn uncertainty(&self) -> Delta {
        self.hi
            .checked_sub(&self.lo)
            .expect("Span maintains lo <= hi")
    }

    /// Whether the magnitude is known exactly.
    pub fn is_exact(&self) -> bool {
        self.lo == self.hi
    }

    /// Interval addition: `lo` with `lo`, `hi` with `hi` (Rule U).
    pub fn checked_add(&self, other: &Span) -> Result<Span> {
        Span::new(
            self.lo.checked_add(&other.lo)?,
            self.hi.checked_add(&other.hi)?,
        )
    }

    /// Interval subtraction, `[lo - other.hi, hi - other.lo]`.
    ///
    /// The lower bound clamps at zero rather than failing: a magnitude is
    /// non-negative, and two overlapping spans genuinely admit a difference of
    /// zero. The clamp is reported so a caller can tell it happened.
    pub fn checked_sub(&self, other: &Span) -> Result<(Span, bool)> {
        let hi = self.hi.checked_sub(&other.lo)?;
        match self.lo.checked_sub(&other.hi) {
            Ok(lo) => Ok((Span::new(lo, hi)?, false)),
            Err(_) => Ok((Span::new(Delta::zero(), hi)?, true)),
        }
    }

    /// Collapse to a single magnitude under an explicit mode (Rule U).
    pub fn midpoint(&self, mode: Rounding) -> Result<Delta> {
        let (half, rem) = self.uncertainty().divmod(&Delta::from_u64(2))?;
        let mut d = self.lo.checked_add(&half)?;
        if !rem.is_zero() {
            let bump = match mode {
                Rounding::Trunc => false,
                Rounding::Ceil | Rounding::HalfUp => true,
                Rounding::HalfEven => d.ticks().is_odd(),
            };
            if bump {
                d = d.checked_add(&Delta::one_tick())?;
            }
        }
        Ok(d)
    }
}

// ---------------------------------------------------------------------------
// Stated — a value together with the precision it was stated at (Rule T)
// ---------------------------------------------------------------------------

/// An instant together with the precision at which it was stated.
///
/// Rule T requires comparison across unequal precision to use interval semantics
/// and to be able to return indeterminate. Carrying the precision alongside the
/// value makes that the *easy* path: [`Stated::try_compare`] cannot be reached
/// without confronting `UCAL-E0023`, whereas comparing two bare `Instant`s that
/// came from truncated text would quietly compare their floors.
///
/// This is what [`crate::codec::parse`] returns, as a pair; the type exists so
/// the pair need not be carried by hand.
pub struct Stated<P: Profile> {
    value: Instant<P>,
    precision: Precision,
}

impl<P: Profile> Clone for Stated<P> {
    fn clone(&self) -> Self {
        Stated {
            value: self.value.clone(),
            precision: self.precision,
        }
    }
}

#[cfg(feature = "u512")]
impl<P: Profile> Copy for Stated<P> {}

impl<P: Profile> PartialEq for Stated<P> {
    /// Equality is on the *statement*, not the interval: two statements are equal
    /// when they say the same thing at the same precision. Use
    /// [`Stated::try_compare`] to ask about the underlying instants.
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value && self.precision == other.precision
    }
}

impl<P: Profile> Eq for Stated<P> {}

impl<P: Profile> core::fmt::Debug for Stated<P> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "Stated<{}>({:?} @ {:?})",
            P::TAG,
            self.value.ticks(),
            self.precision
        )
    }
}

impl<P: Profile> Stated<P> {
    /// Pair a value with its stated precision.
    pub fn new(value: Instant<P>, precision: Precision) -> Self {
        Stated { value, precision }
    }

    /// A value stated to the tick.
    pub fn exact(value: Instant<P>) -> Self {
        Stated {
            value,
            precision: Precision::Tick,
        }
    }

    /// The value as stated — its floor at the stated precision, not a tick unless
    /// the precision says so.
    pub fn value(&self) -> &Instant<P> {
        &self.value
    }

    /// The precision it was stated at.
    pub fn precision(&self) -> Precision {
        self.precision
    }

    /// Whether the statement pins a single tick.
    pub fn is_exact(&self) -> bool {
        self.precision.is_exact()
    }

    /// The interval the statement denotes (Rule T).
    pub fn window(&self) -> Result<Window<P>> {
        self.value.window_at(self.precision)
    }

    /// Interval-aware comparison, which may be indeterminate.
    pub fn compare(&self, other: &Self) -> Result<IntervalOrdering> {
        Ok(self.window()?.compare(&other.window()?))
    }

    /// Interval-aware comparison as a total order, or `UCAL-E0023` when the
    /// intervals overlap and the order is genuinely not determined.
    pub fn try_compare(&self, other: &Self) -> Result<Ordering> {
        self.window()?.try_compare(&other.window()?)
    }

    /// Re-state at a coarser precision. `UCAL-E0023` if asked to *refine*, since
    /// no operation may invent precision the statement does not carry (F2).
    pub fn coarsen(&self, to: Tier) -> Result<Stated<P>> {
        if to < self.precision.tier() {
            return Err(TimeError::with_context(
                Code::E0023,
                "cannot restate at a finer precision than the value was given at",
            ));
        }
        Ok(Stated {
            value: self.value.floor_to(to),
            precision: if to.is_tick() {
                Precision::Tick
            } else {
                Precision::Tier(to)
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::UC1;

    type I = Instant<UC1>;

    fn at(n: u64) -> I {
        I::from_u64(n).unwrap()
    }
    fn d(n: u64) -> Delta {
        Delta::from_u64(n)
    }

    // ---- Rule Z: the domain is unsigned and closed at the datum ----

    #[test]
    fn nothing_precedes_the_datum() {
        assert_eq!(I::zero().ticks(), &<Ticks as TickInt>::zero());
        let err = I::zero().checked_sub(&Delta::one_tick()).unwrap_err();
        assert_eq!(err.code, Code::E0020);
        // ...and it fails rather than wrapping or saturating (Rule O).
        assert_eq!(at(5).checked_sub(&d(6)).unwrap_err().code, Code::E0020);
        assert_eq!(at(5).checked_sub(&d(5)).unwrap(), I::zero());
    }

    #[test]
    fn since_fails_backwards_but_between_does_not() {
        let early = at(10);
        let late = at(30);
        assert_eq!(late.since(&early).unwrap(), d(20));
        assert_eq!(early.since(&late).unwrap_err().code, Code::E0020);

        // `between` is total and signed (§5.1).
        let b = early.between(&late);
        assert_eq!(b.sign(), Sign::Negative);
        assert_eq!(b.magnitude(), &d(20));
        let f = late.between(&early);
        assert_eq!(f.sign(), Sign::Positive);
        assert_eq!(f.magnitude(), &d(20));
        // No negative zero.
        assert_eq!(early.between(&early).sign(), Sign::Positive);
        assert!(early.between(&early).is_zero());
    }

    // ---- Rule O / Rule W: the ceiling is a typed error on both backends ----

    #[test]
    fn domain_ceiling_is_enforced() {
        let max = I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap();
        assert_eq!(
            max.checked_add(&Delta::one_tick()).unwrap_err().code,
            Code::E0021
        );
        assert_eq!(max.checked_add(&Delta::zero()).unwrap(), max.clone());
        // Rule W: identical ceiling regardless of backend representation.
        assert_eq!(<Ticks as TickInt>::domain_max().bit_len(), 512);
    }

    // ---- Rule M: monotone total order ----

    #[test]
    fn order_is_chronological_and_total() {
        let a = at(1);
        let b = at(2);
        assert!(a < b && b > a && a == a.clone());
        assert_eq!(a.cmp(&b), Ordering::Less);
        // Exactly one of <, =, > holds.
        for (x, y) in [(1u64, 2u64), (2, 1), (7, 7)] {
            let (x, y) = (at(x), at(y));
            let n = [x < y, x == y, x > y].iter().filter(|b| **b).count();
            assert_eq!(n, 1);
        }
    }

    // ---- Rule G: truncation is rounding; prefix comparison is chronological ----

    #[test]
    fn floor_ceil_bracket_the_value() {
        let t = Tier::BEAT;
        let beat = t.ticks();
        // A value one tick into the second beat.
        let v = I::from_ticks(beat.try_mul(&<Ticks as TickInt>::from_u64(2)).unwrap())
            .unwrap()
            .checked_add(&Delta::one_tick())
            .unwrap();
        let lo = v.floor_to(t);
        let hi = v.ceil_to(t).unwrap();
        assert!(lo <= v && v <= hi);
        assert_eq!(lo.ticks(), &beat.try_mul(&<Ticks as TickInt>::from_u64(2)).unwrap());
        assert_eq!(hi.ticks(), &beat.try_mul(&<Ticks as TickInt>::from_u64(3)).unwrap());
        // Already-aligned values are fixed points of both.
        assert_eq!(lo.floor_to(t), lo);
        assert_eq!(lo.ceil_to(t).unwrap(), lo);
    }

    #[test]
    fn truncation_is_monotone() {
        // If a <= b then floor(a) <= floor(b): this is what makes prefix
        // comparison chronological comparison (G3).
        let t = Tier::ARC;
        let step = Tier::BEAT.ticks();
        let mut prev: Option<I> = None;
        for n in 0..40u64 {
            let v = I::from_ticks(step.try_mul(&<Ticks as TickInt>::from_u64(n * 7)).unwrap())
                .unwrap();
            let f = v.floor_to(t);
            if let Some(p) = prev {
                assert!(p <= f);
            }
            prev = Some(f);
        }
    }

    // ---- Rule R: rounding only on rendering, and the mode is explicit ----

    #[test]
    fn rounding_modes_behave() {
        let t = Tier::new(-11).unwrap(); // 5^5 = 3125 ticks
        let unit = t.ticks();
        let half = unit.quot_rem(&<Ticks as TickInt>::from_u64(2)).0; // 1562, since 3125 is odd

        let mk = |mult: u64, extra: &Ticks| {
            I::from_ticks(
                unit.try_mul(&<Ticks as TickInt>::from_u64(mult))
                    .unwrap()
                    .try_add(extra)
                    .unwrap(),
            )
            .unwrap()
        };

        // Below the midpoint (3125 is odd, so 1562 < half of 3125).
        let below = mk(2, &half);
        assert_eq!(below.round_to(t, Rounding::Trunc).unwrap(), mk(2, &<Ticks as TickInt>::zero()));
        assert_eq!(below.round_to(t, Rounding::Ceil).unwrap(), mk(3, &<Ticks as TickInt>::zero()));
        assert_eq!(
            below.round_to(t, Rounding::HalfEven).unwrap(),
            mk(2, &<Ticks as TickInt>::zero())
        );

        // Just above the midpoint.
        let above = mk(2, &half.try_add(&<Ticks as TickInt>::one()).unwrap());
        assert_eq!(
            above.round_to(t, Rounding::HalfEven).unwrap(),
            mk(3, &<Ticks as TickInt>::zero())
        );

        // An exact tie needs an even-sized tier; use ticks with a tier of 5^0's
        // neighbour instead: 2 ticks is not a tier, so exercise the tie through
        // Window::midpoint, which is where ties actually arise.
        let w = Window::new(at(0), at(3)).unwrap();
        assert_eq!(w.midpoint(Rounding::Trunc).unwrap(), at(1));
        assert_eq!(w.midpoint(Rounding::Ceil).unwrap(), at(2));
        assert_eq!(w.midpoint(Rounding::HalfUp).unwrap(), at(2));
        // lo + half = 1, which is odd, so half-even rounds up to 2.
        assert_eq!(w.midpoint(Rounding::HalfEven).unwrap(), at(2));
        // An exact midpoint is returned unchanged by every mode.
        let w2 = Window::new(at(0), at(4)).unwrap();
        for m in [Rounding::Trunc, Rounding::Ceil, Rounding::HalfEven, Rounding::HalfUp] {
            assert_eq!(w2.midpoint(m).unwrap(), at(2));
        }
    }

    // ---- Rule T: truncation is uncertainty ----

    #[test]
    fn tier_precision_denotes_a_closed_interval() {
        let t = Tier::BEAT;
        let v = at(12_345);
        let w = v.window_at(Precision::Tier(t)).unwrap();
        assert_eq!(w.lo(), &v.floor_to(t));
        // [floor, floor + 5^e - 1] — inclusive, so width is 5^e - 1.
        assert_eq!(
            w.width().ticks(),
            &t.ticks().try_sub(&<Ticks as TickInt>::one()).unwrap()
        );
        assert!(w.contains(&v));
        assert!(!w.is_exact());

        // Tick precision is the degenerate window.
        let e = v.window_at(Precision::Tick).unwrap();
        assert!(e.is_exact());
        assert_eq!(e.width(), Delta::zero());
        assert_eq!(e.lo(), e.hi());
        // ...as is an explicit tick tier.
        assert!(v.window_at(Precision::Tier(Tier::TICK)).unwrap().is_exact());
    }

    #[test]
    fn comparison_across_precision_can_be_indeterminate() {
        let t = Tier::BEAT;
        let a = at(10).window_at(Precision::Tier(t)).unwrap();
        let b = at(20).window_at(Precision::Tier(t)).unwrap();
        // Both truncate into the same beat, so their order is undetermined.
        assert_eq!(a.compare(&b), IntervalOrdering::Indeterminate);
        assert_eq!(a.try_compare(&b).unwrap_err().code, Code::E0023);

        // Windows a whole tier apart are determinate.
        let far = I::from_ticks(t.ticks().try_mul(&<Ticks as TickInt>::from_u64(5)).unwrap())
            .unwrap()
            .window_at(Precision::Tier(t))
            .unwrap();
        assert_eq!(a.compare(&far), IntervalOrdering::Before);
        assert_eq!(far.compare(&a), IntervalOrdering::After);
        assert_eq!(a.try_compare(&far).unwrap(), Ordering::Less);

        // Two exact windows at the same tick compare equal.
        let x = Window::exact(at(7));
        assert_eq!(x.compare(&Window::exact(at(7))), IntervalOrdering::EqualExact);
    }

    // ---- Rule U: interval arithmetic, no silent collapse ----

    #[test]
    fn window_arithmetic_is_interval_arithmetic() {
        let w = Window::new(at(10), at(20)).unwrap();
        let s = w.checked_add(&d(5)).unwrap();
        assert_eq!((s.lo(), s.hi()), (&at(15), &at(25)));
        // Translation preserves width exactly.
        assert_eq!(s.width(), w.width());

        assert_eq!(Window::new(at(20), at(10)).unwrap_err().code, Code::E0022);

        let (wide, clipped) = w.widen(&d(5)).unwrap();
        assert_eq!((wide.lo(), wide.hi()), (&at(5), &at(25)));
        assert!(!clipped);
        // Widening past the datum clips at zero and says so (Rule Z).
        let (wide2, clipped2) = w.widen(&d(50)).unwrap();
        assert_eq!(wide2.lo(), &I::zero());
        assert!(clipped2);

        let other = Window::new(at(15), at(30)).unwrap();
        assert!(w.overlaps(&other));
        assert_eq!(w.hull(&other), Window::new(at(10), at(30)).unwrap());
        assert_eq!(w.intersect(&other), Some(Window::new(at(15), at(20)).unwrap()));
        assert_eq!(w.intersect(&Window::new(at(40), at(50)).unwrap()), None);
    }

    #[test]
    fn signed_window_is_inert() {
        // Rule Q.3: the runtime half of the guarantee. The compile-time half is
        // `tests/compile_fail/signed_window_*.rs`, which §21.3-3 requires.
        let claim = UC1::big_bang_claim();
        assert_eq!(claim.lo().sign(), Sign::Negative);
        assert_eq!(claim.hi().sign(), Sign::Positive);
        // It reports, and that is all it does.
        #[cfg(feature = "alloc")]
        assert!(claim.describe().contains("ticks"));
    }

    // ---- Rule B: canonical binary ----

    #[test]
    fn canonical_binary_is_64_bytes_and_round_trips() {
        for n in [0u64, 1, 255, 256, 65_535, u64::MAX] {
            let v = at(n);
            let b = v.to_bytes();
            assert_eq!(b.len(), 64);
            assert_eq!(I::from_bytes(&b).unwrap(), v);
        }
        assert_eq!(I::zero().to_bytes(), [0u8; 64]);
        let max = I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap();
        assert_eq!(max.to_bytes(), [0xffu8; 64]);
    }

    #[test]
    fn byte_order_is_chronological_order() {
        // Rule S: lexicographic order equals chronological order for the binary
        // form, which is what makes it usable directly as a database key.
        let mut vals: Vec<I> = (0..64u64).map(|i| at(i * 2_654_435_761)).collect();
        vals.push(at(0));
        vals.push(I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap());
        vals.sort();
        let mut bytes: Vec<[u8; 64]> = vals.iter().map(|v| v.to_bytes()).collect();
        let numeric = bytes.clone();
        bytes.sort();
        assert_eq!(bytes, numeric, "byte order diverges from numeric order");
    }

    // ---- Rule P: rebase reports the shift ----

    #[test]
    fn rebase_is_identity_within_a_profile() {
        let v = at(1_000_000);
        let (out, shift) = v.rebase::<UC1>().unwrap();
        assert_eq!(out, v);
        assert!(shift.is_zero());
    }

    // ---- Delta ----

    #[test]
    fn delta_arithmetic() {
        assert_eq!(d(3).checked_add(&d(4)).unwrap(), d(7));
        assert_eq!(d(3).checked_sub(&d(4)).unwrap_err().code, Code::E0020);
        assert_eq!(d(12).mul_u64(3).unwrap(), d(36));
        assert_eq!(d(12).div_u64(5).unwrap(), d(2));
        assert_eq!(d(12).divmod(&d(5)).unwrap(), (d(2), d(2)));
        assert_eq!(d(1).div_u64(0).unwrap_err().code, Code::E0021);
        assert!(Delta::zero().is_zero());
        assert_eq!(Delta::one_tick(), d(1));
    }

    #[test]
    fn delta_tier_of_finds_the_largest_fitting_tier() {
        // One beat exactly.
        let beat = Delta::from_ticks(Tier::BEAT.ticks());
        assert_eq!(beat.tier_of(), Some(Tier::BEAT));
        // One tick under a beat lands on the tier below.
        let just_under = beat.checked_sub(&Delta::one_tick()).unwrap();
        assert_eq!(just_under.tier_of(), Some(Tier::new(-1).unwrap()));
        // One tick is the tick tier; zero has no tier.
        assert_eq!(Delta::one_tick().tier_of(), Some(Tier::TICK));
        assert_eq!(Delta::zero().tier_of(), None);
        // Whole-tier counts come back exactly.
        let three_beats = Delta::from_tier(Tier::BEAT, 3).unwrap();
        assert_eq!(three_beats.in_tier(Tier::BEAT).0, <Ticks as TickInt>::from_u64(3));
        assert!(three_beats.in_tier(Tier::BEAT).1.is_zero_ticks());
    }

    // ---- group extraction ----

    #[test]
    fn tier_values_are_base_5_groups() {
        // Build a value with known group digits: 2 beats + 3 arcs.
        let v = I::from_ticks(
            Tier::BEAT
                .ticks()
                .try_mul(&<Ticks as TickInt>::from_u64(2))
                .unwrap()
                .try_add(
                    &Tier::ARC
                        .ticks()
                        .try_mul(&<Ticks as TickInt>::from_u64(3))
                        .unwrap(),
                )
                .unwrap(),
        )
        .unwrap();
        assert_eq!(v.tier_value(Tier::BEAT), 2);
        assert_eq!(v.tier_value(Tier::ARC), 3);
        assert_eq!(v.tier_value(Tier::SWEEP), 0);
        // Every group is in range (Rule G / UCAL-E0004).
        for t in Tier::all_ascending() {
            assert!(v.tier_value(t) < GROUP_BASE);
        }
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn groups_descend_and_reassemble() {
        let v = at(987_654_321);
        let gs = v.groups(Tier::BEAT, Tier::TICK).unwrap();
        assert_eq!(gs.len(), 13); // T0 down to T-12
        // §21.1: tier decomposition reassembles the original value.
        let mut acc = <Ticks as TickInt>::zero();
        let base = <Ticks as TickInt>::from_u64(GROUP_BASE as u64);
        for g in &gs {
            acc = acc
                .try_mul(&base)
                .unwrap()
                .try_add(&<Ticks as TickInt>::from_u64(*g as u64))
                .unwrap();
        }
        assert_eq!(&acc, v.ticks());
        // An ascending range is rejected rather than silently reversed.
        assert_eq!(
            v.groups(Tier::TICK, Tier::BEAT).unwrap_err().code,
            Code::E0006
        );
    }
}

#[cfg(test)]
mod rule_u_tests {
    use super::*;
    use crate::profile::UC1;

    type I = Instant<UC1>;

    fn at(n: u64) -> I {
        I::from_u64(n).unwrap()
    }
    fn d(n: u64) -> Delta {
        Delta::from_u64(n)
    }
    fn w(lo: u64, hi: u64) -> Window<UC1> {
        Window::new(at(lo), at(hi)).unwrap()
    }
    fn sp(lo: u64, hi: u64) -> Span {
        Span::new(d(lo), d(hi)).unwrap()
    }

    // ---- Span ----

    #[test]
    fn span_is_an_interval_of_magnitudes() {
        let s = sp(10, 20);
        assert_eq!(s.uncertainty(), d(10));
        assert!(!s.is_exact());
        assert!(Span::exact(d(7)).is_exact());
        assert_eq!(Span::exact(d(7)).uncertainty(), Delta::zero());
        assert!(Span::zero().is_exact());
        assert_eq!(Span::new(d(20), d(10)).unwrap_err().code, Code::E0022);
    }

    #[test]
    fn span_addition_is_interval_addition() {
        // Rule U: lo combines with lo, hi with hi.
        let a = sp(10, 20);
        let b = sp(1, 5);
        let s = a.checked_add(&b).unwrap();
        assert_eq!((s.lo(), s.hi()), (&d(11), &d(25)));
        // Uncertainty accumulates; it never shrinks.
        assert_eq!(s.uncertainty(), d(14));
        assert!(s.uncertainty() >= a.uncertainty());
        assert!(s.uncertainty() >= b.uncertainty());
    }

    #[test]
    fn span_subtraction_clamps_at_zero_and_says_so() {
        // [10,20] - [1,5] = [5,19]
        let (s, clamped) = sp(10, 20).checked_sub(&sp(1, 5)).unwrap();
        assert_eq!((s.lo(), s.hi()), (&d(5), &d(19)));
        assert!(!clamped);
        // Overlapping spans genuinely admit a difference of zero.
        let (s, clamped) = sp(10, 20).checked_sub(&sp(15, 25)).unwrap();
        assert_eq!(s.lo(), &Delta::zero());
        assert_eq!(s.hi(), &d(5));
        assert!(clamped, "the clamp must be reported, not hidden");
        // Wholly smaller is still an error: the upper bound cannot go negative.
        assert_eq!(
            sp(1, 2).checked_sub(&sp(10, 20)).unwrap_err().code,
            Code::E0020
        );
    }

    #[test]
    fn span_midpoint_must_be_asked_for() {
        let s = sp(0, 3);
        assert_eq!(s.midpoint(Rounding::Trunc).unwrap(), d(1));
        assert_eq!(s.midpoint(Rounding::Ceil).unwrap(), d(2));
        assert_eq!(s.midpoint(Rounding::HalfUp).unwrap(), d(2));
        assert_eq!(sp(0, 4).midpoint(Rounding::Trunc).unwrap(), d(2));
    }

    // ---- Window x Span, the Rule J.2 propagation path ----

    #[test]
    fn uncertain_instant_plus_uncertain_duration_widens() {
        // The operation a derived calendar performs: an anchor known to a window,
        // displaced by an elapsed time known to a span.
        let anchor = w(100, 110); // 10 ticks of anchor uncertainty
        let elapsed = sp(1000, 1005); // 5 ticks of parameter uncertainty
        let out = anchor.checked_add_span(&elapsed).unwrap();
        assert_eq!((out.lo(), out.hi()), (&at(1100), &at(1115)));
        // Rule U: the result cannot be narrower than either input.
        assert_eq!(out.width(), d(15));
        assert!(out.width() >= anchor.width());
        assert!(out.width() >= elapsed.uncertainty());
    }

    #[test]
    fn subtracting_a_span_is_outward_and_strict_at_the_datum() {
        let out = w(1000, 1010).checked_sub_span(&sp(100, 200)).unwrap();
        // [1000 - 200, 1010 - 100]
        assert_eq!((out.lo(), out.hi()), (&at(800), &at(910)));
        assert!(out.width() > w(1000, 1010).width());
        // Strict rather than clamped: a silently clipped window is not the
        // interval that was asked for.
        assert_eq!(
            w(10, 20).checked_sub_span(&sp(0, 100)).unwrap_err().code,
            Code::E0020
        );
    }

    #[test]
    fn elapsed_between_windows_is_a_span() {
        let earlier = w(100, 110);
        let later = w(1000, 1010);
        let (s, clamped) = later.since_window(&earlier).unwrap();
        // [1000 - 110, 1010 - 100]
        assert_eq!((s.lo(), s.hi()), (&d(890), &d(910)));
        assert!(!clamped);
        // Overlapping windows admit zero elapsed time.
        let (s, clamped) = w(100, 200).since_window(&w(150, 250)).unwrap();
        assert_eq!(s.lo(), &Delta::zero());
        assert_eq!(s.hi(), &d(50));
        assert!(clamped);
        // Wholly earlier is Rule Z applied to intervals.
        assert_eq!(
            w(10, 20).since_window(&w(100, 200)).unwrap_err().code,
            Code::E0020
        );
    }

    #[test]
    fn round_trip_through_span_preserves_containment() {
        // The property that makes the propagation sound: displacing a window and
        // then undoing it must not lose the original.
        let start = w(10_000, 10_050);
        let s = sp(300, 320);
        let moved = start.checked_add_span(&s).unwrap();
        let back = moved.checked_sub_span(&s).unwrap();
        assert!(back.contains(start.lo()));
        assert!(back.contains(start.hi()));
        // ...and is no narrower, because uncertainty is never recovered.
        assert!(back.width() >= start.width());
    }

    // ---- Stated: Rule T comparison ----

    #[test]
    fn stated_comparison_uses_interval_semantics() {
        let a = Stated::new(at(10).floor_to(Tier::BEAT), Precision::Tier(Tier::BEAT));
        let b = Stated::new(at(20).floor_to(Tier::BEAT), Precision::Tier(Tier::BEAT));
        // Both fall in the same beat, so their order is genuinely undetermined.
        assert_eq!(a.compare(&b).unwrap(), IntervalOrdering::Indeterminate);
        assert_eq!(a.try_compare(&b).unwrap_err().code, Code::E0023);

        // Exact statements at the same tick compare equal.
        let x = Stated::exact(at(7));
        assert_eq!(
            x.compare(&Stated::exact(at(7))).unwrap(),
            IntervalOrdering::EqualExact
        );
        assert_eq!(x.try_compare(&Stated::exact(at(8))).unwrap(), Ordering::Less);
        assert!(x.is_exact());
    }

    #[test]
    fn stated_comparison_across_unequal_precision() {
        // A coarse statement and a fine one, where the fine one lies inside the
        // coarse one's window: indeterminate, not equal and not ordered.
        let coarse = Stated::new(at(0).floor_to(Tier::BEAT), Precision::Tier(Tier::BEAT));
        let fine = Stated::exact(at(5));
        assert_eq!(coarse.compare(&fine).unwrap(), IntervalOrdering::Indeterminate);
        assert_eq!(coarse.try_compare(&fine).unwrap_err().code, Code::E0023);

        // A fine statement outside the coarse window is determinate.
        let far = Stated::exact(
            I::from_ticks(
                Tier::BEAT
                    .ticks()
                    .try_mul(&<Ticks as TickInt>::from_u64(3))
                    .unwrap(),
            )
            .unwrap(),
        );
        assert_eq!(coarse.try_compare(&far).unwrap(), Ordering::Less);
        assert_eq!(far.try_compare(&coarse).unwrap(), Ordering::Greater);
    }

    #[test]
    fn stated_cannot_be_refined_only_coarsened() {
        // F2 as an API constraint: no operation may invent precision.
        let s = Stated::new(at(1_000_000).floor_to(Tier::BEAT), Precision::Tier(Tier::BEAT));
        assert!(s.coarsen(Tier::ARC).is_ok());
        assert_eq!(s.coarsen(Tier::ARC).unwrap().precision(), Precision::Tier(Tier::ARC));
        // Refining is refused.
        assert_eq!(
            s.coarsen(Tier::new(-3).unwrap()).unwrap_err().code,
            Code::E0023
        );
        assert_eq!(s.coarsen(Tier::TICK).unwrap_err().code, Code::E0023);
        // Coarsening widens the window it denotes.
        assert!(s.coarsen(Tier::ARC).unwrap().window().unwrap().width() > s.window().unwrap().width());
    }

    #[test]
    fn stated_equality_is_about_the_statement() {
        // Two statements that denote overlapping intervals are not "equal"; only
        // the same value at the same precision is.
        let a = Stated::new(at(0), Precision::Tier(Tier::BEAT));
        let b = Stated::new(at(0), Precision::Tier(Tier::ARC));
        assert_ne!(a, b);
        assert_eq!(a, Stated::new(at(0), Precision::Tier(Tier::BEAT)));
    }

    // ---- Rule T near the domain ceiling ----

    #[test]
    fn coarse_window_at_the_ceiling_is_clipped_to_the_domain() {
        // `floor + 5^e - 1` can run past domain_max. The interval is intersected
        // with the closed domain rather than failing: no representable instant
        // lies beyond the ceiling, so clamping stays a sound enclosure.
        let top = I::from_ticks(<Ticks as TickInt>::domain_max()).unwrap();
        let t32 = Tier::new(crate::tier::K_MAX).unwrap();
        let win = top.window_at(Precision::Tier(t32)).unwrap();
        assert!(win.contains(&top));
        assert_eq!(win.hi(), &top);
        assert_eq!(win.lo(), &top.floor_to(t32));
        assert!(!win.is_exact());
    }
}
