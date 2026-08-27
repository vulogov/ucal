//! F6 — `ucal seq`, and the two things a generator has to get right.
//!
//! `seq` for time: instants at a tier interval, one decimal tick count per line.
//! It is the thing F2 makes worth having — `ucal seq A B --step T1 | ucal
//! to-civil -` is a pipeline this program could not express at all before 1.9.0.

use ucal_core::backend::TickInt;
use ucal_core::{Ticks, Tier};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";

fn tier(k: i8) -> Tier {
    Tier::new(k).expect("a valid tier")
}

fn plus(base: &str, tiers: i8, n: u64) -> String {
    let a = <Ticks as TickInt>::from_dec_str(base).expect("ticks");
    let step = tier(tiers).ticks();
    let mult = <Ticks as TickInt>::from_u64(n);
    a.try_add(&step.try_mul(&mult).expect("in domain"))
        .expect("in domain")
        .to_dec_string()
}

/// The walk is inclusive at both ends and steps by exactly one tier.
#[test]
fn it_walks_from_one_instant_to_the_other() {
    let to = plus(T, 1, 3);
    let lines = ucal::cmd_seq(T, &to, tier(1), 1_000).expect("three arcs");
    assert_eq!(lines.len(), 4, "inclusive at both ends: 0, 1, 2, 3");
    assert_eq!(lines[0], T);
    assert_eq!(lines[3], to);

    // And each gap is exactly one tier.
    let step = tier(1).ticks().to_dec_string();
    for pair in lines.windows(2) {
        let a = <Ticks as TickInt>::from_dec_str(&pair[0]).expect("ticks");
        let b = <Ticks as TickInt>::from_dec_str(&pair[1]).expect("ticks");
        assert_eq!(
            b.try_sub(&a).expect("forwards").to_dec_string(),
            step,
            "a gap is not one tier"
        );
    }
}

/// **The cap refuses rather than hangs**, and says what it would have printed.
///
/// A tier interval can be very small and a span very large: one tick across one
/// second is 1.8 x 10^43 lines. A program that starts printing and never stops
/// is worse than one that says why, so the count is computed first.
#[test]
fn an_impossible_walk_is_refused_with_its_own_number() {
    let to = plus(T, 1, 3);
    let e = ucal::cmd_seq(T, &to, tier(-12), 1_000_000).expect_err("far too many");
    assert_eq!(e.code, ucal_core::Code::E0018);
    let msg = e.to_string();
    assert!(msg.contains("limit is 1000000"), "{msg}");
    assert!(msg.contains("--step"), "the message names the way out: {msg}");
}

/// The cap is a limit, not a guess: one under it runs, one over it does not.
#[test]
fn the_cap_is_exactly_where_it_says() {
    let to = plus(T, 0, 10);
    assert!(ucal::cmd_seq(T, &to, tier(0), 11).is_ok(), "ten steps under a cap of eleven");
    assert!(
        ucal::cmd_seq(T, &to, tier(0), 10).is_err(),
        "ten steps against a cap of ten is refused, because the walk emits eleven lines"
    );
}

/// Backwards is a refusal, not an empty list.
///
/// An empty result would be indistinguishable from a span shorter than one step,
/// and the two mean different things: one is a caller's mistake and the other is
/// an answer.
#[test]
fn a_backwards_span_is_refused() {
    let to = plus(T, 1, 3);
    let e = ucal::cmd_seq(&to, T, tier(1), 1_000).expect_err("counts forwards");
    assert_eq!(e.code, ucal_core::Code::E0018);
    assert!(e.to_string().contains("forwards"), "{e}");
}

/// A span shorter than one step yields the start, and only the start.
#[test]
fn a_span_shorter_than_a_step_is_one_line() {
    let to = plus(T, 0, 1);
    let lines = ucal::cmd_seq(T, &to, tier(1), 1_000).expect("one beat apart");
    assert_eq!(lines, vec![T.to_string()], "the start, and nothing it can reach");
}

/// Every line `seq` prints is an instant every other command accepts.
///
/// The whole point is the pipeline, and a generator emitting something the
/// reader cannot parse would be a pipeline with a gap in the middle.
#[test]
fn every_line_parses_as_an_instant() {
    let to = plus(T, 1, 5);
    for line in ucal::cmd_seq(T, &to, tier(1), 1_000).expect("a walk") {
        ucal::parse_instant(&line)
            .unwrap_or_else(|e| panic!("`seq` emitted a line nothing can read: {line}: {e}"));
    }
}

// ---- G6: stepping by a body's own unit ---------------------------------

use ucal::Stride;

/// **The walk this project is actually for.**
///
/// F6 shipped `seq` stepping by tiers only, so it could not express *every
/// sunrise on Mars between these two instants* — the one thing F1 and F9 had
/// just made nameable. The proof is not that it produces lines: it is that each
/// line lands on the next Martian day at the *same fraction through it*, which
/// only an exact stride can do.
#[test]
fn a_step_of_one_martian_day_lands_on_consecutive_martian_days() {
    let start = T.to_string();
    let stride = Stride::calendar("mars-d").expect("mars has a solar day");
    let out = ucal::cmd_seq_by(&start, &far(4), &stride, 1000).expect("a walk");
    assert!(out.len() >= 4, "{out:?}");

    // Read the fields from the document rather than scraping the rendering:
    // a test that greps a layout breaks when the layout moves and says nothing
    // about the thing it was checking.
    let field = |doc: &ucal::emit::Doc, name: &str| -> String {
        let Some(ucal::emit::Value::Section(rows)) = doc.get("fields") else {
            panic!("no fields section");
        };
        rows.iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.rendered_text().trim().to_string())
            .unwrap_or_else(|| panic!("no field `{name}`"))
    };

    let mut seen: Vec<(i64, String)> = Vec::new();
    for line in &out {
        let doc = ucal::cmd_cal_show("mars-d", line).expect("a date");
        let day: i64 = field(&doc, "day").parse().expect("a day number");
        seen.push((day, field(&doc, "day_fraction")));
    }
    for w in seen.windows(2) {
        assert_eq!(
            w[1].0,
            w[0].0 + 1,
            "not consecutive Martian days: {seen:?}"
        );
        // The same fraction through the day, every time. Only an exact stride
        // does that; a truncated one drifts through the day and the walk is
        // wrong by the accumulated remainder.
        assert_eq!(
            w[1].1, w[0].1,
            "the stride is not exactly one Martian day: {seen:?}"
        );
    }
}

/// A body's *year* steps too.
#[test]
fn a_calendar_year_is_a_stride() {
    assert!(Stride::calendar("mars-d-year").is_ok());
    assert!(Stride::calendar("earth-d-year").is_ok());
}

/// **A derived solar day is refused, not truncated.**
///
/// Six of the shipped calendars compute their solar day from two published
/// figures, and the result is a rational that is almost never a whole number of
/// ticks. Truncating would make every step short by a little and the walk wrong
/// by the accumulated remainder — which is precisely the error a tier cannot
/// make, since `5^(60+5k)` is an integer by construction.
#[test]
fn a_stride_that_is_not_a_whole_number_of_ticks_is_refused() {
    let e = Stride::calendar("europa-d").expect_err("europa's solar day is derived");
    assert_eq!(e.code, ucal_core::Code::E0043);
    let msg = e.to_string();
    assert!(msg.contains("not a whole number"), "{msg}");
    assert!(msg.contains("accumulated remainder"), "{msg}");
}

/// An unanchored calendar can still be a stride.
///
/// Thirteen of the fifteen have no anchor, and `calendar::by_id` refuses those
/// because it builds a calendar, which needs a phase. A stride is a *duration*
/// and needs none, so looking a body up that way would have refused it for a
/// reason that does not apply.
#[test]
fn an_unanchored_calendar_is_still_a_stride() {
    // `jupiter-d` has no anchor and a measured solar day.
    let s = Stride::calendar("jupiter-d");
    assert!(s.is_ok(), "jupiter-d should be usable as a stride");
    // And a bare body name works too.
    assert!(Stride::calendar("mars").is_ok());
}

/// A name that is neither a tier nor a calendar.
#[test]
fn an_unknown_stride_is_refused() {
    let e = Stride::calendar("vulcan-d").expect_err("no such body");
    assert_eq!(e.code, ucal_core::Code::E0016);
}

/// Four Martian days past `T`, as a decimal tick count.
fn far(days: u64) -> String {
    let stride = Stride::calendar("mars-d").expect("mars has a solar day");
    let mut t = <ucal_core::Ticks as TickInt>::from_dec_str(T).expect("a tick count");
    for _ in 0..days {
        t = t.try_add(&stride.ticks()).expect("in range");
    }
    t.to_dec_string()
}
