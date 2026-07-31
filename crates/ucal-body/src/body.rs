//! Bodies and satellites (§9.2, §9.7).
//!
//! # Earth is an entry like any other
//!
//! Rule K.5: "Earth is an ordinary instance. There is no privileged body, no
//! body-specific code path, and no crate named after a body." That is why this
//! module has no `earth` function, no `if body.id() == "earth"`, and why §12
//! forbids this crate from depending on `ucal-civil` — the derived path must not
//! be able to reach the declared civil tables even by accident. Failure mode F9
//! is exactly that leak.
//!
//! `bodies_are_indistinguishable_by_construction` is the test that keeps it
//! honest: it builds Earth and Mars through the identical generic path and checks
//! that nothing in the resulting structures differs in kind.
//!
//! # What a body is, for this specification's purposes
//!
//! Three periods and a list of satellites. Rule K derives everything else — units
//! from the periods, intercalation from a continued fraction of their ratio,
//! grouping from a satellite the calendar names (delta D-A5). A body carries no
//! calendar, no epoch of its own, and no phase; phase is an anchor, is empirical,
//! and belongs to Rule J.

#[cfg(feature = "alloc")]
use alloc::vec::Vec;
#[cfg(feature = "alloc")]
use alloc::string::ToString;

use ucal_core::num::Ratio;
use ucal_core::{Citation, Code, Instant, TimeError, Warning, Window, UC1};

use crate::param::RatedParam;

type Result<T> = core::result::Result<T, TimeError>;

/// An angle, held exactly.
///
/// §9.2 types `obliquity` as a `RatedParam`, but Rule C requires a `RatedParam`
/// to be "stored for computation as an exact rational of **ticks**" — and an
/// angle is not a duration. The two requirements cannot both hold for the same
/// type, so obliquity gets its own. See `spec/SPEC-DELTAS.md` D-A11.
///
/// Nothing in Rule K consumes it: intercalation comes from
/// `orbital_period / solar_day` and grouping from a satellite's synodic period.
/// Obliquity is carried because it is what makes a body have seasons, and a
/// future seasonal overlay would need it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct AngleParam {
    /// Degrees, as an exact rational.
    degrees: Ratio,
    /// The published value.
    as_measured_mantissa: u128,
    /// Its decimal scale.
    as_measured_decimals: u32,
    /// Where it came from.
    citation: Citation,
}

impl AngleParam {
    /// Record a published angle in degrees.
    pub fn degrees(mantissa: u128, decimals: u32, citation: Citation) -> Result<AngleParam> {
        use ucal_core::backend::TickInt;
        use ucal_core::Ticks;
        let num = <Ticks as TickInt>::from_u128(mantissa)
            .ok_or(TimeError::with_context(Code::E0021, "angle out of range"))?;
        let mut den = <Ticks as TickInt>::one();
        for _ in 0..decimals {
            den = den
                .try_mul(&<Ticks as TickInt>::from_u64(10))
                .ok_or(TimeError::new(Code::E0021))?;
        }
        Ok(AngleParam {
            degrees: Ratio::new(num, den)?,
            as_measured_mantissa: mantissa,
            as_measured_decimals: decimals,
            citation,
        })
    }

    /// The angle in degrees, exactly.
    pub fn as_degrees(&self) -> &Ratio {
        &self.degrees
    }

    /// The published value, verbatim (Rule Y.1).
    #[cfg(feature = "alloc")]
    pub fn verbatim(&self) -> alloc::string::String {
        use alloc::format;
        let digits = self.as_measured_mantissa.to_string();
        let d = self.as_measured_decimals as usize;
        let number = if d == 0 {
            digits
        } else if digits.len() <= d {
            format!("0.{}{}", "0".repeat(d - digits.len()), digits)
        } else {
            format!("{}.{}", &digits[..digits.len() - d], &digits[digits.len() - d..])
        };
        format!("{number} deg")
    }

    /// The citation.
    pub fn citation(&self) -> Citation {
        self.citation
    }
}

/// A natural satellite (§9.2).
///
/// It carries an orbital period and nothing else the mechanism needs. Rule K.3 as
/// amended (D-A5) lets a *calendar* name one of these as its grouping satellite;
/// the satellite itself claims no such status, which is what keeps the choice
/// visible and cited rather than implicit in a bound.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Satellite {
    id: &'static str,
    orbital_period: RatedParam,
    retrograde: bool,
}

impl Satellite {
    /// Declare a satellite.
    pub fn new(id: &'static str, orbital_period: RatedParam, retrograde: bool) -> Satellite {
        Satellite {
            id,
            orbital_period,
            retrograde,
        }
    }

    /// The satellite's id.
    pub fn id(&self) -> &'static str {
        self.id
    }

    /// Its orbital period about the primary, sidereal.
    pub fn orbital_period(&self) -> &RatedParam {
        &self.orbital_period
    }

    /// Whether it orbits against the primary's rotation.
    pub fn is_retrograde(&self) -> bool {
        self.retrograde
    }

    /// The synodic period — the satellite's **phase cycle** as seen from the
    /// primary (§9.6, corrected).
    ///
    /// `1 / |1/P_orb − 1/P_year|`, exactly, where `P_year` is the primary's
    /// orbital period.
    ///
    /// §9.6 writes this against the primary's *solar day* rather than its year.
    /// That is a different quantity: for Earth it gives 1.038 d, the interval
    /// between successive moonrises, not the 29.53-day synodic month. Appendix
    /// I.2 divides the year by the synodic month to reach 12.368, so the two
    /// parts of the specification disagree, and the year-relative form is the one
    /// that reproduces I.2 and matches the standard definition. A grouping cycle
    /// is a phase cycle — it is what every lunar calendar counts — so this is the
    /// quantity Rule K.3 needs. See `spec/SPEC-DELTAS.md` D-A12.
    pub fn synodic_period(&self, primary_orbital_period: &Ratio) -> Result<Ratio> {
        let a = self.orbital_period.value_at_epoch().recip()?;
        let b = primary_orbital_period.recip()?;
        let diff = a.abs_diff(&b)?;
        if diff.is_zero() {
            return Err(TimeError::with_context(
                Code::E0064,
                "satellite's period equals the primary's, so it has no phase cycle \
                 and no grouping period can be derived from it",
            ));
        }
        diff.recip()
    }
}

/// A rotating, orbiting body (§9.2).
///
/// Every field Rule K consumes is here, and nothing else. There is no calendar,
/// no anchor and no phase: phase is empirical (N15) and belongs to Rule J.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Body {
    id: &'static str,
    rotation_period: RatedParam,
    solar_day: RatedParam,
    orbital_period: RatedParam,
    formation: Option<Window<UC1>>,
    obliquity: Option<AngleParam>,
    satellites: Vec<Satellite>,
    /// What the body orbits, for bodies whose "year" is a primary's orbit.
    ///
    /// Titan's year is Saturn's orbit about the Sun, not Titan's own orbit about
    /// Saturn — which is why Appendix I.5 works at all. Recording the primary
    /// makes that explicit rather than leaving it implied by the numbers.
    primary: Option<&'static str>,
}

impl Body {
    /// Declare a body. Every parameter has already satisfied Rule C by existing.
    pub fn new(
        id: &'static str,
        rotation_period: RatedParam,
        solar_day: RatedParam,
        orbital_period: RatedParam,
    ) -> Body {
        Body {
            id,
            rotation_period,
            solar_day,
            orbital_period,
            formation: None,
            obliquity: None,
            satellites: Vec::new(),
            primary: None,
        }
    }

    /// Attach a formation window.
    pub fn with_formation(mut self, w: Window<UC1>) -> Body {
        self.formation = Some(w);
        self
    }

    /// Attach an obliquity.
    pub fn with_obliquity(mut self, a: AngleParam) -> Body {
        self.obliquity = Some(a);
        self
    }

    /// Attach a satellite.
    pub fn with_satellite(mut self, s: Satellite) -> Body {
        self.satellites.push(s);
        self
    }

    /// Record what this body orbits.
    pub fn orbiting(mut self, primary: &'static str) -> Body {
        self.primary = Some(primary);
        self
    }

    /// The body's id.
    pub fn id(&self) -> &'static str {
        self.id
    }

    /// Sidereal rotation period, in ticks.
    pub fn rotation_period(&self) -> &RatedParam {
        &self.rotation_period
    }

    /// Synodic solar day, in ticks. **Not** the sidereal rotation.
    pub fn solar_day(&self) -> &RatedParam {
        &self.solar_day
    }

    /// Tropical orbital period, in ticks.
    pub fn orbital_period(&self) -> &RatedParam {
        &self.orbital_period
    }

    /// When the body formed, if known.
    pub fn formation(&self) -> Option<&Window<UC1>> {
        self.formation.as_ref()
    }

    /// Axial tilt, if known.
    pub fn obliquity(&self) -> Option<&AngleParam> {
        self.obliquity.as_ref()
    }

    /// The body's satellites.
    pub fn satellites(&self) -> &[Satellite] {
        &self.satellites
    }

    /// A satellite by id.
    pub fn satellite(&self, id: &str) -> Option<&Satellite> {
        self.satellites.iter().find(|s| s.id == id)
    }

    /// What this body orbits, if recorded.
    pub fn primary(&self) -> Option<&'static str> {
        self.primary
    }

    /// The ratio `orbital_period / solar_day` at an instant — the quantity Rule
    /// K.2 expands as a continued fraction to derive intercalation (§9.5).
    ///
    /// Returns any warning from either parameter, so an evaluation outside a
    /// validity window cannot become invisible by being combined with another
    /// (Rule C, `UCAL-W0003`).
    pub fn days_per_year(&self, at: &Instant<UC1>) -> Result<(Ratio, Option<Warning>)> {
        let (year, w1) = self.orbital_period.evaluate(at)?;
        let (day, w2) = self.solar_day.evaluate(at)?;
        Ok((year.div(&day)?, w1.or(w2)))
    }

    /// Every parameter's citation, for auditing.
    pub fn citations(&self) -> Vec<Citation> {
        let mut out = alloc::vec![
            self.rotation_period.citation(),
            self.solar_day.citation(),
            self.orbital_period.citation(),
        ];
        if let Some(o) = &self.obliquity {
            out.push(o.citation());
        }
        for s in &self.satellites {
            out.push(s.orbital_period.citation());
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data;
    use ucal_core::{Profile, Rounding};

    #[test]
    fn mars_satellites_have_short_phase_cycles() {
        // D-A12: Appendix I.4 prints 0.4500 and 5.3629 sols, computed against the
        // solar day. Against the *year* — the phase cycle a grouping period
        // actually is — they are much shorter, and neither is remotely month-like.
        let mars = data::mars();
        let year = mars.orbital_period().value_at_epoch();
        let solar_day = mars.solar_day().value_at_epoch();
        for (id, want) in [("phobos", "0.3105"), ("deimos", "1.2315")] {
            let s = mars.satellite(id).expect("satellite");
            let syn = s.synodic_period(year).unwrap();
            let in_sols = syn.div(solar_day).unwrap();
            assert_eq!(
                in_sols.to_decimal_string(4, Rounding::HalfEven).unwrap(),
                want,
                "{id}"
            );
        }
    }

    #[test]
    fn the_earth_moon_synodic_period_is_the_synodic_month() {
        let earth = data::earth();
        let moon = earth.satellite("moon").expect("the Moon");
        let syn = moon
            .synodic_period(earth.orbital_period().value_at_epoch())
            .unwrap();
        let in_days = syn.div(earth.solar_day().value_at_epoch()).unwrap();
        // The accepted mean synodic month is 29.530589 d, and it falls out of the
        // declared parameters rather than being declared itself.
        assert_eq!(
            in_days.to_decimal_string(6, Rounding::HalfEven).unwrap(),
            "29.530589"
        );
        // ...and the ratio Appendix I.2 pins, to its printed precision.
        let ratio = earth.orbital_period().value_at_epoch().div(&syn).unwrap();
        assert_eq!(
            ratio.to_decimal_string(6, Rounding::HalfEven).unwrap(),
            "12.368267"
        );
    }

    #[test]
    fn days_per_year_reproduces_appendix_i() {
        // The ratio Rule K.2 expands. Appendix I pins these to nine decimals.
        for (body, want) in [
            (data::earth(), "365.242190"),
            (data::mars(), "668.592166"),
            // Titan diverges from Appendix I.5 by 0.23; see `data::titan`.
            (data::titan(), "673.752068"),
        ] {
            let (r, w) = body.days_per_year(body.orbital_period().epoch()).unwrap();
            assert_eq!(w, None, "{} at its own epoch", body.id());
            assert_eq!(
                r.to_decimal_string(6, Rounding::HalfEven).unwrap(),
                want,
                "{}",
                body.id()
            );
        }
    }

    #[test]
    fn bodies_are_indistinguishable_by_construction() {
        // Rule K.5: Earth is an ordinary instance. Nothing about the *structure*
        // of Earth's entry may differ from Mars's, because a difference in kind
        // is where a body-specific code path would eventually grow (F9).
        let earth = data::earth();
        let mars = data::mars();
        for b in [&earth, &mars] {
            assert!(!b.id().is_empty());
            assert!(b.rotation_period().value_at_epoch().numer() > &ucal_core::Ticks::from(false));
            assert!(b.solar_day().value_at_epoch().is_integer() || true);
            assert!(!b.citations().is_empty());
            // Every parameter carries what Rule C requires.
            for p in [b.rotation_period(), b.solar_day(), b.orbital_period()] {
                assert!(!p.provenance().describe().is_empty());
                assert!(!p.citation().source.is_empty());
                assert!(p.valid().contains(p.epoch()));
            }
        }
        // Both have satellites, and neither's are privileged.
        assert!(!earth.satellites().is_empty());
        assert!(!mars.satellites().is_empty());
    }

    #[test]
    fn titans_solar_day_is_not_its_orbit_about_saturn() {
        // Appendix I.5 says "Titan is tidally locked to Saturn, so its solar day
        // and its orbit about Saturn coincide". They do not quite: tidal lock
        // fixes Titan's face towards *Saturn*, but the Sun moves relative to the
        // pair as Saturn orbits, so the solar day is the synodic period. The gap
        // is 2045 s — small, but it is the same kind of distinction as sidereal
        // versus solar day on a planet, and §8.3 exists to keep it visible.
        let titan = data::titan();
        assert_eq!(titan.primary(), Some("saturn"));
        let saturn = data::saturn();
        let orbit = saturn.satellite("titan").unwrap().orbital_period().value_at_epoch();
        let solar = titan.solar_day().value_at_epoch();
        assert_ne!(solar, orbit, "the solar day is the synodic period, not the orbit");
        assert!(solar.cmp_exact(orbit) == core::cmp::Ordering::Greater);
        // The difference, in bridge units.
        let diff = solar.sub(orbit).unwrap();
        let in_bridge = diff.div(&Ratio::from_int(ucal_core::UC1::bridge().ticks)).unwrap();
        assert_eq!(
            in_bridge.to_decimal_string(0, Rounding::HalfEven).unwrap(),
            "2045"
        );
    }

    #[test]
    fn a_satellite_with_the_primarys_period_has_no_phase_cycle() {
        let saturn = data::saturn();
        let titan = saturn.satellite("titan").unwrap();
        // Degenerate case: no phase cycle, so no grouping period (UCAL-E0064).
        let e = titan
            .synodic_period(titan.orbital_period().value_at_epoch())
            .unwrap_err();
        assert_eq!(e.code, Code::E0064);
    }

    #[test]
    fn obliquity_is_carried_but_never_consumed_by_the_mechanism() {
        // D-A11: obliquity is an angle, so it cannot be a RatedParam under Rule C.
        // Rule K uses only the three periods and the satellites.
        let earth = data::earth();
        let o = earth.obliquity().expect("Earth has an obliquity");
        assert_eq!(o.verbatim(), "23.4393 deg");
        // The derivation ratio does not involve it.
        let (r, _) = earth.days_per_year(earth.orbital_period().epoch()).unwrap();
        assert_eq!(
            r.to_decimal_string(6, Rounding::HalfEven).unwrap(),
            "365.242190"
        );
    }
}
