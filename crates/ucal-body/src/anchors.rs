//! Anchor data (Rule J.5) — versioned, cited, and deliberately incomplete.
//!
//! # Why this file is data and not code
//!
//! Rule J.5: "Anchors are versioned data, not code. Re-determination bumps
//! `revision`; renderings carry it so values from different revisions are never
//! silently compared." Everything here is a declaration with a citation and a
//! revision number; nothing is computed.
//!
//! # GE-3, answered
//!
//! > *Can Earth, Mars, and Titan anchors be established to a window narrower than
//! > one local solar day from published ephemerides?*
//! >
//! > Kill criterion: if not, `DerivedFields` windows exceed one day and the
//! > derived calendars are honest but coarse; document the width rather than
//! > narrowing it by assumption.
//!
//! | body | determinable? | window | as a fraction of its solar day |
//! |---|---|---|---|
//! | Earth | yes | ±1 ms | 2.3×10⁻⁸ |
//! | Mars | yes | ±1 s | 2.3×10⁻⁵ |
//! | Titan | **no** | — | — |
//!
//! **The kill criterion is not triggered for Earth or Mars.** Both have
//! established meridian conventions with published constants, and both resolve
//! four to eight orders of magnitude finer than one local day.
//!
//! **Titan has no anchor, and that is the honest output.** There is no
//! established prime meridian convention with a published epoch for Titan
//! comparable to UT1 or Mars24. An anchor could be invented — the mechanism would
//! accept one — but inventing it would be exactly the "narrowing by assumption"
//! GE-3 forbids. Appendix I.6 anticipates this state: a calendar complete in
//! units, intercalation and cycles, and incomplete in phase, with the API saying
//! so (`UCAL-E0062`) rather than defaulting it away.
//!
//! # Why only Earth and Mars, and why that is not Earth privilege
//!
//! Rule K.5 says Earth is an ordinary instance, and it is: the code path here is
//! identical for every body. What differs is that Earth and Mars have had
//! landers, orbiters and centuries of meridian argument, and Titan has not. That
//! is a fact about where the instruments are, not about the mechanism — the same
//! concession Rule Y makes for measurement generally.

use ucal_core::backend::TickInt;
use ucal_core::{Citation, Delta, Instant, Ticks, Window, UC1};

use crate::anchor::{Anchor, Determination, Meridian, PhaseDefinition};

/// Calendars for which an anchor has been determined.
///
/// A calendar absent from this list is complete in everything Rule K derives and
/// incomplete in phase; asking it for local fields is `UCAL-E0062`.
pub const CALENDARS_WITH_ANCHORS: &[&str] = &["earth-d", "mars-d"];

const IERS: Citation = Citation::new(
        "IERS Conventions (2010) and the IERS Earth Orientation Centre's \
             published Delta-T series",
        Some("https://www.iers.org/"),
    );

const BIPM_TREATY: Citation = Citation::new(
        "International Meridian Conference (1884); UT1 is mean solar time at \
             the prime meridian by definition",
        None,
    );

const MARS24: Citation = Citation::new(
        "Allison, M. and McEwen, M. (2000), A post-Pathfinder evaluation of \
             areocentric solar coordinates with improved timing recipes for \
             Mars seasonal/diurnal climate studies, Planet. Space Sci. 48, 215",
        Some("doi:10.1016/S0032-0633(99)00092-6"),
    );

const AIRY_0: Citation = Citation::new(
        "de Vaucouleurs, Davies & Sturms (1973), Mariner 9 areographic \
             coordinate system; Airy-0 fixes the Mars prime meridian",
        None,
    );

fn ticks(decimal: &str) -> Instant<UC1> {
    Instant::from_ticks(
        <Ticks as TickInt>::from_dec_str(decimal).expect("anchor tick within the domain"),
    )
    .expect("anchor tick within the domain")
}

/// A window of `± seconds x 10^-scale` about a tick.
fn window_about(centre: &Instant<UC1>, mantissa: u64, scale: u32) -> Window<UC1> {
    let mut den = <Ticks as TickInt>::one();
    for _ in 0..scale {
        den = den
            .try_mul(&<Ticks as TickInt>::from_u64(10))
            .expect("within the domain");
    }
    let (half, rem) = <UC1 as ucal_core::Profile>::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(mantissa))
        .expect("within the domain")
        .quot_rem(&den);
    debug_assert!(rem.is_zero_ticks(), "anchor windows must be exact in ticks");
    let d = Delta::from_ticks(half);
    Window::new(
        centre.checked_sub(&d).expect("after the datum"),
        centre.checked_add(&d).expect("within the domain"),
    )
    .expect("lo <= hi")
}

/// The Earth anchor, revision 1.
///
/// **Phase**: mean solar midnight at the prime meridian. This is a physical event
/// of Earth — its rotation bringing Greenwich to face away from the Sun — and
/// names no other body's calendar, as Rule J.1 requires.
///
/// **Determination**: UT1 *is* mean solar time at the prime meridian, by
/// definition, so the phase instant is `2000-01-01T00:00:00 UT1`. Converting to
/// absolute time needs ΔT, which is observed rather than computed:
/// `TT = UT1 + ΔT`, and ΔT at 2000.0 is 63.8285 s. The date is chosen near J2000
/// because that is where ΔT is best determined.
///
/// **Window**: ±1 ms, dominated by the ΔT series' own resolution. That is
/// 2.3×10⁻⁸ of a solar day — GE-3's criterion met with eight orders to spare.
///
/// Note the definition/determination split Rule J.1 turns on: the *definition*
/// mentions only Earth's rotation and its own meridian; the *determination* cites
/// a value published on a foreign timescale, which Rule Y expressly permits.
pub fn earth_d() -> Anchor {
    let tick = ticks("8070205173569172848597429796163475680530466139316558837890625");
    let window = window_about(&tick, 1, 3); // ±0.001 s
    Anchor::new(
        "earth-d",
        tick,
        PhaseDefinition::MeanSolarMidnight {
            meridian: Meridian {
                name: "greenwich",
                citation: BIPM_TREATY,
            },
        },
        Determination {
            method: "mean solar midnight at the prime meridian on 2000-01-01, i.e. \
                     00:00:00 UT1, converted through TT = UT1 + Delta-T with \
                     Delta-T(2000.0) = 63.8285 s",
            citation: IERS,
            uncertainty_note: "dominated by the resolution of the published \
                               Delta-T series, which is quoted to 0.0001 s near \
                               2000.0; the window is widened to 1 ms to cover the \
                               series' own stated scatter",
        },
        window,
        IERS,
        1,
    )
    .expect("the shipped Earth anchor must satisfy Rule J")
}

/// The Mars anchor, revision 1.
///
/// **Phase**: mean solar midnight at Airy-0, the crater that fixes Mars's prime
/// meridian. Again a physical event of Mars itself.
///
/// **Determination**: Mars Sol Date 0, from the Mars24 recipe of Allison and
/// McEwen (2000):
///
/// ```text
/// MSD = (JD_TT − 2451549.5) / 1.0274912517 + 44796.0 − 0.0009626
/// ```
///
/// Setting `MSD = 0` gives `JD_TT = 2405522.0028779…`, which is the tick value
/// declared here.
///
/// **Window**: ±1 s. The sol-length constant carries eleven significant figures,
/// so extrapolating back 44796 sols accumulates roughly 0.4 s of error; the
/// window is rounded up from that. It is 2.3×10⁻⁵ of a sol — four orders inside
/// GE-3's criterion.
///
/// This anchor matters beyond Mars: it is the demonstration that the mechanism
/// works on a body that is not Earth, which is Rule K's entire claim.
pub fn mars_d() -> Anchor {
    let tick = ticks("8070205099813623989919952358914893463523316507316558837890625");
    let window = window_about(&tick, 1, 0); // ±1 s
    Anchor::new(
        "mars-d",
        tick,
        PhaseDefinition::MeanSolarMidnight {
            meridian: Meridian {
                name: "airy-0",
                citation: AIRY_0,
            },
        },
        Determination {
            method: "Mars Sol Date 0 under the Mars24 recipe, \
                     MSD = (JD_TT - 2451549.5)/1.0274912517 + 44796.0 - 0.0009626, \
                     solved for MSD = 0 to give JD_TT = 2405522.0028779",
            citation: MARS24,
            uncertainty_note: "dominated by the eleven-significant-figure sol \
                               length: extrapolating 44796 sols back from the \
                               fit's epoch accumulates about 0.4 s, rounded up to \
                               1 s here rather than quoted more precisely than the \
                               constants support",
        },
        window,
        MARS24,
        1,
    )
    .expect("the shipped Mars anchor must satisfy Rule J")
}

/// The anchor for a calendar, if one has been determined.
///
/// `None` is a meaningful answer, not a failure: see the module documentation and
/// Rule J.3. Callers should route through [`crate::anchor::require_anchor`] so
/// that the absence becomes `UCAL-E0062` rather than a silent default.
pub fn for_calendar(calendar_id: &str) -> Option<Anchor> {
    match calendar_id {
        "earth-d" => Some(earth_d()),
        "mars-d" => Some(mars_d()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;
    use ucal_core::num::Ratio;
    use ucal_core::{Profile, Rounding};

    /// A body's solar day, in ticks, as a rational.
    fn solar_day(body: &crate::body::Body) -> Ratio {
        body.solar_day().value_at_epoch().clone()
    }

    #[test]
    fn ge3_earth_resolves_far_finer_than_a_solar_day() {
        let a = earth_d();
        let width = Ratio::from_int(a.uncertainty().ticks().clone());
        let day = solar_day(&data::earth());
        let fraction = width.div(&day).unwrap();
        // ±1 ms about the tick means a 2 ms window.
        assert_eq!(
            a.uncertainty().ticks(),
            &UC1::bridge()
                .ticks
                .try_mul(&<Ticks as TickInt>::from_u64(2))
                .unwrap()
                .quot_rem(&<Ticks as TickInt>::from_u64(1000))
                .0
        );
        // GE-3's criterion: narrower than one local solar day, by a lot.
        assert_eq!(
            fraction.to_decimal_string(10, Rounding::HalfEven).unwrap(),
            "0.0000000231"
        );
    }

    #[test]
    fn ge3_mars_resolves_far_finer_than_a_sol() {
        let a = mars_d();
        let width = Ratio::from_int(a.uncertainty().ticks().clone());
        let sol = solar_day(&data::mars());
        let fraction = width.div(&sol).unwrap();
        assert_eq!(
            fraction.to_decimal_string(8, Rounding::HalfEven).unwrap(),
            "0.00002253"
        );
    }

    #[test]
    fn ge3_titan_has_no_anchor_and_that_is_the_answer() {
        // The kill criterion says: document the width rather than narrowing it by
        // assumption. For Titan there is no width to document, because there is
        // no established convention — so there is no anchor, and the calendar is
        // incomplete in phase. Inventing one would be precisely the narrowing
        // GE-3 forbids.
        assert!(for_calendar("titan-d").is_none());
        assert!(!CALENDARS_WITH_ANCHORS.contains(&"titan-d"));
    }

    #[test]
    fn the_earth_anchor_is_a_whole_millisecond_after_midnight_tt() {
        // The determination, checked: the anchor is 63.8285 s after
        // 2000-01-01T00:00:00 TT, which is J2000.0 minus twelve hours.
        let midnight_tt = data::j2000()
            .checked_sub(&Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .try_mul(&<Ticks as TickInt>::from_u64(12 * 3600))
                    .unwrap(),
            ))
            .unwrap();
        let offset = earth_d().tick().since(&midnight_tt).unwrap();
        // 63.8285 s, exactly.
        let expect = UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(638_285))
            .unwrap()
            .quot_rem(&<Ticks as TickInt>::from_u64(10_000))
            .0;
        assert_eq!(offset.ticks(), &expect, "Delta-T at 2000.0 is 63.8285 s");
    }

    #[test]
    fn the_mars_anchor_precedes_the_earth_anchor() {
        // MSD 0 is in 1873; the Earth anchor is in 2000. A sanity check that the
        // two literals are on the same scale and not transcribed from different
        // epochs.
        assert!(mars_d().tick() < earth_d().tick());
        let gap = earth_d().tick().since(mars_d().tick()).unwrap();
        // Roughly 126 years, in bridge units.
        let years = Ratio::new(
            gap.ticks().clone(),
            UC1::bridge()
                .ticks
                .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
                .unwrap(),
        )
        .unwrap();
        assert_eq!(
            years.to_decimal_string(1, Rounding::HalfEven).unwrap(),
            "126.0"
        );
    }

    #[test]
    fn anchors_carry_everything_rule_j_requires() {
        for a in [earth_d(), mars_d()] {
            assert!(a.window().contains(a.tick()));
            assert_eq!(a.revision(), 1);
            assert!(a.phase().check_is_a_body_event().is_ok());
            assert!(a.method().method.len() > 40, "a real determination");
            assert!(a.method().uncertainty_note.len() > 40, "a real note");
            assert!(a.citation().source.len() > 20);
            // Every shipped phase is a solar-midnight one, so every anchor needs
            // the body's rotation and none needs its orbit.
            assert!(a.phase().needs_rotation());
            assert!(!a.phase().needs_orbit());
        }
    }

    #[test]
    fn the_shipped_anchors_are_evaluable_for_their_bodies() {
        // Rule J.4: an anchor whose phase cannot be evaluated is UCAL-E0063.
        earth_d().check_evaluable(&data::earth()).unwrap();
        mars_d().check_evaluable(&data::mars()).unwrap();
    }
}
