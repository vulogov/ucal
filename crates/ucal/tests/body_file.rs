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

/// B1 — the leak is bounded by distinct strings, not by loads.
///
/// §15.1's loader must produce `&'static str` from owned data, so it leaks;
/// [D-A20] records that as the reason the loader is in the binary rather than in
/// `ucal-body`. The objection is specific: *"a caller loading in a loop leaks
/// without bound"*.
///
/// Interning answers that objection and not the whole of D-A20. This test is the
/// part that can be asserted: the same string, loaded twice, is the **same
/// pointer** — so a thousand loads of one file cost what one load costs.
///
/// What it cannot assert is that nothing leaks. A thousand *different* files
/// still accumulate, and that is why the delta stays `UNIMPLEMENTED`.
///
/// [D-A20]: https://github.com/vulogov/ucal/blob/main/spec/SPEC-DELTAS.md
#[test]
fn loading_the_same_file_twice_interns_rather_than_leaking_twice() {
    let (_d, p) = tmp("intern", GOOD);

    let first = body_file::load(&p).expect("loads");
    let second = body_file::load(&p).expect("loads again");

    // Same id, and the same allocation behind it.
    assert_eq!(first.id(), second.id());
    assert!(
        std::ptr::eq(first.id().as_ptr(), second.id().as_ptr()),
        "the second load allocated a second copy of an identical string"
    );

    // And the citations, which are the strings a file carries most of.
    let a = first.rotation_period().citation().source;
    let b = second.rotation_period().citation().source;
    assert!(
        std::ptr::eq(a.as_ptr(), b.as_ptr()),
        "an identical citation was leaked twice"
    );
}

/// Distinct strings still get distinct allocations, so the pool is a pool and
/// not a single slot.
#[test]
fn distinct_strings_are_not_conflated() {
    let (_d1, p1) = tmp("intern-a", GOOD);
    let other = GOOD.replace("id: europa", "id: europa-variant");
    let (_d2, p2) = tmp("intern-b", &other);

    let a = body_file::load(&p1).expect("loads");
    let b = body_file::load(&p2).expect("loads");
    assert_ne!(a.id(), b.id());
    assert!(!std::ptr::eq(a.id().as_ptr(), b.id().as_ptr()));
}

/// F5 — hours and minutes, so a file can quote its source verbatim.
///
/// **The check that matters.** `data::jupiter` states its rotation in seconds
/// and converts in a comment — `9.9250 h x 3600 = 35 730 s, exact` — because the
/// fact sheet publishes hours and the format had no way to say so. A file
/// quoting the fact sheet as printed must derive the same calendar as the
/// hand-converted body, or the new units are a second and laxer way of stating
/// a parameter.
#[test]
fn a_file_quoting_hours_derives_what_the_shipped_body_does() {
    let jupiter = r#"
id: jupiter
primary: sun
rotation_period: {
  value: 9.9250
  unit: h
  citation: NASA Planetary Fact Sheets
  valid_years: 1000
}
solar_day: {
  value: 9.9259
  unit: h
  citation: NASA Planetary Fact Sheets
  valid_years: 1000
}
orbital_period: {
  value: 4332.589
  unit: d
  citation: NASA Planetary Fact Sheets
  valid_years: 10000
}
"#;
    let (_d, p) = tmp("jupiter-hours", jupiter);
    let from_file = rule_of(&body_file::load(&p).expect("jupiter in hours loads"));
    let shipped = rule_of(&ucal_body::data::jupiter());
    assert_eq!(
        (
            from_file.chosen.value.numer().to_dec_string(),
            from_file.chosen.value.denom().to_dec_string()
        ),
        (
            shipped.chosen.value.numer().to_dec_string(),
            shipped.chosen.value.denom().to_dec_string()
        ),
        "hours and the hand conversion to seconds disagree"
    );
    assert_eq!(shipped.chosen.value.denom().to_dec_string(), "81");
}

/// Minutes work too, and the conversion is exact.
///
/// 60 and 3600 are exact multiples of the second, which is the condition Z1.2
/// set for admitting a unit at all: one that was not would put a rounding inside
/// the conversion, and that is a different decision from this one.
#[test]
fn a_minute_is_sixty_seconds_exactly() {
    let a = GOOD_EUROPA.replace(
        "SOLAR_DAY",
        "solar_day: {\n  value: 120\n  unit: min\n  citation: c\n  valid_years: 10000\n}",
    );
    let b = GOOD_EUROPA.replace(
        "SOLAR_DAY",
        "solar_day: {\n  value: 7200\n  unit: s\n  citation: c\n  valid_years: 10000\n}",
    );
    let (_d1, p1) = tmp("minutes", &a);
    let (_d2, p2) = tmp("seconds", &b);
    let m = body_file::load(&p1).expect("minutes load");
    let s = body_file::load(&p2).expect("seconds load");
    assert_eq!(
        m.solar_day().value_at_epoch().cmp_exact(s.solar_day().value_at_epoch()),
        core::cmp::Ordering::Equal,
        "120 min is not 7200 s"
    );
}

/// A unit the format does not accept is still refused, and says what it takes.
#[test]
fn an_unknown_unit_is_still_refused() {
    let bad = GOOD.replace("unit: d", "unit: parsec");
    let (_d, p) = tmp("bad-unit-f5", &bad);
    let e = body_file::load(&p).expect_err("parsec is not a duration");
    assert_eq!(e.code, Code::E0018);
    assert!(e.to_string().contains("`h`"), "{e}");
}

// ---- F4: `ucal cal validate` -------------------------------------------

/// The finding this command exists for: **the file is fine and the calendar is
/// not**.
///
/// `cal derive` on a body whose year is a whole number of its solar days returns
/// `UCAL-E0060`, and an author reading a red exit code cannot tell whether their
/// file is malformed or their body simply has no fractional day to distribute.
/// `validate` answers both questions separately, and this is the case where the
/// two answers differ.
#[test]
fn validate_separates_a_bad_file_from_a_body_with_no_calendar() {
    let whole = GOOD_EUROPA
        .replace(
            "SOLAR_DAY",
            "solar_day: {\n  value: 1\n  unit: d\n  citation: c\n  valid_years: 10000\n}",
        )
        .replace("value: 4332.589", "value: 4332");
    let (_d, p) = tmp("validate-whole-days", &whole);

    // `cal derive` refuses outright.
    assert!(ucal::cmd_cal_derive(p.to_str().expect("utf-8")).is_err());

    // `validate` succeeds, and says both things.
    let doc = ucal::cmd_cal_validate(p.to_str().expect("utf-8"), None).expect("a report");
    assert!(
        verdict(&doc, "loads").starts_with("ok"),
        "the file did not load: {}",
        verdict(&doc, "loads")
    );
    let why = verdict(&doc, "intercalation");
    assert!(
        why.contains("whole number of its solar days"),
        "the wrong reason: {why}"
    );
    assert!(
        why.contains("not a defect in what declares it"),
        "the two answers were not separated: {why}"
    );
    // And not the bound-failure advice, which cannot help here.
    assert!(!why.contains("widen the bound"), "{why}");
}

/// One check's verdict, read from the report rather than from its rendering.
///
/// `to_text` word-wraps, so a `contains` over the rendered page fails on any
/// phrase long enough to be worth asserting.
fn probe(doc: &ucal::emit::Doc, parameter: &str) -> String {
    let Some(ucal::emit::Value::Section(rows)) = doc.get("checks") else {
        panic!("no checks section");
    };
    let Some((_, ucal::emit::Value::Section(probes))) =
        rows.iter().find(|(k, _)| k == "precision")
    else {
        panic!("no precision probes");
    };
    probes
        .iter()
        .find(|(k, _)| k == parameter)
        .map(|(_, v)| v.rendered_text())
        .unwrap_or_else(|| panic!("`{parameter}` was not probed"))
}

fn verdict(doc: &ucal::emit::Doc, name: &str) -> String {
    let Some(ucal::emit::Value::Section(rows)) = doc.get("checks") else {
        panic!("no checks section");
    };
    rows.iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v.rendered_text())
        .unwrap_or_else(|| panic!("no check named `{name}`"))
}

/// A malformed file still fails, and with the loader's own diagnosis.
///
/// A validator that reported "does not load" and nothing else would be worse
/// than the error it replaced.
#[test]
fn validate_still_fails_on_a_malformed_file() {
    let bad = GOOD.replace("unit: d", "unit: parsec");
    let (_d, p) = tmp("validate-bad-unit", &bad);
    let e = ucal::cmd_cal_validate(p.to_str().expect("utf-8"), None).expect_err("parsec");
    assert_eq!(e.code, Code::E0018);
}

/// An anchor file handed to the positional argument is recognised as one.
///
/// The body loader is strict, so an anchor file fed to it reports `UCAL-E0012`,
/// *unknown key* — which is true, unhelpful, and about a file that is perfectly
/// valid. The second loader is tried before that error is reported.
#[test]
fn validate_names_the_kind_of_file_it_was_given() {
    let doc = ucal::cmd_cal_validate(&anchor_example(), None).expect("an anchor report");
    let text = doc.to_text();
    assert!(text.contains("anchor file"), "{text}");
    assert!(!text.contains("unknown key"), "{text}");
    assert_eq!(verdict(&doc, "calendar"), "`earth-d` ");
}

/// The precision probe is a measurement and reports both outcomes.
///
/// Every release note this project has published carries *a rounded parameter is
/// a different calendar*, and until now it was unmeasurable. Europa's orbital
/// period is the case that proves the probe is not a rubber stamp: one unit in
/// its last published place moves the rule off `1/24`.
#[test]
fn the_precision_probe_finds_a_parameter_its_last_digit_decides() {
    let doc = ucal::cmd_cal_validate(&body_example(), None).expect("a report");
    let v = probe(&doc, "orbital_period");
    assert!(v.starts_with("sensitive"), "{v}");
    assert!(v.contains("gives 1/24"), "{v}");
}

/// And it reports *stable* where the figure is precise enough, which is the
/// half that makes the other half mean something.
#[test]
fn the_precision_probe_is_not_only_ever_alarmed() {
    // Europa's rotation period is not probed — it does not feed the
    // intercalation — so the stable case needs a body whose two feeding
    // parameters are stated at different precisions. Mars is one.
    let mars = r#"
id: mars
primary: sun
rotation_period: {
  value: 88642.663
  unit: s
  citation: c
  valid_years: 10000
}
solar_day: {
  value: 88775.244
  unit: s
  citation: c
  valid_years: 10000
}
orbital_period: {
  value: 59355036.0
  unit: s
  citation: c
  valid_years: 10000
}
"#;
    let (_d, p) = tmp("validate-stable", mars);
    let doc = ucal::cmd_cal_validate(p.to_str().expect("utf-8"), None).expect("a report");
    let v = probe(&doc, "orbital_period");
    assert!(v.starts_with("stable"), "{v}");
}

/// A body file and an anchor file that are not a pair are told so.
#[test]
fn validate_checks_the_pair() {
    let doc = ucal::cmd_cal_validate(&body_example(), Some(&anchor_example())).expect("a report");
    let v = verdict(&doc, "anchor:names");
    assert!(v.contains("not a pair"), "europa and earth's anchor are not a pair: {v}");
}

fn body_example() -> String {
    format!(
        "{}/../../Documentation/examples/europa.hjson",
        env!("CARGO_MANIFEST_DIR")
    )
}

fn anchor_example() -> String {
    format!(
        "{}/../../Documentation/examples/earth-anchor.hjson",
        env!("CARGO_MANIFEST_DIR")
    )
}

// ---- G5: the probe, turned on this project's own data -------------------

/// `cal validate` takes a shipped calendar id, not only a file.
///
/// F4 built the probe and pointed it only at files somebody else wrote, which
/// left this project's own fifteen calendars outside the one check built to
/// measure how fragile a calendar's parameters are.
#[test]
fn validate_takes_a_shipped_calendar_id() {
    let doc = ucal::cmd_cal_validate("earth-d", None).expect("a report");
    assert_eq!(verdict(&doc, "loads").split(' ').next(), Some("compiled"));
    assert!(verdict(&doc, "intercalation").starts_with("31/128"), "{doc:?}");
    // And the probe runs on it, which is the point.
    assert!(probe(&doc, "solar_day").starts_with("sensitive"));
    assert!(probe(&doc, "orbital_period").starts_with("stable"));
}

/// A path wins over an id.
///
/// Both are accepted in one argument, so the tie has to be broken somewhere and
/// stated. A caller who names a file that exists means that file.
#[test]
fn a_file_that_exists_wins_over_a_calendar_id() {
    let (_d, p) = tmp("earth-d-shadow", &GOOD);
    let named = p.to_str().expect("utf-8");
    let doc = ucal::cmd_cal_validate(named, None).expect("a report");
    assert!(
        verdict(&doc, "loads").starts_with("ok"),
        "the file was not read: {}",
        verdict(&doc, "loads")
    );
}

/// **The G5 measurement, asserted so a data change cannot move it silently.**
///
/// Fifteen calendars rest on nineteen distinct published figures, and fourteen
/// of those decide their calendar's leap rule outright. That is not a defect: a
/// leap rule is a convergent of a continued fraction and continued fractions are
/// violently sensitive to their inputs, which `CLI.md` has said in words since
/// 1.4.0. This is that sentence measured.
///
/// The numbers are asserted rather than merely printed for the same reason W4's
/// were: a finding nobody can notice changing is a finding that will change.
#[test]
fn the_shipped_data_has_the_measured_fragility() {
    let doc = ucal::cmd_cal_validate_all().expect("a survey");
    let n = |k: &str| -> usize {
        let Some(ucal::emit::Value::Section(rows)) = doc.get("figures") else {
            panic!("no figures section");
        };
        rows.iter()
            .find(|(name, _)| name == k)
            .map(|(_, v)| v.rendered_text().trim().parse::<usize>().expect("a number"))
            .unwrap_or_else(|| panic!("no `{k}`"))
    };

    // Fifteen calendars, two intercalation parameters each.
    assert_eq!(n("parameters_probed") + n("parameters_derived"), 30);
    // Six solar days are `derived:` — the tidally locked moons, which have no
    // published figure to have a last digit.
    assert_eq!(n("parameters_derived"), 6);
    // The parts must sum to the whole, which is the arithmetic the first
    // version of this got wrong: it counted parameter slots and reported
    // nineteen sensitive out of nineteen distinct figures.
    assert_eq!(
        n("distinct_sensitive") + n("distinct_stable"),
        n("distinct_figures"),
        "the sensitive and stable counts do not partition the distinct figures"
    );
    assert_eq!(n("distinct_figures"), 19);
    assert_eq!(n("distinct_sensitive"), 14);
}

/// **One published figure decides five calendars.**
///
/// A satellite's year is its primary's orbit, so Jupiter's `4332.589 d` is the
/// orbital period of `jupiter-d`, `io-d`, `europa-d`, `ganymede-d` and
/// `callisto-d` alike — and its last digit decides all five leap rules. A count
/// of sensitive *parameters* hides that completely; it is the reason this survey
/// reports distinct figures and who rests on each.
#[test]
fn one_figure_carries_five_calendars() {
    let doc = ucal::cmd_cal_validate_all().expect("a survey");
    // `Rows` and not `Section`: keyed by a published figure, which is data.
    let Some(ucal::emit::Value::Rows { rows, .. }) = doc.get("carried_by_more_than_one")
    else {
        panic!("no shared-figure rows");
    };
    let jovian = rows
        .iter()
        .find(|(fig, _)| fig.starts_with("4332.589"))
        .map(|(_, v)| v.rendered_text())
        .expect("Jupiter's year is not reported as shared");
    for who in ["jupiter-d", "io-d", "europa-d", "ganymede-d", "callisto-d"] {
        assert!(jovian.contains(who), "{who} is missing: {jovian}");
    }
    assert!(jovian.contains('5'), "{jovian}");
}

// ---- G4: the figure that decides the month -----------------------------

/// The probe reaches the grouping satellite's period — the figure that decides
/// a calendar's *cycle*.
///
/// F4 covered `solar_day` and `orbital_period`, which feed the intercalation,
/// and stopped. A calendar's cycles come from a different figure, so a body
/// whose cycle was one digit from a different cycle passed with no comment — in
/// the feature built to measure exactly that class of fragility.
#[test]
fn the_probe_reaches_the_cycle() {
    let doc = ucal::cmd_cal_validate("earth-d", None).expect("a report");
    let v = probe(&doc, "grouping_period");
    assert!(v.contains("term"), "{v}");
}

/// **It reports a depth, not a rule, and that is the whole design.**
///
/// The first version compared the *chosen* cycle the way the intercalation probe
/// compares the chosen leap rule, and was worthless: a leap rule is selected by a
/// drift bound, so which rule it is can survive a nudge, while nothing selects a
/// cycle and the deepest convergent is the ratio itself. Any nudge changes it, so
/// the check could only ever print `sensitive`.
#[test]
fn the_cycle_probe_reports_a_useful_depth() {
    let earth = probe(
        &ucal::cmd_cal_validate("earth-d", None).expect("a report"),
        "grouping_period",
    );
    let depth: usize = earth
        .split("agrees for ")
        .nth(1)
        .and_then(|t| t.split(' ').next())
        .and_then(|n| n.parse().ok())
        .unwrap_or_else(|| panic!("no depth in: {earth}"));
    // A depth of 0 or of the full expansion would both mean the probe had
    // stopped discriminating. Earth's moon agrees for several terms and then
    // parts company, which is the informative middle.
    assert!(depth > 0, "nothing survives, which is the useless answer: {earth}");
    assert!(
        earth.contains("becomes"),
        "no divergence was shown: {earth}"
    );
}

/// **A calendar's grouping satellite is the calendar's declaration, not the
/// body's first moon.**
///
/// Mars has Phobos and Deimos and `mars-d` declares neither: D-A5 makes the
/// choice the calendar's, because no bracket over orbital periods can pick one
/// without smuggling in an Earth predicate. Reading `satellites().first()` here
/// made `cal validate mars-d` report a cycle that `cal show mars-d` says the
/// calendar does not have — one calendar, two answers, because a declaration
/// existed and the check went round it.
#[test]
fn a_calendar_declares_its_grouping_satellite() {
    let mars = ucal::cmd_cal_validate("mars-d", None).expect("a report");
    let cycles = verdict(&mars, "cycles");
    assert!(
        cycles.starts_with("none"),
        "mars-d declares no grouping satellite: {cycles}"
    );
    assert!(!cycles.contains("phobos"), "{cycles}");

    // Earth's is declared and is found.
    let earth = ucal::cmd_cal_validate("earth-d", None).expect("a report");
    assert!(verdict(&earth, "cycles").contains("moon"));

    // And a *file* keeps the first-listed rule, because a file has nowhere to
    // declare one. Same body, different source, different and correct answer.
    let (_d, p) = tmp("grouping-from-file", &GOOD);
    let from_file = ucal::cmd_cal_validate(p.to_str().expect("utf-8"), None).expect("a report");
    assert!(
        verdict(&from_file, "cycles").starts_with("none"),
        "the example body lists no satellite"
    );
}

/// The survey accounts for **every** calendar's cycle, including the fourteen
/// with none.
///
/// One of fifteen has a grouping satellite. A section showing that one and
/// silently omitting the rest is the shape V1 Finding 1 caught fourteen times in
/// this tree — a report that looks complete because nothing says what it left
/// out.
#[test]
fn the_survey_accounts_for_every_calendar_s_month() {
    let doc = ucal::cmd_cal_validate_all().expect("a survey");
    let Some(ucal::emit::Value::Rows { rows, .. }) = doc.get("cycles") else {
        panic!("no cycles rows");
    };
    let total = match doc.get("calendars") {
        Some(v) => v.rendered_text().trim().parse::<usize>().expect("a number"),
        None => panic!("no calendar count"),
    };
    assert_eq!(rows.len(), total, "the cycles section omitted calendars");
    // A calendar with a cycle names the satellite; one without says why not.
    let with = rows
        .iter()
        .filter(|(_, v)| {
            let ucal::emit::Value::Section(f) = v else {
                return false;
            };
            f.iter().any(|(k, val)| {
                k == "grouped_by" && val.rendered_text().trim() != "—"
            })
        })
        .count();
    assert_eq!(with, 1, "only earth-d declares a grouping satellite");
}

// ---- N2: a shipped calendar, exported and read back ---------------------

/// **Every shipped calendar survives a round trip through §15.1.**
///
/// The claim that a file can express exactly what a compiled-in body expresses
/// was checked by hand-written fixtures: somebody typed Mars's parameters into a
/// string literal and asserted `45/76` came back. That tests the fixture as much
/// as the loader, and only for the bodies somebody bothered to type.
///
/// Exported, it is a property over all fifteen — and it holds for the derived
/// solar days too, because those export as `derived:` rather than as a rounding
/// of their own result. 1.9.0 measured what that rounding would cost: Europa's
/// rule moves through five values across the first six decimals of its solar
/// day, and `europa-d` is in this loop.
#[test]
fn every_shipped_calendar_round_trips_through_a_file() {
    let mut checked = 0usize;
    for (id, body, _) in ucal_body::calendar::registered() {
        let text = ucal::cmd_cal_export(id).unwrap_or_else(|e| panic!("{id}: {e}"));
        let (_d, p) = tmp(&format!("roundtrip-{id}"), &text);

        // What the compiled-in body derives.
        let want = ucal_body::derive_leap_rule(
            body.solar_day().value_at_epoch(),
            body.orbital_period().value_at_epoch(),
            ucal_body::DriftBound::DEFAULT,
            32,
        )
        .unwrap_or_else(|e| panic!("{id} derives nothing: {e}"));

        // What the exported file derives, through the real loader.
        let loaded = ucal::body_file::load(&p).unwrap_or_else(|e| panic!("{id}: {e}"));
        let got = ucal_body::derive_leap_rule(
            loaded.solar_day().value_at_epoch(),
            loaded.orbital_period().value_at_epoch(),
            ucal_body::DriftBound::DEFAULT,
            32,
        )
        .unwrap_or_else(|e| panic!("{id} from file derives nothing: {e}"));

        assert_eq!(
            want.chosen.value.numer().to_dec_string(),
            got.chosen.value.numer().to_dec_string(),
            "{id}: the exported file derives a different rule"
        );
        assert_eq!(
            want.chosen.value.denom().to_dec_string(),
            got.chosen.value.denom().to_dec_string(),
            "{id}: the exported file derives a different rule"
        );
        checked += 1;
    }
    // A floor. A loop over an empty registry would pass having compared nothing,
    // which is the shape the 1.6.0 audit found fourteen times.
    assert_eq!(checked, 15, "expected every shipped calendar");
}

/// The exported file passes the validator, and is a *file* to it.
#[test]
fn an_exported_file_validates_as_a_file() {
    let text = ucal::cmd_cal_export("mars-d").expect("exports");
    let (_d, p) = tmp("export-validates", &text);
    let doc = ucal::cmd_cal_validate(p.to_str().expect("utf-8"), None).expect("a report");
    assert!(verdict(&doc, "loads").starts_with("ok"), "{}", verdict(&doc, "loads"));
    assert!(verdict(&doc, "intercalation").starts_with("45/76"));
}

/// **A derived parameter exports as `derived:`, never as its own result.**
///
/// Writing the computed solar day down would be writing down a rounding, which
/// is the exact defect the documented Europa example carried — a solar day no
/// source publishes, wrong in the third decimal.
#[test]
fn a_derived_parameter_does_not_export_as_a_decimal() {
    let text = ucal::cmd_cal_export("europa-d").expect("exports");
    assert!(
        text.contains("derived: synodic"),
        "europa's solar day was flattened into a number:\n{text}"
    );
    // And the block for it carries no `value:` at all.
    let solar = text
        .split("solar_day: {")
        .nth(1)
        .and_then(|s| s.split('}').next())
        .expect("a solar_day block");
    assert!(!solar.contains("value:"), "{solar}");
}

/// **G5's lookup reached two of fifteen calendars.**
///
/// `cal validate <id>` resolved a body through `calendar::by_id`, which *builds*
/// a calendar and so needs an anchor — and thirteen of the fifteen have none. So
/// it reported *no such body file* about a calendar `cal list` prints. The same
/// defect and the same fix as `Stride::calendar` in G6, which was corrected there
/// and not carried here.
#[test]
fn every_shipped_calendar_id_validates() {
    for (id, _, _) in ucal_body::calendar::registered() {
        ucal::cmd_cal_validate(id, None)
            .unwrap_or_else(|e| panic!("`cal validate {id}` failed: {e}"));
    }
}
