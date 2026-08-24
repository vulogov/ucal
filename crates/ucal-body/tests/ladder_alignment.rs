//! W4 step 1 — where a body's own units land on the universal ladder.
//!
//! [`W4-two-ladders.md`] proposes a view with the universal grid on one side and
//! a body's local calendar on the other. Before any of that is built, the
//! question underneath it can be answered on its own: **does placing a body's
//! units on the universal ladder show anything a reader could not get from two
//! separate commands?**
//!
//! This is that question and nothing else. No command, no rendering, no public
//! API — a probe, of the kind GE-U4 learned to run before building the
//! expensive version. If the placements are unremarkable, the view has nothing
//! to display and the proposal can be closed at the cost of one afternoon.
//!
//! # The arithmetic
//!
//! Every local unit has a length in ticks, exactly, as a `Ratio`. Every rung is
//! `5^(60 + 5k)`, exactly, as an integer. So a unit's placement is:
//!
//! - the largest rung `r` with `r <= length`;
//! - the residual `length / r`, which lies in `[1, 3125)` and says how far above
//!   that rung the unit sits.
//!
//! All of it is exact rational arithmetic and none of it renders a decimal, so
//! nothing here needs a rounding mode (Rule R) and nothing touches a float
//! (Rule E). The assertions are bounds on exact `Ratio`s.
//!
//! Run it with output to read the table:
//!
//! ```text
//! cargo test -p ucal-body --test ladder_alignment -- --nocapture
//! ```
//!
//! [`W4-two-ladders.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/W4-two-ladders.md

use ucal_body::{calendar, data};
use ucal_core::backend::TickInt;
use ucal_core::num::Ratio;
use ucal_core::{Ticks, Tier};

/// Where one duration sits on the universal ladder.
struct Placement {
    tier: Tier,
    /// `length / tier`, in `[1, 3125)`.
    above: Ratio,
}

/// Place a duration, given in ticks.
///
/// `None` for a duration below one tick, which no body's unit is: the smallest
/// here is Jupiter's day, some thirty-eight orders of magnitude above it.
fn place(length: &Ratio) -> Option<Placement> {
    let mut best: Option<Tier> = None;
    for tier in Tier::all_descending() {
        let r = Ratio::from_int(tier.ticks());
        if r.cmp_exact(length) != core::cmp::Ordering::Greater {
            best = Some(tier);
            break;
        }
    }
    let tier = best?;
    let above = length.div(&Ratio::from_int(tier.ticks())).ok()?;
    Some(Placement { tier, above })
}

/// A tier's name, or its index where the grid has none.
fn label(t: Tier) -> String {
    match ucal_core::tier::name_of(t) {
        Some(n) => format!("{t} {}", n.key()),
        None => t.to_string(),
    }
}

/// Every unit every shipped body has, placed.
fn table() -> Vec<(String, &'static str, Placement)> {
    let mut out = Vec::new();
    for (id, body, _) in calendar::registered() {
        let day = body.solar_day().value_at_epoch().clone();
        let year = body.orbital_period().value_at_epoch().clone();
        for (what, len) in [("solar day", day), ("year", year)] {
            if let Some(p) = place(&len) {
                out.push((id.to_string(), what, p));
            }
        }
    }
    // The one grouping cycle any shipped calendar declares.
    if let Ok(earth) = calendar::by_id("earth-d") {
        if let Some(c) = earth.cycles().first() {
            if let Some(p) = place(&c.synodic_period) {
                out.push(("earth-d".to_string(), "synodic month", p));
            }
        }
    }
    out
}

/// Print the table. Not an assertion — the point of a probe is to be read.
#[test]
fn the_placements_are_worth_looking_at() {
    println!();
    println!("  {:32}  {:10}  {:>10}", "body / unit", "rung", "above the rung");
    println!("  {}  {}  {}", "─".repeat(32), "─".repeat(10), "─".repeat(10));
    for (id, what, p) in table() {
        // ucal-lint-allow-begin(rounding-is-declared): one decimal, half-even,
        // for a line a person reads. There is no outer caller whose mode could
        // be honoured, and nothing depends on this value — every assertion in
        // this file compares exact `Ratio`s instead, which is why the rendering
        // can be fixed here without hiding anything.
        let above = p
            .above
            .to_decimal_string(1, ucal_core::Rounding::HalfEven)
            .unwrap_or_default();
        // ucal-lint-allow-end(rounding-is-declared)
        println!("  {:32}  {:10}  {:>10}", format!("{id} {what}"), label(p.tier), above);
    }
    println!();
    assert!(!table().is_empty());
}

/// The arithmetic agrees with the specification's own published figure.
///
/// §4.3's bridge table states `1 d = 591.25 arc`. If this placement machinery
/// puts Earth's day anywhere else, it is wrong — and it is worth checking
/// against a number nobody involved in writing this test chose.
#[test]
fn earths_day_lands_where_the_bridge_table_says() {
    let earth = data::earth();
    let p = place(earth.solar_day().value_at_epoch()).expect("Earth's day is placeable");
    assert_eq!(label(p.tier), "T1 arc");

    // 591.25 to the precision §4.3 prints. Bounds on exact ratios rather than a
    // rendered comparison, so no rounding mode is involved.
    let lo = Ratio::from_u64(591);
    let hi = Ratio::from_u64(592);
    assert_eq!(p.above.cmp_exact(&lo), core::cmp::Ordering::Greater);
    assert_eq!(p.above.cmp_exact(&hi), core::cmp::Ordering::Less);
}

/// **The finding.** Two unrelated bodies put their day on the same rung.
///
/// Earth's day is 591 arcs and Mars's sol is 607 — 2.7% apart, on a ladder whose
/// steps are a factor of 3125. Nothing arranged that: the numbers come from
/// separate measurements of separate planets, and the ladder was built from
/// powers of five with no knowledge of either.
///
/// This is what a two-column view would show and two separate command outputs
/// do not, because seeing it requires both bodies against the *same* grid at
/// once. It is the strongest argument for the proposal that exists.
#[test]
fn earth_and_mars_land_on_the_same_rung() {
    let e = place(data::earth().solar_day().value_at_epoch()).expect("Earth");
    let m = place(data::mars().solar_day().value_at_epoch()).expect("Mars");
    assert_eq!(label(e.tier), label(m.tier), "the two days are on different rungs");
    assert_eq!(label(e.tier), "T1 arc");

    // And close on it: the ratio between them is under 1.1, where a full step is
    // 3125. Stated as a bound rather than a figure so the test says what it
    // means — these are *near* each other, on a scale where they need not be.
    let ratio = m.above.div(&e.above).expect("non-zero");
    assert_eq!(
        ratio.cmp_exact(&Ratio::from_u64(2)),
        core::cmp::Ordering::Less,
        "Mars's sol and Earth's day should be within a factor of two on the rung"
    );
}

/// The years land together too, one rung up.
#[test]
fn the_years_share_a_rung_as_well() {
    let e = place(data::earth().orbital_period().value_at_epoch()).expect("Earth");
    let m = place(data::mars().orbital_period().value_at_epoch()).expect("Mars");
    assert_eq!(label(e.tier), "T2 sweep");
    assert_eq!(label(m.tier), "T2 sweep");
}

/// **The other finding, and the one that decides the feature.**
///
/// Every unit every shipped body has lands on `T1` or above. There is nothing
/// below `arc` — no local hour, no local minute, no local second — because a
/// derived calendar has a year, a day and sometimes a cycle, and below the day
/// nothing is a period of the body at all. Thirteen rungs, `beat` down to
/// `tick`, hold nothing for any body.
///
/// So a two-column view has forty-five rungs on the left and, for the busiest
/// body, three entries on the right, all crowded into two adjacent rungs near
/// the middle. The lower two-thirds is empty and will stay empty.
///
/// That is not a rendering problem to be solved. It is the model, and the
/// proposal's kill criterion says the view must show it rather than fill it.
#[test]
fn nothing_a_body_has_lands_below_the_arc() {
    let mut lowest: Option<(String, &'static str, Tier)> = None;
    for (id, what, p) in table() {
        match &lowest {
            Some((_, _, t)) if *t <= p.tier => {}
            _ => lowest = Some((id, what, p.tier)),
        }
    }
    let (id, what, tier) = lowest.expect("some unit");
    assert_eq!(
        label(tier),
        "T1 arc",
        "the lowest local unit is {id} {what}, at {}",
        label(tier)
    );

    // Thirteen rungs exist below T1 — T0 `beat` down to T-12 `tick` — and no
    // body has a unit on any of them. Stated as a count so a future body that
    // *did* would fail here and be noticed rather than silently filling one in.
    let below = Tier::all_descending().filter(|t| *t < tier).count();
    assert_eq!(
        below, 13,
        "thirteen rungs below the lowest local unit, all of them empty"
    );
}

/// Two independently derived quantities agree, which nothing arranged.
///
/// `earth-d`'s grouping cycle comes from `derive_cycles`, which takes Earth's
/// orbital period and the Moon's and expands a continued fraction. `luna-d`'s
/// solar day comes from `data::luna`, a published synodic period read off a
/// fact sheet. They are the same physical quantity reached by different routes
/// through different data, and they land on the same rung at the same residual.
///
/// Found by running this probe rather than by looking for it, which is the
/// reason to place things on a common scale at all.
#[test]
fn the_moons_day_is_earths_month_by_two_routes() {
    let earth = calendar::by_id("earth-d").expect("earth-d");
    let cycle = earth.cycles().first().expect("Earth names the Moon");
    let from_cycle = place(&cycle.synodic_period).expect("placeable");
    let from_body = place(data::luna().solar_day().value_at_epoch()).expect("placeable");

    assert_eq!(label(from_cycle.tier), label(from_body.tier));

    // Within a per cent of each other. Not equal: one is a synodic period
    // derived from two orbital periods, the other a published figure with its
    // own rounding, and Rule Y's whole position is that those need not coincide
    // exactly. That they coincide at all is the check.
    let ratio = from_cycle.above.div(&from_body.above).expect("non-zero");
    let lo = Ratio::new(<Ticks as TickInt>::from_u64(99), <Ticks as TickInt>::from_u64(100)).unwrap();
    let hi = Ratio::new(<Ticks as TickInt>::from_u64(101), <Ticks as TickInt>::from_u64(100)).unwrap();
    assert_eq!(ratio.cmp_exact(&lo), core::cmp::Ordering::Greater);
    assert_eq!(ratio.cmp_exact(&hi), core::cmp::Ordering::Less);
}

/// **The finding that decides the proposal.**
///
/// Every unit of every shipped body — seven calendars, fifteen units, days and
/// years and one month — lands on `T1` or `T2`. Two adjacent rungs out of
/// forty-five.
///
/// The right-hand column of a two-ladder view is therefore not sparse. It is
/// *degenerate*: forty-three rungs empty, everything crowded into two, and the
/// alignment the view exists to display is a pair of tick marks in the middle
/// of a very long ruler.
///
/// That is a fact about the model and not about a rendering, and no amount of
/// zoom improves it — zooming in far enough to separate `T1` from `T2` shows
/// two rows and no ladder.
/// **Three rungs since 1.9.0, and the third arrived by adding a body.**
///
/// W4 step 1 found two — `T1` and `T2` — across the twelve calendars that
/// shipped then, all of them inside Saturn's orbit or on it. F9 added Uranus,
/// Neptune and Pluto, and a Plutonian year is 248 Earth years against a `T3`
/// span of 45, so the outer solar system reaches a rung the inner one never
/// touches.
///
/// The finding is not weakened by this; it is measured better. **The whole solar
/// system's days and years occupy three rungs out of forty-five**, and the two
/// extra bodies moved it by one. That is still a ladder whose steps are a factor
/// of 3125 spanning a range its contents do not, which was the point.
#[test]
fn every_local_unit_lands_on_one_of_three_rungs() {
    let mut rungs: Vec<String> = table().into_iter().map(|(_, _, p)| label(p.tier)).collect();
    rungs.sort();
    rungs.dedup();
    assert_eq!(
        rungs,
        vec![
            "T1 arc".to_string(),
            "T2 sweep".to_string(),
            "T3 span".to_string()
        ],
        "every local unit of every shipped body should sit on one of three rungs"
    );
    // Forty-five rungs exist. Forty-two of them hold nothing, for every body.
    assert_eq!(Tier::all_descending().count(), 45);
}

/// The distribution, which is not the tidy story it looks like.
///
/// It is tempting to summarise the placements as *days on one rung, years on the
/// next*, and that is wrong. Fifteen days split **eight on `T1`, seven on `T2`**
/// — Luna, Mercury, Venus, Titan, Ganymede, Callisto and Pluto have days long
/// enough to reach the sweep — and fifteen years split **twelve on `T2`, three
/// on `T3`**.
///
/// So days and years *overlap* on `T2`, and the ladder does not separate them.
/// A body's day and another body's year can sit on the same rung, which is what
/// a grid built from powers of five with no knowledge of either would do.
///
/// The neat version was written here first and asserted that Pluto's was the
/// only day off the arc. Seven others already were, before F9 added anything.
/// The tidy claim was a property of the sample, and the sample was never looked
/// at.
#[test]
fn days_and_years_overlap_on_one_rung() {
    let mut days: Vec<String> = Vec::new();
    let mut years: Vec<String> = Vec::new();
    for (_, what, p) in table() {
        match what {
            "solar day" => days.push(label(p.tier)),
            "year" => years.push(label(p.tier)),
            _ => {}
        }
    }
    let uniq = |mut v: Vec<String>| {
        v.sort();
        v.dedup();
        v
    };
    assert_eq!(uniq(days.clone()), vec!["T1 arc".to_string(), "T2 sweep".to_string()]);
    assert_eq!(
        uniq(years.clone()),
        vec!["T2 sweep".to_string(), "T3 span".to_string()]
    );

    // The overlap itself: one rung carries both kinds of unit.
    assert!(
        days.contains(&"T2 sweep".to_string()) && years.contains(&"T2 sweep".to_string()),
        "the sweep should carry some bodies' days and other bodies' years"
    );
}
