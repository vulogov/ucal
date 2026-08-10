//! Y3 — the five tidally locked moons, and the check that caught a wrong source.
//!
//! # Why this file exists
//!
//! JPL's satellite mean elements page publishes a `P(days)` column for every
//! moon. It is not the sidereal orbital period — it is taken against a
//! precessing frame — and for Io it differs from the sidereal figure in the
//! third decimal. Nothing about the number looks wrong. It is published by JPL,
//! it is labelled a period in days, it is the right order of magnitude, and a
//! calendar derived from it would have been silently different from a calendar
//! derived from the right one, forever.
//!
//! What caught it is a fact about the bodies rather than a fact about the
//! source: Io, Europa and Ganymede are in a Laplace resonance, which fixes
//!
//! ```text
//!   n_Io - 3·n_Europa + 2·n_Ganymede = 0
//! ```
//!
//! exactly, where `n` is mean motion. This is not a convention or a rounding —
//! it is the dynamical state the three moons are locked into, and it holds to
//! whatever precision the periods are published at.
//!
//! So the parameters are checkable against physics without trusting the page
//! they came from, and this test is that check. It is the same shape as every
//! other check in this project: **a claim with no mechanism is the claim that is
//! wrong**, and "these are the sidereal periods" was a claim with no mechanism
//! until the resonance supplied one.

use ucal_body::data;
use ucal_core::backend::TickInt;
use ucal_core::Ratio;

/// Mean motion, as a rate: `1 / P`.
fn ratio(num: u64, den: u64) -> Ratio {
    Ratio::new(
        <ucal_core::Ticks as TickInt>::from_u64(num),
        <ucal_core::Ticks as TickInt>::from_u64(den),
    )
    .expect("a valid ratio")
}

fn n(p: &Ratio) -> Ratio {
    p.recip().expect("a period is non-zero")
}

/// The Laplace resonance, evaluated exactly.
///
/// Computed in rationals, so what is measured is the residual of the published
/// figures and not the residual of a float sum.
#[test]
fn the_galilean_periods_satisfy_the_laplace_resonance() {
    let io = n(data::io().rotation_period().value_at_epoch());
    let eu = n(data::europa().rotation_period().value_at_epoch());
    let ga = n(data::ganymede().rotation_period().value_at_epoch());

    // n_Io + 2·n_Ganymede - 3·n_Europa, kept positive-side-first so the two
    // subtractions are of comparable magnitudes.
    let lhs = io
        .add(&ga.mul(&Ratio::from_u64(2)).expect("finite"))
        .expect("finite");
    let rhs = eu.mul(&Ratio::from_u64(3)).expect("finite");
    let residual = lhs.abs_diff(&rhs).expect("finite");

    // The published figures carry six decimals, so the residual is bounded by
    // the rounding in them and nothing else. A wrong-frame period misses by
    // four thousandths — nearly four orders of magnitude above this bound.
    let bound = ratio(1, 100_000);
    assert!(
        residual.cmp_exact(&bound) == core::cmp::Ordering::Less,
        "the Galilean periods do not satisfy the Laplace resonance; residual is \
         larger than the published precision can explain, which means at least \
         one of them is not a sidereal period"
    );
}

/// The resonance is a real check and not one that anything would pass.
///
/// The value pinned here is JPL's `P(days)` for Io — a published figure, from a
/// reputable source, wrong for this purpose. If substituting it did not break
/// the test above, that test would be decoration.
#[test]
fn a_wrong_frame_period_fails_that_check() {
    // 1.762732 d, JPL satellite mean elements, P(days) column.
    let wrong = ratio(1_762_732, 1_000_000);
    let io = n(&wrong);
    let eu = n(data::europa().rotation_period().value_at_epoch());
    let ga = n(data::ganymede().rotation_period().value_at_epoch());

    let lhs = io
        .add(&ga.mul(&Ratio::from_u64(2)).expect("finite"))
        .expect("finite");
    let rhs = eu.mul(&Ratio::from_u64(3)).expect("finite");
    let residual = lhs.abs_diff(&rhs).expect("finite");
    let bound = ratio(1, 100_000);
    assert!(
        residual.cmp_exact(&bound) != core::cmp::Ordering::Less,
        "the resonance check accepts a known-wrong period, so it checks nothing"
    );
}

/// A locked moon's solar day is longer than its rotation, and by the synodic amount.
///
/// Tidal lock is the whole reason these bodies need a derived solar day: the
/// moon's face is fixed towards its primary, not towards the Sun. The failure
/// this guards against is the obvious simplification — solar day = rotation —
/// which is right for no body in the catalogue and looks right for all of them.
#[test]
fn every_locked_moon_has_a_solar_day_longer_than_its_rotation() {
    for body in [
        data::io(),
        data::europa(),
        data::ganymede(),
        data::callisto(),
        data::enceladus(),
        data::titan(),
    ] {
        let rot = body.rotation_period().value_at_epoch();
        let solar = body.solar_day().value_at_epoch();
        assert_eq!(
            solar.cmp_exact(rot),
            core::cmp::Ordering::Greater,
            "{}'s solar day is not longer than its rotation",
            body.id()
        );
        // And by the synodic amount exactly: 1/solar = 1/rot - 1/year.
        let year = body.orbital_period().value_at_epoch();
        let expect = rot
            .recip()
            .expect("non-zero")
            .abs_diff(&year.recip().expect("non-zero"))
            .and_then(|d| d.recip())
            .expect("finite");
        assert_eq!(
            solar.cmp_exact(&expect),
            core::cmp::Ordering::Equal,
            "{}'s solar day is not the synodic period of its rotation and year",
            body.id()
        );
    }
}

/// Every new body derives a leap rule, and none of them is Earth's.
///
/// Rule K.5 says Earth is an ordinary instance. Five bodies added without the
/// mechanism moving is the only evidence that claim can have, and this is the
/// third time it has been collected — 0.8.0 added four, 1.4.0's loader added a
/// route, and this adds five.
#[test]
fn every_new_body_derives_its_own_rule() {
    let earth = rule_of(&data::earth());
    for body in [
        data::io(),
        data::europa(),
        data::ganymede(),
        data::callisto(),
        data::enceladus(),
    ] {
        let r = rule_of(&body);
        assert_ne!(
            r.chosen.value.denom().to_dec_string(),
            earth.chosen.value.denom().to_dec_string(),
            "{} derived Earth's intercalation, which would mean the derivation \
             is not reading the body",
            body.id()
        );
    }
}

fn rule_of(b: &ucal_body::Body) -> ucal_body::LeapRule {
    ucal_body::derive_leap_rule(
        b.solar_day().value_at_epoch(),
        b.orbital_period().value_at_epoch(),
        ucal_body::DriftBound::DEFAULT,
        32,
    )
    .expect("a rule")
}
