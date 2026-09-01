//! D3 — the CLI manual's *contents*, not only its surface.
//!
//! `xtask -- check-docs` already asserts that every command and global option in
//! `Documentation/CLI.md` exists and that nothing documented has been removed.
//! It says nothing about the field descriptions, which are the reason the file
//! exists: a reader who cannot guess what `remainder_ticks` means has nowhere
//! else to look.
//!
//! # Why this is a test and not a lint
//!
//! It needs the *real output* of every command, and here the commands are
//! ordinary functions. In `xtask` they would have to be shelled out to.
//!
//! # What it can and cannot check
//!
//! It checks that every field name the manual documents appears in some
//! command's output, and that every field a command emits is documented. Both
//! directions matter: the first catches a description of something that no
//! longer exists, the second catches a field nobody wrote down.
//!
//! **It cannot check that the prose is true.** Nothing mechanical can. That
//! limit is stated here rather than left for a reader to assume away — the file
//! is verified to be *complete and current*, not *correct*.
//!
//! The check was run once by hand at 0.3.0's close and found two errors on its
//! first run: `arithmetic_width` and `parameter_width` named Rust struct fields
//! that never appear in any output. A check that finds real defects the first
//! time it runs should not depend on someone thinking to run it.

use std::collections::BTreeSet;

use ucal::emit::{Doc, Value};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";
const T2: &str = "8070205189999984864657505252035637180530466139316558837890625";

/// Every command, so that every emitted field is reachable.
fn all_commands() -> Vec<(&'static str, Doc)> {
    let mut v: Vec<(&'static str, Doc)> = vec![
        ("datum", ucal::cmd_datum().unwrap()),
        // `now` reads the clock, so its *values* vary. Its field names do not,
        // and field names are what this file is about.
        (
            "now",
            ucal::cmd_now(ucal::parse_tier("T-12").unwrap(), ucal_core::codec::Form::HumanGroups)
                .unwrap(),
        ),
        ("doctor", ucal::cmd_doctor().unwrap()),
        ("explain", ucal::cmd_explain(T, true).unwrap()),
("between", ucal::cmd_between(T, T2, Some(ucal_core::Tier::BEAT)).unwrap()),
        ("verify", ucal::cmd_verify().unwrap()),
        ("tour", ucal::cmd_tour().unwrap()),
        ("explain-why", ucal::cmd_explain_why(T, false).unwrap()),
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
        v.push((
            "ruler",
            ucal::cmd_ruler(
                "0",
                "100000000000000000000000000000000000000000000",
                ucal::parse_tier("sweep").unwrap(),
            )
            .unwrap(),
        ));
    }
    #[cfg(all(feature = "body", feature = "civil"))]
    {
        v.push(("cal list", ucal::cmd_cal_list().unwrap()));
        v.push(("cal show", ucal::cmd_cal_show("earth-d", T).unwrap()));
        v.push(("cal anchor", ucal::cmd_cal_anchor("earth-d").unwrap()));
        v.push(("cal derive", ucal::cmd_cal_derive(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Documentation/examples/europa.hjson")).unwrap()));
        // Both halves: the anchor rows only appear when a pair is given, and the
        // manual documents them.
        v.push(("cal validate", ucal::cmd_cal_validate(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../Documentation/examples/earth.hjson"),
            Some(concat!(env!("CARGO_MANIFEST_DIR"), "/../../Documentation/examples/earth-anchor.hjson")),
        ).unwrap()));
        // A1 — the JD converters emit documents like any other command, and
        // R3's check caught their absence from this list the day they were
        // written, which is the answer to whether it was worth building.
        v.push(("from jd", ucal::cmd_from_jd("2451545.0", "tt", false).unwrap()));
        v.push(("from jd", ucal::cmd_from_jd("2460000.5", "tdb", false).unwrap()));
        v.push(("to jd", ucal::cmd_to_jd(T, "tt", false, 6).unwrap()));
        // And with `--mjd`, because the field is named for the flag: a list that
        // only ever passes `false` documents a field no command emits, which is
        // what `manual_fields` reported the first time this was added.
        v.push(("to jd", ucal::cmd_to_jd(T, "tt", true, 6).unwrap()));
        v.push(("cal from", ucal::cmd_cal_from("mars-d", "82-83").unwrap()));
        v.push((
            "add",
            ucal::cmd_add(T, 1, &ucal::Stride::calendar("mars-d").unwrap()).unwrap(),
        ));
        // `add --in` emits fields `add --step` does not. This list and
        // `json_surface`'s are two hand-maintained enumerations of the same
        // commands, and an invocation added to one is not added to the other:
        // the manual check is what noticed `days_moved` was documented and
        // unreachable here.
        v.push(("add --in", ucal::cmd_add_in(T, 1, "mars-d").unwrap()));
        v.push(("cal validate anchor", ucal::cmd_cal_validate(
            concat!(env!("CARGO_MANIFEST_DIR"), "/../../Documentation/examples/earth-anchor.hjson"),
            None,
        ).unwrap()));
        v.push((
            "show",
            ucal::cmd_show(
                T,
                &["earth-d".into(), "mars-d".into(), "titan-d".into(), "earth-civil".into()],
            )
            .unwrap(),
        ));
        v.push((
            "to-civil",
            ucal::cmd_to_civil(
                T,
                ucal_civil::si::Scale::Tt,
                3,
                ucal_core::Rounding::HalfEven,
                ucal_civil::calendar::CivilCalendar::Gregorian,
            )
            .unwrap(),
        ));
        v.push((
            "from-civil",
            ucal::cmd_from_civil(
                "2026-08-04",
                ucal_civil::si::Scale::Tt,
                ucal_civil::calendar::CivilCalendar::Gregorian,
            )
            .unwrap(),
        ));
    }
    // G8 — the face is a document now, so the manual's account of its fields is
    // held to the same standard as every other command's.
    #[cfg(feature = "tui")]
    {
        let f = ucal::wallclock::Face::at(
            ucal::parse_instant(T).unwrap().0,
            ucal_core::LocaleId::En,
            Some("earth-d"),
        )
        .unwrap();
        v.push(("wallclock", ucal::cmd_wallclock_json(&f, "plain").unwrap()));
        // And one with the optional keys present: `dials` and `since` are
        // emitted only when asked for, so a face without them documents fields
        // no command emitted.
        let d = ucal::wallclock::Dials::new(ucal_core::LocaleId::En)
            .unwrap()
            .with_clock_local(&["earth-d".to_string()])
            .with_since(ucal::parse_instant("0").unwrap().0, "the datum");
        let g = ucal::wallclock::Face::of(ucal::parse_instant(T).unwrap().0, &d).unwrap();
        v.push(("wallclock-full", ucal::cmd_wallclock_json(&g, "plain").unwrap()));
    }
    #[cfg(feature = "civil")]
    {
        // E2 — both kinds of answer: an exact unit and the bracketed one.
        v.push(("lighttime", ucal::cmd_lighttime("1", "ly", 9).unwrap()));
        v.push(("lighttime", ucal::cmd_lighttime("1", "pc", 9).unwrap()));
    }
    #[cfg(all(feature = "events", feature = "civil"))]
    {
        // B — an ephemeris file, and the certified dilation beside it. The path
        // is relative to this crate, the way the body-file invocations are.
        const EPH: &str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../Documentation/examples/ephemeris.hjson"
        );
        v.push(("ephem show", ucal::cmd_ephem_show(EPH).unwrap()));
        v.push(("ephem at", ucal::cmd_ephem_at(EPH, 5000, 3).unwrap()));
        v.push(("ephem next", ucal::cmd_ephem_next(EPH, T, 3, 2).unwrap()));
        // O1 — residuals read observations from a **committed** fixture.
        //
        // The first version wrote one into `std::env::temp_dir()` under a fixed
        // name, and this file and `manual_fields.rs` both did it — two test
        // binaries, one path, run concurrently. One truncated the file while the
        // other read it, which produced a residuals run that had read nothing.
        // It passed locally every time and failed in CI on the cut commit.
        const OBS: &str = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../Documentation/examples/observations.txt"
        );
        v.push((
            "ephem residuals",
            ucal::cmd_ephem_residuals(EPH, OBS, 1).unwrap(),
        ));
        // T2 — both verdicts, because `consistent` is a bool and a list that
        // only ever produced `true` would document a field with one value.
        v.push((
            "ephem validate",
            ucal::cmd_ephem_validate(EPH, 5000).unwrap(),
        ));
        v.push((
            "ephem validate",
            ucal::cmd_ephem_validate(EPH, 1).unwrap(),
        ));
    }
    #[cfg(feature = "cosmo")]
    {
        v.push(("dilate", ucal::cmd_dilate("0.35", 40, 18, false).unwrap()));
        v.push(("dilate", ucal::cmd_dilate("0.35", 40, 18, true).unwrap()));
        v.push(("cosmo model", ucal::cmd_cosmo_model().unwrap()));
        v.push(("cosmo age", ucal::cmd_cosmo_age("1100", 4, 8).unwrap()));
        v.push((
            "cosmo z",
            ucal::cmd_cosmo_z(T, 1_000_000, 4, 8).unwrap(),
        ));
        v.push((
            "cosmo age --audit",
            ucal::cmd_cosmo_age_audited("1090..1110", 4, 8, true).unwrap(),
        ));
    }
    v
}

/// Every field name a command emits, with `--bridge` on so the foreign-unit
/// fields are reachable too.
fn emitted() -> BTreeSet<String> {
    /// `keyed` distinguishes a *schema* key from a *data* key.
    ///
    /// A section's keys are field names — `enclosure.lo_ticks` is a field. A
    /// table's outer keys are row identifiers: `T5`, `earth-d`, `recombination`,
    /// `0`. Documenting those would mean documenting the catalogue, and a new
    /// event would fail the manual.
    fn walk(fields: &[(String, Value)], keyed: bool, out: &mut BTreeSet<String>) {
        for (k, v) in fields {
            if keyed {
                out.insert(k.clone());
            }
            // Sections whose keys come from *data* rather than from a schema:
            // the audit's prose step labels, the model's parameter names, and
            // `explain`'s tier decomposition, whose keys are `T5 deep` and so
            // on. Documenting these would mean documenting the catalogue, and a
            // new parameter or a renamed tier would fail the manual.
            let inner = match v {
                Value::Bridge(b) => b.as_ref(),
                other => other,
            };
            // Sections whose keys come from *data* rather than a schema: the
            // audit's prose step labels, the model's parameter names, and
            // `explain`'s tier decomposition, whose keys are `T5 deep` and so
            // on. Documenting these would mean documenting the catalogue.
            //
            // `ladder`'s `tiers` is a *table*, so its row keys are already
            // skipped and its row fields — exponent, name, beats — are real
            // schema and stay.
            if matches!(k.as_str(), "audit" | "as_published")
                || (k == "tiers" && matches!(inner, Value::Section(_)))
            {
                continue;
            }
            match inner {
                Value::Section(f) => walk(f, true, out),
                Value::Rows { rows, .. } => walk(rows, false, out),
                _ => {}
            }
        }
    }
    let mut out = BTreeSet::new();
    for (_, doc) in all_commands() {
        walk(doc.fields(), true, &mut out);
    }
    out
}

/// Field names the manual documents, from its markdown tables.
///
/// A row's first cell is a backticked name. Options start with `-`, prose rows
/// carry spaces, and both are skipped — this is about *fields*.
fn documented() -> BTreeSet<String> {
    let md = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .expect("workspace root")
            .join("Documentation/CLI.md"),
    )
    .expect("Documentation/CLI.md");

    let mut out = BTreeSet::new();
    for line in md.lines() {
        let l = line.trim();
        if !l.starts_with("| `") {
            continue;
        }
        let Some(rest) = l.strip_prefix("| ") else { continue };
        // A cell may hold several names: `widths.arithmetic_ticks` / `_drifts`
        // / `_years`. Take every backticked run in the first cell, not only the
        // first — taking only the first is why `arithmetic_drifts` looked
        // undocumented when it had been documented all along.
        let cell = rest.split(" | ").next().unwrap_or(rest);
        for name in cell.split('`').skip(1).step_by(2) {
            // `[]` marks a list in the manual and is not part of the name.
            let name = name.trim().trim_end_matches("[]");
            // Spaces are allowed: `seconds (bridge)` and `T4s since the datum`
            // are real field names. Skipping anything with a space — the first
            // draft did — silently dropped exactly those.
            if name.starts_with('-') || name.is_empty() {
                continue;
            }
            // Every segment of a documented path is documented: `enclosure` is
            // a field in its own right and `enclosure.lo_ticks` documents both.
            // A placeholder or a wildcard is not a field.
            for seg in name.split('.') {
                if seg.starts_with('<')
                    || seg.is_empty()
                    || seg.contains('*')
                    || seg.contains('<')
                    || seg.starts_with('_')
                    || seg.chars().all(|c| c.is_ascii_digit())
                {
                    continue;
                }
                out.insert(seg.to_string());
            }
        }
    }
    out
}

/// Names that are structural rather than fields a command emits.
const NOT_A_FIELD: &[&str] = &[
    // The recurring-fields section documents these as concepts.
    "ticks",
    "window",
    "kind",
    "citation",
    "notes",
    "precision",
    // Shape placeholders in path examples.
    "T",
    "id",
    // Command names, which appear in the contents table.
    "show",
    "now",
    "datum",
    "explain",
    "ladder",
    "cal",
    "events",
    "timeline",
    "ruler",
    "cosmo",
    "doctor",
];

/// Fields only a `tui` build emits.
///
/// The manual documents `ucal --json wallclock`, and this suite also runs under
/// the default features where `tui` is absent and that command does not exist —
/// so its fields would read as *documented and never emitted*. The same
/// reasoning as `json_surface`'s presence filter, and the same mitigation: CI
/// runs this suite under `--features full` too, where nothing is skipped.
#[cfg(not(feature = "tui"))]
const TUI_ONLY: &[&str] = &[
    "hands", "hero", "dials", "since", "index", "per_mille", "origin",
    "counting_down", "drums", "theme", "position", "through_day_percent",
];
#[cfg(feature = "tui")]
const TUI_ONLY: &[&str] = &[];

#[test]
fn every_documented_field_is_emitted_by_some_command() {
    // Catches a description of something that no longer exists — which is how
    // `arithmetic_width` and `parameter_width` survived in the manual naming
    // Rust struct fields that never reach any output.
    let emitted = emitted();
    let missing: Vec<String> = documented()
        .into_iter()
        .filter(|d| !emitted.contains(d))
        .filter(|d| !NOT_A_FIELD.contains(&d.as_str()))
        .filter(|d| !TUI_ONLY.contains(&d.as_str()))
        .collect();
    assert!(
        missing.is_empty(),
        "Documentation/CLI.md documents fields no command emits:\n  {}\n\
         Either the field was removed and the manual is stale, or the name is \
         wrong.",
        missing.join("\n  ")
    );
}

#[test]
fn every_emitted_field_is_documented() {
    // The other direction: a field nobody wrote down. A reader meeting it in
    // output has nowhere to look, which is the whole reason the manual exists.
    let documented = documented();
    let missing: Vec<String> = emitted()
        .into_iter()
        .filter(|e| !documented.contains(e))
        .collect();
    assert!(
        missing.is_empty(),
        "these fields reach the output and are undocumented:\n  {}\n\
         Add them to Documentation/CLI.md.",
        missing.join("\n  ")
    );
}

#[test]
fn the_check_says_what_it_cannot_check() {
    // Stated as a test so it is not quietly dropped. This file verifies that the
    // manual is complete and current. Whether a description is *true* is not
    // mechanically checkable, and claiming otherwise would be the overclaim the
    // whole project is about.
    let md = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|p| p.parent())
            .unwrap()
            .join("Documentation/CLI.md"),
    )
    .unwrap();
    assert!(
        md.contains("complete and current") || md.contains("not that the prose is true"),
        "Documentation/CLI.md should say what its checks do and do not cover"
    );
}
