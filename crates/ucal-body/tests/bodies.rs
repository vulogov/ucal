//! The bodies added in 0.8.0, and the claims made about them.
//!
//! A3's argument is that a body without an anchor is the *ordinary* case and
//! Earth and Mars are the exceptions. That argument is only worth making if the
//! mechanism really is body-independent — if `luna-d` and `mercury-d` are
//! produced by the same code path as `earth-d` and not by a special case wearing
//! a different name.
//!
//! These tests check the claims the data files make in prose, because prose is
//! where a wrong number survives longest.

use ucal_body::{calendar, data, derive_leap_rule, DriftBound};
use ucal_core::backend::TickInt;
use ucal_core::num::Ratio;
use ucal_core::{Profile, Rounding, Ticks, UC1};

/// A duration in ticks, rendered in SI days.
///
/// The parameters are `Ratio`s of ticks, so comparing two of them at two
/// decimal places compares to a hundredth of a Planck time — which is not what
/// "agrees with the published value" means for a number published to four
/// significant figures. Bringing both sides into days first makes the
/// comparison the one the source supports. A day is an Earth unit and this is
/// a test, not output; Rule A.5 is about what the program tells a reader.
fn days(r: &Ratio, digits: u32) -> String {
    let day = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(86_400))
        .expect("a day fits the domain");
    // ucal-lint-allow-begin(rounding-is-declared): in a test the caller *is*
    // this file, and it declares the mode here — there is no outer caller whose
    // choice could be honoured instead. The digit count is passed in by each
    // test, set from the precision its published source actually carries.
    r.div(&Ratio::from_int(day))
        .expect("non-zero")
        .to_decimal_string(digits, Rounding::HalfEven)
        .expect("renderable")
    // ucal-lint-allow-end(rounding-is-declared)
}

/// Every registered calendar's body derives an intercalation rule.
///
/// `ucal cal list` prints one for the anchorless calendars and says they are
/// "complete in units, intercalation and cycles". If a body could not produce a
/// rule, that sentence would be false for it and the row would be showing a
/// claim it cannot support.
#[test]
fn every_registered_body_derives_a_rule() {
    for (id, body, _) in calendar::registered() {
        let rule = derive_leap_rule(
            body.solar_day().value_at_epoch(),
            body.orbital_period().value_at_epoch(),
            DriftBound::DEFAULT,
            32,
        );
        assert!(
            rule.is_ok(),
            "`{id}` is registered and its body derives no leap rule, so the \
             `cal list` row claims an intercalation it does not have"
        );
    }
}

/// No anchor was invented for any of the new bodies.
///
/// The temptation with a new body is to give it a zero so that the calendar
/// renders. GE-3 forbids it, and this is the check that the temptation was
/// resisted — stated as an exact set, so that adding an anchor is a deliberate
/// edit to this test rather than something that slips in.
#[test]
fn only_earth_and_mars_have_anchors() {
    let anchored: Vec<&str> = calendar::registered()
        .into_iter()
        .map(|(id, _, _)| id)
        .filter(|id| ucal_body::anchors::for_calendar(id).is_some())
        .collect();
    assert_eq!(
        anchored,
        vec!["earth-d", "mars-d"],
        "an anchor appeared for a body that has no published phase convention"
    );
}

/// The Moon's published synodic month is what the synodic formula gives.
///
/// `data::luna`'s solar day is the *published* 29.53 d, where Titan's has to be
/// derived because nobody publishes one. That makes the Moon a free check on
/// the derivation Titan depends on: the same formula, against a number somebody
/// else measured.
#[test]
fn luna_synodic_month_agrees_with_the_derivation() {
    let luna = data::luna();
    let rot = luna.rotation_period().value_at_epoch();
    let year = luna.orbital_period().value_at_epoch();

    // 1 / (1/P_rot - 1/P_year), the synodic period.
    let derived = rot
        .recip()
        .and_then(|a| year.recip().and_then(|b| a.abs_diff(&b)))
        .and_then(|d| d.recip())
        .expect("the Moon's periods differ");

    let published = luna.solar_day().value_at_epoch();
    // To the published precision: 29.53 d is four significant figures, and the
    // derivation lands on 29.53 as well.
    assert_eq!(
        days(&derived, 2),
        days(published, 2),
        "the synodic derivation disagrees with the published synodic month"
    );
    assert_eq!(days(published, 2), "29.53");
}

/// Mercury's solar day outlasts its year, and Rule K does not blink.
///
/// The 3:2 spin–orbit resonance makes the solar day about twice the orbit, so a
/// Mercurian calendar has fewer than one "day" per "year" — the ratio Rule K
/// intercalates is below 1, where every other body's is above it. There is no
/// special case for this; the test exists because "no special case" is a claim.
#[test]
fn mercury_has_fewer_than_one_day_per_year() {
    let m = data::mercury();
    let solar = m.solar_day().value_at_epoch();
    let year = m.orbital_period().value_at_epoch();

    assert_eq!(
        solar.cmp_exact(year),
        core::cmp::Ordering::Greater,
        "Mercury's solar day should be longer than its year"
    );

    let rule = derive_leap_rule(solar, year, DriftBound::DEFAULT, 32)
        .expect("Rule K must derive a rule for a ratio below one");
    // The resonance in one number: the year is half a solar day.
    assert_eq!(rule.chosen.value.numer().to_dec_string(), "1");
    assert_eq!(rule.chosen.value.denom().to_dec_string(), "2");
}

/// Venus's published solar day is only explicable by retrograde rotation.
///
/// `data::venus` records that [`ucal_body::param::Measured`] has no sign, so the
/// rotation is stored as a magnitude and the retrograde sense lives in a
/// comment. This is that comment, made checkable: run the synodic formula with
/// the magnitude and the answer is wrong by a factor of twenty-five, which is
/// what the missing sign costs.
///
/// It also pins the value: if someone "corrects" the solar day to the number the
/// unsigned arithmetic produces, this fails.
#[test]
fn venus_solar_day_needs_the_sign_the_model_cannot_hold() {
    let v = data::venus();
    let rot = v.rotation_period().value_at_epoch();
    let year = v.orbital_period().value_at_epoch();

    // What the magnitude alone gives.
    let unsigned = rot
        .recip()
        .and_then(|a| year.recip().and_then(|b| a.abs_diff(&b)))
        .and_then(|d| d.recip())
        .expect("periods differ");

    // What is published, and what `venus-d` is actually built from.
    let published = v.solar_day().value_at_epoch();

    let ratio = unsigned
        .div(published)
        .expect("a non-zero solar day");
    // ucal-lint-allow-begin(rounding-is-declared): an order-of-magnitude check,
    // not a rendered value. The assertion below is `> 10` against a ratio of
    // about 25, so the mode cannot change the outcome.
    let times: u64 = ratio
        .to_decimal_string(0, Rounding::HalfEven)
        .unwrap()
        .parse()
        .unwrap();
    // ucal-lint-allow-end(rounding-is-declared)
    assert!(
        times > 10,
        "the unsigned derivation should be wildly wrong for a retrograde body; \
         it came out {times}x the published value, so either the sign no longer \
         matters here or a parameter has changed"
    );

    // And the signed form does land on it: 1 / (1/P_year + 1/P_rot).
    let signed = year
        .recip()
        .and_then(|a| rot.recip().and_then(|b| a.add(&b)))
        .and_then(|d| d.recip())
        .expect("periods are non-zero");
    assert_eq!(
        days(&signed, 2),
        days(published, 2),
        "the retrograde synodic period should reproduce the published solar day"
    );
    // 2802.0 h, which is what the fact sheet prints.
    assert_eq!(days(published, 2), "116.75");
}

/// Jupiter, a body with no surface, still produces a calendar.
///
/// Recorded rather than assumed: the mechanism takes periods and returns a
/// rule, and it has no opinion about whether the rotation belongs to ground or
/// to a magnetic field. That is either elegant or absurd depending on the
/// reader, and it is worth being explicit that it is deliberate.
#[test]
fn a_body_with_no_surface_still_derives() {
    let j = data::jupiter();
    assert!(derive_leap_rule(
        j.solar_day().value_at_epoch(),
        j.orbital_period().value_at_epoch(),
        DriftBound::DEFAULT,
        32,
    )
    .is_ok());
}

/// Every new body's parameters carry the four Rule C obligations.
///
/// Epoch, window, citation, verbatim value. The constructor enforces it, so this
/// is a check that the enforcement was not bypassed by some other route.
#[test]
fn every_body_parameter_is_cited() {
    for body in data::all() {
        for (what, p) in [
            ("rotation", body.rotation_period()),
            ("solar_day", body.solar_day()),
            ("orbital_period", body.orbital_period()),
        ] {
            assert!(
                !p.citation().source.is_empty(),
                "{}'s {what} has no citation",
                body.id()
            );
        }
    }
}

/// A parameter is a `Ratio`; this file compares them, so the helper is real.
#[allow(dead_code)]
fn _typecheck(r: &Ratio) -> &Ratio {
    r
}
