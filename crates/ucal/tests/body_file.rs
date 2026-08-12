//! X1.3 / X1.4 — the body-file loader, and what it is held to.
//!
//! §15.1 requires a strict loader with body files versioned independently of
//! anchor files. [D-A20] records that `ucal-body` does not have one and why: the
//! data model is `&'static str` throughout, so a runtime loader must either leak
//! or change a published type, and the second is 2.0's.
//!
//! This one lives in the binary, where the leak is bounded by a process that
//! exits. What it must not do is become a second, laxer way of declaring a body:
//! Rule C's obligations are the whole reason the compiled-in data is trustworthy,
//! and a file that could omit them would be a hole in the middle of the argument.
//!
//! [D-A20]: https://github.com/vulogov/ucal/blob/main/spec/SPEC-DELTAS.md

use std::io::Write;
use ucal::body_file;
use ucal_core::backend::TickInt;
use ucal_core::Code;

/// Write a file and return its path, kept alive by the returned handle.
///
/// `label` must be unique per call site. The first version keyed the directory
/// on the process id alone, and every test in this binary shares one — so four
/// tests running in parallel wrote the same file and read each other's. Caught
/// immediately, and only because they were checking different things.
fn tmp(label: &str, contents: &str) -> (tempdir::Dir, std::path::PathBuf) {
    let dir = tempdir::Dir::new(label);
    let path = dir.path().join("body.hjson");
    let mut f = std::fs::File::create(&path).expect("create");
    f.write_all(contents.as_bytes()).expect("write");
    (dir, path)
}

/// A minimal temporary directory, removed on drop.
///
/// Hand-rolled rather than a dependency: this needs one directory in one test
/// file, and `cargo install ucal` should not grow a tree for it.
mod tempdir {
    pub struct Dir(std::path::PathBuf);
    impl Dir {
        pub fn new(label: &str) -> Dir {
            let mut p = std::env::temp_dir();
            // Process id *and* a per-call label: the id alone is shared by every
            // test in the binary, and they run in parallel. No entropy is used,
            // and none is needed — the labels are distinct by construction.
            p.push(format!("ucal-body-file-{}-{label}", std::process::id()));
            let _ = std::fs::create_dir_all(&p);
            Dir(p)
        }
        pub fn path(&self) -> &std::path::Path {
            &self.0
        }
    }
    impl Drop for Dir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
}

/// Europa, with the solar-day block left as a hole the tests fill.
const GOOD_EUROPA: &str = r#"
id: europa
primary: jupiter
rotation_period: {
  value: 3.551181
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
SOLAR_DAY
orbital_period: {
  value: 4332.589
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
"#;

const GOOD: &str = r#"
id: europa
primary: jupiter
rotation_period: {
  value: 3.551181
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
solar_day: {
  value: 3.552106
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
orbital_period: {
  value: 4332.589
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
"#;

/// A well-formed file produces a body.
#[test]
fn a_good_file_loads() {
    let (_d, p) = tmp("good", GOOD);
    let body = body_file::load(&p).expect("europa should load");
    assert_eq!(body.id(), "europa");
    assert_eq!(body.primary(), Some("jupiter"));
}

/// §15.1: an unknown key is `UCAL-E0012`.
///
/// The code was defined for this loader and, until it existed, had **no raiser
/// anywhere in the workspace** — which is how D-A20 came to be written. This is
/// the first caller it has ever had.
#[test]
fn an_unknown_key_is_e0012() {
    let bad = GOOD.replace("primary: jupiter", "primary: jupiter\nrotation_speed: 12");
    let (_d, p) = tmp("unknown-key", &bad);
    let e = body_file::load(&p).expect_err("an unknown key must be refused");
    assert_eq!(e.code, Code::E0012);
}

/// D-A22: a file that will not load is `UCAL-E0017`, not a locale-table failure.
///
/// Both conditions, because they were one code before and the context string is
/// what distinguishes them.
#[test]
fn an_unloadable_file_is_e0017() {
    let missing = std::path::Path::new("/nonexistent/ucal-no-such-body.hjson");
    let e = body_file::load(missing).expect_err("a missing file must be refused");
    assert_eq!(e.code, Code::E0017);

    let (_d, p) = tmp("not-hjson", "id: europa\n  {{{ this is not hjson");
    let e = body_file::load(&p).expect_err("a malformed file must be refused");
    assert_eq!(e.code, Code::E0017);
}

/// Rule C is not optional in a file.
///
/// The obligations are what make the compiled-in data trustworthy. A loader that
/// let a parameter arrive without a citation, a unit or a validity window would
/// be a second and laxer way of declaring a body, and the argument this project
/// makes would have a hole in the middle of it.
#[test]
fn a_parameter_without_its_obligations_is_refused() {
    for missing in ["citation", "unit", "valid_years", "value"] {
        // Drop one line from the *second* parameter block only, so the rest of
        // the file is fine and the refusal cannot be blamed on anything else.
        // HJSON runs an unquoted string to end of line, so the format is one key
        // per line and dropping a key is dropping a line.
        let mut seen = 0;
        let bad: String = GOOD
            .lines()
            .filter(|l| {
                if l.trim().starts_with(&format!("{missing}:")) {
                    seen += 1;
                    return seen != 2;
                }
                true
            })
            .collect::<Vec<_>>()
            .join("\n");
        let (_d, p) = tmp(&format!("missing-{missing}"), &bad);
        assert!(
            body_file::load(&p).is_err(),
            "a parameter missing `{missing}` was accepted"
        );
    }
}

/// Z1.1 — a derived parameter reproduces the exact value, not a rounded one.
///
/// This is the whole point of the form. The measured version of this file needs
/// twelve decimals written by hand to reach the same calendar; the derived
/// version states no figure at all and reaches it exactly, because the value is
/// never a decimal.
#[test]
fn a_derived_solar_day_reproduces_the_shipped_body() {
    let derived = GOOD_EUROPA.replace(
        "SOLAR_DAY",
        "solar_day: {\n  derived: synodic\n  citation: derived from the two figures cited here\n  valid_years: 10000\n}",
    );
    let (_d, p) = tmp("derived-synodic", &derived);
    let from_file = rule_of(&body_file::load(&p).expect("loads"));
    let shipped = rule_of(&ucal_body::data::europa());
    assert_eq!(
        from_file.chosen.value.denom().to_dec_string(),
        shipped.chosen.value.denom().to_dec_string(),
        "a derived solar day did not reproduce the compiled-in body"
    );
}

/// And it is exact, not merely close: one fewer decimal than the measured form
/// needs would already disagree.
#[test]
fn the_derived_form_beats_every_decimal_short_of_exact() {
    let derived = GOOD_EUROPA.replace(
        "SOLAR_DAY",
        "solar_day: {\n  derived: synodic\n  citation: derived\n  valid_years: 10000\n}",
    );
    let (_d1, p1) = tmp("derived-exact", &derived);
    let exact = body_file::load(&p1).expect("loads");

    let five = GOOD_EUROPA.replace(
        "SOLAR_DAY",
        "solar_day: {\n  value: 3.55409\n  unit: d\n  citation: rounded\n  valid_years: 10000\n}",
    );
    let (_d2, p2) = tmp("derived-five", &five);
    let rounded = body_file::load(&p2).expect("loads");

    assert_ne!(
        rule_of(&exact).chosen.value.denom().to_dec_string(),
        rule_of(&rounded).chosen.value.denom().to_dec_string(),
        "five decimals already reproduce the derivation, so the form saves nothing"
    );
}

/// A parameter is measured or derived, and never both or neither.
#[test]
fn a_parameter_cannot_be_measured_and_derived_at_once() {
    for (label, block) in [
        (
            "both",
            "solar_day: {\n  value: 3.554094\n  unit: d\n  derived: synodic\n  citation: c\n  valid_years: 10000\n}",
        ),
        ("neither", "solar_day: {\n  citation: c\n  valid_years: 10000\n}"),
        (
            "derived-with-unit",
            "solar_day: {\n  derived: synodic\n  unit: d\n  citation: c\n  valid_years: 10000\n}",
        ),
        (
            "unknown-relation",
            "solar_day: {\n  derived: sidereal\n  citation: c\n  valid_years: 10000\n}",
        ),
    ] {
        let bad = GOOD_EUROPA.replace("SOLAR_DAY", block);
        let (_d, p) = tmp(&format!("ambiguous-{label}"), &bad);
        assert!(
            body_file::load(&p).is_err(),
            "`{label}` was accepted as a parameter"
        );
    }
}

/// Z1.3 — a tidally locked body has no solar day, and the derivation says so.
///
/// The most likely habitable-zone case around an M dwarf, and the state of every
/// large moon in the outer solar system. Deriving the synodic day of a body
/// whose rotation equals its year divides by zero, and the honest answer is that
/// the star does not move in that sky.
#[test]
fn a_tidally_locked_body_is_told_it_has_no_day() {
    let locked = GOOD_EUROPA
        .replace("SOLAR_DAY", "solar_day: {\n  derived: synodic\n  citation: c\n  valid_years: 10000\n}")
        .replace("value: 4332.589", "value: 3.551181");
    let (_d, p) = tmp("locked", &locked);
    let e = body_file::load(&p).expect_err("a locked body has no solar day");
    let msg = e.to_string();
    assert!(msg.contains("tidally locked"), "{msg}");
    assert!(msg.contains("unbounded"), "{msg}");
}

/// Z1.3 — a whole number of days per year is an answer, not a bound failure.
///
/// Reported as `UCAL-E0061` — *no convergent meets the drift bound* — advising a
/// wider bound or a greater depth, when neither can help: there is no fractional
/// part to approximate.
#[test]
fn a_whole_number_of_days_per_year_is_not_a_bound_failure() {
    let whole = GOOD_EUROPA
        .replace(
            "SOLAR_DAY",
            "solar_day: {\n  value: 1\n  unit: d\n  citation: c\n  valid_years: 10000\n}",
        )
        .replace("value: 4332.589", "value: 4332");
    let (_d, p) = tmp("whole-days", &whole);
    let e = ucal::cmd_cal_derive(p.to_str().expect("utf-8")).expect_err("no intercalation");
    let msg = e.to_string();
    assert!(msg.contains("whole number of its solar days"), "{msg}");
    assert!(!msg.contains("widen the bound"), "{msg}");
}

/// A file cannot smuggle in a value the type system would refuse.
#[test]
fn a_malformed_value_is_refused() {
    for value in ["", "abc", "1.2.3", "-5", "1e6"] {
        let bad = GOOD.replace("value: 3.552106", &format!("value: \"{value}\""));
        let (_d, p) = tmp(&format!("bad-value-{}", value.len()), &bad);
        assert!(
            body_file::load(&p).is_err(),
            "`{value}` was accepted as a parameter value"
        );
    }
}

/// **The check that matters: a file reproduces a body that ships.**
///
/// Mars's parameters written as a file must derive the same leap rule as
/// `data::mars()` does compiled in. If they did not, the loader would be a
/// second implementation of the data model agreeing with the first only by
/// coincidence, and every calendar authored through it would be subtly
/// different from every calendar authored in Rust.
///
/// Mars and not Titan, for the reason the next test records.
#[test]
fn a_file_reproduces_a_body_that_ships() {
    let mars = r#"
id: mars
primary: sun
rotation_period: {
  value: 88642.6632
  unit: s
  citation: IAU WGCCRE
  valid_years: 1000
}
solar_day: {
  value: 88775.244
  unit: s
  citation: IAU WGCCRE
  valid_years: 1000
}
orbital_period: {
  value: 686.9726
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
"#;
    let (_d, p) = tmp("mars", mars);
    let from_file = body_file::load(&p).expect("mars should load");
    let shipped = ucal_body::data::mars();

    let a = rule_of(&from_file);
    let b = rule_of(&shipped);
    assert_eq!(
        (a.chosen.value.numer().to_dec_string(), a.chosen.value.denom().to_dec_string()),
        (b.chosen.value.numer().to_dec_string(), b.chosen.value.denom().to_dec_string()),
        "the file and the compiled-in body derive different leap rules"
    );
    assert_eq!(a.chosen.value.denom().to_dec_string(), "76", "Mars's rule is 45/76");
}

/// **A rounded input is a different calendar**, and the file format cannot hide it.
///
/// `data::titan()` does not state its solar day: it *derives* it exactly, as
/// `1/(1/P_rot − 1/P_year)`, because no source publishes one. A body file can
/// only state a measured figure, so writing Titan as a file means rounding that
/// derivation — to six decimals, `15.969088` against an exact `15.969087612…`.
///
/// A difference of 3.9 × 10⁻⁷ days. The continued fraction of days-per-year
/// diverges at the **fifth term** — `[1, 3, 35, 1, 1, 1, 5, 1]` becomes
/// `[1, 3, 35, 1, 106, 6, 3, 1]` — and the chosen convergent changes with it.
///
/// This is not a defect in the loader. It is Rule K working: an intercalation
/// rule is a continued fraction, continued fractions are violently sensitive to
/// their inputs, and a calendar derived from a rounded parameter is a different
/// calendar. Anyone authoring a body file is deciding their calendar's
/// intercalation with the last digit they write, and the test exists so that is
/// recorded rather than discovered.
#[test]
fn rounding_a_derived_parameter_changes_the_calendar() {
    let titan = r#"
id: titan
primary: saturn
rotation_period: {
  value: 15.945421
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
solar_day: {
  value: 15.969088
  unit: d
  citation: derived, then rounded to six decimals
  valid_years: 10000
}
orbital_period: {
  value: 10759.2058
  unit: d
  citation: NASA fact sheet
  valid_years: 10000
}
"#;
    let (_d, p) = tmp("titan-rounded", titan);
    let from_file = body_file::load(&p).expect("titan should load");
    let shipped = ucal_body::data::titan();

    let a = rule_of(&from_file);
    let b = rule_of(&shipped);
    assert_ne!(
        a.chosen.value.denom().to_dec_string(),
        b.chosen.value.denom().to_dec_string(),
        "if these agreed the sensitivity would not be worth recording"
    );
    // The shipped one, from the exact derivation.
    assert_eq!(b.chosen.value.denom().to_dec_string(), "117");
}

/// The leap rule for a body, by the one path everything here uses.
fn rule_of(b: &ucal_body::Body) -> ucal_body::LeapRule {
    ucal_body::derive_leap_rule(
        b.solar_day().value_at_epoch(),
        b.orbital_period().value_at_epoch(),
        ucal_body::DriftBound::DEFAULT,
        32,
    )
    .expect("a rule")
}

/// A body naming no satellite gets no cycle, and the command says so.
///
/// §15.3 forbids a fallback structure. The failure this guards against is a
/// loader that helpfully supplies a month because a calendar looks incomplete
/// without one — which is exactly the Earth-shaped assumption Rule K exists to
/// keep out.
#[test]
fn a_body_with_no_satellite_gets_no_month() {
    let (_d, p) = tmp("no-satellite", GOOD);
    let body = body_file::load(&p).expect("loads");
    assert!(body.satellites().is_empty());

    let doc = ucal::cmd_cal_derive(p.to_str().expect("utf-8")).expect("derives");
    let text = doc.to_text();
    assert!(text.contains("no grouping satellite"), "{text}");
    assert!(text.contains("not a gap"), "{text}");
}

/// The derived calendar has no anchor, and the output says why.
///
/// Phase is empirical (Rule J) and D5 established what establishing one costs.
/// A command that produced a calendar and left the reader to discover that its
/// dates are unavailable would be the reverse of what this project does.
#[test]
fn a_derived_calendar_states_that_it_has_no_phase() {
    let (_d, p) = tmp("no-phase", GOOD);
    let doc = ucal::cmd_cal_derive(p.to_str().expect("utf-8")).expect("derives");
    let text = doc.to_text();
    assert!(text.contains("Phase is empirical"), "{text}");
    assert!(text.contains("never derived"), "{text}");
}

/// The example file in the documentation derives the body it names.
///
/// It used to only have to *load*, and that was too weak a check. The file
/// stated a solar day of `3.552106` and cited the NASA fact sheet for it — a
/// figure that source does not publish, wrong in the third decimal — and it
/// loaded perfectly for a full release cycle. Y3 added `europa-d` to the
/// catalogue and the two could finally be compared: `202/279` against `1/24`.
///
/// So the check is now that the documented example and the compiled-in body
/// agree on the calendar. An example nobody can check is a claim with no
/// mechanism.
#[test]
fn the_documented_example_derives_the_body_it_names() {
    let body = body_file::load(&example_path()).expect("the documented example loads");
    let from_file = rule_of(&body);
    let shipped = rule_of(&ucal_body::data::europa());
    assert_eq!(
        (
            from_file.chosen.value.numer().to_dec_string(),
            from_file.chosen.value.denom().to_dec_string()
        ),
        (
            shipped.chosen.value.numer().to_dec_string(),
            shipped.chosen.value.denom().to_dec_string()
        ),
        "Documentation/examples/europa.hjson derives a different calendar from \
         the europa this program ships"
    );
}

/// The example's own table is checked against the example.
///
/// The file carries a table of what each precision yields and says six decimals
/// are the fewest that reach `1/24`. This substitutes the measured form at five
/// into the real file and requires the agreement to break, so the table cannot
/// rot into decoration.
///
/// An earlier version of this test trimmed to eight decimals and passed for the
/// wrong reason: it had read the *fourth* convergent from a table rather than
/// the convergent the derivation chooses, which for Europa is the second and is
/// reached long before the far terms start moving.
#[test]
fn the_examples_precision_table_is_true_of_the_example() {
    let text = std::fs::read_to_string(example_path()).expect("read");
    let block = text
        .split("solar_day: {")
        .nth(1)
        .and_then(|t| t.split_once("\n}"))
        .map(|(b, _)| format!("solar_day: {{{b}\n}}"))
        .expect("the example states a solar_day block");
    assert!(
        block.contains("derived: synodic"),
        "the example no longer uses the derived form this test substitutes for"
    );

    let five = text.replace(
        &block,
        "solar_day: {\n  value: 3.55409\n  unit: d\n  citation: rounded to five\n  valid_years: 10000\n}",
    );
    let (_d, p) = tmp("example-five", &five);
    let rounded = rule_of(&body_file::load(&p).expect("loads"));
    let shipped = rule_of(&ucal_body::data::europa());
    assert_ne!(
        rounded.chosen.value.denom().to_dec_string(),
        shipped.chosen.value.denom().to_dec_string(),
        "five decimals reproduce the shipped calendar, so the example's table is wrong"
    );
}

/// The example file in the documentation is a file this loader accepts.
#[test]
fn the_documented_example_loads() {
    let body = body_file::load(&example_path()).unwrap_or_else(|e| {
        panic!("the documented example does not load: {e}");
    });
    assert_eq!(body.id(), "europa");
}

/// The one documented example file.
fn example_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("Documentation/examples/europa.hjson")
}
