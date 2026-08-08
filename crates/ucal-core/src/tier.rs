//! The tier grid (§4, Rule G) and tier naming (Rule N).
//!
//! Rule G: tiers are the powers `5^(5k)`, indexed relative to the beat, so
//! `T[k] = 5^(60 + 5k)`. Each tier is exactly five base-5 digits — 3125 units of
//! the tier below. The consequences are the design: a timestamp is the tick count
//! written in base 5 and grouped in fives, truncation *is* rounding, prefix
//! comparison *is* chronological comparison.
//!
//! Rule N: a tier's canonical identity is its **exponent**. Names come from a
//! locale table and are display-and-parse aliases only. Nothing in this module
//! decides behaviour from a name.
//!
//! §13.5 requires the tier table, the locale table and the documentation table to
//! come from one source of truth. That source is [`GRID`] plus [`NAMED`]; the
//! table in the RFC's §4.1 and Appendix B is generated from it, which is how the
//! imprecise seconds column in Appendix B (delta D-A3) stops being possible.

use crate::backend::{TickInt, Ticks};
use crate::error::{Code, Result, TimeError};

/// Digits per tier. Each tier is `5^5 = 3125` units of the tier below.
pub const GROUP_BASE: u16 = 3125;

/// Base-5 digits per tier group.
pub const DIGITS_PER_TIER: u32 = 5;

/// Exponent of the base tier: the beat is `5^60` (D-2).
pub const BEAT_EXPONENT: u32 = 60;

/// Lowest tier index. `T-12 = 5^0` is one tick — the resolution floor (G2, N10).
pub const K_MIN: i8 = -12;

/// Highest tier index. `T32 = 5^220` is 511 bits, the largest power of five the
/// 512-bit domain holds, so the grid cannot extend further without widening the
/// domain — and Rule B makes the width a wire-format commitment.
pub const K_MAX: i8 = 32;

/// Number of tiers in the grid.
pub const TIER_COUNT: usize = (K_MAX as isize - K_MIN as isize + 1) as usize;

/// A tier index, relative to the beat.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Tier(i8);

impl Tier {
    /// The base tier, `5^60`, ~46.762 ms.
    pub const BEAT: Tier = Tier(0);
    /// One tick, `5^0`. The finest addressable interval (G2).
    pub const TICK: Tier = Tier(K_MIN);
    /// `5^85`, ~441.607 Myr.
    pub const DEEP: Tier = Tier(5);
    /// `5^80`, ~141.314 kyr.
    pub const DRIFT: Tier = Tier(4);
    /// `5^75`, ~45.221 yr.
    pub const SPAN: Tier = Tier(3);
    /// `5^70`, ~5.285 d.
    pub const SWEEP: Tier = Tier(2);
    /// `5^65`, ~146.130 s.
    pub const ARC: Tier = Tier(1);
    /// `5^55`, ~14.964 us.
    pub const FLICKER: Tier = Tier(-1);
    /// `5^50`, ~4.788 ns.
    pub const GLINT: Tier = Tier(-2);
    /// `5^45`, ~1.532 ps.
    pub const SPARK: Tier = Tier(-3);

    /// Construct from a tier index, rejecting indices outside the grid.
    pub const fn new(k: i8) -> Result<Tier> {
        if k < K_MIN || k > K_MAX {
            Err(TimeError::with_context(
                Code::E0080,
                "tier index outside [-12, 32]",
            ))
        } else {
            Ok(Tier(k))
        }
    }

    /// Construct from a power-of-five exponent, which must lie on the grid.
    pub const fn from_exponent(exponent: u32) -> Result<Tier> {
        if exponent % DIGITS_PER_TIER != 0 {
            return Err(TimeError::with_context(
                Code::E0080,
                "exponent is not a multiple of 5, so it is not on the 5^(5k) grid. Try `5^60` (the beat) or an index like `T0`; `ucal ladder` lists every rung",
            ));
        }
        // (exponent - 60) / 5, computed without going negative in unsigned space.
        let k = if exponent >= BEAT_EXPONENT {
            ((exponent - BEAT_EXPONENT) / DIGITS_PER_TIER) as i64
        } else {
            -(((BEAT_EXPONENT - exponent) / DIGITS_PER_TIER) as i64)
        };
        if k < K_MIN as i64 || k > K_MAX as i64 {
            return Err(TimeError::with_context(
                Code::E0080,
                "exponent outside the profile grid",
            ));
        }
        Ok(Tier(k as i8))
    }

    /// The tier index.
    pub const fn index(self) -> i8 {
        self.0
    }

    /// The power-of-five exponent. This is the tier's canonical identity (Rule N).
    pub const fn exponent(self) -> u32 {
        // k >= K_MIN = -12 guarantees 60 + 5k >= 0.
        (BEAT_EXPONENT as i64 + DIGITS_PER_TIER as i64 * self.0 as i64) as u32
    }

    /// Whether this is the tick tier, where truncation is the identity.
    pub const fn is_tick(self) -> bool {
        self.0 == K_MIN
    }

    /// The next coarser tier, or `None` at the top of the grid.
    pub const fn coarser(self) -> Option<Tier> {
        if self.0 >= K_MAX {
            None
        } else {
            Some(Tier(self.0 + 1))
        }
    }

    /// The next finer tier, or `None` at the tick.
    pub const fn finer(self) -> Option<Tier> {
        if self.0 <= K_MIN {
            None
        } else {
            Some(Tier(self.0 - 1))
        }
    }

    /// The tier's magnitude in ticks: `5^(60 + 5k)`.
    pub fn ticks(self) -> Ticks {
        <Ticks as TickInt>::pow5(self.exponent())
            .expect("tier grid is bounded so that every tier fits the domain")
    }

    /// Every tier, coarsest first.
    pub fn all_descending() -> impl Iterator<Item = Tier> {
        (K_MIN..=K_MAX).rev().map(Tier)
    }

    /// Every tier, finest first.
    pub fn all_ascending() -> impl Iterator<Item = Tier> {
        (K_MIN..=K_MAX).map(Tier)
    }
}

impl core::fmt::Display for Tier {
    /// Renders as `T<k>`, the notation Rule N requires implementations to accept
    /// wherever a name is accepted.
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "T{}", self.0)
    }
}

/// The stable identity of a named tier. Locale tables map these keys to display
/// strings (Appendix D); the key itself is never localised.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
#[non_exhaustive]
pub enum TierName {
    /// T5
    Deep,
    /// T4
    Drift,
    /// T3
    Span,
    /// T2
    Sweep,
    /// T1
    Arc,
    /// T0
    Beat,
    /// T-1
    Flicker,
    /// T-2
    Glint,
    /// T-3
    Spark,
    /// T-12
    Tick,
}

impl TierName {
    /// The stable key, used in locale tables and `--json` output.
    pub const fn key(self) -> &'static str {
        match self {
            TierName::Deep => "deep",
            TierName::Drift => "drift",
            TierName::Span => "span",
            TierName::Sweep => "sweep",
            TierName::Arc => "arc",
            TierName::Beat => "beat",
            TierName::Flicker => "flicker",
            TierName::Glint => "glint",
            TierName::Spark => "spark",
            TierName::Tick => "tick",
        }
    }
}

/// The single source of truth for tier naming (§13.5, Appendix D).
///
/// Tiers absent from this table are unnamed and addressed by index — D-20 makes
/// that a deliberate choice, and Rule N makes naming them a locale change rather
/// than a specification change.
pub const NAMED: &[(i8, TierName)] = &[
    (5, TierName::Deep),
    (4, TierName::Drift),
    (3, TierName::Span),
    (2, TierName::Sweep),
    (1, TierName::Arc),
    (0, TierName::Beat),
    (-1, TierName::Flicker),
    (-2, TierName::Glint),
    (-3, TierName::Spark),
    (-12, TierName::Tick),
];

/// The name of a tier, if it has one.
pub fn name_of(tier: Tier) -> Option<TierName> {
    let mut i = 0;
    while i < NAMED.len() {
        if NAMED[i].0 == tier.index() {
            return Some(NAMED[i].1);
        }
        i += 1;
    }
    None
}

/// The tier a key names, if any. Accepts only the stable key; locale aliases are
/// resolved by the locale table, not here.
pub fn tier_of_key(key: &str) -> Option<Tier> {
    NAMED
        .iter()
        .find(|(_, n)| n.key() == key)
        .map(|(k, _)| Tier(*k))
}

/// The materialised grid: `5^(60 + 5k)` for every `k`, ascending.
///
/// Built on demand rather than as a `static`, because on the `bigint` backend a
/// `Ticks` is heap-allocated and cannot be a `const`. The default backend's
/// values are all `const`-constructible; §13.5's requirement is that the table
/// be *generated*, which it is — from [`Tier::exponent`], never transcribed.
pub struct TierTable {
    ticks: [Option<Ticks>; TIER_COUNT],
}

impl TierTable {
    /// Materialise the whole grid.
    pub fn build() -> TierTable {
        let mut ticks: [Option<Ticks>; TIER_COUNT] = core::array::from_fn(|_| None);
        for k in K_MIN..=K_MAX {
            let idx = (k as isize - K_MIN as isize) as usize;
            ticks[idx] = Some(Tier(k).ticks());
        }
        TierTable { ticks }
    }

    /// The magnitude of a tier in ticks.
    pub fn get(&self, tier: Tier) -> &Ticks {
        let idx = (tier.index() as isize - K_MIN as isize) as usize;
        self.ticks[idx]
            .as_ref()
            .expect("grid is fully populated by build()")
    }

    /// Number of entries. Always [`TIER_COUNT`].
    pub fn len(&self) -> usize {
        TIER_COUNT
    }

    /// Always false; the grid is never empty. Present to satisfy clippy.
    pub fn is_empty(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_has_45_tiers() {
        assert_eq!(TIER_COUNT, 45);
        assert_eq!(Tier::all_ascending().count(), 45);
    }

    #[test]
    fn exponent_is_canonical_identity() {
        // Rule N: identity is the exponent, and the round trip must be exact.
        for t in Tier::all_ascending() {
            assert_eq!(Tier::from_exponent(t.exponent()).unwrap(), t);
        }
        assert_eq!(Tier::BEAT.exponent(), 60);
        assert_eq!(Tier::TICK.exponent(), 0);
        assert_eq!(Tier::DEEP.exponent(), 85);
        assert_eq!(Tier::new(K_MAX).unwrap().exponent(), 220);
    }

    #[test]
    fn off_grid_exponents_rejected() {
        // 61 is not a multiple of 5, so it is not a tier (Rule G).
        assert_eq!(Tier::from_exponent(61).unwrap_err().code, Code::E0080);
        // 225 would be T33, past the domain.
        assert_eq!(Tier::from_exponent(225).unwrap_err().code, Code::E0080);
    }

    #[test]
    fn each_tier_is_3125_of_the_one_below() {
        let table = TierTable::build();
        let gb = <Ticks as TickInt>::from_u64(GROUP_BASE as u64);
        for k in (K_MIN + 1)..=K_MAX {
            let hi = table.get(Tier::new(k).unwrap());
            let lo = table.get(Tier::new(k - 1).unwrap());
            assert_eq!(
                hi,
                &lo.try_mul(&gb).expect("within domain"),
                "T{k} is not 3125 x T{}",
                k - 1
            );
        }
    }

    #[test]
    fn top_tier_fits_and_next_would_not() {
        // 5^220 is 511 bits; 5^225 exceeds the 512-bit domain. This is why the
        // grid stops at T32 (see K_MAX).
        assert_eq!(<Ticks as TickInt>::pow5(220).unwrap().bit_len(), 511);
        assert!(<Ticks as TickInt>::pow5(225).is_none());
    }

    #[test]
    fn tick_tier_is_one_tick() {
        assert_eq!(Tier::TICK.ticks(), <Ticks as TickInt>::one());
        assert!(Tier::TICK.is_tick());
        assert_eq!(Tier::TICK.finer(), None);
        assert_eq!(Tier::new(K_MAX).unwrap().coarser(), None);
    }

    #[test]
    fn names_are_display_only() {
        assert_eq!(name_of(Tier::BEAT).unwrap().key(), "beat");
        assert_eq!(tier_of_key("deep"), Some(Tier::DEEP));
        // Unnamed tiers are addressable but have no name (D-20).
        assert_eq!(name_of(Tier::new(6).unwrap()), None);
        assert_eq!(name_of(Tier::new(-4).unwrap()), None);
        assert_eq!(tier_of_key("nonexistent"), None);
    }

    #[test]
    fn no_duplicate_names() {
        // Rule N / UCAL-E0011: a collision in the active table is an error.
        for (i, (_, a)) in NAMED.iter().enumerate() {
            for (_, b) in NAMED.iter().skip(i + 1) {
                assert_ne!(a.key(), b.key(), "duplicate tier name key");
            }
        }
    }

    #[test]
    fn display_uses_index_notation() {
        assert_eq!(Tier::BEAT.to_string(), "T0");
        assert_eq!(Tier::DEEP.to_string(), "T5");
        assert_eq!(Tier::TICK.to_string(), "T-12");
    }
}
