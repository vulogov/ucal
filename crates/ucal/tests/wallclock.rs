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

    // `vulcan-d` and not `pluto-d`. Pluto was the stand-in for a calendar this
    // program does not have, and 1.9.0 added it — so the test began asserting
    // something false about the catalogue rather than about the lookup. The same
    // substitution was needed in `data::tests` in the same commit, which is how
    // often a body name gets used to mean "absent".
    let e = Face::at(instant(), en(), Some("vulcan-d")).expect_err("no such calendar");
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

/// `--once` is deterministic: same instant, same size, same bytes.
///
/// The property the committed frame in `CLI-EXAMPLES.md` depends on. If it were
/// not true, `check-docs` would fail on a clean tree at random and the artefact
/// would have to be deleted rather than fixed.
#[test]
fn a_frame_is_reproducible() {
    let f = face();
    let theme = theme::by_name("startrek").expect("a theme");
    let a = ucal::wallclock::once(&f, theme, 80, 26, false).expect("a frame");
    let b = ucal::wallclock::once(&f, theme, 80, 26, false).expect("a frame");
    assert_eq!(a, b);
    assert_eq!(a.lines().count(), 26);
    // Plain means plain.
    assert!(!a.contains('\u{1b}'), "escape sequences in an uncoloured frame");
}

/// With colour on, the frame carries SGR sequences and the same glyphs.
#[test]
fn a_coloured_frame_is_the_same_picture() {
    let f = face();
    let theme = theme::by_name("startrek").expect("a theme");
    let plain = ucal::wallclock::once(&f, theme, 80, 26, false).expect("a frame");
    let colour = ucal::wallclock::once(&f, theme, 80, 26, true).expect("a frame");
    assert!(colour.contains('\u{1b}'), "no escapes in a coloured frame");
    let stripped: String = strip_ansi(&colour);
    for line in plain.lines() {
        assert!(
            stripped.contains(line.trim_end()),
            "a line of the plain frame is missing from the coloured one: {line}"
        );
    }
}

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c == '\u{1b}' {
            for c in chars.by_ref() {
                if c == 'm' {
                    break;
                }
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// A frame outside the size the layout can say anything at is refused.
#[test]
fn an_absurd_frame_size_is_refused() {
    let f = face();
    let theme = theme::by_name("plain").expect("a theme");
    for (w, h) in [(19, 24), (401, 24), (80, 9), (80, 201)] {
        assert!(
            ucal::wallclock::once(&f, theme, w, h, false).is_err(),
            "{w}x{h} was accepted"
        );
    }
    assert!(ucal::wallclock::once(&f, theme, 20, 10, false).is_ok());
    assert!(ucal::wallclock::once(&f, theme, 400, 200, false).is_ok());
}

/// Every theme produces a frame, and no two themes produce the same one.
///
/// A palette-only theme still changes the bytes, because the colour is in them.
/// A theme that rendered identically to another would be a duplicate entry in a
/// catalogue, which is the thing `--theme list` exists to prevent.
#[test]
fn no_two_themes_render_the_same_frame() {
    let f = face();
    let mut seen: Vec<(&str, String)> = Vec::new();
    for t in theme::ALL {
        let frame = ucal::wallclock::once(&f, t, 90, 28, true).expect("a frame");
        for (other, prev) in &seen {
            assert_ne!(*prev, frame, "themes `{other}` and `{}` are the same", t.key);
        }
        seen.push((t.key, frame));
    }
    assert_eq!(seen.len(), theme::ALL.len());
}

/// Three layouts, three genuinely different arrangements.
///
/// `Theme` carried a `lcars: bool` for as long as there were two, and Z2 said
/// where that would stop being honest. This is the check that keeps the enum
/// earning itself: a "layout" that differed from another only in colour would
/// be a palette wearing a layout's name.
#[test]
fn each_layout_is_a_different_arrangement() {
    use ucal::wallclock::theme::Layout;
    let f = face();
    let mut by_layout: Vec<(Layout, String)> = Vec::new();
    for t in theme::ALL {
        // Uncoloured, so only the *arrangement* can differ.
        let frame = ucal::wallclock::once(&f, t, 90, 28, false).expect("a frame");
        if let Some((_, other)) = by_layout.iter().find(|(l, _)| *l == t.layout) {
            assert_eq!(
                *other, frame,
                "two themes share a layout but draw differently: {}",
                t.key
            );
        } else {
            for (l, other) in &by_layout {
                assert_ne!(
                    *other, frame,
                    "layouts {:?} and {:?} draw identically without colour",
                    l, t.layout
                );
            }
            by_layout.push((t.layout, frame));
        }
    }
    // Every theme's layout must be one of these, and every one of these must
    // be reached by some theme: a layout nothing selects is unreachable code,
    // and a theme whose layout nothing else shares is why the enum exists.
    assert_eq!(
        by_layout.len(),
        6,
        "expected six distinct layouts, found {:?}",
        by_layout.iter().map(|(l, _)| *l).collect::<Vec<_>>()
    );
}

/// The 1960s pair: an enamelled plate and a keypad, both drawn.
///
/// Two answers to the same decade, and the check is that they are answers rather
/// than palettes — `gagarin` is a surface with bezelled gauges, `armstrong` a
/// terminal with a verb, a noun and three registers.
#[test]
fn the_space_programme_faces_draw_their_own_furniture() {
    let f = face();
    let gagarin = ucal::wallclock::once(
        &f,
        theme::by_name("gagarin").expect("a theme"),
        96,
        28,
        false,
    )
    .expect("a frame");
    // The furniture, not the language: which language the plates are engraved
    // in is `--locale`'s business and is asserted in
    // `a_theme_does_not_override_the_locale`. This face is a *surface*, and what
    // makes it one is the bezels and the plate under the main instrument.
    assert!(gagarin.contains("READY"), "no lamp:\n{gagarin}");
    assert!(gagarin.contains("PRIMARY COUNT"), "no engraved plate:\n{gagarin}");
    assert!(gagarin.contains("┌────────────┐"), "no bezel:\n{gagarin}");

    let armstrong = ucal::wallclock::once(
        &f,
        theme::by_name("armstrong").expect("a theme"),
        96,
        28,
        false,
    )
    .expect("a frame");
    for want in ["COMP ACTY", "VERB", "NOUN", "R1", "R2", "R3", "PROG"] {
        assert!(armstrong.contains(want), "no `{want}`:\n{armstrong}");
    }
    // V16 N65 is a real pair — monitor, decimal, time — and not decoration.
    assert!(armstrong.contains("16"), "{armstrong}");
    assert!(armstrong.contains("65"), "{armstrong}");
}

/// A theme does not override `--locale` — not even the Cyrillic one.
///
/// Through 1.8.0 the Vostok panel's chrome was hardcoded Russian while its tier
/// names followed the flag, so `--gagarin --locale en` drew Cyrillic plates
/// around English names. That was the one place in the program where a theme
/// beat a user's flag, and F10 closed it: under `--locale en` the whole face is
/// English, and under `--locale ru` the whole face is Russian.
#[test]
fn a_theme_does_not_override_the_locale() {
    let gagarin = theme::by_name("gagarin").expect("a theme");
    let f_en = Face::at(instant(), en(), None).expect("a face");
    let out = ucal::wallclock::once(&f_en, gagarin, 96, 28, false).expect("a frame");
    assert!(out.contains("UNIVERSE CALENDAR"), "chrome should be English:\n{out}");
    assert!(out.contains("T0 BEAT"), "names should follow --locale:\n{out}");
    assert!(
        !out.contains("ВРЕМЯ ВСЕЛЕННОЙ"),
        "the 1.8.0 behaviour, and the bug:\n{out}"
    );

    let ru = LocaleId::parse("ru").expect("ru ships");
    let f_ru = Face::at(instant(), ru, None).expect("a face");
    let out_ru = ucal::wallclock::once(&f_ru, gagarin, 96, 28, false).expect("a frame");
    assert!(out_ru.contains("ВРЕМЯ ВСЕЛЕННОЙ"), "{out_ru}");
    assert!(out_ru.contains("ДУГА"), "{out_ru}");
}

/// Every face, in both locales, prints chrome in the language that was asked
/// for.
///
/// The Vostok panel is the one that was *reported*, because Cyrillic under
/// `--locale en` is visibly a flag being ignored. English chrome under
/// `--locale ru` is the same bug and looks like a translation nobody finished,
/// so it went unnoticed on the other six faces. This test is over all of them.
#[test]
fn every_face_follows_the_locale() {
    let ru = LocaleId::parse("ru").expect("ru ships");
    for t in theme::ALL {
        let f_en = Face::at(instant(), en(), None).expect("a face");
        let out_en = ucal::wallclock::once(&f_en, t, 100, 30, false).expect("a frame");
        assert!(
            !out_en.chars().any(|c| ('\u{400}'..='\u{4ff}').contains(&c)),
            "{}: Cyrillic under --locale en:\n{out_en}",
            t.key
        );

        let f_ru = Face::at(instant(), ru, None).expect("a face");
        let out_ru = ucal::wallclock::once(&f_ru, t, 100, 30, false).expect("a frame");
        assert!(
            out_ru.chars().any(|c| ('\u{400}'..='\u{4ff}').contains(&c)),
            "{}: nothing Russian under --locale ru:\n{out_ru}",
            t.key
        );
    }
}

/// The targeting face draws its instrument furniture.
#[test]
fn the_starwars_face_is_a_gunsight() {
    let f = face();
    let theme = theme::by_name("starwars").expect("a theme");
    let out = ucal::wallclock::once(&f, theme, 90, 28, false).expect("a frame");
    assert!(out.contains("TARGETING"), "{out}");
    assert!(out.contains('┼'), "no crosshair:\n{out}");
    assert!(out.contains("┌──"), "no canopy bracket:\n{out}");
    // Every hand on one strip, because an instrument does not give a number its
    // own panel.
    for h in &f.hands {
        assert!(
            out.contains(&format!("{:04}", h.position)),
            "hand {} missing from the strip:\n{out}",
            h.label()
        );
    }
}

/// The dials draw round, and the hand points where the arithmetic says.
///
/// A dial is easy to get subtly wrong in a way that still looks like a clock:
/// an ellipse instead of a circle, a hand running anticlockwise, a quadrant
/// swapped. This checks the geometry rather than the appearance — the dot the
/// hand ends on must be the one `cos_sin` names, and the rim must be the same
/// distance from the centre in both axes.
#[test]
fn a_dial_is_round_and_its_hand_points_the_right_way() {
    use ucal::wallclock::dial::{cos_sin, Canvas, SCALE};

    // Straight up, a quarter clockwise, half, three quarters.
    for (p, want_c, want_s) in [
        (0u32, 1i64, 0i64),
        (3125 / 4, 0, 1),
        (3125 / 2, -1, 0),
        (3 * 3125 / 4, 0, -1),
    ] {
        let (c, s) = cos_sin(p);
        assert!(
            (c - want_c * SCALE).abs() < SCALE / 50 && (s - want_s * SCALE).abs() < SCALE / 50,
            "position {p} gave ({c}, {s}), wanted about ({}, {})",
            want_c * SCALE,
            want_s * SCALE
        );
    }

    // Round: the rim's extent in x and in y must match, because braille dots are
    // square. An aspect correction applied at the wrong level draws an ellipse
    // and looks deliberate.
    let mut canvas = Canvas::new(20, 10);
    canvas.dial(0);
    let lines = canvas.lines();
    let blank = '\u{2800}';
    let rows: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.chars().any(|c| c != blank))
        .map(|(i, _)| i)
        .collect();
    let cols: Vec<usize> = (0..20)
        .filter(|c| {
            lines
                .iter()
                .any(|l| l.chars().nth(*c).is_some_and(|ch| ch != blank))
        })
        .collect();
    let (h_cells, w_cells) = (
        rows.last().unwrap_or(&0) - rows.first().unwrap_or(&0) + 1,
        cols.last().unwrap_or(&0) - cols.first().unwrap_or(&0) + 1,
    );
    // Cells are 2 dots wide and 4 tall, so a round dial covers about twice as
    // many columns as rows.
    let ratio = w_cells * 100 / h_cells.max(1);
    assert!(
        (150..=260).contains(&ratio),
        "the dial is {w_cells} cells wide and {h_cells} tall: ratio {ratio}, wanted about 200"
    );
}

/// The orbit face draws dials and no block digits.
///
/// It is the only face here with no big numerals, which is the point of it. If
/// block digits appeared, the layout would have quietly become another variant
/// of `plain` with a decoration on top.
#[test]
fn the_orbit_face_has_hands_and_no_block_digits() {
    let f = face();
    let out = ucal::wallclock::once(
        &f,
        theme::by_name("orbit").expect("a theme"),
        96,
        24,
        false,
    )
    .expect("a frame");
    assert!(out.contains('\u{2800}') || out.contains('⠿'), "no braille:\n{out}");
    assert!(!out.contains('█'), "orbit should have no block digits:\n{out}");
    for h in &f.hands {
        assert!(out.contains(&h.position.to_string()), "{out}");
    }
}

// ---- F3: several dials, a chosen hero, and an odometer ------------------

use ucal::wallclock::Dials;

fn dials() -> Dials {
    Dials::new(en()).expect("defaults")
}

/// An airport wall: both dials are drawn, in the order asked for.
///
/// Only two of the fifteen calendars can be a dial at all — a dial shows local
/// fields, local fields need a phase, and phase is empirical (Rule J.3) — so
/// this is the whole wall that can be built today, and that is a fact about
/// anchors rather than about the flag.
#[test]
fn several_dials_are_all_drawn() {
    let d = dials().with_clock_local(&["earth-d".to_string(), "mars-d".to_string()]);
    let f = Face::of(instant(), &d).expect("a face");
    assert_eq!(f.dials.len(), 2, "{:?}", f.dials);
    let out = drawn_face(&f, "plain", 100, 32);
    let earth = out.find("EARTH-D").expect("no earth dial");
    let mars = out.find("MARS-D").expect("no mars dial");
    assert!(earth < mars, "the dials came out in the wrong order:\n{out}");
}

/// A dial that cannot exist is a message and an exit code, not a blank panel —
/// and adding a second dial did not lose that.
#[test]
fn an_unanchored_dial_is_still_refused() {
    let d = dials().with_clock_local(&["earth-d".to_string(), "titan-d".to_string()]);
    assert!(
        Face::of(instant(), &d).is_err(),
        "titan-d has no anchor and was accepted as a dial"
    );
}

/// `--tier` promotes a hand to the big readout.
#[test]
fn the_hero_tier_can_be_chosen() {
    let t2 = Tier::new(2).expect("a tier");
    let d = dials().with_hero(t2);
    let f = Face::of(instant(), &d).expect("a face");
    let beat = f.beat().expect("a hero");
    assert_eq!(beat.tier.index(), 2);

    // And the readout actually shows it: T2's position, not T0's.
    let plain = Face::of(instant(), &dials()).expect("a face");
    let t0 = plain.beat().expect("a hero").position;
    assert_ne!(beat.position, t0, "T2 and T0 happened to coincide");
}

/// Z2's kill criterion for `--tier`, answered in the display.
///
/// "Every choice but `T0` produces a screen where nothing moves, which is a
/// stopped clock with extra steps." `T1` moves every 2 min 26 s and is still a
/// clock; `T2` is 5.3 days and `T3` is 45 years, and those are calendar
/// displays. Refusing them would refuse the flag's stated purpose, so the face
/// says which it is — a hand that changes every 45 years is pixel-identical to a
/// clock that has stopped.
#[test]
fn a_slow_hero_says_it_is_a_calendar_and_not_a_clock() {
    for (k, expected) in [(0i8, false), (1, false), (2, true), (3, true)] {
        let d = dials().with_hero(Tier::new(k).expect("a tier"));
        let f = Face::of(instant(), &d).expect("a face");
        let out = drawn_face(&f, "plain", 100, 32);
        assert_eq!(
            out.contains("does not move while you watch"),
            expected,
            "T{k} said the wrong thing about itself:\n{out}"
        );
    }
}

/// The odometer counts up from an origin, on the same ladder the hands are on.
#[test]
#[cfg(feature = "events")]
fn the_odometer_counts_from_an_origin() {
    let (origin, label) = ucal::wallclock_origin("bridge-epoch").expect("an exact origin");
    let d = dials().with_since(origin, label);
    let f = Face::of(instant(), &d).expect("a face");
    let o = f.since.as_ref().expect("an odometer");
    assert!(!o.counting_down, "the bridge epoch is in the past");
    assert_eq!(o.drums.len(), 5, "five rungs, like the face");

    let out = drawn_face(&f, "plain", 100, 32);
    assert!(out.contains("SINCE"), "{out}");
    assert!(out.contains("bridge-epoch"), "{out}");
}

/// An origin in the future counts towards it rather than reporting a negative.
///
/// Absolute time is unsigned (Rule B) and `Ticks` cannot hold a negative, so the
/// direction is a word beside a magnitude — which is what `SignedWindow` already
/// does, for the same reason.
#[test]
fn an_origin_in_the_future_counts_down() {
    let later: String = {
        let mut s = T.to_string();
        s.push('0'); // ten times further out, and comfortably later
        s
    };
    let (origin, label) = ucal::wallclock_origin(&later).expect("an instant");
    let d = dials().with_since(origin, label);
    let f = Face::of(instant(), &d).expect("a face");
    assert!(f.since.as_ref().expect("an odometer").counting_down);
    let out = drawn_face(&f, "plain", 100, 32);
    assert!(out.contains("UNTIL"), "{out}");
}

/// **Z2's kill criterion for `--since`, and the reason it is a feature.**
///
/// > Stop if: the window of the chosen origin exceeds the resolution of the
/// > display [...] Then it should refuse and say why, rather than render a
/// > number whose last twelve digits are decoration.
///
/// Checked both ways over the whole catalogue: every event with a window wider
/// than `T-1` is refused, and every event with one no wider is accepted. A check
/// that only ever refused would pass with the accept path broken.
#[test]
#[cfg(feature = "events")]
fn a_wide_window_is_refused_and_an_exact_one_is_not() {
    let finest = Tier::new(-1).expect("a tier");
    let mut refused = 0;
    let mut accepted = 0;
    for e in ucal_events::all() {
        let got = ucal::wallclock_origin(e.id);
        if e.uncertainty().ticks() > &finest.ticks() {
            let err = got.expect_err("a window wider than the finest hand was accepted");
            assert_eq!(err.code, ucal_core::Code::E0023, "{}: {err}", e.id);
            refused += 1;
        } else {
            got.unwrap_or_else(|e| panic!("an exact origin was refused: {e}"));
            accepted += 1;
        }
    }
    assert!(refused > 0, "no event in the catalogue was refused");
    assert!(
        accepted > 0,
        "no event in the catalogue was accepted, so the accept path is untested"
    );
}

/// The odometer's leading drum does not wrap.
///
/// Every other rung is a position out of 3125 and reads mod 3125, like the
/// face's hands. Without an unwrapped leading drum this reading could not tell
/// 2 000 years from 142 000: one `T3` span is 45 years and 3125 of them is
/// 141 000.
#[test]
#[cfg(feature = "events")]
fn the_leading_drum_carries_the_whole_count() {
    let (origin, label) = ucal::wallclock_origin("bridge-epoch").expect("an origin");
    let near = Face::of(instant(), &dials().with_since(origin, label)).expect("a face");
    let a = near.since.as_ref().expect("an odometer").drums[0].position;

    // The same origin, read 3125 T3 spans later. A wrapping drum reads the same.
    let far = {
        let bump = Tier::new(3)
            .expect("a tier")
            .ticks()
            .try_mul(&<Ticks as TickInt>::from_u64(3125))
            .expect("in range");
        let t = Instant::<UC1>::from_ticks(
            instant().ticks().try_add(&bump).expect("in range"),
        )
        .expect("inside the domain");
        let (origin, label) = ucal::wallclock_origin("bridge-epoch").expect("an origin");
        Face::of(t, &dials().with_since(origin, label)).expect("a face")
    };
    let b = far.since.as_ref().expect("an odometer").drums[0].position;
    assert_eq!(b, a + 3125, "the leading drum wrapped: {a} then {b}");
}

// ---- G8: a face as a document ------------------------------------------

/// `--json` was the one global flag `wallclock` accepted and ignored.
///
/// It drew a face anyway. That alone is the defect G2 catalogued; what makes it
/// worth more than a refusal is that a face **is** structured data — hands with
/// tier indices and positions, dials with local fields, an odometer with its
/// drums — every one of which the text renderer already has and throws into
/// glyphs.
#[test]
fn a_face_renders_as_a_document() {
    let f = face();
    let doc = ucal::cmd_wallclock_json(&f, "plain").expect("a document");
    let Some(ucal::emit::Value::Rows { rows: hands, .. }) = doc.get("hands") else {
        panic!("no hands");
    };
    assert_eq!(hands.len(), 5, "T3 down to T-1");

    // The index is not localised and the name is (Rule N). Both are emitted,
    // because the index is what a reader compares across two machines set to
    // different languages.
    for (_, v) in hands {
        let ucal::emit::Value::Section(fields) = v else {
            panic!("a hand is a section");
        };
        for want in ["index", "name", "position", "per_mille"] {
            assert!(
                fields.iter().any(|(k, _)| k == want),
                "a hand has no `{want}`"
            );
        }
    }
}

/// The document is a *reading*, and matches the face it came from.
///
/// A second rendering of the same data is a second place for it to be wrong, so
/// the check is that the two agree rather than that the JSON looks plausible.
#[test]
fn the_document_agrees_with_the_face_it_came_from() {
    let ru = LocaleId::parse("ru").expect("ru ships");
    let d = Dials::new(ru)
        .expect("defaults")
        .with_clock_local(&["earth-d".to_string()]);
    let f = Face::of(instant(), &d).expect("a face");
    let doc = ucal::cmd_wallclock_json(&f, "gagarin").expect("a document");

    let Some(ucal::emit::Value::Rows { rows: hands, .. }) = doc.get("hands") else {
        panic!("no hands");
    };
    for h in &f.hands {
        let (_, v) = hands
            .iter()
            .find(|(k, _)| *k == h.tier.to_string())
            .unwrap_or_else(|| panic!("no hand for {}", h.tier));
        let text = v.rendered_text();
        assert!(
            text.contains(&h.position.to_string()),
            "{} position missing: {text}",
            h.tier
        );
        // The localised name travels with it.
        assert!(text.contains(&h.name), "{} name missing: {text}", h.tier);
    }

    // The dial is there, and named by its calendar.
    let Some(ucal::emit::Value::Rows { rows: dials, .. }) = doc.get("dials") else {
        panic!("no dials");
    };
    assert_eq!(dials.len(), 1);
    assert_eq!(dials[0].0, "earth-d");
}

/// A face with no second dial and no odometer emits neither key.
///
/// An empty section would say *asked for and empty*, which is a different fact
/// from *not asked for*.
#[test]
fn absent_dials_are_absent_rather_than_empty() {
    let doc = ucal::cmd_wallclock_json(&face(), "plain").expect("a document");
    assert!(doc.get("dials").is_none(), "an unasked dial was emitted");
    assert!(doc.get("since").is_none(), "an unasked odometer was emitted");
}

/// `--tier` reaches the document, so a scripted reader sees which hand is the
/// hero rather than having to know the default.
#[test]
fn the_hero_is_named_in_the_document() {
    let d = dials().with_hero(Tier::new(2).expect("a tier"));
    let f = Face::of(instant(), &d).expect("a face");
    let doc = ucal::cmd_wallclock_json(&f, "plain").expect("a document");
    assert_eq!(
        doc.get("hero").map(ucal::emit::Value::rendered_text),
        Some("T2 ".to_string())
    );
}
