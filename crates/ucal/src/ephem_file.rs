//! B — declaring an ephemeris in a file.
//!
//! # Why a file rather than a shipped catalogue
//!
//! §15.1 lets somebody who is not the author declare a body and get a calendar
//! from it, and that turned out to be the more valuable half of `ucal-body`:
//! the compiled-in list is fifteen entries, and the format is unbounded.
//!
//! The same reasoning applies here with more force. **This release ships no
//! ephemerides**, and the reason is Rule C rather than time. A shipped
//! ephemeris must quote `T_0`, `P`, and both σ verbatim from a paper (Rule Y.1),
//! and a figure typed from memory is exactly the defect `cal validate` found in
//! this project's own `europa.hjson` — which cited NASA for a solar day NASA
//! does not publish, wrong in the third decimal, deriving a different calendar.
//! [`D5`] recorded the same outcome for Titan's anchor and the answer there was
//! *no anchor*.
//!
//! So the machinery ships, the format ships, and the figures wait for somebody
//! holding the papers. That is the honest order.
//!
//! # The epoch is a Julian Date, which is why A1 came first
//!
//! Every published ephemeris states `T_0` as a `BJD`, `HJD` or `JD` with a
//! scale, so the file takes one — and the scale is **mandatory**, for the reason
//! `from-jd` makes it mandatory: a converter that defaults is silently wrong by
//! 69 seconds whenever it guesses.
//!
//! [`D5`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/D5-titan-anchor.md

use serde::Deserialize;

use ucal_core::num::Ratio;
use ucal_core::profile::Citation;
use ucal_core::{Code, Delta, Profile, Ticks, TimeError, UC1};
use ucal_core::backend::TickInt;
use ucal_events::ephem::Ephemeris;

use crate::body_file::leak;

type Result<T> = core::result::Result<T, TimeError>;

/// The `epoch:` block.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EpochFile {
    /// The published epoch, as a Julian Date.
    jd: String,
    /// Its time scale. Required — there is no default and there will not be one.
    scale: String,
    /// The published 1σ on the epoch, in days.
    sigma_days: String,
}

/// The `period:` block.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct PeriodFile {
    /// The period, in days, exactly as published.
    days: String,
    /// The published 1σ, in days.
    sigma_days: String,
    /// `Ṗ`, dimensionless. Omitted is zero, which is the usual case.
    #[serde(default)]
    pdot: Option<String>,
}

/// An ephemeris file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EphemFile {
    id: String,
    label: String,
    epoch: EpochFile,
    period: PeriodFile,
    /// The cycle range the published fit covers: `[first, last]`.
    ///
    /// **Not optional.** A fit with no stated range is a model with no domain,
    /// and it is the only field that makes `UCAL-W0003` mean anything here —
    /// without it every extrapolation looks exactly like an interpolation.
    fitted_cycles: [i64; 2],
    citation: String,
    #[serde(default)]
    locator: Option<String>,
}

/// Days, as a decimal string, in ticks. Exact or an error.
fn days_to_ticks(v: &str, what: &str) -> Result<Ratio> {
    let days = Ratio::from_decimal_str(v).map_err(|_| {
        TimeError::with_context(
            Code::E0001,
            leak(alloc_format(what, "is not a decimal number of days")),
        )
    })?;
    let day = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(86_400))
        .ok_or(TimeError::new(Code::E0021))?;
    days.mul(&Ratio::from_int(day))
}

fn alloc_format(what: &str, why: &str) -> String {
    format!("`{what}` {why}")
}

/// Load an ephemeris from a §15.x-shaped file.
pub fn load(text: &str) -> Result<Ephemeris> {
    let f: EphemFile = deser_hjson::from_str(text).map_err(|e| {
        TimeError::with_context(
            Code::E0012,
            leak(format!(
                "this ephemeris file did not parse: {e}. Unknown keys are \
                 refused rather than ignored — a misspelled key that is skipped \
                 is a parameter silently taking its default"
            )),
        )
    })?;

    // The epoch, through the same converter `from-jd` uses, with the same
    // mandatory scale and the same refusals.
    let scale = ucal_civil::jd::JdScale::parse(&f.epoch.scale)?;
    let jd = Ratio::from_decimal_str(&f.epoch.jd).map_err(|_| {
        TimeError::with_context(Code::E0001, "`epoch.jd` is not a decimal Julian Date")
    })?;
    let w = ucal_civil::jd::from_jd(&jd, scale)?;

    // A TDB epoch arrives as a window 3.4 ms wide, and that width is *not* the
    // published uncertainty — it is this project's refusal to evaluate a series.
    // Folding the two together would report one as the other, so the epoch takes
    // the window's low end and the scale's own width is added to σ instead.
    let scale_half = w
        .width()
        .ticks()
        .clone()
        .quot_rem(&<Ticks as TickInt>::from_u64(2))
        .0;
    let epoch = w.lo().clone();

    let sigma_t0 = days_to_ticks(&f.epoch.sigma_days, "epoch.sigma_days")?
        .ceil()
        .try_add(&scale_half)
        .ok_or(TimeError::new(Code::E0021))?;

    let period = days_to_ticks(&f.period.days, "period.days")?;
    let period_sigma = days_to_ticks(&f.period.sigma_days, "period.sigma_days")?;
    let pdot = match &f.period.pdot {
        None => Ratio::zero(),
        Some(v) => Ratio::from_decimal_str(v).map_err(|_| {
            TimeError::with_context(
                Code::E0001,
                "`period.pdot` is dimensionless (s/s) and must be a decimal",
            )
        })?,
    };

    if period_sigma.is_zero() && sigma_t0.is_zero_ticks() {
        return Err(TimeError::with_context(
            Code::E0018,
            "this ephemeris declares no uncertainty at all. A published \
             ephemeris has one, and an ephemeris without one predicts a point — \
             which is the claim this whole type exists to stop being made. If \
             the source really states none, say so in the citation and give the \
             last quoted digit as σ",
        ));
    }

    Ephemeris::new(
        &f.id,
        &f.label,
        epoch,
        Delta::from_ticks(sigma_t0),
        period,
        period_sigma,
        pdot,
        (f.fitted_cycles[0], f.fitted_cycles[1]),
        Citation::new(leak(f.citation), f.locator.map(leak)),
        leak(format!("{} d", f.period.days)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOOD: &str = r#"
id: example
label: an example ephemeris
epoch: {
  jd: 2452826.628521
  scale: tdb
  sigma_days: 0.000087
}
period: {
  days: 3.52474859
  sigma_days: 0.00000038
}
fitted_cycles: [-500, 500]
citation: illustrative only; see Documentation/examples/ephemeris.hjson
"#;

    #[test]
    fn a_well_formed_file_loads() {
        let e = load(GOOD).expect("loads");
        assert_eq!(e.id(), "example");
        assert_eq!(e.fitted(), (-500, 500));
        assert!(!e.period().is_zero());
    }

    /// An unknown key is refused, not ignored.
    ///
    /// A misspelled key that is skipped is a parameter silently taking its
    /// default, which for `pdot` would be a wrong prediction that looks right.
    #[test]
    fn an_unknown_key_is_refused() {
        let bad = GOOD.replace("sigma_days: 0.00000038", "sigma_dayz: 0.00000038");
        let e = load(&bad).expect_err("deny_unknown_fields");
        assert_eq!(e.code, Code::E0012);
    }

    /// The scale is mandatory, and `utc` is refused with its reason.
    #[test]
    fn the_epoch_scale_is_required_and_utc_is_refused() {
        let no_scale = GOOD.replace("  scale: tdb\n", "");
        assert!(load(&no_scale).is_err(), "a missing scale must not default");
        let utc = GOOD.replace("scale: tdb", "scale: utc");
        let e = load(&utc).expect_err("JD(UTC) is not a uniform day count");
        assert_eq!(e.code, Code::E0016);
    }

    /// An ephemeris with no uncertainty at all is refused.
    #[test]
    fn an_ephemeris_with_no_uncertainty_is_refused() {
        let none = GOOD
            .replace("sigma_days: 0.000087", "sigma_days: 0")
            .replace("sigma_days: 0.00000038", "sigma_days: 0");
        // A TDB epoch still carries the scale's own 1.7 ms, so this one is
        // refused only for a scale that converts exactly.
        let none = none.replace("scale: tdb", "scale: tt");
        let e = load(&none).expect_err("a prediction is not a point");
        assert_eq!(e.code, Code::E0018);
    }

    /// A TDB epoch's 1.7 ms is added to σ, not silently dropped.
    ///
    /// That width is this project's refusal to evaluate a series, and the
    /// published σ is the source's measurement. They are different claims, so
    /// they are combined into the uncertainty rather than one being reported as
    /// the other.
    #[test]
    fn the_scales_own_width_reaches_sigma() {
        let tdb = load(GOOD).expect("loads");
        let tt = load(&GOOD.replace("scale: tdb", "scale: tt")).expect("loads");
        assert!(
            tdb.epoch_sigma().ticks() > tt.epoch_sigma().ticks(),
            "TDB is the less certain of the two and must say so"
        );
    }
}
