//! What kind of number a printed number is.
//!
//! # The gap this closes
//!
//! The project's claim is that every quantity is exact, a certified enclosure,
//! or a declared rounding. Until 0.4.0 the *output* did not say which, and one
//! line of `ucal ladder` shows the cost:
//!
//! ```text
//! T5     85   deep / deeps      298023223876953125.000000   13936046862139962.492833
//! T-3    45   spark / sparks                     0.000000                   0.000000
//! ```
//!
//! Three different kinds of number, rendered identically, with the mode
//! undeclared in all three cases:
//!
//! - `T5` in **beats** is exact. A tier is a whole power of five of the beat, so
//!   above `T0` the expansion terminates at once and those six zeros *are* the
//!   value.
//! - `T-3` in **beats** is a rounding, and rounds to something a reader will
//!   misread. `5^45 / 5^60 = 5^-15 ≈ 3.2768 × 10^-11` — a finite expansion, but
//!   fifteen places long, so six digits render it `0.000000`. Four tiers do
//!   this. Nothing in the output says the value is not zero.
//! - **Bridge seconds** never terminate at any digit count: `5^e / SECOND`
//!   carries `18 548 584 399 861` in its denominator, which is neither a power
//!   of two nor of five.
//!
//! The first is a labelling gap. The second is an accuracy defect that shipped.
//!
//! # Decided, not annotated
//!
//! A [`Certification`] is computed from the value at the moment it is rendered —
//! [`Ratio::terminates_at`](ucal_core::num::Ratio::terminates_at) answers it by
//! arithmetic. Nothing here is a hand-written label that could drift from what
//! the renderer actually did.
//!
//! # Where it is reported
//!
//! Exactness is the expectation, so only the **exceptions** are listed: a
//! `certification` block naming the fields that are roundings, with the mode and
//! digit count that produced them. A reader who sees a field absent from that
//! block is being told it is exact, and that is a claim the tests in
//! `certification.rs` enforce rather than a convention.

use core::fmt;

use ucal_core::Rounding;

/// What kind of number a rendered value is.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum Certification {
    /// The printed digits are the value. The expansion terminated.
    Exact,
    /// The printed digits are a rounding of something longer.
    Rounded {
        /// The mode applied. Rule R makes this a choice, so it is stated.
        mode: Rounding,
        /// How many fractional digits were kept.
        digits: u32,
    },
    /// One bound of a certified pair: the value lies between this and its
    /// partner, and the pair is the answer rather than either half of it.
    Enclosure,
}

impl Certification {
    /// Whether this needs reporting. Exact is the expectation.
    pub fn is_exact(self) -> bool {
        matches!(self, Certification::Exact)
    }

    /// The certification of `r` rendered to `digits` under `mode`.
    ///
    /// Falls back to `Rounded` when the exactness test itself cannot be
    /// evaluated — the scaling overflows the domain — because claiming `Exact`
    /// on a question that could not be answered is the one wrong answer here.
    pub fn of_ratio(r: &ucal_core::Ratio, digits: u32, mode: Rounding) -> Certification {
        match r.terminates_at(digits) {
            Ok(true) => Certification::Exact,
            _ => Certification::Rounded { mode, digits },
        }
    }
}

impl fmt::Display for Certification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Certification::Exact => write!(f, "exact"),
            Certification::Rounded { mode, digits } => {
                write!(f, "rounded, {}, {digits} digits", mode_name(*mode))
            }
            Certification::Enclosure => write!(f, "enclosure"),
        }
    }
}

/// The spelling used on the `--round` flag, so the report and the option agree.
pub fn mode_name(m: Rounding) -> &'static str {
    match m {
        Rounding::Trunc => "trunc",
        Rounding::Ceil => "ceil",
        Rounding::HalfEven => "half-even",
        Rounding::HalfUp => "half-up",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::Ratio;

    fn ratio(n: u64, d: u64) -> Ratio {
        Ratio::from_u64(n).div(&Ratio::from_u64(d)).unwrap()
    }

    #[test]
    fn a_terminating_expansion_is_exact() {
        // 1/2, 1/4, 1/8 and 1/5 all terminate inside six digits.
        for (n, d) in [(1u64, 2u64), (1, 4), (1, 8), (1, 5), (3, 25), (7, 1)] {
            assert_eq!(
                Certification::of_ratio(&ratio(n, d), 6, Rounding::HalfEven),
                Certification::Exact,
                "{n}/{d} should be exact at six digits"
            );
        }
    }

    #[test]
    fn a_repeating_expansion_is_rounded() {
        for (n, d) in [(1u64, 3u64), (1, 7), (2, 11)] {
            assert!(
                !Certification::of_ratio(&ratio(n, d), 6, Rounding::HalfEven).is_exact(),
                "{n}/{d} does not terminate and must not be called exact"
            );
        }
    }

    #[test]
    fn exactness_depends_on_the_digit_count() {
        // 1/8 = 0.125: exact at three digits, a rounding at two.
        let r = ratio(1, 8);
        assert!(Certification::of_ratio(&r, 3, Rounding::HalfEven).is_exact());
        assert!(!Certification::of_ratio(&r, 2, Rounding::HalfEven).is_exact());
    }

    #[test]
    fn the_ladder_columns_certify_three_different_ways() {
        // The defect that anchored 0.4.0, as a test — including the part the
        // first draft of this test got wrong. Above T0 a tier in beats is a
        // whole number and exact at any digit count. *Below* T0 it is still a
        // finite expansion, because the denominator is a power of five, but it
        // is 5k places long and six digits do not reach it. Bridge seconds never
        // terminate at any digit count at all.
        use ucal_core::{Profile, Tier, UC1};
        let beat = Tier::BEAT.ticks();
        let second = <UC1 as Profile>::bridge().ticks;
        for k in [5i8, 4, 1, 0] {
            let t = Tier::new(k).unwrap();
            assert!(
                Certification::of_ratio(&crate::ratio_of(&t.ticks(), &beat), 6, Rounding::HalfEven)
                    .is_exact(),
                "T{k} in beats is a whole number and must certify exact"
            );
        }
        for k in [-2i8, -3, -12] {
            let t = Tier::new(k).unwrap();
            let r = crate::ratio_of(&t.ticks(), &beat);
            assert!(
                !Certification::of_ratio(&r, 6, Rounding::HalfEven).is_exact(),
                "T{k} in beats does not fit six digits and must not certify exact"
            );
            // And it renders as zero, which is the part that misleads.
            assert_eq!(
                r.to_decimal_string(6, Rounding::HalfEven).unwrap(),
                "0.000000",
                "T{k} should be the case that reads as zero"
            );
            assert!(!r.is_zero(), "T{k} is not actually zero");
            // Given enough digits it *is* exact — the expansion is finite.
            assert!(
                Certification::of_ratio(&r, 60, Rounding::HalfEven).is_exact(),
                "T{k} in beats terminates eventually"
            );
        }
        for k in [5i8, 0, -12] {
            let t = Tier::new(k).unwrap();
            let r = crate::ratio_of(&t.ticks(), &second);
            for digits in [6u32, 30, 60] {
                assert!(
                    !Certification::of_ratio(&r, digits, Rounding::HalfEven).is_exact(),
                    "T{k} in bridge seconds cannot terminate at {digits} digits"
                );
            }
        }
    }

    #[test]
    fn the_mode_name_matches_the_flag() {
        // The report and `--round` must spell the mode the same way, or a reader
        // cannot act on what they are told.
        for (m, s) in [
            (Rounding::Trunc, "trunc"),
            (Rounding::Ceil, "ceil"),
            (Rounding::HalfEven, "half-even"),
            (Rounding::HalfUp, "half-up"),
        ] {
            assert_eq!(mode_name(m), s);
            assert!(crate::parse_rounding(s).is_ok(), "`{s}` is not accepted by --round");
        }
    }

    #[test]
    fn a_certification_says_enough_to_act_on() {
        let c = Certification::Rounded {
            mode: Rounding::HalfEven,
            digits: 6,
        };
        let s = c.to_string();
        assert!(s.contains("rounded"));
        assert!(s.contains("half-even"));
        assert!(s.contains("6 digits"));
        assert_eq!(Certification::Exact.to_string(), "exact");
    }
}
