//! D-A24 — one code for *a value this program does not accept*, and only that.
//!
//! The survey behind it is in `spec/SPEC-DELTAS.md`. The short version: twelve
//! `UCAL-E0001` raisers, of which two were malformed timestamps, plus `E0060` —
//! *body parameter missing required provenance* — attached to a flag
//! combination and an arithmetic overflow.
//!
//! What this file holds is the boundary. `E0018` must cover all four shapes of
//! *bad value*, and must not creep back over the conditions the survey
//! deliberately left alone.

use ucal_core::Code;

/// Every shape `E0018` is declared to cover, from both directions it arrives.
#[test]
fn every_shape_of_a_bad_value_is_e0018() {
    // A closed vocabulary, missed — from the command line.
    assert_eq!(
        ucal::style::ColorChoice::parse("mauve").expect_err("no such colour").code,
        Code::E0018
    );

    // A shape constraint, broken. Two characters, and a digit (§6.3).
    for sep in ["ab", "7"] {
        assert_eq!(
            ucal::style::parse_group_sep(sep).expect_err("bad separator").code,
            Code::E0018,
            "for `{sep}`"
        );
    }

    // A range, left.
    assert_eq!(
        ucal::cmd_cosmo_age("-1", 40, 40).expect_err("a redshift is not negative").code,
        Code::E0018
    );
}

/// `E0001` keeps its meaning, and has only raisers that fit it.
///
/// The point of the survey was not to empty `E0001` but to leave in it exactly
/// what its name describes: a string that was meant to be an instant and is not.
#[test]
fn e0001_is_a_malformed_timestamp_and_nothing_else() {
    // The last is 2^512 written out: one past the domain ceiling, and a decimal
    // tick count in every other respect. The first draft used a 32-digit number
    // as "obviously too big", which parses perfectly — the domain is 155 digits
    // wide, and a guess about this calendar's scale was wrong by 123 of them.
    let past_the_ceiling = "1".to_string() + &"0".repeat(200);
    for bad in ["abc", "", "UC1 nonsense", &past_the_ceiling] {
        let e = ucal::parse_instant(bad).expect_err("not an instant");
        assert_eq!(e.code, Code::E0001, "for `{bad}`");
    }
}

/// The conditions the survey left alone stay where they are.
///
/// A derivation with no answer is a **result**, not a bad input, and calling it
/// one would be a second inversion of the kind D-A23 fixed. If this test starts
/// failing because someone folded them into `E0018`, the argument for doing so
/// belongs in a delta, not in a commit.
#[cfg(feature = "body")]
#[test]
fn a_derivation_with_no_answer_is_not_a_bad_value() {
    use std::io::Write;
    let dir = std::env::temp_dir().join(format!("ucal-argcodes-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("locked.hjson");
    let mut f = std::fs::File::create(&path).expect("create");
    f.write_all(
        b"id: locked\nprimary: star\nrotation_period: {\n  value: 3.5\n  unit: d\n  \
          citation: hypothetical\n  valid_years: 100\n}\nsolar_day: {\n  derived: synodic\n  \
          citation: derived\n  valid_years: 100\n}\norbital_period: {\n  value: 3.5\n  \
          unit: d\n  citation: hypothetical\n  valid_years: 100\n}\n",
    )
    .expect("write");

    let e = ucal::body_file::load(&path).expect_err("a locked body has no solar day");
    assert_eq!(
        e.code,
        Code::E0060,
        "a body whose periods make a quantity undefined is not a bad value"
    );
    assert!(e.to_string().contains("tidally locked"), "{e}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A file's closed vocabulary is the same condition as a flag's.
///
/// `kind: whenever` and `--scale gps` differ in where the value came from, not
/// in what is wrong with it, and the remedy is identical.
#[cfg(feature = "body")]
#[test]
fn a_files_closed_vocabulary_is_the_same_class_as_a_flags() {
    use std::io::Write;
    let dir = std::env::temp_dir().join(format!("ucal-argcodes-vocab-{}", std::process::id()));
    let _ = std::fs::create_dir_all(&dir);
    let path = dir.join("body.hjson");
    let mut f = std::fs::File::create(&path).expect("create");
    f.write_all(
        b"id: x\nrotation_period: {\n  value: 1\n  unit: parsec\n  citation: c\n  \
          valid_years: 1\n}\nsolar_day: {\n  value: 2\n  unit: d\n  citation: c\n  \
          valid_years: 1\n}\norbital_period: {\n  value: 3\n  unit: d\n  citation: c\n  \
          valid_years: 1\n}\n",
    )
    .expect("write");
    let e = ucal::body_file::load(&path).expect_err("parsec is not a unit here");
    assert_eq!(e.code, Code::E0018);
    let _ = std::fs::remove_dir_all(&dir);
}

/// `E0018` exits 2, as `E0001` did. The caller's situation has not changed.
#[test]
fn the_exit_status_did_not_move() {
    assert_eq!(Code::E0018.exit_code(), Code::E0001.exit_code());
    assert_eq!(Code::E0018.exit_code(), 2);
}
