//! Rule K's derivation mechanism (§9.5, §9.6).
//!
//! # One mechanism, no tables
//!
//! Rule K.1 and K.2: units come from the body's periods, and intercalation comes
//! from a continued-fraction expansion of their ratio. No calendar may declare a
//! unit length and no calendar may declare an intercalation rule. Everything here
//! is computed from three numbers and a citation.
//!
//! §15.2 requires the derivation to be **deterministic** and to return the **full
//! convergent sequence walked**, so a derived calendar is auditable end to end:
//! a reader can see not only the rule chosen but every rule rejected and why.
//!
//! # What the mechanism recovers, unprompted
//!
//! Applied to Earth it produces the Julian rule 1/4 as its first convergent, and
//! then improves on it — reaching 31/128 at the default bound, which is 124 times
//! more accurate than the Gregorian 97/400 while using a denominator three times
//! smaller. **97/400 never appears at any depth** (§21.3-6). Applied to Earth's
//! Moon it recovers the 19-year Metonic cycle with no special-casing at all.
//!
//! That the mechanism reproduces a rule known since antiquity, and rejects one
//! adopted in 1582, is the strongest available evidence that it is a derivation
//! rather than a rationalisation.
//!
//! # Two corrections carried here
//!
//! - **D-A5**: grouping comes from a satellite the *calendar names*, not from a
//!   global bound. The bound was calibrated on Earth's Moon, which put an
//!   Earth-derived constant inside the one mechanism Rule K exists to keep
//!   Earth-free.
//! - **D-A12**: a synodic period is measured against the primary's **year**, not
//!   its solar day. §9.6 writes the latter, which computes the interval between
//!   moonrises rather than a phase cycle.

use alloc::vec::Vec;

use ucal_core::num::{cf_expand, convergents, Ratio};
use ucal_core::{Code, TimeError};

use crate::body::Body;

type Result<T> = core::result::Result<T, TimeError>;

/// How far a calendar may drift before the rule is considered inadequate.
///
/// Expressed as **local days per local years** — the body's own days and its own
/// years. That matters: a bound in SI seconds would be an Earth-derived constant
/// inside the derivation mechanism, which is the mistake D-A5 records for the
/// grouping bounds. A bound of "one day per ten thousand years" means the same
/// thing on Mars as on Earth without meaning the same *duration*.
///
/// §9.5 types this parameter as a `Delta` — a tick count. A tick count is a
/// duration, and a drift bound is a rate, so the two cannot be the same type; see
/// `spec/SPEC-DELTAS.md` D-A13.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct DriftBound {
    /// Local days of drift permitted.
    pub days: u64,
    /// Over this many local years.
    pub per_years: u64,
}

impl DriftBound {
    /// D-12's default: one local day per ten thousand local years.
    pub const DEFAULT: DriftBound = DriftBound {
        days: 1,
        per_years: 10_000,
    };

    /// The bound as an exact rational: days per year.
    pub fn as_ratio(&self) -> Result<Ratio> {
        Ratio::new(
            ucal_core::backend::TickInt::from_u64(self.days),
            ucal_core::backend::TickInt::from_u64(self.per_years),
        )
    }
}

impl Default for DriftBound {
    fn default() -> Self {
        DriftBound::DEFAULT
    }
}

/// One convergent of a continued fraction, with its exact error.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Convergent {
    /// The approximating fraction.
    pub value: Ratio,
    /// `|value − target|`, exactly.
    pub error: Ratio,
    /// How many local years before one local day of drift accumulates.
    ///
    /// `None` when the convergent is exact, which the last one always is for a
    /// rational target.
    pub one_day_slips_in: Option<Ratio>,
}

/// A derived intercalation rule (§9.5).
///
/// Carries the whole sequence walked, not merely the answer, so the choice can be
/// audited (§15.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LeapRule {
    /// `orbital_period / solar_day`, exactly.
    pub ratio: Ratio,
    /// Whole local days in a local year.
    pub whole_days: Ratio,
    /// The fractional part the rule approximates.
    pub fraction: Ratio,
    /// The full continued-fraction expansion of the fractional part.
    pub continued_fraction: Vec<u64>,
    /// Every convergent walked, in order — including those rejected.
    pub walked: Vec<Convergent>,
    /// Which convergent was chosen, 1-based.
    pub depth: usize,
    /// The chosen rule: `numerator` leap days per `denominator` local years.
    pub chosen: Convergent,
    /// The bound that was asked for.
    pub bound: DriftBound,
}

impl LeapRule {
    /// Leap days per cycle.
    pub fn leap_days(&self) -> &Ratio {
        &self.chosen.value
    }

    /// Whether a given fraction appears anywhere in the walked sequence.
    ///
    /// §21.3-6 requires a test that 97/400 does not, at any depth.
    pub fn contains(&self, numerator: u64, denominator: u64) -> bool {
        let Ok(target) = Ratio::new(
            ucal_core::backend::TickInt::from_u64(numerator),
            ucal_core::backend::TickInt::from_u64(denominator),
        ) else {
            return false;
        };
        self.walked.iter().any(|c| c.value == target)
    }
}

/// Derive an intercalation rule (§9.5, Rule K.2).
///
/// Forms `orbital_period / solar_day` exactly, splits off the whole day count,
/// expands the fraction as a continued fraction, and walks the convergents in
/// order until one meets the drift bound. `UCAL-E0061` if none does within
/// `max_depth`.
///
/// The whole walk is returned regardless, so a caller can see what was rejected.
pub fn derive_leap_rule(
    solar_day: &Ratio,
    orbital_period: &Ratio,
    bound: DriftBound,
    max_depth: u32,
) -> Result<LeapRule> {
    if solar_day.is_zero() {
        return Err(TimeError::with_context(
            Code::E0061,
            "a body with no solar day has no days to intercalate",
        ));
    }
    let ratio = orbital_period.div(solar_day)?;
    let whole = Ratio::from_int(ratio.floor());
    let fraction = ratio.frac();
    let limit = bound.as_ratio()?;

    let cf = cf_expand(&fraction, max_depth);
    // The leading term of a proper fraction's expansion is 0; it is not a
    // candidate rule, so the walk starts at the next one.
    let all = convergents(&cf);
    let mut walked = Vec::new();
    let mut chosen = None;

    for (i, c) in all.iter().enumerate().skip(1) {
        let error = c.abs_diff(&fraction)?;
        let one_day_slips_in = if error.is_zero() {
            None
        } else {
            Some(error.recip()?)
        };
        let entry = Convergent {
            value: c.clone(),
            error: error.clone(),
            one_day_slips_in,
        };
        let meets = error.cmp_exact(&limit) != core::cmp::Ordering::Greater;
        walked.push(entry.clone());
        if meets && chosen.is_none() {
            chosen = Some((i, entry));
            // Keep walking: §15.2 wants the full sequence, not a truncated one.
        }
    }

    let Some((depth, chosen)) = chosen else {
        return Err(TimeError::with_context(
            Code::E0061,
            "no convergent within the permitted depth meets the requested drift \
             bound; either widen the bound or raise the depth",
        ));
    };

    Ok(LeapRule {
        ratio,
        whole_days: whole,
        fraction,
        continued_fraction: cf,
        walked,
        depth,
        chosen,
        bound,
    })
}

/// A derived grouping cycle (§9.6, Rule K.3 as amended by D-A5).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Cycle {
    /// Which satellite the calendar named.
    pub satellite: &'static str,
    /// The satellite's synodic period, in ticks. Measured against the primary's
    /// **year** (D-A12).
    pub synodic_period: Ratio,
    /// `orbital_period / synodic_period` — cycles per local year.
    pub ratio: Ratio,
    /// The full continued-fraction expansion.
    pub continued_fraction: Vec<u64>,
    /// Every convergent, in order: the candidate commensurability rules.
    pub convergents: Vec<Convergent>,
}

impl Cycle {
    /// Whether a given commensurability appears among the convergents.
    ///
    /// §21.3-7 requires a test that Earth's sequence contains 235/19.
    pub fn contains(&self, numerator: u64, denominator: u64) -> bool {
        let Ok(target) = Ratio::new(
            ucal_core::backend::TickInt::from_u64(numerator),
            ucal_core::backend::TickInt::from_u64(denominator),
        ) else {
            return false;
        };
        self.convergents.iter().any(|c| c.value == target)
    }
}

/// Derive a calendar's grouping cycles (§9.6, Rule K.3 as amended).
///
/// # The amendment, restated
///
/// D-A5 replaces §9.6's global admission bound with a satellite the calendar
/// *names*. A calendar declaring none has no cycles — which is not a fallback and
/// not an error at construction, but a statement about that calendar. Requesting a
/// cycle field from it is `UCAL-E0064`.
///
/// The change matters because "month-like" is not derivable: *month* is an Earth
/// predicate, and any absolute bracket for it is calibrated on Earth's Moon. What
/// **is** derivable, once a satellite is named, is the commensurability structure
/// — and that is what this returns.
pub fn derive_cycles(
    body: &Body,
    grouping_satellite: Option<&str>,
    max_depth: u32,
) -> Result<Vec<Cycle>> {
    let Some(id) = grouping_satellite else {
        // Declared no grouping satellite. The empty result is the correct output.
        return Ok(Vec::new());
    };
    let Some(sat) = body.satellite(id) else {
        return Err(TimeError::with_context(
            Code::E0064,
            "the calendar names a grouping satellite the body does not have",
        ));
    };

    let year = body.orbital_period().value_at_epoch();
    let synodic = sat.synodic_period(year)?;
    let ratio = year.div(&synodic)?;

    let cf = cf_expand(&ratio, max_depth);
    let mut out = Vec::new();
    for c in convergents(&cf) {
        let error = c.abs_diff(&ratio)?;
        let one_day_slips_in = if error.is_zero() {
            None
        } else {
            Some(error.recip()?)
        };
        out.push(Convergent {
            value: c,
            error,
            one_day_slips_in,
        });
    }

    Ok(alloc::vec![Cycle {
        satellite: sat.id(),
        synodic_period: synodic,
        ratio,
        continued_fraction: cf,
        convergents: out,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;
    use ucal_core::backend::TickInt;
    use ucal_core::Rounding;

    fn rule_for(body: &Body) -> LeapRule {
        derive_leap_rule(
            body.solar_day().value_at_epoch(),
            body.orbital_period().value_at_epoch(),
            DriftBound::DEFAULT,
            32,
        )
        .expect("a rule must be derivable")
    }

    fn frac(c: &Convergent) -> alloc::string::String {
        alloc::format!(
            "{}/{}",
            c.value.numer().to_dec_string(),
            c.value.denom().to_dec_string()
        )
    }

    // ---- §21.3-6 ----

    #[test]
    fn earth_reproduces_appendix_i1_exactly() {
        let r = rule_for(&data::earth());
        assert_eq!(r.whole_days.numer().to_dec_string(), "365");
        assert_eq!(
            r.continued_fraction[..9],
            [0, 4, 7, 1, 3, 24, 6, 2, 2],
            "Appendix I.1's continued fraction"
        );
        let want = ["1/4", "7/29", "8/33", "31/128", "752/3105", "4543/18758"];
        for (i, w) in want.iter().enumerate() {
            assert_eq!(&frac(&r.walked[i]), w, "convergent {}", i + 1);
        }
    }

    #[test]
    fn the_julian_rule_is_convergent_one() {
        // §21.3-6. The mechanism recovers a rule adopted in 46 BC as its first
        // approximation, from nothing but two periods.
        let r = rule_for(&data::earth());
        assert_eq!(frac(&r.walked[0]), "1/4");
    }

    #[test]
    fn the_gregorian_rule_appears_at_no_depth() {
        // §21.3-6, the load-bearing assertion. 97/400 is a choice, not a
        // consequence — which is why the Gregorian calendar is legacy (§8.6).
        let r = rule_for(&data::earth());
        assert!(
            !r.contains(97, 400),
            "97/400 must not appear at any depth; walked {} convergents: {:?}",
            r.walked.len(),
            r.walked.iter().map(frac).collect::<Vec<_>>()
        );
        // And the mechanism does better with a smaller denominator, which is the
        // RFC's own point.
        let chosen = &r.chosen;
        assert_eq!(frac(chosen), "31/128");
        let gregorian_error = Ratio::from_decimal_str("0.000310")
            .unwrap();
        assert!(
            chosen.error.cmp_exact(&gregorian_error) == core::cmp::Ordering::Less,
            "the derived rule must beat the one it declines to produce"
        );
    }

    #[test]
    fn earth_chooses_31_128_at_the_default_bound() {
        // D-12's default is one day per ten thousand years. The first three
        // convergents miss it; the fourth clears it by two orders.
        let r = rule_for(&data::earth());
        assert_eq!(r.depth, 4);
        assert_eq!(frac(&r.chosen), "31/128");
        // One day slips in 400 000 years.
        assert_eq!(
            r.chosen
                .one_day_slips_in
                .as_ref()
                .unwrap()
                .to_decimal_string(0, Rounding::HalfEven)
                .unwrap(),
            "400000"
        );
        // The rejected ones are still reported (§15.2).
        assert!(r.walked.len() > r.depth, "the full walk must be returned");
        for i in 0..3 {
            assert!(
                r.walked[i].error.cmp_exact(&r.bound.as_ratio().unwrap())
                    == core::cmp::Ordering::Greater,
                "convergent {} should have been rejected",
                i + 1
            );
        }
    }

    #[test]
    fn mars_reproduces_appendix_i3_and_chooses_45_76() {
        let r = rule_for(&data::mars());
        assert_eq!(r.whole_days.numer().to_dec_string(), "668");
        assert_eq!(r.continued_fraction[..9], [0, 1, 1, 2, 4, 1, 2, 2, 1]);
        let want = ["1/1", "1/2", "3/5", "13/22", "16/27", "45/76", "106/179"];
        for (i, w) in want.iter().enumerate() {
            assert_eq!(&frac(&r.walked[i]), w, "convergent {}", i + 1);
        }
        // At the same bound Mars lands on its sixth convergent.
        assert_eq!(r.depth, 6);
        assert_eq!(frac(&r.chosen), "45/76");
    }

    #[test]
    fn the_bound_is_body_relative_not_earth_calibrated() {
        // The same bound, stated in each body's own days and years, selects
        // different rules — because it is a statement about that calendar's
        // accuracy, not about a duration.
        let earth = rule_for(&data::earth());
        let mars = rule_for(&data::mars());
        assert_ne!(frac(&earth.chosen), frac(&mars.chosen));
        assert_eq!(earth.bound, mars.bound);
    }

    #[test]
    fn a_tighter_bound_walks_further() {
        let tight = DriftBound {
            days: 1,
            per_years: 100_000_000,
        };
        let r = derive_leap_rule(
            data::earth().solar_day().value_at_epoch(),
            data::earth().orbital_period().value_at_epoch(),
            tight,
            32,
        )
        .unwrap();
        assert!(r.depth > 4, "a tighter bound must reject 31/128");
        assert_eq!(frac(&r.chosen), "4543/18758");
    }

    #[test]
    fn an_unreachable_bound_is_e0061() {
        // §9.5: if no depth meets the bound, UCAL-E0061.
        let r = derive_leap_rule(
            data::earth().solar_day().value_at_epoch(),
            data::earth().orbital_period().value_at_epoch(),
            DriftBound::DEFAULT,
            3, // too shallow to reach 31/128
        );
        assert_eq!(r.unwrap_err().code, Code::E0061);
    }

    #[test]
    fn derivation_is_deterministic() {
        // §15.2.
        let a = rule_for(&data::earth());
        let b = rule_for(&data::earth());
        assert_eq!(a, b);
    }

    // ---- §21.3-7 ----

    #[test]
    fn earth_derives_the_metonic_cycle() {
        // 235 synodic months in 19 tropical years, recovered from the tick, the
        // datum and the body's periods — with no special-casing, from parameters
        // chosen for Appendix I.1 rather than for this.
        let earth = data::earth();
        let cycles = derive_cycles(&earth, Some("moon"), 32).unwrap();
        assert_eq!(cycles.len(), 1);
        let c = &cycles[0];
        assert_eq!(c.satellite, "moon");
        assert!(
            c.contains(235, 19),
            "the Metonic cycle must appear: {:?}",
            c.convergents.iter().map(frac).collect::<Vec<_>>()
        );
        // Appendix I.2's own convergents, in order.
        let want = ["12/1", "25/2", "37/3", "99/8", "136/11", "235/19", "4131/334"];
        for (i, w) in want.iter().enumerate() {
            assert_eq!(&frac(&c.convergents[i]), w, "convergent {}", i + 1);
        }
    }

    #[test]
    fn mars_yields_no_cycle() {
        // §21.3-7's second half. Under D-A5 this is a statement about `mars-d`'s
        // declared data rather than about a global bound: Mars's calendar names
        // no grouping satellite, so it has no cycles.
        let mars = data::mars();
        assert!(derive_cycles(&mars, None, 32).unwrap().is_empty());
        // The satellites exist; the calendar simply does not name one.
        assert_eq!(mars.satellites().len(), 2);
    }

    #[test]
    fn naming_a_satellite_the_body_lacks_is_e0064() {
        let e = derive_cycles(&data::mars(), Some("europa"), 32).unwrap_err();
        assert_eq!(e.code, Code::E0064);
    }

    #[test]
    fn mars_satellites_would_yield_nothing_month_like_anyway() {
        // D-A12's consequence, recorded as a fact rather than an argument: even if
        // `mars-d` named a satellite, neither phase cycle resembles a month. Under
        // the corrected formula Deimos is 1.23 sols, not the 5.36 Appendix I.4
        // printed — so Appendix I.4's conclusion was right, by a route its own
        // working did not support.
        let mars = data::mars();
        let sol = mars.solar_day().value_at_epoch();
        for (id, want) in [("phobos", "0.3105"), ("deimos", "1.2315")] {
            let cycles = derive_cycles(&mars, Some(id), 32).unwrap();
            let syn = &cycles[0].synodic_period;
            assert_eq!(
                syn.div(sol).unwrap().to_decimal_string(4, Rounding::HalfEven).unwrap(),
                want
            );
        }
    }

    #[test]
    fn a_calendar_with_no_grouping_satellite_is_not_an_error() {
        // Rule K.3 as amended: the empty result is a statement, not a failure.
        // `UCAL-E0064` belongs at the point a cycle field is *requested*.
        for body in [data::earth(), data::mars(), data::titan()] {
            assert!(derive_cycles(&body, None, 32).unwrap().is_empty());
        }
    }

    #[test]
    fn titan_derives_a_rule_from_its_own_parameters() {
        // Titan diverges from Appendix I.5 (see `data::titan`), so its derived
        // rule is not I.5's. It is still a rule, derived by the identical path —
        // which is the point of Rule K.5.
        let r = rule_for(&data::titan());
        assert_eq!(r.whole_days.numer().to_dec_string(), "673");
        assert!(!r.walked.is_empty());
        assert!(r.depth >= 1);
        // Not Appendix I.5's 60/61, because the parameters differ.
        assert_ne!(frac(&r.walked[1]), "60/61");
    }
}
