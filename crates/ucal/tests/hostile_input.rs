//! §19.5, made checkable: the binary does not panic, and every failure carries
//! an exit code and a sentence.
//!
//! # What this is for
//!
//! A panic reaching a user is two failures at once. The message is wrong — it
//! names a Rust source location and suggests `RUST_BACKTRACE` to someone who
//! typed a date badly — and the exit code is 101, which §19.5 does not define,
//! so a script cannot tell "your input was wrong" from "the program broke".
//!
//! Three properties, checked against the real binary rather than the library,
//! because the exit code and the stream a message lands on are properties of the
//! *process*:
//!
//! 1. no invocation panics;
//! 2. a failing invocation exits non-zero and says why, on stderr;
//! 3. a failing invocation writes nothing to stdout, so `ucal … > f` never
//!    leaves a half-written document behind.
//!
//! # Why a corpus and not a fuzzer
//!
//! A fuzzer would be better and is not free: it needs a harness, a corpus that
//! is committed or regenerated, and a time budget in CI. This is the cheap
//! version — inputs chosen where the parsing actually is, run in a second. It
//! finds the class of defect that reached 0.9.0 (an exit 0 on total failure)
//! and would not find a rare arithmetic edge. Stated so a green run is not read
//! as more than it is.

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

struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(args: &[&str]) -> Run {
    let out = Command::new(bin())
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap_or_else(|e| panic!("could not run {}: {e}", bin().display()));
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// Inputs aimed at the parsers, one per line of attack.
const HOSTILE: &[&[&str]] = &[
    // instants
    &["explain", ""],
    &["explain", " "],
    &["explain", "abc"],
    &["explain", "-1"],
    &["explain", "0x10"],
    &["explain", "1e300"],
    &["explain", "UC1"],
    &["explain", "UC1 ····"],
    &["explain", "١٢٣"],
    &["to-civil", "zzz"],
    &["between", "0", "abc"],
    &["between", "abc", "0"],
    &["between", "", ""],
    // tiers
    &["now", "--precision", "T99"],
    &["now", "--precision", "T-99"],
    &["now", "--precision", "5^"],
    &["now", "--precision", "5^999"],
    &["now", "--precision", ""],
    &["between", "0", "100", "--at", "nope"],
    &["timeline", "--tier", "T999999999999999999999"],
    &["ruler", "--from", "0", "--to", "100", "--step", "zzz"],
    &["ruler", "--from", "100", "--to", "0"],
    // civil dates
    &["from-civil", ""],
    &["from-civil", "0000-00-00"],
    &["from-civil", "2026-13-45"],
    &["from-civil", "2026-02-30"],
    &["from-civil", "2026-01-01T99:99:99"],
    // calendars
    &["cal", "show", "nope", "0"],
    &["cal", "show", "", "0"],
    &["cal", "anchor", "nope"],
    &["show", "0", "--calendars", "a,b"],
    &["show", "0", "--calendars", ""],
    // events and cosmology
    &["events", "show", "nope"],
    &["events", "show", ""],
    &["cosmo", "age", "--z", "-1"],
    &["cosmo", "age", "--z", "abc"],
    &["cosmo", "age", "--z", "5..1"],
    &["cosmo", "age", "--z", ".."],
    // global options
    &["--sep", "1", "explain", "0"],
    &["--tick-sep", "aa", "explain", "0"],
    &["--profile", "UC-2", "datum"],
    &["completions", "nope"],
    &["completions"],
];

/// Inputs that must succeed, so the suite cannot pass by rejecting everything.
const BENIGN: &[&[&str]] = &[
    // Both of these look hostile and are not: the domain ceiling is 5^220, so a
    // 68-digit tick count and a ten-digit year are ordinary values. They sat in
    // the hostile corpus until it was run, which is the corpus being wrong
    // rather than the program.
    &["explain", "99999999999999999999999999999999999999999999999999999999999999999999"],
    &["from-civil", "9999999999-01-01"],
    &["datum"],
    &["doctor"],
    &["verify"],
    &["ladder"],
    &["cal", "list"],
    &["explain", "0"],
    &["between", "0", "100"],
    &["between", "0", "100", "--at", "beat"],
    &["--json", "verify"],
    &["completions", "bash"],
    &["completions", "zsh"],
    &["completions", "fish"],
    &["completions", "powershell"],
    &["completions", "elvish"],
];

/// Nothing panics, whatever it is given.
#[test]
fn no_input_produces_a_panic() {
    let mut bad = Vec::new();
    for args in HOSTILE.iter().chain(BENIGN.iter()) {
        let r = run(args);
        let all = format!("{}{}", r.stdout, r.stderr);
        if all.contains("panicked at")
            || all.contains("RUST_BACKTRACE")
            || all.contains("internal error")
            || r.code == 101
        {
            bad.push(format!("ucal {} -> exit {}", args.join(" "), r.code));
        }
    }
    assert!(
        bad.is_empty(),
        "these invocations panicked:\n  {}",
        bad.join("\n  ")
    );
}

/// Every rejection exits non-zero and says why, on stderr, having written
/// nothing to stdout.
#[test]
fn every_rejection_is_a_code_and_a_sentence() {
    let mut bad = Vec::new();
    for args in HOSTILE {
        let r = run(args);
        let what = format!("ucal {}", args.join(" "));
        if r.code == 0 {
            bad.push(format!("{what}: exited 0 on input it could not use"));
            continue;
        }
        if r.stderr.trim().is_empty() {
            bad.push(format!("{what}: exit {} with nothing on stderr", r.code));
        }
        if !r.stdout.trim().is_empty() {
            bad.push(format!("{what}: wrote to stdout while failing"));
        }
    }
    assert!(
        bad.is_empty(),
        "§19.5 says a failure is an exit code and a message:\n  {}",
        bad.join("\n  ")
    );
}

/// The exit codes are the ones §19.5 defines, not arbitrary numbers.
#[test]
fn exit_codes_come_from_the_defined_set() {
    // 0–9 are §19.5's. 2 is also clap's usage error, which is the same category.
    // 70 is `EX_SOFTWARE`, reserved here for a panic that reached the handler —
    // and no input in this corpus should produce it.
    let mut bad = Vec::new();
    for args in HOSTILE {
        let r = run(args);
        if !(0..=9).contains(&r.code) {
            bad.push(format!("ucal {} -> exit {}", args.join(" "), r.code));
        }
    }
    assert!(
        bad.is_empty(),
        "exit codes outside §19.5's table:\n  {}",
        bad.join("\n  ")
    );
}

/// What should work still works — otherwise the two tests above pass trivially.
#[test]
fn the_benign_corpus_succeeds() {
    for args in BENIGN {
        let r = run(args);
        assert_eq!(
            r.code,
            0,
            "ucal {} should succeed, exited {}:\n{}",
            args.join(" "),
            r.code,
            r.stderr
        );
        assert!(!r.stdout.trim().is_empty(), "ucal {} printed nothing", args.join(" "));
    }
}
