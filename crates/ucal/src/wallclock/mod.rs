//! `ucal wallclock` — the Universe calendar as a clock on a wall.
//!
//! # What a clock for this calendar can and cannot be
//!
//! A wall clock shows a moving hand. Which of this calendar's units *move* at a
//! rate a person can watch is a question with a numeric answer, and it decides
//! the whole design:
//!
//! | tier | one of them is | visible? |
//! |---|---|---|
//! | `T1` arc | 2 min 26 s | changes while you watch |
//! | `T0` beat | 46.8 ms | ~21 per second — the seconds hand |
//! | `T-1` flicker | 15 µs | a blur, and drawn as one |
//! | `T-2` glint and below | ns and less | not drawable at any refresh rate |
//!
//! So the clock has a face like any other: slow hands you read, one fast hand
//! you watch, and a blur below it that is shown as a bar rather than as digits
//! nobody can catch. The bar is honest — it is the flicker's real position, and
//! it moves 66 000 times a second, which is what a bar is for.
//!
//! Above `T1` the hands are calendar rather than clock: `T2` sweep is 5.3 days,
//! `T3` span is 45 years. They are on the face because they are what the
//! calendar *is*, and because a clock that showed only the fast end would be a
//! stopwatch.
//!
//! # Themes
//!
//! [`Theme`] is a palette and a layout switch, not a plug-in system. Two ship:
//! `plain`, which is the default, and `startrek`, which is LCARS. Adding one is
//! a `const` in [`theme`] and an entry in [`theme::by_name`].
//!
//! # Not in the default install
//!
//! `ratatui` and `crossterm` are a large tree and this module is behind the
//! non-default `tui` feature, which [`GE-U4-tier-navigator.md`] asked for in as
//! many words: `cargo install ucal` should stay lean. The release binaries are
//! built with it.
//!
//! # No panics, including on the way out
//!
//! A TUI takes the terminal out of cooked mode, and a process that dies without
//! putting it back leaves the user with no echo and no line editing. Every exit
//! path here restores the terminal, including the one taken when drawing itself
//! fails, which is why [`run`] separates the restore from the result it returns.
//!
//! [`GE-U4-tier-navigator.md`]: https://github.com/vulogov/ucal/blob/main/Documentation/Proposals/GE-U4-tier-navigator.md

pub mod digits;
pub mod face;
pub mod theme;

use ucal_core::{Instant, LocaleId, TimeError, UC1};

pub use face::Face;
pub use theme::Theme;

use crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::time::{Duration, Instant as StdInstant};

/// How often the face is redrawn.
///
/// The beat advances about 21 times a second, so 20 Hz shows most of them and
/// costs little. Faster would show more and buy nothing a reader can use: the
/// next tier down moves 3125 times faster still, and is drawn as a bar for
/// exactly that reason.
const REFRESH: Duration = Duration::from_millis(50);

/// Run the clock until the user quits.
///
/// `q`, `Esc` or `Ctrl-C` stops it. Any theme key listed by
/// [`theme::by_name`] may be given; an unknown one is `UCAL-E0016`, because a
/// theme is a name in a declared catalogue like any other.
///
/// `locale` is the *language* the tier names are drawn in (Rule N).
/// `clock_local` is the *place* — a body's own calendar, shown as a second
/// dial, which is what the second face on a wall clock has always been for.
/// Two vocabularies, two flags: `--locale ru` and `--clock-local mars-d`.
pub fn run(
    theme_name: &str,
    locale: LocaleId,
    clock_local: Option<&str>,
) -> Result<(), TimeError> {
    let theme = theme::by_name(theme_name)?;
    // Read the second dial once before taking over the terminal. A calendar id
    // that does not exist, or one that exists and has no anchor, is a message
    // and an exit code — not a full-screen clock with an empty panel on it and
    // no way to see why.
    if let Some(id) = clock_local {
        Face::at(now_instant()?, locale, Some(id))?;
    }

    let mut term = enter()?;
    // The clock's own result is kept separate from the restore, so a failure
    // inside the loop cannot leave the terminal in raw mode. Both are reported;
    // the loop's failure wins, because it is the one that explains anything.
    let outcome = clock_loop(&mut term, theme, locale, clock_local);
    let restored = leave(&mut term);
    outcome.and(restored)
}

fn enter() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, TimeError> {
    use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen};
    let mut out = std::io::stdout();
    enable_raw_mode().map_err(terminal_failure)?;
    crossterm::execute!(out, EnterAlternateScreen, crossterm::cursor::Hide)
        .map_err(terminal_failure)?;
    Terminal::new(CrosstermBackend::new(out)).map_err(terminal_failure)
}

fn leave(term: &mut Terminal<CrosstermBackend<std::io::Stdout>>) -> Result<(), TimeError> {
    use crossterm::terminal::{disable_raw_mode, LeaveAlternateScreen};
    disable_raw_mode().map_err(terminal_failure)?;
    crossterm::execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        crossterm::cursor::Show
    )
    .map_err(terminal_failure)?;
    term.show_cursor().map_err(terminal_failure)
}

fn terminal_failure(e: std::io::Error) -> TimeError {
    use ucal_core::Code;
    // Not a diagnostic about time. E0010's family is a resource that would not
    // load; a terminal that will not enter raw mode is the same shape.
    let _ = e;
    TimeError::with_context(
        Code::E0017,
        "the terminal could not be put into full-screen mode; `ucal wallclock` \
         needs an interactive terminal",
    )
}

fn clock_loop(
    term: &mut Terminal<CrosstermBackend<std::io::Stdout>>,
    theme: &'static Theme,
    locale: LocaleId,
    clock_local: Option<&str>,
) -> Result<(), TimeError> {
    let mut last = StdInstant::now();
    loop {
        let face = Face::read_now(locale, clock_local)?;
        term.draw(|f| face.render(f, theme))
            .map_err(terminal_failure)?;

        // Poll for the remainder of the refresh interval, so a keypress is
        // answered immediately rather than up to one frame later.
        let waited = last.elapsed();
        // A frame that took longer than the refresh interval gets a zero poll
        // budget, which is the right answer: it is already late. Rule O is about
        // a clamped *result* standing in for an error, and there is no error
        // here to stand in for — this is a `std::time::Duration` describing how
        // long to wait for a keypress, not a quantity in this calendar.
        // ucal-lint-allow-begin(no-wrapping-arithmetic): a poll budget, already late
        let budget = REFRESH.saturating_sub(waited);
        // ucal-lint-allow-end(no-wrapping-arithmetic)
        if event::poll(budget).map_err(terminal_failure)? {
            if let Event::Key(k) = event::read().map_err(terminal_failure)? {
                if k.kind == KeyEventKind::Press && quits(&k) {
                    return Ok(());
                }
            }
        }
        last = StdInstant::now();
    }
}

fn quits(k: &event::KeyEvent) -> bool {
    use crossterm::event::KeyModifiers;
    matches!(k.code, KeyCode::Char('q') | KeyCode::Char('Q') | KeyCode::Esc)
        || (k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL))
}

/// The current instant, by the same route `ucal now` takes.
///
/// Shared rather than reimplemented: a clock that read the system time by a
/// different path than the command would be a second implementation of *now*,
/// and the two would eventually disagree by a leap second.
pub(crate) fn now_instant() -> Result<Instant<UC1>, TimeError> {
    crate::now_instant()
}
