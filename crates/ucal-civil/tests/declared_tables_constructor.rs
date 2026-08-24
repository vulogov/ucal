//! A1 — `DeclaredTables::new`, and the four obligations a struct literal cannot
//! state.
//!
//! §8.6 keeps legacy calendars "for interoperation". Until this constructor
//! existed that admitted no calendar this crate did not already ship: the type
//! is `#[non_exhaustive]` with no way to build one, so a downstream
//! `LegacyCalendar` had to return `Gregorian.tables()` and call it its own.

use ucal_civil::legacy::{DeclaredLeapRule, DeclaredTables, Discontinuity, LegacyCalendar};
use ucal_civil::Gregorian;
use ucal_core::Code;

const GREGORIAN_MONTHS: [u8; 12] = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

fn rule() -> DeclaredLeapRule {
    DeclaredLeapRule::new(97, 400, false).expect("400 years is a cycle")
}

fn arbitrary() -> &'static [&'static str] {
    &["the month lengths correspond to no period of any body"]
}

/// **The check that matters.** The constructor reproduces a shipped calendar.
///
/// If it did not, it would be a second and laxer way of declaring tables, and
/// every legacy calendar built through it would differ from the two that ship
/// in some way nobody had chosen.
#[test]
fn the_constructor_reproduces_the_shipped_gregorian_tables() {
    let built = DeclaredTables::new(GREGORIAN_MONTHS, 7, rule(), None, arbitrary())
        .expect("the shipped month lengths are a common year");
    let shipped = Gregorian.tables();
    assert_eq!(built.month_lengths, shipped.month_lengths);
    assert_eq!(built.week_length, shipped.week_length);
    assert_eq!(built.leap_rule, shipped.leap_rule);
}

/// Months that do not make a year are refused.
///
/// Every `fields`/`instant` round-trip in the module assumes 365, and a table
/// that broke it would produce dates rather than an error.
#[test]
fn months_that_do_not_sum_to_a_year_are_refused() {
    let mut short = GREGORIAN_MONTHS;
    short[0] = 30;
    let e = DeclaredTables::new(short, 7, rule(), None, arbitrary())
        .expect_err("364 days is not a common year");
    assert_eq!(e.code, Code::E0018);
    assert!(e.to_string().contains("365"), "{e}");
}

/// A week with no days, and a cycle with no years, are refused.
#[test]
fn a_zero_week_and_a_zero_cycle_are_refused() {
    let e = DeclaredTables::new(GREGORIAN_MONTHS, 0, rule(), None, arbitrary())
        .expect_err("weekday arithmetic would divide by zero");
    assert_eq!(e.code, Code::E0018);

    // The rule refuses its own zero cycle before `DeclaredTables` ever sees it,
    // which is where the check belongs: a rule with no cycle is not a rule
    // whether or not anyone puts it in a table.
    let e = DeclaredLeapRule::new(97, 0, false).expect_err("a rule needs a cycle");
    assert_eq!(e.code, Code::E0018);
}

/// **§8.6's actual requirement**: a legacy calendar states what is arbitrary.
///
/// This is the reason legacy calendars are tolerable at all in a project whose
/// every other calendar is derived from a body's periods. One that declares
/// nothing arbitrary is claiming to be derived, and it is not — so an empty
/// list is refused rather than defaulted to silence.
#[test]
fn a_calendar_that_declares_nothing_arbitrary_is_refused() {
    let e = DeclaredTables::new(GREGORIAN_MONTHS, 7, rule(), None, &[])
        .expect_err("§8.6 requires the arbitrariness to be stated");
    assert_eq!(e.code, Code::E0018);
    assert!(e.to_string().contains("derived"), "{e}");
}

/// Both shipped calendars satisfy what the constructor requires.
///
/// The rule is only worth having if the data it exists to protect already obeys
/// it. If a shipped table could not be rebuilt through this path, the check
/// would be describing a calendar nobody has.
#[test]
fn both_shipped_calendars_would_pass_their_own_constructor() {
    for tables in [Gregorian.tables(), ucal_civil::Julian.tables()] {
        assert_eq!(
            tables.month_lengths.iter().map(|d| u32::from(*d)).sum::<u32>(),
            365
        );
        assert!(tables.week_length > 0);
        assert!(tables.leap_rule.denominator > 0);
        assert!(
            !tables.arbitrary.is_empty(),
            "a shipped legacy calendar declares what is arbitrary about it"
        );
    }
}

/// **The finding that A1 nearly shipped without.**
///
/// `DeclaredTables::new` takes a `DeclaredLeapRule` and an `Option<Discontinuity>`,
/// and both were `#[non_exhaustive]` with no constructor of their own. The
/// constructor compiled, read well, and was useless to the one caller it existed
/// for: an outsider could not build its arguments.
///
/// An extension point is only as usable as the least constructible type in its
/// signature. This test is the whole chain, exercised the way a downstream crate
/// would meet it.
#[test]
fn every_type_in_the_signature_can_be_built() {
    let rule = DeclaredLeapRule::new(1, 4, true).expect("a Julian cycle");
    let reform = Discontinuity::new(
        "a reform this crate has never heard of",
        (1700, 2, 18),
        (1700, 3, 1),
        11,
    )
    .expect("eleven days is a skip");
    let tables = DeclaredTables::new(
        GREGORIAN_MONTHS,
        7,
        rule,
        Some(reform),
        &["everything about this calendar is arbitrary; that is the point"],
    )
    .expect("a downstream calendar, built from parts");
    assert_eq!(tables.leap_rule.denominator, 4);
    assert_eq!(
        tables.discontinuity.map(|d| d.days_skipped),
        Some(11)
    );
}

/// The two new constructors refuse what they say they refuse.
#[test]
fn a_discontinuity_needs_a_description_and_a_skip() {
    for (desc, skipped) in [("", 10u8), ("   ", 10), ("a real reform", 0)] {
        let e = Discontinuity::new(desc, (1, 1, 1), (1, 1, 2), skipped)
            .expect_err("refused");
        assert_eq!(e.code, Code::E0018);
    }
}
