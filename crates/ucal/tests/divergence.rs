//! §21.3-8: **`earth-d` and `earth-civil` diverge, with the divergence measured
//! and recorded — not asserted away.**
//!
//! §9.8 is blunt about it: "The derived Earth calendar (`earth-d`) will **not**
//! reproduce the civil Gregorian calendar and MUST NOT be presented as doing so."
//!
//! The temptation such a rule guards against is real. A derived Earth calendar
//! looks enough like the civil one to invite the claim that it *is* the civil one
//! done properly, and that claim would be false in at least three independent
//! ways. This file measures each.

use ucal_body::calendar as bodycal;
use ucal_civil::calendar::CivilCalendar;
use ucal_civil::legacy::{Gregorian, LegacyCalendar};
use ucal_civil::si::{self, Scale, SubSecond};
use ucal_core::backend::TickInt;
use ucal_core::{Delta, Instant, Profile, Rounding, Ticks, UC1};

fn tt(y: i64, m: u8, d: u8) -> Instant<UC1> {
    si::from_civil(
        y, m, d, 0, 0, 0,
        SubSecond::zero(), Scale::Tt, CivilCalendar::Gregorian,
    )
    .unwrap()
}

fn day_ticks() -> Ticks {
    UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(86_400))
        .unwrap()
}

#[test]
fn divergence_1_the_intercalation_rules_differ() {
    // The Gregorian calendar inserts 97 leap days per 400 years. The derived
    // calendar reaches 31/128 at the default bound and never produces 97/400 at
    // any depth, because it is not a convergent of the tropical year.
    let e = bodycal::by_id("earth-d").unwrap();
    let rule = e.leap_rule();

    assert_eq!(rule.chosen.value.numer().to_dec_string(), "31");
    assert_eq!(rule.chosen.value.denom().to_dec_string(), "128");
    assert!(!rule.contains(97, 400), "97/400 must not appear at any depth");

    let civil = Gregorian.tables().leap_rule;
    assert_eq!((civil.numerator, civil.denominator), (97, 400));
    assert!(!civil.is_convergent);

    // Measured: how much better the derived rule is.
    let civil_ratio = ucal_core::num::Ratio::new(
        <Ticks as TickInt>::from_u64(97),
        <Ticks as TickInt>::from_u64(400),
    )
    .unwrap();
    let civil_err = civil_ratio.abs_diff(&rule.fraction).unwrap();
    let ratio = civil_err.div(&rule.chosen.error).unwrap();
    assert_eq!(
        ratio.to_decimal_string(1, Rounding::HalfEven).unwrap(),
        "124.0",
        "the derived rule is 124x more accurate, with a denominator 3x smaller"
    );
}

#[test]
fn divergence_2_the_year_lengths_differ_over_a_cycle() {
    // Computed from the rules rather than walked day by day: the cumulative days
    // before local year `y` are `y x whole + floor(y x p / q)`, which is the same
    // arithmetic the calendar itself uses and needs no iteration.
    let e = bodycal::by_id("earth-d").unwrap();
    let rule = e.leap_rule();

    let years = 400u64;
    let whole: u64 = rule.whole_days.numer().to_dec_string().parse().unwrap();
    let p: u64 = rule.chosen.value.numer().to_dec_string().parse().unwrap();
    let q: u64 = rule.chosen.value.denom().to_dec_string().parse().unwrap();

    let derived_400 = years * whole + (years * p) / q;
    let gregorian_400 = years * 365 + 97;

    assert_eq!(gregorian_400, 146_097, "the Gregorian cycle is 146097 days");
    assert_eq!(derived_400, 146_096, "400 derived years at 31/128");
    assert_ne!(
        derived_400, gregorian_400,
        "the two calendars must not agree over a cycle"
    );

    // Measured: one day per four centuries, and it accumulates without bound.
    let gap = gregorian_400 as i64 - derived_400 as i64;
    assert_eq!(gap, 1, "400 Gregorian years run one day longer");
    // Over ten thousand years the two are four days apart. But the telling
    // comparison is not against each other — it is against the tropical year they
    // are both approximating, which is what a leap rule is *for*:
    //
    //   10 000 tropical years need 2421.90 leap days
    //   31/128  inserts 2421   ->  0.90 days adrift
    //   97/400  inserts 2425   ->  3.10 days adrift
    //
    // The derived rule is closer, and it got there without being told the answer.
    let years = 10_000u64;
    let derived = years * whole + (years * p) / q;
    let gregorian = years * 365 + (years * 97) / 400;
    assert_eq!(gregorian as i64 - derived as i64, 4);

    let needed = rule
        .fraction
        .mul(&ucal_core::num::Ratio::from_u64(years))
        .unwrap();
    let derived_drift = needed
        .abs_diff(&ucal_core::num::Ratio::from_u64((years * p) / q))
        .unwrap();
    let gregorian_drift = needed
        .abs_diff(&ucal_core::num::Ratio::from_u64((years * 97) / 400))
        .unwrap();
    assert_eq!(
        derived_drift.to_decimal_string(2, Rounding::HalfEven).unwrap(),
        "0.90"
    );
    assert_eq!(
        gregorian_drift.to_decimal_string(2, Rounding::HalfEven).unwrap(),
        "3.10"
    );
    assert!(
        derived_drift.cmp_exact(&gregorian_drift) == std::cmp::Ordering::Less,
        "the derived rule must track the tropical year more closely"
    );
}

#[test]
fn divergence_3_the_epochs_are_unrelated() {
    // The derived calendar counts from its anchor — mean solar midnight at the
    // prime meridian in 2000 — so its year numbers bear no relation to a civil
    // year label. A reader who saw `0027` and thought "27 CE" would be wrong by
    // two millennia.
    let e = bodycal::by_id("earth-d").unwrap();
    let t = tt(2026, 7, 29);
    let f = e.fields(&t).unwrap();
    assert_eq!(f.year, 27, "derived year 27, not civil 2026");

    let civil = Gregorian
        .fields(&t, Scale::Tt, 0, Rounding::Trunc)
        .unwrap();
    assert_eq!(civil.year, 2026);
    assert_eq!(
        civil.year - f.year,
        1999,
        "the epochs differ by the anchor's own civil year, less one"
    );
}

#[test]
fn divergence_4_the_derived_calendar_has_no_months() {
    // §15.3: no month field unless a cycle was derived — and Earth's cycle is a
    // *phase* cycle from the Moon, not the twelve irregular blocks the civil
    // calendar declares. The two are not the same kind of thing.
    let e = bodycal::by_id("earth-d").unwrap();
    let f = e.fields(&tt(2026, 7, 29)).unwrap();
    let cycle = f.cycle.expect("earth-d names the Moon");
    assert_eq!(cycle.satellite, "moon");

    // The civil calendar's months are declared table data, and irregular.
    let lengths = Gregorian.tables().month_lengths;
    assert_eq!(lengths[0], 31);
    assert_eq!(lengths[1], 28);
    assert!(
        lengths.iter().collect::<std::collections::BTreeSet<_>>().len() > 1,
        "civil months are irregular; a derived cycle is uniform"
    );

    // The derived cycle count in a year is not twelve.
    let per_year = &e.cycles()[0].ratio;
    assert_eq!(
        per_year.to_decimal_string(6, Rounding::HalfEven).unwrap(),
        "12.368267",
        "a year is not a whole number of phase cycles, which is why the civil \
         calendar abandoned them"
    );
}

#[test]
fn the_divergence_is_recorded_not_asserted_away() {
    // The summary §21.3-8 asks for: the two calendars differ, in what way, and by
    // how much. Nothing here claims they agree.
    let e = bodycal::by_id("earth-d").unwrap();
    let t = tt(2026, 7, 29);

    let derived = e.render(&t).unwrap().to_string();
    let civil = Gregorian
        .render(&t, Scale::Tt, 0, Rounding::Trunc)
        .unwrap()
        .to_string();

    assert!(derived.starts_with("earth-d/1: "));
    assert!(civil.starts_with("earth-civil: "));
    assert_ne!(derived, civil);

    // Both are qualified, so neither can be mistaken for the other (§6.6).
    assert!(derived.contains("earth-d"));
    assert!(civil.contains("earth-civil"));
}
