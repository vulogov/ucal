//! The inverse of a derived calendar: a local date, back to absolute time.
//!
//! Earth's legacy calendars have gone both ways since 0.1.0 — `to-civil` and
//! `from-civil`. The fifteen derived ones went one way, while §15.4 says
//! *"Earth's entry has no special code path, no extra fields, and no
//! compile-time distinction from Mars's"*. That held inside `ucal-body` and not
//! at the surface a reader touches, which is a real dent in Rule K.5's claim
//! that Earth is an ordinary instance of the mechanism.
#![cfg(all(feature = "body", feature = "civil"))]

use ucal_core::backend::TickInt;
use ucal_core::{Instant, Ticks, UC1};

/// Walk an anchored calendar and check the inverse recovers every date.
///
/// **The property, over every anchored calendar and a spread of offsets.** For
/// any instant `t`, the window for `fields(t)` must contain `t` — which is the
/// same shape as N2's export round trip, and that one found a real defect on its
/// first run.
#[test]
fn the_inverse_recovers_every_instant_it_was_given() {
    let mut checked = 0usize;
    for (id, _, _) in ucal_body::calendar::registered() {
        let Ok(cal) = ucal_body::calendar::by_id(id) else {
            continue; // thirteen of fifteen have no anchor (Rule J.3)
        };
        let day = cal.body().solar_day().value_at_epoch().clone();
        for offset in [0u64, 1, 2, 40, 399, 400, 401, 4_000, 40_000, 100_000] {
            let span = day
                .mul(&ucal_core::num::Ratio::from_int(
                    <Ticks as TickInt>::from_u64(offset),
                ))
                .expect("in range")
                .floor();
            let t = Instant::<UC1>::from_ticks(
                cal.anchor()
                    .tick()
                    .ticks()
                    .clone()
                    .try_add(&span)
                    .expect("in range"),
            )
            .expect("inside the domain");

            let f = cal.fields(&t).expect("fields");
            let w = cal.instant_of(f.year, f.day, None).expect("the inverse");
            assert!(
                w.lo().ticks() <= t.ticks() && t.ticks() <= w.hi().ticks(),
                "{id}: +{offset} local days is year {} day {}, and the inverse \
                 window does not contain the instant it came from",
                f.year,
                f.day
            );
            checked += 1;
        }
    }
    // A floor: a loop over nothing would pass having compared nothing, which is
    // what the 1.6.0 audit found fourteen times.
    assert!(checked >= 20, "only {checked} instants were checked");
}

/// **A fraction narrows the window to the anchor's uncertainty alone.**
///
/// Without one the answer is a whole local day, because a local day *is* a span.
/// With one it is a moment inside that day, and all that remains is the anchor's
/// width — which is the other reason the answer is an interval, and does not go
/// away.
#[test]
fn a_fraction_narrows_the_window_to_the_anchor_s_own_width() {
    let cal = ucal_body::calendar::by_id("mars-d").expect("mars-d is anchored");
    let whole = cal.instant_of(82, 83, None).expect("a day");
    let moment = cal
        .instant_of(82, 83, Some(&ucal_core::num::Ratio::from_u64(0)))
        .expect("a moment");

    assert!(
        moment.width().ticks() < whole.width().ticks(),
        "a fraction did not narrow the window"
    );
    // And what remains is the anchor's uncertainty, not zero: Rule J.2 says it
    // propagates, so an exact local moment is still an interval in absolute time.
    assert!(
        !moment.width().ticks().is_zero_ticks(),
        "the anchor's uncertainty vanished"
    );
}

/// The window is taken **outward** to tick boundaries, never inward.
///
/// A local day boundary almost never lands on a tick boundary, so the endpoints
/// are rationals. Narrowing them to fit would be narrowing by assumption, which
/// GE-3 forbids; Rule R makes rendering the only place information may be lost,
/// and this is not rendering.
#[test]
fn consecutive_local_days_leave_no_gap() {
    let cal = ucal_body::calendar::by_id("earth-d").expect("earth-d is anchored");
    let a = cal.instant_of(27, 200, None).expect("a day");
    let b = cal.instant_of(27, 201, None).expect("the next day");
    assert!(
        b.lo().ticks() <= a.hi().ticks(),
        "consecutive local days left a gap no instant belongs to"
    );
}

/// Every refusal, and each one says something different.
#[test]
fn the_impossible_dates_are_refused() {
    let cal = ucal_body::calendar::by_id("mars-d").expect("mars-d is anchored");
    // There is no year 0: year 1 is the year that began at the anchor.
    assert_eq!(
        cal.instant_of(0, 1, None).expect_err("no year 0").code,
        ucal_core::Code::E0020
    );
    // Days are 1-based too.
    assert_eq!(
        cal.instant_of(1, 0, None).expect_err("no day 0").code,
        ucal_core::Code::E0018
    );
    // A day past the end of that year is refused rather than rolled forward,
    // because the length of a local year is set by the leap rule and varies.
    assert_eq!(
        cal.instant_of(1, 9_999, None).expect_err("no such day").code,
        ucal_core::Code::E0018
    );
    // A fraction is a position *through* a day: 1 is the next day.
    assert_eq!(
        cal.instant_of(1, 1, Some(&ucal_core::num::Ratio::one()))
            .expect_err("1 is not inside the day")
            .code,
        ucal_core::Code::E0018
    );
}

/// **An unanchored calendar has no inverse either**, and says so with the code
/// it already uses for the forward direction.
#[test]
fn an_unanchored_calendar_refuses_both_directions() {
    assert!(ucal::cmd_cal_from("titan-d", "1-1").is_err());
}

/// The command reads what `cal show` writes.
///
/// Both spellings: the padded form `cal show` renders, and the bare one a person
/// types. If they disagreed, the round trip would be a thing only a test could
/// run.
#[test]
fn the_command_accepts_the_form_cal_show_prints() {
    for local in ["82-83", "0082-083", "0082-083.442043"] {
        ucal::cmd_cal_from("mars-d", local)
            .unwrap_or_else(|e| panic!("`{local}` was refused: {e}"));
    }
    for bad in ["82", "82-", "-83", "82-83.", "82-83.9x", "eighty-two"] {
        assert!(
            ucal::cmd_cal_from("mars-d", bad).is_err(),
            "`{bad}` was accepted"
        );
    }
}
