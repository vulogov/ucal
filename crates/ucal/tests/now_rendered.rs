//! `ucal now` renders at the precision it was asked for, or refuses.
//!
//! The human form anchors at T0 (D-A8) and cannot state a coarser tier. Until
//! 1.12.0 `cmd_now` swallowed that refusal: `ucal now --precision T4` exited 0
//! with a document that simply had no `rendered` field, while the manual
//! documented the field as always present. A document missing a field, with
//! nothing to say why, is the failure this project's error vocabulary exists to
//! prevent — it looks like a value. The refusal is now a §19.5 exit with the
//! remedy in the sentence.
//!
//! The second half is a smaller matter of the same kind. `explain` printed the
//! digit form with §6.2's `.` and `now --form digit5` printed it with §6.3's
//! `·`, so one form left one binary with two separators depending on which
//! command produced it. Both parse (§6.3), so nothing was wrong, and a reader
//! comparing the two had to know that to be sure.

use std::path::PathBuf;
use std::process::Command;

/// The binary under test, as cargo built it beside this test.
fn bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("test binary path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(format!("ucal{}", std::env::consts::EXE_SUFFIX))
}

fn run(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(bin())
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap_or_else(|e| panic!("could not run {}: {e}", bin().display()));
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// The instant the book's samples are pinned to; any instant would do.
const T: &str = "8070205189128471254993117657693008777530466139316558837890625";

fn line_starting<'a>(doc: &'a str, key: &str) -> Option<&'a str> {
    doc.lines().find(|l| l.starts_with(key))
}

/// A precision the human form cannot state is refused, with the remedy, and
/// nothing reaches stdout — so `ucal now --precision T4 > f` leaves no
/// half-document that looks complete.
#[test]
fn a_precision_the_human_form_cannot_state_is_refused_not_omitted() {
    let (code, out, err) = run(&["now", "--precision", "T4"]);
    assert_ne!(code, 0, "stdout: {out}");
    assert!(out.is_empty(), "nothing on stdout, got: {out}");
    assert!(err.contains("E0006"), "names the code: {err}");
    assert!(
        err.contains("--form digit5") && err.contains("--form named"),
        "names both forms that can state it: {err}"
    );
}

/// The two forms that carry their own anchor render every tier, and the human
/// form still renders everything from T0 down.
#[test]
fn the_forms_that_can_state_it_render_it() {
    for form in ["digit5", "named"] {
        let (code, out, err) = run(&["now", "--precision", "T4", "--form", form]);
        assert_eq!(code, 0, "{form}: {err}");
        assert!(line_starting(&out, "rendered").is_some(), "{form}: {out}");
        assert!(line_starting(&out, "precision").is_some(), "{form}: {out}");
    }
    for tier in ["T0", "T-3", "T-12"] {
        let (code, out, err) = run(&["now", "--precision", tier]);
        assert_eq!(code, 0, "{tier}: {err}");
        assert!(line_starting(&out, "rendered").is_some(), "{tier}: {out}");
    }
}

/// One form, one separator, whichever command prints it — and `--sep` still
/// moves both.
#[test]
fn the_digit_form_has_one_separator_whichever_command_prints_it() {
    let (_, exp, _) = run(&["explain", T]);
    let d = line_starting(&exp, "digit5").expect("explain prints digit5");
    assert!(d.contains('·') && !d.contains('.'), "explain: {d}");

    let (_, now, _) = run(&["now", "--form", "digit5"]);
    let r = line_starting(&now, "rendered").expect("now prints rendered");
    assert!(r.contains('·') && !r.contains('.'), "now: {r}");

    let (_, exp, _) = run(&["--sep", ".", "explain", T]);
    let d = line_starting(&exp, "digit5").expect("explain prints digit5");
    assert!(d.contains('.') && !d.contains('·'), "explain --sep .: {d}");

    let (_, now, _) = run(&["--sep", ".", "now", "--form", "digit5"]);
    let r = line_starting(&now, "rendered").expect("now prints rendered");
    assert!(r.contains('.') && !r.contains('·'), "now --sep .: {r}");
}
