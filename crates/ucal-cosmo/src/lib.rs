//! # ucal-cosmo — flat ΛCDM by certified integer quadrature
//!
//! §10.1: maps absolute time to redshift and back "under a declared flat ΛCDM
//! model, and back, with integer arithmetic only (Rule E)". No float appears in
//! any signature, field, constant or intermediate, and no transcendental function
//! is evaluated anywhere (N13).
//!
//! # The substitution that makes this possible
//!
//! §10.3 states the integral as
//!
//! ```text
//! t(z) = (1/H0) ∫_z^∞ dz' / [ (1+z') E(z') ],   E(z) = √(Ω_r(1+z)⁴ + Ω_m(1+z)³ + Ω_Λ)
//! ```
//!
//! which is improper: the upper limit is infinite. Certified quadrature needs a
//! bounded interval, so substitute `u = 1/(1+z)`:
//!
//! ```text
//! t(z) = (1/H0) ∫_0^{u₀} u du / √(Ω_r + Ω_m u + Ω_Λ u⁴),   u₀ = 1/(1+z)
//! ```
//!
//! The improper limit becomes `u = 0`, the integrand is bounded and smooth on the
//! whole of `[0, u₀]` (because `Ω_r > 0` keeps the root away from zero), and the
//! panel bounds below are valid without any limiting argument.
//!
//! # Why the panels use an interval extension rather than endpoints
//!
//! Appendix H.4 permits endpoint bounds for a **monotone** integrand, and requires
//! that monotonicity be "asserted, not assumed". It is not monotone here. With
//! `f(u)² = u²/g(u)`, the derivative's numerator is `u(2Ω_r + Ω_m u − 2Ω_Λ u⁴)`,
//! which changes sign near `u ≈ 0.604` — so `f` rises and then falls, and for
//! `z ≲ 0.66` the integration range straddles the turn.
//!
//! H.4 anticipates exactly this: "where it fails, the panel is bounded by the
//! interval extension of the integrand". Since `g` **is** increasing on `[0, 1]`
//! for non-negative densities, the extension on a panel `[a, b]` is
//!
//! ```text
//! f([a,b]) ⊆ [ a / √g(b) , b / √g(a) ]
//! ```
//!
//! which is valid everywhere, needs no case analysis, and tightens quadratically
//! as panels shrink. [`monotonicity_turns_at`] exposes the turning point so the
//! claim above is checkable rather than assertable.
//!
//! # Rule X, and the two widths
//!
//! Every result is a [`CosmoResult`] carrying **two widths, never merged**:
//! `arithmetic_width` from quadrature and subdivision depth, and
//! `parameter_width` from the model's own uncertainty. Failure mode F8 is
//! precisely their conflation — "float error and parameter uncertainty conflated
//! into one tolerance" — and on these numbers they differ by seven orders of
//! magnitude, so merging them would hide the one that matters behind the one that
//! does not.

#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

#[cfg(feature = "alloc")]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;

use ucal_core::backend::TickInt;
use ucal_core::num::{RatInterval, Ratio};
use ucal_core::{
    Citation, Code, Delta, Instant, Profile, Rounding, Ticks, TimeError, Warning, Window, UC1,
};

/// The default subdivision depth: `2^12 = 4096` panels.
///
/// # GE-1, measured: the kill criterion fires, and it does not matter
///
/// §21 asked whether "certified interval quadrature at the depth needed for a
/// useful enclosure may be too slow", with the kill criterion "if depth-24
/// quadrature exceeds ~2 s, reduce the default depth and expose a high-precision
/// mode". Measured on this implementation, release build, `z = 1100`:
///
/// ```text
/// depth   panels     wall      arithmetic width
///     4       16    1.3 ms       64 251 yr
///     8      256     37 ms        4 011 yr
///    12    4 096    476 ms          251 yr
///    14   16 384    2.01 s           63 yr
///    16   65 536    8.66 s           16 yr
/// ```
///
/// Cost grows about 4× per depth step — 2× the panels, and larger rationals in
/// each — so depth 24 is roughly six hours, not two seconds. **The kill
/// criterion fires.** The default is 12 and the depth is an explicit argument on
/// every entry point, which is the "high-precision mode" GE-1 asked for.
///
/// The reason this costs nothing scientifically is the second column. At depth
/// 12 the arithmetic width at `z = 1100` is 251 years while the **parameter**
/// width — the propagated consequence of Planck's own error bars — is 10 917
/// years. The quadrature is already forty times sharper than the measurement it
/// is integrating. Depth 16 buys a factor of sixteen on a term that contributes
/// two per cent of the total, for eighteen times the wall clock. Rule X's
/// insistence on reporting the two widths separately is what makes that visible;
/// a single merged tolerance would have hidden it.
pub const DEFAULT_DEPTH: u32 = 12;

/// The default fixed-point scale for the directed square roots (D-6): twelve
/// decimal digits.
///
/// The roots run from `√Ω_r ≈ 0.0096` upward, so twelve digits is a relative
/// precision near `10^-10` — four orders below the panel error at any depth this
/// crate will attempt, and therefore never the binding constraint.
pub const DEFAULT_SCALE: u32 = 12;

/// GE-2, measured: the achievable enclosure width, published as the experiment
/// required.
///
/// §21 asked "what fixed-point scale is needed for a ≤ 1-tick enclosure at
/// `z = 1100`", with the kill criterion "publish the achievable width and set
/// `UCAL-W0004`". The answer is that **no scale reaches one tick**, and the
/// obstacle is not the scale.
///
/// One tick is `5^-220 · 5^220`; a year is about `1.4 × 10^51` ticks. At the
/// default depth the enclosure at `z = 1100` is about 11 000 years wide, or
/// `1.6 × 10^55` ticks — fifty-five orders of magnitude above the target. Even
/// with exact parameters, the panel error would need depth near 180 to close,
/// which is `2^180` panels.
///
/// This is not a defect of the arithmetic. A cosmological age is a *derived*
/// quantity whose inputs are measured to four significant figures; asking it to
/// land on a Planck tick is asking for fifty more digits than were measured.
/// Every result therefore carries [`Warning::W0004`], and the width is reported
/// rather than rounded away — Rule T, applied to a quantity the RFC hoped might
/// escape it.
pub const GE2_ACHIEVABLE_WIDTH: &str =
    "z=1100 at depth 12, scale 12: 360 432..371 600 yr; arithmetic width 251 yr, \
     parameter width 10 917 yr. One tick is unreachable by ~55 orders of magnitude; \
     UCAL-W0004 is set on every result.";

/// §16 names a distinct error type; it is the crate's, because the Appendix E
/// codes are the contract.
pub type CosmoError = TimeError;

type Result<T> = core::result::Result<T, CosmoError>;

/// A model identifier, carried on every result (Rule X).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ModelId(pub &'static str);

/// A measured cosmological parameter, recorded as published (Rule Y.1).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MeasuredParam {
    /// The parameter's name.
    pub name: &'static str,
    /// The published value and uncertainty, verbatim.
    pub verbatim: &'static str,
}

/// A flat ΛCDM model with interval-valued parameters (§10.2).
///
/// Every density is an interval, not a point. §10.5 takes Planck 2018's published
/// uncertainties as the bounds, so a result's `parameter_width` is the propagated
/// consequence of what was actually measured rather than a guess at it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LambdaCdm {
    /// Matter density.
    pub omega_m: RatInterval,
    /// Dark energy density.
    pub omega_l: RatInterval,
    /// Radiation density, photons plus relativistic neutrinos.
    pub omega_r: RatInterval,
    /// The Hubble time `1/H0`, in **ticks**. Converting from km/s/Mpc is done
    /// once, here, and recorded — see [`LambdaCdm::planck2018`].
    pub hubble_time: RatInterval,
    /// The published inputs, verbatim.
    pub as_measured: &'static [MeasuredParam],
    /// The source.
    pub citation: Citation,
    /// Which model this is.
    pub model: ModelId,
}

const PLANCK_2018: Citation = Citation {
    source: "Planck 2018 results VI: Cosmological parameters, A&A 641, A6 (2020), \
             TT,TE,EE+lowE+lensing+BAO",
    locator: Some("doi:10.1051/0004-6361/201833910"),
};

fn r(decimal: &str) -> Ratio {
    Ratio::from_decimal_str(decimal).expect("model constant is an exact decimal")
}

fn iv(lo: &str, hi: &str) -> RatInterval {
    RatInterval::new(r(lo), r(hi)).expect("model interval is ordered")
}

impl LambdaCdm {
    /// The default parameter set: Planck 2018, cited, with published
    /// uncertainties as interval bounds (§10.5).
    ///
    /// # The Hubble time, and π
    ///
    /// `H0` is published in km/s/Mpc, and a megaparsec is defined through π:
    /// `1 pc = (648000/π) AU` with `1 AU = 149 597 870 700 m` exactly. π is not
    /// rational, so `1/H0` cannot be either.
    ///
    /// It is therefore carried as an **interval**, bounded by a rational
    /// enclosure of π. That is rigorous rather than approximate, and the width it
    /// contributes is negligible beside `H0`'s own ±0.42 km/s/Mpc — but it is
    /// propagated rather than assumed away, because the alternative would be a
    /// silent irrational in an exact system.
    ///
    /// `1/H0 = 10⁶ × 648000 × 149597870700 / (H0_in_m_per_s × π)` seconds,
    /// converted to ticks through the bridge constant.
    pub fn planck2018() -> LambdaCdm {
        // H0 = 67.66 ± 0.42 km/s/Mpc, so 1/H0 is largest at the smallest H0.
        let hubble_time = hubble_time_ticks("67.24", "68.08").expect("Planck 2018 H0");
        LambdaCdm {
            // Omega_m = 0.3111 +/- 0.0056
            omega_m: iv("0.3055", "0.3167"),
            // Omega_Lambda = 0.6889 +/- 0.0056
            omega_l: iv("0.6833", "0.6945"),
            // Omega_r h^2 = 4.18e-5 (photons + 3 relativistic neutrino species);
            // with h = 0.6766 this is 9.14e-5, and the bounds follow h's own.
            omega_r: iv("0.00009000", "0.00009300"),
            hubble_time,
            as_measured: &[
                MeasuredParam {
                    name: "H0",
                    verbatim: "67.66 +/- 0.42 km/s/Mpc",
                },
                MeasuredParam {
                    name: "Omega_m",
                    verbatim: "0.3111 +/- 0.0056",
                },
                MeasuredParam {
                    name: "Omega_Lambda",
                    verbatim: "0.6889 +/- 0.0056",
                },
                MeasuredParam {
                    name: "Omega_r",
                    verbatim: "9.14e-5 (derived from Omega_r h^2 = 4.18e-5)",
                },
            ],
            citation: PLANCK_2018,
            model: ModelId("flat-LambdaCDM/planck2018"),
        }
    }

    /// The turning point of the integrand's monotonicity, as an exact rational.
    ///
    /// `f(u) = u/√g(u)` rises then falls; the turn is where
    /// `2Ω_r + Ω_m u − 2Ω_Λ u⁴ = 0`. Exposed so that Appendix H.4's requirement
    /// to *assert* monotonicity rather than assume it can be discharged by a
    /// test rather than by a comment.
    pub fn monotonicity_turns_at(&self) -> Result<RatInterval> {
        // Bisect on the sign of the derivative's numerator, which is monotone
        // decreasing in u over [0,1] for non-negative densities.
        let mut lo = Ratio::zero();
        let mut hi = Ratio::one();
        let two = Ratio::from_u64(2);
        let om = self.omega_m.lo().clone();
        let ol = self.omega_l.hi().clone();
        let orr = self.omega_r.lo().clone();
        for _ in 0..40 {
            let mid = lo.add(&hi)?.div(&two)?;
            let u4 = mid.mul(&mid)?.mul(&mid.mul(&mid)?)?;
            let positive = two
                .mul(&orr)?
                .add(&om.mul(&mid)?)?
                .cmp_exact(&two.mul(&ol)?.mul(&u4)?)
                == core::cmp::Ordering::Greater;
            if positive {
                lo = mid;
            } else {
                hi = mid;
            }
        }
        RatInterval::new(lo, hi)
    }
}

/// `1/H0` in ticks, as an interval, from a published range in km/s/Mpc.
fn hubble_time_ticks(h0_lo: &str, h0_hi: &str) -> Result<RatInterval> {
    // 1 AU = 149 597 870 700 m exactly (IAU 2012); 1 pc = (648000/pi) AU.
    // 1/H0 = 1e6 * 648000 * AU / (H0_m_per_s * pi) seconds.
    let au = Ratio::from_u64(149_597_870_700);
    let mpc_numer = Ratio::from_u64(1_000_000)
        .mul(&Ratio::from_u64(648_000))?
        .mul(&au)?;
    // A rational enclosure of pi. Twenty digits is far tighter than any
    // measurement here, but it is an enclosure rather than a value.
    let pi_lo = r("3.14159265358979323846");
    let pi_hi = r("3.14159265358979323847");

    let second = Ratio::from_int(UC1::bridge().ticks);
    // Largest 1/H0 uses the smallest H0 and the smallest pi.
    let hi = mpc_numer
        .div(&r(h0_lo).mul(&Ratio::from_u64(1000))?.mul(&pi_lo)?)?
        .mul(&second)?;
    let lo = mpc_numer
        .div(&r(h0_hi).mul(&Ratio::from_u64(1000))?.mul(&pi_hi)?)?
        .mul(&second)?;
    RatInterval::new(lo, hi)
}

/// A cosmological result, with its two widths kept apart (Rule X).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CosmoResult<T> {
    /// The certified enclosure.
    pub value: T,
    /// Width contributed by quadrature and subdivision depth.
    pub arithmetic_width: Delta,
    /// Width contributed by the model's parameter uncertainty.
    pub parameter_width: Delta,
    /// The subdivision depth used.
    pub depth: u32,
    /// The fixed-point scale used for the directed square roots (D-6).
    pub scale: u32,
    /// Which model produced it.
    pub model: ModelId,
    /// The model's citation.
    pub citation: Citation,
    /// Any warning; `UCAL-W0006` inside the claim half-width (§10.6),
    /// `UCAL-W0004` when the enclosure is wider than one tick.
    pub warnings: Vec<Warning>,
}

impl<T> CosmoResult<T> {
    /// The two widths, summed — offered explicitly so that a caller who wants a
    /// single number has to ask for it by name.
    ///
    /// Rule X forbids *merging* them in the result; it does not forbid a caller
    /// adding them up deliberately. The distinction is that this is a method with
    /// a name, not a field that quietly lost the split.
    pub fn total_width(&self) -> Result<Delta> {
        self.arithmetic_width.checked_add(&self.parameter_width)
    }
}

/// The dimensionless integral `∫_0^{u₀} u du / √(Ω_r + Ω_m u + Ω_Λ u⁴)`,
/// as a certified enclosure.
///
/// Panels are bounded by the interval extension described in the module
/// documentation: on `[a, b]`, `f ∈ [a/√g(b), b/√g(a)]`. The lower sum uses an
/// **upper** bound on each root and the upper sum a **lower** one, so both
/// directions round outward and the enclosure is rigorous.
fn integral_enclosure(model: &LambdaCdm, u0: &Ratio, depth: u32, scale: u32) -> Result<RatInterval> {
    if depth > 30 {
        return Err(TimeError::with_context(
            Code::E0071,
            "subdivision depth beyond 2^30 panels is not attempted; the enclosure \
             would take longer than it is worth (GE-1)",
        ));
    }
    let panels: u64 = 1u64 << depth;
    let n = Ratio::from_u64(panels);
    let h = u0.div(&n)?;

    // Widest densities: the lower sum wants the largest g, the upper the smallest.
    let om_hi = model.omega_m.hi();
    let ol_hi = model.omega_l.hi();
    let or_hi = model.omega_r.hi();
    let om_lo = model.omega_m.lo();
    let ol_lo = model.omega_l.lo();
    let or_lo = model.omega_r.lo();

    let g = |u: &Ratio, orr: &Ratio, om: &Ratio, ol: &Ratio| -> Result<Ratio> {
        let u2 = u.mul(u)?;
        let u4 = u2.mul(&u2)?;
        orr.add(&om.mul(u)?)?.add(&ol.mul(&u4)?)
    };

    // The accumulator's grid. Exact accumulation is impossible — see
    // [`Ratio::snap`] — so each partial sum is snapped outward onto a fixed
    // decimal grid. The grid is chosen finer than the quadrature error it has to
    // stay under: `depth` extra digits cover the `2^depth` snaps, `scale` matches
    // the roots, and the constant is headroom. Ten thousand panels on a grid of
    // `10^-38` contribute `10^-34` of slop to a sum near `10^-5`; the panels
    // themselves contribute `2^-depth` of it.
    let grid = scale + depth + 16;

    let mut lo_sum = Ratio::zero();
    let mut hi_sum = Ratio::zero();

    for i in 0..panels {
        let a = h.mul(&Ratio::from_u64(i))?;
        let b = h.mul(&Ratio::from_u64(i + 1))?;

        // Lower bound on the panel: smallest numerator over largest root.
        let g_b = g(&b, or_hi, om_hi, ol_hi)?;
        let (root_b, _) = RatInterval::exact(g_b).sqrt_enclosure(scale)?;
        if !root_b.hi().is_zero() {
            let term = h.mul(&a)?.div(root_b.hi())?.snap(grid, Rounding::Trunc)?;
            lo_sum = lo_sum.add(&term)?;
        }

        // Upper bound: largest numerator over smallest root.
        let g_a = g(&a, or_lo, om_lo, ol_lo)?;
        let (root_a, _) = RatInterval::exact(g_a).sqrt_enclosure(scale)?;
        if root_a.lo().is_zero() {
            return Err(TimeError::with_context(
                Code::E0070,
                "the integrand's denominator reached zero; Omega_r must be positive",
            ));
        }
        let term = h.mul(&b)?.div(root_a.lo())?.snap(grid, Rounding::Ceil)?;
        hi_sum = hi_sum.add(&term)?;
    }

    RatInterval::new(lo_sum, hi_sum)
}

/// The same integral with the parameters pinned to their midpoints, so that the
/// arithmetic width can be separated from the parameter width (Rule X).
fn integral_at_central_parameters(
    model: &LambdaCdm,
    u0: &Ratio,
    depth: u32,
    scale: u32,
) -> Result<RatInterval> {
    let mid = |iv: &RatInterval| -> Result<Ratio> {
        iv.lo().add(iv.hi())?.div(&Ratio::from_u64(2))
    };
    let central = LambdaCdm {
        omega_m: RatInterval::exact(mid(&model.omega_m)?),
        omega_l: RatInterval::exact(mid(&model.omega_l)?),
        omega_r: RatInterval::exact(mid(&model.omega_r)?),
        hubble_time: RatInterval::exact(mid(&model.hubble_time)?),
        ..model.clone()
    };
    integral_enclosure(&central, u0, depth, scale)
}

impl LambdaCdm {
    /// The age of the universe at redshift `z`, as a certified enclosure (§16).
    ///
    /// Returns a [`Window`] provably containing the true value under this model,
    /// with the arithmetic and parameter widths reported separately (Rule X).
    pub fn t_of_z(&self, z: &Ratio, depth: u32, scale: u32) -> Result<CosmoResult<Window<UC1>>> {
        // u0 = 1/(1+z)
        let u0 = Ratio::one().add(z)?.recip()?;

        let full = integral_enclosure(self, &u0, depth, scale)?;
        let central = integral_at_central_parameters(self, &u0, depth, scale)?;

        // t = (1/H0) x integral, with both intervals multiplied outward.
        let t_lo = full.lo().mul(self.hubble_time.lo())?;
        let t_hi = full.hi().mul(self.hubble_time.hi())?;

        let to_instant = |v: &Ratio| -> Result<Instant<UC1>> {
            Instant::from_ticks(v.floor())
        };
        let value = Window::new(to_instant(&t_lo)?, to_instant(&t_hi)?)?;

        // The arithmetic width is what remains with the parameters pinned: it is
        // the quadrature's own contribution and nothing else.
        let arithmetic = central
            .hi()
            .mul(self.hubble_time.lo())?
            .abs_diff(&central.lo().mul(self.hubble_time.lo())?)?;
        let arithmetic_width = Delta::from_ticks(arithmetic.floor());
        // The parameter width is the rest of the total.
        let total = value.width();
        let parameter_width = total
            .checked_sub(&arithmetic_width)
            .unwrap_or_else(|_| Delta::zero());

        let mut warnings = Vec::new();
        if total.ticks() > &<Ticks as TickInt>::one() {
            warnings.push(Warning::W0004);
        }
        // §10.6: inside the claim half-width, the datum's own identification is
        // comparable to the quantity being discussed.
        let half = UC1::big_bang_claim().hi().magnitude().ticks().clone();
        if value.lo().ticks() < &half {
            warnings.push(Warning::W0006);
        }

        Ok(CosmoResult {
            value,
            arithmetic_width,
            parameter_width,
            depth,
            scale,
            model: self.model,
            citation: self.citation,
            warnings,
        })
    }

    /// The largest redshift the model can search: `z = 10 000` is well before
    /// recombination and comfortably outside anything the catalogue records.
    const Z_CEILING: u64 = 10_000;

    /// Redshift from absolute time, by monotone bisection (§10.4).
    ///
    /// # Why this brackets both sides
    ///
    /// `t(z)` is not a function here — it is an *interval-valued* map, because
    /// the parameters are intervals. Inverting it means finding every `z` whose
    /// age-enclosure meets the requested time window:
    ///
    /// ```text
    /// { z : t_enclosure(z) ∩ [T_lo, T_hi] ≠ ∅ }
    /// ```
    ///
    /// Because both `t_lo` and `t_hi` decrease in `z`, that set is itself an
    /// interval `[z_min, z_max]`, and its two ends come from **two different
    /// comparisons**:
    ///
    /// * `z_max` is the largest `z` at which even the *oldest* admissible age is
    ///   still at least `T_lo`;
    /// * `z_min` is the smallest `z` at which even the *youngest* admissible age
    ///   is already at most `T_hi`.
    ///
    /// Bisecting on one endpoint alone — the obvious implementation — returns a
    /// bracket that is narrow, plausible, and not an enclosure of anything: it
    /// locates where one bound of the interval crosses, and says nothing about
    /// the other. Two bisections cost twice as much and are the difference
    /// between a certified answer and a confident one.
    ///
    /// # The tolerance
    ///
    /// `tolerance` is the resolution required *in time*, and §10.4 requires it to
    /// be at least one tick — a finer request is asking for sub-tick resolution,
    /// which N10 forbids. Each bisection stops when the age difference across its
    /// remaining bracket has fallen below it. If the step budget runs out first,
    /// the answer is `UCAL-E0071` rather than a bracket that quietly failed to
    /// converge.
    pub fn z_of_t(
        &self,
        t: &Window<UC1>,
        tolerance: &Delta,
        depth: u32,
        scale: u32,
    ) -> Result<CosmoResult<RatInterval>> {
        if tolerance.ticks() < &<Ticks as TickInt>::one() {
            return Err(TimeError::with_context(
                Code::E0071,
                "the requested inversion tolerance must be at least one tick (§10.4, N10)",
            ));
        }

        // `upper`: bisect on t_hi(z) >= T_lo, keeping the largest such z.
        let z_max = self.bisect(t.lo(), tolerance, depth, scale, true)?;
        // `lower`: bisect on t_lo(z) > T_hi, keeping the smallest z that is not.
        let z_min = self.bisect(t.hi(), tolerance, depth, scale, false)?;

        let value = RatInterval::new(z_min, z_max)?;
        Ok(CosmoResult {
            value,
            // The inversion's arithmetic error is the tolerance it converged to;
            // the parameter uncertainty is already inside the bracket's width,
            // because both ends were found against interval-valued ages.
            arithmetic_width: tolerance.clone(),
            parameter_width: Delta::zero(),
            depth,
            scale,
            model: self.model,
            citation: self.citation,
            warnings: alloc::vec![Warning::W0004],
        })
    }

    /// One side of the inversion. `use_hi` selects which bound of the age
    /// enclosure is compared against `target`, which is what distinguishes the
    /// two ends of the answer.
    fn bisect(
        &self,
        target: &Instant<UC1>,
        tolerance: &Delta,
        depth: u32,
        scale: u32,
        use_hi: bool,
    ) -> Result<Ratio> {
        let two = Ratio::from_u64(2);
        let mut lo = Ratio::zero();
        let mut hi = Ratio::from_u64(Self::Z_CEILING);

        let age = |z: &Ratio| -> Result<Instant<UC1>> {
            let w = self.t_of_z(z, depth, scale)?.value;
            Ok(if use_hi { w.hi().clone() } else { w.lo().clone() })
        };
        let mut t_lo = age(&lo)?;
        let mut t_hi = age(&hi)?;

        for _ in 0..64 {
            // Converged when the bracket no longer resolves anything in time.
            if t_lo.since(&t_hi)? <= *tolerance {
                return Ok(if use_hi { hi } else { lo });
            }
            let mid = lo.add(&hi)?.div(&two)?;
            let t_mid = age(&mid)?;
            // t decreases in z: if the age at `mid` is still at or above the
            // target, the crossing is above `mid`.
            if &t_mid >= target {
                lo = mid;
                t_lo = t_mid;
            } else {
                hi = mid;
                t_hi = t_mid;
            }
        }
        Err(TimeError::with_context(
            Code::E0071,
            "inversion did not reach the requested tolerance within the step budget",
        ))
    }

    /// A human-readable summary of a result's provenance (Rule X).
    pub fn describe(&self) -> String {
        use alloc::format;
        let mut s = String::new();
        for p in self.as_measured {
            s.push_str(&format!("{} = {}; ", p.name, p.verbatim));
        }
        format!("{} [{}] {}", self.model.0, s.trim_end(), self.citation.source)
    }
}

#[cfg(test)]
mod tests;
