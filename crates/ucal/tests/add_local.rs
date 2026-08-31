//! Moving by a **date** rather than by a duration.
//!
//! `ucal add --step <id>-year` adds a body's mean orbital period. That is a real
//! interval and a perfectly good thing to add, and it is not *next year, same
//! date*: a local year is not a constant span, because the leap rule packs whole
//! days into years and the lengths differ by one.
//!
//! Until `--in`, adding a mean was the only thing this program could do, so the
//! one operation every calendar exists for — the same day of the next year — was
//! the one it could not perform, on twenty-odd calendars it derives itself.
//!
//! The measurement that motivates the whole feature is
//! [`a_local_year_is_not_the_mean_year`], and it is written so that
//! reimplementing `add_years` as `n x orbital_period` fails it.
#![cfg(all(feature = "body", feature = "civil"))]

use ucal_core::backend::TickInt;
use ucal_core::num::Ratio;
use ucal_core::{Code, Instant, Ticks, UC1};
use ucal_body::calendar::BodyCalendar;

/// Midday on a local date, as an instant.
///
/// The window's low end: `instant_of` widens by the anchor's uncertainty, which
/// is far narrower than half a local day for every anchored calendar, so this
/// lands inside the day it names. The tests below assert that rather than
/// assuming it.
fn midday(cal: &BodyCalendar, year: i64, day: u32) -> Instant<UC1> {
    let half = Ratio::new(
        <Ticks as TickInt>::from_u64(1),
        <Ticks as TickInt>::from_u64(2),
    )
    .expect("1/2");
    let w = cal
        .instant_of(year, day, Some(&half))
        .expect("a local date that exists");
    w.lo().clone()
}

/// **The measurement.** A local year is not the mean year, and the difference
/// shows up in three.
///
/// From `earth-d` year 2000 day 100, adding the body's mean orbital period lands
/// on day 100, day 100, then **day 101**. Adding a local year lands on day 100
/// every time, at the same position within it.
///
/// This is the test that fails if `add_years` is ever "simplified" to a
/// multiplication by the orbital period — which is why it goes through
/// `Stride::calendar`, the code path `--step earth-d-year` actually runs, rather
/// than a mean computed here to match.
#[test]
fn a_local_year_is_not_the_mean_year() {
    let cal = ucal_body::calendar::by_id("earth-d").expect("anchored");
    let start = midday(&cal, 2000, 100);
    let mean = ucal::Stride::calendar("earth-d-year").expect("the mean year");

    let mut mean_days = Vec::new();
    let mut local_days = Vec::new();
    for n in 1..=4u64 {
        let span = <Ticks as TickInt>::from_u64(n)
            .try_mul(mean.ticks())
            .expect("in range");
        let by_mean = Instant::<UC1>::from_ticks(
            start.ticks().clone().try_add(&span).expect("in range"),
        )
        .expect("in the domain");
        mean_days.push(cal.fields(&by_mean).expect("fields").day);

        let step = cal.add_years(&start, n as i64).expect("the date exists");
        local_days.push(cal.fields(&step.to).expect("fields").day);
    }

    // The recorded numbers, not merely "they differ somewhere".
    assert_eq!(mean_days, vec![100, 100, 101, 100], "the mean year drifts");
    assert_eq!(local_days, vec![100, 100, 100, 100], "a local year does not");
}

/// The day of the year and the position within it survive the move exactly.
///
/// Over every anchored calendar, a spread of starting dates and both
/// directions. The fraction is compared with `cmp_exact` and not rendered:
/// carrying it across unchanged is the property, and a comparison that rounded
/// would not be able to see it fail.
#[test]
fn the_day_and_the_fraction_are_carried_across_unchanged() {
    let mut checked = 0usize;
    let mut refused_at_the_seam = 0usize;

    for (id, _, _) in ucal_body::calendar::registered() {
        let Ok(cal) = ucal_body::calendar::by_id(id) else {
            continue; // thirteen of fifteen have no anchor (Rule J.3)
        };
        // `None` means the last day of that year — the only starting point from
        // which the seam can be reached at all. Without it this test exercises
        // days every year has, and a `add_years` that clamped instead of
        // refusing would pass it: injecting that clamp is how the case was
        // found missing.
        for (year, day) in [
            (40i64, Some(1u32)),
            (100, Some(200)),
            (400, Some(300)),
            (1000, Some(1)),
            (100, None),
            (401, None),
        ] {
            let day = day.unwrap_or_else(|| cal.year_length(year).expect("a length"));
            let t = midday(&cal, year, day);
            let before = cal.fields(&t).expect("fields");
            assert_eq!(before.day, day, "{id}: midday landed outside its own day");

            for n in [-30i64, -7, -1, 1, 3, 7, 30] {
                match cal.add_years(&t, n) {
                    Ok(step) => {
                        let after = cal.fields(&step.to).expect("fields");
                        assert_eq!(after.year, before.year + n, "{id} {n}");
                        assert_eq!(after.day, before.day, "{id} {n}: the day moved");
                        assert_eq!(
                            after.day_fraction.cmp_exact(&before.day_fraction),
                            core::cmp::Ordering::Equal,
                            "{id} {n}: the fraction through the day moved"
                        );
                        checked += 1;
                    }
                    // The only tolerated refusal is the seam, and only for a day
                    // the destination year genuinely lacks. Any other code is a
                    // failure rather than a skip: a test that skips on an
                    // unexpected error passes by not looking.
                    Err(e) => {
                        assert_eq!(e.code, Code::E0018, "{id} {n}: {e}");
                        let to = before.year + n;
                        assert!(
                            day > cal.year_length(to).expect("a length"),
                            "{id} {n}: refused a day that year does have"
                        );
                        refused_at_the_seam += 1;
                    }
                }
            }
        }
    }

    assert!(checked >= 40, "only {checked} moves were checked");
    // The seam must actually be reached. Both anchored calendars have uneven
    // year lengths, so starting on the last day of a year and moving one year
    // has to be refused somewhere — and a run where it never was would mean
    // this test had stopped covering the case it was extended to cover.
    assert!(
        refused_at_the_seam > 0,
        "the seam was never reached, so nothing here checks that it refuses"
    );
}

/// Moving out and back returns the same tick, not merely the same date.
///
/// The round trip is exact because the step is a whole number of local days
/// converted to ticks once, and nothing along the way is re-derived from the
/// anchor. A `Window` round trip could only have promised containment.
#[test]
fn moving_back_returns_the_instant_itself() {
    for (id, _, _) in ucal_body::calendar::registered() {
        let Ok(cal) = ucal_body::calendar::by_id(id) else {
            continue;
        };
        let t = midday(&cal, 100, 300);
        for n in [1i64, 2, 37, 99] {
            let out = cal.add_years(&t, n).expect("the date exists");
            let back = cal.add_years(&out.to, -n).expect("and back");
            assert_eq!(
                back.to.ticks().to_dec_string(),
                t.ticks().to_dec_string(),
                "{id}: {n} years out and back is not the same instant"
            );
        }
    }
}

/// A day the destination year does not have is refused, not clamped.
///
/// `earth-d` year 2003 has 366 local days and 2004 has 365. Clamping to day 365
/// or rolling into 2005 day 1 are both answers; neither is *the* answer, and a
/// date that quietly moves is what Rule R refuses at rendering time.
#[test]
fn a_day_the_destination_year_lacks_is_refused() {
    let cal = ucal_body::calendar::by_id("earth-d").expect("anchored");
    assert_eq!(cal.year_length(2003).expect("length"), 366);
    assert_eq!(cal.year_length(2004).expect("length"), 365);

    let t = midday(&cal, 2003, 366);
    let e = cal.add_years(&t, 1).expect_err("2004 has no day 366");
    assert_eq!(e.code, Code::E0018);

    // And the year after that does, so the refusal is about the destination and
    // not about day 366 being unreachable in general.
    assert_eq!(cal.year_length(2007).expect("length"), 366);
    let ok = cal.add_years(&t, 4).expect("2007 has day 366");
    assert_eq!(cal.fields(&ok.to).expect("fields").day, 366);
}

/// `year_length` and the inverse agree about where a year ends.
///
/// Two computations of one fact — the length `add_years` checks against, and the
/// bound `instant_of` refuses past. They come from the same helper by
/// construction; this is the test that says so, and that would fail if either
/// grew its own copy.
#[test]
fn the_length_of_a_year_is_the_last_day_the_inverse_accepts() {
    for (id, _, _) in ucal_body::calendar::registered() {
        let Ok(cal) = ucal_body::calendar::by_id(id) else {
            continue;
        };
        for year in [1i64, 2, 5, 99, 400, 1001] {
            let len = cal.year_length(year).expect("a length");
            assert!(cal.instant_of(year, len, None).is_ok(), "{id} {year}: {len}");
            let past = cal
                .instant_of(year, len + 1, None)
                .expect_err("one day past the end");
            assert_eq!(past.code, Code::E0018, "{id} {year}");
        }
    }
}

/// Backwards past the anchor is `UCAL-E0020`, not year zero.
#[test]
fn moving_before_local_year_one_is_refused() {
    let cal = ucal_body::calendar::by_id("mars-d").expect("anchored");
    let t = midday(&cal, 100, 300);
    let e = cal.add_years(&t, -100).expect_err("there is no year 0");
    assert_eq!(e.code, Code::E0020);
    // Year 1 itself is reachable: the bound is exclusive of 0, not of 1.
    assert!(cal.add_years(&t, -99).is_ok());
}
