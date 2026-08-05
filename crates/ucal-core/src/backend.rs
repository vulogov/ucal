//! The integer backend.
//!
//! Rule W: the value domain of `UC-1` is `[0, 2^512)` on **every** backend, so
//! the two backends are behaviourally identical and differential testing between
//! them is a conformance test rather than an approximation. The `bigint` backend
//! therefore enforces the same ceiling as the fixed-width one, even though its
//! representation could hold more.
//!
//! GE-4 anticipates churn in the fixed-width library. All of it is absorbed here:
//! `Ticks` is an alias for whichever type implements [`TickInt`], and nothing
//! outside this module names a backend type. bnum 0.14 already justified the
//! layer — it renamed `parse_str_radix` to `from_str_radix` and made `ZERO`
//! and `ONE` private relative to the API the RFC §3.3 quotes.

#[cfg(all(feature = "u512", feature = "bigint"))]
compile_error!(
    "features `u512` and `bigint` are mutually exclusive: pick exactly one backend. \
     Rule B makes the value width a wire-format commitment, so a build with both \
     would have an ambiguous canonical encoding."
);

#[cfg(not(any(feature = "u512", feature = "bigint")))]
compile_error!("exactly one of the `u512` or `bigint` features must be enabled");

/// Width of the canonical binary encoding in bytes (§7.1, Rule B).
pub const CANONICAL_BYTES: usize = 64;

/// Bit width of the `UC-1` domain: `DOMAIN = [0, 2^512)` (§2.1, Rule W).
pub const DOMAIN_BITS: u32 = 512;

/// The arithmetic surface every backend must provide.
///
/// Deliberately narrow, and deliberately *checked*: Rule O forbids exposing
/// wrapping or saturating arithmetic on time types, so no unchecked operator is
/// part of this trait. Division is total only because the divisor is a caller
/// obligation — [`TickInt::quot_rem`] panics on a zero divisor, which is an
/// internal invariant violation (§19.5 exit code 9), not a user-reachable error.
pub trait TickInt:
    Clone + PartialEq + Eq + PartialOrd + Ord + core::hash::Hash + core::fmt::Debug
{
    /// An integer twice this backend's width, used for multiply-then-divide so
    /// that the intermediate cannot overflow (Appendix H.1).
    ///
    /// It is deliberately opaque: only [`TickInt::wide_mul`],
    /// [`TickInt::wide_quot_rem`] and [`TickInt::narrow`] operate on it, so a
    /// wide value can never escape into a public signature and become a second,
    /// undeclared value domain.
    type Wide: Clone + PartialEq + Eq + PartialOrd + Ord + core::fmt::Debug;

    /// Additive identity. Also the datum (Rule Z).
    fn zero() -> Self;
    /// Multiplicative identity.
    fn one() -> Self;
    /// The largest representable value: `2^512 - 1` on every backend (Rule W).
    fn domain_max() -> Self;

    /// Exact product in the widened type. Cannot overflow: two 512-bit values
    /// need at most 1024 bits.
    fn wide_mul(&self, other: &Self) -> Self::Wide;

    /// Quotient and remainder of a wide value by a narrow one.
    ///
    /// The quotient stays wide because `a x n / d` need not fit the domain; the
    /// remainder is always narrower than the divisor and so always fits.
    fn wide_quot_rem(wide: &Self::Wide, divisor: &Self) -> (Self::Wide, Self);

    /// Bring a wide value back into the domain, or `None` if it does not fit
    /// (Rules O, W).
    fn narrow(wide: &Self::Wide) -> Option<Self>;

    /// Zero test on the widened type.
    fn wide_is_zero(wide: &Self::Wide) -> bool;

    /// `2^exponent`, or `None` if it leaves the domain.
    ///
    /// Provided by repeated doubling so that adding a backend needs no shift
    /// operation. The exponent is bounded by the domain width, so this is at
    /// most 512 checked additions.
    fn pow2(exponent: u32) -> Option<Self> {
        let mut acc = Self::one();
        for _ in 0..exponent {
            acc = acc.try_add(&acc)?;
        }
        Some(acc)
    }

    /// Widen a small integer.
    fn from_u64(v: u64) -> Self;

    /// Widen a 128-bit integer.
    ///
    /// Provided so a backend need not implement it: `SubSecond` carries up to
    /// thirty decimal digits, which exceeds `u64` but fits `u128`.
    fn from_u128(v: u128) -> Option<Self> {
        let hi = Self::from_u64((v >> 64) as u64);
        let lo = Self::from_u64(v as u64);
        let shift = Self::pow2(64)?;
        hi.try_mul(&shift)?.try_add(&lo)
    }
    /// Parse an unsigned decimal string. `None` on any non-digit or on overflow.
    fn from_dec_str(s: &str) -> Option<Self>;

    /// `self + other`, or `None` if the result leaves the domain (Rules O, W).
    fn try_add(&self, other: &Self) -> Option<Self>;
    /// `self - other`, or `None` if the result would precede the datum (Rule Z).
    fn try_sub(&self, other: &Self) -> Option<Self>;
    /// `self * other`, or `None` if the result leaves the domain.
    fn try_mul(&self, other: &Self) -> Option<Self>;

    /// Truncating quotient and remainder. Panics if `divisor` is zero.
    fn quot_rem(&self, divisor: &Self) -> (Self, Self);

    /// `5^exponent`, or `None` if it leaves the domain. `5^220` is the largest
    /// power of five the domain holds, at 511 bits — which is exactly why the
    /// tier grid stops at T32.
    ///
    /// Square-and-multiply, so a tier costs about 14 multiplications rather than
    /// 220. `Tier::ticks()` is on the hot path of every codec call, and the naive
    /// loop made the exhaustive Rule T sweep take 38 seconds.
    ///
    /// The `e > 0` guard before squaring matters: computing `5^220` needs `5^128`
    /// as an intermediate, but squaring that to `5^256` would overflow the domain
    /// even though the result does not.
    fn pow5(exponent: u32) -> Option<Self> {
        let mut result = Self::one();
        let mut base = Self::from_u64(5);
        let mut e = exponent;
        while e > 0 {
            if e & 1 == 1 {
                result = result.try_mul(&base)?;
            }
            e >>= 1;
            if e > 0 {
                base = base.try_mul(&base)?;
            }
        }
        Some(result)
    }

    /// Number of significant bits; `0` for zero.
    fn bit_len(&self) -> u32;

    /// Canonical binary form: 64 bytes, big-endian, zero-padded (Rule B).
    fn to_canonical_bytes(&self) -> [u8; CANONICAL_BYTES];
    /// Inverse of [`TickInt::to_canonical_bytes`]. `None` if the value exceeds
    /// the domain, which cannot happen for a 512-bit domain but can for a
    /// profile that narrows it.
    fn from_canonical_bytes(bytes: &[u8; CANONICAL_BYTES]) -> Option<Self>;

    /// Whether `self` is the additive identity.
    fn is_zero_ticks(&self) -> bool {
        *self == Self::zero()
    }

    /// Whether `self` is odd. Needed for half-even rounding.
    fn is_odd(&self) -> bool;

    /// Decimal rendering.
    #[cfg(feature = "alloc")]
    fn to_dec_string(&self) -> alloc::string::String;

    /// Rendering in an arbitrary radix from 2 to 36.
    #[cfg(feature = "alloc")]
    fn to_radix_string(&self, radix: u32) -> alloc::string::String;
}

// ---------------------------------------------------------------------------
// Default backend: bnum fixed-width U512
// ---------------------------------------------------------------------------

#[cfg(feature = "u512")]
mod imp {
    use super::{TickInt, CANONICAL_BYTES, DOMAIN_BITS};

    /// The tick count type on the default backend.
    pub type Ticks = bnum::types::U512;

    /// Parse a decimal literal in a `const` context.
    ///
    /// §3.3 requires the profile constants to be `const` on the default backend.
    /// This is the whole reason the fixed-width backend is the default.
    pub const fn konst(s: &str) -> Ticks {
        match Ticks::from_str_radix(s, 10) {
            Ok(v) => v,
            Err(_) => panic!("invalid decimal literal in a const profile constant"),
        }
    }

    /// The widened intermediate: 1024 bits, so a product of two domain values
    /// cannot overflow it.
    pub type Wide = bnum::types::U1024;

    fn widen(v: &Ticks) -> Wide {
        let mut buf = [0u8; 2 * CANONICAL_BYTES];
        buf[CANONICAL_BYTES..].copy_from_slice(&v.to_be_bytes());
        Wide::from_be_bytes(buf)
    }

    impl TickInt for Ticks {
        type Wide = Wide;

        fn zero() -> Self {
            Self::MIN
        }
        fn one() -> Self {
            konst("1")
        }
        fn domain_max() -> Self {
            Self::MAX
        }
        fn wide_mul(&self, other: &Self) -> Wide {
            // Both operands fit in 512 bits, so the product fits in 1024 and this
            // multiplication is total.
            widen(self) * widen(other)
        }
        fn wide_quot_rem(wide: &Wide, divisor: &Self) -> (Wide, Self) {
            assert!(
                !TickInt::is_zero_ticks(divisor),
                "internal invariant: division by zero"
            );
            let d = widen(divisor);
            let q = *wide / d;
            let r = *wide % d;
            // r < divisor <= domain_max, so narrowing is infallible here.
            (
                q,
                Self::narrow(&r).expect("a remainder is smaller than its divisor"),
            )
        }
        fn narrow(wide: &Wide) -> Option<Self> {
            let bytes = wide.to_be_bytes();
            if bytes[..CANONICAL_BYTES].iter().any(|b| *b != 0) {
                return None;
            }
            let mut low = [0u8; CANONICAL_BYTES];
            low.copy_from_slice(&bytes[CANONICAL_BYTES..]);
            Some(Self::from_be_bytes(low))
        }
        fn wide_is_zero(wide: &Wide) -> bool {
            *wide == Wide::MIN
        }
        fn from_u64(v: u64) -> Self {
            let mut bytes = [0u8; CANONICAL_BYTES];
            bytes[CANONICAL_BYTES - 8..].copy_from_slice(&v.to_be_bytes());
            Self::from_be_bytes(bytes)
        }
        fn from_dec_str(s: &str) -> Option<Self> {
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            Self::from_str_radix(s, 10).ok()
        }
        fn try_add(&self, other: &Self) -> Option<Self> {
            bnum::types::U512::checked_add(*self, *other)
        }
        fn try_sub(&self, other: &Self) -> Option<Self> {
            bnum::types::U512::checked_sub(*self, *other)
        }
        fn try_mul(&self, other: &Self) -> Option<Self> {
            bnum::types::U512::checked_mul(*self, *other)
        }
        fn quot_rem(&self, divisor: &Self) -> (Self, Self) {
            assert!(
                !TickInt::is_zero_ticks(divisor),
                "internal invariant: division by zero"
            );
            (*self / *divisor, *self % *divisor)
        }
        fn bit_len(&self) -> u32 {
            // Written without `saturating_sub` so that Rule O's lint can forbid
            // the whole family of tokens outright rather than carrying an
            // exemption list. `leading_zeros()` cannot exceed the width, so the
            // branch is defensive rather than reachable.
            let lz = self.leading_zeros();
            if lz >= DOMAIN_BITS {
                0
            } else {
                DOMAIN_BITS - lz
            }
        }
        fn to_canonical_bytes(&self) -> [u8; CANONICAL_BYTES] {
            self.to_be_bytes()
        }
        fn from_canonical_bytes(bytes: &[u8; CANONICAL_BYTES]) -> Option<Self> {
            Some(Self::from_be_bytes(*bytes))
        }
        fn is_zero_ticks(&self) -> bool {
            *self == Self::MIN
        }
        fn is_odd(&self) -> bool {
            self.bit(0)
        }
        #[cfg(feature = "alloc")]
        fn to_dec_string(&self) -> alloc::string::String {
            self.to_str_radix(10)
        }
        #[cfg(feature = "alloc")]
        fn to_radix_string(&self, radix: u32) -> alloc::string::String {
            self.to_str_radix(radix)
        }
    }
}

// ---------------------------------------------------------------------------
// Alternative backend: num-bigint heap BigUint, ceiling enforced (Rule W)
// ---------------------------------------------------------------------------

// `not(u512)` as well as `bigint`, so that a build with both features
// produces the guard above and *only* the guard. It used to emit
// "the name `imp` is defined multiple times" first, which is the
// error a caller reads and the one that says nothing.
#[cfg(all(feature = "bigint", not(feature = "u512")))]
mod imp {
    use super::{TickInt, CANONICAL_BYTES, DOMAIN_BITS};
    use alloc::string::String;
    use alloc::vec;
    use num_bigint::BigUint;
    use num_integer::Integer;
    use num_traits::{One, Zero};

    /// The tick count type on the `bigint` backend.
    pub type Ticks = BigUint;

    fn ceiling() -> BigUint {
        <BigUint as One>::one() << DOMAIN_BITS
    }

    /// Reject any value at or above `2^512`, so the domain matches the default
    /// backend exactly (Rule W). Without this the two backends would disagree
    /// near the ceiling and Rule B's fixed 64-byte encoding would be unsound.
    fn within_domain(v: BigUint) -> Option<BigUint> {
        if v < ceiling() {
            Some(v)
        } else {
            None
        }
    }

    impl TickInt for Ticks {
        /// `BigUint` is already unbounded, so the widened type is the same type.
        /// The distinction still matters: [`TickInt::narrow`] is where the Rule W
        /// ceiling is re-imposed, and it is the only way back to a `Ticks`.
        type Wide = BigUint;

        fn zero() -> Self {
            <BigUint as Zero>::zero()
        }
        fn one() -> Self {
            <BigUint as One>::one()
        }
        fn domain_max() -> Self {
            ceiling() - <BigUint as One>::one()
        }
        fn wide_mul(&self, other: &Self) -> BigUint {
            self * other
        }
        fn wide_quot_rem(wide: &BigUint, divisor: &Self) -> (BigUint, Self) {
            assert!(
                !Zero::is_zero(divisor),
                "internal invariant: division by zero"
            );
            Integer::div_rem(wide, divisor)
        }
        fn narrow(wide: &BigUint) -> Option<Self> {
            within_domain(wide.clone())
        }
        fn wide_is_zero(wide: &BigUint) -> bool {
            Zero::is_zero(wide)
        }
        fn from_u64(v: u64) -> Self {
            BigUint::from(v)
        }
        fn from_dec_str(s: &str) -> Option<Self> {
            if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            within_domain(BigUint::parse_bytes(s.as_bytes(), 10)?)
        }
        fn try_add(&self, other: &Self) -> Option<Self> {
            within_domain(self + other)
        }
        fn try_sub(&self, other: &Self) -> Option<Self> {
            if self < other {
                None
            } else {
                Some(self - other)
            }
        }
        fn try_mul(&self, other: &Self) -> Option<Self> {
            within_domain(self * other)
        }
        fn quot_rem(&self, divisor: &Self) -> (Self, Self) {
            assert!(
                !Zero::is_zero(divisor),
                "internal invariant: division by zero"
            );
            Integer::div_rem(self, divisor)
        }
        fn bit_len(&self) -> u32 {
            self.bits() as u32
        }
        fn to_canonical_bytes(&self) -> [u8; CANONICAL_BYTES] {
            let raw = self.to_bytes_be();
            let raw: &[u8] = if Zero::is_zero(self) { &[] } else { &raw };
            debug_assert!(raw.len() <= CANONICAL_BYTES, "Rule W violated");
            let mut out = [0u8; CANONICAL_BYTES];
            out[CANONICAL_BYTES - raw.len()..].copy_from_slice(raw);
            out
        }
        fn from_canonical_bytes(bytes: &[u8; CANONICAL_BYTES]) -> Option<Self> {
            within_domain(BigUint::from_bytes_be(bytes))
        }
        fn is_zero_ticks(&self) -> bool {
            Zero::is_zero(self)
        }
        fn is_odd(&self) -> bool {
            Integer::is_odd(self)
        }
        fn to_dec_string(&self) -> String {
            self.to_str_radix(10)
        }
        fn to_radix_string(&self, radix: u32) -> String {
            self.to_str_radix(radix)
        }
    }

    /// Present on the `bigint` backend only so that call sites shared with the
    /// default backend compile. It is *not* `const` — that asymmetry is the
    /// documented cost of the feature (§3.3).
    pub fn konst(s: &str) -> Ticks {
        <Ticks as TickInt>::from_dec_str(s)
            .expect("invalid decimal literal in a profile constant")
    }

    // Keep `vec` referenced under no_std+alloc without std's prelude.
    #[allow(dead_code)]
    fn _vec_used() -> alloc::vec::Vec<u8> {
        vec![0u8; 0]
    }
}

pub use imp::{konst, Ticks};

/// Whether the active backend makes the public value types `Copy` (§3.2).
pub const TICKS_IS_COPY: bool = cfg!(feature = "u512");
