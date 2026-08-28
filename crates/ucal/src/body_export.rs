//! N2 — a shipped calendar, written back out as the §15.1 file that declares it.
//!
//! # Why this exists
//!
//! §15.1 lets somebody who is not the author declare a body and get a calendar
//! from it. Nothing wrote one. An author started from
//! `Documentation/examples/europa.hjson` and edited — which is how that example
//! came to cite NASA for a solar day NASA does not publish, wrong in the third
//! decimal, and derive `202/279` where the body derives `1/24`.
//!
//! # What it is for, beyond a template
//!
//! **It makes an assertion into a command.** The claim that a file can express
//! exactly what a compiled-in body expresses was checked by hand-written test
//! fixtures: somebody typed Mars's parameters into a string literal and asserted
//! the rule came back `45/76`. That tests the fixture as much as the loader.
//!
//! Exported, the round trip is `ucal cal export mars | ucal cal derive -`, and
//! the rule must be `45/76` for the same reason the shipped one is — because the
//! same figures went in. A property a corpus mutation can attack, rather than a
//! literal somebody maintains.
//!
//! # What it must not do
//!
//! **Round a derived parameter into a decimal.** Six shipped calendars compute
//! their solar day from two published figures; writing the *result* down would
//! be writing down a rounding, and 1.9.0 measured what that costs — Europa's
//! rule moves through five different values across the first six decimals of its
//! solar day. A derived parameter exports as `derived:`, which is the whole
//! reason Z1.1 added that key.

use ucal_body::param::{MeasuredUnit, Provenance, RatedParam};
use ucal_body::{Body, Satellite};
use ucal_core::backend::TickInt;
use ucal_core::{Code, Profile, Ticks, TimeError, UC1};

/// The file's key for a unit.
///
/// Deliberately not [`MeasuredUnit::symbol`], which renders for a reader —
/// `"d (86400 s)"` — where a file needs the key its loader accepts. Two
/// spellings of one unit, and only one of them round-trips.
/// `MeasuredUnit` is `#[non_exhaustive]`, so this cannot be an exhaustive match
/// from here and needs a wildcard. The wildcard **fails** rather than guessing:
/// a unit added to the enum without a key in `unit_of` would otherwise export as
/// something the loader silently misreads, and F5 added two units to this enum
/// in one cycle.
fn unit_key(u: MeasuredUnit) -> Result<&'static str, TimeError> {
    Ok(match u {
        MeasuredUnit::SiSecond => "s",
        MeasuredUnit::SiMinute => "min",
        MeasuredUnit::Hour => "h",
        MeasuredUnit::SiDay => "d",
        MeasuredUnit::JulianYear => "yr",
        _ => {
            return Err(TimeError::with_context(
                Code::E0018,
                "this parameter's unit has no §15.1 key, so a file cannot express \
                 it. A unit was added to `MeasuredUnit` and not to the loader's \
                 `unit_of`",
            ))
        }
    })
}

/// The published figure, exactly as the file must carry it.
///
/// From the mantissa and decimal count rather than from a rendering: `4332.589`
/// has three decimals and `60189.0` has one, and a value formatted from a
/// rational would lose the distinction. Rule Y.1 says *recorded verbatim*, and
/// the trailing zero is part of what was published.
fn verbatim(mantissa: u128, decimals: u32) -> String {
    let digits = mantissa.to_string();
    let d = decimals as usize;
    if d == 0 {
        return digits;
    }
    if digits.len() <= d {
        format!("0.{}{}", "0".repeat(d - digits.len()), digits)
    } else {
        format!("{}.{}", &digits[..digits.len() - d], &digits[digits.len() - d..])
    }
}

/// The half-width of a parameter's validity window, in whole Julian years.
///
/// The loader takes `valid_years` and builds `± years` about J2000, so this is
/// that construction read backwards. A window that is not a whole number of
/// Julian years cannot be expressed by the format at all, and is reported rather
/// than rounded to one that can be — a silently narrowed window is the thing
/// GE-3 forbids, and this is the direction it would be narrowed in.
fn valid_years(p: &RatedParam) -> Result<u64, TimeError> {
    let half = p
        .valid()
        .width()
        .ticks()
        .clone()
        .quot_rem(&<Ticks as TickInt>::from_u64(2))
        .0;
    let year = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .ok_or_else(|| TimeError::new(Code::E0021))?;
    let (years, rem) = half.quot_rem(&year);
    if !rem.is_zero_ticks() {
        return Err(TimeError::with_context(
            Code::E0043,
            "this parameter's validity window is not a whole number of Julian \
             years, and §15.1's `valid_years` cannot express it. Rounding it \
             would narrow or widen a window by assumption",
        ));
    }
    years
        .to_dec_string()
        .parse()
        .map_err(|_| TimeError::new(Code::E0021))
}

/// One parameter, as an HJSON block.
fn param_block(name: &str, p: &RatedParam) -> Result<String, TimeError> {
    let years = valid_years(p)?;
    let c = p.citation();
    let mut out = format!("{name}: {{\n");
    match p.provenance() {
        Provenance::Measured(m) => {
            out.push_str(&format!("  value: {}\n", verbatim(m.mantissa, m.decimals)));
            out.push_str(&format!("  unit: {}\n", unit_key(m.unit)?));
        }
        Provenance::Derived { relation, .. } => {
            // The relation's *key*, not its formula. `relation()` renders
            // `1 / (1/P_rotation - 1/P_orbital_period)` for a reader; the file
            // says `synodic`, which is what `Relation::parse` accepts.
            let key = if relation.contains("1/P_rotation") {
                "synodic"
            } else {
                return Err(TimeError::with_context(
                    Code::E0018,
                    "this parameter is derived by a relation §15.1 has no key for, \
                     so the file cannot express it. The format names `synodic` and \
                     nothing else, deliberately",
                ));
            };
            out.push_str(&format!("  derived: {key}\n"));
        }
    }
    out.push_str(&format!("  citation: {}\n", one_line(c.source)));
    if let Some(l) = c.locator {
        out.push_str(&format!("  locator: {l}\n"));
    }
    out.push_str(&format!("  valid_years: {years}\n"));
    out.push_str("}\n");
    Ok(out)
}

/// A citation on one line.
///
/// The shipped citations are wrapped across source lines and arrive with the
/// newlines still in them; HJSON's unquoted string form ends at a newline, so a
/// citation carrying one would truncate to its first clause and the rest would
/// be a parse error. Collapsed here rather than quoted, because a quoted string
/// would then need its own escaping and the citations contain quotes.
fn one_line(s: &str) -> String {
    s.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// A satellite block.
fn satellite_block(s: &Satellite) -> Result<String, TimeError> {
    let mut out = format!("  {{\n    id: {}\n", s.id());
    let inner = param_block("orbital_period", s.orbital_period())?;
    for line in inner.lines() {
        out.push_str(&format!("    {line}\n"));
    }
    if s.is_retrograde() {
        out.push_str("    retrograde: true\n");
    }
    out.push_str("  }\n");
    Ok(out)
}

/// Write a body as the §15.1 file that declares it.
///
/// `grouping` is the calendar's cycle declaration, which is **not** part of the
/// body (D-A5). Without it the round trip preserved the leap rule and lost the
/// cycle: `mars` has two satellites and `mars-d` groups by neither, and a file
/// that only listed them would have grouped by Phobos.
pub fn body_file(body: &Body, grouping: Option<&str>) -> Result<String, TimeError> {
    let mut out = String::new();
    out.push_str(&format!(
        "# {} — a §15.1 body file, written by `ucal cal export {}`.\n\
         #\n\
         # Every figure here is the published one, verbatim, with the citation it\n\
         # came from (Rule C). A parameter shown as `derived:` is computed exactly\n\
         # from the others: writing its result down would be writing down a\n\
         # rounding, and a rounded parameter is a different calendar.\n\
         #\n\
         # `ucal cal validate <this file>` checks it; `ucal cal derive <this file>`\n\
         # derives the same calendar the compiled-in body does.\n\n",
        body.id(),
        body.id()
    ));
    out.push_str(&format!("id: {}\n", body.id()));
    if let Some(p) = body.primary() {
        out.push_str(&format!("primary: {p}\n"));
    }
    out.push_str(&param_block("rotation_period", body.rotation_period())?);
    out.push_str(&param_block("solar_day", body.solar_day())?);
    out.push_str(&param_block("orbital_period", body.orbital_period())?);

    if !body.satellites().is_empty() {
        out.push_str("satellites: [\n");
        for s in body.satellites() {
            out.push_str(&satellite_block(s)?);
        }
        out.push_str("]\n");

        // N1 — always written when there are satellites, never left implicit.
        // Omitting it means *the first listed*, which is a decision made by line
        // order; an exported file states the decision the calendar actually
        // made, including that it made none.
        out.push_str("# Which satellite groups this calendar's cycle (D-A5).\n");
        out.push_str("# Omitting this key means the first listed, which is line\n");
        out.push_str("# order deciding a calendar.\n");
        out.push_str(&format!(
            "grouping_satellite: {}\n",
            grouping.unwrap_or("none")
        ));
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A published figure keeps its trailing zero.
    ///
    /// `60189.0 d` is Neptune's year to one decimal and `60189 d` is the same
    /// number to none. Rule Y.1 records the value *verbatim*, and a file that
    /// dropped the zero would declare a figure nobody published — and, by
    /// 1.9.0's measurement, quite possibly a different calendar.
    #[test]
    fn a_published_figure_keeps_its_precision() {
        assert_eq!(verbatim(601_890, 1), "60189.0");
        assert_eq!(verbatim(4_332_589, 3), "4332.589");
        assert_eq!(verbatim(90_560, 0), "90560");
        assert_eq!(verbatim(5, 3), "0.005");
    }

    /// The unit key is the loader's, not the reader's.
    #[test]
    fn the_unit_key_is_the_one_the_loader_accepts() {
        assert_eq!(unit_key(MeasuredUnit::SiDay).expect("a key"), "d");
        assert_eq!(unit_key(MeasuredUnit::Hour).expect("a key"), "h");
        // And *not* the rendering, which carries its conversion in brackets.
        assert!(MeasuredUnit::SiDay.symbol().contains('('));
    }

    /// A citation arrives wrapped and leaves on one line.
    #[test]
    fn a_citation_is_collapsed_to_one_line() {
        assert_eq!(one_line("two\n   lines here"), "two lines here");
    }
}
