//! F2 — `-` as an instant, and what a filter has to guarantee.
//!
//! Until 1.9.0 no command read stdin. Every conversion was one process for one
//! timestamp, which meant `ucal` could not appear in a pipeline at all — the
//! single largest thing standing between it and ordinary use.
//!
//! These run the built binary rather than calling into the library, because the
//! property under test is the *process*: what it reads, what it writes, and what
//! it exits with.

use std::io::Write;
use std::process::{Command, Stdio};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";
const DATUM_ISH: &str = "8070204002895596515944343085635637180530466139316558837890625";

fn bin() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("target/release/ucal")
}

/// Run the binary with `input` on stdin. Returns (stdout, exit code).
fn run(args: &[&str], input: &str) -> Option<(String, i32)> {
    let path = bin();
    if !path.exists() {
        return None; // built without --release; the suite says so below
    }
    let mut child = Command::new(path)
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .ok()?;
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(input.as_bytes())
        .expect("write");
    let out = child.wait_with_output().expect("wait");
    Some((
        String::from_utf8_lossy(&out.stdout).into_owned(),
        out.status.code().unwrap_or(-1),
    ))
}

/// **One record in, one record out.** The property that makes it a filter.
#[test]
fn json_mode_emits_one_line_per_input() {
    let Some((out, code)) = run(&["--json", "to-civil", "-"], &format!("{T}\n{DATUM_ISH}\n")) else {
        return;
    };
    assert_eq!(code, 0);
    let lines: Vec<&str> = out.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 2, "two instants in, two records out:\n{out}");
    for l in &lines {
        assert!(l.starts_with('{') && l.ends_with('}'), "not one JSON record: {l}");
        assert!(l.contains("\"ucal-json/1\""), "the surface is unchanged: {l}");
    }
}

/// Blank lines are skipped rather than being an error.
///
/// A file ending in a newline, or a `grep` that let one through, should not be a
/// failure — the caller meant to convert timestamps, and a blank line is not one.
#[test]
fn blank_lines_are_skipped() {
    let Some((out, code)) = run(&["--json", "to-civil", "-"], &format!("\n{T}\n\n\n")) else {
        return;
    };
    assert_eq!(code, 0);
    assert_eq!(out.lines().filter(|l| !l.trim().is_empty()).count(), 1);
}

/// **A bad line does not throw away the good ones**, and the exit code still says
/// something went wrong.
///
/// A filter that dies on line 3 of 10 000 has discarded 9 997 answers it had
/// already computed. A filter that exits 0 on a partial run lets a script treat
/// it as complete. Both are wrong, and they are wrong in opposite directions.
#[test]
fn a_bad_line_is_reported_and_the_rest_still_run() {
    let Some((out, code)) = run(
        &["--json", "to-civil", "-"],
        &format!("{T}\nnot-an-instant\n{DATUM_ISH}\n"),
    ) else {
        return;
    };
    assert_eq!(
        out.lines().filter(|l| l.starts_with('{')).count(),
        2,
        "the two good lines still produced records"
    );
    assert_ne!(code, 0, "a partial run must not exit 0");
}

/// Streaming and single-shot agree **on the answer**.
///
/// The stream substitutes one value into the same dispatch, so a streamed record
/// must say what the same arguments say alone. If they ever differ, the stream
/// has grown a second code path.
///
/// Compared field by field rather than byte by byte. The first version of this
/// test stripped all whitespace from the single-shot output and compared the
/// results — which also strips the spaces *inside* strings, so it was measuring
/// a naive compaction against the string-aware one and failing on
/// `"earth-civil: 2026-..."`. The test was wrong and the code was right, which
/// is the same mistake `compact_json`'s own unit tests exist to catch.
#[test]
fn a_streamed_record_agrees_with_the_single_shot_one() {
    let Some((streamed, _)) = run(&["--json", "to-civil", "-", "--digits", "3"], &format!("{T}\n"))
    else {
        return;
    };
    let Some((single, _)) = run(&["--json", "to-civil", T, "--digits", "3"], "") else {
        return;
    };
    for field in ["qualified", "calendar_id", "ticks", "kind"] {
        let a = value_of(&streamed, field);
        let b = value_of(&single, field);
        assert_eq!(a, b, "`{field}` differs between streamed and single-shot");
        assert!(!a.is_empty(), "`{field}` was not found in either output");
    }
}

/// The quoted value following `"field":`, whitespace after the colon allowed.
fn value_of(json: &str, field: &str) -> String {
    let key = format!("\"{field}\"");
    let Some(at) = json.find(&key) else {
        return String::new();
    };
    let rest = json[at + key.len()..].trim_start();
    let Some(rest) = rest.strip_prefix(':') else {
        return String::new();
    };
    let rest = rest.trim_start();
    let Some(rest) = rest.strip_prefix('"') else {
        return String::new();
    };
    rest.split('"').next().unwrap_or("").to_string()
}

/// Only the commands taking exactly one instant accept `-`.
///
/// `between` takes two, and a line-oriented filter has no answer for which of
/// the two a line is — so it treats `-` as what it is, an unparseable instant,
/// rather than guessing.
#[test]
fn two_instant_commands_do_not_pretend_to_stream() {
    let Some((_, code)) = run(&["between", "-", T], "") else {
        return;
    };
    assert_ne!(code, 0, "`between -` should be a rejection, not a stream");
}

/// G3 — `-` on a command that does not read stdin says so.
///
/// F2 added stdin and left `UCAL-E0001` listing three accepted forms out of
/// four, so a caller who tried to stream into `between` was told "malformed
/// timestamp" about an argument that is not malformed at all. The message a
/// caller hits *while getting the syntax wrong* was missing a quarter of the
/// syntax.
#[test]
fn a_dash_on_a_non_streaming_command_explains_itself() {
    let e = ucal::parse_instant("-").expect_err("`-` is not an instant");
    let msg = e.to_string();
    assert!(msg.contains("reads instants from stdin"), "{msg}");
    // And it names the commands that do take it, rather than leaving the reader
    // to find out by trying them.
    for c in ["to-civil", "explain", "show", "cal show"] {
        assert!(msg.contains(c), "`{c}` is not named: {msg}");
    }
}

/// And the general message now mentions `-` too.
#[test]
fn the_malformed_timestamp_message_knows_about_stdin() {
    let e = ucal::parse_instant("not-an-instant").expect_err("malformed");
    let msg = e.to_string();
    assert!(msg.contains("tick count"), "{msg}");
    assert!(msg.contains("UCID"), "{msg}");
    assert!(
        msg.contains("stdin"),
        "the four accepted forms are three in this message: {msg}"
    );
}
