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
// Needs `ucal-body`: the file it loads declares a `Body`. Gated for the same
// reason every other body-dependent item here is — `ucal` builds without that
// feature, and the features workflow catches it when it does not.
#[cfg(feature = "body")]
pub mod anchor_file;
#[cfg(feature = "tui")]
pub mod wallclock;
#[cfg(feature = "body")]
pub mod body_file;
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
        "expected a decimal tick count like 8070205189123984864657505252035637180530466139316558837890625, a UC1 text form like `UC1 0031\u{00b7}0687\u{00b7}...`, or a 52-character UCID. `ucal now` prints one of each; `ucal tour` shows what to do with them",
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

/// A ratio of two tick counts.
///
/// ucal-lint-allow-begin(no-panic-in-cli): every denominator this is called with
/// is a profile constant or a tier — `BEAT`, `SECOND`, a Julian year, `5^e` —
/// and none of them is zero. The alternative to asserting that is returning
/// `Result` through twenty-three call sites, most of them inside a
/// `Value::Section` literal, to handle a case no input can produce.
///
/// The fallback that would avoid both is worse than either: substituting any
/// value for a zero denominator prints a *wrong number* where this prints
/// nothing at all, and this project would rather stop than answer incorrectly.
/// `main.rs`'s panic hook turns the stop into a diagnostic and exit 70 rather
/// than a backtrace.
fn tick_ratio(a: &Ticks, b: &Ticks) -> Ratio {
    Ratio::new(a.clone(), b.clone()).expect("non-zero denominator")
}
// ucal-lint-allow-end(no-panic-in-cli)

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
        Fmt::default()
            .with_form(Form::Named)
            .with_precision(Precision::Tier(tier))
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

    // 1b. FRAME_BRIDGE_CLAIM (D-A25). Rule F required a profile to declare its
    // frame and said nothing about the distance to the scale it converts
    // through, which for UC-1 is TT — a clock on Earth's geoid, and Earth is not
    // comoving with the CMB. Printed next to the frame because that is where a
    // reader forms the belief this bounds.
    let fb = UC1::frame_bridge_claim();
    let fb_citation = UC1::frame_bridge_claim_citation();
    let fb_half = fb.hi().magnitude().ticks();
    doc = doc.field(
        "frame_bridge_claim",
        Value::Section(vec![
            ("bridge_scale".into(), Value::text("TT (§8.1)")),
            (
                "half_width_ticks".into(),
                Value::number(fb_half.to_dec_string()),
            ),
            (
                "bound".into(),
                Value::text(
                    "5 x 10^-6 of elapsed time: the rate difference between this \
                     profile's declared frame and its bridge scale",
                ),
            ),
            ("citation".into(), Value::text(fb_citation.source)),
            (
                "cancels_in".into(),
                Value::text(
                    "any difference of two instants carried through the same bridge, \
                     which is every interval this program computes. It bears only on \
                     reading an absolute tick count as elapsed cosmological time",
                ),
            ),
            (
                "status".into(),
                Value::text(
                    "metadata only; no arithmetic operation may consume it, for the \
                     same reason big_bang_claim may not (Rule Q.3)",
                ),
            ),
        ]),
    );

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
            (
                "seconds".into(),
                Value::bridge(Value::quantity(&implied_s, 6, Rounding::HalfEven)),
            ),
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

/// Why each field of `explain` is there, by the rule or section that requires it.
///
/// T3. `explain` is the densest output in the program and every field of it is
/// defensible; none of them says *why* it is present. A reader meeting
/// `precision`, `digit5` and `beats_since_datum` at once has no way to tell
/// which are consequences of the model and which are conveniences — and the
/// answer is that almost none are conveniences.
///
/// Opt-in, so the ordinary output is unchanged, and additive to `ucal-json/1`.
const WHY_EXPLAIN: &[(&str, &str)] = &[
    ("ticks", "Rule Z: the value itself, an unsigned integer count from the datum. Everything else on this page is a rendering of this number."),
    ("precision", "Rule T: a form printed to a coarser tier denotes an interval, not a point with trailing zeros. This says which one you are holding."),
    ("human", "§6: the text form anchored at T0, for reading aloud."),
    ("digit5", "§6 and Rule S: fixed-width, so lexicographic order equals chronological order. That is why it opens with 27 groups of zeros."),
    ("ucid", "§6.5: a sortable identifier for an instant, or a statement that this one is outside the 2^256 UCID range."),
    ("tiers", "§4.2: the instant decomposed onto the universal ladder. It reassembles to `ticks` exactly, because every tier is a power of five."),
    ("window", "Rule T again: present only when the input was stated to a tier, because then it named an interval, and this is that interval."),
    ("beats_since_datum", "§0.5: the beat is the universe second, 5^60 ticks. This count carries no Earth content, which is the point of having it."),
    ("si_bridge", "Rule A.5 and D-A16: an SI second is an Earth unit, so the conversion is shown on request with --bridge and never unasked."),
    ("claim", "Rule Q.3: BIG_BANG_CLAIM is metadata. Printed with --claim, and it can never enter a computation; three compile-fail tests hold that."),
    ("warning", "§10.6: set when the instant lies inside the claim's own half-width, where the datum's identification is larger than the thing being discussed."),
];

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
        let d = t
            .ticks()
            .try_sub(&UC1::origin_offset())
            .ok_or(TimeError::new(Code::E0020))?;
        format!("+{}", dec(&tick_ratio(&d, &bridge.ticks), 6))
    } else {
        let d = UC1::origin_offset()
            .try_sub(t.ticks())
            .ok_or(TimeError::new(Code::E0020))?;
        format!("-{}", dec(&tick_ratio(&d, &bridge.ticks), 6))
    };
    // §4.3 said `ucal explain` "always prints the SI equivalent alongside".
    // Amended in 0.4.0 (D-A16): the conversion is available on request and is
    // not performed unasked, because an SI second is an Earth unit and this is
    // not an Earth command. `--bridge` prints it.
    doc = doc.field(
        "si_bridge",
        Value::bridge(Value::Section(vec![
            ("unit".into(), Value::text(bridge.name)),
            ("epoch".into(), Value::text(bridge.epoch_label)),
            ("seconds_from_epoch".into(), Value::text(since_epoch)),
        ])),
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

/// `ucal explain <instant> --why` — the same document, annotated.
///
/// Built by asking the document what fields it has and looking each up, so a
/// field added to `explain` without a reason appears as a gap here rather than
/// going quietly undocumented. `tour.rs` checks that there are no gaps.
pub fn cmd_explain_why(input: &str, show_claim: bool) -> CmdResult {
    let doc = cmd_explain(input, show_claim)?;
    let mut rows: Vec<(String, Value)> = Vec::new();
    for (name, _) in doc.fields() {
        let why = WHY_EXPLAIN
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| *v)
            .unwrap_or("— (no reason recorded for this field; please report it, because every field of this command is supposed to have one)");
        rows.push((name.clone(), Value::text(why)));
    }
    Ok(doc.field("why", Value::rows("field", rows)).note(
        "Each line names the rule or section that requires the field above it. Almost none is a convenience: this command's output is dense because the model is, not because more seemed better.",
    ))
}

// ---------------------------------------------------------------------------
// ucal verify — the self-check, inside the shipped binary
// ---------------------------------------------------------------------------

/// A Julian year in SI seconds, exact by definition (§2.2's `unit_defs`).
const JULIAN_YEAR_S: u64 = 31_557_600;

/// One check: what was claimed, what this build computed, and whether they meet.
fn checked(name: &str, note: &str, derived: &Ticks, declared: &Ticks) -> (String, Value) {
    let agrees = derived == declared;
    (
        name.to_string(),
        Value::Section(vec![
            ("agrees".into(), Value::Bool(agrees)),
            ("value".into(), Value::number(declared.to_dec_string())),
            (
                "derived".into(),
                Value::number(derived.to_dec_string()),
            ),
            ("from".into(), Value::text(note)),
        ]),
    )
}

/// Re-derive the declared constants and check this build reproduces them.
///
/// # What this is for
///
/// Re-deriving the constants otherwise needs the repository and `xtask`, and
/// `xtask` is `publish = false`. Someone who typed `cargo install ucal` could
/// not check that the binary they were holding agreed with the published
/// values, and the first question an external implementer asks is *what should
/// I get* — to which the answer was "clone a repository first".
///
/// # What it does not establish
///
/// **This is a self-check, not an independent verification, and the output says
/// so.** Every number here is computed by one implementation from one
/// specification. Agreement means this build's arithmetic works and that it
/// reproduces `fixtures/vectors.json`; it cannot mean the specification is
/// right, because nothing here is independent of it. That is C1, and it needs
/// somebody else's code.
///
/// What it *can* catch is real and worth having: a miscompiled backend, a
/// feature combination that silently changes a value, a corrupted install, and
/// an implementer's transcription error — since the values are printed in full
/// for comparison rather than only compared internally.
pub fn cmd_verify() -> CmdResult {
    let one = <Ticks as TickInt>::one();
    let five = <Ticks as TickInt>::from_u64(5);
    let ten = <Ticks as TickInt>::from_u64(10);
    let mul = |a: &Ticks, b: &Ticks| a.try_mul(b).ok_or(TimeError::new(Code::E0002));

    // BEAT = 5^60, by repeated multiplication rather than by asking the profile.
    let mut beat = one.clone();
    for _ in 0..60 {
        beat = mul(&beat, &five)?;
    }

    // SECOND = 18 548 584 399 861 x 10^30 (D-3). The mantissa is the declared
    // one; what is being checked is the arithmetic and the build, not the
    // mantissa, which no amount of recomputation here could confirm.
    let mut second = <Ticks as TickInt>::from_u64(18_548_584_399_861);
    for _ in 0..30 {
        second = mul(&second, &ten)?;
    }

    // ORIGIN_OFFSET, by re-executing §2.2's chain from the *provenance record's
    // own input* rather than from a copy of the answer.
    let prov = UC1::datum_provenance()?;
    // "13.787" Gyr -> 13 787 000 000 Julian years, exactly, without a float.
    let gyr = prov.input.verbatim;
    let (whole, frac) = gyr.split_once('.').unwrap_or((gyr, ""));
    let scaled = format!("{whole}{frac}");
    let years = <Ticks as TickInt>::from_dec_str(&scaled)
        .and_then(|v| {
            // 10^(9 - |frac|), so 13.787 Gyr becomes 13 787 000 000 years.
            let mut p = <Ticks as TickInt>::one();
            for _ in 0..(9usize.checked_sub(frac.len())?) {
                p = p.try_mul(&ten)?;
            }
            v.try_mul(&p)
        })
        .ok_or(TimeError::with_context(
            Code::E0002,
            "the provenance input is not a decimal in Gyr",
        ))?;
    let age_s = mul(&years, &<Ticks as TickInt>::from_u64(JULIAN_YEAR_S))?;
    let age_ticks = mul(&age_s, &second)?;
    // beats = round_half_even(AGE_ticks / BEAT), then ORIGIN_OFFSET = beats x BEAT.
    let (q, r) = age_ticks.quot_rem(&beat);
    let twice = mul(&r, &<Ticks as TickInt>::from_u64(2))?;
    let beats = match twice.cmp(&beat) {
        core::cmp::Ordering::Greater => q.try_add(&one).ok_or(TimeError::new(Code::E0002))?,
        core::cmp::Ordering::Less => q,
        // Exactly a half: to even.
        core::cmp::Ordering::Equal if q.is_odd() => {
            q.try_add(&one).ok_or(TimeError::new(Code::E0002))?
        }
        core::cmp::Ordering::Equal => q,
    };
    let origin = mul(&beats, &beat)?;

    let constants = vec![
        checked(
            "BEAT",
            "5^60, by repeated multiplication",
            &beat,
            &UC1::beat(),
        ),
        checked(
            "SECOND",
            "18548584399861 x 10^30 (D-3)",
            &second,
            &UC1::bridge().ticks,
        ),
        checked(
            "ORIGIN_OFFSET",
            "round_half_even(AGE_ticks / BEAT) x BEAT, from the provenance input",
            &origin,
            &UC1::origin_offset(),
        ),
    ];
    let mut disagreements: Vec<String> = constants
        .iter()
        .filter(|(_, v)| {
            v.as_rows()
                .and_then(|r| r.iter().find(|(k, _)| k == "agrees"))
                .map(|(_, b)| !matches!(b, Value::Bool(true)))
                .unwrap_or(true)
        })
        .map(|(n, _)| n.clone())
        .collect();

    // Structural invariants (§2.4). These are checks on relationships rather
    // than on values, so a transcription error that happened to be internally
    // consistent would still fail them.
    let whole_beats = UC1::origin_offset().quot_rem(&UC1::beat()).1.is_zero_ticks();
    // The bridge claims 5^divisibility divides it exactly, and no more (D-3).
    let div = UC1::bridge().divisibility;
    let mut s = UC1::bridge().ticks;
    let mut divides = true;
    for _ in 0..div {
        let (q, r) = s.quot_rem(&five);
        if !r.is_zero_ticks() {
            divides = false;
            break;
        }
        s = q;
    }
    let exact_power = divides && !s.quot_rem(&five).1.is_zero_ticks();

    // Every rung is 5^(60 + 5k), recomputed rather than read from the table.
    let mut grid_ok = true;
    for tier in Tier::all_descending() {
        let mut p = <Ticks as TickInt>::one();
        for _ in 0..tier.exponent() {
            match p.try_mul(&five) {
                Some(v) => p = v,
                None => {
                    grid_ok = false;
                    break;
                }
            }
        }
        if p != tier.ticks() {
            grid_ok = false;
            break;
        }
    }

    let invariants = vec![
        (
            "origin_offset_is_whole_beats".to_string(),
            Value::Bool(whole_beats),
        ),
        (
            "bridge_divisibility_is_exact".to_string(),
            Value::Bool(exact_power),
        ),
        ("tier_grid_is_five_powers".to_string(), Value::Bool(grid_ok)),
    ];
    for (n, v) in &invariants {
        if !matches!(v, Value::Bool(true)) {
            disagreements.push(n.clone());
        }
    }

    let ok = disagreements.is_empty();
    let doc = Doc::new()
        .title("ucal verify")
        .field("profile", Value::text(UC1::TAG))
        .field(
            "backend",
            Value::text(if cfg!(feature = "bigint") {
                "bigint"
            } else {
                "u512"
            }),
        )
        .field("agrees", Value::Bool(ok))
        .field("constants", Value::Section(constants))
        .field("invariants", Value::Section(invariants))
        .field(
            "compare_with",
            Value::text(
                "fixtures/vectors.json in the source repository, whose digest is \
                 signed; spec/CONFORMANCE.md describes the file and the key",
            ),
        )
        .field(
            "what_this_does_not_establish",
            Value::text(
                "This is a self-check. Every number above was computed by one \
                 implementation from one specification, so agreement means this \
                 build's arithmetic works and reproduces the published values — \
                 not that the specification is right. An independent \
                 implementation reproducing these constants is the check that \
                 would mean that, and it has never been done. See \
                 Documentation/CONTACT.md.",
            ),
        );
    if !ok {
        // Exit non-zero, not merely say so in a note.
        //
        // Until 1.0.1 this returned `Ok`, so a build that did not reproduce its
        // own constants printed `agrees false` and exited **0** — a verification
        // command whose failure a script could not see, which is the exact class
        // of defect the 0.9.0 stability pass existed to remove and which this
        // command had all along. Found by writing the release workflow, whose
        // whole purpose is to refuse to package a binary that fails this.
        //
        // `E0015` and not a code borrowed for its exit value. The first attempt
        // used `E0025` — "BIG_BANG_CLAIM used as a computational operand" —
        // because it carried the right *number*, which is precisely the defect
        // D-A17 was written to fix one cycle earlier: a code whose canonical
        // meaning describes something that did not happen.
        return Err(TimeError::with_context(
            Code::E0015,
            "this build does not reproduce the declared constants; that is a \
             defect in the build or the install, not a difference of opinion — \
             every quantity involved is an exact integer. Please report it.",
        ));
    }
    Ok(doc)
}

// ---------------------------------------------------------------------------
// ucal between — a duration, on the ladder
// ---------------------------------------------------------------------------

/// Where a duration *sits* on the universal ladder: the rung, and how far above it.
///
/// Not to be confused with `between`'s `on_the_ladder`, which decomposes a
/// duration into a count of every tier. This one answers a different question —
/// which single rung is this period's size — and so carries different columns
/// under a different key, because one name with two shapes in the
/// `ucal-json/1` surface is a consumer's problem, not a saving.
///
/// Y1, and the one thing [`W4-two-ladders.md`] recommended keeping. Its step 1
/// placed every unit of every shipped body and found Earth's day and Mars's sol
/// on the *same rung* — 591 arcs and 607 — on a ladder whose steps are a factor
/// of 3125. Two separate planets, two separate measurements, and a grid built
/// from powers of five with no knowledge of either.
///
/// That is worth a row and is not worth a view: every unit of every body lands
/// on `T1` or `T2`, two adjacent rungs out of forty-five, so a two-column
/// display would be forty-three empty lines and two full ones.
///
/// [`W4-two-ladders.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/W4-two-ladders.md
#[cfg(feature = "body")]
fn ladder_placement(length: &Ratio) -> Option<(Tier, Ratio)> {
    let tier = Tier::all_descending()
        .find(|t| Ratio::from_int(t.ticks()).cmp_exact(length) != core::cmp::Ordering::Greater)?;
    let above = length.div(&Ratio::from_int(tier.ticks())).ok()?;
    Some((tier, above))
}

/// A `(rung, above)` pair as a row.
#[cfg(feature = "body")]
fn ladder_row(name: &str, length: &Ratio) -> Option<(String, Value)> {
    let (tier, above) = ladder_placement(length)?;
    let label = match ucal_core::tier::name_of(tier) {
        Some(n) => format!("{tier} {}", n.key()),
        None => tier.to_string(),
    };
    Some((
        name.to_string(),
        Value::Section(vec![
            ("rung".into(), Value::text(label)),
            (
                "above_rung".into(),
                Value::quantity(&above, 1, Rounding::HalfEven),
            ),
        ]),
    ))
}

/// The named tiers, coarsest first.
///
/// The grid has forty-five rungs and ten of them have names (Rule N). A
/// decomposition across all forty-five would be arithmetically identical and
/// unreadable; across the named ones it is the sentence a person would say.
///
/// The floor is `TICK`, which is `5^0` ticks — one — so the decomposition is
/// exact and total even though the named tiers are not contiguous: whatever
/// falls below one spark lands in the tick row, whole.
const NAMED_DESCENDING: &[Tier] = &[
    Tier::DEEP,
    Tier::DRIFT,
    Tier::SPAN,
    Tier::SWEEP,
    Tier::ARC,
    Tier::BEAT,
    Tier::FLICKER,
    Tier::GLINT,
    Tier::SPARK,
    Tier::TICK,
];

/// A tier's label: `T4 drift`, or `T7` where the grid has no name.
fn tier_label(tier: Tier) -> String {
    match ucal_core::tier::name_of(tier) {
        Some(n) => format!("{tier} {}", n.key()),
        None => tier.to_string(),
    }
}

/// How far apart two instants are, stated on the tier ladder.
///
/// The project's claim is that a duration belongs on the grid rather than in a
/// foreign unit, and until 0.8.0 no command put one there: `explain` describes a
/// point and `ruler` marks a span without measuring it. The arithmetic existed
/// and was unreachable from the binary.
///
/// The sign is reported rather than absorbed. [`Instant::between`] returns a
/// [`ucal_core::Signed`] because the domain is unsigned (Rule Z) and the
/// difference need not be; quietly taking the magnitude would make
/// `between a b` and `between b a` print the same thing, which is the kind of
/// convenience Rule Q refuses.
pub fn cmd_between(from: &str, to: &str, at: Option<Tier>) -> CmdResult {
    let (a, _) = parse_instant(from)?;
    let (b, _) = parse_instant(to)?;
    let signed = b.between(&a);
    let mag = signed.magnitude();

    // Coarsest named tier that fits, and the whole/remainder walk below it.
    let mut rows: Vec<(String, Value)> = Vec::new();
    let mut rest = mag.ticks().clone();
    let mut started = false;
    for tier in NAMED_DESCENDING {
        let (whole, rem) = rest.quot_rem(&tier.ticks());
        // Leading zeros are noise: a span of three arcs should not open with
        // six rows of `0`. A zero *between* two non-zero tiers is information
        // and is kept.
        if !started && whole.is_zero_ticks() {
            rest = rem;
            continue;
        }
        started = true;
        rows.push((tier_label(*tier), Value::number(whole.to_dec_string())));
        rest = rem;
    }
    if rows.is_empty() {
        rows.push((tier_label(Tier::TICK), Value::number("0".to_string())));
    }

    let mut doc = Doc::new()
        .title("ucal between")
        .field("from", Value::number(a.ticks().to_dec_string()))
        .field("to", Value::number(b.ticks().to_dec_string()))
        .field(
            "direction",
            Value::text(match signed.sign() {
                _ if signed.is_zero() => "the same instant",
                ucal_core::Sign::Positive => "`to` is later than `from`",
                ucal_core::Sign::Negative => "`to` is earlier than `from`",
            }),
        )
        .field("ticks", Value::number(mag.ticks().to_dec_string()))
        .field(
            "natural_tier",
            Value::text(match mag.tier_of() {
                Some(t) => tier_label(t),
                None => "— (zero: no tier contains it)".to_string(),
            }),
        )
        .field("on_the_ladder", Value::rows_of("tier", "whole", rows));

    // `--at <tier>`: the divmod a reader actually asked for, rather than the
    // decomposition's opinion about which tiers are interesting.
    if let Some(tier) = at {
        let (whole, rem) = mag.in_tier(tier);
        doc = doc.field(
            "at",
            Value::Section(vec![
                ("tier".into(), Value::text(tier_label(tier))),
                ("whole".into(), Value::number(whole.to_dec_string())),
                (
                    "remainder_ticks".into(),
                    Value::number(rem.to_dec_string()),
                ),
            ]),
        );
    }

    // SI on request only. A second is an Earth unit and a duration between two
    // absolute instants is not an Earth quantity (Rule A.5, D-A16).
    let bridge = UC1::bridge();
    doc = doc.field(
        "si_bridge",
        Value::bridge(Value::Section(vec![
            ("unit".into(), Value::text(bridge.name)),
            (
                "seconds".into(),
                Value::quantity(
                    &tick_ratio(mag.ticks(), &bridge.ticks),
                    6,
                    Rounding::HalfEven,
                ),
            ),
        ])),
    );

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

/// The system clock as a `UC1` instant, through the bundled leap table.
///
/// Extracted so `ucal wallclock` reads *now* by the same route `ucal now` does.
/// A clock with its own path to the system time would be a second
/// implementation, and the two would eventually disagree by a leap second — the
/// one quantity §8.4 says cannot be computed and must be looked up.
#[cfg(feature = "civil")]
pub fn now_instant() -> Result<Instant<UC1>, TimeError> {
    use std::time::{SystemTime, UNIX_EPOCH};
    let d = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| TimeError::with_context(Code::E0020, "system clock precedes the Unix epoch"))?;
    let unix_secs = d.as_secs() as i64;
    let nanos = d.subsec_nanos();

    let days = 719_528 + unix_secs.div_euclid(86_400);
    let sod = unix_secs.rem_euclid(86_400);
    let (y, mo, dd) = ucal_civil::calendar::civil_from_days(days, CivilCalendar::Gregorian);
    si::from_civil(
        y,
        mo,
        dd,
        (sod / 3_600) as u8,
        ((sod % 3_600) / 60) as u8,
        (sod % 60) as u8,
        SubSecond::new(nanos as u128, 9)?,
        Scale::Utc,
        CivilCalendar::Gregorian,
    )
}

/// `ucal now` — the system clock, converted through the bundled leap table.
///
/// §8.4: the clock is read as UTC and converted offline. Unix time does not count
/// leap seconds, so its value is a *label-linear* count and is converted as a UTC
/// label rather than as an elapsed duration — which is exactly the distinction
/// Rule L exists to keep visible.
#[cfg(all(feature = "civil", feature = "std"))]
#[cfg(feature = "civil")]
pub fn cmd_now(precision: Tier, form: Form) -> CmdResult {
    let t = now_instant()?;

    let fmt = Fmt::default()
        .with_form(form)
        .with_precision(if precision.is_tick() {
            Precision::Tick
        } else {
            Precision::Tier(precision)
        })
        .with_pad(matches!(form, Form::Digit5));
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
                    Value::bridge(Value::quantity(&in_bridge, 6, Rounding::HalfEven)),
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
    //
    // Read from the registry rather than listed here. This was a hard-coded
    // `["titan-d"]` until 0.8.0, which would have silently omitted every body
    // added since — and four were added in the same cycle.
    for (id, body, _) in bodycal::registered() {
        if anchors::for_calendar(id).is_some() {
            continue;
        }
        let mut fields = vec![
            ("kind".into(), Value::text("derived — Rule K")),
            ("body".into(), Value::text(body.id().to_string())),
        ];
        // The status says these calendars are complete in intercalation. Show
        // it, rather than asking the reader to take the sentence on trust — the
        // whole claim of Rule K is that the rule falls out of the periods, and
        // an anchor is not needed to see that it does.
        if let Ok(rule) = ucal_body::derive_leap_rule(
            body.solar_day().value_at_epoch(),
            body.orbital_period().value_at_epoch(),
            ucal_body::DriftBound::DEFAULT,
            32,
        ) {
            fields.push((
                "leap_rule".into(),
                Value::text(format!(
                    "{}/{} (convergent {})",
                    rule.chosen.value.numer().to_dec_string(),
                    rule.chosen.value.denom().to_dec_string(),
                    rule.depth
                )),
            ));
        }
        fields.push((
            "status".into(),
            Value::text(
                "no anchor: complete in units, intercalation and cycles, \
                 incomplete in phase. Asking for local fields is \
                 UCAL-E0062 (Rule J.3).",
            ),
        ));
        rows.push((id.to_string(), Value::Section(fields)));
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
    cmd_show_inner(input, calendars, None)
}

/// F1 — the multi-calendar view, with one calendar coming from §15.1 files.
///
/// This is where the loaders stopped being useful. `cal derive --anchor --at`
/// could already produce a date from a pair of files; what it could not do is
/// put that calendar **beside** the shipped ones, which is the comparison the
/// whole exercise is for. A body that does not ship is now a row in the same
/// table as Earth and Mars.
///
/// `cal show` deliberately did *not* grow the same flags. `cal derive` already
/// prints that view from the same code, and two spellings of one question is
/// how they come to disagree.
#[cfg(all(feature = "body", feature = "civil"))]
pub fn cmd_show_with_file(
    input: &str,
    calendars: &[String],
    body_path: &str,
    anchor_path: &str,
) -> CmdResult {
    let extra = calendar_from_files(body_path, anchor_path)?;
    cmd_show_inner(input, calendars, Some(extra))
}

#[cfg(all(feature = "body", feature = "civil"))]
fn cmd_show_inner(
    input: &str,
    calendars: &[String],
    extra: Option<ucal_body::calendar::BodyCalendar>,
) -> CmdResult {
    let (t, _) = parse_instant(input)?;
    let mut rows: Vec<(String, Value)> = Vec::new();
    let mut produced = 0usize;

    for id in calendars {
        let entry = match id.as_str() {
            "earth-civil" | "earth-julian" => {
                let c: &dyn LegacyCalendar = if id == "earth-civil" {
                    &Gregorian
                } else {
                    &Julian
                };
                let r = c.render(&t, Scale::Tt, 0, Rounding::Trunc)?;
                produced += 1;
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
                    produced += 1;
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

    // The file-defined calendar, rendered by the same arm the shipped ones use.
    if let Some(c) = extra {
        let id = ucal_core::qualified::CalendarIdentity::id(&c).to_string();
        let r = c.render(&t)?;
        let f = c.fields(&t)?;
        produced += 1;
        rows.push((
            id,
            Value::Section(vec![
                ("rendered".into(), Value::text(r.to_string())),
                ("kind".into(), Value::text("derived (Rule K), from files")),
                (
                    "anchor_revision".into(),
                    Value::number(f.anchor_revision.to_string()),
                ),
                (
                    "window_ticks".into(),
                    Value::number(f.window.width().ticks().to_dec_string()),
                ),
                (
                    "source".into(),
                    Value::text("§15.1 body and anchor files, loaded at run time"),
                ),
            ]),
        ));
    }

    // Every requested calendar failed, so nothing was produced and the process
    // must say so. It exited 0 until 0.9.0 — a script asking for a calendar that
    // does not exist got a success and a table of dashes.
    //
    // A *partial* failure still exits 0 deliberately: the output is useful, the
    // per-row `error` field is visible in text and in `--json`, and turning the
    // default invocation non-zero the moment one body lacks an anchor would
    // make the ordinary case look broken.
    if produced == 0 && !calendars.is_empty() {
        return Err(TimeError::with_context(
            Code::E0062,
            "none of the requested calendars could be rendered; `ucal cal list` \
             names the ones that exist",
        ));
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

/// F1 — a calendar built from §15.1 files rather than from the registry.
///
/// `cal derive --anchor --at` could produce a date from a body file and an
/// anchor file, and then that calendar was stranded: `cal show` and `show` knew
/// only the compiled-in registry, so the one thing the loaders exist for — using
/// a body this program does not ship — stopped at a single command.
///
/// The calendar id is derived from the body's, as `<body>-d`, and the anchor
/// file must name it. Same check `cal derive` makes, same reason: two files that
/// each load, pair up, and quietly produce a date for one body using another
/// body's phase is the borrowing Rule J forbids.
#[cfg(feature = "body")]
pub fn calendar_from_files(
    body_path: &str,
    anchor_path: &str,
) -> Result<ucal_body::calendar::BodyCalendar, TimeError> {
    let body = body_file::load(std::path::Path::new(body_path))?;
    let anchor = anchor_file::load(std::path::Path::new(anchor_path))?;
    let id: &'static str = body_file::leak(format!("{}-d", body.id()));
    if anchor.calendar_id() != id {
        return Err(TimeError::with_context(
            Code::E0062,
            body_file::leak(format!(
                "the anchor file names calendar `{}`, and this body file derives `{id}`",
                anchor.calendar_id()
            )),
        ));
    }
    let satellite = body.satellites().first().map(|s| s.id());
    ucal_body::calendar::BodyCalendar::build(
        id,
        body,
        anchor,
        satellite,
        ucal_body::DriftBound::DEFAULT,
        32,
    )
}

/// `ucal cal show <id> <T>` — one calendar's derivation, in full.
#[cfg(feature = "body")]
pub fn cmd_cal_show(id: &str, input: &str) -> CmdResult {
    let c = bodycal::by_id(id)?;
    cal_show_of(&c, id, input)
}

#[cfg(feature = "body")]
fn cal_show_of(c: &ucal_body::calendar::BodyCalendar, id: &str, input: &str) -> CmdResult {
    let (t, _) = parse_instant(input)?;
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

    // Y1: where this body's own units sit on the universal grid.
    let mut ladder: Vec<(String, Value)> = Vec::new();
    if let Some(r) = ladder_row("solar_day", c.body().solar_day().value_at_epoch()) {
        ladder.push(r);
    }
    if let Some(r) = ladder_row("year", c.body().orbital_period().value_at_epoch()) {
        ladder.push(r);
    }
    if let Some(cy) = c.cycles().first() {
        if let Some(r) = ladder_row("cycle", &cy.synodic_period) {
            ladder.push(r);
        }
    }

    let mut doc = Doc::new()
        .title(format!("ucal cal show {id}"))
        .field("calendar", Value::text(id))
        .field("kind", Value::text("derived — Rule K"))
        .field("body", Value::text(c.body().id()))
        .field("ladder_placement", Value::rows("unit", ladder))
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
    // A calendar that does not exist and a calendar with no anchor are different
    // answers, and until 0.9.0 both produced the same confident document —
    // `ucal cal anchor nope` reported `anchor: none` and exited 0, which reads
    // as "this calendar exists and its phase is undetermined" rather than "there
    // is no such calendar".
    if !bodycal::ids().contains(&id) {
        return Err(TimeError::with_context(
            Code::E0062,
            "no such derived calendar; `ucal cal list` names the ones that exist",
        ));
    }
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
            // Kept, though doubling the catalogue in 1.4.0 pushed the text
            // rendering past one screen. Dropping this field would have fixed
            // that and removed a `ucal-json/1` path, which promise 4 forbids —
            // the fields a consumer reads are not free to move because a
            // *terminal* rendering grew. §20's "one-screen demo" was written
            // against eleven events; the claim is what changed, not the data.
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
    let mid = e.window.midpoint(Rounding::HalfEven)?;

    let mut doc = Doc::new()
        .title(format!("ucal events show {id}"))
        .field("label", Value::text(e.label))
        .field("year", Value::bridge(Value::text(YEAR_DEFINITION)))
        .field("description", Value::text(e.description))
        .field("as_published", Value::text(e.as_published))
        .field(
            "stated_as",
            Value::text(match e.stated_as {
                events::StatedAs::AfterDatum => "after the datum",
                events::StatedAs::BeforeBridgeEpoch => "before the bridge epoch",
                // `StatedAs` is #[non_exhaustive] from 0.6.0: a source could
                // state an event some third way — by redshift, say — and this
                // crate would then be older than the catalogue it is rendering.
                //
                // Saying so is the honest fallback. Guessing a label would put a
                // wrong description of a source's own words into the output,
                // which is the one thing an `as_published` field exists to
                // prevent.
                _ => "stated in a form this version does not recognise; \
                      see `as_published` for the source's own words",
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
                ("width_years".into(), Value::bridge(years_quantity(e.uncertainty().ticks(), 0))),
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
/// The whole of absolute time in one document: each milestone placed at a stated
/// tier, with the tier's own name, so the ladder and the catalogue are read
/// together.
///
/// §20 called this the one-screen demo and it was, at eleven events. The 1.4.0
/// catalogue is twenty-two and reaches T31, so it no longer fits a screen —
/// which is the claim changing, not the data: trimming a field to fit would
/// have removed a `ucal-json/1` path, and promise 4 does not bend for a
/// terminal.
#[cfg(feature = "events")]
pub fn cmd_timeline(tier: Tier) -> CmdResult {
    let mut rows: Vec<(String, Value)> = Vec::new();
    for e in events::chronological() {
        let mid = e.window.midpoint(Rounding::HalfEven)?;
        let at_tier = mid.floor_to(tier);
        let mut fields = vec![
            ("at".into(), Value::text(render_at(&at_tier, tier))),
            (
                // A stable key. This was `format!("{tier}s since the datum")`,
                // so `--tier arc` and `--tier drift` produced different *field
                // names* and no consumer could write an accessor that survived
                // a flag. The tier belongs in a value, and the document already
                // carries it in its own `tier` field.
                "tiers_since_datum".to_string(),
                Value::number(mid.ticks().quot_rem(&tier.ticks()).0.to_dec_string()),
            ),
            // Kept, though doubling the catalogue in 1.4.0 pushed the text
            // rendering past one screen. Dropping this field would have fixed
            // that and removed a `ucal-json/1` path, which promise 4 forbids —
            // the fields a consumer reads are not free to move because a
            // *terminal* rendering grew. §20's "one-screen demo" was written
            // against eleven events; the claim is what changed, not the data.
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
            "the ruler's start must not follow its end; swap `--from` and `--to`",
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
    // ucal-lint-allow-begin(no-panic-in-cli): SECOND x 31 557 600 is a constant
    // some forty orders of magnitude inside the domain ceiling. See `tick_ratio`
    // for why this is asserted rather than propagated.
    let year = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .expect("a Julian year fits the domain");
    // ucal-lint-allow-end(no-panic-in-cli)
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
    // A negative redshift reached the decimal parser and came back "not an
    // exact decimal", which is false: -1 is an exact decimal and simply not a
    // redshift. Saying so here means the reader is told what is wrong with
    // their input rather than something that is wrong about it.
    if t.starts_with('-') {
        return Err(TimeError::with_context(
            Code::E0018,
            "a redshift is not negative: z = 0 is now and larger is earlier. Try `--z 1100` for recombination, or `--z 0.5`",
        ));
    }
    match t.split_once("..") {
        Some((lo, hi)) => {
            let (lo, hi) = (
                Ratio::from_decimal_str(lo.trim())?,
                Ratio::from_decimal_str(hi.trim())?,
            );
            RatInterval::new(lo, hi).map_err(|_| {
                TimeError::with_context(
                    Code::E0022,
                    "a redshift interval is written low..high, e.g. `--z 1090..1110`",
                )
            })
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
    // ucal-lint-allow-begin(no-panic-in-cli): as `ticks_in_years`.
    let year = UC1::bridge()
        .ticks
        .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
        .expect("a Julian year fits the domain");
    // ucal-lint-allow-end(no-panic-in-cli)
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
        .field("year", Value::bridge(Value::text(YEAR_DEFINITION)))
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
                ("lo_years".into(), Value::bridge(years_quantity(out.value.lo().ticks(), 0))),
                ("hi_years".into(), Value::bridge(years_quantity(out.value.hi().ticks(), 0))),
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
                ("arithmetic_years".into(), Value::bridge(years_quantity(out.arithmetic_width.ticks(), 1))),
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
                ("parameter_years".into(), Value::bridge(years_quantity(out.parameter_width.ticks(), 1))),
                (
                    "note".into(),
                    Value::text(
                        "Rule X: quadrature error and parameter uncertainty are \
                         reported separately and never merged (F8). The second is \
                         what the measurement does not know; the first is what this \
                         program does not know. Each is given in ticks and in \
                         drifts, both body-independent; `--bridge` adds the \
                         foreign-unit conversion, which is not performed unasked.",
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
                ("years".into(), Value::bridge(years_quantity(out.input_width.ticks(), 1))),
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
        .ok_or(TimeError::with_context(Code::E0021, "a Julian year in ticks overflows the domain"))?;
    let tolerance = Delta::from_ticks(
        year.try_mul(&<Ticks as TickInt>::from_u64(tolerance_years.max(1)))
            .ok_or(TimeError::with_context(Code::E0021, "the tolerance window overflows the domain"))?,
    );
    let out = model.z_of_t(&window, &tolerance, depth, scale)?;

    Ok(Doc::new()
        .title("ucal cosmo z")
        .field("instant_ticks", Value::number(t.ticks().to_dec_string()))
        .field("years_after_datum", Value::bridge(Value::text(ticks_in_years(t.ticks(), 0))))
        .field("year", Value::bridge(Value::text(YEAR_DEFINITION)))
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
        .field("year", Value::bridge(Value::text(YEAR_DEFINITION)))
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
                    Value::bridge(Value::text(dec(
                        &model
                            .hubble_time
                            .lo()
                            .div(&Ratio::from_int(
                                // ucal-lint-allow-begin(no-panic-in-cli): a
                                // gigayear in ticks, still far inside the
                                // domain. As `tick_ratio`.
                                UC1::bridge()
                                    .ticks
                                    .try_mul(&<Ticks as TickInt>::from_u64(31_557_600))
                                    .and_then(|y| {
                                        y.try_mul(&<Ticks as TickInt>::from_u64(1_000_000_000))
                                    })
                                    .expect("a gigayear fits the domain"),
                                // ucal-lint-allow-end(no-panic-in-cli)
                            ))?,
                        3,
                    ))),
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

// ---------------------------------------------------------------------------
// ucal tour — the first five minutes
// ---------------------------------------------------------------------------

/// One step of the tour: what to type, what it shows, and why it looks like that.
///
/// `shows` is not a transcript. It is pulled out of the command by **running
/// it**, here, now — so a step cannot advertise output the program does not
/// produce. Two documents in this repository drifted from the code in 2026 and
/// both were copies of something generated; a tour is the worst possible place
/// for a third, because its whole audience is people with no way to tell.
struct Step {
    command: &'static str,
    shows: String,
    why: &'static str,
}

/// The shortest path from *installed* to *I see what this is*.
///
/// # What this is not
///
/// Not the manual: [`Documentation/CLI.md`] answers *what does this field mean*
/// for a reader who already knows which command to run. Not the book, which is
/// the argument at length. This answers **what should I type first**, which
/// nothing did — a stranger arriving with the binary in front of them had
/// `--help`, a reference, and ninety pages, in ascending order of commitment.
///
/// # Why it is a guess, and says so
///
/// Every step here is a choice about a reader who has never existed. Four
/// cycles of asking (`Documentation/CONTACT.md`) have produced nobody, so this
/// is not informed by use — it is the author's best guess at which five things
/// make the point, and the closing note says as much rather than presenting
/// itself as a considered curriculum.
///
/// [`Documentation/CLI.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/CLI.md
/// `ucal wallclock --theme list` — the themes, as a document.
///
/// A catalogue, so it is enumerable: a caller told "no such theme" should be
/// able to find out what there is, which is the same reason `ucal cal list`
/// exists.
#[cfg(feature = "tui")]
pub fn cmd_wallclock_themes() -> Doc {
    Doc::new()
        .title("ucal wallclock themes")
        .field(
            "themes",
            Value::rows(
                "theme",
                wallclock::theme::ALL
                    .iter()
                    .map(|t| {
                        (
                            t.key.to_string(),
                            Value::Section(vec![("about".into(), Value::text(t.about))]),
                        )
                    })
                    .collect::<Vec<_>>(),
            ),
        )
        .note(
            "The clock shows T3 span down to T-1 flicker. Above T3 a hand does not move \
             within a human lifetime — one T4 is 141 000 years — and below T-1 it moves \
             66 000 times a second, which no refresh rate reaches.",
        )
}

/// `ucal tour` — the first five minutes.
#[allow(clippy::doc_markdown)]
pub fn cmd_tour() -> CmdResult {
    let t = "8070205189123984864657505252035637180530466139316558837890625";

    // Each `shows` is read out of a real document, by field, so it is the value
    // the command actually prints today.
    let field_of = |doc: &Doc, key: &str| -> String {
        doc.fields()
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.rendered_text().trim().to_string())
            .unwrap_or_default()
    };

    let datum = cmd_datum()?;
    let explain = cmd_explain(t, false)?;
    let between = cmd_between("0", t, Some(Tier::BEAT))?;
    let verify = cmd_verify()?;
    let ladder = cmd_ladder(LocaleId::En, true)?;

    let steps = [
        Step {
            command: "ucal datum",
            shows: field_of(&datum, "datum"),
            why: "Start here. Tick 0 is stipulated, not measured — the command \
                  says so itself, and prints the chain of published values that \
                  fixed it. Everything else in the program is counting from a \
                  point this command refuses to overclaim.",
        },
        Step {
            command: "ucal explain <instant>",
            shows: field_of(&explain, "ticks"),
            why: "Sixty-one digits, exact. Not a float, not a truncation, not a \
                  timestamp with a resolution — an integer count of Planck \
                  times. The forms below it are the same integer written for \
                  different readers.",
        },
        Step {
            command: "ucal between 0 <instant> --at beat",
            shows: field_of(&between, "natural_tier"),
            why: "A duration's home is the tier ladder, not a number of \
                  seconds. This is the one command that answers `how far \
                  apart`, and it answers in the grid's own units before it \
                  answers in anybody else's.",
        },
        Step {
            command: "ucal ladder --named-only",
            shows: field_of(&ladder, "note"),
            why: "Forty-five rungs, each 3125 times the last. The names are \
                  decoration — a tier's identity is its exponent — and nothing \
                  on the ladder is near an hour, which is the cost of leaving \
                  the Earth paradigm rather than an oversight.",
        },
        Step {
            command: "ucal verify",
            shows: field_of(&verify, "agrees"),
            why: "The binary re-derives the constants it ships with and says \
                  whether it reproduces them. It also says, in the output, that \
                  agreeing with itself is not verification — which is the ask in \
                  CONTACT.md and the one thing this project cannot do alone.",
        },
    ];

    // A step whose `shows` is empty reads a field that no longer exists. The
    // first draft omitted the row, silently, after exactly that happened — the
    // failure the design was meant to prevent, in the one document written for
    // readers who cannot tell.
    //
    // Not a runtime error. Whether these fields resolve is a property of the
    // build, fixed at compile time and checked by `tour.rs` on every push, so
    // it cannot reach a release; raising an Appendix E code for it would mean
    // borrowing one whose meaning is something else, which this project has
    // done twice and corrected twice. What ships instead is a marker that is
    // impossible to mistake for output.
    const MISSING: &str = "— (this step's source field is missing; please report it)";

    let rows: Vec<(String, Value)> = steps
        .iter()
        .map(|s| {
            (
                s.command.to_string(),
                Value::Section(vec![
                    (
                        "shows".into(),
                        Value::text(if s.shows.is_empty() {
                            MISSING.to_string()
                        } else {
                            s.shows.clone()
                        }),
                    ),
                    ("why".into(), Value::text(s.why)),
                ]),
            )
        })
        .collect();

    Ok(Doc::new()
        .title("ucal tour")
        .field("start", Value::text(
            "Five commands, in order. Each line below is one you can type; the \
             `shows` beside it was produced by running that command just now, \
             not copied from a transcript.",
        ))
        .field("instant", Value::number(t))
        .field("steps", Value::rows("command", rows))
        .field("next", Value::text(
            "`ucal --help` lists everything. `ucal man` is the manual page and \
             `ucal completions <shell>` the completions. Documentation/CLI.md \
             explains every field of every command.",
        ))
        .note(
            "This tour is a guess. Nobody outside this repository has used the \
             program, so which five commands make the point is the author's \
             opinion and not a finding — see Documentation/CONTACT.md, where \
             saying it is the wrong five would be a useful thing to report.",
        ))
}

// ---------------------------------------------------------------------------
// ucal cal derive — X1.4: what calendar does this body imply?
// ---------------------------------------------------------------------------

/// `ucal cal derive <file>` — read a body file and show the calendar it derives.
///
/// The answer to `X1-authoring-local-calendars.md`'s question: somebody who is
/// not the author can now write a body and see what falls out of it, without
/// editing this crate.
///
/// What falls out is the intercalation and the cycles. What does not is the
/// **phase**, and this command says so rather than leaving it to be discovered:
/// an anchor is cited and determined, never derived, and D5's literature search
/// established what one costs to establish honestly. A body file therefore
/// produces a calendar that is complete in units, intercalation and cycles and
/// incomplete in phase — the ordinary case, and the state most shipped calendars
/// are in. The count is taken from the registry rather than written down: it was
/// written down once, said "five of the seven", and was wrong the moment Y3
/// added five bodies.
#[cfg(feature = "body")]
fn anchorless_note() -> String {
    let derived = ucal_body::calendar::registered();
    let total = derived.len();
    let anchorless = derived
        .iter()
        .filter(|(id, _, _)| ucal_body::anchors::for_calendar(id).is_none())
        .count();
    format!(
        "none. Phase is empirical (Rule J): it is determined and cited, never derived and \
         never borrowed from another body. Without one this calendar is complete in units, \
         intercalation and cycles and incomplete in phase, which is the ordinary case — \
         {anchorless} of the {total} derived calendars that ship are in it"
    )
}

/// Derive a calendar from a body file (§15.1), and say what it is missing.
///
/// See [`body_file`] for why the loader lives in the binary.
#[cfg(feature = "body")]
pub fn cmd_cal_derive(path: &str) -> CmdResult {
    cmd_cal_derive_with(path, None, None)
}

/// Y2 — the same derivation, optionally given §15.1's other file.
///
/// A body file yields intercalation and cycles. It cannot yield a **date**,
/// because a date needs a phase, and Rule J makes phase empirical: determined
/// and cited, never derived. That is the whole reason `cal derive` has always
/// ended with a paragraph about what is missing.
///
/// With an anchor file, the missing half arrives — from a file, subject to the
/// identical checks `Anchor::new` applies to the compiled-in anchors. The
/// loader adds none of its own and must not: a file is a much easier place to
/// narrow a window by assumption than a Rust constant, which is what GE-3
/// forbids and what `X1.3` named as this feature's kill criterion.
#[cfg(feature = "body")]
pub fn cmd_cal_derive_with(path: &str, anchor: Option<&str>, at: Option<&str>) -> CmdResult {
    let body = body_file::load(std::path::Path::new(path))?;

    let solar = body.solar_day().value_at_epoch();
    let year = body.orbital_period().value_at_epoch();
    // Z1.3: a year that is a whole number of solar days needs no intercalation,
    // and that is an answer rather than a failure. The derivation reports it as
    // UCAL-E0061 — *no convergent meets the drift bound* — advising a wider
    // bound or a greater depth, neither of which can help: there is no
    // fractional part to approximate. Checked here because it can only arise
    // from a file; no shipped body is in this state.
    let days_per_year = year.div(solar)?;
    if days_per_year.is_integer() {
        return Err(TimeError::with_context(
            Code::E0060,
            "this body's year is a whole number of its solar days, so its calendar needs no \
             intercalation at all: there is no fractional day to distribute, and Rule K has \
             nothing to derive. That is the answer, not a gap",
        ));
    }
    let rule = ucal_body::derive_leap_rule(solar, year, ucal_body::DriftBound::DEFAULT, 32)?;

    let mut doc = Doc::new()
        .title("ucal cal derive")
        .field("body", Value::text(body.id()))
        .field(
            "primary",
            Value::text(body.primary().unwrap_or("— (orbits nothing this file names)")),
        )
        .field(
            "days_per_year",
            Value::quantity(&days_per_year, 6, Rounding::HalfEven),
        )
        .field(
            "leap_rule",
            Value::Section(vec![
                (
                    "rule".into(),
                    Value::text(format!(
                        "{}/{}",
                        rule.chosen.value.numer().to_dec_string(),
                        rule.chosen.value.denom().to_dec_string()
                    )),
                ),
                ("convergent".into(), Value::number(rule.depth.to_string())),
                (
                    "whole_days_per_year".into(),
                    Value::number(rule.whole_days.numer().to_dec_string()),
                ),
                (
                    "placement".into(),
                    Value::text(
                        "even: days_before_year(y) = y x whole + floor(y x p / q), declared by D-A21",
                    ),
                ),
            ]),
        );

    // Cycles, or the statement that there are none. §15.3 forbids a fallback.
    //
    // The grouping satellite is the *calendar's* declaration, not the body's —
    // D-A5 made cycles declared per body rather than admitted by a global
    // bracket, because "month-like" is an Earth predicate. A file that lists
    // satellites gets the first as the grouping one; a file that lists none
    // gets no cycle, which is the correct output and not a gap.
    let grouping = body.satellites().first().map(|s| s.id());
    let cycles = ucal_body::derive_cycles(&body, grouping, 32)?;
    doc = doc.field(
        "cycles",
        match cycles.first() {
            None => Value::text(
                "none — this body names no grouping satellite, so its calendar has no month. \
                 That is the output, not a gap (§15.3 forbids a fallback structure)",
            ),
            Some(c) => Value::Section(vec![
                ("satellite".into(), Value::text(c.satellite)),
                (
                    "cycles_per_year".into(),
                    Value::quantity(&c.ratio, 6, Rounding::HalfEven),
                ),
                (
                    "chosen".into(),
                    Value::text(match c.convergents.last() {
                        Some(v) => format!(
                            "{}/{}",
                            v.value.numer().to_dec_string(),
                            v.value.denom().to_dec_string()
                        ),
                        None => "— (no convergent)".to_string(),
                    }),
                ),
            ]),
        },
    );

    let doc = match anchor {
        None => {
            if at.is_some() {
                return Err(TimeError::with_context(
                    Code::E0062,
                    "--at asks for a date, and a date needs a phase: pass --anchor <FILE> as \
                     well. Phase is empirical (Rule J) and is never derived from a body file",
                ));
            }
            doc.field("anchor", Value::text(anchorless_note()))
        }
        Some(a) => {
            let anchor = anchor_file::load(std::path::Path::new(a))?;
            let id: &'static str = body_file::leak(format!("{}-d", body.id()));
            if anchor.calendar_id() != id {
                return Err(TimeError::with_context(
                    Code::E0062,
                    body_file::leak(format!(
                        "the anchor file names calendar `{}`, and this body file derives `{id}`",
                        anchor.calendar_id()
                    )),
                ));
            }
            let satellite = body.satellites().first().map(|s| s.id());
            let cal = ucal_body::calendar::BodyCalendar::build(
                id,
                body.clone(),
                anchor,
                satellite,
                ucal_body::DriftBound::DEFAULT,
                32,
            )?;
            let a = cal.anchor();
            let doc = doc.field(
                "anchor",
                Value::Section(vec![
                    ("phase".into(), Value::text(a.phase().label())),
                    ("revision".into(), Value::number(a.revision().to_string())),
                    ("method".into(), Value::text(a.method().method)),
                    (
                        "uncertainty".into(),
                        Value::text(a.method().uncertainty_note),
                    ),
                    (
                        "window_ticks".into(),
                        Value::number(a.uncertainty().ticks().to_dec_string()),
                    ),
                    ("citation".into(), Value::text(a.citation().source)),
                ]),
            );
            match at {
                None => doc.note(
                    "An anchor is present, so this calendar can produce a date: pass \
                     --at <INSTANT> for one.",
                ),
                Some(t) => {
                    let (t, _) = parse_instant(t)?;
                    let f = cal.fields(&t)?;
                    doc.field(
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
                    )
                }
            }
        }
    };

    Ok(doc.note(
        "Loaded by the binary, not by ucal-body. §15.1 puts the loader in the library and \
         D-A20 records that it is not there: every string in the data model is a \
         `&'static str`, so a runtime loader must either leak or change a published type. \
         This one leaks, bounded by a process that exits.",
    ))
}
