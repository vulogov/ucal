//! Body parameters (§9.2, Rule C).
//!
//! # What Rule C demands, and why
//!
//! > Every parameter MUST carry an epoch, a validity window, a citation, and its
//! > verbatim as-measured value, and MUST be stored for computation as an exact
//! > rational of ticks. Evaluation outside the validity window MUST warn
//! > (`UCAL-W0003`) and MUST NOT silently extrapolate.
//!
//! Four requirements, and each closes a specific hole:
//!
//! - **Stored in ticks.** Rule A.1 makes the tick primitive. A parameter kept in
//!   seconds would put a foreign unit inside the derivation mechanism, which is
//!   exactly what Rule K exists to prevent.
//! - **Verbatim as-measured value.** Rule Y.1 concedes that measurement arrives
//!   in foreign units, and requires the original be recorded rather than
//!   discarded. Without it a conversion cannot be audited, only trusted.
//! - **Epoch and rate.** Failure mode F6 is "a derived calendar drifts because
//!   parameters were treated as constants". Earth's rotation lengthens by roughly
//!   1.8 ms per century and its tropical year shortens by about 0.53 s per
//!   century; both are rates, not constants.
//! - **Validity window.** A parameter evaluated far outside the interval its
//!   source supports is an extrapolation, and an extrapolation presented as a
//!   measurement is a lie of omission. Hence `UCAL-W0003`.
//!
//! # The conversion boundary
//!
//! Rule Y permits foreign units at exactly three points, and body parameters are
//! one of them. The conversion into ticks must be **exact through the declared
//! bridge constant, or rejected** (`UCAL-E0043`). There is no rounding path: a
//! period that cannot be expressed exactly in ticks is refused, not approximated.

#[cfg(feature = "alloc")]
use alloc::string::{String, ToString};

use ucal_core::backend::TickInt;
use ucal_core::num::Ratio;
use ucal_core::{Citation, Code, Instant, Profile, Ticks, TimeError, Warning, Window, UC1};

type Result<T> = core::result::Result<T, TimeError>;

/// The units a *published* body parameter may arrive in (Rule Y.1).
///
/// # Every variant is a multiple of the bridge constant, and nothing else
///
/// These are **not** anybody's day or year. `SiDay` is 86400 SI seconds by
/// definition; it is not Earth's rotation, which is neither exactly 86400 s nor
/// constant. `JulianYear` is 31 557 600 SI seconds by definition; it is not
/// Earth's orbit. Naming them for what they are — exact multiples of the declared
/// bridge constant — is the same distinction §8.3 draws between `DAY_SI` and a
/// rotation, and it is the one that keeps failure mode F9 shut.
///
/// # Why no body's own units appear here
///
/// Not an oversight, and not Earth being privileged. A body's parameters cannot
/// be expressed in that body's own units without circularity — "Titan's solar day
/// is one Titan day" states nothing. Measurements must arrive in some *external*
/// system, and profile `UC-1` declares exactly one bridge constant (Rule A.3), so
/// SI is the only external system reachable.
///
/// Earth appears here only because Earth is where the instruments are: journals
/// publish in seconds and days. That is a fact about publication practice, which
/// Rule Y concedes explicitly, and not a fact about the calendar. Nothing derived
/// from these values keeps them — [`Measured::ticks`] converts exactly on the way
/// in, and every stored parameter is a rational of ticks.
///
/// A parameter that follows from others by definition, rather than from an
/// instrument, should not be forced through this enum at all. Use
/// [`RatedParam::derived`].
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub enum MeasuredUnit {
    /// SI seconds.
    SiSecond,
    /// 86400 SI seconds, exactly. Not a rotation of any body.
    SiDay,
    /// 31 557 600 SI seconds, exactly. Not an orbit of any body.
    JulianYear,

    /// 60 SI seconds, exactly.
    ///
    /// Added in 1.9.0 with [`MeasuredUnit::Hour`], and for the same reason: Rule
    /// C asks for the **published value verbatim**, and the sources this project
    /// cites most publish rotation periods in hours. Without these an author had
    /// to convert by hand, which is either a rounding — and a rounded parameter
    /// is a different calendar — or an exact conversion the file no longer shows
    /// the working for.
    ///
    /// **Appended rather than placed in reading order**, like `Code::E0014` and
    /// for the same reason: the variants carry implicit discriminants, so
    /// inserting one in the middle changes the value of every later variant and
    /// breaks a caller who casts. It was written in the middle first and
    /// `cargo semver-checks` reported `enum_no_repr_variant_discriminant_changed`
    /// — the same lesson, learned the same way, three releases after it was
    /// written down.
    SiMinute,
    /// 3600 SI seconds, exactly.
    ///
    /// The NASA planetary fact sheets print "Length of Day (hrs)" and rotation
    /// periods in hours for every planet. `data::jupiter` converts them in a
    /// comment — `9.9250 h x 3600 = 35 730 s, exact` — which is what a file
    /// could not do.
    Hour,
}

impl MeasuredUnit {
    /// How many bridge units one of these is. Exact by definition in every case.
    pub const fn bridge_units(self) -> u64 {
        match self {
            MeasuredUnit::SiSecond => 1,
            MeasuredUnit::SiMinute => 60,
            MeasuredUnit::Hour => 3_600,
            MeasuredUnit::SiDay => 86_400,
            MeasuredUnit::JulianYear => 31_557_600,
        }
    }

    /// The unit's symbol, as it appears in a citation.
    pub const fn symbol(self) -> &'static str {
        match self {
            MeasuredUnit::SiSecond => "s",
            MeasuredUnit::SiMinute => "min (60 s)",
            MeasuredUnit::Hour => "h (3600 s)",
            MeasuredUnit::SiDay => "d (86400 s)",
            MeasuredUnit::JulianYear => "yr (Julian, 31557600 s)",
        }
    }
}

/// A measured value, recorded exactly as published (Rule Y.1).
///
/// The value is held as an integer numerator and a decimal scale rather than as a
/// parsed number, so `88775.244 s` round-trips to the character. Rule Y.1 says
/// "recorded verbatim"; a value that has been through a lossy parse is not.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[non_exhaustive]
pub struct Measured {
    /// The digits, with the decimal point removed.
    pub mantissa: u128,
    /// How many of those digits are after the decimal point.
    pub decimals: u32,
    /// What unit the publication used.
    pub unit: MeasuredUnit,
    /// Where it came from.
    pub citation: Citation,
}

impl Measured {
    /// Record a published value.
    pub const fn new(
        mantissa: u128,
        decimals: u32,
        unit: MeasuredUnit,
        citation: Citation,
    ) -> Measured {
        Measured {
            mantissa,
            decimals,
            unit,
            citation,
        }
    }

    /// The value as published, e.g. `"88775.244 s"`.
    #[cfg(feature = "alloc")]
    pub fn verbatim(&self) -> String {
        use alloc::format;
        let digits = self.mantissa.to_string();
        let d = self.decimals as usize;
        let number = if d == 0 {
            digits
        } else if digits.len() <= d {
            format!("0.{}{}", "0".repeat(d - digits.len()), digits)
        } else {
            format!("{}.{}", &digits[..digits.len() - d], &digits[digits.len() - d..])
        };
        format!("{number} {}", self.unit.symbol())
    }

    /// The exact value in ticks, as a rational (Rule C).
    ///
    /// `mantissa × unit_factor × SECOND / 10^decimals`. `UCAL-E0043` if the
    /// division is not exact — that is, if the published value is finer than the
    /// bridge can represent. Rule Y.2 requires exactness or rejection, and Rule R
    /// forbids rounding on the way in.
    pub fn ticks(&self) -> Result<Ratio> {
        let mantissa = <Ticks as TickInt>::from_u128(self.mantissa)
            .ok_or(TimeError::with_context(Code::E0021, "mantissa out of range"))?;
        let factor = <Ticks as TickInt>::from_u64(self.unit.bridge_units());
        let numerator = mantissa
            .try_mul(&factor)
            .and_then(|v| v.try_mul(&UC1::bridge().ticks))
            .ok_or(TimeError::new(Code::E0021))?;
        let denominator = pow10(self.decimals)?;

        // The bridge carries thirty decimal places (D-3). Anything finer cannot
        // be converted exactly, and Rule Y.2 says reject rather than round.
        let (q, r) = numerator.quot_rem(&denominator);
        if !r.is_zero_ticks() {
            return Err(TimeError::with_context(
                Code::E0043,
                "measured value is finer than the bridge constant can represent \
                 exactly; Rule Y.2 requires exact conversion or rejection",
            ));
        }
        Ok(Ratio::from_int(q))
    }
}

fn pow10(e: u32) -> Result<Ticks> {
    let ten = <Ticks as TickInt>::from_u64(10);
    let mut acc = <Ticks as TickInt>::one();
    for _ in 0..e {
        acc = acc.try_mul(&ten).ok_or(TimeError::new(Code::E0021))?;
    }
    Ok(acc)
}

/// Where a parameter's value came from.
///
/// Rule Y.1 requires a *measured* value to be recorded verbatim in the unit its
/// source used. But not every parameter is measured: some follow exactly from
/// others by definition, and for those there is no publication to record.
///
/// Writing a derived value back as though it were measured would be two errors
/// at once — it fabricates a citation-shaped number, and it rounds a quantity
/// that was exact. Titan's solar day is the case in point: no source publishes
/// it, it follows exactly from the orbit and the primary's year, and declaring it
/// as "15.969088 d" both invented a measurement and discarded precision.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Provenance {
    /// Published, and recorded verbatim in the unit the source used (Rule Y.1).
    Measured(Measured),
    /// Computed exactly from other parameters. Carries the relation and the
    /// citations of its inputs, so the derivation is auditable end to end.
    Derived {
        /// The relation, written out.
        relation: &'static str,
        /// Why this parameter is derived rather than measured.
        because: &'static str,
        /// The citations of the inputs it was computed from.
        inputs: &'static [Citation],
    },
}

impl Provenance {
    /// The published value, for a measured parameter.
    pub fn measured(&self) -> Option<&Measured> {
        match self {
            Provenance::Measured(m) => Some(m),
            Provenance::Derived { .. } => None,
        }
    }

    /// A citation for this parameter: its own, or its inputs' first.
    pub fn citation(&self) -> Citation {
        match self {
            Provenance::Measured(m) => m.citation,
            Provenance::Derived { inputs, .. } => inputs.first().copied().unwrap_or(Citation::new(
        "derived; no input citation recorded",
        None,
    )),
        }
    }

    /// A one-line description, for `ucal body show`.
    #[cfg(feature = "alloc")]
    pub fn describe(&self) -> String {
        use alloc::format;
        match self {
            Provenance::Measured(m) => m.verbatim(),
            Provenance::Derived { relation, .. } => format!("derived: {relation}"),
        }
    }
}

/// A parameter with an epoch, an optional rate, a validity window, and its
/// provenance (§9.2, Rule C).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RatedParam {
    /// The value at [`RatedParam::epoch`], in ticks. Exact.
    value: Ratio,
    /// First derivative per tick, if the parameter is known to drift.
    ///
    /// Held per *tick*, not per century, so that applying it needs no foreign
    /// unit — the rate is a pure ratio of ticks to ticks.
    rate: Option<Ratio>,
    /// The instant the value is stated at.
    epoch: Instant<UC1>,
    /// The interval over which the source supports the value.
    valid: Window<UC1>,
    /// Where the value came from: measured and recorded verbatim, or derived
    /// exactly from other parameters.
    provenance: Provenance,
    /// The rate as published, where there is one.
    rate_as_measured: Option<Measured>,
}

impl RatedParam {
    /// Construct from a measured value.
    ///
    /// Everything Rule C requires is a parameter of this function, so a parameter
    /// missing its epoch, window or citation cannot be built — which is stricter
    /// than `UCAL-E0060` and makes that code a backstop for data loaded at
    /// runtime rather than the primary defence.
    pub fn new(
        as_measured: Measured,
        epoch: Instant<UC1>,
        valid: Window<UC1>,
    ) -> Result<RatedParam> {
        let value = as_measured.ticks()?;
        if value.is_zero() {
            return Err(TimeError::with_context(
                Code::E0060,
                "a body parameter must be a positive duration",
            ));
        }
        if !valid.contains(&epoch) {
            return Err(TimeError::with_context(
                Code::E0060,
                "a parameter's epoch must lie inside its own validity window",
            ));
        }
        Ok(RatedParam {
            value,
            rate: None,
            epoch,
            valid,
            provenance: Provenance::Measured(as_measured),
            rate_as_measured: None,
        })
    }

    /// A parameter computed exactly from others.
    ///
    /// The value arrives already in ticks, as an exact rational, so nothing is
    /// rounded and no foreign unit is involved. This is the constructor for a
    /// quantity that follows from others by definition — a tidally locked moon's
    /// solar day, say — where inventing a "published" decimal would be a fiction
    /// and a loss of precision at the same time.
    pub fn derived(
        value: Ratio,
        epoch: Instant<UC1>,
        valid: Window<UC1>,
        relation: &'static str,
        because: &'static str,
        inputs: &'static [Citation],
    ) -> Result<RatedParam> {
        if value.is_zero() {
            return Err(TimeError::with_context(
                Code::E0060,
                "a body parameter must be a positive duration",
            ));
        }
        if !valid.contains(&epoch) {
            return Err(TimeError::with_context(
                Code::E0060,
                "a parameter's epoch must lie inside its own validity window",
            ));
        }
        Ok(RatedParam {
            value,
            rate: None,
            epoch,
            valid,
            provenance: Provenance::Derived {
                relation,
                because,
                inputs,
            },
            rate_as_measured: None,
        })
    }

    /// Attach a drift rate, published per Julian century.
    ///
    /// Failure mode F6 is a derived calendar drifting because parameters were
    /// treated as constants. Earth's rotation lengthens by about 1.8 ms per
    /// century; over the 45-year span of a single T3 group that is 0.8 ms, and
    /// over a T5 deep it is millions of seconds.
    ///
    /// The rate is converted to a per-tick ratio on the way in, so that applying
    /// it later touches no foreign unit.
    pub fn with_rate_per_julian_century(mut self, rate: Measured) -> Result<RatedParam> {
        let per_century = rate.ticks()?;
        // One Julian century in ticks: 100 × 31 557 600 bridge units.
        let century = <Ticks as TickInt>::from_u64(100 * 31_557_600)
            .try_mul(&UC1::bridge().ticks)
            .ok_or(TimeError::new(Code::E0021))?;
        self.rate = Some(per_century.div(&Ratio::from_int(century))?);
        self.rate_as_measured = Some(rate);
        Ok(self)
    }

    /// The value at the parameter's own epoch, in ticks.
    pub fn value_at_epoch(&self) -> &Ratio {
        &self.value
    }

    /// The per-tick rate, if declared.
    pub fn rate_per_tick(&self) -> Option<&Ratio> {
        self.rate.as_ref()
    }

    /// The epoch the value is stated at.
    pub fn epoch(&self) -> &Instant<UC1> {
        &self.epoch
    }

    /// The interval the source supports.
    pub fn valid(&self) -> &Window<UC1> {
        &self.valid
    }

    /// Where the value came from.
    pub fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    /// The published value, verbatim, for a measured parameter (Rule Y.1).
    ///
    /// `None` for a derived one, which by definition has no publication.
    pub fn as_measured(&self) -> Option<&Measured> {
        self.provenance.measured()
    }

    /// The published rate, verbatim.
    pub fn rate_as_measured(&self) -> Option<&Measured> {
        self.rate_as_measured.as_ref()
    }

    /// The citation: this parameter's own, or its inputs' if derived.
    pub fn citation(&self) -> Citation {
        self.provenance.citation()
    }

    /// Evaluate at an instant, applying the rate.
    ///
    /// Returns the value **and any warning**. `UCAL-W0003` accompanies a result
    /// outside the validity window — Rule C says such an evaluation must warn and
    /// must not silently extrapolate. The value is still returned, because
    /// refusing outright would make a calendar unusable a century either side of
    /// its source's window; what Rule C forbids is doing it *silently*.
    pub fn evaluate(&self, at: &Instant<UC1>) -> Result<(Ratio, Option<Warning>)> {
        let warning = if self.valid.contains(at) {
            None
        } else {
            Some(Warning::W0003)
        };

        let Some(rate) = &self.rate else {
            return Ok((self.value.clone(), warning));
        };

        // value + rate × (at − epoch), signed, exactly.
        let elapsed = at.between(self.epoch());
        let magnitude = Ratio::from_int(elapsed.magnitude().ticks().clone());
        let delta = rate.mul(&magnitude)?;
        let value = match elapsed.sign() {
            ucal_core::Sign::Positive => self.value.add(&delta)?,
            ucal_core::Sign::Negative => self.value.sub(&delta).map_err(|_| {
                TimeError::with_context(
                    Code::E0060,
                    "the declared rate drives this parameter through zero before \
                     the requested instant; the extrapolation is not physical",
                )
            })?,
        };
        Ok((value, warning))
    }

    /// Evaluate, refusing rather than warning outside the window.
    ///
    /// For callers that would rather not carry a warning: `UCAL-W0003` becomes an
    /// error. Offered because a *derived calendar* has good reason to be strict,
    /// while an exploratory query does not.
    pub fn evaluate_strict(&self, at: &Instant<UC1>) -> Result<Ratio> {
        let (v, w) = self.evaluate(at)?;
        match w {
            None => Ok(v),
            Some(_) => Err(TimeError::with_context(
                Code::E0060,
                "instant lies outside the parameter's validity window and strict \
                 evaluation was requested (Rule C, UCAL-W0003)",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ucal_core::Delta;

    const IAU: Citation = Citation::new(
        "IAU WGCCRE 2015 report on cartographic coordinates and rotational elements",
        Some("doi:10.1007/s10569-017-9805-5"),
    );

    fn j2000() -> Instant<UC1> {
        // 2000-01-01T12:00:00 TT — Appendix C's fixture, as a literal so that
        // ucal-body needs no dependency on ucal-civil (§12).
        Instant::from_ticks(
            <Ticks as TickInt>::from_dec_str(
                "8070205173569972963515184424835637180530466139316558837890625",
            )
            .unwrap(),
        )
        .unwrap()
    }

    fn wide_window() -> Window<UC1> {
        let span = Delta::from_ticks(
            UC1::bridge()
                .ticks
                .try_mul(&<Ticks as TickInt>::from_u64(31_557_600 * 1_000_000))
                .unwrap(),
        );
        Window::new(
            j2000().checked_sub(&span).unwrap(),
            j2000().checked_add(&span).unwrap(),
        )
        .unwrap()
    }

    fn mars_solar_day() -> Measured {
        // Appendix G: 88775.244 s.
        Measured::new(88_775_244, 3, MeasuredUnit::SiSecond, IAU)
    }

    #[test]
    fn measured_values_round_trip_verbatim() {
        // Rule Y.1: recorded verbatim. A value that has been through a lossy
        // parse is not verbatim, so the digits and the scale are kept separately.
        assert_eq!(mars_solar_day().verbatim(), "88775.244 s");
        assert_eq!(
            Measured::new(686_9726, 4, MeasuredUnit::SiDay, IAU).verbatim(),
            "686.9726 d (86400 s)"
        );
        assert_eq!(
            Measured::new(25_19, 2, MeasuredUnit::SiSecond, IAU).verbatim(),
            "25.19 s"
        );
        // A value smaller than one keeps its leading zero.
        assert_eq!(
            Measured::new(4500, 4, MeasuredUnit::SiSecond, IAU).verbatim(),
            "0.4500 s"
        );
    }

    #[test]
    fn parameters_are_stored_in_ticks_not_in_the_measured_unit() {
        // Rule C: "MUST be stored for computation as an exact rational of ticks".
        let p = RatedParam::new(mars_solar_day(), j2000(), wide_window()).unwrap();
        let ticks = p.value_at_epoch();
        // 88775.244 s × SECOND, exactly.
        let expect = <Ticks as TickInt>::from_u64(88_775_244)
            .try_mul(&UC1::bridge().ticks)
            .unwrap()
            .quot_rem(&<Ticks as TickInt>::from_u64(1000))
            .0;
        assert_eq!(ticks, &Ratio::from_int(expect));
        assert!(ticks.is_integer(), "88775.244 s is a whole number of ticks");
        // ...and the published form is still there (Rule Y.1).
        assert_eq!(p.as_measured().unwrap().verbatim(), "88775.244 s");
        assert_eq!(p.citation().source, IAU.source);
    }

    #[test]
    fn every_unit_converts_exactly() {
        for (unit, mantissa, decimals) in [
            (MeasuredUnit::SiSecond, 88_775_244u128, 3u32),
            (MeasuredUnit::SiDay, 686_9726, 4),
            (MeasuredUnit::JulianYear, 1_0000, 4),
        ] {
            let m = Measured::new(mantissa, decimals, unit, IAU);
            let t = m.ticks().unwrap_or_else(|e| panic!("{:?}: {e}", unit));
            assert!(t.is_integer(), "{unit:?} did not convert to whole ticks");
        }
    }

    #[test]
    fn a_value_finer_than_the_bridge_is_refused_not_rounded() {
        // Rule Y.2: exact conversion or rejection. The bridge carries thirty
        // decimal places; the thirty-first is UCAL-E0043.
        let ok = Measured::new(1, 30, MeasuredUnit::SiSecond, IAU);
        assert!(ok.ticks().is_ok(), "thirty decimals must be exact");
        let too_fine = Measured::new(1, 31, MeasuredUnit::SiSecond, IAU);
        assert_eq!(too_fine.ticks().unwrap_err().code, Code::E0043);
    }

    #[test]
    fn a_parameter_cannot_be_built_without_what_rule_c_requires() {
        // The constructor takes the epoch and window as arguments, so a parameter
        // missing them does not typecheck. What it *can* catch at runtime is an
        // epoch outside its own window, which would make the validity claim
        // incoherent.
        let narrow = Window::new(j2000(), j2000()).unwrap();
        let far = j2000()
            .checked_add(&Delta::from_ticks(UC1::bridge().ticks))
            .unwrap();
        let e = RatedParam::new(mars_solar_day(), far, narrow).unwrap_err();
        assert_eq!(e.code, Code::E0060);
    }

    #[test]
    fn a_constant_parameter_evaluates_to_itself_everywhere_in_window() {
        let p = RatedParam::new(mars_solar_day(), j2000(), wide_window()).unwrap();
        let later = j2000()
            .checked_add(&Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
                    .unwrap(),
            ))
            .unwrap();
        let (v, w) = p.evaluate(&later).unwrap();
        assert_eq!(&v, p.value_at_epoch());
        assert_eq!(w, None, "inside the window there is no warning");
    }

    #[test]
    fn evaluation_outside_the_window_warns_and_does_not_hide_it() {
        // Rule C: "MUST warn (UCAL-W0003) and MUST NOT silently extrapolate."
        let narrow = {
            let span = Delta::from_ticks(UC1::bridge().ticks);
            Window::new(
                j2000().checked_sub(&span).unwrap(),
                j2000().checked_add(&span).unwrap(),
            )
            .unwrap()
        };
        let p = RatedParam::new(mars_solar_day(), j2000(), narrow).unwrap();

        let far = j2000()
            .checked_add(&Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
                    .unwrap(),
            ))
            .unwrap();
        let (_, w) = p.evaluate(&far).unwrap();
        assert_eq!(w, Some(Warning::W0003), "outside the window must warn");

        // A caller that would rather refuse can.
        assert!(p.evaluate_strict(&far).is_err());
        assert!(p.evaluate_strict(&j2000()).is_ok());
    }

    #[test]
    fn a_rate_makes_the_parameter_drift() {
        // F6: a derived calendar drifts because parameters were treated as
        // constants. Earth's rotation lengthens by about 1.8 ms per century.
        let day = Measured::new(86_400_002, 3, MeasuredUnit::SiSecond, IAU);
        let rate = Measured::new(18, 4, MeasuredUnit::SiSecond, IAU); // 0.0018 s/century
        let p = RatedParam::new(day, j2000(), wide_window())
            .unwrap()
            .with_rate_per_julian_century(rate)
            .unwrap();
        assert!(p.rate_per_tick().is_some());
        assert_eq!(p.rate_as_measured().unwrap().verbatim(), "0.0018 s");

        // One century on, the value must be longer by exactly the published rate.
        let century = Delta::from_ticks(
            UC1::bridge()
                .ticks
                .try_mul(&<Ticks as TickInt>::from_u64(100 * 31_557_600))
                .unwrap(),
        );
        let later = j2000().checked_add(&century).unwrap();
        let (v, _) = p.evaluate(&later).unwrap();
        let grew = v.sub(p.value_at_epoch()).unwrap();
        let expect = Measured::new(18, 4, MeasuredUnit::SiSecond, IAU).ticks().unwrap();
        assert_eq!(grew, expect, "one century of drift must equal the rate");

        // ...and a century *before* the epoch it is shorter by the same amount.
        let earlier = j2000().checked_sub(&century).unwrap();
        let (v, _) = p.evaluate(&earlier).unwrap();
        let shrank = p.value_at_epoch().sub(&v).unwrap();
        assert_eq!(shrank, expect);
    }

    #[test]
    fn drift_is_proportional_and_exact_at_arbitrary_offsets() {
        let day = Measured::new(86_400_002, 3, MeasuredUnit::SiSecond, IAU);
        let rate = Measured::new(18, 4, MeasuredUnit::SiSecond, IAU);
        let p = RatedParam::new(day, j2000(), wide_window())
            .unwrap()
            .with_rate_per_julian_century(rate)
            .unwrap();

        let century_ticks = UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(100 * 31_557_600))
            .unwrap();
        // Half a century is exactly half the drift — no rounding anywhere.
        let half = Delta::from_ticks(century_ticks.quot_rem(&<Ticks as TickInt>::from_u64(2)).0);
        let (v, _) = p.evaluate(&j2000().checked_add(&half).unwrap()).unwrap();
        let grew = v.sub(p.value_at_epoch()).unwrap();
        let full = Measured::new(18, 4, MeasuredUnit::SiSecond, IAU).ticks().unwrap();
        assert_eq!(grew.mul(&Ratio::from_u64(2)).unwrap(), full);
    }

    #[test]
    fn a_rate_that_drives_the_value_through_zero_is_refused() {
        // Extrapolating a linear rate far enough backwards makes any period
        // negative, which is not physical. Better to refuse than to return a
        // duration that cannot exist.
        let short = Measured::new(1, 0, MeasuredUnit::SiSecond, IAU);
        let rate = Measured::new(1, 0, MeasuredUnit::SiSecond, IAU);
        let p = RatedParam::new(short, j2000(), wide_window())
            .unwrap()
            .with_rate_per_julian_century(rate)
            .unwrap();
        let long_ago = j2000()
            .checked_sub(&Delta::from_ticks(
                UC1::bridge()
                    .ticks
                    .try_mul(&<Ticks as TickInt>::from_u64(31_557_600 * 1_000))
                    .unwrap(),
            ))
            .unwrap();
        assert_eq!(p.evaluate(&long_ago).unwrap_err().code, Code::E0060);
    }
}
