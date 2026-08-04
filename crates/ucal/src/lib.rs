//! The `ucal` command line (§19), as a library.
//!
//! Every subcommand is a pure function from parsed arguments to an [`emit::Doc`],
//! and `main` does nothing but parse, dispatch and print. That split is what
//! makes §20's golden-output tests cheap — they call the functions directly
//! rather than spawning a process — and it keeps the exit-code mapping of §19.5
//! in one place.
//!
//! Rule E holds here too: no floating-point value appears anywhere, including in
//! the human-readable output. Where the CLI shows a foreign-unit approximation it
//! is rendered from an exact rational under a stated rounding mode (Rule R).

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod cert;
pub mod emit;
pub mod style;
pub mod table;

use emit::{Doc, Value};
use ucal_core::backend::TickInt;
use ucal_core::codec::{self, Fmt, Form};
use ucal_core::num::{RatInterval, Ratio};
use ucal_core::locale::{self, LocaleId};
use ucal_core::qualified::Kind;
use ucal_core::{
    Code, Delta, Instant, Precision, Profile, Rounding, Tier, Ticks, TimeError, Ucid, UC1,
};

#[cfg(feature = "civil")]
use ucal_civil::{
    calendar::CivilCalendar,
    legacy::{Gregorian, Julian, LegacyCalendar},
    leap,
    si::{self, Scale, SubSecond},
};

/// A command's result: a document, or a diagnostic.
pub type CmdResult = Result<Doc, TimeError>;

/// Map a diagnostic to its process exit code (§19.5).
pub fn exit_code(e: &TimeError) -> i32 {
    e.code.exit_code() as i32
}

// ---------------------------------------------------------------------------
// shared parsing
// ---------------------------------------------------------------------------

/// Parse an instant from any form the CLI accepts.
///
/// Three notations, distinguished without ambiguity: a tagged text form
/// (`UC1 …` or `UC1/5 …`), a 52-character UCID, or a bare decimal tick count.
/// A truncated text form yields a coarser precision, which is returned alongside
/// so that Rule T's uncertainty is never silently discarded.
pub fn parse_instant(s: &str) -> Result<(Instant<UC1>, Precision), TimeError> {
    let s = s.trim();
    if s.starts_with(UC1::TAG) {
        return codec::parse::<UC1>(s, &Fmt::human());
    }
    if s.len() == ucal_core::UCID_LEN && s.bytes().all(|b| b.is_ascii_alphanumeric()) {
        let u = Ucid::parse(s)?;
        return Ok((Instant::<UC1>::from_ucid(&u)?, Precision::Tick));
    }
    if !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()) {
        let t = <Ticks as TickInt>::from_dec_str(s)
            .ok_or(TimeError::with_context(Code::E0001, "tick count out of range"))?;
        return Ok((Instant::<UC1>::from_ticks(t)?, Precision::Tick));
    }
    Err(TimeError::with_context(
        Code::E0001,
        "expected a UC1 text form, a 52-character UCID, or a decimal tick count",
    ))
}

/// Parse a tier by name, `T<k>` or `5^e` (Rule N).
pub fn parse_tier(s: &str) -> Result<Tier, TimeError> {
    codec::resolve_tier_name(s)
}

/// Parse a tier in a stated locale (Rule N).
///
/// `--locale` was reaching only the *display* of tier names, so `--step пролёт`
/// failed under `--locale ru` while `--step span` worked in every locale. Rule N
/// makes names display aliases, which is a statement about what decides
/// behaviour — not licence for one locale's aliases to be the only ones a parser
/// accepts.
///
/// The stable keys and `T[k]`/`5^e` continue to resolve in every locale, so
/// nothing that worked before stops working.
pub fn parse_tier_in(locale: LocaleId, s: &str) -> Result<Tier, TimeError> {
    codec::resolve_tier_name_in(locale, s)
}

/// Parse a rounding mode.
pub fn parse_rounding(s: &str) -> Result<Rounding, TimeError> {
    match s {
        "trunc" => Ok(Rounding::Trunc),
        "ceil" => Ok(Rounding::Ceil),
        "half-even" => Ok(Rounding::HalfEven),
        "half-up" => Ok(Rounding::HalfUp),
        _ => Err(TimeError::with_context(
            Code::E0024,
            "rounding must be one of trunc, ceil, half-even, half-up",
        )),
    }
}

/// Render an exact rational to a decimal string under a stated mode (Rule R).
fn dec(r: &Ratio, digits: u32) -> String {
    r.to_decimal_string(digits, Rounding::HalfEven)
        .unwrap_or_else(|_| r.to_ratio_string())
}

#[cfg(test)]
pub(crate) fn ratio_of(a: &Ticks, b: &Ticks) -> Ratio {
    tick_ratio(a, b)
}

fn tick_ratio(a: &Ticks, b: &Ticks) -> Ratio {
    Ratio::new(a.clone(), b.clone()).expect("non-zero denominator")
}

/// Render an instant at a stated tier, choosing a form that can express it.
///
/// The human form anchors at T0 and cannot state a coarser precision (D-A8), and
/// a timeline or a ruler lives mostly above T0. Falling back to the named form —
/// which carries its tiers explicitly and so needs no anchor — keeps every tier
/// renderable.
///
/// Written as a helper because the alternative was `unwrap_or_default()` at each
/// call site, which turns a *rejected precision* into an empty string. A silent
/// blank is the worst of the available failures: it looks like a value.
fn render_at(t: &Instant<UC1>, tier: Tier) -> String {
    let fmt = if tier.index() > 0 {
        Fmt {
            form: Form::Named,
            precision: Precision::Tier(tier),
            ..Fmt::default()
        }
    } else {
        Fmt::human_at(tier)
    };
    match codec::render(t, &fmt) {
        Ok(s) => s,
        // Should be unreachable given the choice above; say so rather than
        // returning a blank that reads as a value.
        Err(e) => format!("<unrenderable at {tier}: {e}>"),
    }
}

// ---------------------------------------------------------------------------
// ucal datum (§19.2)
// ---------------------------------------------------------------------------

/// `ucal datum` — the datum statement, the claim, the provenance, the residual.
///
/// §19.2 fixes the **order**: the datum statement, then `BIG_BANG_CLAIM` with its
/// citation, then the full provenance chain from §2.2, then the rounding
/// residual. And it forbids presenting the implied age as a measurement of the
/// universe — so the implied age appears under a heading that says what it is:
/// a consequence of the declared datum (Rule Q.1).
pub fn cmd_datum() -> CmdResult {
    let prov = UC1::datum_provenance()?;
    let claim = UC1::big_bang_claim();
    let citation = UC1::big_bang_claim_citation();

    let mut doc = Doc::new()
        .title("Profile UC-1 — the datum")
        // 1. the datum statement
        .field("datum", Value::text(UC1::datum_statement()))
        .field("frame", Value::text(UC1::FRAME.describe()))
        .field("tick_zero", Value::number("0"));

    // 2. BIG_BANG_CLAIM, with citation
    let half = claim.hi().magnitude().ticks();
    doc = doc.field(
        "big_bang_claim",
        Value::Section(vec![
            ("window".into(), Value::text(claim.describe())),
            ("half_width_ticks".into(), Value::number(half.to_dec_string())),
            (
                "half_width_drifts".into(),
                Value::quantity(&tick_ratio(half, &Tier::DRIFT.ticks()), 2, Rounding::HalfEven),
            ),
            ("citation".into(), Value::text(citation.source)),
            (
                "locator".into(),
                Value::text(citation.locator.unwrap_or("—")),
            ),
            (
                "status".into(),
                Value::text(
                    "metadata only; no arithmetic operation may consume it (Rule Q.3)",
                ),
            ),
        ]),
    );

    // 3. the provenance chain
    doc = doc.field(
        "datum_provenance",
        Value::Section(vec![
            (
                "input".into(),
                Value::text(format!(
                    "{} {} ± {} ({})",
                    prov.input.verbatim,
                    prov.input.unit,
                    prov.input.uncertainty.unwrap_or("—"),
                    prov.input.quantity
                )),
            ),
            ("citation".into(), Value::text(prov.input.citation.source)),
            (
                "unit_defs".into(),
                Value::list(
                    prov.unit_defs
                        .iter()
                        .map(|(k, v)| format!("{k} = {v}")),
                ),
            ),
            ("chain".into(), Value::list(prov.chain.iter().copied())),
        ]),
    );

    // 4. the rounding residual
    let r = prov.rounding;
    doc = doc.field(
        "rounding",
        Value::Section(vec![
            ("to".into(), Value::text(r.to)),
            ("mode".into(), Value::text(r.mode)),
            ("residual_ticks".into(), Value::number(r.residual_ticks)),
            ("residual_rendered".into(), Value::text(r.residual_rendered)),
            ("rationale".into(), Value::text(r.rationale)),
        ]),
    );

    doc = doc.field("earth_dependency", Value::text(prov.earth_dependency));

    // The implied age, labelled as what it is. §19.2: this must not be presented
    // as a measurement of the universe.
    let bridge = UC1::bridge();
    let implied_s = tick_ratio(&UC1::origin_offset(), &bridge.ticks);
    doc = doc.field(
        "implied_age",
        Value::Section(vec![
            (
                "note".into(),
                Value::text(
                    "a consequence of the declared datum, not a measurement \
                     (Rule Q.1). The measurement is the `input` above.",
                ),
            ),
            ("seconds".into(), Value::quantity(&implied_s, 6, Rounding::HalfEven)),
        ]),
    );

    Ok(doc.note(
        "Changing the datum, BIG_BANG_CLAIM or datum_provenance produces a new \
         profile; Rule P then keeps values from the two from mixing.",
    ))
}

// ---------------------------------------------------------------------------
// ucal doctor (§19.3)
// ---------------------------------------------------------------------------

/// `ucal doctor` — profile, backend, domain ceiling, leap table, features,
/// provenance presence (§19.3).
pub fn cmd_doctor() -> CmdResult {
    let backend = if cfg!(feature = "bigint") {
        "bigint (num-bigint, heap; Instant is not Copy)"
    } else {
        "u512 (bnum, stack, const-constructible; Instant is Copy)"
    };
    let features: Vec<&str> = {
        let mut f = Vec::new();
        if cfg!(feature = "u512") {
            f.push("u512");
        }
        if cfg!(feature = "bigint") {
            f.push("bigint");
        }
        if cfg!(feature = "std") {
            f.push("std");
        }
        if cfg!(feature = "civil") {
            f.push("civil");
        }
        f
    };

    let provenance_present = UC1::datum_provenance().is_ok();
    let domain_max = <Ticks as TickInt>::domain_max();

    let mut doc = Doc::new()
        .title("ucal doctor")
        .field("profile", Value::text(UC1::TAG))
        .field("frame", Value::text(UC1::FRAME.describe()))
        .field("backend", Value::text(backend))
        .field("domain_max_ticks", Value::number(domain_max.to_dec_string()))
        .field("domain_bits", Value::number(ucal_core::DOMAIN_BITS.to_string()))
        .field("features", Value::list(features))
        .field(
            "datum_provenance",
            Value::Section(vec![
                ("present".into(), Value::Bool(provenance_present)),
                (
                    "note".into(),
                    Value::text(if provenance_present {
                        "present; absence would be UCAL-E0013 (Rule Q.4)"
                    } else {
                        "ABSENT — UCAL-E0013 (Rule Q.4)"
                    }),
                ),
            ]),
        );

    #[cfg(feature = "civil")]
    {
        doc = doc.field(
            "leap_seconds",
            Value::Section(vec![
                ("table_version".into(), Value::text(leap::leap_table_version())),
                (
                    "entries".into(),
                    Value::number(leap::leap_count().to_string()),
                ),
                (
                    "complete_through".into(),
                    Value::text(format!(
                        "{:04}-{:02}-{:02}",
                        leap::TABLE_COMPLETE_THROUGH.0,
                        leap::TABLE_COMPLETE_THROUGH.1,
                        leap::TABLE_COMPLETE_THROUGH.2
                    )),
                ),
                (
                    "pre_1972".into(),
                    Value::text(
                        "the 1961-1972 rubber-second era is modelled exactly; UTC \
                         before 1961-01-01 is UCAL-E0041",
                    ),
                ),
                (
                    "network".into(),
                    Value::text("never; the table is bundled and offline (§8.4)"),
                ),
            ]),
        );
    }

    doc = doc.field(
        "spec",
        Value::Section(vec![
            ("rfc".into(), Value::text(ucal_core::RFC)),
            (
                "deltas".into(),
                Value::list(ucal_core::SPEC_DELTAS.iter().copied()),
            ),
        ]),
    );

    Ok(doc.note("No network access is performed by any command (§8.4)."))
}

// ---------------------------------------------------------------------------
// ucal explain (§19)
// ---------------------------------------------------------------------------

/// `ucal explain <T> [--claim]` — what an instant is, in several registers.
pub fn cmd_explain(input: &str, show_claim: bool) -> CmdResult {
    let (t, precision) = parse_instant(input)?;
    let bridge = UC1::bridge();

    let mut tiers: Vec<(String, Value)> = Vec::new();
    for tier in [
        Tier::DEEP,
        Tier::DRIFT,
        Tier::SPAN,
        Tier::SWEEP,
        Tier::ARC,
        Tier::BEAT,
    ] {
        let name = ucal_core::tier::name_of(tier)
            .map(|n| n.key())
            .unwrap_or("—");
        tiers.push((
            format!("{tier} {name}"),
            Value::number(t.tier_value(tier).to_string()),
        ));
    }

    let mut doc = Doc::new()
        .title("ucal explain")
        .field("ticks", Value::number(t.ticks().to_dec_string()))
        .field(
            "precision",
            Value::text(match precision {
                Precision::Tick => "tick (exact)".to_string(),
                Precision::Tier(k) => format!("{k} — denotes a window (Rule T)"),
            }),
        )
        .field(
            "human",
            Value::form(codec::render(&t, &Fmt::human()).unwrap_or_default()),
        )
        .field(
            "digit5",
            Value::form(codec::render(&t, &Fmt::digit5()).unwrap_or_default()),
        )
        .field(
            "ucid",
            Value::form(match t.to_ucid() {
                Ok(u) => u.to_string(),
                Err(_) => "— (outside 2^256, UCAL-E0031)".to_string(),
            }),
        )
        .field("tiers", Value::Section(tiers));

    // The window a truncated statement denotes (Rule T).
    if !matches!(precision, Precision::Tick) {
        let w = t.window_at(precision)?;
        doc = doc.field(
            "window",
            Value::Section(vec![
                ("lo_ticks".into(), Value::number(w.lo().ticks().to_dec_string())),
                ("hi_ticks".into(), Value::number(w.hi().ticks().to_dec_string())),
                (
                    "width_ticks".into(),
                    Value::number(w.width().ticks().to_dec_string()),
                ),
            ]),
        );
    }

    // The universal second first (§0.5): how many beats since the datum. Exact,
    // because the datum is a whole beat count and every tier is a power of five.
    let (beats, rem) = t.ticks().quot_rem(&UC1::beat());
    doc = doc.field(
        "beats_since_datum",
        Value::Section(vec![
            ("whole".into(), Value::number(beats.to_dec_string())),
            ("remainder_ticks".into(), Value::number(rem.to_dec_string())),
            (
                "note".into(),
                Value::text(
                    "the beat is the universe second (§0.5), 5^60 ticks; this count \
                     carries no Earth content",
                ),
            ),
        ]),
    );

    // The bridge equivalent, shown alongside (§4.3) — never instead.
    let since_epoch = if t.ticks() >= &UC1::origin_offset() {
        let d = t.ticks().try_sub(&UC1::origin_offset()).expect("ge");
        format!("+{}", dec(&tick_ratio(&d, &bridge.ticks), 6))
    } else {
        let d = UC1::origin_offset().try_sub(t.ticks()).expect("lt");
        format!("-{}", dec(&tick_ratio(&d, &bridge.ticks), 6))
    };
    doc = doc.field(
        "si_bridge",
        Value::Section(vec![
            ("unit".into(), Value::text(bridge.name)),
            ("epoch".into(), Value::text(bridge.epoch_label)),
            ("seconds_from_epoch".into(), Value::text(since_epoch)),
        ]),
    );

    // §10.6: a quantity inside the claim half-width warrants UCAL-W0006.
    let half = UC1::big_bang_claim().hi().magnitude().ticks().clone();
    if t.ticks() < &half {
        doc = doc.field(
            "warning",
            Value::text(
                "UCAL-W0006: this instant lies within BIG_BANG_CLAIM's half-width, \
                 where the datum's own physical identification is comparable to or \
                 larger than the quantity being discussed (§10.6). The arithmetic \
                 is unaffected.",
            ),
        );
    }

    if show_claim {
        let claim = UC1::big_bang_claim();
        doc = doc.field(
            "claim",
            Value::Section(vec![
                ("window".into(), Value::text(claim.describe())),
                (
                    "citation".into(),
                    Value::text(UC1::big_bang_claim_citation().source),
                ),
                (
                    "status".into(),
                    Value::text("reportable metadata; never an operand (Rule Q.3)"),
                ),
            ]),
        );
    }

    Ok(doc)
}

// ---------------------------------------------------------------------------
// ucal now / from-civil / to-civil (§19, feature `civil`)
// ---------------------------------------------------------------------------

/// Parse a civil date such as `2026-07-29`, `2026-07-29T12:34:56.5`,
/// `-0043-03-15`, or `44 BC-03-15` (§8.5, §2.5).
#[cfg(feature = "civil")]
pub fn parse_civil(s: &str) -> Result<(i64, u8, u8, u8, u8, u8, SubSecond), TimeError> {
    let s = s.trim();
    let malformed = TimeError::with_context(Code::E0041, "expected YYYY-MM-DD[THH:MM:SS[.frac]]");

    // Era suffix on the year: `44 BC-03-15`, `2026 CE-07-29`.
    let (year_part, rest, era_bc) = if let Some(i) = s.find(" BC-").or_else(|| s.find(" BCE-")) {
        let (y, r) = s.split_at(i);
        (y, r.trim_start_matches(" BC").trim_start_matches("E"), true)
    } else if let Some(i) = s.find(" AD-").or_else(|| s.find(" CE-")) {
        let (y, r) = s.split_at(i);
        (y, r.trim_start_matches(" AD").trim_start_matches(" CE"), false)
    } else {
        // Astronomical numbering; a leading '-' is part of the year.
        let neg = s.starts_with('-');
        let body = if neg { &s[1..] } else { s };
        let i = body.find('-').ok_or(malformed)?;
        let (y, r) = body.split_at(i);
        let y_owned = if neg { format!("-{y}") } else { y.to_string() };
        return finish_civil(&y_owned, r, false);
    };
    finish_civil(year_part, rest, era_bc)
}

#[cfg(feature = "civil")]
fn finish_civil(
    year: &str,
    rest: &str,
    era_bc: bool,
) -> Result<(i64, u8, u8, u8, u8, u8, SubSecond), TimeError> {
    let malformed = TimeError::with_context(Code::E0041, "expected YYYY-MM-DD[THH:MM:SS[.frac]]");
    let mut year: i64 = year.trim().parse().map_err(|_| malformed)?;
    if era_bc {
        // §2.5: astronomical numbering. 1 BC is year 0, so n BC is year 1 - n.
        year = 1 - year;
    }
    let rest = rest.trim_start_matches('-');
    let (date, time) = match rest.split_once('T') {
        None => (rest, ""),
        Some((d, t)) => (d, t),
    };
    let mut dp = date.split('-');
    let month: u8 = dp.next().ok_or(malformed)?.parse().map_err(|_| malformed)?;
    let day: u8 = dp.next().ok_or(malformed)?.parse().map_err(|_| malformed)?;
    if dp.next().is_some() {
        return Err(malformed);
    }

    if time.is_empty() {
        return Ok((year, month, day, 0, 0, 0, SubSecond::zero()));
    }
    let time = time.trim_end_matches('Z');
    let (sec_part, sub) = match time.split_once('.') {
        None => (time, SubSecond::zero()),
        Some((t, f)) => (t, SubSecond::parse(f)?),
    };
    let mut tp = sec_part.split(':');
    let hour: u8 = tp.next().ok_or(malformed)?.parse().map_err(|_| malformed)?;
    let minute: u8 = tp.next().unwrap_or("0").parse().map_err(|_| malformed)?;
    let second: u8 = tp.next().unwrap_or("0").parse().map_err(|_| malformed)?;
    Ok((year, month, day, hour, minute, second, sub))
}

/// `ucal from-civil <DATE>` (§19).
#[cfg(feature = "civil")]
pub fn cmd_from_civil(date: &str, scale: Scale, cal: CivilCalendar) -> CmdResult {
    let (y, mo, d, h, mi, s, sub) = parse_civil(date)?;
    let t = si::from_civil(y, mo, d, h, mi, s, sub, scale, cal)?;
    Ok(instant_doc("ucal from-civil", &t)
        .field(
            "input",
            Value::Section(vec![
                ("label".into(), Value::text(date)),
                ("scale".into(), Value::text(format!("{scale:?}").to_lowercase())),
                (
                    "calendar".into(),
                    Value::text(format!("{cal:?}").to_lowercase()),
                ),
                (
                    "exactness".into(),
                    Value::text("exact; construction never rounds (Rule R)"),
                ),
            ]),
        ))
}

/// `ucal to-civil <T>` (§19).
#[cfg(feature = "civil")]
pub fn cmd_to_civil(
    input: &str,
    scale: Scale,
    digits: u8,
    rounding: Rounding,
    cal: CivilCalendar,
) -> CmdResult {
    let (t, _) = parse_instant(input)?;
    // §6.6: a local calendar rendering must carry its id and kind. Routing
    // through the legacy calendar is what makes that unavoidable.
    let legacy: &dyn LegacyCalendar = match cal {
        CivilCalendar::Gregorian => &Gregorian,
        CivilCalendar::Julian => &Julian,
    };
    let rendered = legacy.render(&t, scale, digits, rounding)?;
    let f = legacy.fields(&t, scale, digits, rounding)?;

    let mut doc = Doc::new()
        .title("ucal to-civil")
        .field("ticks", Value::number(t.ticks().to_dec_string()))
        // The qualified form, which is the only renderable one.
        .field("qualified", Value::text(rendered.to_string()))
        .field("calendar_id", Value::text(rendered.qualifier().id()))
        .field(
            "kind",
            Value::text(match rendered.qualifier().kind() {
                Kind::Legacy => "legacy — declared table data, outside Rule K (§8.6)",
                Kind::Derived => "derived",
            }),
        )
        .field(
            "fields",
            Value::Section(vec![
                ("year".into(), Value::number(f.year.to_string())),
                ("month".into(), Value::number(f.month.to_string())),
                ("day".into(), Value::number(f.day.to_string())),
                ("hour".into(), Value::number(f.hour.to_string())),
                ("minute".into(), Value::number(f.minute.to_string())),
                ("second".into(), Value::number(f.second.to_string())),
                (
                    "weekday".into(),
                    Value::text(ucal_civil::legacy::WEEKDAY_NAMES[f.weekday as usize]),
                ),
            ]),
        )
        .field("rounding", Value::text(format!("{rounding:?}").to_lowercase()))
        .field("lossy", Value::Bool(f.lossy));

    if let Some(w) = rendered.warning() {
        doc = doc.field("warning", Value::text(w.to_string()));
    }
    if f.second == 60 {
        doc = doc.note(
            "This label falls in a leap second. UTC labels are not unique across \
             one; absolute time is (Rule L).",
        );
    }
    Ok(doc)
}

/// `ucal now` — the system clock, converted through the bundled leap table.
///
/// §8.4: the clock is read as UTC and converted offline. Unix time does not count
/// leap seconds, so its value is a *label-linear* count and is converted as a UTC
/// label rather than as an elapsed duration — which is exactly the distinction
/// Rule L exists to keep visible.
#[cfg(all(feature = "civil", feature = "std"))]
pub fn cmd_now(precision: Tier, form: Form) -> CmdResult {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TimeError::with_context(Code::E0020, "system clock precedes the Unix epoch"))?;
    let unix_secs = d.as_secs() as i64;
    let nanos = d.subsec_nanos();

    let days = 719_528 + unix_secs.div_euclid(86_400);
    let sod = unix_secs.rem_euclid(86_400);
    let (y, mo, dd) = ucal_civil::calendar::civil_from_days(days, CivilCalendar::Gregorian);
    let t = si::from_civil(
        y,
        mo,
        dd,
        (sod / 3_600) as u8,
        ((sod % 3_600) / 60) as u8,
        (sod % 60) as u8,
        SubSecond::new(nanos as u128, 9)?,
        Scale::Utc,
        CivilCalendar::Gregorian,
    )?;

    let fmt = Fmt {
        form,
        precision: if precision.is_tick() {
            Precision::Tick
        } else {
            Precision::Tier(precision)
        },
        pad: matches!(form, Form::Digit5),
        ..Fmt::default()
    };
    let mut doc = instant_doc("ucal now", &t);
    if let Ok(r) = codec::render(&t, &fmt) {
        doc = doc.field("rendered", Value::form(r));
    }
    Ok(doc
        .field("precision", Value::text(precision.to_string()))
        .field(
            "source",
            Value::Section(vec![
                ("clock".into(), Value::text("system, read as UTC")),
                (
                    "leap_table".into(),
                    Value::text(leap::leap_table_version()),
                ),
                ("network".into(), Value::text("none (§8.4)")),
            ]),
        ))
}

fn instant_doc(title: &str, t: &Instant<UC1>) -> Doc {
    Doc::new()
        .title(title)
        .field("ticks", Value::number(t.ticks().to_dec_string()))
        .field(
            "human",
            Value::form(codec::render(t, &Fmt::human()).unwrap_or_default()),
        )
        .field(
            "ucid",
            Value::form(match t.to_ucid() {
                Ok(u) => u.to_string(),
                Err(_) => "— (outside 2^256)".to_string(),
            }),
        )
}

// ---------------------------------------------------------------------------
// ucal ladder (§19)
// ---------------------------------------------------------------------------

/// `ucal ladder [--locale ru]` — the universal tier grid (§4.2).
///
/// §4.2: the tier grid is the **universal** ladder, body-independent, and the
/// canonical way to state any duration. Calendar units are a local overlay and
/// never appear here.
///
/// The SI equivalent is printed alongside every row, because §4.3 concedes the
/// point: nothing on the ladder is near a second or an hour, and that is the
/// accepted cost of leaving the Earth paradigm (D-2). A reader needs the bridge
/// to get any purchase at all.
pub fn cmd_ladder(loc: LocaleId, named_only: bool) -> CmdResult {
    let bridge = UC1::bridge();
    let mut rows: Vec<(String, Value)> = Vec::new();

    for k in (ucal_core::tier::K_MIN..=ucal_core::tier::K_MAX).rev() {
        let tier = Tier::new(k)?;
        let names = locale::names_of(loc, tier);
        if named_only && names.is_none() {
            continue;
        }
        let in_bridge = tick_ratio(&tier.ticks(), &bridge.ticks);
        let in_beats = tick_ratio(&tier.ticks(), &UC1::beat());
        rows.push((
            tier.to_string(),
            Value::Section(vec![
                ("exponent".into(), Value::number(tier.exponent().to_string())),
                (
                    "name".into(),
                    Value::text(match names {
                        Some(n) => format!("{} / {}", n.singular, n.plural),
                        // D-20: unnamed, and addressable by index (Rule N).
                        None => format!("— (address as {tier} or 5^{})", tier.exponent()),
                    }),
                ),
                // The universal second first: the beat is 5^60 ticks and carries
                // no Earth content (§0.5). Every tier is a whole power of five of
                // it, so these are exact.
                ("beats".into(), Value::quantity(&in_beats, 6, Rounding::HalfEven)),
                // The bridge equivalent second, printed alongside as §4.3
                // requires — and only ever alongside.
                (
                    format!("{}s (bridge)", bridge.name),
                    Value::quantity(&in_bridge, 6, Rounding::HalfEven),
                ),
                ("ticks".into(), Value::number(tier.ticks().to_dec_string())),
            ]),
        ));
    }

    Ok(Doc::new()
        .title(format!("ucal ladder — locale {}", loc.tag()))
        .field("locale", Value::text(loc.tag()))
        .field(
            "note",
            Value::text(
                "the universal ladder (§4.2): body-independent, and the canonical \
                 way to state any duration. Names are display-only (Rule N); the \
                 canonical identity of a tier is its exponent.",
            ),
        )
        .field("tiers", Value::rows("tier", rows))
        .note(
            "The beat is the universe second (§0.5): 5^60 ticks, a pure power of \
             the tick with no Earth content. The bridge second is a declared \
             foreign unit (Rule A.3) and is shown only alongside.",
        )
        .note(
            "The two seconds are incommensurable above T-6: one bridge second is \
             21.385061835 beats, not a whole number, because BEAT carries 5^60 \
             while SECOND carries only 5^30. They share a common measure only at \
             the tick, which is why Rule A.1 makes the tick primitive.",
        ))
}

// ---------------------------------------------------------------------------
// ucal cal * and ucal show --calendars (§19.4)
// ---------------------------------------------------------------------------

#[cfg(feature = "body")]
use ucal_body::{anchors, calendar as bodycal};

/// `ucal cal list` — every calendar, with its kind (§19.4).
///
/// §19.4: "`ucal cal list` MUST display `kind` for every entry." Legacy and
/// derived calendars appear in one list precisely so the distinction is visible;
/// hiding either would be the confusion Rule K exists to prevent.
#[cfg(all(feature = "body", feature = "civil"))]
pub fn cmd_cal_list() -> CmdResult {
    use ucal_core::qualified::CalendarIdentity;
    let mut rows: Vec<(String, Value)> = Vec::new();

    for c in bodycal::all() {
        let rule = c.leap_rule();
        rows.push((
            c.id().to_string(),
            Value::Section(vec![
                ("kind".into(), Value::text("derived — Rule K")),
                ("body".into(), Value::text(c.body().id())),
                (
                    "anchor_revision".into(),
                    Value::number(c.anchor().revision().to_string()),
                ),
                (
                    "leap_rule".into(),
                    Value::text(format!(
                        "{}/{} (convergent {})",
                        rule.chosen.value.numer().to_dec_string(),
                        rule.chosen.value.denom().to_dec_string(),
                        rule.depth
                    )),
                ),
                (
                    "cycles".into(),
                    Value::text(match c.cycles().first() {
                        None => "none — the calendar names no grouping satellite".to_string(),
                        Some(cy) => format!("from {}", cy.satellite),
                    }),
                ),
            ]),
        ));
    }

    // Calendars whose body is known but whose phase is not (Rule J.3).
    for id in ["titan-d"] {
        if anchors::for_calendar(id).is_none() {
            rows.push((
                id.to_string(),
                Value::Section(vec![
                    ("kind".into(), Value::text("derived — Rule K")),
                    (
                        "status".into(),
                        Value::text(
                            "no anchor: complete in units, intercalation and cycles, \
                             incomplete in phase. Asking for local fields is \
                             UCAL-E0062 (Rule J.3).",
                        ),
                    ),
                ]),
            ));
        }
    }

    for c in [&Gregorian as &dyn LegacyCalendar, &Julian] {
        rows.push((
            c.id().to_string(),
            Value::Section(vec![
                ("kind".into(), Value::text("legacy — declared tables (§8.6)")),
                (
                    "arbitrary".into(),
                    Value::number(c.tables().arbitrary.len().to_string()),
                ),
                (
                    "leap_rule".into(),
                    Value::text(format!(
                        "{}/{} ({})",
                        c.tables().leap_rule.numerator,
                        c.tables().leap_rule.denominator,
                        if c.tables().leap_rule.is_convergent {
                            "a convergent"
                        } else {
                            "NOT a convergent — declared, not derived"
                        }
                    )),
                ),
            ]),
        ));
    }

    Ok(Doc::new()
        .title("ucal cal list")
        .field("calendars", Value::rows("calendar", rows))
        .note(
            "A derived calendar is a consequence of a body's periods (Rule K). A \
             legacy one is a declared table preserved for interoperation (§8.6) \
             and is outside that mechanism.",
        ))
}

/// `ucal show <T> --calendars …` — one instant, several local renderings (§19.4).
///
/// §19.4 calls this "the primary demonstration of Rules K and J": one absolute
/// instant, rendered in several local calendars, each carrying its anchor
/// revision and uncertainty window, with legacy Gregorian shown alongside and
/// labelled.
#[cfg(all(feature = "body", feature = "civil"))]
pub fn cmd_show(input: &str, calendars: &[String]) -> CmdResult {
    let (t, _) = parse_instant(input)?;
    let mut rows: Vec<(String, Value)> = Vec::new();

    for id in calendars {
        let entry = match id.as_str() {
            "earth-civil" | "earth-julian" => {
                let c: &dyn LegacyCalendar = if id == "earth-civil" {
                    &Gregorian
                } else {
                    &Julian
                };
                let r = c.render(&t, Scale::Tt, 0, Rounding::Trunc)?;
                Value::Section(vec![
                    ("rendered".into(), Value::text(r.to_string())),
                    ("kind".into(), Value::text("legacy (§8.6)")),
                    (
                        "warning".into(),
                        Value::text(
                            r.warning().map(|w| w.to_string()).unwrap_or_default(),
                        ),
                    ),
                    (
                        "note".into(),
                        Value::text("declared tables; not a Rule K derivation"),
                    ),
                ])
            }
            other => match bodycal::by_id(other) {
                Err(e) => Value::Section(vec![
                    ("rendered".into(), Value::text("—")),
                    ("kind".into(), Value::text("derived (Rule K)")),
                    ("error".into(), Value::text(e.to_string())),
                ]),
                Ok(c) => {
                    let r = c.render(&t)?;
                    let f = c.fields(&t)?;
                    Value::Section(vec![
                        ("rendered".into(), Value::text(r.to_string())),
                        ("kind".into(), Value::text("derived (Rule K)")),
                        (
                            "anchor_revision".into(),
                            Value::number(f.anchor_revision.to_string()),
                        ),
                        (
                            "window_ticks".into(),
                            Value::number(f.window.width().ticks().to_dec_string()),
                        ),
                        (
                            "day_is_ambiguous".into(),
                            Value::Bool(f.day_is_ambiguous),
                        ),
                    ])
                }
            },
        };
        rows.push((id.clone(), entry));
    }

    Ok(Doc::new()
        .title("ucal show")
        .field("ticks", Value::number(t.ticks().to_dec_string()))
        .field(
            "human",
            Value::text(codec::render(&t, &Fmt::human()).unwrap_or_default()),
        )
        .field("calendars", Value::rows("calendar", rows))
        .note(
            "One instant, several local calendars. Each derived rendering carries \
             its anchor revision (Rule J.5) and the width of the window that \
             revision implies (Rule J.2); each legacy one is labelled as declared \
             table data (§8.6).",
        ))
}

/// `ucal cal show <id> <T>` — one calendar's derivation, in full.
#[cfg(feature = "body")]
pub fn cmd_cal_show(id: &str, input: &str) -> CmdResult {
    let (t, _) = parse_instant(input)?;
    let c = bodycal::by_id(id)?;
    let f = c.fields(&t)?;
    let rule = c.leap_rule();

    // §15.2: the whole walk, so the choice is auditable.
    let walked: Vec<String> = rule
        .walked
        .iter()
        .enumerate()
        .map(|(i, cv)| {
            format!(
                "{}: {}/{} — 1 day slips in {} local years{}",
                i + 1,
                cv.value.numer().to_dec_string(),
                cv.value.denom().to_dec_string(),
                cv.one_day_slips_in
                    .as_ref()
                    .map(|r| r.to_decimal_string(0, Rounding::HalfEven).unwrap_or_default())
                    .unwrap_or_else(|| "never (exact)".into()),
                if i + 1 == rule.depth { "   <- chosen" } else { "" }
            )
        })
        .collect();

    let mut doc = Doc::new()
        .title(format!("ucal cal show {id}"))
        .field("calendar", Value::text(id))
        .field("kind", Value::text("derived — Rule K"))
        .field("body", Value::text(c.body().id()))
        .field(
            "anchor",
            Value::Section(vec![
                ("phase".into(), Value::text(c.anchor().phase().label())),
                (
                    "revision".into(),
                    Value::number(c.anchor().revision().to_string()),
                ),
                ("method".into(), Value::text(c.anchor().method().method)),
                (
                    "uncertainty".into(),
                    Value::text(c.anchor().method().uncertainty_note),
                ),
                (
                    "window_ticks".into(),
                    Value::number(c.anchor().uncertainty().ticks().to_dec_string()),
                ),
                ("citation".into(), Value::text(c.anchor().citation().source)),
            ]),
        )
        .field(
            "intercalation",
            Value::Section(vec![
                (
                    "whole_days_per_year".into(),
                    Value::number(rule.whole_days.numer().to_dec_string()),
                ),
                (
                    "rule".into(),
                    Value::text(format!(
                        "{}/{}",
                        rule.chosen.value.numer().to_dec_string(),
                        rule.chosen.value.denom().to_dec_string()
                    )),
                ),
                (
                    "bound".into(),
                    Value::text(format!(
                        "{} local day per {} local years",
                        rule.bound.days, rule.bound.per_years
                    )),
                ),
                ("walked".into(), Value::list(walked)),
            ]),
        )
        .field(
            "fields",
            Value::Section(vec![
                ("year".into(), Value::number(f.year.to_string())),
                ("day".into(), Value::number(f.day.to_string())),
                (
                    "day_fraction".into(),
                    Value::quantity(&f.day_fraction, 6, Rounding::Trunc),
                ),
                (
                    "anchor_revision".into(),
                    Value::number(f.anchor_revision.to_string()),
                ),
                (
                    "window_ticks".into(),
                    Value::number(f.window.width().ticks().to_dec_string()),
                ),
            ]),
        );

    doc = doc.field(
        "cycles",
        match c.cycles().first() {
            None => Value::text(
                "none — this calendar names no grouping satellite, so it has \
                 years and days only (Rule K.3 as amended, D-A5)",
            ),
            Some(cy) => Value::Section(vec![
                ("satellite".into(), Value::text(cy.satellite)),
                (
                    "cycles_per_year".into(),
                    Value::quantity(&cy.ratio, 9, Rounding::HalfEven),
                ),
                (
                    "convergents".into(),
                    Value::list(cy.convergents.iter().take(8).map(|v| {
                        format!(
                            "{}/{}",
                            v.value.numer().to_dec_string(),
                            v.value.denom().to_dec_string()
                        )
                    })),
                ),
            ]),
        },
    );

    Ok(doc)
}

/// `ucal cal anchor <id>` — the anchor, or the fact that there is none (Rule J).
#[cfg(feature = "body")]
pub fn cmd_cal_anchor(id: &str) -> CmdResult {
    let Some(a) = anchors::for_calendar(id) else {
        return Ok(Doc::new()
            .title(format!("ucal cal anchor {id}"))
            .field("calendar", Value::text(id))
            .field("anchor", Value::text("none"))
            .field(
                "consequence",
                Value::text(
                    "UCAL-E0062: this calendar cannot produce local fields. Phase \
                     is empirical (N15); it is determined and cited, never guessed \
                     and never borrowed from another body (Rule J.3).",
                ),
            )
            .note(
                "The calendar is complete in units, intercalation and cycles, and \
                 incomplete in phase — the state Appendix I.6 describes.",
            ));
    };
    Ok(Doc::new()
        .title(format!("ucal cal anchor {id}"))
        .field("calendar", Value::text(id))
        .field("phase", Value::text(a.phase().label()))
        .field("revision", Value::number(a.revision().to_string()))
        .field("tick", Value::number(a.tick().ticks().to_dec_string()))
        .field(
            "window",
            Value::Section(vec![
                ("lo".into(), Value::number(a.window().lo().ticks().to_dec_string())),
                ("hi".into(), Value::number(a.window().hi().ticks().to_dec_string())),
                (
                    "width_ticks".into(),
                    Value::number(a.uncertainty().ticks().to_dec_string()),
                ),
            ]),
        )
        .field(
            "determination",
            Value::Section(vec![
                ("method".into(), Value::text(a.method().method)),
                ("uncertainty".into(), Value::text(a.method().uncertainty_note)),
                ("citation".into(), Value::text(a.method().citation.source)),
            ]),
        )
        .note(
            "The phase is defined by an event of this body alone (Rule J.1). Its \
             determination may cite an observation on any timescale (Rule Y) — the \
             definition may not.",
        ))
}

// ---------------------------------------------------------------------------
// ucal events / timeline / ruler (§19)
// ---------------------------------------------------------------------------

#[cfg(feature = "events")]
use ucal_events as events;

/// `ucal events list` — the catalogue, chronologically (§19).
#[cfg(feature = "events")]
pub fn cmd_events_list() -> CmdResult {
    let mut rows: Vec<(String, Value)> = Vec::new();
    for e in events::chronological() {
        let mut fields = vec![
            ("label".into(), Value::text(e.label)),
            ("as_published".into(), Value::text(e.as_published)),
            (
                "window_ticks".into(),
                Value::text(format!(
                    "{} .. {}",
                    e.window.lo().ticks().to_dec_string(),
                    e.window.hi().ticks().to_dec_string()
                )),
            ),
            ("citation".into(), Value::text(e.citation.source)),
        ];
        if let Some(w) = e.warning() {
            fields.push(("warning".into(), Value::text(w.to_string())));
        }
        rows.push((e.id.to_string(), Value::Section(fields)));
    }
    Ok(Doc::new()
        .title("ucal events list")
        .field("citation_set", Value::text(events::CITATION_SET))
        .field("events", Value::rows("event", rows))
        .note(
            "Every entry is an interval, because not one of them is known to a \
             tick. The one exact value is a declaration, not a measurement.",
        ))
}

/// `ucal events show <id>` — one milestone, in full.
#[cfg(feature = "events")]
pub fn cmd_events_show(id: &str) -> CmdResult {
    let e = events::by_id(id)?;
    let bridge = UC1::bridge();
    let mid = e.window.midpoint(Rounding::HalfEven)?;

    let mut doc = Doc::new()
        .title(format!("ucal events show {id}"))
        .field("label", Value::text(e.label))
        .field("year", Value::text(YEAR_DEFINITION))
        .field("description", Value::text(e.description))
        .field("as_published", Value::text(e.as_published))
        .field(
            "stated_as",
            Value::text(match e.stated_as {
                events::StatedAs::AfterDatum => "after the datum",
                events::StatedAs::BeforeBridgeEpoch => "before the bridge epoch",
            }),
        )
        .field(
            "window",
            Value::Section(vec![
                ("lo".into(), Value::number(e.window.lo().ticks().to_dec_string())),
                ("hi".into(), Value::number(e.window.hi().ticks().to_dec_string())),
                (
                    "width_ticks".into(),
                    Value::number(e.uncertainty().ticks().to_dec_string()),
                ),
                ("width_years".into(), years_quantity(e.uncertainty().ticks(), 0)),
            ]),
        )
        .field(
            "midpoint",
            Value::Section(vec![
                ("ticks".into(), Value::number(mid.ticks().to_dec_string())),
                ("at_drift".into(), Value::text(render_at(&mid, Tier::DRIFT))),
                (
                    "note".into(),
                    Value::text(
                        "a midpoint is a rendering choice, not a measurement \
                         (Rule U): the window is the value",
                    ),
                ),
            ]),
        )
        .field("citation", Value::text(e.citation.source));

    if let Some(w) = e.warning() {
        doc = doc.field("warning", Value::text(w.to_string())).note(
            "This event lies inside BIG_BANG_CLAIM's half-width. The datum's own \
             physical identification is uncertain by more than the interval being \
             quoted — but the arithmetic above is exact, and the claim is never an \
             operand (Rule Q.3).",
        );
    }
    Ok(doc)
}

/// `ucal timeline` — the catalogue against the tier ladder (§19).
///
/// A one-screen view of the whole of absolute time: each milestone placed at a
/// stated tier, with the tier's own name, so the ladder and the catalogue are
/// read together.
#[cfg(feature = "events")]
pub fn cmd_timeline(tier: Tier) -> CmdResult {
    let mut rows: Vec<(String, Value)> = Vec::new();
    for e in events::chronological() {
        let mid = e.window.midpoint(Rounding::HalfEven)?;
        let at_tier = mid.floor_to(tier);
        let mut fields = vec![
            ("at".into(), Value::text(render_at(&at_tier, tier))),
            (
                format!("{tier}s since the datum"),
                Value::number(mid.ticks().quot_rem(&tier.ticks()).0.to_dec_string()),
            ),
            ("as_published".into(), Value::text(e.as_published)),
        ];
        if e.warning().is_some() {
            fields.push((
                "warning".into(),
                Value::text("UCAL-W0006 — inside the claim half-width"),
            ));
        }
        rows.push((e.label.to_string(), Value::Section(fields)));
    }
    Ok(Doc::new()
        .title(format!("ucal timeline — at tier {tier}"))
        .field("tier", Value::text(tier.to_string()))
        .field("events", Value::rows("event", rows))
        .note(
            "Positions are the windows' midpoints floored to the stated tier. The \
             midpoint is a rendering choice; the window is the value (Rule U).",
        ))
}

/// `ucal ruler --from --to --step` — evenly spaced marks on the tier grid (§19).
pub fn cmd_ruler(from: &str, to: &str, step: Tier) -> CmdResult {
    let (a, _) = parse_instant(from)?;
    let (b, _) = parse_instant(to)?;
    if a > b {
        return Err(TimeError::with_context(
            Code::E0022,
            "the ruler's start must not follow its end",
        ));
    }
    let span = b.since(&a)?;
    let (count, _) = span.divmod(&Delta::from_ticks(step.ticks()))?;
    let n: u64 = count
        .ticks()
        .to_dec_string()
        .parse()
        .unwrap_or(u64::MAX);
    // A ruler with a million marks helps nobody; report the count and cap.
    const MAX_MARKS: u64 = 64;
    let shown = n.min(MAX_MARKS);

    let mut marks: Vec<(String, Value)> = Vec::new();
    for i in 0..=shown {
        let offset = Delta::from_tier(step, i)?;
        let t = a.checked_add(&offset)?;
        marks.push((format!("{i:>4}"), Value::text(render_at(&t, step))));
    }

    let mut doc = Doc::new()
        .title("ucal ruler")
        .field("from", Value::number(a.ticks().to_dec_string()))
        .field("to", Value::number(b.ticks().to_dec_string()))
        .field("step", Value::text(step.to_string()))
        .field("whole_steps", Value::number(n.to_string()))
        .field("marks", Value::rows_of("n", "at", marks));
    if n > MAX_MARKS {
        // No silent caps: §21.3's spirit, and the note the workflow guidance asks
        // for when a bound truncates output.
        doc = doc.note(format!(
            "The span holds {n} whole steps; the first {MAX_MARKS} are shown. \
             Choose a coarser tier to see the whole span."
        ));
    }
    Ok(doc)
}

// ---------------------------------------------------------------------------
// cosmology (§10, §19.4)
// ---------------------------------------------------------------------------

/// Years, as a rendering of a tick count.
#[cfg(feature = "cosmo")]
fn ticks_in_years(t: &Ticks, digits: u32) -> String {
    let year = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .expect("a Julian year fits the domain");
    dec(&tick_ratio(t, &year), digits)
}

/// Parse a redshift: a point, or an interval written `lo..hi`.
///
/// The machinery is interval-valued end to end and until 0.4.0 would only accept
/// a point, so a caller carrying their own uncertainty had to pick a midpoint
/// and lose it — which is the move this project spends thirty chapters objecting
/// to when other people make it.
#[cfg(feature = "cosmo")]
fn parse_redshift(s: &str) -> Result<RatInterval, TimeError> {
    let t = s.trim();
    match t.split_once("..") {
        Some((lo, hi)) => {
            let (lo, hi) = (
                Ratio::from_decimal_str(lo.trim())?,
                Ratio::from_decimal_str(hi.trim())?,
            );
            RatInterval::new(lo, hi)
        }
        None => Ok(RatInterval::exact(Ratio::from_decimal_str(t)?)),
    }
}

/// The definition behind every `*_years` field this program prints.
///
/// "Years" is ambiguous by roughly `2 × 10^-5` — Julian 365.25 d, Gregorian
/// 365.2425 d, tropical ≈365.24219 d — which sounds negligible and is not: at
/// the ages `ucal cosmo age` reports it is about eight years on 371 600, and
/// `arithmetic_years` is printed to one decimal, so the ambiguity lands in
/// digits a reader can see rather than below them.
///
/// `ucal datum` already declares this for `Gyr`. Everything else printed a year
/// and left the reader to guess until 0.4.0.
pub const YEAR_DEFINITION: &str =
    "Julian year = 31 557 600 s exactly (365.25 d of 86 400 s), the same \
     definition ucal datum uses for Gyr. Not Gregorian (365.2425 d) and not \
     tropical: at 371 600 years those differ by about 8. And it is an EARTH \
     unit -- 365.25 of Earth's rotations, a rounded Earth orbit -- used here to \
     describe epochs before Earth existed. It is a bridge (Rule A.3), \
     informative only (Rule A.5), and shown alongside the tick counts that are \
     the actual answer, never instead of them.";

/// The same conversion, certified.
///
/// A count of ticks divided by a Julian year is a rational, and whether its
/// expansion fits the digits asked for depends on the value — so it goes through
/// the certified constructor like every other rendered rational.
fn years_quantity(t: &Ticks, digits: u32) -> Value {
    let year = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .expect("a Julian year fits the domain");
    Value::quantity(&tick_ratio(t, &year), digits, Rounding::HalfEven)
}

/// `ucal cosmo age --z <z>`: the age of the universe at a redshift, as a
/// certified enclosure (§10.3, Rule X).
///
/// The output leads with the enclosure and reports the two widths separately.
/// It never prints a single "best estimate" without the interval around it:
/// F8 is exactly the habit of collapsing those into one number.
#[cfg(feature = "cosmo")]
pub fn cmd_cosmo_age(z: &str, depth: u32, scale: u32) -> CmdResult {
    cmd_cosmo_age_audited(z, depth, scale, false)
}

/// `ucal cosmo age --audit`: the same, with how the enclosure was reached.
///
/// An enclosure's claim rests on every rounding in the chain widening it. Two
/// numbers cannot show that; the audit names each step and the direction it
/// rounds in, so the claim is checkable rather than asserted.
#[cfg(feature = "cosmo")]
pub fn cmd_cosmo_age_audited(z: &str, depth: u32, scale: u32, audit: bool) -> CmdResult {
    let model = ucal_cosmo::LambdaCdm::planck2018();
    let zi = parse_redshift(z)?;
    let z = zi.lo().clone();
    let out = model.t_of_z_interval(&zi, depth, scale)?;

    let mut doc = Doc::new()
        .title("ucal cosmo age")
        .field(
            "z",
            if zi.lo() == zi.hi() {
                Value::quantity(zi.lo(), 4, Rounding::HalfEven)
            } else {
                Value::text(format!(
                    "{} .. {}",
                    dec(zi.lo(), 4),
                    dec(zi.hi(), 4)
                ))
            },
        )
        .field("model", Value::text(out.model.0))
        .field("year", Value::text(YEAR_DEFINITION))
        .field(
            "enclosure",
            Value::Section(vec![
                (
                    "lo_ticks".into(),
                    Value::number(out.value.lo().ticks().to_dec_string()),
                ),
                (
                    "hi_ticks".into(),
                    Value::number(out.value.hi().ticks().to_dec_string()),
                ),
                ("lo_years".into(), years_quantity(out.value.lo().ticks(), 0)),
                ("hi_years".into(), years_quantity(out.value.hi().ticks(), 0)),
                (
                    "at_drift".into(),
                    Value::text(render_at(out.value.lo(), Tier::DRIFT)),
                ),
            ]),
        )
        .field(
            "widths",
            Value::Section(vec![
                // Ticks first, because ticks are the answer. §4.3 and Rule A.5:
                // the bridge unit is informative and shown *alongside*, never
                // instead. Until 0.4.0 these three widths were reported in
                // Julian years and nothing else -- an Earth orbit used as the
                // sole measure of an epoch 13.4 Gyr before Earth existed, in the
                // one program written to object to exactly that.
                (
                    "arithmetic_ticks".into(),
                    Value::number(out.arithmetic_width.ticks().to_dec_string()),
                ),
                (
                    "arithmetic_drifts".into(),
                    Value::quantity(
                        &tick_ratio(out.arithmetic_width.ticks(), &Tier::DRIFT.ticks()),
                        6,
                        Rounding::HalfEven,
                    ),
                ),
                ("arithmetic_years".into(), years_quantity(out.arithmetic_width.ticks(), 1)),
                (
                    "parameter_ticks".into(),
                    Value::number(out.parameter_width.ticks().to_dec_string()),
                ),
                (
                    "parameter_drifts".into(),
                    Value::quantity(
                        &tick_ratio(out.parameter_width.ticks(), &Tier::DRIFT.ticks()),
                        6,
                        Rounding::HalfEven,
                    ),
                ),
                ("parameter_years".into(), years_quantity(out.parameter_width.ticks(), 1)),
                (
                    "note".into(),
                    Value::text(
                        "Rule X: quadrature error and parameter uncertainty are \
                         reported separately and never merged (F8). The second is \
                         what the measurement does not know; the first is what this \
                         program does not know. Each is given in ticks, in drifts, \
                         and in Julian years — the last being a foreign unit shown \
                         alongside and never instead (§4.3, Rule A.5).",
                    ),
                ),
            ]),
        )
        .field(
            "quadrature",
            Value::Section(vec![
                ("depth".into(), Value::number(depth.to_string())),
                ("panels".into(), Value::number((1u64 << depth).to_string())),
                ("sqrt_scale_digits".into(), Value::number(scale.to_string())),
            ]),
        )
        .field("parameters", Value::text(model.describe()))
        .field("citation", Value::text(out.citation.source));

    // A third width, and only when the caller supplied an interval. Always
    // present and always zero would be noise; absent says the input was a point.
    // Separate from the other two for the reason Rule X separates those: a
    // caller's uncertainty is not the measurement's and is not this program's.
    if !zi.lo().eq(zi.hi()) {
        doc = doc.field(
            "input_width",
            Value::Section(vec![
                ("ticks".into(), Value::number(out.input_width.ticks().to_dec_string())),
                (
                    "drifts".into(),
                    Value::quantity(
                        &tick_ratio(out.input_width.ticks(), &Tier::DRIFT.ticks()),
                        6,
                        Rounding::HalfEven,
                    ),
                ),
                ("years".into(), years_quantity(out.input_width.ticks(), 1)),
                (
                    "note".into(),
                    Value::text(
                        "what the requested z interval contributed, over and above \
                         what a point at its lower end would already have cost. \
                         Reported apart from the other two widths so that no one \
                         of the three can be mistaken for another.",
                    ),
                ),
            ]),
        );
    }

    if audit {
        doc = doc.field(
            "audit",
            Value::Section(
                model
                    .audit(&z, depth, scale)?
                    .into_iter()
                    .map(|(step, detail)| (step, Value::text(detail)))
                    .collect(),
            ),
        );
    }

    for w in &out.warnings {
        doc = doc.field("warning", Value::text(w.to_string()));
    }
    Ok(doc.note(
        "The enclosure is certified: the true age under this model provably lies \
         inside it. It is not a measurement of the universe — it is what this \
         parameter set implies, with the parameter set's own uncertainty carried \
         through (Rule X).",
    ))
}

/// `ucal cosmo z --at <instant>`: the redshift at an absolute time (§10.4).
///
/// The tolerance is stated in years rather than ticks because a tick is
/// unreachable: sixty-four bisection steps resolve `z` to about `5e-16`, which
/// is still tens of seconds of cosmic time. Asking for a tick returns
/// `UCAL-E0071` — the honest answer, and the one §10.4's error code exists for.
#[cfg(feature = "cosmo")]
pub fn cmd_cosmo_z(instant: &str, tolerance_years: u64, depth: u32, scale: u32) -> CmdResult {
    let model = ucal_cosmo::LambdaCdm::planck2018();
    let (t, precision) = parse_instant(instant)?;
    let window = t.window_at(precision)?;
    let year = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .ok_or(TimeError::with_context(Code::E0060, "year overflows"))?;
    let tolerance = Delta::from_ticks(
        year.try_mul(&<Ticks as TickInt>::from_u64(tolerance_years.max(1)))
            .ok_or(TimeError::with_context(Code::E0060, "tolerance overflows"))?,
    );
    let out = model.z_of_t(&window, &tolerance, depth, scale)?;

    Ok(Doc::new()
        .title("ucal cosmo z")
        .field("instant_ticks", Value::number(t.ticks().to_dec_string()))
        .field("years_after_datum", Value::text(ticks_in_years(t.ticks(), 0)))
        .field("year", Value::text(YEAR_DEFINITION))
        .field("model", Value::text(out.model.0))
        .field("tolerance_years", Value::number(tolerance_years.to_string()))
        .field(
            "z",
            Value::Section(vec![
                ("lo".into(), Value::number(dec(out.value.lo(), 6))),
                ("hi".into(), Value::number(dec(out.value.hi(), 6))),
            ]),
        )
        .field("citation", Value::text(out.citation.source))
        .note(
            "The bracket is an enclosure of every redshift whose age-interval \
             meets the given instant. It is wide because the parameters are \
             intervals: a range of redshifts is consistent with any one age, and \
             a narrow answer here would be a claim the model cannot support.",
        ))
}

/// `ucal cosmo model`: the parameter set, its provenance, and the two
/// experiments' measured outcomes (§21).
#[cfg(feature = "cosmo")]
pub fn cmd_cosmo_model() -> CmdResult {
    let model = ucal_cosmo::LambdaCdm::planck2018();
    let params: Vec<(String, Value)> = model
        .as_measured
        .iter()
        .map(|p| (p.name.to_string(), Value::text(p.verbatim)))
        .collect();
    let turn = model.monotonicity_turns_at()?;

    Ok(Doc::new()
        .title("ucal cosmo model")
        .field("model", Value::text(model.model.0))
        .field("year", Value::text(YEAR_DEFINITION))
        .field("as_published", Value::Section(params))
        .field("citation", Value::text(model.citation.source))
        .field(
            "hubble_time",
            Value::Section(vec![
                (
                    "ticks_lo".into(),
                    Value::number(model.hubble_time.lo().numer().to_dec_string()),
                ),
                (
                    "gyr".into(),
                    Value::text(dec(
                        &model
                            .hubble_time
                            .lo()
                            .div(&Ratio::from_int(
                                UC1::bridge()
                                    .ticks
                                    .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
                                    .and_then(|y| {
                                        y.try_mul(&<Ticks as TickInt>::from_u64(1_000_000_000))
                                    })
                                    .expect("a gigayear fits the domain"),
                            ))?,
                        3,
                    )),
                ),
                (
                    "note".into(),
                    Value::text(
                        "1/H0 involves pi through the parsec, so it is an interval \
                         bounded by a rational enclosure of pi rather than a value \
                         (Rule E).",
                    ),
                ),
            ]),
        )
        .field(
            "monotonicity",
            Value::Section(vec![
                ("turns_at_u".into(), Value::number(dec(turn.lo(), 6))),
                (
                    "note".into(),
                    Value::text(
                        "Appendix H.4 requires monotonicity to be asserted, not \
                         assumed. It fails here, so every panel is bounded by the \
                         interval extension instead.",
                    ),
                ),
            ]),
        )
        .field("ge1", Value::text(
            "depth-24 quadrature is hours, not seconds: the GE-1 kill criterion \
             fires. The default depth is 12 and --depth is the high-precision mode.",
        ))
        .field("ge2", Value::text(ucal_cosmo::GE2_ACHIEVABLE_WIDTH)))
}
