//! The panic handler, verified rather than read.
//!
//! `crates/ucal/src/main.rs` installs a hook so that a panic leaves as a
//! diagnostic and a defined exit code instead of `thread 'main' panicked at
//! …` plus a suggestion to set `RUST_BACKTRACE`. A hook is exactly the sort of
//! thing that looks right and is never executed, so this induces a panic in the
//! real binary — via `UCAL_PANIC_SELFTEST`, which is an environment variable
//! precisely so it is not CLI surface — and asserts on what comes out.
//!
//! The `no-panic-in-cli` lint is the policy; this is the backstop's own test.

use std::path::PathBuf;
use std::process::Command;

fn bin() -> PathBuf {
    let mut p = std::env::current_exe().expect("test binary path");
    p.pop();
    if p.ends_with("deps") {
        p.pop();
    }
    p.join(format!("ucal{}", std::env::consts::EXE_SUFFIX))
}

fn induced() -> (i32, String, String) {
    let out = Command::new(bin())
        .arg("datum")
        .env("UCAL_PANIC_SELFTEST", "1")
        .env("NO_COLOR", "1")
        .output()
        .expect("run ucal");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

/// A panic exits 70, not 101.
///
/// 101 is Rust's default and means nothing in §19.5, so a script cannot tell it
/// from anything else. 70 is `EX_SOFTWARE` from `sysexits.h` — "an internal
/// software error" — which is what it is, and it sits outside §19.5's 0–9 so it
/// cannot be confused with a diagnosed failure.
#[test]
fn a_panic_exits_seventy() {
    let (code, _, _) = induced();
    assert_eq!(code, 70, "a panic should exit EX_SOFTWARE, not Rust's 101");
}

/// The message says whose fault it is, and how to report it.
///
/// The default hook's audience is the person who wrote the program. This one's
/// audience is the person running it, who needs to know that their input was
/// not the problem and that the thing is worth reporting.
#[test]
fn the_message_is_for_the_person_running_it() {
    let (_, _, err) = induced();
    assert!(err.contains("internal error"), "{err}");
    assert!(
        err.contains("bug in ucal, not in your input"),
        "the message should say the input was not at fault:\n{err}"
    );
    assert!(
        err.contains("github.com/vulogov/ucal/issues"),
        "the message should say where to report it:\n{err}"
    );
}

/// No backtrace machinery reaches the user.
#[test]
fn no_traceback_is_shown() {
    let (_, out, err) = induced();
    let all = format!("{out}{err}");
    for noise in [
        "RUST_BACKTRACE",
        "thread 'main' panicked",
        "note: run with",
        "stack backtrace",
    ] {
        assert!(
            !all.contains(noise),
            "the default panic output leaked `{noise}`:\n{all}"
        );
    }
}

/// The location survives, because a bug report without one is worth much less.
///
/// This is the one part of the default hook worth keeping: it costs a line and
/// it is the difference between a reproducible report and "it crashed".
#[test]
fn the_location_is_kept() {
    let (_, _, err) = induced();
    assert!(
        err.contains("main.rs:"),
        "the panic location should be reported:\n{err}"
    );
}

/// A panic writes nothing to stdout.
#[test]
fn stdout_stays_clean() {
    let (_, out, _) = induced();
    assert!(
        out.trim().is_empty(),
        "a panic put this on stdout, where a redirect would capture it:\n{out}"
    );
}
