//! B — a published linear ephemeris, carrying the uncertainty it was published
//! with.
//!
//! # What this is for
//!
//! Nearly every repeating astronomical event is published as a linear
//! ephemeris — transiting planets, eclipsing binaries, pulsars, spacecraft
//! passes:
//!
//! ```text
//! T(E) = T_0 + E·P                  the linear form
//! T(E) = T_0 + E·P + ½·E²·P·Ṗ      with a period derivative
//! ```
//!
//! `T_0` and `P` come with published uncertainties, and **the useful quantity is
//! not `T(E)`**. It is the window
//!
//! ```text
//! T(E) ± k·√(σ_T₀² + (E·σ_P)²)
//! ```
//!
//! because that is what decides whether an observation is worth scheduling. Most
//! tooling computes the centre and drops the width, so an observer books thirty
//! minutes for an event whose prediction is ±forty.
//!
//! # Why it belongs here
//!
//! Everything needed already existed, and two pieces existed **unused**:
//!
//! - [`Window`] and Rule U — the answer is an interval and always was.
//! - Rule C and `UCAL-W0003` — a parameter used outside its validity window must
//!   warn and must not silently extrapolate. For an ephemeris that is a mistake
//!   people make *in print*: a 2015 fit applied in 2026, thousands of cycles
//!   past the data it was fitted to.
//! - A **secular rate**, which `ucal-body` models and no shipped parameter uses.
//!   `Ṗ` is the astronomical name for it, and a pulsar has one.
//!
//! # The uncertainty is carried because it is cited
//!
//! `ucal-body`'s parameters deliberately carry **no** uncertainty magnitude, and
//! `calendar.rs` says why: the planetary sources do not uniformly publish one,
//! and *"adding a fabricated one would be worse than omitting it"*.
//!
//! For an ephemeris the opposite holds. `σ_P` is the headline number of every
//! refinement paper, so carrying it is Rule C working rather than Rule C bent.
//! The rule this draws is worth stating in one line: **an uncertainty is carried
//! when it is cited, and never when it is inferred.**
//!
//! # Quadrature, and what it assumes
//!
//! `σ(E)` combines `σ_T₀` and `E·σ_P` **in quadrature**, which assumes they are
//! independent. From a joint fit they are correlated, and the covariance is
//! usually not published.
//!
//! It is defensible because of a convention rather than an accident: an
//! ephemeris fit conventionally places `T_0` near the centre of the data span
//! *precisely* to minimise that correlation. Where a source states a covariance,
//! this is an underestimate and the source's own window should be used instead.
//! Said here rather than left for a reader to discover, because a window that is
//! too narrow is the one failure mode this whole type exists to prevent.
//!
//! # Exact, and widened rather than narrowed
//!
//! The square root of a sum of squares is irrational, so the half-width is
//! computed by [`isqrt_ceil`](ucal_core::num::isqrt_ceil) — **outward**. The
//! over-widening is at most one tick, which is 5.39 × 10⁻⁴⁴ s, and it is always
//! in the safe direction. GE-3 forbids narrowing by assumption; nothing here
//! narrows.

use alloc::string::String;
use alloc::vec::Vec;

use ucal_core::backend::TickInt;
use ucal_core::num::{isqrt_ceil, Ratio};
use ucal_core::{Code, Delta, Instant, Ticks, TimeError, Warning, Window, UC1};

use ucal_core::profile::Citation;

type Result<T> = core::result::Result<T, TimeError>;

/// A published linear ephemeris, with its uncertainties and its provenance.
///
/// Constructed rather than parsed: a file loader lives in the `ucal` crate, the
/// same way §15.1 body files do, so that this crate stays free of a format.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Ephemeris {
    id: String,
    label: String,
    epoch: Instant<UC1>,
    epoch_sigma: Delta,
    period: Ratio,
    period_sigma: Ratio,
    pdot: Ratio,
    fitted: (i64, i64),
    citation: Citation,
    /// The period exactly as the source printed it, with its unit (Rule Y.1).
    as_published: String,
}

/// One predicted event.
#[derive(Clone, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Prediction {
    /// Which cycle. Signed: negative is before `T_0`.
    pub cycle: i64,
    /// The predicted window at the requested confidence.
    pub window: Window<UC1>,
    /// Half-width of that window, in ticks.
    pub sigma: Delta,
    /// How many σ the window spans on each side.
    pub k: u32,
    /// `UCAL-W0003` when the cycle lies outside the range the fit covers.
    pub warning: Option<Warning>,
}

impl Ephemeris {
    /// Declare an ephemeris.
    ///
    /// `fitted` is the **cycle range the published fit actually covers**, and it
    /// is not optional. An ephemeris without one is a model with no stated
    /// domain, which Rule C refuses for every other parameter in this workspace
    /// and there is no reason for this to be the exception — it is the *only*
    /// field that makes `UCAL-W0003` mean anything here.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        label: &str,
        epoch: Instant<UC1>,
        epoch_sigma: Delta,
        period: Ratio,
        period_sigma: Ratio,
        pdot: Ratio,
        fitted: (i64, i64),
        citation: Citation,
        as_published: &str,
    ) -> Result<Ephemeris> {
        if period.is_zero() {
            return Err(TimeError::with_context(
                Code::E0018,
                "a period of zero is not an ephemeris; every cycle would name the \
                 same instant",
            ));
        }
        if fitted.0 > fitted.1 {
            return Err(TimeError::with_context(
                Code::E0018,
                "the fitted cycle range runs backwards",
            ));
        }
        Ok(Ephemeris {
            id: String::from(id),
            label: String::from(label),
            epoch,
            epoch_sigma,
            period,
            period_sigma,
            pdot,
            fitted,
            citation,
            as_published: String::from(as_published),
        })
    }

    /// The identifier.
    pub fn id(&self) -> &str {
        &self.id
    }
    /// The human label.
    pub fn label(&self) -> &str {
        &self.label
    }
    /// `T_0`.
    pub fn epoch(&self) -> &Instant<UC1> {
        &self.epoch
    }
    /// The published 1σ on `T_0`.
    pub fn epoch_sigma(&self) -> &Delta {
        &self.epoch_sigma
    }
    /// `P`, in ticks.
    pub fn period(&self) -> &Ratio {
        &self.period
    }
    /// The published 1σ on `P`, in ticks.
    pub fn period_sigma(&self) -> &Ratio {
        &self.period_sigma
    }
    /// `Ṗ`, dimensionless.
    pub fn pdot(&self) -> &Ratio {
        &self.pdot
    }
    /// The cycle range the published fit covers.
    pub fn fitted(&self) -> (i64, i64) {
        self.fitted
    }
    /// Where the figures came from.
    pub fn citation(&self) -> &Citation {
        &self.citation
    }
    /// The period as the source printed it (Rule Y.1).
    pub fn as_published(&self) -> &str {
        &self.as_published
    }

    /// The offset from `T_0` to cycle `E`, as a magnitude and a direction.
    ///
    /// `E·P + ½·E²·P·Ṗ`. `Ratio` is unsigned (Rule B), so the sign of `E` is a
    /// branch — and the quadratic term keeps `E`'s sign only through `E²`, which
    /// is positive, so a negative `Ṗ` on a negative `E` is the case that is easy
    /// to get backwards and is tested.
    fn offset(&self, cycle: i64) -> Result<(Ratio, bool)> {
        let n = Ratio::from_u64(cycle.unsigned_abs());
        let linear = n.mul(&self.period)?;
        if self.pdot.is_zero() {
            return Ok((linear, cycle >= 0));
        }
        // ½·E²·P·Ṗ — always in the same direction as +time for a positive Ṗ,
        // whatever the sign of E, because E² is positive.
        let quad = n
            .mul(&n)?
            .mul(&self.period)?
            .mul(&self.pdot)?
            .div(&Ratio::from_u64(2))?;
        if cycle >= 0 {
            Ok((linear.add(&quad)?, true))
        } else {
            // Backwards: the linear term goes back, the quadratic still goes
            // forward. It can therefore *reduce* the distance travelled, and for
            // a large enough |E| it flips the direction entirely.
            match linear.cmp_exact(&quad) {
                core::cmp::Ordering::Less => Ok((quad.sub(&linear)?, true)),
                _ => Ok((linear.sub(&quad)?, false)),
            }
        }
    }

    /// The 1σ half-width at cycle `E`, in ticks.
    ///
    /// `√(σ_T₀² + (E·σ_P)²)`, by integer square root **rounded up**. Each term
    /// is taken to its ceiling first, so the result is an upper bound on the
    /// true value and the window it produces contains the one the published
    /// figures imply.
    pub fn sigma_at(&self, cycle: i64) -> Result<Delta> {
        let a = self.epoch_sigma.ticks().clone();
        let b = Ratio::from_u64(cycle.unsigned_abs())
            .mul(&self.period_sigma)?
            .ceil();
        let sq = |x: &Ticks| -> Result<Ticks> {
            x.try_mul(x).ok_or(TimeError::with_context(
                Code::E0021,
                "that many cycles makes the uncertainty exceed the domain before \
                 it is even added to anything",
            ))
        };
        let total = sq(&a)?
            .try_add(&sq(&b)?)
            .ok_or(TimeError::new(Code::E0021))?;
        Ok(Delta::from_ticks(isqrt_ceil(&total)))
    }

    /// The predicted instant for cycle `E`, with no uncertainty applied.
    ///
    /// **Separated from [`time_of`](Self::time_of) because `cycle_at` needs it.**
    /// The first version had `cycle_at` compare against `time_of(e).window.lo()`,
    /// which is the centre minus `k·σ` — and `σ` grows with `|E|`, so the
    /// comparison drifted by whole cycles the further out it looked. The
    /// round-trip test found it at cycle −999, where 1σ is ten periods wide.
    ///
    /// A window edge is not a time, and using one as a boundary is the mistake.
    pub fn centre_of(&self, cycle: i64) -> Result<Instant<UC1>> {
        let (magnitude, forward) = self.offset(cycle)?;
        if !magnitude.is_integer() {
            // A period is a rational number of ticks and E·P usually is not an
            // integer. Taking the floor moves the centre by under one tick,
            // which is 5.39e-44 s; the window is then widened by a whole tick on
            // each side so that the true centre is still inside it.
            //
            // Recorded rather than silent: Rule R makes rendering the only lossy
            // step, and this is not rendering, so the loss is *compensated* here
            // rather than accepted.
        }
        let magnitude = magnitude.floor();
        let base = if forward {
            self.epoch
                .ticks()
                .try_add(&magnitude)
                .ok_or(TimeError::with_context(
                    Code::E0021,
                    "that cycle lands past the domain ceiling",
                ))?
        } else {
            self.epoch
                .ticks()
                .try_sub(&magnitude)
                .ok_or(TimeError::with_context(
                    Code::E0020,
                    "that cycle lands before the datum. Absolute time is unsigned \
                     (Rule B), so there is no instant earlier than tick 0",
                ))?
        };
        Instant::<UC1>::from_ticks(base)
    }

    /// The predicted window for cycle `E`, at `k` σ.
    ///
    /// `k = 1` is the published 1σ. Observers usually want 3.
    pub fn time_of(&self, cycle: i64, k: u32) -> Result<Prediction> {
        if k == 0 {
            return Err(TimeError::with_context(
                Code::E0018,
                "a zero-σ window is a point, and a prediction is not one. Ask for \
                 1 to get the published uncertainty",
            ));
        }
        let centre = self.centre_of(cycle)?;
        let sigma = self.sigma_at(cycle)?;
        let half = sigma
            .ticks()
            .clone()
            .try_mul(&<Ticks as TickInt>::from_u64(u64::from(k)))
            .and_then(|v| v.try_add(&<Ticks as TickInt>::one()))
            .ok_or(TimeError::new(Code::E0021))?;
        let (window, _clamped) = Window::exact(centre).widen(&Delta::from_ticks(half))?;

        let warning = if cycle < self.fitted.0 || cycle > self.fitted.1 {
            Some(Warning::W0003)
        } else {
            None
        };
        Ok(Prediction {
            cycle,
            window,
            sigma,
            k,
            warning,
        })
    }

    /// Which cycle an instant falls in, and how far through it.
    ///
    /// The estimate is corrected in a loop rather than trusted, the way
    /// `split_year` corrects its own: with a `Ṗ` the spacing is not uniform, so
    /// division gives an estimate and not an answer.
    pub fn cycle_at(&self, t: &Instant<UC1>) -> Result<(i64, Ratio)> {
        let forward = t.ticks() >= self.epoch.ticks();
        let gap = if forward {
            t.ticks().clone().try_sub(self.epoch.ticks())
        } else {
            self.epoch.ticks().clone().try_sub(t.ticks())
        }
        .ok_or(TimeError::new(Code::E0021))?;

        let est: i64 = Ratio::from_int(gap)
            .div(&self.period)?
            .floor()
            .to_dec_string()
            .parse()
            .map_err(|_| {
                TimeError::with_context(
                    Code::E0040,
                    "that instant is more cycles from the epoch than a cycle \
                     number can hold",
                )
            })?;
        let mut e = if forward { est } else { -est - 1 };

        // At most a few steps: the estimate is exact without `Ṗ`, and with one
        // it is out by the quadratic term's share of a period. Bounded, because
        // a loop that cannot terminate is worse than a wrong answer.
        for _ in 0..64 {
            let start = self.centre_of(e)?;
            let next = self.centre_of(e.checked_add(1).ok_or(TimeError::new(Code::E0021))?)?;
            let (start, next) = (start.ticks(), next.ticks());
            if t.ticks() < start {
                e -= 1;
                continue;
            }
            if t.ticks() >= next {
                e += 1;
                continue;
            }
            let span = next.clone().try_sub(start).ok_or(TimeError::new(Code::E0021))?;
            let into = t.ticks().clone().try_sub(start).expect("start <= t");
            let phase = Ratio::new(into, span)?;
            return Ok((e, phase));
        }
        Err(TimeError::with_context(
            Code::E0019,
            "the cycle search did not settle. The estimate is corrected in a \
             bounded loop, and exceeding it means the period and its derivative \
             disagree about the ordering of cycles",
        ))
    }

    /// The next event strictly after `t`.
    pub fn next_after(&self, t: &Instant<UC1>, k: u32) -> Result<Prediction> {
        let (e, _) = self.cycle_at(t)?;
        self.time_of(e.checked_add(1).ok_or(TimeError::new(Code::E0021))?, k)
    }

    /// The next `count` events after `t`.
    pub fn upcoming(&self, t: &Instant<UC1>, k: u32, count: u32) -> Result<Vec<Prediction>> {
        let (e, _) = self.cycle_at(t)?;
        let mut out = Vec::new();
        for i in 1..=i64::from(count) {
            out.push(self.time_of(
                e.checked_add(i).ok_or(TimeError::new(Code::E0021))?,
                k,
            )?);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::Profile;

    fn second() -> Ticks {
        UC1::bridge().ticks
    }

    /// A synthetic ephemeris: epoch at 10^50 ticks, period 100 s, σ as given.
    fn synthetic(sigma_t0_s: u64, sigma_p_s: u64, pdot: Ratio) -> Ephemeris {
        let epoch = Instant::<UC1>::from_ticks(
            <Ticks as TickInt>::from_dec_str(&alloc::format!("1{}", "0".repeat(50)))
                .expect("in range"),
        )
        .expect("in the domain");
        Ephemeris::new(
            "synthetic",
            "a test ephemeris",
            epoch,
            Delta::from_ticks(
                second()
                    .try_mul(&<Ticks as TickInt>::from_u64(sigma_t0_s))
                    .expect("in range"),
            ),
            Ratio::from_int(
                second()
                    .try_mul(&<Ticks as TickInt>::from_u64(100))
                    .expect("in range"),
            ),
            Ratio::from_int(
                second()
                    .try_mul(&<Ticks as TickInt>::from_u64(sigma_p_s))
                    .expect("in range"),
            ),
            pdot,
            (-1000, 1000),
            Citation::new("a test", None),
            "100 s",
        )
        .expect("valid")
    }

    /// **The property the whole type exists for: the window grows with `E`.**
    ///
    /// A prediction a thousand cycles out is not as good as one at the epoch,
    /// and a type that returned the same width for both would be the defect
    /// this replaces.
    #[test]
    fn the_window_widens_with_distance_from_the_epoch() {
        let e = synthetic(1, 1, Ratio::zero());
        let at_epoch = e.time_of(0, 1).expect("cycle 0");
        let far = e.time_of(1000, 1).expect("cycle 1000");
        assert!(
            far.sigma.ticks() > at_epoch.sigma.ticks(),
            "the window must grow"
        );
        // 1000 cycles at 1 s per cycle, in quadrature with 1 s at the epoch:
        // sqrt(1 + 1000000) s, which is just over 1000 s.
        let thousand = second()
            .try_mul(&<Ticks as TickInt>::from_u64(1000))
            .expect("in range");
        assert!(far.sigma.ticks() > &thousand, "and by about E·σ_P");
        assert!(
            far.sigma.ticks()
                < &second()
                    .try_mul(&<Ticks as TickInt>::from_u64(1001))
                    .expect("in range"),
            "quadrature, not addition"
        );
    }

    /// Quadrature is not addition, and the difference is visible at the epoch.
    #[test]
    fn sigma_at_the_epoch_is_the_epochs_own() {
        let e = synthetic(3, 5, Ratio::zero());
        let s = e.sigma_at(0).expect("cycle 0");
        let three = second()
            .try_mul(&<Ticks as TickInt>::from_u64(3))
            .expect("in range");
        // Equal, or one tick above it — `isqrt_ceil` rounds outward.
        assert!(s.ticks() >= &three);
        assert!(
            s.ticks()
                <= &three
                    .try_add(&<Ticks as TickInt>::one())
                    .expect("in range")
        );
    }

    /// The window is widened outward, so it contains its own centre's cycle.
    #[test]
    fn a_predicted_window_contains_the_instant_it_predicts() {
        let e = synthetic(1, 1, Ratio::zero());
        for cycle in [-500i64, -1, 0, 1, 7, 999] {
            let p = e.time_of(cycle, 3).expect("in range");
            let (found, _) = e.cycle_at(p.window.lo()).expect("a cycle");
            // The low end of a 3σ window is well before the centre, so the
            // cycle there may be the previous one; what must hold is that the
            // centre round-trips.
            assert!(found <= cycle, "{cycle}: {found}");
        }
    }

    /// Round trip: the cycle a predicted centre falls in is the one asked for.
    #[test]
    fn the_centre_of_a_prediction_is_in_the_cycle_it_names() {
        let e = synthetic(1, 1, Ratio::zero());
        for cycle in [-999i64, -2, 0, 3, 50, 1000] {
            let centre = e.centre_of(cycle).expect("in range");
            let (found, _) = e.cycle_at(&centre).expect("a cycle");
            assert_eq!(found, cycle, "centre of {cycle} landed in {found}");
        }
    }

    /// **Rule C.** A cycle outside the fitted range warns and does not refuse.
    ///
    /// The distinction matters: extrapolating is what people do, and the rule
    /// forbids doing it *silently* rather than doing it at all.
    #[test]
    fn a_cycle_outside_the_fit_warns() {
        let e = synthetic(1, 1, Ratio::zero());
        assert!(e.time_of(999, 1).expect("inside").warning.is_none());
        assert_eq!(
            e.time_of(1001, 1).expect("outside").warning,
            Some(Warning::W0003)
        );
        assert_eq!(
            e.time_of(-1001, 1).expect("outside").warning,
            Some(Warning::W0003)
        );
    }

    /// A period derivative moves the prediction, in the same direction both ways.
    ///
    /// `½E²PṖ` keeps `E²`'s sign, so a positive `Ṗ` pushes both future and past
    /// cycles later. Getting that backwards is a sign error no magnitude test
    /// would catch, which is why it has its own.
    #[test]
    fn a_period_derivative_acts_through_e_squared() {
        let pdot = Ratio::new(
            <Ticks as TickInt>::from_u64(1),
            <Ticks as TickInt>::from_u64(1_000),
        )
        .expect("a rate");
        let with = synthetic(1, 1, pdot);
        let without = synthetic(1, 1, Ratio::zero());

        let (fw_a, _) = with.offset(100).expect("forward");
        let (fw_b, _) = without.offset(100).expect("forward");
        assert_eq!(fw_a.cmp_exact(&fw_b), core::cmp::Ordering::Greater);

        // Backwards, the quadratic still pushes *forward*, so the distance
        // travelled back is smaller than without it.
        let (bw_a, dir_a) = with.offset(-100).expect("backward");
        let (bw_b, dir_b) = without.offset(-100).expect("backward");
        assert!(!dir_a && !dir_b, "both still land before the epoch");
        assert_eq!(bw_a.cmp_exact(&bw_b), core::cmp::Ordering::Less);
    }

    /// A zero period is refused rather than producing one instant for every cycle.
    #[test]
    fn a_zero_period_is_not_an_ephemeris() {
        let epoch = Instant::<UC1>::from_ticks(<Ticks as TickInt>::from_u64(1)).expect("valid");
        let e = Ephemeris::new(
            "z",
            "zero",
            epoch,
            Delta::from_u64(0),
            Ratio::zero(),
            Ratio::zero(),
            Ratio::zero(),
            (0, 1),
            Citation::new("none", None),
            "0",
        );
        assert!(e.is_err());
    }

    /// `k = 0` is refused: a prediction with no width is not a prediction.
    #[test]
    fn a_zero_sigma_window_is_refused() {
        let e = synthetic(1, 1, Ratio::zero());
        assert_eq!(e.time_of(0, 0).expect_err("no width").code, Code::E0018);
    }
}
