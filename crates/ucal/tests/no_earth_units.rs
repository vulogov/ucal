//! No Earth unit appears outside an Earth context unless it was asked for.
//!
//! A Julian year is 365.25 of Earth's rotations. An SI second is an Earth unit.
//! Using either to describe something that is not of Earth is the substitution
//! this program was written to object to, and it had crept into the program
//! itself: `cosmo age` reported its widths in Julian years and nothing else, for
//! epochs 13.4 billion years before Earth existed.
//!
//! Rule A.3 admits foreign units at three declared points and Rule A.5 makes
//! them informative. `--bridge` is the request; without it the output uses ticks
//! and the tier ladder, which are body-independent by construction.
//!
//! Two Earth contexts are exempt, and the list is short on purpose:
//!
//! - `to-civil` and `from-civil` *are* Earth calendar commands. A civil label is
//!   an Earth label; rendering it is the whole request.
//! - `ucal datum`'s provenance chain. §19.2 requires it, and it records where an
//!   Earth-sourced measurement came from — the point is precisely that Earth
//!   entered there and nowhere else (Rule Y).

use ucal::emit::{Doc, Value};
use ucal::style::{Render, Style};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";

/// Commands that are *not* about Earth. Their default output must contain no
/// foreign unit at all.
fn non_earth() -> Vec<(&'static str, Doc)> {
    let mut v: Vec<(&'static str, Doc)> = vec![
        ("explain", ucal::cmd_explain(T, false).unwrap()),
        (
            "ladder",
            ucal::cmd_ladder(ucal_core::LocaleId::En, false).unwrap(),
        ),
    ];
    #[cfg(feature = "events")]
    {
        v.push(("events show", ucal::cmd_events_show("recombination").unwrap()));
        v.push((
            "timeline",
            ucal::cmd_timeline(ucal::parse_tier("drift").unwrap()).unwrap(),
        ));
    }
    #[cfg(feature = "cosmo")]
    {
        v.push(("cosmo model", ucal::cmd_cosmo_model().unwrap()));
        v.push(("cosmo age", ucal::cmd_cosmo_age("1100", 4, 8).unwrap()));
    }
    v
}

/// Words that name a unit belonging to a body.
const EARTH_UNITS: &[&str] = &[
    "_years", "years_", "(bridge)", "gyr", "seconds_from_epoch",
];

/// Field paths in a document, with their leaf names.
fn paths(doc: &Doc, r: &Render) -> Vec<String> {
    fn walk(fields: &[(String, Value)], prefix: &str, r: &Render, out: &mut Vec<String>) {
        for (k, v) in fields {
            if matches!(v, Value::Bridge(_)) && !r.bridge {
                continue;
            }
            let path = if prefix.is_empty() {
                k.clone()
            } else {
                format!("{prefix}.{k}")
            };
            let inner = match v {
                Value::Bridge(b) => b.as_ref(),
                other => other,
            };
            match inner {
                Value::Section(f) | Value::Rows { rows: f, .. } => walk(f, &path, r, out),
                _ => out.push(path),
            }
        }
    }
    let mut out = Vec::new();
    walk(doc.fields(), "", r, &mut out);
    out
}

#[test]
fn no_foreign_unit_reaches_a_non_earth_command_by_default() {
    for (name, doc) in non_earth() {
        for p in paths(&doc, &Render::PLAIN) {
            let leaf = p.rsplit('.').next().unwrap_or(&p).to_ascii_lowercase();
            for u in EARTH_UNITS {
                assert!(
                    !leaf.contains(u),
                    "`{name}` prints `{p}` without being asked. A foreign unit \
                     belongs behind `--bridge` unless the command is about Earth."
                );
            }
        }
    }
}

#[test]
fn the_default_text_names_no_earth_unit_either() {
    // The field names are half of it; a value or a note that quietly gives a
    // figure in years is the other half.
    for (name, doc) in non_earth() {
        let text = doc.render(&Render::PLAIN).to_ascii_lowercase();
        for phrase in ["julian year", "in years", "gyr)"] {
            assert!(
                !text.contains(phrase),
                "`{name}` mentions `{phrase}` in its default output"
            );
        }
    }
}

#[test]
fn asking_for_the_bridge_brings_them_back() {
    // The conversion is available. It is not performed unasked, which is a
    // different thing from being unavailable.
    let with = Render::PLAIN.bridge(true);
    let mut found = 0;
    for (_, doc) in non_earth() {
        for p in paths(&doc, &with) {
            let leaf = p.rsplit('.').next().unwrap_or(&p).to_ascii_lowercase();
            if EARTH_UNITS.iter().any(|u| leaf.contains(u)) {
                found += 1;
            }
        }
    }
    assert!(
        found >= 5,
        "--bridge produced only {found} foreign-unit fields; the conversion \
         should still be reachable"
    );
}

#[test]
fn an_earth_command_keeps_its_earth_units_unconditionally() {
    // `to-civil` renders a civil label. Gating that behind `--bridge` would gate
    // the command's entire purpose.
    #[cfg(all(feature = "body", feature = "civil"))]
    {
        let doc = ucal::cmd_to_civil(
            T,
            ucal_civil::si::Scale::Tt,
            3,
            ucal_core::Rounding::HalfEven,
            ucal_civil::calendar::CivilCalendar::Gregorian,
        )
        .unwrap();
        let p = paths(&doc, &Render::PLAIN);
        for want in ["fields.year", "fields.month", "fields.day"] {
            assert!(
                p.iter().any(|x| x == want),
                "to-civil lost `{want}` from its default output"
            );
        }
    }
}

#[test]
fn the_datum_keeps_the_provenance_19_2_requires() {
    // §19.2 mandates the provenance chain and the rounding residual. They record
    // an Earth-sourced measurement, and that is the point: Earth entered there
    // and nowhere else (Rule Y). Gating them would hide the audit trail.
    let doc = ucal::cmd_datum().unwrap();
    let p = paths(&doc, &Render::PLAIN);
    assert!(p.iter().any(|x| x.starts_with("datum_provenance.chain")));
    assert!(p.iter().any(|x| x == "rounding.residual_rendered"));
}

#[test]
fn hiding_a_field_leaves_the_json_valid() {
    // An omitted field must not leave a dangling comma or an empty object where
    // a section used to be.
    for (name, doc) in non_earth() {
        for r in [Render::PLAIN, Render::PLAIN.bridge(true)] {
            let json = doc.to_json_with(&r);
            assert_eq!(
                json.matches('{').count(),
                json.matches('}').count(),
                "`{name}`: unbalanced JSON"
            );
            assert!(!json.contains(",\n}"), "`{name}`: dangling comma");
            assert!(!json.contains(",\n  }"), "`{name}`: dangling comma in a section");
        }
    }
}

#[test]
fn colour_and_width_do_not_resurrect_a_hidden_field() {
    for (name, doc) in non_earth() {
        let plain = doc.render(&Render::PLAIN);
        let fancy = doc.render(&Render::styled(Style::colored()).width(200));
        let stripped = ucal::style::strip_ansi(&fancy);
        for phrase in ["_years", "(bridge)"] {
            assert_eq!(
                plain.contains(phrase),
                stripped.contains(phrase),
                "`{name}`: `{phrase}` appears under one rendering and not another"
            );
        }
    }
}
