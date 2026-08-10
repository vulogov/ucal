//! §15.1's body file, loaded — in the binary, and only in the binary.
//!
//! # Why this is here and not in `ucal-body`
//!
//! §15.1 puts the loader in `ucal-body`, and [D-A20] records that it is not
//! there. The obstacle is not the file format. Every string in the data model is
//! `&'static str` — `Citation::new(source: &'static str, ..)` in `ucal-core`,
//! `Body::new(id: &'static str, ..)`, `Measured`'s `verbatim`, `unit` and
//! `quantity`, thirty-four sites in all. That is exactly right for data compiled
//! into the binary and admits no runtime string.
//!
//! There are two ways across, and only one of them is available in a minor
//! release:
//!
//! - **Leak.** `Box::leak` turns a `String` into a `&'static str`. It works, and
//!   it leaks once per load. In a process that loads a file and exits, that is
//!   bounded by the number of files named on the command line. In a library, a
//!   caller loading calendars in a loop leaks without bound, and handing that to
//!   every downstream user is not a trade this crate gets to make for them.
//! - **Own the strings.** `Cow<'static, str>` or `String` throughout, which is a
//!   breaking change to `ucal-core`'s public API and therefore 2.0's.
//!
//! So the loader lives in the binary, where the leak is bounded by the process
//! and no library consumer inherits it. `ucal-body` is untouched, §15.1 stays
//! `UNIMPLEMENTED` for the library, and D-A20 carries the reason.
//!
//! **This is a partial implementation of a normative requirement and is
//! described as one.** What it buys is that somebody who is not the author can
//! author a body and see what calendar it derives, which was the question
//! `X1-authoring-local-calendars.md` asked.
//!
//! # Strictness
//!
//! §15.1 says strict, unknown keys → `UCAL-E0012`. `deser-hjson` with
//! `deny_unknown_fields` does that, and it gives `E0012` its first raiser in the
//! workspace — the code was defined for this loader and had none, which is how
//! D-A20 came to be written.
//!
//! # What the file must carry
//!
//! Rule C's four obligations on every parameter: the published value verbatim,
//! its unit, the epoch it is stated at, a validity window, and a citation. A
//! format that let any of them be omitted would reintroduce the uncited constant
//! this project exists to refuse, so all of them are required fields and
//! `serde` rejects a file missing one.
//!
//! [D-A20]: https://github.com/vulogov/ucal/blob/main/spec/SPEC-DELTAS.md

use serde::Deserialize;
use ucal_body::param::{Measured, MeasuredUnit, RatedParam};
use ucal_body::Body;
use ucal_core::backend::TickInt;
use ucal_core::{Citation, Code, Delta, Instant, Ticks, TimeError, Window, UC1};

type Result<T> = core::result::Result<T, TimeError>;

/// One measured parameter, as it appears in a file.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamFile {
    /// The published figure, verbatim, as a decimal string.
    value: String,
    /// `s`, `d` or `yr` — SI second, SI day of 86 400 s, Julian year.
    unit: String,
    /// Full source reference.
    citation: String,
    /// DOI, bibcode or URL, where one exists.
    #[serde(default)]
    locator: Option<String>,
    /// Half-width of the validity window, in Julian years about J2000.0.
    valid_years: u64,
}

/// A satellite, for the grouping cycle a calendar may name.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SatelliteFile {
    id: String,
    orbital_period: ParamFile,
    #[serde(default)]
    retrograde: bool,
}

/// A body file (§15.1).
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BodyFile {
    /// Stable id, e.g. `europa`.
    id: String,
    /// What it orbits, if anything.
    #[serde(default)]
    primary: Option<String>,
    /// Sidereal rotation.
    rotation_period: ParamFile,
    /// The day the Sun makes, which is not the rotation (§8.3).
    solar_day: ParamFile,
    /// The orbit the year is measured against — for a satellite, its primary's.
    orbital_period: ParamFile,
    #[serde(default)]
    satellites: Vec<SatelliteFile>,
}

/// `&'static str` from an owned one, by leaking.
///
/// The bounded leak this module's header describes. Called once per string in a
/// loaded file, in a process that then exits.
fn leak(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

/// J2000.0, the epoch every shipped parameter is stated at.
fn j2000() -> Result<Instant<UC1>> {
    Instant::from_ticks(
        <Ticks as TickInt>::from_dec_str(
            "8070205173569972963515184424835637180530466139316558837890625",
        )
        .ok_or(TimeError::new(Code::E0021))?,
    )
}

/// A window of `± years` Julian years about J2000.0.
fn window(years: u64) -> Result<Window<UC1>> {
    use ucal_core::Profile;
    let span = Delta::from_ticks(
        UC1::bridge()
            .ticks
            .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
            .and_then(|v| v.try_mul(&<Ticks as TickInt>::from_u64(years)))
            .ok_or(TimeError::new(Code::E0021))?,
    );
    let e = j2000()?;
    Window::new(e.checked_sub(&span)?, e.checked_add(&span)?)
}

/// Split a decimal string into the mantissa and decimal count `Measured` wants.
fn mantissa_of(v: &str) -> Result<(u128, u32)> {
    let malformed = || {
        TimeError::with_context(
            Code::E0060,
            "a parameter value must be a decimal number, e.g. `86400` or `15.945421`",
        )
    };
    let v = v.trim();
    let (whole, frac) = match v.split_once('.') {
        Some((a, b)) => (a, b),
        None => (v, ""),
    };
    if whole.is_empty() || !whole.bytes().chain(frac.bytes()).all(|b| b.is_ascii_digit()) {
        return Err(malformed());
    }
    let digits = format!("{whole}{frac}");
    let mantissa: u128 = digits.parse().map_err(|_| malformed())?;
    let decimals: u32 = frac.len().try_into().map_err(|_| malformed())?;
    Ok((mantissa, decimals))
}

fn unit_of(u: &str) -> Result<MeasuredUnit> {
    match u {
        "s" => Ok(MeasuredUnit::SiSecond),
        "d" => Ok(MeasuredUnit::SiDay),
        "yr" => Ok(MeasuredUnit::JulianYear),
        _ => Err(TimeError::with_context(
            Code::E0060,
            "unit must be `s` (SI second), `d` (86 400 s) or `yr` (Julian year)",
        )),
    }
}

impl ParamFile {
    fn build(self) -> Result<RatedParam> {
        let (mantissa, decimals) = mantissa_of(&self.value)?;
        let citation = Citation::new(leak(self.citation), self.locator.map(leak));
        RatedParam::new(
            Measured::new(mantissa, decimals, unit_of(&self.unit)?, citation),
            j2000()?,
            window(self.valid_years)?,
        )
    }
}

/// Read a body file and build the `Body` it declares.
///
/// Strict: an unknown key is `UCAL-E0012`, as §15.1 requires.
pub fn load(path: &std::path::Path) -> Result<Body> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        TimeError::with_context(
            Code::E0010,
            match e.kind() {
                std::io::ErrorKind::NotFound => "no such body file",
                _ => "the body file could not be read",
            },
        )
    })?;

    let file: BodyFile = deser_hjson::from_str(&text).map_err(|e| {
        // §15.1: unknown keys are E0012. Everything else the deserialiser
        // rejects is a malformed file, which is E0010's family.
        let msg = e.to_string();
        if msg.contains("unknown field") {
            TimeError::with_context(
                Code::E0012,
                "unknown key in the body file; the accepted keys are id, primary, \
                 rotation_period, solar_day, orbital_period and satellites",
            )
        } else if msg.contains("missing field") {
            TimeError::with_context(
                Code::E0060,
                "a body file must give id, rotation_period, solar_day and \
                 orbital_period, and every parameter needs value, unit, citation \
                 and valid_years (Rule C)",
            )
        } else {
            TimeError::with_context(
                Code::E0010,
                "the body file is not well-formed HJSON",
            )
        }
    })?;

    let mut body = Body::new(
        leak(file.id),
        file.rotation_period.build()?,
        file.solar_day.build()?,
        file.orbital_period.build()?,
    );
    if let Some(p) = file.primary {
        body = body.orbiting(leak(p));
    }
    for s in file.satellites {
        body = body.with_satellite(ucal_body::body::Satellite::new(
            leak(s.id),
            s.orbital_period.build()?,
            s.retrograde,
        ));
    }
    Ok(body)
}
