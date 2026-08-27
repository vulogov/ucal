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
use ucal_core::{Instant, Profile, Ticks, Tier, TimeError, UC1};

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

/// One process, one clock.
///
/// # What this is for, which is not precision
///
/// Interpolating between wall-clock ticks with the monotonic clock is worth
/// `24/3125` of a rung on the machine these notes were written on — [`measured`]
/// prints that, and it is nothing. **This exists for monotonicity.**
///
/// The system clock is disciplined by something outside this process. It can
/// step *backwards*: NTP correcting a large offset, a VM resuming, an operator
/// setting the date. A one-shot command never notices; `ucal wallclock` reads
/// the clock twenty times a second for as long as it is left running, and a
/// backward step there is a face that goes back in time.
///
/// # The rule
///
/// ```text
/// reading = max(wall_now, anchor + monotonic_elapsed)
/// ```
///
/// - **Ordinarily** the two agree to within the clock's quantum and either wins.
/// - **A backward step** loses to the monotonic branch, which keeps advancing at
///   the oscillator's rate rather than freezing until the wall clock catches up.
/// - **A forward step** wins, because a forward jump is a correction arriving and
///   refusing it would be preferring this process's opinion to the system's.
///
/// So a reading never goes backwards, and the clock still tracks the system
/// forwards. Those are the two properties; neither is a claim about accuracy.
///
/// # What it does not do
///
/// It does not make a reading finer, more accurate, or traceable to anything.
/// Over a long run the monotonic branch and the wall branch diverge by the rate
/// difference between the oscillator and whatever disciplines the wall clock —
/// `max` bounds the result by whichever is ahead, and that bound is the honest
/// statement of what a session clock costs.
#[derive(Clone, Debug)]
pub struct Session {
    /// The wall reading this session was anchored at.
    anchor: Ticks,
    /// The largest reading handed out so far.
    last: Ticks,
}

impl Session {
    /// Anchor a session at a wall reading.
    pub fn anchored_at(wall: &Instant<UC1>) -> Session {
        Session {
            anchor: wall.ticks().clone(),
            last: wall.ticks().clone(),
        }
    }

    /// The next reading, given the wall clock now and monotonic elapsed since
    /// the anchor.
    ///
    /// Pure: no clock is read here, which is what lets a test drive a wall clock
    /// backwards and check that the answer does not follow it.
    pub fn reading(
        &mut self,
        wall_now: &Instant<UC1>,
        monotonic_elapsed: &Ticks,
    ) -> Result<Instant<UC1>, TimeError> {
        let projected = self
            .anchor
            .try_add(monotonic_elapsed)
            .ok_or_else(|| TimeError::new(ucal_core::Code::E0021))?;
        // The largest of the three. `last` is belt and braces: the first two
        // cannot go backwards on their own, and a caller handing back a smaller
        // elapsed would make them, and this type sells monotonicity.
        let mut best = wall_now.ticks().clone();
        if projected > best {
            best = projected;
        }
        if self.last > best {
            best = self.last.clone();
        }
        self.last = best.clone();
        Instant::from_ticks(best)
    }

    /// How far the wall clock has moved away from this session's projection.
    ///
    /// Signed by a word rather than a number, because a tick count is unsigned
    /// (Rule B) — the same shape `SignedWindow` and the odometer take.
    pub fn divergence(&self, wall_now: &Instant<UC1>, monotonic_elapsed: &Ticks) -> (Ticks, bool) {
        let projected = match self.anchor.try_add(monotonic_elapsed) {
            Some(p) => p,
            None => return (<Ticks as TickInt>::zero(), false),
        };
        let w = wall_now.ticks();
        if w >= &projected {
            (w.try_sub(&projected).unwrap_or_else(<Ticks as TickInt>::zero), true)
        } else {
            (projected.try_sub(w).unwrap_or_else(<Ticks as TickInt>::zero), false)
        }
    }
}

/// The process-wide session, anchored on first use.
///
/// `Option`, and anchored inside [`session_now`] rather than in `get_or_init`:
/// anchoring needs a clock reading, reading a clock can fail, and `OnceLock`'s
/// initialiser cannot return a `Result`. The first draft filled the gap with an
/// `.expect()` and the `no-panic-in-cli` lint refused it — §19.5 says a failure
/// in this crate leaves through a code and an exit status.
#[cfg(all(feature = "std", feature = "civil"))]
type SessionState = std::sync::Mutex<Option<(Session, std::time::Instant)>>;

#[cfg(all(feature = "std", feature = "civil"))]
fn session() -> &'static SessionState {
    use std::sync::{Mutex, OnceLock};
    static S: OnceLock<SessionState> = OnceLock::new();
    S.get_or_init(|| Mutex::new(None))
}

/// The session clock's reading, for [`crate::now_instant`].
#[cfg(all(feature = "std", feature = "civil"))]
pub fn session_now() -> Result<Instant<UC1>, TimeError> {
    let wall = crate::wall_instant()?;
    // A poisoned lock means another thread panicked holding it. The state is a
    // pair of tick counts and cannot be torn, so taking the inner value is
    // correct rather than merely convenient — the same judgement
    // `body_file::leak` makes about its intern pool.
    let mut g = match session().lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    // Anchored on first use rather than at start-up: a command that never asks
    // the time should not read a clock, and most of them do not.
    let (sess, base) = g.get_or_insert_with(|| {
        (Session::anchored_at(&wall), std::time::Instant::now())
    });
    let elapsed = ticks_in_nanos(base.elapsed().as_nanos())
        .ok_or_else(|| TimeError::new(ucal_core::Code::E0021))?;
    sess.reading(&wall, &elapsed)
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

    // ---- the session clock ---------------------------------------------

    fn at(secs: u64) -> Instant<UC1> {
        let t = ticks_in_nanos(u128::from(secs) * 1_000_000_000).expect("in range");
        Instant::from_ticks(t).expect("in the domain")
    }

    fn ns(n: u128) -> Ticks {
        ticks_in_nanos(n).expect("in range")
    }

    /// Ordinarily the two branches agree and the reading tracks the wall clock.
    #[test]
    fn a_quiet_session_follows_the_wall_clock() {
        let mut s = Session::anchored_at(&at(100));
        let r = s
            .reading(&at(105), &ns(5_000_000_000))
            .expect("a reading");
        assert_eq!(r.ticks().to_dec_string(), at(105).ticks().to_dec_string());
    }

    /// **The reason this type exists.** A wall clock that steps backwards does
    /// not take the reading with it.
    ///
    /// `ucal wallclock` reads the clock twenty times a second for as long as it
    /// is left running, and NTP correcting a large offset, a VM resuming, or an
    /// operator setting the date all step it backwards. A face that goes back in
    /// time is the failure this prevents.
    #[test]
    fn a_backward_step_does_not_move_the_reading_back() {
        let mut s = Session::anchored_at(&at(100));
        let before = s
            .reading(&at(110), &ns(10_000_000_000))
            .expect("a reading");
        // The wall clock jumps back five seconds; monotonic keeps counting.
        let after = s
            .reading(&at(105), &ns(11_000_000_000))
            .expect("a reading");
        assert!(
            after.ticks() >= before.ticks(),
            "the reading went backwards: {} then {}",
            before.ticks().to_dec_string(),
            after.ticks().to_dec_string()
        );
        // And it kept advancing rather than freezing until the wall caught up.
        assert!(
            after.ticks() > before.ticks(),
            "the reading stalled instead of advancing at the oscillator's rate"
        );
    }

    /// A forward step is a correction arriving, and is accepted.
    ///
    /// Refusing it would be preferring this process's opinion of the time to the
    /// system's, which is not a trade a clock gets to make on its reader's
    /// behalf.
    #[test]
    fn a_forward_step_is_accepted() {
        let mut s = Session::anchored_at(&at(100));
        s.reading(&at(101), &ns(1_000_000_000)).expect("a reading");
        let jumped = s
            .reading(&at(160), &ns(2_000_000_000))
            .expect("a reading");
        assert_eq!(
            jumped.ticks().to_dec_string(),
            at(160).ticks().to_dec_string(),
            "a forward correction was refused"
        );
    }

    /// Monotone over a walk that steps backwards repeatedly.
    ///
    /// One backward step is a case; a clock being dragged around is the
    /// condition, and the guarantee is over the whole sequence.
    #[test]
    fn the_sequence_is_monotone_however_the_wall_clock_behaves() {
        let mut s = Session::anchored_at(&at(1_000));
        let mut prev = at(1_000).ticks().clone();
        // A wall clock that lurches: forward, back, back further, forward again.
        let walk = [1_001u64, 1_002, 999, 1_000, 990, 1_010, 1_005];
        for (i, w) in walk.iter().enumerate() {
            let elapsed = ns((i as u128 + 1) * 1_000_000_000);
            let r = s.reading(&at(*w), &elapsed).expect("a reading");
            assert!(
                r.ticks() >= &prev,
                "step {i} went backwards: {} then {}",
                prev.to_dec_string(),
                r.ticks().to_dec_string()
            );
            prev = r.ticks().clone();
        }
    }

    /// Divergence is reported as a magnitude and a direction, never a negative.
    ///
    /// A tick count is unsigned by Rule B, so the sign is a word beside the
    /// number — the same shape `SignedWindow` and the wall clock's odometer take.
    #[test]
    fn divergence_carries_its_direction_separately() {
        let s = Session::anchored_at(&at(100));
        let (ahead, wall_ahead) = s.divergence(&at(105), &ns(4_000_000_000));
        assert!(wall_ahead, "the wall clock is ahead of the projection");
        assert_eq!(ahead.to_dec_string(), ns(1_000_000_000).to_dec_string());

        let (behind, wall_ahead) = s.divergence(&at(103), &ns(4_000_000_000));
        assert!(!wall_ahead, "the wall clock is behind the projection");
        assert_eq!(behind.to_dec_string(), ns(1_000_000_000).to_dec_string());
    }

    /// **It does not claim to be finer.** A session reading lands on the same
    /// rung a raw one does.
    ///
    /// The whole point of the accounting above is that interpolation is worth
    /// `24/3125` of a rung, so a type built for monotonicity must not be read as
    /// having bought precision.
    #[test]
    fn a_session_reading_is_no_finer_than_a_raw_one() {
        let q = ticks_in_nanos(1_000).expect("in range");
        let raw = finest_tier_for(&q).expect("a tier");
        // Anchoring and projecting changes nothing about what a quantum fills.
        let mut s = Session::anchored_at(&at(100));
        let _ = s.reading(&at(100), &ns(0)).expect("a reading");
        assert_eq!(finest_tier_for(&q).expect("a tier").index(), raw.index());
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
