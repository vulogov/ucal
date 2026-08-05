//! Derived calendars (§9.3, §15.3–15.5).
//!
//! # One struct, no body-specific path
//!
//! §15.4: "Earth's entry has no special code path, no extra fields, and no
//! compile-time distinction from Mars's." So [`BodyCalendar`] is a single
//! concrete struct, not a trait with one implementation per body. There is
//! nowhere for a special case to live, which is stronger than a rule against
//! writing one.
//!
//! # What a derived calendar has, and what it refuses to have
//!
//! §15.3: "`DerivedFields` MUST NOT contain a month or weekday unless a cycle was
//! derived. No fallback structure is permitted." [`DerivedFields`] therefore has
//! no `month` field and no `weekday` field at all — only an optional
//! [`CyclePosition`], present exactly when the calendar named a grouping
//! satellite. A month one could read as zero would be a fallback; an absent field
//! cannot be.
//!
//! # Why `instant()` returns a window
//!
//! §9.3: "`instant()` returns a `Window`, never an `Instant`: local fields cannot
//! resolve to a single tick while the anchor has width." Rule J.2 makes the same
//! point from the other side — anchor uncertainty propagates, so every `fields()`
//! result is interval-valued and reports the revision that produced it.
//!
//! # What the window does and does not include
//!
//! It carries the **anchor's** uncertainty, which is the dominant term and the
//! one Rule J.2 names. It does not carry a ± on the body's periods, because the
//! sources do not uniformly publish one: `RatedParam` records a validity window
//! and a drift rate, both of which the RFC requires, but not a magnitude of
//! uncertainty on the value itself. Adding a fabricated one would be worse than
//! omitting it, so the omission is stated here rather than papered over.

use alloc::vec::Vec;
use alloc::string::ToString;

use ucal_core::num::Ratio;
use ucal_core::qualified::{CalendarIdentity, CalendarQualifier, Kind, Qualified};
use ucal_core::{backend::TickInt, Code, Delta, Instant, Ticks, TimeError, Window, UC1};

use crate::anchor::Anchor;
use crate::body::Body;
use crate::derive::{derive_cycles, derive_leap_rule, Cycle, DriftBound, LeapRule};

type Result<T> = core::result::Result<T, TimeError>;

/// Where an instant falls within a derived grouping cycle (§9.6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CyclePosition {
    /// Which satellite the cycle comes from.
    pub satellite: &'static str,
    /// Completed cycles since the anchor.
    pub index: u64,
    /// Position within the current cycle, in `[0, 1)`.
    pub phase: Ratio,
}

/// A local date, produced by Rule K's mechanism (§9.3).
///
/// Note the absent fields. There is no month and no weekday, because §15.3
/// forbids them unless a cycle was derived — and a cycle, when there is one,
/// appears as [`CyclePosition`] rather than as a month number that could be
/// mistaken for a table lookup.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DerivedFields {
    /// Local years since the anchor, 1-based: the anchor instant is year 1.
    pub year: i64,
    /// Day of the local year, 1-based.
    pub day: u32,
    /// Position within the local day, in `[0, 1)`.
    pub day_fraction: Ratio,
    /// Where in the grouping cycle, if the calendar declared one.
    pub cycle: Option<CyclePosition>,
    /// The interval of absolute time these fields are consistent with, given the
    /// anchor's uncertainty (Rule J.2, Rule U).
    pub window: Window<UC1>,
    /// Which anchor determination produced this (Rule J.5).
    pub anchor_revision: u32,
    /// Whether the anchor's uncertainty spans a local day boundary, so that the
    /// day number itself is not determined.
    pub day_is_ambiguous: bool,
}

/// A calendar derived from a body under Rule K (§9.3).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct BodyCalendar {
    id: &'static str,
    body: Body,
    anchor: Anchor,
    leap_rule: LeapRule,
    cycles: Vec<Cycle>,
}

impl BodyCalendar {
    /// Build a calendar from a body, an anchor, and an optional grouping
    /// satellite.
    ///
    /// This is the **only** constructor, and it takes the same arguments for
    /// every body. §21.3-10 requires Earth and Mars to be produced by the
    /// identical generic path; here there is no other path to take.
    pub fn build(
        id: &'static str,
        body: Body,
        anchor: Anchor,
        grouping_satellite: Option<&str>,
        bound: DriftBound,
        max_depth: u32,
    ) -> Result<BodyCalendar> {
        if anchor.calendar_id() != id {
            return Err(TimeError::with_context(
                Code::E0062,
                "the anchor names a different calendar",
            ));
        }
        anchor.check_evaluable(&body)?;
        let leap_rule = derive_leap_rule(
            body.solar_day().value_at_epoch(),
            body.orbital_period().value_at_epoch(),
            bound,
            max_depth,
        )?;
        let cycles = derive_cycles(&body, grouping_satellite, max_depth)?;
        Ok(BodyCalendar {
            id,
            body,
            anchor,
            leap_rule,
            cycles,
        })
    }

    /// The body this calendar is derived from.
    pub fn body(&self) -> &Body {
        &self.body
    }

    /// The anchor (Rule J).
    pub fn anchor(&self) -> &Anchor {
        &self.anchor
    }

    /// The derived intercalation rule.
    pub fn leap_rule(&self) -> &LeapRule {
        &self.leap_rule
    }

    /// The derived grouping cycles. Empty when the calendar names no satellite.
    pub fn cycles(&self) -> &[Cycle] {
        &self.cycles
    }

    /// Local days in a whole intercalation cycle: `q x whole + p`.
    fn days_per_cycle(&self) -> Result<Ticks> {
        let q = self.leap_rule.chosen.value.denom().clone();
        let p = self.leap_rule.chosen.value.numer().clone();
        let whole = self.leap_rule.whole_days.numer().clone();
        q.try_mul(&whole)
            .and_then(|v| v.try_add(&p))
            .ok_or(TimeError::new(Code::E0021))
    }

    /// Cumulative local days before local year `y` of a cycle, 0-based:
    /// `y x whole + floor(y x p / q)`.
    ///
    /// This is what distributes the leap days evenly through the cycle without a
    /// table — the intercalation is a consequence of the fraction, exactly as
    /// Rule K.2 requires.
    fn days_before_year(&self, y: &Ticks) -> Result<Ticks> {
        let whole = self.leap_rule.whole_days.numer();
        let p = self.leap_rule.chosen.value.numer();
        let q = self.leap_rule.chosen.value.denom();
        let base = y.try_mul(whole).ok_or(TimeError::new(Code::E0021))?;
        let extra = y
            .try_mul(p)
            .ok_or(TimeError::new(Code::E0021))?
            .quot_rem(q)
            .0;
        base.try_add(&extra).ok_or(TimeError::new(Code::E0021))
    }

    /// Split a whole local day count into (year, day-of-year), both 0-based.
    fn split_year(&self, days: &Ticks) -> Result<(Ticks, Ticks)> {
        let per_cycle = self.days_per_cycle()?;
        let q = self.leap_rule.chosen.value.denom().clone();
        let (cycles, rem) = days.quot_rem(&per_cycle);

        // Estimate the year within the cycle, then correct. The estimate is never
        // more than one out, but it is checked rather than trusted.
        let mut y = rem
            .try_mul(&q)
            .ok_or(TimeError::new(Code::E0021))?
            .quot_rem(&per_cycle)
            .0;
        loop {
            let start = self.days_before_year(&y)?;
            if start > rem {
                y = y
                    .try_sub(&<Ticks as TickInt>::one())
                    .ok_or(TimeError::new(Code::E0020))?;
                continue;
            }
            let next = y
                .try_add(&<Ticks as TickInt>::one())
                .ok_or(TimeError::new(Code::E0021))?;
            if self.days_before_year(&next)? <= rem {
                y = next;
                continue;
            }
            let day_of_year = rem.try_sub(&start).expect("start <= rem");
            let year = cycles
                .try_mul(&q)
                .and_then(|v| v.try_add(&y))
                .ok_or(TimeError::new(Code::E0021))?;
            return Ok((year, day_of_year));
        }
    }

    /// Decompose an instant into local fields (§15.5).
    ///
    /// `UCAL-E0020` for an instant before the anchor: local counting had not
    /// begun, and a negative year would be an invention.
    pub fn fields(&self, t: &Instant<UC1>) -> Result<DerivedFields> {
        let solar_day = {
            let (v, _) = self.body.solar_day().evaluate(t)?;
            v
        };
        let elapsed = t.since(self.anchor.tick()).map_err(|_| {
            TimeError::with_context(
                Code::E0020,
                "this instant precedes the calendar's anchor, so local counting \
                 had not begun",
            )
        })?;

        // Whole local days and the fraction within the day, exactly.
        let elapsed_r = Ratio::from_int(elapsed.ticks().clone());
        let in_days = elapsed_r.div(&solar_day)?;
        let whole_days = in_days.floor();
        let day_fraction = in_days.frac();

        let (year0, day0) = self.split_year(&whole_days)?;
        let year: i64 = year0
            .to_dec_string()
            .parse::<i64>()
            .map_err(|_| TimeError::with_context(Code::E0040, "local year out of range"))?
            + 1;
        let day: u32 = day0
            .to_dec_string()
            .parse::<u32>()
            .map_err(|_| TimeError::with_context(Code::E0040, "day of year out of range"))?
            + 1;

        // Rule J.2: the anchor's uncertainty propagates. The window is the set of
        // instants indistinguishable from `t` given that uncertainty.
        let half = {
            let (h, _) = self.anchor.uncertainty().divmod(&Delta::from_u64(2))?;
            h
        };
        let (window, _clamped) = Window::exact(t.clone()).widen(&half)?;

        // If that window spans a local day boundary, the day number is not
        // determined and the caller is told so rather than being given a number
        // with more authority than it has.
        let day_is_ambiguous = {
            let lo = Ratio::from_int(window.lo().since(self.anchor.tick()).map(|d| d.ticks().clone()).unwrap_or_else(|_| <Ticks as TickInt>::zero()));
            let hi = Ratio::from_int(window.hi().since(self.anchor.tick())?.ticks().clone());
            lo.div(&solar_day)?.floor() != hi.div(&solar_day)?.floor()
        };

        let cycle = match self.cycles.first() {
            None => None,
            Some(c) => {
                let in_cycles = elapsed_r.div(&c.synodic_period)?;
                let index: u64 = in_cycles
                    .floor()
                    .to_dec_string()
                    .parse()
                    .map_err(|_| TimeError::with_context(Code::E0040, "cycle index"))?;
                Some(CyclePosition {
                    satellite: c.satellite,
                    index,
                    phase: in_cycles.frac(),
                })
            }
        };

        Ok(DerivedFields {
            year,
            day,
            day_fraction,
            cycle,
            window,
            anchor_revision: self.anchor.revision(),
            day_is_ambiguous,
        })
    }

    /// Recompose an instant from local fields (§9.3).
    ///
    /// Returns a **window**, never an instant: the anchor has width, so a local
    /// date names an interval of absolute time and nothing narrower.
    pub fn instant(&self, f: &DerivedFields) -> Result<Window<UC1>> {
        if f.year < 1 || f.day < 1 {
            return Err(TimeError::with_context(
                Code::E0041,
                "local years and days are 1-based; the anchor is year 1, day 1",
            ));
        }
        let solar_day = self.body.solar_day().value_at_epoch();
        let y = <Ticks as TickInt>::from_u64((f.year - 1) as u64);
        let days = self
            .days_before_year(&y)?
            .try_add(&<Ticks as TickInt>::from_u64((f.day - 1) as u64))
            .ok_or(TimeError::new(Code::E0021))?;

        let offset = Ratio::from_int(days)
            .add(&f.day_fraction)?
            .mul(solar_day)?;
        // The offset is exact in ticks only if the fraction divides through; take
        // the floor and let the window carry the rest.
        let ticks = offset.floor();
        let centre = self
            .anchor
            .tick()
            .checked_add(&Delta::from_ticks(ticks))?;
        let (w, _) = Window::exact(centre).widen(&{
            let (h, _) = self.anchor.uncertainty().divmod(&Delta::from_u64(2))?;
            h
        })?;
        Ok(w)
    }

    /// Render a local date, always qualified (§6.6).
    ///
    /// The revision is part of the qualifier, so values from different anchor
    /// determinations can never be silently compared (Rule J.5).
    pub fn render(&self, t: &Instant<UC1>) -> Result<Qualified<'_, alloc::string::String>> {
        use alloc::format;
        let f = self.fields(t)?;
        // ucal-lint-allow-begin(rounding-is-declared): not the caller's choice, and
        // the mode is forced rather than preferred. `Trunc` is required: a day
        // fraction rounded *up* can reach 1.0000 and name the following day, so
        // half-even here would make `render` disagree with `fields`. The four
        // digits are part of the label's grammar (§6.6), like the two digits in
        // a civil minute — not a precision the caller sets.
        let frac = f
            .day_fraction
            .to_decimal_string(4, ucal_core::Rounding::Trunc)
            .unwrap_or_default();
        // ucal-lint-allow-end(rounding-is-declared)
        let frac = frac.split('.').nth(1).unwrap_or("0000").to_string();
        let body = match &f.cycle {
            None => format!("{:04}-{:03}.{}", f.year, f.day, frac),
            Some(c) => format!("{:04}-{:03}.{} c{}", f.year, f.day, frac, c.index),
        };
        Ok(CalendarQualifier::derived(self.id, f.anchor_revision).attach(body))
    }
}

impl CalendarIdentity for BodyCalendar {
    fn id(&self) -> &str {
        self.id
    }
    /// Always [`Kind::Derived`] (§9.3). There is no branch here.
    fn kind(&self) -> Kind {
        Kind::Derived
    }
    fn revision(&self) -> Option<u32> {
        Some(self.anchor.revision())
    }
}

/// Build every calendar for which an anchor exists.
///
/// Calendars without an anchor are absent, not defaulted — Rule J.3, and the
/// state Appendix I.6 describes.
pub fn all() -> Vec<BodyCalendar> {
    let mut out = Vec::new();
    for (id, body, grouping) in [
        ("earth-d", crate::data::earth(), Some("moon")),
        ("mars-d", crate::data::mars(), None),
        ("titan-d", crate::data::titan(), None),
    ] {
        if let Some(anchor) = crate::anchors::for_calendar(id) {
            if let Ok(c) = BodyCalendar::build(
                id,
                body,
                anchor,
                grouping,
                DriftBound::DEFAULT,
                32,
            ) {
                out.push(c);
            }
        }
    }
    out
}

/// A calendar by id, or `UCAL-E0062` when it has no anchor (Rule J.3).
pub fn by_id(id: &str) -> Result<BodyCalendar> {
    all()
        .into_iter()
        .find(|c| c.id == id)
        .ok_or_else(|| {
            TimeError::with_context(
                Code::E0062,
                "no such derived calendar, or it has no anchor and so cannot \
                 produce local fields (Rule J.3)",
            )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{anchors, data};
    use ucal_core::{Profile, Rounding};

    fn earth_d() -> BodyCalendar {
        by_id("earth-d").unwrap()
    }
    fn mars_d() -> BodyCalendar {
        by_id("mars-d").unwrap()
    }

    // ---- §21.3-10 ----

    #[test]
    fn earth_and_mars_are_built_by_the_identical_path() {
        // "verified by a test that constructs both from data alone". The same
        // constructor, the same argument shapes, differing only in the data.
        let cases: [(&'static str, Body, Option<&str>); 2] = [
            ("earth-d", data::earth(), Some("moon")),
            ("mars-d", data::mars(), None),
        ];
        let mut built = Vec::new();
        for (id, body, grouping) in cases {
            let anchor = anchors::for_calendar(id).expect("anchor");
            built.push(
                BodyCalendar::build(id, body, anchor, grouping, DriftBound::DEFAULT, 32)
                    .expect("both must build"),
            );
        }
        assert_eq!(built.len(), 2);
        for c in &built {
            assert_eq!(c.kind(), Kind::Derived);
            assert!(c.leap_rule().depth >= 1);
            assert_eq!(c.anchor().revision(), 1);
        }
        // The only difference is data: Earth's calendar named a satellite.
        assert_eq!(built[0].cycles().len(), 1);
        assert_eq!(built[1].cycles().len(), 0);
    }

    #[test]
    fn a_calendar_is_always_derived_never_legacy() {
        for c in all() {
            assert_eq!(c.kind(), Kind::Derived);
            assert!(c.id().ends_with("-d"), "{}", c.id());
        }
    }

    // ---- Rule J.3 ----

    #[test]
    fn titan_has_no_calendar_because_it_has_no_anchor() {
        // Complete in units, intercalation and cycles; incomplete in phase.
        assert!(anchors::for_calendar("titan-d").is_none());
        let e = by_id("titan-d").unwrap_err();
        assert_eq!(e.code, Code::E0062);
        // But the derivation itself works — it is only phase that is missing.
        let rule = crate::derive::derive_leap_rule(
            data::titan().solar_day().value_at_epoch(),
            data::titan().orbital_period().value_at_epoch(),
            DriftBound::DEFAULT,
            32,
        )
        .unwrap();
        assert_eq!(rule.whole_days.numer().to_dec_string(), "673");
    }

    // ---- §15.3 ----

    #[test]
    fn fields_carry_no_month_or_weekday_without_a_cycle() {
        // The field simply does not exist on the struct, so there is nothing to
        // read as zero. Mars, which names no satellite, gets `None`.
        let m = mars_d();
        let f = m.fields(m.anchor().tick()).unwrap();
        assert!(f.cycle.is_none());
        // Earth, which names one, gets a position rather than a month number.
        let e = earth_d();
        let f = e.fields(e.anchor().tick()).unwrap();
        let c = f.cycle.expect("earth-d declares the Moon");
        assert_eq!(c.satellite, "moon");
    }

    // ---- Rule J.2 / §9.3 ----

    #[test]
    fn fields_are_interval_valued_and_carry_the_revision() {
        let e = earth_d();
        let t = e
            .anchor()
            .tick()
            .checked_add(&Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .try_mul(&<Ticks as TickInt>::from_u64(86_400 * 100))
                    .unwrap(),
            ))
            .unwrap();
        let f = e.fields(&t).unwrap();
        assert_eq!(f.anchor_revision, 1);
        assert!(!f.window.is_exact(), "the anchor has width, so the window must");
        assert!(f.window.contains(&t));
        // The window is the anchor's uncertainty, carried forward.
        assert_eq!(f.window.width(), e.anchor().uncertainty());
    }

    #[test]
    fn instant_returns_a_window_never_a_tick() {
        let e = earth_d();
        let t = e
            .anchor()
            .tick()
            .checked_add(&Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .try_mul(&<Ticks as TickInt>::from_u64(86_400 * 500))
                    .unwrap(),
            ))
            .unwrap();
        let f = e.fields(&t).unwrap();
        let w = e.instant(&f).unwrap();
        assert!(!w.is_exact());
        // §21.1: instant(fields(t)) must contain t.
        assert!(
            w.contains(&t),
            "the round trip must contain the instant it came from"
        );
    }

    #[test]
    fn the_anchor_is_year_one_day_one() {
        for c in all() {
            let f = c.fields(c.anchor().tick()).unwrap();
            assert_eq!((f.year, f.day), (1, 1), "{}", c.id());
        }
    }

    #[test]
    fn an_instant_before_the_anchor_is_e0020() {
        let e = earth_d();
        let before = e
            .anchor()
            .tick()
            .checked_sub(&Delta::from_ticks(UC1::bridge().ticks))
            .unwrap();
        assert_eq!(e.fields(&before).unwrap_err().code, Code::E0020);
    }

    // ---- the intercalation actually intercalates ----

    #[test]
    fn year_lengths_follow_the_derived_rule() {
        // Earth's derived rule is 31/128: 31 leap days per 128 years, so a cycle
        // is 128 x 365 + 31 = 46 751 days. The mechanism must produce exactly
        // that, without a table.
        let e = earth_d();
        assert_eq!(e.days_per_cycle().unwrap().to_dec_string(), "46751");
        // And the year lengths are 365 or 366, never anything else.
        let mut long = 0;
        for y in 0..128u64 {
            let a = e
                .days_before_year(&<Ticks as TickInt>::from_u64(y))
                .unwrap();
            let b = e
                .days_before_year(&<Ticks as TickInt>::from_u64(y + 1))
                .unwrap();
            let len = b.try_sub(&a).unwrap().to_dec_string();
            assert!(len == "365" || len == "366", "year {y} had {len} days");
            if len == "366" {
                long += 1;
            }
        }
        assert_eq!(long, 31, "31 leap days per 128-year cycle");
    }

    #[test]
    fn day_numbers_advance_by_one_per_local_day() {
        let e = earth_d();
        let day = UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(86_400))
            .unwrap();
        let mut prev: Option<(i64, u32)> = None;
        for n in 0..400u64 {
            let t = e
                .anchor()
                .tick()
                .checked_add(&Delta::from_ticks(
                    day.try_mul(&<Ticks as TickInt>::from_u64(n)).unwrap(),
                ))
                .unwrap();
            let f = e.fields(&t).unwrap();
            if let Some((py, pd)) = prev {
                let advanced = (f.year == py && f.day == pd + 1) || (f.year == py + 1 && f.day == 1);
                assert!(advanced, "day {n}: {py}-{pd} -> {}-{}", f.year, f.day);
            }
            prev = Some((f.year, f.day));
        }
    }

    // ---- §6.6 ----

    #[test]
    fn rendering_carries_the_id_kind_and_revision() {
        let e = earth_d();
        let t = e.anchor().tick();
        let r = e.render(t).unwrap();
        assert!(r.to_string().starts_with("earth-d/1: "), "{}", r);
        assert_eq!(r.qualifier().kind(), Kind::Derived);
        assert_eq!(r.qualifier().revision(), Some(1));
        // No W0005: this is a derivation, not legacy data.
        assert_eq!(r.warning(), None);

        let m = mars_d();
        assert!(m.render(m.anchor().tick()).unwrap().to_string().starts_with("mars-d/1: "));
    }

    #[test]
    fn a_cycle_appears_in_the_rendering_only_when_one_was_derived() {
        let e = earth_d();
        let m = mars_d();
        let er = e.render(e.anchor().tick()).unwrap().to_string();
        let mr = m.render(m.anchor().tick()).unwrap().to_string();
        assert!(er.contains(" c"), "earth-d has a cycle: {er}");
        assert!(!mr.contains(" c"), "mars-d has none: {mr}");
    }

    #[test]
    fn the_cycle_position_tracks_the_moon() {
        // One synodic month after the anchor, the cycle index must be 1.
        let e = earth_d();
        let syn = e.cycles()[0].synodic_period.clone();
        //  lands one tick short of a whole cycle, so the index is still 0
        // there — which is correct, and worth pinning from both sides.
        let just_short = e
            .anchor()
            .tick()
            .checked_add(&Delta::from_ticks(syn.floor()))
            .unwrap();
        assert_eq!(e.fields(&just_short).unwrap().cycle.unwrap().index, 0);

        let just_past = just_short.checked_add(&Delta::one_tick()).unwrap();
        assert_eq!(e.fields(&just_past).unwrap().cycle.unwrap().index, 1);

        // ...and twelve cycles in is index 12.
        let twelve = e
            .anchor()
            .tick()
            .checked_add(&Delta::from_ticks(
                syn.mul(&Ratio::from_u64(12)).unwrap().floor(),
            ))
            .unwrap()
            .checked_add(&Delta::one_tick())
            .unwrap();
        assert_eq!(e.fields(&twelve).unwrap().cycle.unwrap().index, 12);
    }

    #[test]
    fn a_derived_year_is_not_a_civil_year() {
        // Sanity: the anchor is year 1, so a derived year number is an offset
        // from the anchor and bears no relation to a civil year label. §9.8.
        let e = earth_d();
        let day = UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(86_400))
            .unwrap();
        let t = e
            .anchor()
            .tick()
            .checked_add(&Delta::from_ticks(
                day.try_mul(&<Ticks as TickInt>::from_u64(365 * 26)).unwrap(),
            ))
            .unwrap();
        let f = e.fields(&t).unwrap();
        assert!(f.year < 100, "a derived year counts from the anchor, not from 1 CE");
        let _ = Rounding::Trunc;
    }
}
