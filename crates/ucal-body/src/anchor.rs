//! Anchors (Rule J) — the one empirical component of a calendar.
//!
//! # What an anchor is for
//!
//! Rule K derives everything about a calendar from the tick, the datum and a
//! body's periods: the units, the intercalation, the grouping. It cannot derive
//! **phase**. The datum and a tick give elapsed intervals; they cannot say where a
//! planet was pointing, and saying so would require ephemerides (N6).
//!
//! N15 states the concession plainly: *phase is not derived*. §9.1 prices it:
//! "one cited, interval-valued constant per body, with the same status the datum's
//! own physical identification has under Rule Q.3."
//!
//! # The five obligations, and how each is enforced
//!
//! | Rule J | requirement | enforcement |
//! |---|---|---|
//! | J.1 | `phase` names a physical event **of this body** | the enum admits only body events; `Custom` is screened |
//! | J.2 | `window` required, must contain `tick`; uncertainty propagates | checked in the constructor; [`Anchor::elapsed_at`] returns a [`Span`] |
//! | J.3 | no anchor ⇒ no local fields | `UCAL-E0062`, never a guess or a fallback |
//! | J.4 | phase not evaluable for the body ⇒ `UCAL-E0063` | [`Anchor::check_evaluable`] |
//! | J.5 | anchors are versioned data | `revision`, carried into every rendering (§6.6) |
//!
//! # The definition/determination split
//!
//! Rule J.1 forbids an anchor being **defined** by another body's calendar, but
//! Rule Y explicitly permits its **determination** to cite an observation
//! timestamped in any scale. That split is what makes J.1 satisfiable at all:
//! Earth's anchor is *defined* as mean solar midnight at its own prime meridian,
//! and *determined* from a published ΔT. The definition contains no foreign
//! reference; the determination is metrology.

#[cfg(feature = "alloc")]
use alloc::string::String;

use ucal_core::{Citation, Code, Delta, Instant, Span, TimeError, Window, UC1};

type Result<T> = core::result::Result<T, TimeError>;

/// A reference meridian on a body.
///
/// A meridian is a choice, not a discovery, so it is named and cited rather than
/// computed. Earth's is Greenwich by treaty; Mars's is Airy-0, the crater chosen
/// to continue a nineteenth-century convention.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Meridian {
    /// The meridian's name, e.g. `"greenwich"` or `"airy-0"`.
    pub name: &'static str,
    /// Who fixed it.
    pub citation: Citation,
}

impl Meridian {
    /// Name a meridian and say who fixed it.
    ///
    /// Added in 1.5.0, because the struct is `#[non_exhaustive]` and had no
    /// constructor — so it could be built inside this crate and nowhere else.
    /// §15.1 requires anchor **files**, and a loader outside `ucal-body` could
    /// not construct one of these at all. That was invisible for as long as
    /// nobody tried: the body-file loader landed in 1.4.0 and worked, because
    /// `Body`, `Measured` and `RatedParam` all have public constructors and
    /// these two did not.
    pub const fn new(name: &'static str, citation: Citation) -> Meridian {
        Meridian { name, citation }
    }
}

/// What physical event of the body fixes where local counting begins (Rule J.1).
///
/// D-15 makes this an open enum: bodies vary more than a closed set can
/// anticipate. Every variant names an event **of the body itself** — a rotation
/// reaching a meridian, an orbit reaching an equinox or perihelion. None of them
/// can name another body's calendar, which is J.1 discharged structurally for
/// everything but `Custom`.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum PhaseDefinition {
    /// The body's rotation brings a stated meridian to mean solar midnight.
    MeanSolarMidnight {
        /// Which meridian.
        meridian: Meridian,
    },
    /// The body's orbit reaches the northward equinox.
    NorthwardEquinox,
    /// The body's orbit reaches the southward equinox.
    SouthwardEquinox,
    /// The body's orbit reaches perihelion.
    Perihelion,
    /// Anything else. D-15 requires a citation, so a custom phase cannot be
    /// asserted without a source.
    Custom {
        /// What the event is.
        description: &'static str,
        /// Where the definition comes from.
        citation: Citation,
    },
}

/// Phrases that would make a phase definition refer to another body's calendar,
/// clock or epoch — which Rule J.1 forbids.
///
/// A partial defence, and honest about being one: a determined author can always
/// smuggle a foreign reference past a word list. What this catches is the
/// *accidental* case, which is the likely one — reaching for a familiar epoch
/// because it is the handiest number to hand. The structural defence is that the
/// other variants cannot express a foreign reference at all.
const FOREIGN_REFERENCES: &[&str] = &[
    "unix epoch",
    "gregorian",
    "julian day",
    "julian date",
    "j2000",
    "utc",
    "tai",
    "gps epoch",
    "earth calendar",
    "civil calendar",
];

impl PhaseDefinition {
    /// A short label, for rendering and for `ucal cal anchor`.
    pub fn label(&self) -> &'static str {
        match self {
            PhaseDefinition::MeanSolarMidnight { .. } => "mean solar midnight",
            PhaseDefinition::NorthwardEquinox => "northward equinox",
            PhaseDefinition::SouthwardEquinox => "southward equinox",
            PhaseDefinition::Perihelion => "perihelion",
            PhaseDefinition::Custom { description, .. } => description,
        }
    }

    /// Whether this phase depends on the body's rotation, its orbit, or both.
    ///
    /// Used by [`Anchor::check_evaluable`]: a phase that needs a parameter the
    /// body does not declare is `UCAL-E0063`.
    pub fn needs_rotation(&self) -> bool {
        matches!(self, PhaseDefinition::MeanSolarMidnight { .. })
    }

    /// Whether this phase depends on the body's orbit.
    pub fn needs_orbit(&self) -> bool {
        matches!(
            self,
            PhaseDefinition::NorthwardEquinox
                | PhaseDefinition::SouthwardEquinox
                | PhaseDefinition::Perihelion
        )
    }

    /// Reject a definition that refers to another body's calendar (Rule J.1).
    pub fn check_is_a_body_event(&self) -> Result<()> {
        let PhaseDefinition::Custom { description, .. } = self else {
            // The other variants name body events by construction.
            return Ok(());
        };
        let lowered = to_lower(description);
        for foreign in FOREIGN_REFERENCES {
            if lowered.contains(foreign) {
                return Err(TimeError::with_context(
                    Code::E0063,
                    "a phase definition must name a physical event of the body \
                     itself (Rule J.1); its *determination* may cite an \
                     observation in any scale, but its definition may not",
                ));
            }
        }
        Ok(())
    }
}

#[cfg(feature = "alloc")]
fn to_lower(s: &str) -> String {
    s.to_lowercase()
}

#[cfg(not(feature = "alloc"))]
fn to_lower(s: &str) -> &str {
    s
}

/// How an anchor's tick value was established (§9.4).
///
/// Informative, but required: an anchor whose determination is unrecorded cannot
/// be checked, only believed. Rule Y permits the determination to cite an
/// observation in any scale — which is exactly what these do.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Determination {
    /// How the instant was obtained, in a sentence.
    pub method: &'static str,
    /// The source of the observation or ephemeris.
    pub citation: Citation,
    /// What dominates the uncertainty, and why the window is the width it is.
    pub uncertainty_note: &'static str,
}

impl Determination {
    /// Record how an anchor instant was obtained.
    ///
    /// All three arguments, because Rule J.3 makes all three obligations and a
    /// constructor that let one be omitted would be a laxer way of declaring an
    /// anchor than the struct literal it replaces. See [`Meridian::new`] for why
    /// this exists at all.
    pub const fn new(
        method: &'static str,
        citation: Citation,
        uncertainty_note: &'static str,
    ) -> Determination {
        Determination {
            method,
            citation,
            uncertainty_note,
        }
    }
}

/// Where a calendar's local counting begins (Rule J).
///
/// Every field is required. There is no constructor that omits one, which makes
/// Rule J's "MUST be a structure, not a bare number" a property of the type
/// rather than a convention.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Anchor {
    calendar_id: &'static str,
    tick: Instant<UC1>,
    phase: PhaseDefinition,
    method: Determination,
    window: Window<UC1>,
    citation: Citation,
    revision: u32,
}

impl Anchor {
    /// Declare an anchor.
    ///
    /// `UCAL-E0062` if the window does not contain the tick — Rule J.2 requires
    /// it, and a window that excludes its own best estimate is not an uncertainty
    /// but a contradiction. `UCAL-E0063` if the phase names a foreign reference.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        calendar_id: &'static str,
        tick: Instant<UC1>,
        phase: PhaseDefinition,
        method: Determination,
        window: Window<UC1>,
        citation: Citation,
        revision: u32,
    ) -> Result<Anchor> {
        phase.check_is_a_body_event()?;
        if !window.contains(&tick) {
            return Err(TimeError::with_context(
                Code::E0062,
                "an anchor's uncertainty window must contain its own tick value \
                 (Rule J.2)",
            ));
        }
        if revision == 0 {
            return Err(TimeError::with_context(
                Code::E0062,
                "anchor revisions start at 1; revision 0 would make an \
                 unversioned anchor indistinguishable from a versioned one \
                 (Rule J.5)",
            ));
        }
        Ok(Anchor {
            calendar_id,
            tick,
            phase,
            method,
            window,
            citation,
            revision,
        })
    }

    /// Which calendar this anchors.
    pub fn calendar_id(&self) -> &'static str {
        self.calendar_id
    }

    /// The best estimate of where local counting begins.
    pub fn tick(&self) -> &Instant<UC1> {
        &self.tick
    }

    /// The physical event that defines the phase.
    pub fn phase(&self) -> &PhaseDefinition {
        &self.phase
    }

    /// How the tick value was established.
    pub fn method(&self) -> &Determination {
        &self.method
    }

    /// The uncertainty window. Always contains [`Anchor::tick`].
    pub fn window(&self) -> &Window<UC1> {
        &self.window
    }

    /// The citation for the anchor as a whole.
    pub fn citation(&self) -> Citation {
        self.citation
    }

    /// The revision (Rule J.5). Renderings carry it, so values from different
    /// determinations are never silently compared.
    pub fn revision(&self) -> u32 {
        self.revision
    }

    /// How wide the anchor's uncertainty is.
    pub fn uncertainty(&self) -> Delta {
        self.window.width()
    }

    /// Whether this anchor's phase can be evaluated for a body's parameters
    /// (Rule J.4).
    ///
    /// `UCAL-E0063` when the phase needs something the body does not have — a
    /// mean solar midnight on a body with no declared rotation, say.
    pub fn check_evaluable(&self, body: &crate::body::Body) -> Result<()> {
        if self.phase.needs_rotation() {
            // A body with no meaningful solar day cannot have a solar midnight.
            if body.solar_day().value_at_epoch().is_zero() {
                return Err(TimeError::with_context(
                    Code::E0063,
                    "this phase needs a solar day, which the body does not declare",
                ));
            }
        }
        if self.phase.needs_orbit() && body.orbital_period().value_at_epoch().is_zero() {
            return Err(TimeError::with_context(
                Code::E0063,
                "this phase needs an orbital period, which the body does not declare",
            ));
        }
        Ok(())
    }

    /// Elapsed time from the anchor to an instant, as an **interval** (Rule J.2).
    ///
    /// This is the point at which anchor uncertainty enters a derived calendar,
    /// and the reason it comes back as a [`Span`] rather than a `Delta`: an
    /// instant measured from an uncertain origin is itself uncertain, and typing
    /// it that way is what stops the uncertainty being dropped on the way to
    /// `fields()`.
    ///
    /// The second element reports whether the lower bound was clamped at zero,
    /// which happens when the instant falls inside the anchor's own window — a
    /// genuinely ambiguous case, since local counting may not have begun yet.
    pub fn elapsed_at(&self, t: &Instant<UC1>) -> Result<(Span, bool)> {
        Window::exact(t.clone()).since_window(&self.window)
    }

    /// Whether an instant falls inside the anchor's own uncertainty window.
    ///
    /// Inside it, the sign of the elapsed time is not determined: the calendar may
    /// or may not have begun. A caller that needs a definite answer should widen
    /// its own tolerance rather than pretend otherwise.
    pub fn is_ambiguous_at(&self, t: &Instant<UC1>) -> bool {
        self.window.contains(t)
    }
}

/// The result of looking for a calendar's anchor.
///
/// Rule J.3: "A calendar without an anchor MUST NOT produce local fields —
/// `UCAL-E0062`, not a guess and not a fallback to another body."
///
/// The absence is a *state*, not a failure of the library: Appendix I.6 says a
/// calendar may be complete in units, intercalation and cycles while incomplete
/// in phase, and that the API should represent that explicitly rather than
/// defaulting it away.
pub fn require_anchor<'a>(
    calendar_id: &str,
    anchor: Option<&'a Anchor>,
) -> Result<&'a Anchor> {
    match anchor {
        Some(a) => Ok(a),
        None => {
            let _ = calendar_id;
            Err(TimeError::with_context(
                Code::E0062,
                "this calendar has no anchor, so it cannot produce local fields. \
                 Phase is empirical (N15) and must be determined and cited; it is \
                 never guessed and never borrowed from another body (Rule J.3).",
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchors;
    use crate::data;
    use ucal_core::backend::TickInt;
    use ucal_core::{Profile, Ticks};

    const SRC: Citation = Citation::new("test", None);

    fn tick(offset_seconds: i64) -> Instant<UC1> {
        let base = data::j2000();
        let d = Delta::from_ticks(
            UC1::bridge()
                .ticks
                .try_mul(&<Ticks as TickInt>::from_u64(offset_seconds.unsigned_abs()))
                .unwrap(),
        );
        if offset_seconds < 0 {
            base.checked_sub(&d).unwrap()
        } else {
            base.checked_add(&d).unwrap()
        }
    }

    fn a_window(half_seconds: i64) -> Window<UC1> {
        Window::new(tick(-half_seconds), tick(half_seconds)).unwrap()
    }

    fn a_method() -> Determination {
        Determination {
            method: "test",
            citation: SRC,
            uncertainty_note: "test",
        }
    }

    fn greenwich() -> Meridian {
        Meridian {
            name: "greenwich",
            citation: SRC,
        }
    }

    // ---- Rule J.2: the window is required and must contain the tick ----

    #[test]
    fn an_anchor_window_must_contain_its_own_tick() {
        let phase = PhaseDefinition::MeanSolarMidnight {
            meridian: greenwich(),
        };
        // The tick sits outside its own window: a contradiction, not an
        // uncertainty.
        let e = Anchor::new(
            "test-d",
            tick(1000),
            phase,
            a_method(),
            a_window(10),
            SRC,
            1,
        )
        .unwrap_err();
        assert_eq!(e.code, Code::E0062);
        // Inside, it is fine.
        assert!(Anchor::new("test-d", tick(0), phase, a_method(), a_window(10), SRC, 1).is_ok());
    }

    #[test]
    fn revisions_start_at_one() {
        // Rule J.5: renderings carry the revision so determinations are never
        // silently compared. Revision 0 would be indistinguishable from absent.
        let phase = PhaseDefinition::Perihelion;
        assert!(Anchor::new("t", tick(0), phase, a_method(), a_window(1), SRC, 0).is_err());
        assert!(Anchor::new("t", tick(0), phase, a_method(), a_window(1), SRC, 1).is_ok());
    }

    // ---- Rule J.1: the phase is a body event ----

    #[test]
    fn a_phase_may_not_be_defined_by_a_foreign_calendar() {
        for bad in [
            "the Unix epoch",
            "midnight on the Gregorian new year",
            "J2000.0",
            "00:00 UTC",
            "the GPS epoch",
        ] {
            let phase = PhaseDefinition::Custom {
                description: bad,
                citation: SRC,
            };
            let e = phase.check_is_a_body_event().unwrap_err();
            assert_eq!(e.code, Code::E0063, "{bad} should be refused");
        }
    }

    #[test]
    fn a_phase_naming_a_body_event_is_accepted() {
        for good in [
            "the first perihelion after the northern vernal equinox",
            "sub-solar longitude crossing the reference meridian",
            "aphelion",
        ] {
            let phase = PhaseDefinition::Custom {
                description: good,
                citation: SRC,
            };
            assert!(phase.check_is_a_body_event().is_ok(), "{good}");
        }
        // The non-custom variants name body events by construction.
        for phase in [
            PhaseDefinition::MeanSolarMidnight {
                meridian: greenwich(),
            },
            PhaseDefinition::NorthwardEquinox,
            PhaseDefinition::SouthwardEquinox,
            PhaseDefinition::Perihelion,
        ] {
            assert!(phase.check_is_a_body_event().is_ok());
        }
    }

    // ---- Rule J.3: no anchor, no fields ----

    #[test]
    fn a_calendar_without_an_anchor_is_e0062() {
        let e = require_anchor("titan-d", None).unwrap_err();
        assert_eq!(e.code, Code::E0062);
        assert_eq!(e.code.exit_code(), 7);
        // The message must say what is missing and refuse to substitute.
        let text = alloc::format!("{e}");
        assert!(text.contains("no anchor"));
        assert!(text.contains("never borrowed from another body"));
    }

    #[test]
    fn the_shipped_anchors_are_the_ones_that_could_be_determined() {
        // Earth and Mars have established conventions with published constants.
        assert!(anchors::for_calendar("earth-d").is_some());
        assert!(anchors::for_calendar("mars-d").is_some());
        // Titan has none, and the absence is the correct output (Appendix I.6) —
        // not a placeholder, not a borrowed epoch.
        assert!(anchors::for_calendar("titan-d").is_none());
        assert_eq!(
            require_anchor("titan-d", anchors::for_calendar("titan-d").as_ref())
                .unwrap_err()
                .code,
            Code::E0062
        );
    }

    // ---- Rule J.4 ----

    #[test]
    fn a_phase_the_body_cannot_support_is_e0063() {
        let earth = data::earth();
        let a = anchors::for_calendar("earth-d").unwrap();
        assert!(a.check_evaluable(&earth).is_ok());
    }

    // ---- Rule J.2: uncertainty propagates ----

    #[test]
    fn elapsed_time_from_an_anchor_is_an_interval() {
        let a = anchors::for_calendar("earth-d").unwrap();
        let later = a
            .tick()
            .checked_add(&Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .try_mul(&<Ticks as TickInt>::from_u64(86_400))
                    .unwrap(),
            ))
            .unwrap();
        let (span, clamped) = a.elapsed_at(&later).unwrap();
        assert!(!clamped);
        // The span is at least as wide as the anchor's own uncertainty: an
        // instant measured from an uncertain origin cannot be known better.
        assert_eq!(span.uncertainty(), a.uncertainty());
        assert!(!span.is_exact(), "the result must carry the uncertainty");
    }

    #[test]
    fn an_instant_inside_the_anchor_window_is_ambiguous() {
        let a = anchors::for_calendar("earth-d").unwrap();
        assert!(a.is_ambiguous_at(a.tick()));
        let (span, clamped) = a.elapsed_at(a.tick()).unwrap();
        assert!(clamped, "the lower bound clamps at zero and says so");
        assert!(span.lo().is_zero());
    }

    #[test]
    fn every_shipped_anchor_satisfies_rule_j() {
        for id in anchors::CALENDARS_WITH_ANCHORS {
            let a = anchors::for_calendar(id).expect("listed anchors must exist");
            // J.2: the window contains the tick.
            assert!(a.window().contains(a.tick()), "{id}");
            // J.5: a real revision.
            assert!(a.revision() >= 1, "{id}");
            // J.1: a body event.
            assert!(a.phase().check_is_a_body_event().is_ok(), "{id}");
            // A determination and a citation, both non-empty (§9.4).
            assert!(!a.method().method.is_empty(), "{id}");
            assert!(!a.method().uncertainty_note.is_empty(), "{id}");
            assert!(!a.citation().source.is_empty(), "{id}");
        }
    }
}
