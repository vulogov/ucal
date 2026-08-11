//! The wall clock, rendered into a buffer.
//!
//! A TUI is the easiest thing in a program to leave untested, because running it
//! needs a terminal. `TestBackend` does not, so there is no excuse: the face is
//! drawn into a fixed-size buffer and read back as text.
#![cfg(feature = "tui")]

use ratatui::backend::TestBackend;
use ratatui::Terminal;
use ucal::wallclock::{theme, Face};
use ucal_core::backend::TickInt;
use ucal_core::{Instant, LocaleId, Ticks, Tier, UC1};

const T: &str = "8070205189123984864657505252035637180530466139316558837890625";

fn en() -> LocaleId {
    LocaleId::parse("en").expect("en ships")
}

fn instant() -> Instant<UC1> {
    Instant::<UC1>::from_ticks(
        <Ticks as TickInt>::from_dec_str(T).expect("a decimal tick count"),
    )
    .expect("inside the domain")
}

fn face() -> Face {
    Face::at(instant(), en(), None).expect("a face")
}

fn drawn(theme_key: &str, w: u16, h: u16) -> String {
    drawn_face(&face(), theme_key, w, h)
}

fn drawn_face(face: &Face, theme_key: &str, w: u16, h: u16) -> String {
    let theme = theme::by_name(theme_key).expect("a theme");
    let mut term = Terminal::new(TestBackend::new(w, h)).expect("a test terminal");
    term.draw(|f| face.render(f, theme)).expect("draws");
    let buf = term.backend().buffer().clone();
    (0..buf.area.height)
        .map(|y| {
            (0..buf.area.width)
                .map(|x| buf[(x, y)].symbol().to_string())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Every theme draws, at a size a terminal might actually be.
///
/// The failure this catches is the one a TUI has that nothing else does: a
/// layout that indexes past the end of a `split` and takes the process down. The
/// CLI carries no panicking construct by lint, and `no-panic-in-cli` cannot see
/// an index into a `Vec` that a constraint list made shorter than expected.
#[test]
fn every_theme_draws_at_every_plausible_size() {
    for t in theme::ALL {
        for (w, h) in [(80, 24), (120, 40), (200, 60), (40, 20), (20, 10), (10, 5)] {
            let out = drawn(t.key, w, h);
            assert_eq!(
                out.lines().count(),
                h as usize,
                "{} at {w}x{h} produced the wrong number of rows",
                t.key
            );
        }
    }
}

/// The readout is the beat, in block digits, and it is on the screen.
#[test]
fn the_beat_is_drawn_large() {
    let f = face();
    let beat = f.beat().expect("a beat hand");
    // Every block-digit row is made of these two characters and spaces.
    let out = drawn("plain", 80, 24);
    assert!(
        out.contains('█'),
        "no block digits in the readout:\n{out}"
    );
    assert!(beat.position < 3125, "a hand has 3125 stops, not {}", beat.position);
}

/// LCARS draws its rail and its header, and plain draws neither.
#[test]
fn the_startrek_theme_is_visibly_a_different_layout() {
    let lcars = drawn("startrek", 100, 30);
    let plain = drawn("plain", 100, 30);
    assert!(lcars.contains("UNIVERSE CALENDAR"), "{lcars}");
    assert!(lcars.contains('▄'), "no LCARS elbow:\n{lcars}");
    assert!(!plain.contains('▄'), "the plain theme grew chrome:\n{plain}");
    assert_ne!(lcars, plain);
}

/// Rule A.5: no Earth unit on the face.
///
/// A clock is where the temptation lives — every clock a reader has seen counts
/// in hours, minutes and seconds, and the whole point of this one is that it
/// does not. The face names tiers, and the only place a second appears is in the
/// sentence explaining how fast the blurred tier moves, which is a statement
/// about the display rather than a unit the clock counts in.
#[test]
fn the_face_names_no_earth_unit_as_a_unit() {
    for t in theme::ALL {
        let out = drawn(t.key, 120, 40).to_lowercase();
        for unit in ["hour", "minute", " am ", " pm ", "o'clock", "utc", "gregorian"] {
            assert!(
                !out.contains(unit),
                "the {} face shows `{unit}`:\n{out}",
                t.key
            );
        }
    }
}

/// Every hand is inside its dial, for an instant at each end of the domain.
#[test]
fn every_hand_is_a_position_on_a_dial_of_3125() {
    for ticks in ["0", "1", T, &Ticks::domain_max().to_dec_string()] {
        let Some(v) = <Ticks as TickInt>::from_dec_str(ticks) else {
            continue;
        };
        let Ok(t) = Instant::<UC1>::from_ticks(v) else {
            continue;
        };
        let f = Face::at(t, en(), None).expect("a face");
        for h in &f.hands {
            assert!(
                h.position < 3125,
                "{} is at {}, off the end of its dial",
                h.tier,
                h.position
            );
            assert!(h.per_mille() <= 1000);
        }
    }
}

/// The clock's hands agree with the tier arithmetic they claim to show.
///
/// `T1` is 3125 `T0`s. A face whose `T1` hand did not advance once per 3125
/// beats would be a picture of a clock rather than a clock.
#[test]
fn a_tiers_hand_advances_once_per_3125_of_the_one_below() {
    let base = <Ticks as TickInt>::from_dec_str(T).expect("ticks");
    let t0 = Tier::new(0).expect("T0");
    let one_beat = t0.ticks();

    let f0 = Face::at(
        Instant::<UC1>::from_ticks(base.clone()).expect("in domain"),
        en(),
        None,
    )
    .expect("face");
    let a = f0.beat().expect("beat").position;

    let plus = base
        .try_add(&one_beat)
        .expect("one beat later is inside the domain");
    let f1 = Face::at(Instant::<UC1>::from_ticks(plus).expect("in domain"), en(), None)
        .expect("face");
    let b = f1.beat().expect("beat").position;

    assert_eq!(
        b,
        (a + 1) % 3125,
        "one beat later, the beat hand did not advance one stop"
    );
}

/// An unknown theme is `UCAL-E0016`, and says where to find the real ones.
#[test]
fn an_unknown_theme_is_a_catalogue_miss() {
    let e = theme::by_name("klingon").expect_err("no such theme");
    assert_eq!(e.code, ucal_core::Code::E0016);
    assert!(e.to_string().contains("--theme list"), "{e}");
}

/// Rule N: the tier names on the face are the locale's, and the indices are not.
///
/// The face is display, so it uses the locale's names. `T0` is beside each one
/// and stays `T0` in every language — which is what a reader compares when two
/// machines are set differently, and the reason Rule N scopes *names* to a
/// locale and nothing else.
#[test]
fn the_face_is_drawn_in_the_chosen_locale() {
    let ru = LocaleId::parse("ru").expect("ru ships");
    let f = Face::at(instant(), ru, None).expect("a face");
    let out = drawn_face(&f, "plain", 100, 30);
    assert!(out.contains("бой"), "no Russian tier name on the face:\n{out}");
    assert!(out.contains("T0"), "the tier index should not be localised:\n{out}");

    let english = drawn("plain", 100, 30);
    assert!(english.contains("beat"), "{english}");
    assert_ne!(out, english, "the locale changed nothing");
}

/// The second dial shows a body's own calendar, and says which anchor made it.
#[test]
fn a_second_dial_shows_a_local_calendar() {
    let f = Face::at(instant(), en(), Some("mars-d")).expect("mars-d has an anchor");
    let local = f.local.clone().expect("a second dial");
    assert_eq!(local.calendar, "mars-d");
    assert!(local.through_day <= 100);
    assert_eq!(local.revision, 1);

    let out = drawn_face(&f, "startrek", 100, 32);
    assert!(out.contains("MARS-D"), "{out}");
    assert!(out.contains("anchor revision"), "{out}");

    // And the universe face is unchanged by its presence: a second dial is an
    // addition, not a replacement.
    let without = drawn("startrek", 100, 32);
    assert!(!without.contains("MARS-D"));
    for hand in f.hands.iter() {
        assert!(out.contains(&hand.position.to_string()), "{out}");
    }
}

/// A calendar with no anchor cannot be a second dial, and says so before the
/// clock takes over the terminal.
///
/// Ten of the twelve derived calendars that ship are in this state. It is the
/// ordinary case (Rule J.3), and the failure worth avoiding is a full-screen
/// clock with an empty panel and no way to see why.
#[test]
fn an_unanchored_calendar_is_refused_as_a_dial() {
    let e = Face::at(instant(), en(), Some("titan-d")).expect_err("titan has no anchor");
    assert_eq!(e.code, ucal_core::Code::E0062);

    let e = Face::at(instant(), en(), Some("pluto-d")).expect_err("no such calendar");
    assert_eq!(e.code, ucal_core::Code::E0016);
}

/// Every theme draws the second dial at every plausible size.
#[test]
fn the_second_dial_draws_everywhere_too() {
    let f = Face::at(instant(), en(), Some("earth-d")).expect("earth-d has an anchor");
    for t in theme::ALL {
        for (w, h) in [(80, 24), (120, 40), (40, 20), (20, 10), (10, 5)] {
            let out = drawn_face(&f, t.key, w, h);
            assert_eq!(out.lines().count(), h as usize, "{} at {w}x{h}", t.key);
        }
    }
}

/// The local year counts from the anchor and starts at **one**.
///
/// The first person to see `year 27` on the face asked whether it meant 2027.
/// It does not: `earth-d` is anchored at 2000-01-01 and counts from 1, so year
/// 27 is the twenty-seventh year of that reckoning and falls in Gregorian 2026.
/// Both wrong readings — "2027" and "2000 + 27" — are off by one in opposite
/// directions, which is what an unlabelled count invites.
///
/// The convention is pinned here rather than described, and the face states it
/// in words beside the number.
#[test]
fn the_local_year_counts_from_one_at_the_anchor() {
    // One tick after `earth-d`'s anchor.
    let anchor_plus_one = <Ticks as TickInt>::from_dec_str(
        "8070205173569172848597429796163475680530466139316558837890626",
    )
    .expect("ticks");
    let t = Instant::<UC1>::from_ticks(anchor_plus_one).expect("in domain");
    let f = Face::at(t, en(), Some("earth-d")).expect("earth-d has an anchor");
    let local = f.local.clone().expect("a dial");
    assert_eq!(local.year, "1", "the year at the anchor must be 1, not 0");
    assert_eq!(local.day, "1", "the day at the anchor must be 1, not 0");

    // And the face says what the number counts, so nobody has to ask again.
    let out = drawn_face(&f, "startrek", 110, 32);
    assert!(
        out.contains("year 1 began there"),
        "the face does not say what the year counts:\n{out}"
    );
}
