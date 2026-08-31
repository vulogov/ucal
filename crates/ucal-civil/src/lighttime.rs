//! E2 — light-travel time, in units that behave differently on purpose.
//!
//! # Three units, three kinds of answer
//!
//! `c = 299 792 458 m/s` is exact **by definition** — the metre is defined from
//! it. So is the astronomical unit, since IAU 2012 Resolution B2:
//! `1 au = 149 597 870 700 m`, a decision rather than a measurement. From those:
//!
//! | unit | light-travel time | kind |
//! |---|---|---|
//! | **1 light-year** | **31 557 600 s exactly** | an integer |
//! | 1 au | `1024642950/2053373` s = 499.0047838… s | an exact rational |
//! | 1 parsec | `648000/π` au | **irrational** |
//!
//! The light-year is the joke that turns out to be true. It is *defined* as a
//! Julian year times `c`, so its light-travel time is a Julian year and the
//! conversion is the identity — **a light-year is a time unit wearing a
//! distance's clothes**, and this crate can say so with no arithmetic at all.
//!
//! The parsec is the interesting one: `648000/π au` is an exact definition of an
//! irrational number, so it can only be **bracketed**. Two of the three convert
//! exactly and the third cannot, for a reason that is about the definition
//! rather than about this code.
//!
//! # B1 — and the barycentric bound falls out of it
//!
//! `|BJD − JD| ≤ 1 au / c`, for any target and any date, because the Earth is
//! never more than an astronomical unit from the solar-system barycentre in the
//! direction that matters. The *value* of a barycentric correction needs an
//! ephemeris, which [`S1`] puts out of bounds; the **bound** needs nothing, and
//! it answers the question most people asking actually have: *is my measurement
//! even sensitive to this?* A transit timed to a minute is not. A pulsar
//! residual is, by six orders of magnitude.
//!
//! [`S1`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/S1-astrophysics-roadmap.md

use ucal_core::num::{RatInterval, Ratio};
use ucal_core::{Code, Profile, TimeError, UC1};

type Result<T> = core::result::Result<T, TimeError>;

/// The speed of light in vacuum, m/s. Exact by definition (SI, 1983).
pub const C_M_PER_S: u64 = 299_792_458;

/// The astronomical unit in metres. Exact by definition (IAU 2012 B2).
pub const AU_METRES: u64 = 149_597_870_700;

/// The light-year in metres. Exact: a Julian year times `c`.
pub const LIGHT_YEAR_METRES: u64 = 9_460_730_472_580_800;

/// `1 pc = 648000/π au`, exactly — the definition of an irrational number.
pub const PARSEC_AU_NUMERATOR: u64 = 648_000;

/// `π`, bracketed.
///
/// A mathematical constant rather than a measurement, so it needs no citation
/// under Rule C — but it does need to be *bracketed* rather than rounded,
/// because a parsec's light-travel time is irrational and a decimal for it would
/// be a value that is neither exact nor honest about it.
///
/// Fifty digits, taken from the standard expansion. The test below checks the
/// bracket against convergents anyone can verify — `333/106 < π < 355/113` — so
/// a transposed digit fails rather than silently widening or, worse, narrowing.
const PI_LO: &str = "3.14159265358979323846264338327950288419716939937510";
const PI_HI: &str = "3.14159265358979323846264338327950288419716939937511";

/// A distance, in one of the units this can convert.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum Unit {
    /// Metres. `c` is defined in them, so this is the base.
    Metre,
    /// Astronomical units. Exact since IAU 2012 B2.
    Au,
    /// Light-years. Exact, and the conversion is the identity.
    LightYear,
    /// Parsecs. `648000/π` au, and therefore irrational.
    Parsec,
}

impl Unit {
    /// The name the CLI uses.
    pub const fn key(self) -> &'static str {
        match self {
            Unit::Metre => "m",
            Unit::Au => "au",
            Unit::LightYear => "ly",
            Unit::Parsec => "pc",
        }
    }

    /// Parse a unit.
    pub fn parse(s: &str) -> Result<Unit> {
        match s {
            "m" | "metre" | "meter" => Ok(Unit::Metre),
            "au" | "AU" => Ok(Unit::Au),
            "ly" | "lightyear" | "light-year" => Ok(Unit::LightYear),
            "pc" | "parsec" => Ok(Unit::Parsec),
            _ => Err(TimeError::with_context(
                Code::E0016,
                "no such distance unit. `m`, `au`, `ly` and `pc`; the first \
                 three convert exactly and `pc` is bracketed, because it is \
                 defined as 648000/π au",
            )),
        }
    }

    /// Whether a distance in this unit converts to an exact time.
    pub const fn is_exact(self) -> bool {
        !matches!(self, Unit::Parsec)
    }
}

/// π, as an interval.
fn pi() -> Result<RatInterval> {
    RatInterval::new(
        Ratio::from_decimal_str(PI_LO)?,
        Ratio::from_decimal_str(PI_HI)?,
    )
}

/// Light-travel time for a distance, in **ticks**, as an interval.
///
/// Exact units give a zero-width interval. A parsec gives a bracket, because
/// `648000/π` is irrational and no decimal for it is the value.
pub fn light_time(distance: &Ratio, unit: Unit) -> Result<RatInterval> {
    let second = UC1::bridge().ticks;
    let per_second = Ratio::from_int(second);

    // Metres to seconds: `d / c`, exactly, because `c` is a defining integer.
    let metres_to_ticks = |m: &Ratio| -> Result<Ratio> {
        m.div(&Ratio::from_u64(C_M_PER_S))?.mul(&per_second)
    };

    match unit {
        Unit::Metre => {
            let t = metres_to_ticks(distance)?;
            Ok(RatInterval::exact(t))
        }
        Unit::Au => {
            let m = distance.mul(&Ratio::from_u64(AU_METRES))?;
            Ok(RatInterval::exact(metres_to_ticks(&m)?))
        }
        Unit::LightYear => {
            // The identity, and the reason it is worth a branch of its own: a
            // light-year *is* a Julian year of light-travel time by definition,
            // so this multiplies rather than dividing and cannot lose anything.
            let m = distance.mul(&Ratio::from_u64(LIGHT_YEAR_METRES))?;
            Ok(RatInterval::exact(metres_to_ticks(&m)?))
        }
        Unit::Parsec => {
            // `d pc = d · 648000/π au`. Dividing by the interval for π means the
            // *larger* π gives the *smaller* distance, so the ends swap — which
            // is exactly the kind of thing interval arithmetic exists to get
            // right and hand-rolled bounds get wrong.
            let p = pi()?;
            let au_hi = distance
                .mul(&Ratio::from_u64(PARSEC_AU_NUMERATOR))?
                .div(p.lo())?;
            let au_lo = distance
                .mul(&Ratio::from_u64(PARSEC_AU_NUMERATOR))?
                .div(p.hi())?;
            let to_ticks = |au: &Ratio| -> Result<Ratio> {
                metres_to_ticks(&au.mul(&Ratio::from_u64(AU_METRES))?)
            };
            RatInterval::new(to_ticks(&au_lo)?, to_ticks(&au_hi)?)
        }
    }
}

/// **B1** — the largest a barycentric correction can be, in ticks.
///
/// `1 au / c`, which is 499.004783836… s. The bound holds for any target and any
/// date and needs no ephemeris; the *value* needs one, and [`S1`] puts that out
/// of bounds deliberately.
pub fn barycentric_bound() -> Result<Ratio> {
    let one = Ratio::one();
    Ok(light_time(&one, Unit::Au)?.lo().clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::backend::TickInt;
    use ucal_core::{Rounding, Ticks};

    fn ticks_to_seconds(t: &Ratio) -> Result<Ratio> {
        t.div(&Ratio::from_int(UC1::bridge().ticks))
    }

    /// **A light-year of light-travel time is a Julian year, exactly.**
    ///
    /// 31 557 600 s, an integer, with no remainder at all — because the
    /// light-year is *defined* as that year times `c`. If this ever fails,
    /// somebody has changed a definition.
    #[test]
    fn a_light_year_is_a_julian_year_of_light() {
        let t = light_time(&Ratio::one(), Unit::LightYear).expect("exact");
        assert!(t.is_exact(), "a light-year converts exactly");
        let s = ticks_to_seconds(t.lo()).expect("in range");
        assert!(s.is_integer(), "and to a whole number of seconds");
        assert_eq!(s.numer().to_dec_string(), "31557600");
    }

    /// The astronomical unit is an exact rational, and the published figure.
    #[test]
    fn an_astronomical_unit_is_499_seconds_exactly() {
        let t = light_time(&Ratio::one(), Unit::Au).expect("exact");
        assert!(t.is_exact());
        let s = ticks_to_seconds(t.lo()).expect("in range");
        assert_eq!(
            s.to_decimal_string(9, Rounding::Trunc).expect("rendered"),
            "499.004783836"
        );
        // Exact means exact: the rational is 1024642950/2053373.
        assert_eq!(s.to_ratio_string(), "1024642950/2053373");
    }

    /// **A parsec is bracketed, and the bracket is narrow but not zero.**
    ///
    /// The one unit here that cannot convert exactly, for a reason about its
    /// definition rather than about this code.
    #[test]
    fn a_parsec_is_an_interval() {
        let t = light_time(&Ratio::one(), Unit::Parsec).expect("bracketed");
        assert!(!t.is_exact(), "648000/π is irrational");
        let lo = ticks_to_seconds(t.lo()).expect("in range");
        let hi = ticks_to_seconds(t.hi()).expect("in range");
        for v in [&lo, &hi] {
            assert!(
                v.to_decimal_string(3, Rounding::Trunc)
                    .expect("rendered")
                    .starts_with("102927125."),
                "{}",
                v.to_decimal_string(3, Rounding::Trunc).expect("rendered")
            );
        }
        assert_eq!(lo.cmp_exact(&hi), core::cmp::Ordering::Less);
    }

    /// The π bracket contains π, checked against convergents anyone can verify.
    ///
    /// `333/106 < π < 355/113` is the classic pair, and it needs no float and no
    /// trust in the fifty digits above: a transposed digit that moved the
    /// bracket outside these fails here.
    #[test]
    fn the_pi_bracket_is_where_pi_is() {
        let p = pi().expect("a bracket");
        let below = Ratio::new(
            <Ticks as TickInt>::from_u64(333),
            <Ticks as TickInt>::from_u64(106),
        )
        .expect("a convergent");
        let above = Ratio::new(
            <Ticks as TickInt>::from_u64(355),
            <Ticks as TickInt>::from_u64(113),
        )
        .expect("a convergent");
        assert_eq!(below.cmp_exact(p.lo()), core::cmp::Ordering::Less);
        assert_eq!(above.cmp_exact(p.hi()), core::cmp::Ordering::Greater);
        assert_eq!(p.lo().cmp_exact(p.hi()), core::cmp::Ordering::Less);
    }

    /// **B1** — the barycentric bound is one astronomical unit of light-time.
    #[test]
    fn the_barycentric_bound_is_an_au_of_light() {
        let b = barycentric_bound().expect("a bound");
        let s = ticks_to_seconds(&b).expect("in range");
        assert_eq!(
            s.to_decimal_string(6, Rounding::Trunc).expect("rendered"),
            "499.004783"
        );
    }

    /// Distance scales linearly, which is the one property worth asserting.
    #[test]
    fn twice_the_distance_is_twice_the_time() {
        for unit in [Unit::Metre, Unit::Au, Unit::LightYear, Unit::Parsec] {
            let one = light_time(&Ratio::one(), unit).expect("converts");
            let two = light_time(&Ratio::from_u64(2), unit).expect("converts");
            let doubled = one.lo().mul(&Ratio::from_u64(2)).expect("in range");
            assert_eq!(
                doubled.cmp_exact(two.lo()),
                core::cmp::Ordering::Equal,
                "{unit:?} is not linear"
            );
        }
    }
}
