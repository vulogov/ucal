//! Y2 — anchor files, and what stops one being a way to invent a phase.
//!
//! §15.1 names body files *and* anchor files, versioned independently because
//! parameters change with better measurement and anchors with re-determination.
//! 1.4.0 did the first. This is the second, and it is the half that turns a
//! derivation into a **date**.
//!
//! It is also the half with the risk. [`X1.3`] named it as a kill criterion
//! rather than a detail:
//!
//! > Loading must not become a way to invent an anchor. GE-3's kill criterion
//! > forbids narrowing a window by assumption, and a file is a much easier place
//! > to do it than a Rust constant.
//!
//! So most of this file is about refusals, and the one acceptance is the
//! strongest check available: an anchor file that reproduces a compiled-in
//! anchor, and produces the same date through it.
//!
//! [`X1.3`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/X1-authoring-local-calendars.md

use std::io::Write;
use ucal::anchor_file;
use ucal_core::Code;

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";

fn examples() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .join("Documentation/examples")
}

fn body_example() -> String {
    examples()
        .join("earth.hjson")
        .to_str()
        .expect("utf-8")
        .to_string()
}

fn anchor_text() -> String {
    std::fs::read_to_string(examples().join("earth-anchor.hjson")).expect("read the example")
}

fn tmp(label: &str, contents: &str) -> (Dir, std::path::PathBuf) {
    let dir = Dir::new(label);
    let path = dir.path().join("anchor.hjson");
    let mut f = std::fs::File::create(&path).expect("create");
    f.write_all(contents.as_bytes()).expect("write");
    (dir, path)
}

/// Per-call label, not the pid alone: every test in this binary shares the pid
/// and they run in parallel. `body_file.rs` learned that the hard way.
struct Dir(std::path::PathBuf);
impl Dir {
    fn new(label: &str) -> Dir {
        let mut p = std::env::temp_dir();
        p.push(format!("ucal-anchor-file-{}-{label}", std::process::id()));
        let _ = std::fs::create_dir_all(&p);
        Dir(p)
    }
    fn path(&self) -> &std::path::Path {
        &self.0
    }
}
impl Drop for Dir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// **The check that matters: a file reproduces a date the program already knows.**
///
/// Earth's body file and Earth's anchor file, both stating exactly what
/// `data::earth` and `anchors::earth` state, must produce the same local fields
/// as `ucal cal show earth-d` for the same instant.
///
/// If they did not, the file route would be a second implementation of the
/// calendar agreeing with the first only by coincidence, and every date produced
/// from a file would be subtly wrong in a way nothing here could detect. Rule K.5
/// says Earth is an ordinary instance and §15.4 says its entry has no special
/// code path; this is that claim in the form an outside author can check.
#[test]
fn a_file_pair_produces_the_same_date_as_the_compiled_in_calendar() {
    let (_d, anchor) = tmp("earth-roundtrip", &anchor_text());
    let from_files = ucal::cmd_cal_derive_with(
        &body_example(),
        Some(anchor.to_str().expect("utf-8")),
        Some(T),
    )
    .expect("the example pair derives a date")
    .to_json();
    let compiled = ucal::cmd_cal_show("earth-d", T)
        .expect("earth-d shows")
        .to_json();

    // Scoped to the `fields` section: `ladder_placement` has a row called
    // `year` too, and the first version of this test compared that instead.
    let from_files = section(&from_files, "fields");
    let compiled = section(&compiled, "fields");
    for field in ["\"year\"", "\"day\"", "\"day_fraction\"", "\"window_ticks\""] {
        let a = value_after(&from_files, field);
        let b = value_after(&compiled, field);
        assert_eq!(a, b, "{field} differs between the file route and the compiled-in one");
    }
}

/// Everything from a named section onward.
fn section<'a>(json: &'a str, name: &str) -> &'a str {
    let at = json
        .find(&format!("\"{name}\""))
        .unwrap_or_else(|| panic!("no {name} section in:\n{json}"));
    &json[at..]
}

/// Pull the first value following a key, for comparing two JSON documents that
/// are not the same shape.
fn value_after(json: &str, key: &str) -> String {
    let at = json
        .find(key)
        .unwrap_or_else(|| panic!("no {key} in:\n{json}"));
    json[at + key.len()..]
        .trim_start_matches([':', ' '])
        .chars()
        .take_while(|c| *c != ',' && *c != '\n' && *c != '}')
        .collect::<String>()
        .trim()
        .to_string()
}

/// Rule J.1: a phase must name an event of the body, not a foreign epoch.
///
/// This is the refusal the whole feature stands on. The named phase kinds cannot
/// express a foreign reference at all — that is the structural half — and
/// `custom` can, which is why it is word-screened. The screen is a partial
/// defence and this crate says so: a determined author can smuggle a foreign
/// reference past a word list. What it catches is the accidental case, which is
/// the likely one, because a familiar epoch is the handiest number to hand.
#[test]
fn a_phase_naming_a_foreign_epoch_is_refused() {
    for foreign in [
        "the unix epoch",
        "J2000.0 in TT",
        "midnight UTC on the Gregorian new year",
        "julian day 2451545",
    ] {
        let bad = swap_phase(
            &anchor_text(),
            &format!(
                "phase: {{\n  kind: custom\n  citation: a real source, cited\n  \
                 description: {foreign}\n}}"
            ),
        );
        let (_d, p) = tmp(&format!("foreign-{}", foreign.len()), &bad);
        let e = anchor_file::load(&p)
            .map(|_| ())
            .expect_err(&format!("`{foreign}` was accepted as a phase definition"));
        assert_eq!(e.code, Code::E0063, "for `{foreign}`");
    }
}

/// And a phase that names an event of the body is accepted, so the screen is
/// refusing the right thing rather than everything custom.
#[test]
fn a_custom_phase_naming_a_body_event_is_accepted() {
    let ok = swap_phase(
        &anchor_text(),
        "phase: {\n  kind: custom\n  citation: a real source, cited\n  \
         description: the first sunrise over the northern ice cap after the \
         southward equinox\n}",
    );
    let (_d, p) = tmp("custom-ok", &ok);
    anchor_file::load(&p).expect("a body event is a valid phase definition");
}

/// Replace the whole `phase:` block, so a test never leaves a half-edited one.
fn swap_phase(text: &str, block: &str) -> String {
    let start = text.find("phase: {").expect("the example states a phase");
    let end = text[start..].find("\n}").expect("the phase block closes") + start + 2;
    format!("{}{block}{}", &text[..start], &text[end..])
}

/// Rule J.2: the window must contain the tick.
///
/// A window that excludes its own best estimate is a contradiction, not an
/// uncertainty. This is the narrowing GE-3 forbids, in the form a file makes
/// easiest: change one digit and the anchor claims a precision it does not have.
#[test]
fn a_window_that_excludes_its_own_tick_is_refused() {
    // Move both bounds above the tick.
    let bad = anchor_text().replace(
        "window_lo: 8070205173569172848578881211763614680530466139316558837890625",
        "window_lo: 8070205173569172848615978380563336680530466139316558837890625",
    );
    let bad = bad.replace(
        "window_hi: 8070205173569172848615978380563336680530466139316558837890625",
        "window_hi: 8070205173569172848615978380563336680530466139316558837890626",
    );
    let (_d, p) = tmp("window-excludes-tick", &bad);
    let e = anchor_file::load(&p).expect_err("a window must contain its own tick");
    assert_eq!(e.code, Code::E0062);
}

/// Rule J.3's three obligations are three required fields.
#[test]
fn a_determination_missing_any_obligation_is_refused() {
    for missing in ["method", "citation", "uncertainty_note"] {
        let bad: String = anchor_text()
            .lines()
            .filter(|l| !l.trim().starts_with(&format!("{missing}:")))
            .collect::<Vec<_>>()
            .join("\n");
        let (_d, p) = tmp(&format!("missing-{missing}"), &bad);
        assert!(
            anchor_file::load(&p).is_err(),
            "a determination missing `{missing}` was accepted"
        );
    }
}

/// §15.1: an unknown key is `UCAL-E0012`, in this file as in the body file.
#[test]
fn an_unknown_key_is_e0012() {
    let bad = anchor_text().replace("revision: 1", "revision: 1\nconfidence: high");
    let (_d, p) = tmp("unknown-key", &bad);
    let e = anchor_file::load(&p).expect_err("an unknown key must be refused");
    assert_eq!(e.code, Code::E0012);
}

/// D-A22: a file that will not load is `UCAL-E0017`.
#[test]
fn an_unloadable_anchor_file_is_e0017() {
    let e = anchor_file::load(std::path::Path::new("/nonexistent/ucal-no-anchor.hjson"))
        .expect_err("a missing file must be refused");
    assert_eq!(e.code, Code::E0017);
}

/// An anchor file must name the calendar the body file derives.
///
/// The failure this catches is quiet and plausible: two files that each load,
/// pair up, and produce a date for a body using another body's phase. Rule J
/// says an anchor is never borrowed from another body, and this is the form
/// borrowing would take.
#[test]
fn an_anchor_for_another_calendar_is_refused() {
    let bad = anchor_text().replace("calendar_id: earth-d", "calendar_id: mars-d");
    let (_d, p) = tmp("wrong-calendar", &bad);
    let e = ucal::cmd_cal_derive_with(&body_example(), Some(p.to_str().expect("utf-8")), None)
        .expect_err("an anchor for another calendar must be refused");
    assert_eq!(e.code, Code::E0062);
    assert!(e.to_string().contains("mars-d"), "{e}");
}

/// `--at` without `--anchor` says why, rather than producing nothing.
#[test]
fn asking_for_a_date_without_an_anchor_says_why() {
    let e = ucal::cmd_cal_derive_with(&body_example(), None, Some(T))
        .expect_err("a date needs a phase");
    assert_eq!(e.code, Code::E0062);
    assert!(e.to_string().contains("--anchor"), "{e}");
}

/// The documented example pair is a pair this program accepts.
#[test]
fn the_documented_examples_load() {
    let a = anchor_file::load(&examples().join("earth-anchor.hjson"))
        .unwrap_or_else(|e| panic!("the documented anchor example does not load: {e}"));
    assert_eq!(a.calendar_id(), "earth-d");
    assert_eq!(a.revision(), 1);
}
