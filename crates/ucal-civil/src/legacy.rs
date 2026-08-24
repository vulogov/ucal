//! Legacy civil calendars (§8.6). **Declared tables, not derivations.**
//!
//! # What makes a calendar legacy
//!
//! Rule K says every calendar in this specification is an instance of one
//! mechanism: units from a body's periods, intercalation from a continued-
//! fraction expansion, grouping from a declared satellite. §8.6 carves out a
//! single exception, and marks it in every output.
//!
//! The Gregorian and Julian calendars are that exception. Nothing about them is
//! derived. Their month lengths are irregular for historical reasons, their week
//! has no astronomical period behind it at all, and their intercalation rule was
//! chosen rather than computed — Appendix I.1 shows that 97/400 is **not** a
//! convergent of the tropical year at any depth, and that 8/33 is more accurate
//! with a denominator twelve times smaller.
//!
//! §8.6 requires each legacy calendar to declare that arbitrary content
//! explicitly, so [`DeclaredTables::arbitrary`] is a list of statements rather
//! than a comment, and `every_legacy_calendar_declares_its_arbitrary_content`
//! checks that all four categories are covered.
//!
//! # Why they exist here at all
//!
//! Interoperation, and nothing else. Empirical inputs arrive as civil dates
//! (Rule Y), and a user who has a date needs to convert it. What they must not
//! be able to do is pass one where Rule K requires a derivation — `LegacyCalendar`
//! and `BodyCalendar` are distinct traits with no blanket conversion, and
//! `UCAL-E0065` is the runtime backstop for erased types.

#[cfg(feature = "alloc")]
use alloc::string::String;

use ucal_core::qualified::{CalendarIdentity, CalendarQualifier, Kind, Qualified};
use ucal_core::{Citation, Code, Instant, Rounding, TimeError, UC1};

use crate::calendar::CivilCalendar;
use crate::si::{from_civil, to_civil, CivilFields, Scale, SubSecond};

type Result<T> = core::result::Result<T, TimeError>;

/// The intercalation rule a legacy calendar declares.
///
/// Declared, not derived — which is the whole distinction. A derived calendar's
/// rule is a convergent of `orbital_period / solar_day` and carries the depth it
/// was taken at; this carries only a fraction someone chose.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct DeclaredLeapRule {
    /// Leap days per cycle.
    pub numerator: u32,
    /// Years per cycle.
    pub denominator: u32,
    /// Whether this fraction is a continued-fraction convergent of the tropical
    /// year. `false` for 97/400 — see Appendix I.1.
    pub is_convergent: bool,
}

impl DeclaredLeapRule {
    /// Declare an intercalation rule.
    ///
    /// Added with [`DeclaredTables::new`], and for a reason worth recording: the
    /// first version of that constructor took a `DeclaredLeapRule` while this
    /// type was still unconstructible from outside, which made the constructor
    /// **decorative**. An extension point is only as usable as the least
    /// constructible type in its signature, and that was found by writing the
    /// test rather than by reading the code.
    ///
    /// `is_convergent` is **declared, not verified**. Whether `p/q` is a
    /// convergent of the tropical year is a fact about a continued fraction this
    /// crate does not have — `ucal-body` derives those and §12 forbids the
    /// dependency. The two shipped rules have theirs checked by the UC-P0
    /// harness against Appendix I.1; a downstream rule is taken at its word, and
    /// that is the difference between a shipped calendar and a declared one.
    ///
    /// `UCAL-E0018` for a cycle of zero years, which is not a rule.
    pub fn new(numerator: u32, denominator: u32, is_convergent: bool) -> Result<DeclaredLeapRule> {
        if denominator == 0 {
            return Err(TimeError::with_context(
                Code::E0018,
                "an intercalation rule needs a cycle: the denominator is years per cycle",
            ));
        }
        Ok(DeclaredLeapRule {
            numerator,
            denominator,
            is_convergent,
        })
    }
}

/// A discontinuity in a legacy calendar's history.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Discontinuity {
    /// What happened, in one line.
    pub description: &'static str,
    /// The last date under the old rule.
    pub last_old: (i64, u8, u8),
    /// The first date under the new rule.
    pub first_new: (i64, u8, u8),
    /// How many days the labelling skipped.
    pub days_skipped: u8,
}

impl Discontinuity {
    /// Declare a historical discontinuity in a calendar's labelling.
    ///
    /// Added for the same reason as [`DeclaredLeapRule::new`]: it appears in
    /// [`DeclaredTables::new`]'s signature, so without it that constructor could
    /// not express a calendar that has one — which is most of the interesting
    /// ones, since a legacy calendar without a reform is a legacy calendar
    /// nobody argued about.
    ///
    /// `UCAL-E0018` for a description that says nothing, or a skip of no days.
    /// The dates are **not** checked against each other: this crate can tell
    /// whether they are well-formed labels in some calendar, and cannot tell
    /// whether *this* calendar's reform actually spanned them, because that is a
    /// fact about the reform and not about arithmetic.
    pub fn new(
        description: &'static str,
        last_old: (i64, u8, u8),
        first_new: (i64, u8, u8),
        days_skipped: u8,
    ) -> Result<Discontinuity> {
        if description.trim().is_empty() {
            return Err(TimeError::with_context(
                Code::E0018,
                "a discontinuity is a historical event and needs saying what it was",
            ));
        }
        if days_skipped == 0 {
            return Err(TimeError::with_context(
                Code::E0018,
                "a discontinuity that skips no days is not one",
            ));
        }
        Ok(Discontinuity {
            description,
            last_old,
            first_new,
            days_skipped,
        })
    }
}

/// The declared table content of a legacy calendar (§8.6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct DeclaredTables {
    /// Month lengths in a common year. Irregular, for historical reasons.
    pub month_lengths: [u8; 12],
    /// Days in the week. Seven, with no astronomical period behind it.
    pub week_length: u8,
    /// The intercalation rule.
    pub leap_rule: DeclaredLeapRule,
    /// Any historical discontinuity.
    pub discontinuity: Option<Discontinuity>,
    /// Explicit statements of what in this calendar is arbitrary (§8.6).
    pub arbitrary: &'static [&'static str],
}

impl DeclaredTables {
    /// Declare a legacy calendar's tables.
    ///
    /// # Why this exists
    ///
    /// `DeclaredTables` is `#[non_exhaustive]` and had no constructor, so a
    /// downstream `LegacyCalendar` could not build one and had to return a
    /// shipped calendar's. §8.6 keeps legacy calendars "for interoperation";
    /// until now that admitted no calendar this crate does not already ship,
    /// which X3 found and recorded.
    ///
    /// # Why it returns a `Result`
    ///
    /// Three of the fields carry obligations that a struct literal cannot state
    /// and this constructor can:
    ///
    /// - **The months must make a common year.** Twelve lengths summing to 365.
    ///   A table whose months do not add up is not a calendar, and every
    ///   `fields`/`instant` round-trip in this module assumes they do.
    /// - **A week must have days in it.** Zero would make weekday arithmetic
    ///   divide by zero.
    /// - **The intercalation must have a cycle.** A denominator of zero is not a
    ///   rule.
    /// - **`arbitrary` must not be empty.** This is §8.6's actual requirement
    ///   and the reason legacy calendars are tolerable at all: they are kept as
    ///   *declared tables* whose arbitrariness is stated, not derived from
    ///   anything. A legacy calendar that declares nothing arbitrary is
    ///   claiming to be derived, and it is not.
    ///
    /// `UCAL-E0018` for all four — a value supplied for a field that cannot take
    /// it (D-A24).
    pub fn new(
        month_lengths: [u8; 12],
        week_length: u8,
        leap_rule: DeclaredLeapRule,
        discontinuity: Option<Discontinuity>,
        arbitrary: &'static [&'static str],
    ) -> Result<DeclaredTables> {
        let days: u32 = month_lengths.iter().map(|d| u32::from(*d)).sum();
        if days != 365 {
            return Err(TimeError::with_context(
                Code::E0018,
                "a common year's months must sum to 365 days; every round-trip in \
                 this module assumes it",
            ));
        }
        if week_length == 0 {
            return Err(TimeError::with_context(
                Code::E0018,
                "a week with no days in it makes weekday arithmetic divide by zero",
            ));
        }
        if leap_rule.denominator == 0 {
            return Err(TimeError::with_context(
                Code::E0018,
                "an intercalation rule needs a cycle: the denominator is years per cycle",
            ));
        }
        if arbitrary.is_empty() {
            return Err(TimeError::with_context(
                Code::E0018,
                "§8.6 keeps a legacy calendar as declared tables whose arbitrariness \
                 is stated. One that declares nothing arbitrary is claiming to be \
                 derived, and it is not",
            ));
        }
        Ok(DeclaredTables {
            month_lengths,
            week_length,
            leap_rule,
            discontinuity,
            arbitrary,
        })
    }
}

/// A civil label produced by a legacy calendar.
///
/// Deliberately has no [`core::fmt::Display`]. §6.6 requires every local calendar
/// rendering to carry its id and kind, so the only way to a string is
/// [`LegacyCalendar::render`], which returns a [`Qualified`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct LegacyFields {
    /// Astronomical year numbering: `0` is 1 BC (§2.5).
    pub year: i64,
    /// Month, 1-12.
    pub month: u8,
    /// Day of month.
    pub day: u8,
    /// Hour, 0-23.
    pub hour: u8,
    /// Minute, 0-59.
    pub minute: u8,
    /// Second, 0-60 (60 during a leap second, §14.2).
    pub second: u8,
    /// Fractional second, as rendered.
    pub sub: SubSecond,
    /// Day of the week, 0 = Monday. Declared table content: the seven-day week
    /// corresponds to no period of any body.
    pub weekday: u8,
    /// Which scale the label is in.
    pub scale: Scale,
    /// Whether the rendering discarded detail (Rule R).
    pub lossy: bool,
}

/// A calendar preserved for interoperation, outside Rule K (§8.6).
///
/// Note what this trait does **not** have: any method returning a bare string,
/// and any relationship to a derived calendar. Both omissions are deliberate.
pub trait LegacyCalendar: CalendarIdentity {
    /// The declared tables. There is no derivation to inspect, only data.
    fn tables(&self) -> &'static DeclaredTables;

    /// Where the tables come from.
    fn citation(&self) -> Citation;

    /// Which underlying civil calendar the arithmetic uses.
    fn civil(&self) -> CivilCalendar;

    /// Decompose an instant into this calendar's fields.
    fn fields(&self, t: &Instant<UC1>, scale: Scale, digits: u8, rounding: Rounding)
        -> Result<LegacyFields>;

    /// Recompose an instant from this calendar's fields.
    fn instant(&self, f: &LegacyFields) -> Result<Instant<UC1>>;

    /// Render, always qualified (§6.6).
    ///
    /// The return type is what enforces the rule: there is no way to obtain the
    /// string without the `earth-civil:` prefix, because `LegacyFields` itself
    /// cannot be displayed.
    #[cfg(feature = "alloc")]
    fn render<'a>(
        &'a self,
        t: &Instant<UC1>,
        scale: Scale,
        digits: u8,
        rounding: Rounding,
    ) -> Result<Qualified<'a, String>> {
        let f = self.fields(t, scale, digits, rounding)?;
        Ok(CalendarQualifier::legacy(self.id()).attach(format_fields(&f, digits)))
    }
}

/// ISO-8601-ish rendering of a legacy label. Never reachable unqualified.
#[cfg(feature = "alloc")]
fn format_fields(f: &LegacyFields, digits: u8) -> String {
    use alloc::format;
    let frac = if digits == 0 {
        String::new()
    } else {
        format!(".{}", f.sub.render(digits))
    };
    let suffix = match f.scale {
        Scale::Utc => "Z",
        Scale::Tai => " TAI",
        Scale::Tt => " TT",
    };
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}{}{}",
        f.year, f.month, f.day, f.hour, f.minute, f.second, frac, suffix
    )
}

/// Day of the week for a day count, 0 = Monday.
///
/// `0000-01-01` proleptic Gregorian was a Saturday, which fixes the phase. The
/// week is declared table content: it tracks no period of any body, and its
/// seven-day length is the clearest single example of why this calendar is
/// legacy rather than derived.
pub const fn weekday_from_days(days_from_origin: i64) -> u8 {
    // 0000-01-01 Gregorian is a Saturday = index 5 with Monday = 0.
    (days_from_origin + 5).rem_euclid(7) as u8
}

/// Names of the days, in the `weekday_from_days` order.
pub const WEEKDAY_NAMES: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

// ---------------------------------------------------------------------------
// Proleptic Gregorian
// ---------------------------------------------------------------------------

/// The proleptic Gregorian calendar. Legacy (§8.6, D-18).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Gregorian;

static GREGORIAN_TABLES: DeclaredTables = DeclaredTables {
    month_lengths: [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    week_length: 7,
    leap_rule: DeclaredLeapRule {
        numerator: 97,
        denominator: 400,
        // Appendix I.1: 97/400 does not appear as a convergent at any depth.
        is_convergent: false,
    },
    discontinuity: Some(Discontinuity {
        description:
            "the Gregorian reform: ten days were removed from the calendar so that \
             the equinox returned to its Nicaean date",
        last_old: (1582, 10, 4),
        first_new: (1582, 10, 15),
        days_skipped: 10,
    }),
    arbitrary: &[
        "month lengths are irregular (31/28/31/30/...) for historical reasons and \
         correspond to no period of any body",
        "the seven-day week has no astronomical period behind it at all",
        "the 97/400 intercalation rule was chosen, not derived: Appendix I.1 shows \
         it is not a continued-fraction convergent of the tropical year at any \
         depth, and that 8/33 is more accurate with a denominator twelve times \
         smaller",
        "the 1582 reform is a discontinuity in the labelling, not in time",
    ],
};

impl CalendarIdentity for Gregorian {
    fn id(&self) -> &str {
        "earth-civil"
    }
    fn kind(&self) -> Kind {
        Kind::Legacy
    }
}

impl LegacyCalendar for Gregorian {
    fn tables(&self) -> &'static DeclaredTables {
        &GREGORIAN_TABLES
    }
    fn citation(&self) -> Citation {
        Citation::new(
        "Inter gravissimas (1582); ISO 8601 for the proleptic extension",
        None,
    )
    }
    fn civil(&self) -> CivilCalendar {
        CivilCalendar::Gregorian
    }
    fn fields(
        &self,
        t: &Instant<UC1>,
        scale: Scale,
        digits: u8,
        rounding: Rounding,
    ) -> Result<LegacyFields> {
        legacy_fields(CivilCalendar::Gregorian, t, scale, digits, rounding)
    }
    fn instant(&self, f: &LegacyFields) -> Result<Instant<UC1>> {
        legacy_instant(CivilCalendar::Gregorian, f)
    }
}

// ---------------------------------------------------------------------------
// Proleptic Julian
// ---------------------------------------------------------------------------

/// The proleptic Julian calendar. Legacy (§8.6, D-18).
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub struct Julian;

static JULIAN_TABLES: DeclaredTables = DeclaredTables {
    month_lengths: [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31],
    week_length: 7,
    leap_rule: DeclaredLeapRule {
        numerator: 1,
        denominator: 4,
        // 1/4 *is* convergent 1 of the tropical year (Appendix I.1) — the Julian
        // rule is the one piece of legacy content that a derivation would also
        // have produced.
        is_convergent: true,
    },
    discontinuity: None,
    arbitrary: &[
        "month lengths are irregular (31/28/31/30/...) for historical reasons and \
         correspond to no period of any body",
        "the seven-day week has no astronomical period behind it at all",
        "the 1/4 intercalation rule is convergent 1 of the tropical year \
         (Appendix I.1), so this rule alone is not arbitrary — a Rule K derivation \
         would have produced it as its first approximation",
        "no discontinuity: the proleptic Julian calendar runs uniformly",
    ],
};

impl CalendarIdentity for Julian {
    fn id(&self) -> &str {
        "earth-julian"
    }
    fn kind(&self) -> Kind {
        Kind::Legacy
    }
}

impl LegacyCalendar for Julian {
    fn tables(&self) -> &'static DeclaredTables {
        &JULIAN_TABLES
    }
    fn citation(&self) -> Citation {
        Citation::new(
        "the Julian reform of 46 BC; proleptic extension by convention",
        None,
    )
    }
    fn civil(&self) -> CivilCalendar {
        CivilCalendar::Julian
    }
    fn fields(
        &self,
        t: &Instant<UC1>,
        scale: Scale,
        digits: u8,
        rounding: Rounding,
    ) -> Result<LegacyFields> {
        legacy_fields(CivilCalendar::Julian, t, scale, digits, rounding)
    }
    fn instant(&self, f: &LegacyFields) -> Result<Instant<UC1>> {
        legacy_instant(CivilCalendar::Julian, f)
    }
}

// ---------------------------------------------------------------------------
// shared implementation
// ---------------------------------------------------------------------------

fn legacy_fields(
    cal: CivilCalendar,
    t: &Instant<UC1>,
    scale: Scale,
    digits: u8,
    rounding: Rounding,
) -> Result<LegacyFields> {
    let f: CivilFields = to_civil(t, scale, digits, rounding, cal)?;
    let days = crate::calendar::days_from_civil(f.year, f.month, f.day, cal);
    Ok(LegacyFields {
        year: f.year,
        month: f.month,
        day: f.day,
        hour: f.hour,
        minute: f.minute,
        second: f.second,
        sub: f.sub,
        weekday: weekday_from_days(days),
        scale: f.scale,
        lossy: f.lossy,
    })
}

fn legacy_instant(cal: CivilCalendar, f: &LegacyFields) -> Result<Instant<UC1>> {
    // The weekday is derived from the date, never an input: accepting one would
    // admit a self-inconsistent label.
    let expected = weekday_from_days(crate::calendar::days_from_civil(f.year, f.month, f.day, cal));
    if f.weekday != expected {
        return Err(TimeError::with_context(
            Code::E0041,
            "weekday does not match the date; it is derived, not an input",
        ));
    }
    from_civil(
        f.year, f.month, f.day, f.hour, f.minute, f.second, f.sub, f.scale, cal,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::calendar::{is_leap, month_length};
    use ucal_core::qualified::require_derived;

    fn at(y: i64, m: u8, d: u8) -> Instant<UC1> {
        from_civil(
            y,
            m,
            d,
            0,
            0,
            0,
            SubSecond::zero(),
            Scale::Tt,
            CivilCalendar::Gregorian,
        )
        .unwrap()
    }

    // ---- §8.6: declared, and declaring what is arbitrary ----

    #[test]
    fn every_legacy_calendar_declares_its_arbitrary_content() {
        // §8.6 names four categories that must be declared explicitly.
        for tables in [Gregorian.tables(), Julian.tables()] {
            let joined = tables.arbitrary.join(" ").to_lowercase();
            assert!(joined.contains("month lengths are irregular"), "month lengths");
            assert!(joined.contains("seven-day week"), "the week");
            assert!(joined.contains("intercalation rule"), "the leap rule");
            assert!(
                joined.contains("discontinuity"),
                "the discontinuity, or its absence"
            );
            assert!(tables.arbitrary.len() >= 4);
        }
    }

    #[test]
    fn the_gregorian_leap_rule_is_declared_non_convergent() {
        // The claim is checkable, not decorative: Appendix I.1 is reproduced in
        // `ucal-core`, and 97/400 is absent from Earth's convergents at every
        // depth. Here the calendar simply admits it.
        let g = Gregorian.tables().leap_rule;
        assert_eq!((g.numerator, g.denominator), (97, 400));
        assert!(!g.is_convergent, "97/400 is not a convergent (Appendix I.1)");

        // The Julian rule is the exception: 1/4 *is* convergent 1.
        let j = Julian.tables().leap_rule;
        assert_eq!((j.numerator, j.denominator), (1, 4));
        assert!(j.is_convergent);
    }

    #[test]
    fn the_declared_leap_rules_match_the_arithmetic() {
        // Over 400 Gregorian years there must be exactly 97 leap days, and over
        // 4 Julian years exactly 1 — the tables and the implementation agree.
        let g = (0..400).filter(|y| is_leap(*y, CivilCalendar::Gregorian)).count();
        assert_eq!(g as u32, Gregorian.tables().leap_rule.numerator);
        let j = (0..4).filter(|y| is_leap(*y, CivilCalendar::Julian)).count();
        assert_eq!(j as u32, Julian.tables().leap_rule.numerator);
    }

    #[test]
    fn month_length_tables_match_the_arithmetic() {
        for (cal, tables) in [
            (CivilCalendar::Gregorian, Gregorian.tables()),
            (CivilCalendar::Julian, Julian.tables()),
        ] {
            for m in 1..=12u8 {
                // 2023 is a common year in both calendars.
                assert_eq!(
                    month_length(2023, m, cal).unwrap() as u8,
                    tables.month_lengths[(m - 1) as usize],
                    "month {m}"
                );
            }
            // ...and the irregularity is real: no two consecutive months follow a
            // rule, which is the point of declaring it arbitrary.
            assert_eq!(tables.month_lengths[0], 31);
            assert_eq!(tables.month_lengths[1], 28);
        }
    }

    #[test]
    fn the_gregorian_discontinuity_is_declared() {
        let d = Gregorian.tables().discontinuity.unwrap();
        assert_eq!(d.last_old, (1582, 10, 4));
        assert_eq!(d.first_new, (1582, 10, 15));
        assert_eq!(d.days_skipped, 10);
        // The Julian calendar has none.
        assert!(Julian.tables().discontinuity.is_none());
    }

    // ---- §6.6: the qualifier cannot be omitted ----

    #[test]
    fn rendering_is_always_qualified() {
        let t = at(2026, 7, 29);
        let r = Gregorian
            .render(&t, Scale::Tt, 0, Rounding::Trunc)
            .unwrap();
        assert_eq!(r.to_string(), "earth-civil: 2026-07-29T00:00:00 TT");
        assert_eq!(r.qualifier().kind(), Kind::Legacy);
        assert_eq!(r.qualifier().id(), "earth-civil");
        // No anchor revision: a legacy calendar has no anchor.
        assert_eq!(r.qualifier().revision(), None);

        let r = Julian.render(&t, Scale::Tt, 0, Rounding::Trunc).unwrap();
        assert!(r.to_string().starts_with("earth-julian: "));
    }

    #[test]
    fn every_legacy_rendering_carries_w0005() {
        let t = at(2026, 7, 29);
        for c in [&Gregorian as &dyn LegacyCalendar, &Julian] {
            let r = c.render(&t, Scale::Tt, 0, Rounding::Trunc).unwrap();
            assert_eq!(
                r.warning(),
                Some(ucal_core::Warning::W0005),
                "{} must carry W0005",
                c.id()
            );
            // And the string always begins with the id.
            assert!(r.to_string().starts_with(c.id()));
        }
    }

    // ---- Rule K.6 / UCAL-E0065 ----

    #[test]
    fn a_legacy_calendar_is_refused_where_a_derivation_is_required() {
        for c in [&Gregorian as &dyn LegacyCalendar, &Julian] {
            // Trait upcasting: `LegacyCalendar: CalendarIdentity`.
            let e = require_derived(c as &dyn ucal_core::CalendarIdentity).unwrap_err();
            assert_eq!(e.code, Code::E0065, "{}", c.id());
        }
    }

    // ---- round trips ----

    #[test]
    fn fields_and_instant_are_inverse() {
        for c in [&Gregorian as &dyn LegacyCalendar, &Julian] {
            for (y, m, d) in [(1, 1, 1), (1582, 10, 15), (1970, 1, 1), (2026, 7, 29)] {
                let t = at(y, m, d);
                let f = c.fields(&t, Scale::Tt, 0, Rounding::Trunc).unwrap();
                assert_eq!(c.instant(&f).unwrap(), t, "{} at {y}-{m}-{d}", c.id());
            }
        }
    }

    #[test]
    fn the_two_calendars_disagree_by_construction() {
        // Same instant, two legacy calendars, two different labels — which is
        // exactly why the qualifier is mandatory.
        let t = at(2026, 7, 29);
        let g = Gregorian.render(&t, Scale::Tt, 0, Rounding::Trunc).unwrap();
        let j = Julian.render(&t, Scale::Tt, 0, Rounding::Trunc).unwrap();
        assert_ne!(g.value(), j.value());
        assert!(g.to_string().contains("2026-07-29"));
        // By 2026 the Julian calendar has drifted thirteen days behind.
        assert!(j.to_string().contains("2026-07-16"));
    }

    #[test]
    fn weekday_is_derived_and_checked() {
        // 2026-07-29 is a Wednesday.
        let t = at(2026, 7, 29);
        let f = Gregorian.fields(&t, Scale::Tt, 0, Rounding::Trunc).unwrap();
        assert_eq!(WEEKDAY_NAMES[f.weekday as usize], "Wednesday");
        // 1970-01-01 was a Thursday.
        let f = Gregorian
            .fields(&at(1970, 1, 1), Scale::Tt, 0, Rounding::Trunc)
            .unwrap();
        assert_eq!(WEEKDAY_NAMES[f.weekday as usize], "Thursday");

        // A weekday inconsistent with the date is refused, because it is derived
        // rather than an input.
        let mut bad = f;
        bad.weekday = (bad.weekday + 1) % 7;
        assert_eq!(Gregorian.instant(&bad).unwrap_err().code, Code::E0041);
    }

    #[test]
    fn the_week_cycles_without_reference_to_anything() {
        // Seven consecutive days give seven distinct weekdays and then repeat —
        // a cycle with no astronomical period behind it, as declared.
        let base = crate::calendar::days_from_gregorian(2026, 7, 29);
        let seen: alloc::vec::Vec<u8> = (0..7).map(|i| weekday_from_days(base + i)).collect();
        let mut sorted = seen.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(sorted.len(), 7);
        assert_eq!(weekday_from_days(base + 7), weekday_from_days(base));
    }
}
