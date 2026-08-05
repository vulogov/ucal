//! §20 UC-P9 exit criterion: **golden-output tests; `ucal datum` matches §19.2
//! ordering and makes no measurement claim.**
//!
//! The commands are pure functions from arguments to a document, so these tests
//! call them directly. That is faster than spawning a process, and it lets the
//! ordering assertion inspect the field list rather than scraping text.

use ucal::emit::Value;
use ucal::{cmd_datum, cmd_doctor, cmd_explain, cmd_from_civil, cmd_to_civil, exit_code};
use ucal_civil::calendar::CivilCalendar;
use ucal_civil::si::Scale;
use ucal_core::{Code, Rounding};

const APPENDIX_C_2026: &str =
    "8070205189123984864657505252035637180530466139316558837890625";
const SI_EPOCH: &str = "8070204002895596515944343085635637180530466139316558837890625";

// ---------------------------------------------------------------------------
// §19.2 — ucal datum
// ---------------------------------------------------------------------------

#[test]
fn datum_prints_its_parts_in_the_order_19_2_requires() {
    // "MUST print, in this order: the datum statement, BIG_BANG_CLAIM with
    //  citation, the full provenance chain from §2.2, and the rounding residual."
    let doc = cmd_datum().unwrap();
    let keys = doc.keys();

    let pos = |k: &str| {
        keys.iter()
            .position(|x| *x == k)
            .unwrap_or_else(|| panic!("`ucal datum` is missing the `{k}` field"))
    };

    let statement = pos("datum");
    let claim = pos("big_bang_claim");
    let provenance = pos("datum_provenance");
    let rounding = pos("rounding");

    assert!(statement < claim, "the datum statement must come first");
    assert!(claim < provenance, "the claim must precede the provenance");
    assert!(
        provenance < rounding,
        "the provenance chain must precede the residual"
    );
}

#[test]
fn datum_makes_no_measurement_claim() {
    // §19.2: "MUST NOT present the implied age as a measurement of the universe."
    // Rule Q.1 forbids describing tick 0 as measured, derived, observed, or as
    // the creation of anything.
    let text = cmd_datum().unwrap().to_text().to_lowercase();

    for forbidden in [
        "creation of the universe",
        "the age of the universe is",
        "the universe is 13.787",
        "beginning of time",
        "when the universe began",
        "the big bang happened",
        "measured age",
    ] {
        assert!(
            !text.contains(forbidden),
            "`ucal datum` claims too much: found {forbidden:?}"
        );
    }

    // And it must positively say what the datum is.
    assert!(text.contains("stipulated"), "the datum must be called stipulated");
    assert!(text.contains("flrw"), "the frame must be named");

    // The implied age must be labelled as a consequence, not a measurement.
    let doc = cmd_datum().unwrap();
    let Some(Value::Section(implied)) = doc.get("implied_age") else {
        panic!("expected an implied_age section");
    };
    let note = implied
        .iter()
        .find(|(k, _)| k == "note")
        .map(|(_, v)| format!("{v:?}"))
        .unwrap_or_default()
        .to_lowercase();
    assert!(
        note.contains("consequence") && note.contains("not a measurement"),
        "the implied age must be labelled a consequence of the datum: {note}"
    );
}

#[test]
fn datum_reports_the_claim_as_non_operand() {
    let doc = cmd_datum().unwrap();
    let Some(Value::Section(claim)) = doc.get("big_bang_claim") else {
        panic!("expected a big_bang_claim section");
    };
    // Rendered, not Debug-formatted. A `Quantity` carries the exact rational
    // and renders late, so its Debug shows a numerator and denominator where a
    // reader sees digits — and it is the digits this test is about.
    let joined = format!(
        "{claim:?} {}",
        Value::Section(claim.clone()).rendered_text()
    )
    .to_lowercase();
    // Rule Q.3: reportable metadata, never an operand.
    assert!(joined.contains("metadata"));
    assert!(joined.contains("rule q.3"));
    // With a citation, as Rule Q.3 requires.
    assert!(joined.contains("planck 2018"));
    // And the half-width, in both ticks and drifts.
    assert!(joined.contains("11706976141141069872000000000000000000000000000000000000000"));
    assert!(joined.contains("141.53"));
}

#[test]
fn datum_provenance_chain_is_printed_in_full() {
    // §19.2 requires the full chain from §2.2, not a summary.
    let doc = cmd_datum().unwrap();
    let Some(Value::Section(prov)) = doc.get("datum_provenance") else {
        panic!("expected a datum_provenance section");
    };
    let Some((_, Value::List(chain))) = prov.iter().find(|(k, _)| k == "chain") else {
        panic!("expected the chain");
    };
    assert_eq!(chain.len(), 4, "the §2.2 chain has four steps");
    assert!(chain[0].contains("435 084 631 200 000 000"));
    assert!(chain[1].contains("8070204002895596516263200000000000000000000000000000000000000"));
    assert!(chain[2].contains("9 304 311 741 502 590 385"));
    assert!(chain[3].contains("8070204002895596515944343085635637180530466139316558837890625"));

    // And the residual, signed, with its rendering.
    let doc = cmd_datum().unwrap();
    let Some(Value::Section(rounding)) = doc.get("rounding") else {
        panic!("expected a rounding section");
    };
    let joined = format!("{rounding:?}");
    assert!(joined.contains("-318856914364362819469533860683441162109375"));
    assert!(joined.contains("-0.017190364"));
    assert!(joined.contains("half_even"));
}

#[test]
fn the_implied_age_is_rendered_exactly_not_rounded() {
    // Appendix A prints the implied age as 435 084 631 200 000 000.0 s, which is
    // the *unrounded* AGE_s. `ORIGIN_OFFSET` is the datum rounded down to a whole
    // beat, so the true quotient is smaller by exactly the documented residual.
    // See spec/SPEC-DELTAS.md D-A10.
    let doc = cmd_datum().unwrap();
    let Some(Value::Section(implied)) = doc.get("implied_age") else {
        panic!("expected implied_age");
    };
    // The implied age is a foreign-unit rendering, so it is behind `--bridge`
    // since D-A16 — but §19.2 still governs *what* it says when asked for.
    let seconds = implied
        .iter()
        .find(|(k, _)| k == "seconds")
        .and_then(|(_, v)| match v {
            Value::Bridge(inner) => inner.rendered_opt(&ucal::style::Render::PLAIN),
            other => other.rendered_opt(&ucal::style::Render::PLAIN),
        })
        .map(|(text, _)| text)
        .unwrap();
    assert!(
        seconds.contains("435084631199999999.982810"),
        "expected the exact quotient, got {seconds}"
    );
}

// ---------------------------------------------------------------------------
// §19.3 — ucal doctor
// ---------------------------------------------------------------------------

#[test]
fn doctor_reports_everything_19_3_names() {
    let doc = cmd_doctor().unwrap();
    let keys = doc.keys();
    for required in [
        "profile",
        "backend",
        "domain_max_ticks",
        "features",
        "datum_provenance",
    ] {
        assert!(keys.contains(&required), "doctor is missing `{required}`");
    }
    let text = doc.to_text();
    // The domain ceiling, exactly. 155 digits do not fit in 80 columns, so it
    // hangs under its own label across three lines — rejoining them must return
    // it character for character, which is the property that makes wrapping safe
    // for an exact quantity. Checked against the *structure* too, so this cannot
    // pass on a value that merely appears somewhere in the prose.
    const CEILING: &str = "13407807929942597099574024998205846127479365820592393377723561443721764030073546976801874298166903427690031858186486050853753882811946569946433649006084095";
    let joined: String = text.lines().map(str::trim_start).collect();
    assert!(joined.contains(CEILING), "the domain ceiling did not survive wrapping");
    assert!(matches!(
        doc.get("domain_max_ticks"),
        Some(Value::Number(n)) if n == CEILING
    ));
    // The leap-second table version (§8.4).
    assert!(text.contains("Bulletin C"), "the leap table version must be reported");
    // Provenance presence (Rule Q.4).
    assert!(text.contains("present"));
    // Offline (§8.4).
    assert!(text.to_lowercase().contains("no network"));
}

// ---------------------------------------------------------------------------
// golden conversions
// ---------------------------------------------------------------------------

#[test]
fn from_civil_reproduces_appendix_c() {
    let doc = cmd_from_civil("2026-07-29", Scale::Tt, CivilCalendar::Gregorian).unwrap();
    assert_eq!(
        doc.get("ticks"),
        Some(&Value::number(APPENDIX_C_2026)),
        "Appendix C's 2026-07-29 fixture"
    );
    let doc = cmd_from_civil("0000-01-01", Scale::Tt, CivilCalendar::Gregorian).unwrap();
    assert_eq!(doc.get("ticks"), Some(&Value::number(SI_EPOCH)));
}

#[test]
fn the_era_suffix_is_normalised_per_2_5() {
    // §2.5: astronomical numbering, so 1 BC is year 0 and 44 BC is year -43.
    let a = cmd_from_civil("44 BC-03-15", Scale::Tt, CivilCalendar::Gregorian).unwrap();
    let b = cmd_from_civil("-0043-03-15", Scale::Tt, CivilCalendar::Gregorian).unwrap();
    assert_eq!(a.get("ticks"), b.get("ticks"));
    // Appendix C's 44 BC fixture, on the proleptic Gregorian calendar.
    assert_eq!(
        a.get("ticks"),
        Some(&Value::number(
            "8070203977843789392286957152835637180530466139316558837890625"
        ))
    );
}

#[test]
fn to_civil_output_is_always_qualified() {
    // §6.6: no rendering path may omit the calendar id and kind.
    let doc = cmd_to_civil(
        APPENDIX_C_2026,
        Scale::Tt,
        0,
        Rounding::Trunc,
        CivilCalendar::Gregorian,
    )
    .unwrap();
    assert_eq!(
        doc.get("qualified"),
        Some(&Value::text("earth-civil: 2026-07-29T00:00:00 TT"))
    );
    assert_eq!(doc.get("calendar_id"), Some(&Value::text("earth-civil")));
    let kind = format!("{:?}", doc.get("kind").unwrap());
    assert!(kind.contains("legacy"), "the kind must be stated");
    // And the legacy warning is carried (§8.6).
    let text = doc.to_text();
    assert!(text.contains("UCAL-W0005"), "legacy output must carry W0005");
}

#[test]
fn to_civil_in_the_julian_calendar_differs_and_says_so() {
    let g = cmd_to_civil(
        APPENDIX_C_2026,
        Scale::Tt,
        0,
        Rounding::Trunc,
        CivilCalendar::Gregorian,
    )
    .unwrap();
    let j = cmd_to_civil(
        APPENDIX_C_2026,
        Scale::Tt,
        0,
        Rounding::Trunc,
        CivilCalendar::Julian,
    )
    .unwrap();
    assert_ne!(g.get("qualified"), j.get("qualified"));
    assert_eq!(j.get("calendar_id"), Some(&Value::text("earth-julian")));
}

#[test]
fn explain_shows_both_forms_and_the_si_bridge() {
    let doc = cmd_explain(APPENDIX_C_2026, false).unwrap();
    let text = doc.to_text();

    // §4.3 said the SI equivalent is *always* printed alongside. D-A16 amends
    // that: an SI second is an Earth unit and `ucal explain` is not an Earth
    // command, so the conversion is printed on request and not unasked. It must
    // still be reachable, and it must still be absent by default — both halves
    // are the amendment.
    assert!(!text.contains("si_bridge"), "a foreign unit appeared unasked");
    let asked = doc.render(&ucal::style::Render::PLAIN.bridge(true));
    assert!(asked.contains("si_bridge"));
    assert!(asked.contains("second"));
    // Both text forms (Rule D).
    assert!(text.contains("UC1 0031·0687·2481·2999·3108·2437"));
    assert!(text.contains("UC1/5"));
    // The UCID.
    assert!(text.contains("0000000000050PM6K45HH4YGQJ6SEDGDDZ1NKFHD32F2XBM29FJ1"));
}

#[test]
fn explain_warns_inside_the_claim_half_width() {
    // §10.6: a quantity comparable to BIG_BANG_CLAIM must surface UCAL-W0006.
    // Recombination, at 380 kyr, is far inside the 141.53-drift half-width.
    let recombination = "222432546681680327568000000000000000000000000000000000000";
    let text = cmd_explain(recombination, false).unwrap().to_text();
    assert!(text.contains("UCAL-W0006"), "expected the claim warning");

    // A present-epoch instant is far outside it and must not warn.
    let text = cmd_explain(APPENDIX_C_2026, false).unwrap().to_text();
    assert!(!text.contains("UCAL-W0006"));
}

#[test]
fn explain_claim_flag_reports_metadata_only() {
    let text = cmd_explain(APPENDIX_C_2026, true).unwrap().to_text();
    assert!(text.contains("never an operand"), "Rule Q.3 must be stated");
    assert!(!cmd_explain(APPENDIX_C_2026, false)
        .unwrap()
        .to_text()
        .contains("never an operand"));
}

#[test]
fn a_truncated_input_reports_a_window_not_a_tick() {
    // Rule T end to end, through the CLI.
    let doc = cmd_explain("UC1 0031·0687·2481·2999·3108·2437", false).unwrap();
    let text = doc.to_text();
    assert!(text.contains("window"), "a truncated form denotes a window");
    assert!(
        text.contains("denotes a window"),
        "the precision must say so"
    );
    assert!(!text.contains("tick (exact)"));
}

// ---------------------------------------------------------------------------
// §19.1 — --json is stable and versioned
// ---------------------------------------------------------------------------

#[test]
fn json_output_is_versioned_and_well_formed() {
    for doc in [
        cmd_datum().unwrap(),
        cmd_doctor().unwrap(),
        cmd_explain(APPENDIX_C_2026, true).unwrap(),
        cmd_from_civil("2026-07-29", Scale::Tt, CivilCalendar::Gregorian).unwrap(),
        cmd_to_civil(
            APPENDIX_C_2026,
            Scale::Tt,
            3,
            Rounding::HalfEven,
            CivilCalendar::Gregorian,
        )
        .unwrap(),
    ] {
        let j = doc.to_json();
        assert!(j.starts_with("{\n"), "JSON must be an object");
        assert!(j.contains("\"format\": \"ucal-json/1\""), "versioned (§19.1)");
        assert!(j.trim_end().ends_with('}'));
        assert_eq!(
            j.matches('{').count(),
            j.matches('}').count(),
            "unbalanced braces"
        );
        assert_eq!(
            j.matches('[').count(),
            j.matches(']').count(),
            "unbalanced brackets"
        );
    }
}

#[test]
fn json_emits_tick_counts_as_strings() {
    // A 61-digit integer would be destroyed by a JSON double, which would quietly
    // undo the exactness the whole specification is for.
    let j = cmd_from_civil("2026-07-29", Scale::Tt, CivilCalendar::Gregorian)
        .unwrap()
        .to_json();
    assert!(j.contains(&format!("\"ticks\": \"{APPENDIX_C_2026}\"")));
}

// ---------------------------------------------------------------------------
// §19.5 — exit codes
// ---------------------------------------------------------------------------

#[test]
fn exit_codes_follow_19_5() {
    let cases: [(Code, i32); 8] = [
        (Code::E0001, 2), // parse error
        (Code::E0041, 2), // invalid civil date
        (Code::E0020, 3), // precedes the datum
        (Code::E0021, 3), // exceeds the domain
        (Code::E0023, 4), // indeterminate precision
        (Code::E0043, 4), // finer than the bridge permits
        (Code::E0050, 5), // profile mismatch
        (Code::E0013, 6), // missing provenance
    ];
    for (code, want) in cases {
        assert_eq!(
            exit_code(&ucal_core::TimeError::new(code)),
            want,
            "{code:?} should exit {want}"
        );
    }
    // Rule Q.3's violation is an internal invariant failure.
    assert_eq!(exit_code(&ucal_core::TimeError::new(Code::E0025)), 9);
    // Rule K.6's is a calendar error.
    assert_eq!(exit_code(&ucal_core::TimeError::new(Code::E0065)), 7);
}

#[test]
fn bad_input_is_an_error_not_a_panic() {
    for bad in ["not-an-instant", "", "UC1 99999", "ZZZ"] {
        assert!(cmd_explain(bad, false).is_err(), "input {bad:?}");
    }
    assert!(cmd_from_civil("nonsense", Scale::Tt, CivilCalendar::Gregorian).is_err());
    // §14.3: out of range is E0040, never a panic.
    let e = cmd_from_civil("999999999999-01-01", Scale::Tt, CivilCalendar::Gregorian)
        .unwrap_err();
    assert_eq!(e.code, Code::E0040);
}

// ---------------------------------------------------------------------------
// §19 — ucal ladder, and the locale tables (Appendix D, Rule N)
// ---------------------------------------------------------------------------

#[test]
fn ladder_renders_every_tier_in_both_locales() {
    use ucal::cmd_ladder;
    use ucal_core::LocaleId;

    for loc in LocaleId::ALL {
        let doc = cmd_ladder(*loc, false).unwrap();
        let text = doc.to_text();
        // One entry per tier, named or not. Asserted on the rows rather than on
        // the rendering: `tiers` is a table now, and whether a row key is
        // followed by a colon is a layout question, not this test's.
        let tiers = doc.rows("tiers").expect("ladder has tier rows");
        for k in [32i8, 5, 0, -3, -12] {
            let want = format!("T{k}");
            assert!(
                tiers.iter().any(|(id, _)| *id == want),
                "T{k} missing in {}",
                loc.tag()
            );
            assert!(text.contains(&want), "T{k} not rendered in {}", loc.tag());
        }
        // The bridge equivalent is always alongside (§4.3, D-2).
        assert!(text.contains("seconds"));
    }
}

#[test]
fn ladder_uses_the_locale_names() {
    use ucal::cmd_ladder;
    use ucal_core::LocaleId;

    let en = cmd_ladder(LocaleId::En, true).unwrap().to_text();
    assert!(en.contains("deep / deeps"));
    assert!(en.contains("beat / beats"));

    let ru = cmd_ladder(LocaleId::Ru, true).unwrap().to_text();
    assert!(ru.contains("глубь / глуби"));
    assert!(ru.contains("бой / бои"));
    // Rule N: the names differ, the exponents do not.
    assert!(en.contains("85") && ru.contains("85"));
}

#[test]
fn unnamed_tiers_are_shown_as_addressable_by_index() {
    use ucal::cmd_ladder;
    use ucal_core::LocaleId;

    let text = cmd_ladder(LocaleId::En, false).unwrap().to_text();
    // D-20: unnamed, but Rule N guarantees T[k] and 5^e work.
    assert!(text.contains("address as T7 or 5^95"));
    // ...and `--named-only` omits them entirely.
    let named = cmd_ladder(LocaleId::En, true).unwrap().to_text();
    assert!(!named.contains("T7:"));
}

#[test]
fn tier_names_from_any_locale_resolve_on_input() {
    use ucal::parse_tier;
    use ucal_core::codec::resolve_tier_name_in;
    use ucal_core::{LocaleId, Tier};

    // Rule N: names are display-only, so a name from either locale, the stable
    // key, T[k] and 5^e must all reach the same tier.
    assert_eq!(parse_tier("beat").unwrap(), Tier::BEAT);
    assert_eq!(parse_tier("T0").unwrap(), Tier::BEAT);
    assert_eq!(parse_tier("5^60").unwrap(), Tier::BEAT);
    assert_eq!(
        resolve_tier_name_in(LocaleId::Ru, "бой").unwrap(),
        Tier::BEAT
    );
    // The stable key works under any locale.
    assert_eq!(
        resolve_tier_name_in(LocaleId::Ru, "beat").unwrap(),
        Tier::BEAT
    );
}
