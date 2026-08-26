//! How finely this program can place a reading of the system clock.
//!
//! # Why this exists
//!
//! Every measured quantity in this program carries its uncertainty. Parameters
//! carry a validity window, anchors carry a determination, events are intervals,
//! and `ucal datum` bounds the frame conversion at `5 × 10⁻⁶` of elapsed time
//! and says it may not be consumed by arithmetic (Rule Q.3).
//!
//! **The one quantity the program measures itself carried none.** `ucal now`
//! rendered to `T-12` — one Planck tick — and labelled that field `precision`,
//! from a clock that moves in microseconds.
//!
//! # Three different numbers, and the distinction is the whole point
//!
//! **Granularity** is structural. [`now_instant`](crate::now_instant) converts
//! through `SubSecond::new(nanos, 9)`, so one nanosecond is the finest an
//! instant can be placed by this code, whatever the machine underneath is doing.
//! It is the same on every machine and needs no measurement.
//!
//! **Resolution** is what the clock on *this* machine actually moves in, which
//! is usually coarser and has to be sampled.
//!
//! **Accuracy** is the distance from the truth, and this program cannot measure
//! it. §8.4 makes offline operation required — there is no reference to compare
//! against, and a rate error cannot be estimated from a short baseline without
//! reporting quantisation as drift. Naming resolution and calling it accuracy is
//! the error this module exists to avoid.
//!
//! # Why the ladder goes below any instrument
//!
//! `T-12` is one Planck tick, `5.4 × 10⁻⁴⁴` s. The shortest interval anyone has
//! measured is of order `10⁻¹⁹` s, which lands between `T-5` and `T-6` — so the
//! bottom of this ladder is about `10²⁴` times finer than the best measurement
//! ever made, by anyone, with any instrument.
//!
//! That is not a defect in the ladder. The tiers are a **coordinate system**,
//! and being able to address a position is not a claim that anything can be
//! observed there — the same posture as the datum, which is stipulated rather
//! than measured. What was wrong was printing a coordinate under the word
//! *precision*.

use crate::emit::Value;
use ucal_core::backend::TickInt;
use ucal_core::{Profile, Ticks, Tier, TimeError, UC1};

/// Decimal places [`now_instant`](crate::now_instant) reads the clock to.
///
/// Nine: `SubSecond::new(nanos, 9)`. A structural ceiling, not a measurement.
pub const CLOCK_DECIMALS: u32 = 9;

/// Ticks in `n` nanoseconds, exactly.
///
/// The bridge second is `18548584399861 × 10³⁰` ticks, so dividing by `10⁹` is
/// exact and nothing is rounded on the way in (Rule R).
pub fn ticks_in_nanos(n: u128) -> Option<Ticks> {
    let nanos = <Ticks as TickInt>::from_u128(n)?;
    let per_second = UC1::bridge().ticks;
    let billion = <Ticks as TickInt>::from_u64(1_000_000_000);
    // Exact: the bridge second is `18548584399861 x 10^30`, so `10^9` divides
    // it and nothing is rounded on the way in (Rule R). Asserted by
    // `a_nanosecond_is_an_exact_number_of_ticks` rather than here — a
    // `debug_assert!` is a panicking construct, and §19.5 says a failure in this
    // crate leaves through a code and an exit status.
    let (per_nano, _) = per_second.quot_rem(&billion);
    per_nano.try_mul(&nanos)
}

/// The finest tier one unit of which a quantum of `q` ticks can distinguish.
///
/// A tier is *fillable* when its own unit is at least as large as the quantum:
/// below that, the position within the unit is not something the instrument can
/// report and any digits there come from the conversion rather than the clock.
pub fn finest_tier_for(q: &Ticks) -> Option<Tier> {
    (-12i8..=32).find_map(|k| {
        Tier::new(k)
            .ok()
            .filter(|t| &t.ticks() >= q)
    })
}

/// The structural facts: true on every machine, and needing no sample.
pub fn facts() -> Result<Vec<(String, Value)>, TimeError> {
    let q = ticks_in_nanos(1).ok_or_else(|| TimeError::new(ucal_core::Code::E0021))?;
    let finest = finest_tier_for(&q).ok_or_else(|| TimeError::new(ucal_core::Code::E0021))?;
    let rendered_to = Tier::new(-12)?;
    // Rungs between what a nanosecond can fill and where `ucal now` renders.
    // Each is a factor of 5^5, so this is the count that matters rather than the
    // ratio, which is 3125 to the power of it.
    let unearned = finest.index() - rendered_to.index();

    Ok(vec![
        (
            "granularity".into(),
            Value::text("1 ns — this program reads the clock to nine decimal places"),
        ),
        ("granularity_ticks".into(), Value::number(q.to_dec_string())),
        (
            "finest_tier".into(),
            Value::text(format!(
                "{finest} — the finest rung a nanosecond can fill"
            )),
        ),
        (
            "rendering_floor".into(),
            Value::text(format!(
                "{rendered_to} — where `ucal now` renders by default, {unearned} rungs \
                 below what the clock can fill. A rung is 5^5, so those digits are the \
                 conversion's and not the instrument's"
            )),
        ),
        (
            "accuracy".into(),
            Value::text(
                "not measurable here. §8.4 makes operation offline, so there is no \
                 reference to compare against, and a rate error estimated from a short \
                 baseline reports quantisation as drift. This is resolution",
            ),
        ),
        (
            "in_a_difference".into(),
            Value::text(
                "a constant offset cancels between two readings and a rate error does \
                 not; quantisation bounds each reading and so bounds their difference \
                 twice over. The frame term `ucal datum` declares cancels too",
            ),
        ),
    ])
}

/// What the two clocks on *this* machine actually do.
///
/// Sampled, so it varies by machine and by run — which is why it is behind a
/// flag rather than in `doctor`'s ordinary output, whose every line is compared
/// against a committed example.
#[cfg(feature = "std")]
pub fn measured() -> Result<Vec<(String, Value)>, TimeError> {
    let wall = sample_wall();
    let mono = sample_monotonic();

    let mut out = Vec::new();
    out.push((
        "wall_resolution".into(),
        Value::text(match wall {
            Some(ns) => format!("{ns} ns — the smallest step observed in `SystemTime`"),
            None => "not observed within the sampling budget".to_string(),
        }),
    ));
    out.push((
        "monotonic_resolution".into(),
        Value::text(match mono {
            Some(ns) => format!("{ns} ns — the smallest step observed in `Instant`"),
            None => "not observed within the sampling budget".to_string(),
        }),
    ));

    if let Some(ns) = wall {
        if let Some(t) = ticks_in_nanos(ns).as_ref().and_then(finest_tier_for) {
            out.push((
                "wall_finest_tier".into(),
                Value::text(format!("{t} — what this machine's wall clock can fill")),
            ));
        }
    }

    // **The idea, and what it is worth.** Reading the wall clock once and adding
    // monotonic elapsed since is real interpolation, and the honest question is
    // how far up the ladder it moves the answer.
    if let (Some(w), Some(m)) = (wall, mono) {
        let gain = if m == 0 { 0 } else { w / m.max(1) };
        let same = ticks_in_nanos(w)
            .as_ref()
            .and_then(finest_tier_for)
            .zip(ticks_in_nanos(m).as_ref().and_then(finest_tier_for))
            .is_some_and(|(a, b)| a.index() == b.index());
        // As a fraction and not a decimal: Rule E forbids a float anywhere in a
        // shipped crate, and `24/3125` is exact where `0.008` is a rendering of
        // it — which is the distinction Rule R draws everywhere else in this
        // program.
        out.push((
            "interpolating".into(),
            Value::text(format!(
                "taking one wall reading and adding monotonic elapsed since would \
                 sharpen it about {gain}x, and one rung of the ladder is 5^5 = 3125 — \
                 so {gain}/3125 of a rung, and the finest fillable tier {}",
                if same { "does not move" } else { "moves" }
            )),
        ));
    }
    Ok(out)
}

/// The smallest non-zero step `SystemTime` reports, in nanoseconds.
#[cfg(feature = "std")]
fn sample_wall() -> Option<u128> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let read = || {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .ok()
            .map(|d| d.as_nanos())
    };
    sample(read)
}

/// The smallest non-zero step the monotonic clock reports, in nanoseconds.
#[cfg(feature = "std")]
fn sample_monotonic() -> Option<u128> {
    let base = std::time::Instant::now();
    sample(|| Some(base.elapsed().as_nanos()))
}

/// Sample a counter until it moves enough times, or the budget runs out.
///
/// Bounded twice over: `doctor` is a diagnostic and must not become a benchmark,
/// and a loop with only an iteration bound takes a very different time on a very
/// different machine.
#[cfg(feature = "std")]
fn sample(read: impl Fn() -> Option<u128>) -> Option<u128> {
    const BUDGET: std::time::Duration = std::time::Duration::from_millis(30);
    const MOVES: u32 = 400;

    let start = std::time::Instant::now();
    let mut last = read()?;
    let mut smallest: Option<u128> = None;
    let mut moves = 0u32;
    while moves < MOVES && start.elapsed() < BUDGET {
        let now = read()?;
        if now > last {
            let step = now - last;
            smallest = Some(smallest.map_or(step, |s: u128| s.min(step)));
            last = now;
            moves += 1;
        }
    }
    smallest
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A nanosecond is an exact number of ticks, and the bridge divides.
    #[test]
    fn a_nanosecond_is_an_exact_number_of_ticks() {
        let ns = ticks_in_nanos(1).expect("in range");
        let per_second = UC1::bridge().ticks;
        let billion = <Ticks as TickInt>::from_u64(1_000_000_000);
        let (_, rem) = per_second.quot_rem(&billion);
        assert!(rem.is_zero_ticks(), "the bridge second must divide by 10^9");
        // And a billion of them is the second back.
        assert_eq!(
            ns.try_mul(&billion).expect("in range").to_dec_string(),
            per_second.to_dec_string()
        );
    }

    /// **A nanosecond fills `T-2` and not `T-3`.**
    ///
    /// The structural ceiling on anything this program reads from a clock, and
    /// the number the whole module exists to state. `T-2` is 4.8 ns and `T-3` is
    /// 1.5 ps, so a nanosecond quantum resolves the first and not the second.
    #[test]
    fn a_nanosecond_fills_t_minus_2() {
        let q = ticks_in_nanos(1).expect("in range");
        let t = finest_tier_for(&q).expect("some tier");
        assert_eq!(t.index(), -2, "a nanosecond should fill T-2, got {t}");
    }

    /// A microsecond — this machine's observed wall resolution — fills `T-1`.
    ///
    /// So a thousandfold coarser clock costs exactly one rung, which is the
    /// scale a reader needs to judge any of this by.
    #[test]
    fn a_microsecond_fills_t_minus_1() {
        let q = ticks_in_nanos(1_000).expect("in range");
        assert_eq!(finest_tier_for(&q).expect("some tier").index(), -1);
    }

    /// **Nothing measurable reaches the bottom of the ladder.**
    ///
    /// The shortest interval ever measured is of order `10⁻¹⁹` s. Even a clock
    /// a million times finer than a nanosecond — a femtosecond — cannot fill
    /// `T-4`, and `T-12` is another eight rungs below that.
    #[test]
    fn no_instrument_reaches_the_bottom_rungs() {
        // One femtosecond, expressed in the finest unit this helper takes.
        let femto = ticks_in_nanos(1).expect("in range");
        let million = <Ticks as TickInt>::from_u64(1_000_000);
        let (q, _) = femto.quot_rem(&million);
        let t = finest_tier_for(&q).expect("some tier");
        assert!(
            t.index() > -5,
            "a femtosecond should not reach T-5 or below, got {t}"
        );
    }

    /// The facts do not need a machine and do not vary.
    ///
    /// `doctor`'s ordinary output is compared against a committed example byte
    /// for byte, so anything sampled has to stay out of it.
    #[test]
    fn the_facts_are_the_same_every_time() {
        let a = facts().expect("facts");
        let b = facts().expect("facts");
        assert_eq!(a.len(), b.len());
        for (x, y) in a.iter().zip(b.iter()) {
            assert_eq!(x.0, y.0);
            assert_eq!(x.1.rendered_text(), y.1.rendered_text());
        }
    }

    /// And the measurement returns something on a machine that has a clock.
    #[cfg(feature = "std")]
    #[test]
    fn the_measurement_observes_both_clocks() {
        let m = measured().expect("measured");
        for want in ["wall_resolution", "monotonic_resolution"] {
            let (_, v) = m
                .iter()
                .find(|(k, _)| k == want)
                .unwrap_or_else(|| panic!("no `{want}`"));
            assert!(
                v.rendered_text().contains("ns"),
                "`{want}` observed nothing: {}",
                v.rendered_text()
            );
        }
    }
}
