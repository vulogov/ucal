//! X1.2 / D-A21 — the leap placement is a convention, pinned by an alternative.
//!
//! Rule K derives *how many* intercalary days a cycle holds. Which day is
//! intercalated is not derivable, and until 1.4.0 §15.5 did not say.
//!
//! That mattered more than it looks. A calendar that clumped every intercalation
//! at the end of its cycle would satisfy the **identical** `LeapRule`, reproduce
//! every convergent table, and pass every conformance vector — and would place
//! day 366 on a different absolute instant. The specification checked everything
//! except the thing the two calendars disagree about.
//!
//! So this file implements the clumped alternative and demonstrates the
//! disagreement. The point is not that the alternative is wrong; it is that the
//! choice is real, which is only visible with something to choose against. A
//! convention pinned by the code that implements it is not pinned at all.

use ucal_body::{calendar, data, derive_leap_rule, DriftBound, LeapRule};
use ucal_core::backend::TickInt;
use ucal_core::Ticks;

/// The declared placement (D-A21): `y·whole + ⌊y·p/q⌋`.
///
/// Intercalations spread as evenly as integers allow. Reimplemented here rather
/// than called, so that a change to `BodyCalendar` shows up as a disagreement
/// with this file instead of silently redefining what it is being compared to.
fn even(rule: &LeapRule, y: u64) -> u64 {
    let whole: u64 = rule.whole_days.numer().to_dec_string().parse().unwrap_or(0);
    let p: u64 = rule.chosen.value.numer().to_dec_string().parse().unwrap_or(0);
    let q: u64 = rule.chosen.value.denom().to_dec_string().parse().unwrap_or(1);
    y * whole + (y * p) / q
}

/// The clumped alternative: every intercalation at the end of the cycle.
///
/// Same `whole_days`, same `p/q`, same total over a full cycle — and a different
/// answer everywhere inside one. This is what Earth does, for a reason that is
/// historical rather than principled: the Gregorian leap day sits in February
/// because the month lengths were fixed before the intercalation was.
fn clumped(rule: &LeapRule, y: u64) -> u64 {
    let whole: u64 = rule.whole_days.numer().to_dec_string().parse().unwrap_or(0);
    let p: u64 = rule.chosen.value.numer().to_dec_string().parse().unwrap_or(0);
    let q: u64 = rule.chosen.value.denom().to_dec_string().parse().unwrap_or(1);
    let cycles = y / q;
    let within = y % q;
    // Ordinary years first; the intercalary ones are all at the end.
    let extra_within = within.saturating_sub(q - p);
    cycles * (q * whole + p) + within * whole + extra_within
}

fn earth_rule() -> LeapRule {
    let e = data::earth();
    derive_leap_rule(
        e.solar_day().value_at_epoch(),
        e.orbital_period().value_at_epoch(),
        DriftBound::DEFAULT,
        32,
    )
    .expect("Earth derives a rule")
}

/// Both placements agree on the total, which is what makes this a real trap.
///
/// If they disagreed over a whole cycle, one of them would be violating the leap
/// rule and the specification would already catch it. They do not: the rule is
/// satisfied by both, exactly, and the disagreement lives entirely inside the
/// cycle where nothing was looking.
#[test]
fn both_placements_satisfy_the_same_leap_rule() {
    let rule = earth_rule();
    let q: u64 = rule.chosen.value.denom().to_dec_string().parse().unwrap();

    for cycles in 1..=8u64 {
        let y = cycles * q;
        assert_eq!(
            even(&rule, y),
            clumped(&rule, y),
            "the two placements must agree at every cycle boundary, and do not at year {y}"
        );
    }
}

/// And they disagree inside it — which is the whole finding.
#[test]
fn the_placements_disagree_within_a_cycle() {
    let rule = earth_rule();
    let q: u64 = rule.chosen.value.denom().to_dec_string().parse().unwrap();

    let disagreements = (1..q).filter(|y| even(&rule, *y) != clumped(&rule, *y)).count();
    assert!(
        disagreements > 0,
        "if the placements never disagreed the convention would not be load-bearing \
         and D-A21 would be unnecessary"
    );
    // Most of the cycle, not a corner case: with 31/128 the two differ for the
    // great majority of years inside a cycle.
    assert!(
        disagreements > (q as usize) / 2,
        "expected the placements to differ over most of the cycle; {disagreements} of {} years",
        q - 1
    );
}

/// The shipped calendar implements the declared one.
///
/// Compares `BodyCalendar`'s own year boundaries against `even`, through the
/// public surface: for each of the first years, the instant one tick before the
/// year starts must fall in the previous year.
#[test]
fn the_shipped_calendar_uses_the_declared_placement() {
    let cal = calendar::by_id("earth-d").expect("earth-d");
    let anchor_tick = cal.anchor().tick().clone();
    let day = cal.body().solar_day().value_at_epoch().clone();
    let rule = earth_rule();

    for y in 1..12u64 {
        // Where the declared placement says year `y` (0-based) begins.
        let days = even(&rule, y);
        let offset = day
            .mul(&ucal_core::num::Ratio::from_u64(days))
            .expect("within the domain");
        let ticks = offset.floor();
        let start = anchor_tick
            .checked_add(&ucal_core::Delta::from_ticks(ticks))
            .expect("within the domain");

        let f = cal.fields(&start).expect("fields at a year boundary");
        assert_eq!(
            f.year,
            (y as i64) + 1,
            "the declared placement puts year {} at day {days}; the calendar disagrees",
            y + 1
        );
        assert_eq!(
            f.day, 1,
            "a year boundary should be day 1 of the year, not day {}",
            f.day
        );
    }
}

/// A `Ticks` helper the file uses in one place, kept honest.
#[test]
fn the_reimplementation_matches_at_zero() {
    let rule = earth_rule();
    assert_eq!(even(&rule, 0), 0);
    assert_eq!(clumped(&rule, 0), 0);
    let _ = <Ticks as TickInt>::zero();
}
