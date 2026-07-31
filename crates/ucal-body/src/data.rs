//! Built-in body data (§9.7).
//!
//! # Every entry is built the same way
//!
//! Rule K.5: Earth is an ordinary instance. There is no `earth`-shaped special
//! case here — every body is assembled by the same constructor calls, with the
//! same four Rule C obligations discharged for every parameter: an epoch, a
//! validity window, a citation, and the published value verbatim.
//!
//! # Where these numbers agree with the RFC, and where they do not
//!
//! Delta D-A6 sets the policy: Appendix I's printed ratios are the pinned test
//! vectors, and a body's parameters are chosen consistent with them to the
//! printed precision. Where they disagree *beyond* that precision, the divergence
//! is recorded here rather than reconciled silently.
//!
//! - **Mars agrees exactly.** Appendix G's published parameters — solar day
//!   88775.244 s, orbital period 686.9726 d — yield a ratio whose continued
//!   fraction and all seven convergents are identical to Appendix I.3's. The two
//!   appendices are fully consistent.
//! - **Earth agrees to seven decimals.** The declared parameters give
//!   12.3682665 against I.2's printed 12.3682668.
//! - **Titan does not agree, and the reason is a physical one.** See
//!   [`titan`].

use ucal_core::backend::TickInt;
use ucal_core::{Citation, Delta, Instant, Profile, Ticks, Window, UC1};

use crate::body::{AngleParam, Body, Satellite};
use crate::param::{Measured, MeasuredUnit, RatedParam};

// ---------------------------------------------------------------------------
// citations
// ---------------------------------------------------------------------------

/// IAU rotational-element report, the standard source for body orientation and
/// rotation periods.
pub const IAU_WGCCRE: Citation = Citation {
    source: "Archinal et al., Report of the IAU Working Group on Cartographic \
             Coordinates and Rotational Elements: 2015",
    locator: Some("doi:10.1007/s10569-017-9805-5"),
};

/// The standard planetary and lunar ephemeris fact sheets.
pub const NASA_FACT_SHEET: Citation = Citation {
    source: "NASA Planetary Fact Sheets (Williams, D. R.), NASA Space Science \
             Data Coordinated Archive",
    locator: Some("https://nssdc.gsfc.nasa.gov/planetary/factsheet/"),
};

/// The tropical year and mean solar day, as used by the civil calendar's own
/// definition. Recorded because Earth's parameters are the ones Appendix I.1 and
/// I.2 are pinned against.
pub const ASTRONOMICAL_ALMANAC: Citation = Citation {
    source: "The Astronomical Almanac, Explanatory Supplement (3rd ed.), \
             Urban & Seidelmann",
    locator: None,
};

// ---------------------------------------------------------------------------
// shared epoch and windows
// ---------------------------------------------------------------------------

/// J2000.0 — `2000-01-01T12:00:00 TT`, the epoch every parameter here is stated
/// at.
///
/// Written as a literal tick count rather than converted from a civil label,
/// because §12 forbids this crate from depending on `ucal-civil`. The value is
/// Appendix C's own J2000.0 fixture, so it is independently checked by the UC-P0
/// harness.
pub fn j2000() -> Instant<UC1> {
    Instant::from_ticks(
        <Ticks as TickInt>::from_dec_str(
            "8070205173569972963515184424835637180530466139316558837890625",
        )
        .expect("J2000.0 is within the domain"),
    )
    .expect("J2000.0 is within the domain")
}

/// A validity window of `± years` Julian years about J2000.0.
///
/// Rule C requires a window and forbids silent extrapolation beyond it. The
/// widths here follow the sources: rotational elements are quoted for a few
/// centuries either side of the epoch, orbital periods for rather longer.
fn window(years: u64) -> Window<UC1> {
    let span = Delta::from_ticks(
        UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
            .and_then(|v| v.try_mul(&<Ticks as TickInt>::from_u64(years)))
            .expect("within the domain"),
    );
    Window::new(
        j2000().checked_sub(&span).expect("after the datum"),
        j2000().checked_add(&span).expect("within the domain"),
    )
    .expect("lo <= hi")
}

fn param(
    mantissa: u128,
    decimals: u32,
    unit: MeasuredUnit,
    citation: Citation,
    valid_years: u64,
) -> RatedParam {
    RatedParam::new(
        Measured::new(mantissa, decimals, unit, citation),
        j2000(),
        window(valid_years),
    )
    .expect("built-in body data must satisfy Rule C")
}

// ---------------------------------------------------------------------------
// bodies
// ---------------------------------------------------------------------------

/// Earth.
///
/// The solar day is 86400 s **exactly** — the nominal mean solar day, which is
/// what the civil second was originally defined from and what Appendix I.1's
/// ratio is stated against. It is not Earth's actual current rotation, which runs
/// a millisecond or two short and is drifting; §8.3's whole point is that
/// `DAY_SI` and a rotation are different things.
///
/// The orbital period is the **tropical** year, chosen so that
/// `orbital_period / solar_day` is exactly Appendix I.1's `365.242190`.
pub fn earth() -> Body {
    Body::new(
        "earth",
        // Sidereal rotation: 86164.0905 s.
        param(861_640_905, 4, MeasuredUnit::SiSecond, IAU_WGCCRE, 1_000),
        // Mean solar day: 86400 s exactly.
        param(86_400, 0, MeasuredUnit::SiSecond, ASTRONOMICAL_ALMANAC, 1_000),
        // Tropical year: 365.242190 mean solar days = 31 556 925.216 s.
        param(
            31_556_925_216,
            3,
            MeasuredUnit::SiSecond,
            ASTRONOMICAL_ALMANAC,
            10_000,
        ),
    )
    .orbiting("sun")
    .with_obliquity(
        AngleParam::degrees(234_393, 4, IAU_WGCCRE).expect("valid angle"),
    )
    .with_satellite(Satellite::new(
        "moon",
        // Tropical month: 27.321582 d. Paired with the tropical year, this gives
        // a synodic month of 29.530589 d and Appendix I.2's ratio to seven
        // decimals.
        param(27_321_582, 6, MeasuredUnit::SiDay, ASTRONOMICAL_ALMANAC, 10_000),
        false,
    ))
}

/// Mars.
///
/// Appendix G's published parameters, unmodified. They reproduce Appendix I.3's
/// continued fraction `[0; 1, 1, 2, 4, 1, 2, 2, 1]` and all seven convergents
/// exactly — the two appendices are consistent, and no adjustment is needed.
pub fn mars() -> Body {
    Body::new(
        "mars",
        param(886_426_632, 4, MeasuredUnit::SiSecond, IAU_WGCCRE, 1_000),
        param(88_775_244, 3, MeasuredUnit::SiSecond, IAU_WGCCRE, 1_000),
        param(686_9726, 4, MeasuredUnit::SiDay, NASA_FACT_SHEET, 10_000),
    )
    .orbiting("sun")
    .with_obliquity(AngleParam::degrees(2519, 2, IAU_WGCCRE).expect("valid angle"))
    .with_satellite(Satellite::new(
        "phobos",
        param(27_553, 0, MeasuredUnit::SiSecond, NASA_FACT_SHEET, 1_000),
        false,
    ))
    .with_satellite(Satellite::new(
        "deimos",
        param(109_123, 0, MeasuredUnit::SiSecond, NASA_FACT_SHEET, 1_000),
        false,
    ))
}

/// Saturn, carried because Titan's year is Saturn's orbit.
pub fn saturn() -> Body {
    Body::new(
        "saturn",
        param(38_018, 0, MeasuredUnit::SiSecond, IAU_WGCCRE, 1_000),
        param(38_018, 0, MeasuredUnit::SiSecond, IAU_WGCCRE, 1_000),
        param(10_759_2058, 4, MeasuredUnit::SiDay, NASA_FACT_SHEET, 10_000),
    )
    .orbiting("sun")
    .with_obliquity(AngleParam::degrees(2673, 2, IAU_WGCCRE).expect("valid angle"))
    .with_satellite(Satellite::new(
        "titan",
        param(15_945_421, 6, MeasuredUnit::SiDay, NASA_FACT_SHEET, 10_000),
        false,
    ))
}

/// Titan.
///
/// # Two things Appendix I.5 gets slightly wrong
///
/// **First**, I.5 says "Titan is tidally locked to Saturn, so its solar day and
/// its orbit about Saturn coincide". They do not quite. Tidal lock fixes Titan's
/// face towards *Saturn*; the Sun moves relative to the pair as Saturn orbits, so
/// a solar day on Titan is the **synodic** period, 15.969088 d against an orbit
/// of 15.945421 d. The gap is 2045 s. It is the same distinction as sidereal
/// versus solar day on a planet, and it is exactly what §8.3 exists to keep
/// visible.
///
/// **Second**, and consequently, I.5's ratio is not reproducible from accepted
/// parameters. With the values below the ratio is `673.752068`, against I.5's
/// printed `673.983719443` — a difference of 0.23, which corresponds to a Titan
/// orbital period about 473 s (0.034%) shorter than the accepted one. The whole
/// part, 673, does match, which confirms the synodic reading; the fractional part
/// does not.
///
/// Per D-A6 the divergence is recorded rather than reconciled: these are the
/// published parameters, and Appendix I.5's convergent table remains pinned in
/// `ucal-core` against its own printed ratio. Feeding the mechanism the published
/// parameters gives a different, and physically correct, intercalation.
pub fn titan() -> Body {
    let rotation = param(15_945_421, 6, MeasuredUnit::SiDay, NASA_FACT_SHEET, 10_000);
    let year = param(10_759_2058, 4, MeasuredUnit::SiDay, NASA_FACT_SHEET, 10_000);

    // The solar day is **derived**, not measured. No source publishes it, it
    // follows exactly from the rotation and the year, and writing it back as a
    // decimal would invent a measurement and round an exact quantity at once.
    //
    //   solar_day = 1 / (1/P_rotation − 1/P_year)
    //
    // computed here in ticks, so nothing passes through a foreign unit.
    let solar = {
        let a = rotation.value_at_epoch().recip().expect("non-zero rotation");
        let b = year.value_at_epoch().recip().expect("non-zero year");
        a.abs_diff(&b)
            .and_then(|d| d.recip())
            .expect("rotation and year differ")
    };
    let solar_day = RatedParam::derived(
        solar,
        j2000(),
        window(10_000),
        "1 / (1/P_rotation - 1/P_orbital_period)",
        "tidal lock fixes Titan's face towards Saturn, not towards the Sun; the \
         Sun moves relative to the pair as Saturn orbits, so a solar day is the \
         synodic period. No source publishes it, and it follows exactly from two \
         that are published.",
        &[NASA_FACT_SHEET],
    )
    .expect("derived from positive published parameters");

    Body::new("titan", rotation, solar_day, year).orbiting("saturn")
}

/// Every built-in body (§9.7).
pub fn all() -> alloc::vec::Vec<Body> {
    alloc::vec![earth(), mars(), saturn(), titan()]
}

/// A built-in body by id.
pub fn by_id(id: &str) -> Option<Body> {
    all().into_iter().find(|b| b.id() == id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::Rounding;

    #[test]
    fn every_parameter_satisfies_rule_c() {
        // An epoch, a validity window containing it, a citation, and the
        // published value verbatim — for every parameter of every body.
        for body in all() {
            for (what, p) in [
                ("rotation", body.rotation_period()),
                ("solar_day", body.solar_day()),
                ("orbital", body.orbital_period()),
            ] {
                assert!(
                    p.valid().contains(p.epoch()),
                    "{} {what}: epoch outside its own window",
                    body.id()
                );
                // A measured parameter records its published value verbatim
                // (Rule Y.1); a derived one records the relation instead.
                assert!(
                    !p.provenance().describe().is_empty(),
                    "{} {what}: no provenance",
                    body.id()
                );
                assert!(
                    !p.citation().source.is_empty(),
                    "{} {what}: no citation",
                    body.id()
                );
                assert!(
                    p.value_at_epoch().numer() > &<Ticks as TickInt>::zero(),
                    "{} {what}: non-positive",
                    body.id()
                );
            }
            for s in body.satellites() {
                let p = s.orbital_period();
                assert!(p.valid().contains(p.epoch()), "{} {}", body.id(), s.id());
                assert!(!p.citation().source.is_empty());
            }
        }
    }

    #[test]
    fn earth_reproduces_appendix_i1_exactly() {
        let e = earth();
        let (r, w) = e.days_per_year(&j2000()).unwrap();
        assert_eq!(w, None);
        // Appendix I.1's ratio, exactly — not merely to printed precision.
        assert_eq!(
            r,
            ucal_core::num::Ratio::new(
                <Ticks as TickInt>::from_dec_str("36524219").unwrap(),
                <Ticks as TickInt>::from_dec_str("100000").unwrap()
            )
            .unwrap()
        );
        assert_eq!(
            r.to_decimal_string(6, Rounding::HalfEven).unwrap(),
            "365.242190"
        );
    }

    #[test]
    fn mars_reproduces_appendix_i3_to_its_printed_precision() {
        let m = mars();
        let (r, _) = m.days_per_year(&j2000()).unwrap();
        assert_eq!(
            r.to_decimal_string(9, Rounding::HalfEven).unwrap(),
            "668.592165627",
            "Appendix G's parameters must reproduce Appendix I.3"
        );
    }

    #[test]
    fn titans_divergence_from_appendix_i5_is_recorded_not_hidden() {
        // D-A6: where parameters and a printed ratio disagree beyond printed
        // precision, record it. Appendix I.5 prints 673.983719443; the published
        // parameters give 673.752068.
        let t = titan();
        let (r, _) = t.days_per_year(&j2000()).unwrap();
        let got = r.to_decimal_string(6, Rounding::HalfEven).unwrap();
        // Exact, because the solar day is derived in ticks rather than declared
        // as a rounded decimal. Declaring it to six places gave 673.752051; the
        // derivation gives the true value.
        assert_eq!(got, "673.752068");
        assert_ne!(
            got, "673.983719",
            "if these ever agree, the delta record is stale and should be removed"
        );
        // The divergence from Appendix I.5 is four orders larger than the
        // rounding of the declared solar day, so it is not a rounding artefact.
        let printed = ucal_core::num::Ratio::from_decimal_str("673.983719443").unwrap();
        let gap = printed.abs_diff(&r).unwrap();
        assert!(
            gap.cmp_exact(&ucal_core::num::Ratio::from_decimal_str("0.2").unwrap())
                == core::cmp::Ordering::Greater,
            "the I.5 divergence must be far larger than any declaration rounding"
        );
        // The whole part does match, which is what confirms the synodic reading.
        assert_eq!(r.floor().to_dec_string(), "673");
    }

    #[test]
    fn no_body_is_privileged() {
        // Rule K.5. Every body is built by the same calls and exposes the same
        // shape; the only asymmetries are data — satellites and obliquity — and
        // both are optional for every body alike.
        let bodies = all();
        assert!(bodies.len() >= 4);
        for b in &bodies {
            assert!(!b.id().is_empty());
            assert_eq!(b.citations().len() >= 3, true);
        }
        // Earth has no field Mars lacks.
        let e = earth();
        let m = mars();
        assert_eq!(e.obliquity().is_some(), m.obliquity().is_some());
        assert_eq!(
            e.satellites().is_empty(),
            m.satellites().is_empty(),
            "both have satellites; neither is special for having them"
        );
        // Titan has none, and that is data rather than a different kind of body.
        assert!(titan().satellites().is_empty());
    }

    #[test]
    fn lookup_by_id_works_and_is_not_case_folded() {
        assert!(by_id("earth").is_some());
        assert!(by_id("mars").is_some());
        assert!(by_id("titan").is_some());
        assert!(by_id("Earth").is_none());
        assert!(by_id("pluto").is_none());
    }
}
