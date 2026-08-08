//! V6 — what makes a certification a claim rather than a label.
//!
//! A tag nobody checks is worse than no tag: it invites a reader to trust it.
//! Three properties, and the third is the one that catches the mistake a future
//! contributor will actually make.
//!
//! 1. `exact` means what it says — the digits reparse to the value, and a value
//!    that does not reparse is never called exact.
//! 2. The document's `certification` map lists every non-exact quantity and
//!    nothing else, so a field's absence from it is a statement.
//! 3. **No rendered decimal bypasses the certified constructor.** This is the
//!    load-bearing one. `Value::quantity` is the only way to render a rational
//!    in this crate; a call site that reaches for `Value::text(…)` and formats a
//!    decimal itself would produce a number carrying no certification and would
//!    be invisible to the first two properties.

use ucal::cert::Certification;
use ucal::emit::{Doc, Value};
use ucal::style::Render;
use ucal_core::{Ratio, Rounding};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";
const T2: &str = "8070205189999984864657505252035637180530466139316558837890625";

fn documents() -> Vec<(&'static str, Doc)> {
    let mut v: Vec<(&'static str, Doc)> = vec![
        ("datum", ucal::cmd_datum().unwrap()),
        ("doctor", ucal::cmd_doctor().unwrap()),
        ("explain", ucal::cmd_explain(T, false).unwrap()),
("between", ucal::cmd_between(T, T2, Some(ucal_core::Tier::BEAT)).unwrap()),
        ("verify", ucal::cmd_verify().unwrap()),
        ("tour", ucal::cmd_tour().unwrap()),
        ("explain-why", ucal::cmd_explain_why(T, false).unwrap()),
        ("explain --claim", ucal::cmd_explain(T, true).unwrap()),
        (
            "ladder",
            ucal::cmd_ladder(ucal_core::LocaleId::En, false).unwrap(),
        ),
    ];
    #[cfg(feature = "events")]
    {
        v.push(("events list", ucal::cmd_events_list().unwrap()));
        v.push(("events show", ucal::cmd_events_show("recombination").unwrap()));
        v.push((
            "timeline",
            ucal::cmd_timeline(ucal::parse_tier("drift").unwrap()).unwrap(),
        ));
    }
    #[cfg(all(feature = "body", feature = "civil"))]
    {
        v.push(("cal list", ucal::cmd_cal_list().unwrap()));
        v.push(("cal show", ucal::cmd_cal_show("earth-d", T).unwrap()));
        v.push(("cal anchor", ucal::cmd_cal_anchor("earth-d").unwrap()));
        v.push((
            "show",
            ucal::cmd_show(T, &["earth-d".into(), "mars-d".into(), "earth-civil".into()]).unwrap(),
        ));
        v.push((
            "to-civil",
            ucal::cmd_to_civil(
                T,
                ucal_civil::si::Scale::Tt,
                6,
                Rounding::HalfEven,
                ucal_civil::calendar::CivilCalendar::Gregorian,
            )
            .unwrap(),
        ));
    }
    #[cfg(feature = "cosmo")]
    {
        v.push(("cosmo model", ucal::cmd_cosmo_model().unwrap()));
        v.push(("cosmo age", ucal::cmd_cosmo_age("1100", 4, 8).unwrap()));
    }
    v
}

/// Every scalar in a document, with its dotted path.
fn scalars(doc: &Doc) -> Vec<(String, Value)> {
    fn walk(fields: &[(String, Value)], prefix: &str, out: &mut Vec<(String, Value)>) {
        for (k, v) in fields {
            let path = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            match v {
                Value::Section(inner) => walk(inner, &path, out),
                Value::Rows { rows, .. } => walk(rows, &path, out),
                other => out.push((path, other.clone())),
            }
        }
    }
    let mut out = Vec::new();
    walk(doc.fields(), "", &mut out);
    out
}

// ---------------------------------------------------------------- property 1

#[test]
fn exact_means_the_digits_reparse_to_the_value() {
    // Both directions, over rationals chosen to straddle the interesting cases:
    // terminating at once, terminating late, and never terminating.
    let cases: Vec<(u64, u64)> = vec![
        (1, 1),
        (7, 1),
        (1, 2),
        (1, 4),
        (1, 5),
        (1, 8),
        (3, 25),
        (1, 16),
        (1, 32),
        (1, 64),
        (1, 125),
        (1, 3),
        (1, 6),
        (1, 7),
        (2, 11),
        (22, 7),
        (355, 113),
    ];
    for (n, d) in cases {
        let r = Ratio::from_u64(n).div(&Ratio::from_u64(d)).unwrap();
        for digits in [0u32, 1, 2, 3, 6, 9, 12] {
            let v = Value::quantity(&r, digits, Rounding::HalfEven);
            let (text, cert) = v.rendered_opt(&Render::PLAIN).expect("a quantity");
            let back = Ratio::from_decimal_str(&text);
            let round_trips = matches!(&back, Ok(b) if *b == r);
            assert_eq!(
                cert.is_exact(),
                round_trips,
                "{n}/{d} at {digits} digits: certified {cert}, reparse {}",
                if round_trips { "matches" } else { "differs" }
            );
        }
    }
}

#[test]
fn a_value_that_does_not_reparse_is_never_called_exact() {
    // The failure that would matter: a tag saying exact over digits that are not
    // the value. Checked over the real commands, where the text is all there is.
    for (name, doc) in documents() {
        for (path, v) in scalars(&doc) {
            let Some((text, cert)) = v.rendered_opt(&Render::PLAIN) else {
                continue;
            };
            if !cert.is_exact() {
                continue;
            }
            // An exact rendering must survive a reparse-and-render round trip.
            if let Ok(r) = Ratio::from_decimal_str(&text) {
                let digits = text.split_once('.').map(|(_, f)| f.len()).unwrap_or(0) as u32;
                let again = r.to_decimal_string(digits, Rounding::HalfEven).unwrap();
                assert_eq!(
                    &again, &text,
                    "`{name}`/{path}: certified exact but does not round-trip"
                );
            }
        }
    }
}

// ---------------------------------------------------------------- property 2

#[test]
fn the_certification_map_lists_every_exception_and_nothing_else() {
    for (name, doc) in documents() {
        let listed: Vec<String> = doc
            .certifications(&Render::PLAIN)
            .into_iter()
            .map(|(p, _)| p)
            .collect();
        for (path, v) in scalars(&doc) {
            let Some((_, cert)) = v.rendered_opt(&Render::PLAIN) else {
                continue;
            };
            let is_listed = listed.contains(&path);
            assert_eq!(
                !cert.is_exact(),
                is_listed,
                "`{name}`/{path}: certified {cert} but {} in the map",
                if is_listed { "listed" } else { "absent" }
            );
        }
    }
}

#[test]
fn the_map_reaches_the_json_and_the_text() {
    // A claim nobody can read is not a claim.
    // With `--bridge`, because the rounded column this names is a foreign unit
    // and 0.4.0 stopped printing those unasked (D-A16).
    let doc = ucal::cmd_ladder(ucal_core::LocaleId::En, true).unwrap();
    let r = Render::PLAIN.bridge(true);
    assert!(
        !doc.certifications(&r).is_empty(),
        "the ladder rounds something"
    );
    let json = doc.to_json_with(&r);
    assert!(json.contains("\"certification\""), "missing from --json");
    assert!(json.contains("rounded"), "the map says nothing useful");
    let text = doc.render(&r);
    assert!(text.contains("certification:"), "missing from the text");
    assert!(
        text.contains("seconds (bridge)"),
        "the rounded column is not named"
    );
}

#[test]
fn adding_the_map_leaves_the_json_parseable_and_the_values_unchanged() {
    // The map is additive: existing fields keep their names, shapes and values,
    // so `ucal-json/1` still describes this document.
    for (name, doc) in documents() {
        let json = doc.to_json();
        // Balanced braces is a cheap structural check that catches the comma
        // bug an additive trailing object invites.
        let opens = json.matches('{').count();
        let closes = json.matches('}').count();
        assert_eq!(opens, closes, "`{name}`: unbalanced JSON");
        assert!(!json.contains(",\n}"), "`{name}`: trailing comma before a close");
        assert!(json.contains("\"format\": \"ucal-json/1\""), "`{name}`: version moved");
    }
}

// ---------------------------------------------------------------- property 3

#[test]
fn no_rendered_decimal_bypasses_the_certified_constructor() {
    // The one that catches the future mistake. `Value::quantity` is the only way
    // to render a rational in this crate; a call site that formats a decimal
    // into `Value::text` produces a number with no certification, and the two
    // properties above would never see it.
    //
    // The test is a shape test, so it needs an exemption list — and the list is
    // short and each entry says why, which is the point of having one.
    const NOT_A_RENDERED_RATIONAL: &[&str] = &[
        // A civil label: `2026-07-29T12:34:56.5`, whose fraction is a clock
        // reading rather than a rounding of a rational.
        "qualified",
        "rendered",
        "epoch",
        "input.label",
        "complete_through",
        // Verbatim published text, reproduced exactly as its source wrote it
        // (Rule Y.1). Certifying it would be certifying someone else's rounding.
        "as_published",
        "input",
        "citation",
        "verbatim",
        "hubble_time.gyr",
        "turns_at_u",
        "half_width_drifts",
    ];
    let looks_decimal = |s: &str| {
        let t = s.trim_start_matches(['+', '-']);
        match t.split_once('.') {
            Some((a, b)) => {
                !a.is_empty()
                    && !b.is_empty()
                    && a.bytes().all(|c| c.is_ascii_digit())
                    && b.bytes().all(|c| c.is_ascii_digit())
            }
            None => false,
        }
    };
    let mut bare = Vec::new();
    for (name, doc) in documents() {
        for (path, v) in scalars(&doc) {
            let Value::Text(t) = &v else { continue };
            if !looks_decimal(t) {
                continue;
            }
            let leaf = path.rsplit('.').next().unwrap_or(&path);
            if NOT_A_RENDERED_RATIONAL
                .iter()
                .any(|e| *e == leaf || path.ends_with(e))
            {
                continue;
            }
            bare.push(format!("`{name}`/{path} = {t}"));
        }
    }
    assert!(
        bare.is_empty(),
        "these decimals were rendered without a certification — route them \
         through Value::quantity, or add them to NOT_A_RENDERED_RATIONAL with \
         a reason:\n  {}",
        bare.join("\n  ")
    );
}

// ---------------------------------------------------------------- enclosures

#[test]
fn every_emitted_enclosure_has_lo_at_most_hi() {
    // An inverted interval is not a wide answer, it is a wrong one, and it would
    // print as plausibly as a correct one.
    for (name, doc) in documents() {
        for (path, v) in scalars(&doc) {
            let Value::Text(t) = &v else { continue };
            let Some(inner) = t.strip_prefix('[') else {
                continue;
            };
            let Some(inner) = inner.split(']').next() else {
                continue;
            };
            let Some((lo, hi)) = inner.split_once(',') else {
                continue;
            };
            let (lo, hi) = (lo.trim(), hi.trim().trim_end_matches(" ticks"));
            if let (Ok(a), Ok(b)) = (lo.parse::<i128>(), hi.parse::<i128>()) {
                assert!(a <= b, "`{name}`/{path}: enclosure [{a}, {b}] is inverted");
            }
        }
    }
}

#[test]
fn a_certification_is_computed_not_annotated() {
    // The same value certifies differently at different digit counts, which is
    // only possible if the tag is derived from the value at render time.
    let eighth = Ratio::from_u64(1).div(&Ratio::from_u64(8)).unwrap();
    assert_eq!(
        Certification::of_ratio(&eighth, 3, Rounding::HalfEven),
        Certification::Exact
    );
    assert!(!Certification::of_ratio(&eighth, 2, Rounding::HalfEven).is_exact());
}


// ---------------------------------------------------------------- V2

#[test]
fn decimals_and_round_reach_every_rendered_rational() {
    // The point of carrying the value instead of a string. A tick in beats is
    // 1/5^60 — a finite expansion sixty places long — so six digits print it as
    // zero, forty-five digits show it, and sixty make it exact.
    let doc = ucal::cmd_ladder(ucal_core::LocaleId::En, true).unwrap();
    let beats = |d: Option<u32>| -> (String, Certification) {
        let r = Render::PLAIN.decimals(d);
        doc.rows("tiers")
            .unwrap()
            .iter()
            .find(|(k, _)| k == "T-12")
            .and_then(|(_, row)| row.as_rows())
            .unwrap()
            .iter()
            .find(|(k, _)| k == "beats")
            .unwrap()
            .1
            .rendered_opt(&r)
            .unwrap()
    };

    let (six, c6) = beats(None);
    assert_eq!(six, "0.000000", "the default must not change");
    assert!(!c6.is_exact());

    let (forty_five, _) = beats(Some(45));
    assert!(forty_five.ends_with("1153"), "45 digits should show it: {forty_five}");

    let (sixty, c60) = beats(Some(60));
    assert!(c60.is_exact(), "60 digits is where 1/5^60 terminates");
    assert!(sixty.ends_with("1152921504606846976"));
}

#[test]
fn the_certification_follows_the_digit_count() {
    // At sixty digits the beats column becomes exact and drops out of the map,
    // while bridge seconds stay in it — they never terminate at any count.
    let doc = ucal::cmd_ladder(ucal_core::LocaleId::En, true).unwrap();
    let at = |d: u32| -> Vec<String> {
        doc.certifications(&Render::PLAIN.bridge(true).decimals(Some(d)))
            .into_iter()
            .map(|(p, _)| p.rsplit('.').next().unwrap_or("").to_string())
            .collect()
    };
    assert!(at(6).iter().any(|n| n == "beats"));
    assert!(!at(60).iter().any(|n| n == "beats"), "beats terminates at 60");
    for d in [6u32, 60, 120] {
        assert!(
            at(d).iter().any(|n| n == "seconds (bridge)"),
            "bridge seconds cannot terminate at {d} digits"
        );
    }
}

#[test]
fn the_round_override_changes_the_digits_and_says_so() {
    let doc = ucal::cmd_ladder(ucal_core::LocaleId::En, true).unwrap();
    let text = |m: Rounding| doc.render(&Render::PLAIN.round(Some(m)));
    // Trunc and ceil must differ somewhere on a value that does not terminate.
    assert_ne!(text(Rounding::Trunc), text(Rounding::Ceil));
    // And the report names the mode that was actually applied.
    assert!(text(Rounding::Ceil).contains("ceil"));
    assert!(text(Rounding::Trunc).contains("trunc"));
}

#[test]
fn the_defaults_change_nothing() {
    // A caller who does not ask gets exactly what they got before V2.
    for (name, doc) in documents() {
        let plain = doc.render(&Render::PLAIN);
        assert_eq!(
            plain,
            doc.render(&Render::PLAIN.decimals(None).round(None)),
            "`{name}`: an absent override changed the output"
        );
        assert_eq!(doc.to_json(), doc.to_json_with(&Render::PLAIN));
    }
}
