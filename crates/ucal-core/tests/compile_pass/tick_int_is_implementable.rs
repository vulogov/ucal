//! `TickInt` must stay implementable from outside this crate.
//!
//! X3, and the most surprising of the four. `TickInt` is the backend
//! abstraction, and Rule B makes the value width a wire-format commitment: the
//! project ships exactly two backends and refuses to compile both at once. It
//! does not follow that a third is impossible — nothing seals this trait, and an
//! outside crate can supply its own integer and its own `Wide`.
//!
//! Whether that *should* be possible is a design question this fixture does not
//! answer. What it does is make the current answer visible, so that changing it
//! is a decision rather than a side effect: adding a required method here breaks
//! an outside backend, and `cargo semver-checks` will not say so.
//!
//! The implementation below is deliberately wrong as arithmetic — `wide_quot_rem`
//! returns zero — because the property under test is that the trait can be
//! *implemented*, not that this is a usable backend. A correct one would be a
//! second `ucal-core`.

use ucal_core::backend::{TickInt, CANONICAL_BYTES};

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Downstream(u128);

impl TickInt for Downstream {
    type Wide = (u128, u128);

    fn zero() -> Self {
        Downstream(0)
    }
    fn one() -> Self {
        Downstream(1)
    }
    fn domain_max() -> Self {
        Downstream(u128::MAX)
    }
    fn wide_mul(&self, other: &Self) -> Self::Wide {
        (0, self.0.checked_mul(other.0).unwrap_or(0))
    }
    fn wide_quot_rem(_wide: &Self::Wide, _divisor: &Self) -> (Self::Wide, Self) {
        ((0, 0), Downstream(0))
    }
    fn narrow(wide: &Self::Wide) -> Option<Self> {
        Some(Downstream(wide.1))
    }
    fn wide_is_zero(wide: &Self::Wide) -> bool {
        wide.0 == 0 && wide.1 == 0
    }
    fn from_u64(v: u64) -> Self {
        Downstream(v as u128)
    }
    fn from_dec_str(s: &str) -> Option<Self> {
        s.parse().ok().map(Downstream)
    }
    fn try_add(&self, o: &Self) -> Option<Self> {
        self.0.checked_add(o.0).map(Downstream)
    }
    fn try_sub(&self, o: &Self) -> Option<Self> {
        self.0.checked_sub(o.0).map(Downstream)
    }
    fn try_mul(&self, o: &Self) -> Option<Self> {
        self.0.checked_mul(o.0).map(Downstream)
    }
    fn quot_rem(&self, d: &Self) -> (Self, Self) {
        (Downstream(self.0 / d.0), Downstream(self.0 % d.0))
    }
    fn bit_len(&self) -> u32 {
        128 - self.0.leading_zeros()
    }
    fn to_canonical_bytes(&self) -> [u8; CANONICAL_BYTES] {
        [0u8; CANONICAL_BYTES]
    }
    fn from_canonical_bytes(_b: &[u8; CANONICAL_BYTES]) -> Option<Self> {
        Some(Downstream(0))
    }
    fn is_odd(&self) -> bool {
        self.0 % 2 == 1
    }
    fn to_dec_string(&self) -> String {
        self.0.to_string()
    }
    fn to_radix_string(&self, _radix: u32) -> String {
        String::new()
    }
}

fn main() {
    assert_eq!(Downstream::from_u64(7).to_dec_string(), "7");
}
