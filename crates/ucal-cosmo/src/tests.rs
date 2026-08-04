//! §20 UC-P16: **`z = 1100` enclosure lies inside the catalogue's recombination
//! window; enclosure narrows monotonically with depth; float oracle contained.**

use super::*;

/// A modest depth that keeps the suite quick. GE-1's measurement lives in
/// `benches`-style tests below and reports the cost honestly.
const D: u32 = 10;
const S: u32 = 12;

fn model() -> LambdaCdm {
    LambdaCdm::planck2018()
}

fn years(t: &Ticks) -> String {
    let year = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .unwrap();
    Ratio::new(t.clone(), year)
        .unwrap()
        .to_decimal_string(0, Rounding::HalfEven)
        .unwrap()
}

// ---------------------------------------------------------------------------
// Rule E — no float, no transcendental
// ---------------------------------------------------------------------------

#[test]
fn the_model_carries_its_provenance() {
    let m = model();
    let d = m.describe();
    assert!(d.contains("67.66 +/- 0.42 km/s/Mpc"));
    assert!(d.contains("Planck 2018"));
    assert_eq!(m.as_measured.len(), 4);
    // Every density is an interval, not a point (§10.2).
    for iv in [&m.omega_m, &m.omega_l, &m.omega_r, &m.hubble_time] {
        assert!(!iv.is_exact(), "a measured parameter must carry its uncertainty");
    }
}

#[test]
fn the_hubble_time_is_about_fourteen_and_a_half_gigayears() {
    // 1/H0 for H0 = 67.66 km/s/Mpc is 14.45 Gyr. Computed through a rational
    // enclosure of pi rather than a float, and still an interval.
    let m = model();
    let gyr = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .unwrap()
        .try_mul(&<Ticks as TickInt>::from_u64(1_000_000_000))
        .unwrap();
    let lo = m.hubble_time.lo().div(&Ratio::from_int(gyr.clone())).unwrap();
    let hi = m.hubble_time.hi().div(&Ratio::from_int(gyr)).unwrap();
    assert_eq!(lo.to_decimal_string(2, Rounding::Trunc).unwrap(), "14.36");
    assert_eq!(hi.to_decimal_string(2, Rounding::Trunc).unwrap(), "14.54");
}

// ---------------------------------------------------------------------------
// Appendix H.4 — monotonicity asserted, not assumed
// ---------------------------------------------------------------------------

#[test]
fn the_integrand_is_not_monotone_and_the_turning_point_is_located() {
    // H.4: "Monotonicity of the LambdaCDM integrand over the integration range
    // MUST be asserted, not assumed; where it fails, the panel is bounded by the
    // interval extension." It fails, near u = 0.604 — so the extension is used
    // everywhere rather than a monotonicity case analysis.
    let turn = model().monotonicity_turns_at().unwrap();
    let lo = turn.lo().to_decimal_string(3, Rounding::Trunc).unwrap();
    assert_eq!(lo, "0.603", "the turn is inside [0,1], so f is not monotone");
    // u = 0.604 is z = 1/u - 1 = 0.656, so any query below that redshift
    // straddles the turn.
    assert!(turn.hi().cmp_exact(&Ratio::one()) == core::cmp::Ordering::Less);
}

// ---------------------------------------------------------------------------
// the exit criterion
// ---------------------------------------------------------------------------

#[test]
fn z_1100_lands_inside_the_catalogue_recombination_window() {
    let m = model();
    let z = Ratio::from_decimal_str("1100").unwrap();
    let out = m.t_of_z(&z, D, S).unwrap();

    let rec = ucal_events::by_id("recombination").unwrap();
    assert!(
        rec.window.contains(out.value.lo()) && rec.window.contains(out.value.hi()),
        "z=1100 gave {} to {} years; the catalogue window is {}",
        years(out.value.lo().ticks()),
        years(out.value.hi().ticks()),
        rec.as_published
    );

    // And it is where the literature puts it: a few hundred thousand years.
    let lo: u64 = years(out.value.lo().ticks()).parse().unwrap();
    let hi: u64 = years(out.value.hi().ticks()).parse().unwrap();
    assert!(
        (300_000..=420_000).contains(&lo) && (300_000..=420_000).contains(&hi),
        "z=1100 should be a few hundred kyr, got {lo}..{hi}"
    );
}

#[test]
fn the_enclosure_narrows_monotonically_with_depth() {
    let m = model();
    let z = Ratio::from_decimal_str("1100").unwrap();
    let mut prev: Option<Delta> = None;
    for depth in [4u32, 6, 8, 10] {
        let out = m.t_of_z(&z, depth, S).unwrap();
        let w = out.arithmetic_width.clone();
        if let Some(p) = prev {
            assert!(
                w <= p,
                "depth {depth} widened the arithmetic enclosure: {} vs {}",
                w.ticks().to_dec_string(),
                p.ticks().to_dec_string()
            );
        }
        prev = Some(w);
    }
}

#[test]
fn the_enclosure_is_rigorous_and_ordered() {
    let m = model();
    for zs in ["0", "0.5", "1", "10", "1100"] {
        let z = Ratio::from_decimal_str(zs).unwrap();
        let out = m.t_of_z(&z, 8, S).unwrap();
        assert!(
            out.value.lo() <= out.value.hi(),
            "z={zs} produced an inverted enclosure"
        );
        assert!(!out.value.is_exact(), "z={zs} claimed exactness");
    }
}

#[test]
fn the_age_at_redshift_zero_matches_the_declared_datum() {
    // The strongest available check on the whole apparatus: t(0) is the age of
    // the universe, and the datum was built from Planck's 13.787 Gyr. The two
    // arrive by completely different routes — one from a published scalar, one
    // from integrating the model — and must agree.
    let m = model();
    let out = m.t_of_z(&Ratio::zero(), 12, S).unwrap();
    let lo: u64 = years(out.value.lo().ticks()).parse().unwrap();
    let hi: u64 = years(out.value.hi().ticks()).parse().unwrap();
    assert!(
        lo < 13_787_000_000 && hi > 13_787_000_000,
        "the enclosure at z=0 must contain the declared age: {lo}..{hi}"
    );
}

// ---------------------------------------------------------------------------
// Rule X — two widths, never merged
// ---------------------------------------------------------------------------

#[test]
fn the_two_widths_are_reported_separately() {
    // F8: "Float error and parameter uncertainty conflated into one tolerance."
    // The result carries both, and they are wildly different sizes — which is
    // exactly why merging them would mislead.
    let m = model();
    let out = m
        .t_of_z(&Ratio::from_decimal_str("1100").unwrap(), D, S)
        .unwrap();
    assert!(out.arithmetic_width.ticks() > &<Ticks as TickInt>::zero());
    assert!(out.parameter_width.ticks() > &<Ticks as TickInt>::zero());
    // The parameter width dominates by orders of magnitude.
    assert!(
        out.parameter_width > out.arithmetic_width,
        "at this depth the model's own uncertainty must dominate the quadrature's"
    );
    // Summing is possible, but only by name.
    assert_eq!(
        out.total_width().unwrap(),
        out.arithmetic_width.checked_add(&out.parameter_width).unwrap()
    );
}

#[test]
fn every_result_carries_its_model_and_citation() {
    // Rule X: "Model, parameter set, and citation MUST accompany every result."
    let m = model();
    let out = m.t_of_z(&Ratio::from_decimal_str("5").unwrap(), 6, S).unwrap();
    assert_eq!(out.model.0, "flat-LambdaCDM/planck2018");
    assert!(out.citation.source.contains("Planck 2018"));
    assert_eq!(out.depth, 6);
    assert_eq!(out.scale, S);
}

#[test]
fn results_wider_than_a_tick_say_so() {
    // UCAL-W0004. Every real result is wider than a tick, which is the honest
    // outcome and the one GE-2 anticipates.
    let m = model();
    let out = m.t_of_z(&Ratio::from_decimal_str("1100").unwrap(), D, S).unwrap();
    assert!(out.warnings.contains(&Warning::W0004));
}

#[test]
fn results_inside_the_claim_half_width_carry_w0006() {
    // §10.6. Recombination is deep inside it.
    let m = model();
    let out = m.t_of_z(&Ratio::from_decimal_str("1100").unwrap(), D, S).unwrap();
    assert!(out.warnings.contains(&Warning::W0006));
    // The present epoch is not.
    let now = m.t_of_z(&Ratio::zero(), 8, S).unwrap();
    assert!(!now.warnings.contains(&Warning::W0006));
}

// ---------------------------------------------------------------------------
// §10.4 — inversion
// ---------------------------------------------------------------------------

/// One Julian year, in ticks — a tolerance a cosmological inversion can
/// actually reach.
fn one_year() -> Delta {
    Delta::from_ticks(
        UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
            .unwrap(),
    )
}

/// One SI second, in ticks. C4 measured this at 71 halvings — inside the budget
/// now, and outside the 64 that shipped in 0.2.0.
fn one_second() -> Delta {
    Delta::from_ticks(UC1::bridge().ticks.clone())
}

#[test]
fn inversion_recovers_the_redshift() {
    let m = model();
    for zs in ["1", "10"] {
        let z = Ratio::from_decimal_str(zs).unwrap();
        let t = m.t_of_z(&z, 8, S).unwrap();
        let back = m.z_of_t(&t.value, &one_year(), 6, S).unwrap();
        assert!(
            back.value.contains(&z),
            "z={zs} inverted to {}..{}, which does not contain it",
            back.value.lo().to_decimal_string(4, Rounding::Trunc).unwrap(),
            back.value.hi().to_decimal_string(4, Rounding::Trunc).unwrap()
        );
    }
}

#[test]
fn inversion_brackets_both_sides_of_an_interval_valued_age() {
    // The bracket must be *wide*, because the parameters are intervals: a whole
    // range of redshifts is consistent with any given age. A narrow answer here
    // would mean the inversion had bisected one bound and ignored the other.
    let m = model();
    let z = Ratio::from_decimal_str("1100").unwrap();
    let t = m.t_of_z(&z, 8, S).unwrap();
    let back = m.z_of_t(&t.value, &one_year(), 6, S).unwrap();
    assert!(back.value.contains(&z));
    let width = back.value.width().unwrap();
    assert!(
        width.cmp_exact(&Ratio::from_u64(1)) == core::cmp::Ordering::Greater,
        "an interval-valued model cannot pin z at recombination to better than \
         a unit; got a width of {}",
        width.to_decimal_string(6, Rounding::Trunc).unwrap()
    );
}

#[test]
fn a_sub_tick_inversion_tolerance_is_refused() {
    // §10.4: "the width MUST be >= 1 tick". N10 forbids sub-tick representation.
    let m = model();
    let t = m.t_of_z(&Ratio::one(), 6, S).unwrap();
    let e = m.z_of_t(&t.value, &Delta::zero(), 6, S).unwrap_err();
    assert_eq!(e.code, Code::E0071);
}

#[test]
fn a_tick_tolerance_is_refused_as_unreachable_rather_than_faked() {
    // One tick is *permitted* by §10.4 and still unreachable. C4 measured why,
    // and it is not the step budget: the bisection reaches step 125 before a
    // midpoint's denominator leaves the 512-bit domain, with the bracket still
    // ~7.8e26 ticks wide. Returning a bracket anyway would claim a resolution
    // the method does not have.
    let m = model();
    let t = m.t_of_z(&Ratio::one(), 6, S).unwrap();
    let e = m.z_of_t(&t.value, &Delta::one_tick(), 6, S).unwrap_err();
    assert_eq!(e.code, Code::E0071);
    // The message names the measured floor, not the budget.
    let msg = e.to_string();
    assert!(msg.contains("millisecond"), "unexpected message: {msg}");
    assert!(msg.contains("512-bit domain"), "unexpected message: {msg}");
}

#[test]
fn a_one_second_tolerance_is_reachable() {
    // C4's operative finding. This needs 71 halvings, so under the 64 that
    // shipped in 0.2.0 it returned E0071 — "did not reach the requested
    // tolerance within the step budget" — for a tolerance that is reachable.
    // The message was true and its implication was not.
    let m = model();
    let t = m.t_of_z(&Ratio::one(), 6, S).unwrap();
    let back = m
        .z_of_t(&t.value, &one_second(), 6, S)
        .expect("a one-second tolerance must converge within the budget");
    assert!(
        back.value.contains(&Ratio::one()),
        "the inversion lost the redshift it was given"
    );
}

#[test]
fn the_budget_is_the_one_the_measurement_supports() {
    // 96 is not a round number chosen for comfort. It is above the 81 halvings a
    // millisecond needs and below the 125 at which the failure stops being about
    // steps at all. Pinned so that changing it requires re-reading why.
    assert_eq!(LambdaCdm::MAX_BISECT_STEPS, 96);
    assert!(crate::C4_ACHIEVABLE_TOLERANCE.contains("UCAL-E0021"));
}

#[test]
fn an_absurd_depth_is_refused_rather_than_attempted() {
    let m = model();
    let e = m.t_of_z(&Ratio::one(), 31, S).unwrap_err();
    assert_eq!(e.code, Code::E0071);
}

// ---------------------------------------------------------------------------
// §21.2 — the float oracle, contained
// ---------------------------------------------------------------------------

/// A floating-point reference implementation, permitted by Rule E **only** as a
/// test oracle and required to be marked as such.
///
/// It is confined to this module, never compiled into a shipped artefact, and
/// used for exactly one purpose: asserting that the certified enclosure
/// *contains* the oracle's answer. The enclosure is the result; the oracle is
/// only evidence that the enclosure is not centred on the wrong number.
// ucal-lint-allow-begin(float-free): Rule E permits a float reference
// implementation in test code, marked as such. Everything between this marker
// and its `-end` is `#[cfg(test)]`, unreachable from any shipped artefact, and
// used only to check that the certified enclosure contains the oracle's answer.
mod float_oracle {
    pub fn age_seconds(z: f64, om: f64, ol: f64, orr: f64, hubble_time_s: f64) -> f64 {
        let u0 = 1.0 / (1.0 + z);
        let n = 200_000;
        let h = u0 / n as f64;
        let mut acc = 0.0;
        for i in 0..n {
            let u = h * (i as f64 + 0.5);
            acc += u / (orr + om * u + ol * u * u * u * u).sqrt();
        }
        acc * h * hubble_time_s
    }
}

#[test]
fn the_enclosure_contains_the_float_oracles_value() {
    // §21.2: "asserting only that the certified enclosure contains the oracle's
    // value". Not equality — the enclosure is rigorous and the oracle is not.
    let m = model();
    let second = Ratio::from_int(UC1::bridge().ticks);

    for (zs, depth) in [("1100", 12u32), ("10", 12), ("0", 12)] {
        let z = Ratio::from_decimal_str(zs).unwrap();
        let out = m.t_of_z(&z, depth, S).unwrap();

        // Central parameter values, as floats, for the oracle only.
        let mid = |iv: &RatInterval| -> f64 {
            iv.lo()
                .add(iv.hi())
                .unwrap()
                .div(&Ratio::from_u64(2))
                .unwrap()
                .to_decimal_string(12, Rounding::HalfEven)
                .unwrap()
                .parse::<f64>()
                .unwrap()
        };
        let ht = m
            .hubble_time
            .lo()
            .add(m.hubble_time.hi())
            .unwrap()
            .div(&Ratio::from_u64(2))
            .unwrap()
            .div(&second)
            .unwrap()
            .to_decimal_string(3, Rounding::HalfEven)
            .unwrap()
            .parse::<f64>()
            .unwrap();

        let oracle_s = float_oracle::age_seconds(
            zs.parse::<f64>().unwrap(),
            mid(&m.omega_m),
            mid(&m.omega_l),
            mid(&m.omega_r),
            ht,
        );

        // Convert the oracle's answer into ticks and check containment.
        let oracle_ticks = Ratio::from_decimal_str(&format!("{oracle_s:.0}"))
            .unwrap()
            .mul(&second)
            .unwrap()
            .floor();
        let oracle_instant = Instant::<UC1>::from_ticks(oracle_ticks).unwrap();
        assert!(
            out.value.contains(&oracle_instant),
            "z={zs}: the enclosure {}..{} yr does not contain the oracle's {} yr",
            years(out.value.lo().ticks()),
            years(out.value.hi().ticks()),
            years(oracle_instant.ticks())
        );
    }
}

// ucal-lint-allow-end(float-free)

// ---------------------------------------------------------------------------
// §21 GE-6 — CMB-anchored provenance for a hypothetical UC-2
// ---------------------------------------------------------------------------

#[test]
fn ge6_a_cmb_anchored_datum_is_wider_than_the_published_one() {
    // GE-6 asks whether deriving the datum offset through this crate, anchored on
    // the CMB, "produces a shorter, fully tick-native chain", with the kill
    // criterion: "if the resulting enclosure is wider than the current published
    // age uncertainty, the route adds auditability without adding rigour; leave
    // D-21 standing."
    //
    // It is wider, by an order of magnitude, and no amount of depth would close
    // the gap: the width is parameter-dominated, not quadrature-dominated. The
    // derived route would replace one cited scalar with four cited scalars and a
    // quadrature, and arrive less certain than it started.
    let m = model();
    let derived = m.t_of_z(&Ratio::zero(), 12, S).unwrap();

    let published = UC1::big_bang_claim().hi().magnitude().ticks().clone();
    let published_width = published
        .try_mul(&<Ticks as TickInt>::from_u64(2))
        .unwrap();

    assert!(
        derived.value.width().ticks() > &published_width,
        "GE-6's kill criterion did not fire: derived {} yr vs published {} yr",
        years(derived.value.width().ticks()),
        years(&published_width)
    );

    // And the reason is the parameters, not the arithmetic.
    assert!(
        derived.parameter_width > derived.arithmetic_width,
        "if the arithmetic dominated, more depth would be the answer"
    );
}

// ---------------------------------------------------------------------------
// §21 GE-1 / GE-2 — measured, not predicted
// ---------------------------------------------------------------------------

/// GE-1: "Certified interval quadrature at the depth needed for a useful
/// enclosure may be too slow. Kill criterion: if depth-24 quadrature exceeds
/// ~2 s, reduce the default depth and expose a high-precision mode."
///
/// GE-2: "Publish the achievable width." Both are measurements, so both are
/// `#[ignore]`d and run on demand: a timing assertion in the default suite would
/// be a flake, and the numbers belong in the documentation rather than in a
/// green tick.
#[test]
#[ignore = "measurement, not an assertion; run with --ignored --nocapture"]
fn ge1_and_ge2_measured() {
    use std::time::Instant as Clock;
    let m = model();
    let z = Ratio::from_decimal_str("1100").unwrap();
    let tick = Ratio::from_int(<Ticks as TickInt>::one());
    let year = Ratio::from_int(
        UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
            .unwrap(),
    );

    println!(
        "\n depth   panels        wall   arith width (ticks)  ~yr   ticks/enclosure"  // ucal-lint-allow(no-indent-in-literal): a column header, aligned on purpose
    );
    for depth in [4u32, 8, 12, 14, 16, 18, 20] {
        let t0 = Clock::now();
        let out = match m.t_of_z(&z, depth, S) {
            Ok(o) => o,
            Err(e) => {
                println!(" {depth:>5}   refused: {e}");
                continue;
            }
        };
        let dt = t0.elapsed();
        let w = Ratio::from_int(out.arithmetic_width.ticks().clone());
        println!(
            " {:>5}   {:>7}   {:>9.3?}   {:>18}   {:>4}   {}",
            depth,
            1u64 << depth,
            dt,
            w.to_decimal_string(0, Rounding::Trunc).unwrap(),
            w.div(&year).unwrap().to_decimal_string(1, Rounding::Trunc).unwrap(),
            if w.cmp_exact(&tick) == core::cmp::Ordering::Greater {
                "wider than 1 tick"
            } else {
                "<= 1 tick"
            }
        );
        if dt.as_secs_f64() > 4.0 {
            println!(" (stopping: past the GE-1 kill threshold)");
            break;
        }
    }

    let out = m.t_of_z(&z, 12, S).unwrap();
    println!(
        "\n GE-2 at depth 12, scale {S}: parameter width {} yr, arithmetic width {} yr",
        Ratio::from_int(out.parameter_width.ticks().clone())
            .div(&year)
            .unwrap()
            .to_decimal_string(1, Rounding::Trunc)
            .unwrap(),
        Ratio::from_int(out.arithmetic_width.ticks().clone())
            .div(&year)
            .unwrap()
            .to_decimal_string(1, Rounding::Trunc)
            .unwrap(),
    );
    println!(
        " t(z=1100) = {} .. {} yr\n",
        years(out.value.lo().ticks()),
        years(out.value.hi().ticks())
    );
}

// ---------------------------------------------------------------------------
// The tick quantisation, which is the last rounding in the chain
// ---------------------------------------------------------------------------

#[test]
fn quantising_to_ticks_rounds_outward_on_both_ends() {
    // Every rounding in this computation widens: the densities are taken at
    // opposite ends, the roots are directed apart, the accumulator snaps outward
    // and the two sums multiply by opposite ends of 1/H0. The last step turns
    // two rationals into two tick counts, and it has to widen too.
    //
    // It did not. Both ends were floored, and flooring the *upper* bound moves
    // it down — inward — so the enclosure could exclude a true value lying in
    // the fraction that was discarded. Found by writing V4's audit and asking
    // what direction each step rounds in.
    let m = model();
    let z = Ratio::from_u64(1100);
    let u0 = Ratio::one().add(&z).unwrap().recip().unwrap();
    let full = super::integral_enclosure(&m, &u0, 6, S).unwrap();

    let t_lo = full.lo().mul(m.hubble_time.lo()).unwrap();
    let t_hi = full.hi().mul(m.hubble_time.hi()).unwrap();
    // Neither bound lands on an integer, so the direction is not academic.
    assert!(!t_lo.frac().is_zero(), "t_lo happens to be integral; pick another z");
    assert!(!t_hi.frac().is_zero(), "t_hi happens to be integral; pick another z");

    let out = m.t_of_z(&z, 6, S).unwrap().value;
    let lo_q = Ratio::from_int(out.lo().ticks().clone());
    let hi_q = Ratio::from_int(out.hi().ticks().clone());

    assert!(
        lo_q <= t_lo,
        "the quantised lower bound rose above the computed one"
    );
    assert!(
        hi_q >= t_hi,
        "the quantised upper bound fell below the computed one: the enclosure \
         no longer provably contains what the quadrature bounded"
    );
}
