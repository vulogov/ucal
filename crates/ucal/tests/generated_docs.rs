//! `completions` and `man` describe the program they were generated from.
//!
//! Both come out of the same `clap` definition the binary parses its arguments
//! with, which is the entire argument for generating them rather than writing
//! them: a hand-written completion script or manual page is a second
//! description of the CLI, and a second description drifts.
//!
//! That argument is only worth making if the generation is actually wired to
//! the parser, so these tests take a command that was added recently and check
//! it appears — the failure being a generator pointed at a stale definition,
//! which would look perfectly healthy from the outside.

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

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(bin())
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .expect("run ucal");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
    )
}

/// Every shell produces a script, and it knows the current commands.
///
/// `between` and `verify` arrived in 0.8.0 and `man` in 1.1.0; a generator
/// reading a stale definition would still emit a plausible script without them.
#[test]
fn completions_know_the_commands_that_exist() {
    for shell in ["bash", "zsh", "fish", "powershell", "elvish"] {
        let (code, out) = run(&["completions", shell]);
        assert_eq!(code, 0, "`completions {shell}` failed");
        assert!(out.len() > 400, "`completions {shell}` produced almost nothing");
        for cmd in ["between", "verify", "man", "completions"] {
            assert!(
                out.contains(cmd),
                "the {shell} completions do not mention `{cmd}`"
            );
        }
    }
}

/// The top-level page is roff, and names the program.
#[test]
fn the_manual_page_is_roff() {
    let (code, out) = run(&["man"]);
    assert_eq!(code, 0);
    assert!(out.starts_with(".ie") || out.contains("\n.TH "), "not roff:\n{}", &out[..80.min(out.len())]);
    assert!(out.contains(".SH NAME"));
    assert!(out.contains(".SH SYNOPSIS"));
    assert!(out.contains(".SH SUBCOMMANDS"));
}

/// Every subcommand the top-level page cross-references has a page.
///
/// roff convention makes the `SUBCOMMANDS` section reference `ucal-now(1)` and
/// its siblings. Until `man` took an argument those were dangling — a reader
/// following one got nothing — so this checks the promise the page makes about
/// itself, by reading the references out of the page rather than from a list
/// maintained here.
#[test]
fn every_cross_reference_resolves() {
    let (_, top) = run(&["man"]);
    let mut referenced: Vec<String> = Vec::new();
    for line in top.lines() {
        let line = line.trim();
        // `ucal\-between(1)` — roff escapes the hyphen.
        if let Some(rest) = line.strip_prefix("ucal\\-") {
            if let Some(name) = rest.strip_suffix("(1)") {
                referenced.push(name.replace("\\-", "-"));
            }
        }
    }
    assert!(
        referenced.len() >= 10,
        "expected the page to cross-reference its subcommands, found {referenced:?}"
    );
    // `help` is clap's own built-in. It is not in the command tree until clap
    // builds it, so no page is generated for it and its cross-reference is the
    // one that stays dangling — clap's, not this program's. Named here rather
    // than quietly filtered, because a skipped case in a test that exists to
    // find dangling references should be visible.
    for name in referenced.iter().filter(|n| *n != "help") {
        let (code, out) = run(&["man", name]);
        assert_eq!(code, 0, "`ucal man {name}` failed, but the page references it");
        assert!(
            out.contains(&format!("ucal\\-{}", name.replace('-', "\\-"))),
            "`ucal man {name}` does not name itself"
        );
    }
}

/// An unknown subcommand is a diagnostic and an exit code, not a panic.
#[test]
fn an_unknown_page_is_refused_properly() {
    let out = Command::new(bin())
        .args(["man", "nope"])
        .output()
        .expect("run ucal");
    assert_ne!(out.status.code(), Some(0));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(err.contains("no subcommand"), "{err}");
    assert!(!err.contains("panicked"), "{err}");
    assert!(
        out.stdout.is_empty(),
        "a refused page should write nothing to stdout"
    );
}
