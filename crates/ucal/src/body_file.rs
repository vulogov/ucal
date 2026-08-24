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
//!   it leaked once per *call* until 1.8.0; it now leaks once per **distinct
//!   string**, because [`leak`] interns. A caller loading one file in a loop no
//!   longer accumulates, which was the sharp form of the objection. A caller
//!   loading a thousand different files still does, and a loaded `Body` still
//!   cannot be dropped — so this remains a trade a library does not get to make
//!   for its callers, and D-A20 does not move.
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
use ucal_core::{Citation, Code, Delta, Instant, Ratio, Ticks, TimeError, Window, UC1};

type Result<T> = core::result::Result<T, TimeError>;

/// One parameter, as it appears in a file: measured, or derived (Z1.1).
///
/// `value` and `derived` are alternatives and exactly one must be present. A
/// measured parameter states a published figure and its unit; a derived one
/// names a relation over the file's other parameters and states neither.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ParamFile {
    /// The published figure, verbatim, as a decimal string. Measured only.
    #[serde(default)]
    value: Option<String>,
    /// `s`, `d` or `yr` — SI second, SI day of 86 400 s, Julian year. Measured only.
    #[serde(default)]
    unit: Option<String>,
    /// A named relation over this file's other parameters. Derived only.
    #[serde(default)]
    derived: Option<String>,
    /// Full source reference.
    citation: String,
    /// DOI, bibcode or URL, where one exists.
    #[serde(default)]
    locator: Option<String>,
    /// Half-width of the validity window, in Julian years about J2000.0.
    valid_years: u64,
}

/// The relations a file may name in a `derived:` parameter.
///
/// # Why there is exactly one
///
/// Six of the twelve derived calendars that ship do not state a solar day. They
/// compute it, because no source publishes a solar day for a tidally locked
/// body and one follows exactly from two figures that are published:
///
/// ```text
///   solar_day = 1 / (1/P_rotation - 1/P_orbital_period)
/// ```
///
/// Until 1.5.0 a file could not say that. It could only write the result down,
/// and writing it down means rounding it, and rounding it changes the calendar:
/// Europa's rule moves through `47/105`, `2/27`, `5/126`, `5/116` and `1/24`
/// across the first six decimals. The documented example file paid that cost in
/// full — it stated a solar day the cited source does not publish, wrong in the
/// third decimal, and derived `202/279` where the body derives `1/24`.
///
/// One relation, because one is what the shipped data uses six times. A
/// vocabulary of derivations is a thing to add when a second is needed, not in
/// advance; §15.1 does not name any, so every entry here is an extension and
/// each should have to earn itself.
#[derive(Clone, Copy)]
enum Relation {
    /// The synodic day of a body whose rotation and year are both stated.
    Synodic,
}

impl Relation {
    fn parse(s: &str) -> Result<Relation> {
        match s {
            "synodic" => Ok(Relation::Synodic),
            _ => Err(TimeError::with_context(
                Code::E0018,
                "the only derivation a body file may name is `synodic`: \
                 1 / (1/rotation_period - 1/orbital_period)",
            )),
        }
    }

    /// The formula, verbatim, for the provenance record §15.2 requires.
    const fn relation(self) -> &'static str {
        match self {
            Relation::Synodic => "1 / (1/P_rotation - 1/P_orbital_period)",
        }
    }

    const fn because(self) -> &'static str {
        match self {
            Relation::Synodic => {
                "tidal lock fixes the body's face towards its primary, not towards the Sun; \
                 the Sun moves relative to the pair as the primary orbits, so a solar day is \
                 the synodic period. No source publishes it, and it follows exactly from two \
                 that are published."
            }
        }
    }

    /// Evaluate over the file's other parameters, exactly.
    fn eval(self, rotation: &RatedParam, year: &RatedParam) -> Result<Ratio> {
        match self {
            Relation::Synodic => {
                let a = rotation.value_at_epoch().recip()?;
                let b = year.value_at_epoch().recip()?;
                let d = a.abs_diff(&b)?;
                if d.is_zero() {
                    return Err(TimeError::with_context(
                        Code::E0060,
                        "this body's rotation period equals its orbital period, so it is \
                         tidally locked to its primary and its solar day is unbounded: the \
                         star does not move in its sky, and it has no day for a calendar to \
                         count",
                    ));
                }
                d.recip()
            }
        }
    }
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

/// `&'static str` from an owned one, **interned**.
///
/// # What this changes, and what it does not
///
/// The data model is `&'static str` throughout, so a runtime loader has to
/// produce one, and the only way to do that from owned data is to leak. That is
/// [D-A20]'s obstacle and it has not moved: `Body` still holds `&'static str`,
/// so a loaded body can never be dropped and its strings reclaimed.
///
/// What interning changes is the **shape** of the leak. `Box::leak` leaks once
/// per call, so loading one file twice leaked twice; this leaks once per
/// *distinct string*, so loading one file a thousand times leaks what loading it
/// once leaks. The bound moves from *number of loads* to *number of distinct
/// strings the process has seen*, which for the case that matters — a caller
/// loading calendars in a loop — is the difference between unbounded and
/// bounded.
///
/// **It is not enough to close D-A20**, and the delta stays `UNIMPLEMENTED` for
/// the library. A caller loading a thousand *different* files still accumulates,
/// and a library that leaked on its callers' behalf would still be making a
/// choice that is not its to make. What this does is remove the case that was
/// unbounded in the loop, and leave the case that is unbounded in the corpus.
///
/// # Cost
///
/// One `Mutex<HashSet<&'static str>>`, and a lookup per string. A body file
/// carries a couple of dozen; the lookup is not on any hot path, because there
/// is no hot path — this runs once per file named on a command line.
///
/// [D-A20]: https://github.com/vulogov/ucal/blob/main/spec/SPEC-DELTAS.md
pub(crate) fn leak(s: String) -> &'static str {
    use std::collections::HashSet;
    use std::sync::{Mutex, OnceLock};

    static POOL: OnceLock<Mutex<HashSet<&'static str>>> = OnceLock::new();
    let pool = POOL.get_or_init(|| Mutex::new(HashSet::new()));
    // A poisoned lock means another thread panicked mid-intern. The pool is a
    // set of immutable strings, so its contents cannot be torn; taking the inner
    // value is correct rather than merely convenient.
    let mut pool = match pool.lock() {
        Ok(g) => g,
        Err(poisoned) => poisoned.into_inner(),
    };
    if let Some(existing) = pool.get(s.as_str()) {
        return existing;
    }
    let leaked: &'static str = Box::leak(s.into_boxed_str());
    pool.insert(leaked);
    leaked
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
        // F5. Rule C asks for the published value *verbatim*, and the fact
        // sheets this project cites most publish rotation periods in hours —
        // `data::jupiter` converts one in a comment, `9.9250 h x 3600 = 35 730
        // s, exact`, which a file could not do. An author without these had to
        // convert by hand: either a rounding, and a rounded parameter is a
        // different calendar, or an exact conversion whose working the file no
        // longer shows.
        //
        // Both are exact multiples of the second, which is the condition Z1.2
        // set. A unit that was not would put a rounding inside the conversion,
        // and that is a different decision.
        "min" => Ok(MeasuredUnit::SiMinute),
        "h" => Ok(MeasuredUnit::Hour),
        "d" => Ok(MeasuredUnit::SiDay),
        "yr" => Ok(MeasuredUnit::JulianYear),
        _ => Err(TimeError::with_context(
            Code::E0018,
            "unit must be `s` (SI second), `min` (60 s), `h` (3600 s), \
             `d` (86 400 s) or `yr` (Julian year)",
        )),
    }
}

impl ParamFile {
    /// Which of the two kinds of parameter this is, refusing anything ambiguous.
    fn relation(&self) -> Result<Option<Relation>> {
        match (&self.derived, &self.value) {
            (Some(_), Some(_)) => Err(TimeError::with_context(
                Code::E0060,
                "a parameter is measured or derived, not both: it has `value` and `derived`",
            )),
            (None, None) => Err(TimeError::with_context(
                Code::E0060,
                "a parameter needs `value` and `unit`, or `derived`",
            )),
            (Some(d), None) => {
                if self.unit.is_some() {
                    return Err(TimeError::with_context(
                        Code::E0060,
                        "a derived parameter has no unit: it is computed in ticks from the \
                         parameters it names, and a unit here would be a second statement of \
                         a quantity that already has one",
                    ));
                }
                Relation::parse(d).map(Some)
            }
            (None, Some(_)) => Ok(None),
        }
    }

    /// A measured parameter. Rule C's obligations are all required fields.
    fn build(self) -> Result<RatedParam> {
        let value = self.value.ok_or_else(|| {
            TimeError::with_context(Code::E0060, "a measured parameter needs `value`")
        })?;
        let unit = self.unit.ok_or_else(|| {
            TimeError::with_context(Code::E0060, "a measured parameter needs `unit`")
        })?;
        let (mantissa, decimals) = mantissa_of(&value)?;
        let citation = Citation::new(leak(self.citation), self.locator.map(leak));
        RatedParam::new(
            Measured::new(mantissa, decimals, unit_of(&unit)?, citation),
            j2000()?,
            window(self.valid_years)?,
        )
    }

    /// A derived parameter, evaluated exactly over two that are not.
    ///
    /// The citation is still required and still carried: a derived value is not
    /// uncited, it is cited to the derivation and to the parameters underneath
    /// it, which is what `RatedParam::derived` records for §15.2.
    fn build_derived(
        self,
        r: Relation,
        rotation: &RatedParam,
        year: &RatedParam,
    ) -> Result<RatedParam> {
        let value = r.eval(rotation, year)?;
        let citation = Citation::new(leak(self.citation), self.locator.map(leak));
        RatedParam::derived(
            value,
            j2000()?,
            window(self.valid_years)?,
            r.relation(),
            r.because(),
            Box::leak(Box::new([citation])),
        )
    }
}

/// Read a body file and build the `Body` it declares.
///
/// Strict: an unknown key is `UCAL-E0012`, as §15.1 requires.
pub fn load(path: &std::path::Path) -> Result<Body> {
    let text = std::fs::read_to_string(path).map_err(|e| {
        TimeError::with_context(
            Code::E0017,
            match e.kind() {
                std::io::ErrorKind::NotFound => "no such body file",
                _ => "the body file could not be read",
            },
        )
    })?;

    let file: BodyFile = deser_hjson::from_str(&text).map_err(|e| {
        // §15.1: unknown keys are E0012. Everything else the deserialiser
        // rejects is a malformed file, which is E0017 (D-A22).
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
            TimeError::with_context(Code::E0017, "the body file is not well-formed HJSON")
        }
    })?;

    // Order matters: a derived solar day is evaluated over the other two, so
    // they are built first and the derivation reads the values they hold rather
    // than the decimals the file wrote.
    let solar_relation = file.solar_day.relation()?;
    let rotation = file.rotation_period.build()?;
    let year = file.orbital_period.build()?;
    let solar_day = match solar_relation {
        Some(r) => file.solar_day.build_derived(r, &rotation, &year)?,
        None => file.solar_day.build()?,
    };

    let mut body = Body::new(leak(file.id), rotation, solar_day, year);
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
